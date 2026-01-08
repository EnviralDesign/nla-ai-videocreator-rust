//! Project data model
//!
//! This module contains the core data structures for a video project:
//! - Project: The top-level container
//! - Track: Timeline tracks
//! - Clip: Media clips on tracks
//! - Marker: Point-in-time annotations

use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::state::{Asset, AssetKind, GenerativeConfig};

// =============================================================================
// Project Settings
// =============================================================================

/// Project-level settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSettings {
    /// Video width in pixels
    pub width: u32,
    /// Video height in pixels
    pub height: u32,
    /// Frame rate (frames per second)
    pub fps: f64,
    /// Project timeline duration in seconds
    #[serde(default = "default_project_duration_seconds")]
    pub duration_seconds: f64,
}

fn default_project_duration_seconds() -> f64 {
    60.0
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            fps: 60.0,
            duration_seconds: default_project_duration_seconds(),
        }
    }
}

// =============================================================================
// Track Types
// =============================================================================

/// The type of track
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrackType {
    /// Video track - holds video clips, image clips, generative visual content
    Video,
    /// Audio track - holds audio clips, generative audio content
    Audio,
    /// Marker track - holds point-in-time markers (singular, not duplicatable)
    Marker,
}

/// A track in the timeline
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Track {
    /// Unique identifier
    pub id: Uuid,
    /// Display name (e.g., "Video 1", "Audio 1", "Markers")
    pub name: String,
    /// Type of track
    pub track_type: TrackType,
}

impl Track {
    /// Create a new track
    pub fn new(name: impl Into<String>, track_type: TrackType) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            track_type,
        }
    }

    /// Create the default video track
    pub fn default_video() -> Self {
        Self::new("Video 1", TrackType::Video)
    }

    /// Create the default audio track
    pub fn default_audio() -> Self {
        Self::new("Audio 1", TrackType::Audio)
    }

    /// Create the markers track
    pub fn markers() -> Self {
        Self::new("Markers", TrackType::Marker)
    }
}

// =============================================================================
// Clips
// =============================================================================

/// Transform controls for a visual clip.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ClipTransform {
    /// Horizontal translation in project pixels.
    pub position_x: f32,
    /// Vertical translation in project pixels.
    pub position_y: f32,
    /// Horizontal scale factor.
    pub scale_x: f32,
    /// Vertical scale factor.
    pub scale_y: f32,
    /// Rotation in degrees.
    pub rotation_deg: f32,
    /// Opacity from 0.0 (transparent) to 1.0 (opaque).
    pub opacity: f32,
}

impl Default for ClipTransform {
    fn default() -> Self {
        Self {
            position_x: 0.0,
            position_y: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            rotation_deg: 0.0,
            opacity: 1.0,
        }
    }
}

/// A clip placed on a track
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Clip {
    /// Unique identifier
    pub id: Uuid,
    /// Reference to the asset this clip uses
    pub asset_id: Uuid,
    /// The track this clip is on
    pub track_id: Uuid,
    /// Start time in seconds
    pub start_time: f64,
    /// Duration in seconds
    pub duration: f64,
    /// Trim-in time in seconds (offset into source media)
    #[serde(default)]
    pub trim_in_seconds: f64,
    /// Transform applied when compositing this clip.
    #[serde(default)]
    pub transform: ClipTransform,
}

impl Clip {
    /// Create a new clip
    #[allow(dead_code)]
    pub fn new(asset_id: Uuid, track_id: Uuid, start_time: f64, duration: f64) -> Self {
        Self {
            id: Uuid::new_v4(),
            asset_id,
            track_id,
            start_time,
            duration,
            trim_in_seconds: 0.0,
            transform: ClipTransform::default(),
        }
    }

    /// Get the end time of this clip
    pub fn end_time(&self) -> f64 {
        self.start_time + self.duration
    }

    /// Check if this clip overlaps with a time range
    #[allow(dead_code)]
    pub fn overlaps(&self, start: f64, end: f64) -> bool {
        self.start_time < end && self.end_time() > start
    }
}

// =============================================================================
// Markers
// =============================================================================

/// A marker (point-in-time annotation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Marker {
    /// Unique identifier
    pub id: Uuid,
    /// Time position in seconds
    pub time: f64,
    /// Optional label
    pub label: Option<String>,
    /// Optional color (hex string, e.g., "#f97316")
    pub color: Option<String>,
}

impl Marker {
    /// Create a new marker at the given time
    #[allow(dead_code)]
    pub fn new(time: f64) -> Self {
        Self {
            id: Uuid::new_v4(),
            time,
            label: None,
            color: None,
        }
    }

    /// Create a marker with a label
    #[allow(dead_code)]
    pub fn with_label(time: f64, label: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            time,
            label: Some(label.into()),
            color: None,
        }
    }
}

// =============================================================================
// Project
// =============================================================================

/// The main project container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    /// Schema version for future compatibility
    pub version: String,
    /// Project name
    pub name: String,
    /// Project settings (resolution, fps, etc.)
    pub settings: ProjectSettings,
    /// All tracks in the project (ordered top to bottom)
    pub tracks: Vec<Track>,
    /// All assets in the project
    pub assets: Vec<Asset>,
    /// All clips placed on tracks
    pub clips: Vec<Clip>,
    /// All markers
    pub markers: Vec<Marker>,
    
    /// Path to the project folder (not serialized - set on load)
    #[serde(skip)]
    pub project_path: Option<PathBuf>,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            name: "Untitled Project".to_string(),
            settings: ProjectSettings::default(),
            tracks: vec![
                Track::default_video(),
                Track::default_audio(),
                Track::markers(),
            ],
            assets: Vec::new(),
            clips: Vec::new(),
            markers: Vec::new(),
            project_path: None,
        }
    }
}

#[allow(dead_code)]
impl Project {
    /// Create a new project with default settings
    #[allow(dead_code)]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Get the project duration (end of last clip or marker)
    pub fn duration(&self) -> f64 {
        let clip_end = self.clips.iter().map(|c| c.end_time()).fold(0.0, f64::max);
        let marker_end = self.markers.iter().map(|m| m.time).fold(0.0, f64::max);
        let configured = self.settings.duration_seconds.max(0.0);
        clip_end.max(marker_end).max(configured)
    }

    /// Find a track by ID
    pub fn find_track(&self, id: Uuid) -> Option<&Track> {
        self.tracks.iter().find(|t| t.id == id)
    }

    /// Find an asset by ID
    pub fn find_asset(&self, id: Uuid) -> Option<&Asset> {
        self.assets.iter().find(|a| a.id == id)
    }

    /// Set the cached duration (in seconds) for an asset
    pub fn set_asset_duration(&mut self, id: Uuid, duration_seconds: Option<f64>) -> bool {
        if let Some(asset) = self.assets.iter_mut().find(|a| a.id == id) {
            asset.set_duration_seconds(duration_seconds);
            return true;
        }
        false
    }

    /// Get the cached duration (in seconds) for an asset
    pub fn asset_duration_seconds(&self, id: Uuid) -> Option<f64> {
        self.find_asset(id).and_then(|asset| asset.duration_seconds)
    }

    /// Get a clip duration for an asset, falling back to a default value
    pub fn asset_clip_duration(&self, id: Uuid, default_duration: f64) -> f64 {
        self.asset_duration_seconds(id).unwrap_or(default_duration)
    }

    /// Get all clips on a specific track
    pub fn clips_on_track(&self, track_id: Uuid) -> Vec<&Clip> {
        self.clips.iter().filter(|c| c.track_id == track_id).collect()
    }

    /// Get all clips that overlap a time range
    pub fn clips_in_range(&self, start: f64, end: f64) -> Vec<&Clip> {
        self.clips.iter().filter(|c| c.overlaps(start, end)).collect()
    }

    /// Get assets that have clips overlapping a time range
    pub fn assets_in_range(&self, start: f64, end: f64) -> Vec<&Asset> {
        let clip_asset_ids: Vec<Uuid> = self
            .clips_in_range(start, end)
            .iter()
            .map(|c| c.asset_id)
            .collect();
        
        self.assets
            .iter()
            .filter(|a| clip_asset_ids.contains(&a.id))
            .collect()
    }

    /// Add a new video track
    pub fn add_video_track(&mut self) -> Uuid {
        let count = self.tracks.iter().filter(|t| t.track_type == TrackType::Video).count();
        let track = Track::new(format!("Video {}", count + 1), TrackType::Video);
        let id = track.id;
        self.tracks.push(track);
        id
    }

    /// Add a new audio track
    pub fn add_audio_track(&mut self) -> Uuid {
        let count = self.tracks.iter().filter(|t| t.track_type == TrackType::Audio).count();
        let track = Track::new(format!("Audio {}", count + 1), TrackType::Audio);
        let id = track.id;
        self.tracks.push(track);
        id
    }

    /// Remove a track by ID (cannot remove the Markers track)
    pub fn remove_track(&mut self, id: Uuid) -> bool {
        // Find the track and check if it's the Markers track
        if let Some(track) = self.tracks.iter().find(|t| t.id == id) {
            if track.track_type == TrackType::Marker {
                return false; // Cannot remove the Markers track
            }
        }
        
        // Remove any clips on this track
        self.clips.retain(|c| c.track_id != id);
        
        // Remove the track
        let len = self.tracks.len();
        self.tracks.retain(|t| t.id != id);
        self.tracks.len() < len
    }

    /// Add an asset to the project
    pub fn add_asset(&mut self, asset: Asset) -> Uuid {
        let id = asset.id;
        self.assets.push(asset);
        id
    }

    /// Import a file into the project
    /// Copies the file to the appropriate project subdirectory and returns a new Asset ID
    pub fn import_file(&mut self, source_path: &Path) -> io::Result<Uuid> {
        let project_root = self.project_path.as_ref().ok_or_else(|| {
            io::Error::new(io::ErrorKind::Other, "Project must be saved before importing files")
        })?;

        // 1. Determine asset type and target subfolder
        let ext = source_path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        let (subfolder, is_video, is_audio, _is_image) = match ext.as_str() {
            "mp4" | "mov" | "avi" | "mkv" | "webm" => ("video", true, false, false),
            "mp3" | "wav" | "ogg" | "flac" => ("audio", false, true, false),
            "png" | "jpg" | "jpeg" | "gif" | "webp" => ("images", false, false, true),
            _ => return Err(io::Error::new(io::ErrorKind::InvalidInput, "Unsupported file type")),
        };

        // 2. Determine target filename with collision handling
        let file_stem = source_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("import");
        
        let target_dir = project_root.join(subfolder);
        // Ensure directory exists (it should, but safety first)
        if !target_dir.exists() {
            fs::create_dir_all(&target_dir)?;
        }

        let mut target_filename = format!("{}.{}", file_stem, ext);
        let mut target_path = target_dir.join(&target_filename);
        let mut counter = 1;

        while target_path.exists() {
            target_filename = format!("{}_{}.{}", file_stem, counter, ext);
            target_path = target_dir.join(&target_filename);
            counter += 1;
        }

        // 3. Copy the file
        fs::copy(source_path, &target_path)?;

        // 4. Create Asset with relative path
        let relative_path = PathBuf::from(subfolder).join(&target_filename);
        let name = file_stem.to_string(); // Use original filename as display name

        let asset = if is_video {
            Asset::new_video(name, relative_path)
        } else if is_audio {
            Asset::new_audio(name, relative_path)
        } else {
            Asset::new_image(name, relative_path)
        };

        Ok(self.add_asset(asset))
    }

    /// Remove an asset by ID (also removes any clips using this asset)
    pub fn remove_asset(&mut self, id: Uuid) -> bool {
        // Remove any clips that reference this asset
        self.clips.retain(|c| c.asset_id != id);
        
        // Remove the asset
        let len = self.assets.len();
        self.assets.retain(|a| a.id != id);
        self.assets.len() < len
    }

    /// Add a clip to the project
    pub fn add_clip(&mut self, clip: Clip) -> Uuid {
        let id = clip.id;
        self.clips.push(clip);
        id
    }

    /// Create and add a clip from an asset at the specified time
    /// Places on first compatible track (Video track for video/image, Audio for audio)
    pub fn add_clip_from_asset(&mut self, asset_id: Uuid, start_time: f64, duration: f64) -> Option<Uuid> {
        // Find the asset to determine what track type to use
        let asset = self.assets.iter().find(|a| a.id == asset_id)?;
        
        let target_track_type = if asset.is_video() || asset.is_image() {
            TrackType::Video
        } else if asset.is_audio() {
            TrackType::Audio
        } else {
            return None; // Can't place this asset type
        };
        
        // Find first matching track
        let track = self.tracks.iter().find(|t| t.track_type == target_track_type)?;
        let track_id = track.id;
        
        // Create the clip
        let clip = Clip::new(asset_id, track_id, start_time, duration);
        Some(self.add_clip(clip))
    }

    /// Add a marker to the project
    pub fn add_marker(&mut self, marker: Marker) -> Uuid {
        let id = marker.id;
        self.markers.push(marker);
        // Keep markers sorted by time
        self.markers.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
        id
    }

    /// Remove a clip by ID
    pub fn remove_clip(&mut self, id: Uuid) -> bool {
        let len = self.clips.len();
        self.clips.retain(|c| c.id != id);
        self.clips.len() < len
    }

    /// Move a clip to a new start time
    pub fn move_clip(&mut self, id: Uuid, new_start_time: f64) -> bool {
        if let Some(clip) = self.clips.iter_mut().find(|c| c.id == id) {
            clip.start_time = new_start_time.max(0.0);
            return true;
        }
        false
    }

    /// Resize a clip (change start and/or duration)
    pub fn resize_clip(&mut self, id: Uuid, new_start: f64, new_duration: f64) -> bool {
        if let Some(clip) = self.clips.iter_mut().find(|c| c.id == id) {
            let old_start = clip.start_time;
            let start_time = new_start.max(0.0);
            let mut duration = new_duration.max(0.1);  // Minimum 0.1 second

            let asset = self.assets.iter().find(|a| a.id == clip.asset_id);
            let max_duration = asset.and_then(|a| a.duration_seconds).filter(|d| *d > 0.0);

            if let Some(max_duration) = max_duration {
                duration = duration.min(max_duration);
            }

            if let Some(asset) = asset {
                if (asset.is_video() || asset.is_audio()) && (start_time - old_start).abs() > f64::EPSILON {
                    let delta = start_time - old_start;
                    clip.trim_in_seconds = (clip.trim_in_seconds + delta).max(0.0);

                    if let Some(max_duration) = max_duration {
                        let max_trim_in = (max_duration - duration).max(0.0);
                        if clip.trim_in_seconds > max_trim_in {
                            clip.trim_in_seconds = max_trim_in;
                        }
                    }
                }
            }

            clip.start_time = start_time;
            clip.duration = duration;
            return true;
        }
        false
    }

    /// Update the transform for a clip.
    pub fn set_clip_transform(&mut self, id: Uuid, transform: ClipTransform) -> bool {
        if let Some(clip) = self.clips.iter_mut().find(|c| c.id == id) {
            clip.transform = transform;
            return true;
        }
        false
    }

    /// Move a clip to the nearest compatible track above or below.
    pub fn move_clip_to_adjacent_track(&mut self, id: Uuid, direction: i32) -> bool {
        if direction == 0 {
            return false;
        }

        let clip_index = match self.clips.iter().position(|clip| clip.id == id) {
            Some(index) => index,
            None => return false,
        };

        let asset_id = self.clips[clip_index].asset_id;
        let asset = match self.find_asset(asset_id) {
            Some(asset) => asset,
            None => return false,
        };

        let target_track_type = if asset.is_visual() {
            TrackType::Video
        } else if asset.is_audio() {
            TrackType::Audio
        } else {
            return false;
        };

        let current_track_id = self.clips[clip_index].track_id;
        let current_track_index = match self.tracks.iter().position(|track| track.id == current_track_id) {
            Some(index) => index,
            None => return false,
        };

        let mut index = current_track_index as i32 + direction.signum();
        while index >= 0 && (index as usize) < self.tracks.len() {
            let track = &self.tracks[index as usize];
            if track.track_type == target_track_type {
                self.clips[clip_index].track_id = track.id;
                return true;
            }
            index += direction.signum();
        }

        false
    }

    /// Remove a marker by ID
    pub fn remove_marker(&mut self, id: Uuid) -> bool {
        let len = self.markers.len();
        self.markers.retain(|m| m.id != id);
        self.markers.len() < len
    }

    /// Move a track up in the list (visually higher)
    pub fn move_track_up(&mut self, id: Uuid) -> bool {
        if let Some(index) = self.tracks.iter().position(|t| t.id == id) {
            if index > 0 {
                self.tracks.swap(index, index - 1);
                return true;
            }
        }
        false
    }

    /// Move a track down in the list (visually lower)
    pub fn move_track_down(&mut self, id: Uuid) -> bool {
        if let Some(index) = self.tracks.iter().position(|t| t.id == id) {
            if index < self.tracks.len() - 1 {
                self.tracks.swap(index, index + 1);
                return true;
            }
        }
        false
    }

    // =========================================================================
    // Save/Load
    // =========================================================================

    /// Save the project to its folder
    #[allow(dead_code)]
    pub fn save(&self) -> io::Result<()> {
        let path = self.project_path.as_ref().ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "Project path not set")
        })?;
        self.save_to(path)
    }

    /// Save the project to a specific folder
    pub fn save_to(&self, folder: &Path) -> io::Result<()> {
        // Create the project folder structure
        fs::create_dir_all(folder)?;
        fs::create_dir_all(folder.join("audio"))?;
        fs::create_dir_all(folder.join("images"))?;
        fs::create_dir_all(folder.join("video"))?;
        fs::create_dir_all(folder.join("generated"))?;
        fs::create_dir_all(folder.join("generated/video"))?;
        fs::create_dir_all(folder.join("generated/images"))?;
        fs::create_dir_all(folder.join("generated/audio"))?;
        fs::create_dir_all(folder.join("exports"))?;

        // Write project.json
        let json = serde_json::to_string_pretty(self)?;
        fs::write(folder.join("project.json"), json)?;

        Ok(())
    }

    /// Load a project from a folder
    pub fn load(folder: &Path) -> io::Result<Self> {
        let project_file = folder.join("project.json");
        let json = fs::read_to_string(&project_file)?;
        let mut project: Project = serde_json::from_str(&json)?;
        project.project_path = Some(folder.to_path_buf());
        project.sync_generative_configs();
        Ok(project)
    }

    /// Create a new project in a folder
    #[allow(dead_code)]
    pub fn create_in(folder: &Path, name: impl Into<String>) -> io::Result<Self> {
        let mut project = Project::new(name);
        project.project_path = Some(folder.to_path_buf());
        project.save_to(folder)?;
        Ok(project)
    }

    /// Create a new project in a folder with explicit settings
    pub fn create_in_with_settings(
        folder: &Path,
        name: impl Into<String>,
        settings: ProjectSettings,
    ) -> io::Result<Self> {
        let mut project = Project::new(name);
        project.settings = settings;
        project.project_path = Some(folder.to_path_buf());
        project.save_to(folder)?;
        Ok(project)
    }

    /// Save the current project to a new folder (initializing it)
    pub fn save_project_as(&mut self, folder: &Path, name: impl Into<String>) -> io::Result<()> {
        self.name = name.into();
        self.project_path = Some(folder.to_path_buf());
        self.save_to(folder)?;
        Ok(())
    }

    fn sync_generative_configs(&mut self) {
        let Some(project_root) = self.project_path.clone() else {
            return;
        };

        for asset in self.assets.iter_mut() {
            let (folder, active_version) = match &mut asset.kind {
                AssetKind::GenerativeVideo {
                    folder,
                    active_version,
                }
                | AssetKind::GenerativeImage {
                    folder,
                    active_version,
                }
                | AssetKind::GenerativeAudio {
                    folder,
                    active_version,
                } => (folder, active_version),
                _ => continue,
            };

            let folder_path = project_root.join(folder);
            let config_path = folder_path.join("config.json");
            let mut config = GenerativeConfig::load(&folder_path).unwrap_or_default();
            let mut changed = !config_path.exists();

            if config.active_version.is_none() {
                if let Some(existing) = active_version.clone() {
                    config.active_version = Some(existing);
                    changed = true;
                }
            }

            if config.active_version != *active_version {
                *active_version = config.active_version.clone();
            }

            if changed {
                let _ = config.save(&folder_path);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_project() {
        let project = Project::default();
        assert_eq!(project.tracks.len(), 3);
        assert_eq!(project.tracks[0].track_type, TrackType::Video);
        assert_eq!(project.tracks[1].track_type, TrackType::Audio);
        assert_eq!(project.tracks[2].track_type, TrackType::Marker);
    }

    #[test]
    fn test_clip_overlap() {
        let clip = Clip::new(Uuid::new_v4(), Uuid::new_v4(), 5.0, 10.0);
        assert!(clip.overlaps(0.0, 10.0));  // Overlaps start
        assert!(clip.overlaps(10.0, 20.0)); // Overlaps end
        assert!(clip.overlaps(7.0, 12.0));  // Overlaps middle
        assert!(!clip.overlaps(0.0, 5.0));  // Just before
        assert!(!clip.overlaps(15.0, 20.0)); // Just after
    }

    #[test]
    fn test_project_serialization() {
        let project = Project::new("Test Project");
        let json = serde_json::to_string_pretty(&project).unwrap();
        let parsed: Project = serde_json::from_str(&json).unwrap();
        assert_eq!(project.name, parsed.name);
        assert_eq!(project.tracks.len(), parsed.tracks.len());
    }

    #[test]
    fn test_add_tracks() {
        let mut project = Project::default();
        let initial_count = project.tracks.len();
        
        project.add_video_track();
        assert_eq!(project.tracks.len(), initial_count + 1);
        assert_eq!(project.tracks.last().unwrap().name, "Video 2");
        
        project.add_audio_track();
        assert_eq!(project.tracks.len(), initial_count + 2);
        assert_eq!(project.tracks.last().unwrap().name, "Audio 2");
    }
}
