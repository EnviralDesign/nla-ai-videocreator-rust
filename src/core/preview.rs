use std::borrow::Cow;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use image::{Rgba, RgbaImage};
use image::imageops::{overlay, resize, FilterType};
use imageproc::geometric_transformations::{rotate_about_center, Interpolation};

use crate::core::preview_store;
use crate::core::video_decode::{DecodeMode, VideoDecodeWorker};
use crate::state::{Asset, AssetKind, ClipTransform, Project, TrackType};

const DEFAULT_MAX_PREVIEW_WIDTH: u32 = 960;
const DEFAULT_MAX_PREVIEW_HEIGHT: u32 = 540;
const FFMPEG_TIME_EPSILON: f64 = 0.001;
const MAX_CACHE_BUCKETS: usize = 120;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PreviewStats {
    pub total_ms: f64,
    pub collect_ms: f64,
    pub composite_ms: f64,
    pub encode_ms: f64,
    pub video_decode_ms: f64,
    pub video_decode_seek_ms: f64,
    pub video_decode_packet_ms: f64,
    pub video_decode_transfer_ms: f64,
    pub video_decode_scale_ms: f64,
    pub video_decode_copy_ms: f64,
    pub still_load_ms: f64,
    pub hw_decode_frames: usize,
    pub sw_decode_frames: usize,
    pub layers: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PreviewDecodeMode {
    Seek,
    Sequential,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PreviewFrameInfo {
    pub version: u64,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct PreviewLayerPlacement {
    pub offset_x: f32,
    pub offset_y: f32,
    pub scaled_w: f32,
    pub scaled_h: f32,
    pub opacity: f32,
    pub rotation_deg: f32,
}

#[derive(Clone, Debug)]
pub struct PreviewLayerGpu {
    pub image: Arc<RgbaImage>,
    pub placement: PreviewLayerPlacement,
}

#[derive(Clone, Debug)]
pub struct PreviewLayerStack {
    pub canvas_width: u32,
    pub canvas_height: u32,
    pub layers: Vec<PreviewLayerGpu>,
}

#[derive(Clone, Debug)]
pub struct RenderOutput {
    pub frame: Option<PreviewFrameInfo>,
    pub layers: Option<PreviewLayerStack>,
    pub stats: PreviewStats,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct FrameKey {
    path: PathBuf,
    frame_index: i64,
}

struct CacheEntry {
    image: Arc<RgbaImage>,
    size_bytes: usize,
    last_used: u64,
}

struct FrameCache {
    max_bytes: usize,
    total_bytes: usize,
    access_counter: u64,
    entries: HashMap<FrameKey, CacheEntry>,
    lru_order: VecDeque<(FrameKey, u64)>,
    asset_index: HashMap<PathBuf, HashSet<i64>>,
}

impl FrameCache {
    fn new(max_bytes: usize) -> Self {
        Self {
            max_bytes,
            total_bytes: 0,
            access_counter: 0,
            entries: HashMap::new(),
            lru_order: VecDeque::new(),
            asset_index: HashMap::new(),
        }
    }

    fn get(&mut self, key: &FrameKey) -> Option<Arc<RgbaImage>> {
        let entry = self.entries.get_mut(key)?;
        self.access_counter = self.access_counter.wrapping_add(1);
        entry.last_used = self.access_counter;
        self.lru_order.push_back((key.clone(), entry.last_used));
        Some(Arc::clone(&entry.image))
    }

    fn insert(&mut self, key: FrameKey, image: Arc<RgbaImage>) {
        let size_bytes = image_size_bytes(&image);
        if size_bytes == 0 || self.max_bytes == 0 || size_bytes > self.max_bytes {
            return;
        }

        if let Some(existing) = self.entries.remove(&key) {
            self.total_bytes = self.total_bytes.saturating_sub(existing.size_bytes);
        }

        self.access_counter = self.access_counter.wrapping_add(1);
        let last_used = self.access_counter;
        self.asset_index
            .entry(key.path.clone())
            .or_default()
            .insert(key.frame_index);
        self.entries.insert(
            key.clone(),
            CacheEntry {
                image,
                size_bytes,
                last_used,
            },
        );
        self.total_bytes = self.total_bytes.saturating_add(size_bytes);
        self.lru_order.push_back((key, last_used));
        self.evict_if_needed();
    }

    fn evict_if_needed(&mut self) {
        while self.total_bytes > self.max_bytes {
            let Some((key, stamp)) = self.lru_order.pop_front() else {
                break;
            };
            let Some(entry) = self.entries.get(&key) else {
                continue;
            };
            if entry.last_used != stamp {
                continue;
            }
            self.total_bytes = self.total_bytes.saturating_sub(entry.size_bytes);
            self.entries.remove(&key);
            if let Some(frames) = self.asset_index.get_mut(&key.path) {
                frames.remove(&key.frame_index);
                if frames.is_empty() {
                    self.asset_index.remove(&key.path);
                }
            }
        }
    }
}

/// Generates composited preview frames for the current timeline time.
pub struct PreviewRenderer {
    project_root: PathBuf,
    max_width: u32,
    max_height: u32,
    video_decoder: VideoDecodeWorker,
    frame_cache: Mutex<FrameCache>,
}

impl PreviewRenderer {
    /// Create a new preview renderer rooted at the project's folder.
    pub fn new(project_root: PathBuf, max_cache_bytes: usize) -> Self {
        Self {
            project_root,
            max_width: DEFAULT_MAX_PREVIEW_WIDTH,
            max_height: DEFAULT_MAX_PREVIEW_HEIGHT,
            video_decoder: VideoDecodeWorker::new(
                DEFAULT_MAX_PREVIEW_WIDTH,
                DEFAULT_MAX_PREVIEW_HEIGHT,
            ),
            frame_cache: Mutex::new(FrameCache::new(max_cache_bytes)),
        }
    }

    /// Render a preview frame for the given time and store the encoded PNG in memory.
    pub fn render_frame(
        &self,
        project: &Project,
        time_seconds: f64,
        decode_mode: PreviewDecodeMode,
        allow_hw_decode: bool,
    ) -> RenderOutput {
        let render_start = Instant::now();
        let mut stats = PreviewStats::default();
        let project_root = project
            .project_path
            .as_ref()
            .unwrap_or(&self.project_root);

        let (canvas_w, canvas_h, preview_scale) = preview_canvas_size(
            project.settings.width,
            project.settings.height,
            self.max_width,
            self.max_height,
        );

        let fps = project.settings.fps.max(1.0);
        let collect_start = Instant::now();
        let layers = self.collect_layers(
            project,
            project_root,
            time_seconds,
            fps,
            decode_mode,
            allow_hw_decode,
            &mut stats,
        );
        stats.collect_ms = elapsed_ms(collect_start);
        stats.layers = layers.len();

        let has_visual_assets = project.clips.iter().any(|clip| {
            project
                .find_asset(clip.asset_id)
                .map(|asset| asset.is_visual())
                .unwrap_or(false)
        });

        if layers.is_empty() && !has_visual_assets {
            stats.total_ms = elapsed_ms(render_start);
            return RenderOutput {
                frame: None,
                layers: None,
                stats,
            };
        }

        let mut canvas = RgbaImage::from_pixel(canvas_w, canvas_h, Rgba([0, 0, 0, 255]));

        let composite_start = Instant::now();
        for layer in layers {
            composite_layer(
                &mut canvas,
                &layer.image,
                layer.transform,
                preview_scale,
            );
        }
        stats.composite_ms = elapsed_ms(composite_start);

        let encode_start = Instant::now();
        let bytes = canvas.into_raw();
        let saved = preview_store::store_preview_frame(canvas_w, canvas_h, bytes);
        stats.encode_ms = elapsed_ms(encode_start);
        stats.total_ms = elapsed_ms(render_start);

        let frame = saved.map(|version| PreviewFrameInfo {
            version,
            width: canvas_w,
            height: canvas_h,
        });
        RenderOutput {
            frame,
            layers: None,
            stats,
        }
    }

    /// Render the per-layer stack for GPU compositing.
    pub fn render_layers(
        &self,
        project: &Project,
        time_seconds: f64,
        decode_mode: PreviewDecodeMode,
        allow_hw_decode: bool,
    ) -> RenderOutput {
        let render_start = Instant::now();
        let mut stats = PreviewStats::default();
        let project_root = project
            .project_path
            .as_ref()
            .unwrap_or(&self.project_root);

        let (canvas_w, canvas_h, preview_scale) = preview_canvas_size(
            project.settings.width,
            project.settings.height,
            self.max_width,
            self.max_height,
        );

        let fps = project.settings.fps.max(1.0);
        let collect_start = Instant::now();
        let layers = self.collect_layers(
            project,
            project_root,
            time_seconds,
            fps,
            decode_mode,
            allow_hw_decode,
            &mut stats,
        );
        stats.collect_ms = elapsed_ms(collect_start);
        stats.layers = layers.len();

        let has_visual_assets = project.clips.iter().any(|clip| {
            project
                .find_asset(clip.asset_id)
                .map(|asset| asset.is_visual())
                .unwrap_or(false)
        });

        if layers.is_empty() && !has_visual_assets {
            stats.total_ms = elapsed_ms(render_start);
            return RenderOutput {
                frame: None,
                layers: None,
                stats,
            };
        }

        let mut gpu_layers = Vec::new();
        let canvas_w_f = canvas_w as f32;
        let canvas_h_f = canvas_h as f32;
        for layer in layers {
            if let Some(placement) = compute_layer_placement(
                &layer.image,
                layer.transform,
                preview_scale,
                canvas_w_f,
                canvas_h_f,
            ) {
                gpu_layers.push(PreviewLayerGpu {
                    image: layer.image,
                    placement,
                });
            }
        }

        stats.total_ms = elapsed_ms(render_start);
        RenderOutput {
            frame: None,
            layers: Some(PreviewLayerStack {
                canvas_width: canvas_w,
                canvas_height: canvas_h,
                layers: gpu_layers,
            }),
            stats,
        }
    }

    fn collect_layers(
        &self,
        project: &Project,
        project_root: &Path,
        time_seconds: f64,
        fps: f64,
        decode_mode: PreviewDecodeMode,
        allow_hw_decode: bool,
        stats: &mut PreviewStats,
    ) -> Vec<PreviewLayer> {
        let mut track_order: HashMap<uuid::Uuid, usize> = HashMap::new();
        let mut video_tracks = 0;
        for track in project.tracks.iter() {
            if track.track_type == TrackType::Video {
                track_order.insert(track.id, video_tracks);
                video_tracks += 1;
            }
        }

        let decode_mode = match decode_mode {
            PreviewDecodeMode::Seek => DecodeMode::Seek,
            PreviewDecodeMode::Sequential => DecodeMode::Sequential,
        };

        let mut layers = Vec::new();
        let mut pending = Vec::new();
        for clip in project.clips.iter() {
            let track_index = match track_order.get(&clip.track_id) {
                Some(index) => *index,
                None => continue,
            };

            if time_seconds < clip.start_time || time_seconds >= clip.end_time() {
                continue;
            }

            let asset = match project.find_asset(clip.asset_id) {
                Some(asset) if asset.is_visual() => asset,
                _ => continue,
            };

            let source_time = (time_seconds - clip.start_time + clip.trim_in_seconds).max(0.0);
            let Some((path, is_video, duration)) = resolve_asset_source(
                project_root,
                asset,
                &["png", "jpg", "jpeg", "webp"],
                &["mp4", "mov", "mkv", "webm"],
            ) else {
                continue;
            };

            let (frame_index, frame_time) = if is_video {
                let time = clamp_time(source_time, duration);
                let index = time_to_frame_index(time, fps);
                let frame_time = frame_index_to_time(index, fps);
                (index, frame_time)
            } else {
                (0, 0.0)
            };

            let cache_key = FrameKey {
                path: path.clone(),
                frame_index,
            };

            if let Ok(mut cache) = self.frame_cache.lock() {
                if let Some(image) = cache.get(&cache_key) {
                    stats.cache_hits += 1;
                    layers.push(PreviewLayer {
                        track_index,
                        start_time: clip.start_time,
                        image,
                        transform: clip.transform,
                    });
                    continue;
                }
            }

            stats.cache_misses += 1;

            if !is_video {
                let decode_start = Instant::now();
                let image = self.load_still(&path);
                let decode_ms = elapsed_ms(decode_start);
                stats.still_load_ms += decode_ms;
                if let Some(image) = image {
                    let image = Arc::new(image);
                    if let Ok(mut cache) = self.frame_cache.lock() {
                        cache.insert(cache_key, Arc::clone(&image));
                    }
                    layers.push(PreviewLayer {
                        track_index,
                        start_time: clip.start_time,
                        image,
                        transform: clip.transform,
                    });
                }
                continue;
            }

            pending.push(PendingDecode {
                track_index,
                start_time: clip.start_time,
                path,
                frame_time,
                cache_key,
                transform: clip.transform,
                lane_id: track_lane_id(clip.track_id),
            });
        }

        if !pending.is_empty() {
            let mut requests = Vec::with_capacity(pending.len());
            for item in pending {
                if let Some(receiver) = self.video_decoder.decode_async(
                    &item.path,
                    item.frame_time,
                    decode_mode,
                    item.lane_id,
                    allow_hw_decode,
                ) {
                    requests.push((item, receiver));
                }
            }

            for (item, receiver) in requests {
                if let Ok(response) = receiver.recv() {
                    let timings = response.timings;
                    stats.video_decode_ms += timings.total_ms();
                    stats.video_decode_seek_ms += timings.seek_ms;
                    stats.video_decode_packet_ms += timings.packet_ms;
                    stats.video_decode_transfer_ms += timings.transfer_ms;
                    stats.video_decode_scale_ms += timings.scale_ms;
                    stats.video_decode_copy_ms += timings.copy_ms;
                    if let Some(image) = response.image {
                        let image = Arc::new(image);
                        if let Ok(mut cache) = self.frame_cache.lock() {
                            cache.insert(item.cache_key, Arc::clone(&image));
                        }
                        if response.used_hw {
                            stats.hw_decode_frames += 1;
                        } else {
                            stats.sw_decode_frames += 1;
                        }
                        layers.push(PreviewLayer {
                            track_index: item.track_index,
                            start_time: item.start_time,
                            image,
                            transform: item.transform,
                        });
                    }
                }
            }
        }

        layers.sort_by(|a, b| {
            b.track_index
                .cmp(&a.track_index)
                .then_with(|| a.start_time.partial_cmp(&b.start_time).unwrap_or(std::cmp::Ordering::Equal))
        });

        layers
    }

    pub fn prefetch_frames(
        &self,
        project: &Project,
        time_seconds: f64,
        direction: i32,
        window_frames: u32,
        decode_mode: PreviewDecodeMode,
        allow_hw_decode: bool,
    ) {
        if window_frames == 0 || direction == 0 {
            return;
        }

        let fps = project.settings.fps.max(1.0);
        let project_root = project
            .project_path
            .as_ref()
            .unwrap_or(&self.project_root);
        let start_frame = time_to_frame_index(time_seconds, fps);
        let step = direction.signum() as i64;

        for offset in 1..=window_frames {
            let frame_index = start_frame + step * offset as i64;
            if frame_index < 0 {
                break;
            }
            let frame_time = frame_index_to_time(frame_index, fps);
            for clip in project.clips.iter() {
                if frame_time < clip.start_time || frame_time >= clip.end_time() {
                    continue;
                }

                let asset = match project.find_asset(clip.asset_id) {
                    Some(asset) if asset.is_visual() => asset,
                    _ => continue,
                };

                let source_time = (frame_time - clip.start_time + clip.trim_in_seconds).max(0.0);
                let _ = self.load_clip_frame(
                    project_root,
                    asset,
                    source_time,
                    fps,
                    decode_mode,
                    track_lane_id(clip.track_id),
                    allow_hw_decode,
                    None,
                );
            }
        }
    }

    pub fn cached_buckets_for_project(
        &self,
        project: &Project,
        bucket_hint_seconds: f64,
    ) -> HashMap<uuid::Uuid, Vec<bool>> {
        let project_root = project
            .project_path
            .as_ref()
            .unwrap_or(&self.project_root);
        let fps = project.settings.fps.max(1.0);
        let min_bucket_seconds = (1.0 / fps).max(0.001);
        let bucket_hint_seconds = bucket_hint_seconds.max(min_bucket_seconds);
        let mut result = HashMap::new();

        let Ok(cache) = self.frame_cache.lock() else {
            return result;
        };

        for clip in project.clips.iter() {
            let Some(asset) = project.find_asset(clip.asset_id) else {
                continue;
            };
            if !asset.is_visual() {
                continue;
            }

            let Some((path, is_video, _duration)) = resolve_asset_source(
                project_root,
                asset,
                &["png", "jpg", "jpeg", "webp"],
                &["mp4", "mov", "mkv", "webm"],
            ) else {
                continue;
            };

            let clip_duration = clip.duration.max(0.0);
            if clip_duration <= 0.0 {
                continue;
            }

            let mut bucket_seconds = bucket_hint_seconds.max(clip_duration / MAX_CACHE_BUCKETS as f64);
            bucket_seconds = bucket_seconds.max(min_bucket_seconds);
            let bucket_count = (clip_duration / bucket_seconds).ceil().max(1.0) as usize;

            let mut buckets = vec![false; bucket_count];
            let Some(asset_frames) = cache.asset_index.get(&path) else {
                result.insert(clip.id, buckets);
                continue;
            };

            if !is_video {
                let cached = asset_frames.contains(&0);
                for bucket in buckets.iter_mut() {
                    *bucket = cached;
                }
                result.insert(clip.id, buckets);
                continue;
            }

            let clip_start = clip.trim_in_seconds.max(0.0);
            let clip_end = clip_start + clip_duration;
            for frame_index in asset_frames.iter() {
                let frame_time = frame_index_to_time(*frame_index, fps);
                if frame_time < clip_start || frame_time > clip_end {
                    continue;
                }
                let time_in_clip = (frame_time - clip_start).max(0.0);
                let bucket_index = (time_in_clip / bucket_seconds).floor() as usize;
                if let Some(bucket) = buckets.get_mut(bucket_index) {
                    *bucket = true;
                }
            }

            result.insert(clip.id, buckets);
        }

        result
    }

    fn load_clip_frame(
        &self,
        project_root: &Path,
        asset: &Asset,
        time_seconds: f64,
        fps: f64,
        decode_mode: PreviewDecodeMode,
        lane_id: u64,
        allow_hw_decode: bool,
        mut stats: Option<&mut PreviewStats>,
    ) -> Option<Arc<RgbaImage>> {
        let (path, is_video, duration) =
            resolve_asset_source(project_root, asset, &["png", "jpg", "jpeg", "webp"], &["mp4", "mov", "mkv", "webm"])?;

        let (frame_index, frame_time) = if is_video {
            let time = clamp_time(time_seconds, duration);
            let index = time_to_frame_index(time, fps);
            let frame_time = frame_index_to_time(index, fps);
            (index, frame_time)
        } else {
            (0, 0.0)
        };

        let cache_key = FrameKey {
            path: path.clone(),
            frame_index,
        };

        if let Ok(mut cache) = self.frame_cache.lock() {
            if let Some(image) = cache.get(&cache_key) {
                if let Some(stats) = stats.as_deref_mut() {
                    stats.cache_hits += 1;
                }
                return Some(image);
            }
        }

        if let Some(stats) = stats.as_deref_mut() {
            stats.cache_misses += 1;
        }

        let image = if is_video {
            let mode = match decode_mode {
                PreviewDecodeMode::Seek => DecodeMode::Seek,
                PreviewDecodeMode::Sequential => DecodeMode::Sequential,
            };
            let response = match mode {
                DecodeMode::Seek => {
                    self.video_decoder
                        .decode(&path, frame_time, lane_id, allow_hw_decode)?
                }
                DecodeMode::Sequential => {
                    self.video_decoder
                        .decode_sequential(&path, frame_time, lane_id, allow_hw_decode)?
                }
            };
            if let Some(stats) = stats.as_deref_mut() {
                let timings = response.timings;
                stats.video_decode_ms += timings.total_ms();
                stats.video_decode_seek_ms += timings.seek_ms;
                stats.video_decode_packet_ms += timings.packet_ms;
                stats.video_decode_transfer_ms += timings.transfer_ms;
                stats.video_decode_scale_ms += timings.scale_ms;
                stats.video_decode_copy_ms += timings.copy_ms;
            }
            if let Some(image) = response.image {
                if let Some(stats) = stats.as_deref_mut() {
                    if response.used_hw {
                        stats.hw_decode_frames += 1;
                    } else {
                        stats.sw_decode_frames += 1;
                    }
                }
                image
            } else {
                return None;
            }
        } else {
            let decode_start = Instant::now();
            let image = self.load_still(&path)?;
            let decode_ms = elapsed_ms(decode_start);
            if let Some(stats) = stats.as_deref_mut() {
                stats.still_load_ms += decode_ms;
            }
            image
        };

        let image = Arc::new(image);
        if let Ok(mut cache) = self.frame_cache.lock() {
            cache.insert(cache_key, Arc::clone(&image));
        }

        Some(image)
    }

    fn load_still(&self, path: &Path) -> Option<RgbaImage> {
        let image = image::open(path).ok()?.into_rgba8();
        Some(scale_image_to_fit(image, self.max_width, self.max_height))
    }

    // Video decoding handled by the in-process decoder worker.
}

struct PendingDecode {
    track_index: usize,
    start_time: f64,
    path: PathBuf,
    frame_time: f64,
    cache_key: FrameKey,
    transform: ClipTransform,
    lane_id: u64,
}

struct PreviewLayer {
    track_index: usize,
    start_time: f64,
    image: Arc<RgbaImage>,
    transform: ClipTransform,
}

fn preview_canvas_size(
    project_width: u32,
    project_height: u32,
    max_width: u32,
    max_height: u32,
) -> (u32, u32, f32) {
    if project_width == 0 || project_height == 0 {
        return (max_width, max_height, 1.0);
    }

    let scale_w = max_width as f32 / project_width as f32;
    let scale_h = max_height as f32 / project_height as f32;
    let scale = scale_w.min(scale_h).min(1.0).max(0.01);

    let canvas_w = (project_width as f32 * scale).round().max(1.0) as u32;
    let canvas_h = (project_height as f32 * scale).round().max(1.0) as u32;

    (canvas_w, canvas_h, scale)
}

fn composite_layer(
    canvas: &mut RgbaImage,
    image: &RgbaImage,
    transform: ClipTransform,
    preview_scale: f32,
) {
    let placement = match compute_layer_placement(
        image,
        transform,
        preview_scale,
        canvas.width() as f32,
        canvas.height() as f32,
    ) {
        Some(placement) => placement,
        None => return,
    };

    let image = if placement.opacity < 1.0 {
        let mut working = image.clone();
        apply_opacity(&mut working, placement.opacity);
        Cow::Owned(working)
    } else {
        Cow::Borrowed(image)
    };
    let scaled_w = placement.scaled_w.round() as u32;
    let scaled_h = placement.scaled_h.round() as u32;
    if scaled_w == 0 || scaled_h == 0 {
        return;
    }

    let resized = resize(image.as_ref(), scaled_w, scaled_h, FilterType::Triangle);
    if placement.rotation_deg.abs() <= 0.01 {
        overlay(
            canvas,
            &resized,
            placement.offset_x.round() as i64,
            placement.offset_y.round() as i64,
        );
        return;
    }

    let rotated = rotate_rgba(&resized, placement.rotation_deg);
    let center_x = placement.offset_x + placement.scaled_w * 0.5;
    let center_y = placement.offset_y + placement.scaled_h * 0.5;
    let dest_x = (center_x - rotated.width() as f32 * 0.5).round() as i64;
    let dest_y = (center_y - rotated.height() as f32 * 0.5).round() as i64;
    overlay(canvas, &rotated, dest_x, dest_y);
}

fn rotate_rgba(image: &RgbaImage, rotation_deg: f32) -> RgbaImage {
    let angle = rotation_deg.to_radians();
    let (sin, cos) = angle.sin_cos();
    let abs_sin = sin.abs();
    let abs_cos = cos.abs();
    let src_w = image.width().max(1) as f32;
    let src_h = image.height().max(1) as f32;
    let new_w = (src_w * abs_cos + src_h * abs_sin).ceil().max(1.0) as u32;
    let new_h = (src_w * abs_sin + src_h * abs_cos).ceil().max(1.0) as u32;

    let mut expanded = RgbaImage::from_pixel(new_w, new_h, Rgba([0, 0, 0, 0]));
    let offset_x = ((new_w as f32 - src_w) * 0.5).round() as i64;
    let offset_y = ((new_h as f32 - src_h) * 0.5).round() as i64;
    overlay(&mut expanded, image, offset_x, offset_y);

    rotate_about_center(
        &expanded,
        angle,
        Interpolation::Bilinear,
        Rgba([0, 0, 0, 0]),
    )
}

fn compute_layer_placement(
    image: &RgbaImage,
    transform: ClipTransform,
    preview_scale: f32,
    canvas_w: f32,
    canvas_h: f32,
) -> Option<PreviewLayerPlacement> {
    let (src_w, src_h) = (image.width() as f32, image.height() as f32);
    if src_w <= 0.0 || src_h <= 0.0 {
        return None;
    }

    let base_scale = (canvas_w / src_w).min(canvas_h / src_h);
    let scaled_w = src_w * base_scale * transform.scale_x.max(0.01);
    let scaled_h = src_h * base_scale * transform.scale_y.max(0.01);
    if scaled_w <= 0.0 || scaled_h <= 0.0 {
        return None;
    }

    let offset_x = ((canvas_w - scaled_w) * 0.5) + (transform.position_x * preview_scale);
    let offset_y = ((canvas_h - scaled_h) * 0.5) + (transform.position_y * preview_scale);
    let opacity = transform.opacity.clamp(0.0, 1.0);

    Some(PreviewLayerPlacement {
        offset_x,
        offset_y,
        scaled_w,
        scaled_h,
        opacity,
        rotation_deg: transform.rotation_deg,
    })
}

fn apply_opacity(image: &mut RgbaImage, opacity: f32) {
    for pixel in image.pixels_mut() {
        let alpha = (pixel.0[3] as f32 * opacity).round().clamp(0.0, 255.0) as u8;
        pixel.0[3] = alpha;
    }
}

fn clamp_time(time_seconds: f64, duration: Option<f64>) -> f64 {
    let mut time = time_seconds.max(0.0);
    if let Some(duration) = duration {
        let limit = (duration - FFMPEG_TIME_EPSILON).max(0.0);
        time = time.min(limit);
    }
    time
}

fn elapsed_ms(start: Instant) -> f64 {
    start.elapsed().as_secs_f64() * 1000.0
}

fn time_to_frame_index(time_seconds: f64, fps: f64) -> i64 {
    let fps = fps.max(1.0);
    (time_seconds.max(0.0) * fps).floor() as i64
}

fn frame_index_to_time(frame_index: i64, fps: f64) -> f64 {
    let fps = fps.max(1.0);
    let frame_index = frame_index.max(0) as f64;
    frame_index / fps
}

fn track_lane_id(track_id: uuid::Uuid) -> u64 {
    let raw = track_id.as_u128();
    (raw as u64) ^ ((raw >> 64) as u64)
}

fn image_size_bytes(image: &RgbaImage) -> usize {
    let width = image.width() as usize;
    let height = image.height() as usize;
    width
        .saturating_mul(height)
        .saturating_mul(4)
}

fn scale_image_to_fit(image: RgbaImage, max_width: u32, max_height: u32) -> RgbaImage {
    let max_width = max_width.max(1);
    let max_height = max_height.max(1);
    let width = image.width();
    let height = image.height();
    if width <= max_width && height <= max_height {
        return image;
    }

    let scale_w = max_width as f32 / width as f32;
    let scale_h = max_height as f32 / height as f32;
    let scale = scale_w.min(scale_h).max(0.01);
    let target_w = (width as f32 * scale).round().max(1.0) as u32;
    let target_h = (height as f32 * scale).round().max(1.0) as u32;

    resize(&image, target_w, target_h, FilterType::Triangle)
}

fn resolve_generative_path(
    project_root: &Path,
    folder: &Path,
    active_version: Option<&str>,
    extensions: &[&str],
) -> Option<PathBuf> {
    let folder_path = project_root.join(folder);

    if let Some(version) = active_version {
        for ext in extensions {
            let candidate = folder_path.join(format!("{}.{}", version, ext));
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    let entries = std::fs::read_dir(&folder_path).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|ext| ext.to_str()) {
                if extensions.iter().any(|allowed| allowed.eq_ignore_ascii_case(ext)) {
                    return Some(path);
                }
            }
        }
    }

    None
}

fn resolve_asset_source(
    project_root: &Path,
    asset: &Asset,
    image_extensions: &[&str],
    video_extensions: &[&str],
) -> Option<(PathBuf, bool, Option<f64>)> {
    match &asset.kind {
        AssetKind::Image { path } => Some((project_root.join(path), false, asset.duration_seconds)),
        AssetKind::Video { path } => Some((project_root.join(path), true, asset.duration_seconds)),
        AssetKind::GenerativeImage {
            folder,
            active_version,
        } => {
            let path = resolve_generative_path(
                project_root,
                folder,
                active_version.as_deref(),
                image_extensions,
            )?;
            Some((path, false, asset.duration_seconds))
        }
        AssetKind::GenerativeVideo {
            folder,
            active_version,
        } => {
            let path = resolve_generative_path(
                project_root,
                folder,
                active_version.as_deref(),
                video_extensions,
            )?;
            Some((path, true, asset.duration_seconds))
        }
        _ => None,
    }
}
