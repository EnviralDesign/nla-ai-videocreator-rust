use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};

use image::{DynamicImage, Rgba, RgbaImage};
use image::imageops::{overlay, resize, FilterType};

use crate::state::{Asset, AssetKind, ClipTransform, Project, TrackType};

const DEFAULT_MAX_PREVIEW_WIDTH: u32 = 960;
const DEFAULT_MAX_PREVIEW_HEIGHT: u32 = 540;
const PREVIEW_FRAME_FILENAME: &str = "frame.png";
const FFMPEG_TIME_EPSILON: f64 = 0.001;

/// Generates composited preview frames for the current timeline time.
pub struct PreviewRenderer {
    project_root: PathBuf,
    cache_root: PathBuf,
    max_width: u32,
    max_height: u32,
    still_cache: Mutex<HashMap<PathBuf, Arc<RgbaImage>>>,
}

impl PreviewRenderer {
    /// Create a new preview renderer rooted at the project's folder.
    pub fn new(project_root: PathBuf) -> Self {
        let cache_root = project_root.join(".cache").join("preview");
        let _ = std::fs::create_dir_all(&cache_root);

        Self {
            project_root,
            cache_root,
            max_width: DEFAULT_MAX_PREVIEW_WIDTH,
            max_height: DEFAULT_MAX_PREVIEW_HEIGHT,
            still_cache: Mutex::new(HashMap::new()),
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

        let layers = self.collect_layers(project, project_root, time_seconds);

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
                layer.image,
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
            if let Some(image) = self.load_clip_frame(project_root, asset, source_time) {
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

    fn load_clip_frame(
        &self,
        project_root: &Path,
        asset: &Asset,
        time_seconds: f64,
    ) -> Option<RgbaImage> {
        match &asset.kind {
            AssetKind::Image { path } => self.load_still(project_root.join(path)),
            AssetKind::Video { path } => {
                let time = clamp_time(time_seconds, asset.duration_seconds);
                self.extract_video_frame(project_root.join(path), time)
            }
            AssetKind::GenerativeImage { folder, active_version } => {
                let path = resolve_generative_path(
                    project_root,
                    folder,
                    active_version.as_deref(),
                    &["png", "jpg", "jpeg", "webp"],
                )?;
                self.load_still(path)
            }
            AssetKind::GenerativeVideo { folder, active_version } => {
                let path = resolve_generative_path(
                    project_root,
                    folder,
                    active_version.as_deref(),
                    &["mp4", "mov", "mkv", "webm"],
                )?;
                let time = clamp_time(time_seconds, asset.duration_seconds);
                self.extract_video_frame(path, time)
            }
            _ => None,
        }
    }

    fn load_still(&self, path: PathBuf) -> Option<RgbaImage> {
        if let Ok(cache) = self.still_cache.lock() {
            if let Some(image) = cache.get(&path) {
                return Some((**image).clone());
            }
        }

        let image = image::open(&path).ok()?.into_rgba8();

        if let Ok(mut cache) = self.still_cache.lock() {
            cache.insert(path, Arc::new(image.clone()));
        }

        Some(image)
    }

    fn extract_video_frame(&self, path: PathBuf, time_seconds: f64) -> Option<RgbaImage> {
        if !path.exists() {
            return None;
        }

        let time_arg = format!("{:.3}", time_seconds.max(0.0));
        let output = Command::new("ffmpeg")
            .args(["-loglevel", "error", "-hide_banner"])
            .args(["-ss", &time_arg])
            .arg("-i")
            .arg(&path)
            .args(["-frames:v", "1"])
            .args(["-f", "image2pipe"])
            .args(["-vcodec", "png"])
            .arg("pipe:1")
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        image::load_from_memory(&output.stdout).ok().map(|img| img.into_rgba8())
    }
}

struct PreviewLayer {
    track_index: usize,
    start_time: f64,
    image: RgbaImage,
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
    mut image: RgbaImage,
    transform: ClipTransform,
    preview_scale: f32,
) {
    let opacity = transform.opacity.clamp(0.0, 1.0);
    if opacity < 1.0 {
        apply_opacity(&mut image, opacity);
    }

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

    let resized = resize(&image, scaled_w, scaled_h, FilterType::Triangle);

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
