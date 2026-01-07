use std::borrow::Cow;
use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use image::{DynamicImage, Rgba, RgbaImage};
use image::imageops::{overlay, resize, FilterType};

use crate::core::video_decode::VideoDecodeWorker;
use crate::state::{Asset, AssetKind, ClipTransform, Project, TrackType};

const DEFAULT_MAX_PREVIEW_WIDTH: u32 = 960;
const DEFAULT_MAX_PREVIEW_HEIGHT: u32 = 540;
const PREVIEW_FRAME_FILENAME: &str = "frame.png";
const FFMPEG_TIME_EPSILON: f64 = 0.001;

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
}

impl FrameCache {
    fn new(max_bytes: usize) -> Self {
        Self {
            max_bytes,
            total_bytes: 0,
            access_counter: 0,
            entries: HashMap::new(),
            lru_order: VecDeque::new(),
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
        }
    }
}

/// Generates composited preview frames for the current timeline time.
pub struct PreviewRenderer {
    project_root: PathBuf,
    cache_root: PathBuf,
    max_width: u32,
    max_height: u32,
    video_decoder: VideoDecodeWorker,
    frame_cache: Mutex<FrameCache>,
}

impl PreviewRenderer {
    /// Create a new preview renderer rooted at the project's folder.
    pub fn new(project_root: PathBuf, max_cache_bytes: usize) -> Self {
        let cache_root = project_root.join(".cache").join("preview");
        let _ = std::fs::create_dir_all(&cache_root);

        Self {
            project_root,
            cache_root,
            max_width: DEFAULT_MAX_PREVIEW_WIDTH,
            max_height: DEFAULT_MAX_PREVIEW_HEIGHT,
            video_decoder: VideoDecodeWorker::new(
                DEFAULT_MAX_PREVIEW_WIDTH,
                DEFAULT_MAX_PREVIEW_HEIGHT,
            ),
            frame_cache: Mutex::new(FrameCache::new(max_cache_bytes)),
        }
    }

    /// Render a preview frame for the given time and write it to the preview cache.
    pub fn render_frame(&self, project: &Project, time_seconds: f64) -> Option<PathBuf> {
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
        let layers = self.collect_layers(project, project_root, time_seconds, fps);

        let has_visual_assets = project.clips.iter().any(|clip| {
            project
                .find_asset(clip.asset_id)
                .map(|asset| asset.is_visual())
                .unwrap_or(false)
        });

        if layers.is_empty() && !has_visual_assets {
            return None;
        }

        let mut canvas = RgbaImage::from_pixel(canvas_w, canvas_h, Rgba([0, 0, 0, 255]));

        for layer in layers {
            composite_layer(
                &mut canvas,
                &layer.image,
                layer.transform,
                preview_scale,
            );
        }

        let output_path = self.cache_root.join(PREVIEW_FRAME_FILENAME);
        let output_image = DynamicImage::ImageRgba8(canvas);
        if output_image.save(&output_path).is_err() {
            return None;
        }

        Some(output_path)
    }

    fn collect_layers(
        &self,
        project: &Project,
        project_root: &Path,
        time_seconds: f64,
        fps: f64,
    ) -> Vec<PreviewLayer> {
        let mut track_order: HashMap<uuid::Uuid, usize> = HashMap::new();
        let mut video_tracks = 0;
        for track in project.tracks.iter() {
            if track.track_type == TrackType::Video {
                track_order.insert(track.id, video_tracks);
                video_tracks += 1;
            }
        }

        let mut layers = Vec::new();
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
            if let Some(image) = self.load_clip_frame(project_root, asset, source_time, fps) {
                layers.push(PreviewLayer {
                    track_index,
                    start_time: clip.start_time,
                    image,
                    transform: clip.transform,
                });
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
                let _ = self.load_clip_frame(project_root, asset, source_time, fps);
            }
        }
    }

    fn load_clip_frame(
        &self,
        project_root: &Path,
        asset: &Asset,
        time_seconds: f64,
        fps: f64,
    ) -> Option<Arc<RgbaImage>> {
        let (path, is_video, duration) = match &asset.kind {
            AssetKind::Image { path } => (project_root.join(path), false, asset.duration_seconds),
            AssetKind::Video { path } => (project_root.join(path), true, asset.duration_seconds),
            AssetKind::GenerativeImage { folder, active_version } => {
                let path = resolve_generative_path(
                    project_root,
                    folder,
                    active_version.as_deref(),
                    &["png", "jpg", "jpeg", "webp"],
                )?;
                (path, false, asset.duration_seconds)
            }
            AssetKind::GenerativeVideo { folder, active_version } => {
                let path = resolve_generative_path(
                    project_root,
                    folder,
                    active_version.as_deref(),
                    &["mp4", "mov", "mkv", "webm"],
                )?;
                (path, true, asset.duration_seconds)
            }
            _ => return None,
        };

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
                return Some(image);
            }
        }

        let image = if is_video {
            self.video_decoder.decode(&path, frame_time)?
        } else {
            self.load_still(&path)?
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
    let opacity = transform.opacity.clamp(0.0, 1.0);
    let image = if opacity < 1.0 {
        let mut working = image.clone();
        apply_opacity(&mut working, opacity);
        Cow::Owned(working)
    } else {
        Cow::Borrowed(image)
    };

    let (canvas_w, canvas_h) = (canvas.width() as f32, canvas.height() as f32);
    let (src_w, src_h) = (image.width() as f32, image.height() as f32);

    if src_w <= 0.0 || src_h <= 0.0 {
        return;
    }

    let base_scale = (canvas_w / src_w).min(canvas_h / src_h);
    let scaled_w = (src_w * base_scale * transform.scale_x.max(0.01)).round() as u32;
    let scaled_h = (src_h * base_scale * transform.scale_y.max(0.01)).round() as u32;

    if scaled_w == 0 || scaled_h == 0 {
        return;
    }

    let resized = resize(image.as_ref(), scaled_w, scaled_h, FilterType::Triangle);

    let offset_x = ((canvas_w - scaled_w as f32) * 0.5) + (transform.position_x * preview_scale);
    let offset_y = ((canvas_h - scaled_h as f32) * 0.5) + (transform.position_y * preview_scale);

    overlay(canvas, &resized, offset_x.round() as i64, offset_y.round() as i64);
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

fn time_to_frame_index(time_seconds: f64, fps: f64) -> i64 {
    let fps = fps.max(1.0);
    (time_seconds.max(0.0) * fps).floor() as i64
}

fn frame_index_to_time(frame_index: i64, fps: f64) -> f64 {
    let fps = fps.max(1.0);
    let frame_index = frame_index.max(0) as f64;
    frame_index / fps
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
