use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use tokio::sync::Semaphore;
use crate::state::Asset;

const THUMBNAIL_INTERVAL_SECONDS: f64 = 1.0;
const THUMBNAIL_HEIGHT: u32 = 120;

/// Manages the generation of thumbnails for assets
#[derive(Debug)]
pub struct Thumbnailer {
    // Semaphore to limit the number of concurrent ffmpeg processes
    semaphore: Arc<Semaphore>,
    cache_root: PathBuf,
    project_root: PathBuf,
}

impl PartialEq for Thumbnailer {
    fn eq(&self, other: &Self) -> bool {
        self.cache_root == other.cache_root
    }
}

impl Thumbnailer {
    pub fn new(project_root: PathBuf) -> Self {
        let cache_root = project_root.join(".cache").join("thumbnails");
        // Ensure cache directory exists
        if !cache_root.exists() {
            let _ = std::fs::create_dir_all(&cache_root);
        }
        
        Self {
            // Limit to 2 concurrent thumbnail tasks to avoid choking the CPU
            semaphore: Arc::new(Semaphore::new(2)),
            cache_root,
            project_root,
        }
    }

    /// Queues a thumbnail generation task for an asset
    /// Returns the path to the thumbnail directory for this asset
    /// If force is true, existing thumbnails will be overwritten (directory cleared)
    pub async fn generate(&self, asset: &Asset, force: bool) -> Option<PathBuf> {
        // Visual assets only
        if !asset.is_visual() {
            return None;
        }

        let asset_id = asset.id.to_string();
        let output_dir = self.cache_root.join(&asset_id);
        
        // If directory exists and has files, assume populated (MVP check)
        // unless forced
        if !force && output_dir.exists() && output_dir.read_dir().map(|mut i| i.next().is_some()).unwrap_or(false) {
            return Some(output_dir);
        }

        // If forced or empty, ensure clean directory
        if output_dir.exists() {
            let _ = std::fs::remove_dir_all(&output_dir);
        }
        let _ = std::fs::create_dir_all(&output_dir);

        // Limit concurrency
        let Ok(_permit) = self.semaphore.acquire().await else {
            return None;
        };

        let source_path = match &asset.kind {
            crate::state::AssetKind::Video { path } => path,
            crate::state::AssetKind::Image { path } => path,
            // Generative handling - might need to find the "active" version
            // For now, let's assume we skip or implement later
            _ => return None,
        };
        
        // Resolve absolute path
        // If source_path is relative, it's relative to project_root
        // If it's absolute, join returns it as is
        let absolute_source_path = self.project_root.join(source_path);
        
        // Clone for move into thread
        let source = absolute_source_path.clone();
        let out = output_dir.clone();

        // Spawn blocking FFmpeg task
        let _ = tokio::task::spawn_blocking(move || {
            // Extract 1 frame per interval, keep aspect ratio
            let output_pattern = out.join("thumb_%04d.jpg");
            
            // Debug check: ensure file exists
            if !source.exists() {
                println!("Thumbnailer Warning: Source file not found: {:?}", source);
                return;
            }
            
            let status = Command::new("ffmpeg")
                .arg("-i")
                .arg(&source)
                .arg("-vf")
                .arg(format!("fps=1/{},scale=-1:{}", THUMBNAIL_INTERVAL_SECONDS, THUMBNAIL_HEIGHT))
                .arg("-q:v")
                .arg("5") // reasonable jpeg quality
                .arg(output_pattern)
                .status();
                
            match status {
                Ok(s) if s.success() => println!("Generated thumbnails for {}", asset_id),
                _ => println!("Failed to generate thumbnails for {}. Valid path? {:?} Status: {:?}", asset_id, source, status),
            }
        }).await;

        Some(output_dir)
    }
    
    /// Get the path to the thumbnail for a specific time
    /// Returns None if not generated yet
    pub fn get_thumbnail_path(&self, asset_id: uuid::Uuid, time_seconds: f64) -> Option<PathBuf> {
        let dir = self.cache_root.join(asset_id.to_string());
        if !dir.exists() {
            return None;
        }
        
        // Map time to index (fps=1/interval)
        // thumb_0001.jpg covers 0-interval
        // thumb_0002.jpg covers interval-2*interval
        let index = (time_seconds / THUMBNAIL_INTERVAL_SECONDS).floor() as u32 + 1;
        
        let path = dir.join(format!("thumb_{:04}.jpg", index));
        if path.exists() {
            Some(path)
        } else {
            // Fallback to first frame if out of bounds (or handle empty)
            let fallback = dir.join("thumb_0001.jpg");
            if fallback.exists() {
                Some(fallback)
            } else {
                None
            }
        }
    }
}
