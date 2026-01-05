//! Asset types
//!
//! Assets represent content in the project - both imported files and generative assets.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// The kind of asset - either a simple file reference or a generative asset
/// The kind of asset - either a simple file reference or a generative asset
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
#[allow(dead_code)]
pub enum AssetKind {
    /// A standard video file
    Video { 
        /// Path relative to project root
        path: PathBuf 
    },
    /// A standard image file
    Image { 
        /// Path relative to project root
        path: PathBuf 
    },
    /// A standard audio file
    Audio { 
        /// Path relative to project root
        path: PathBuf 
    },
    /// A generative video asset with version history
    GenerativeVideo {
        /// Folder path relative to project root (e.g., "generated/video/gen_001")
        folder: PathBuf,
        /// Currently active version (e.g., "v1")
        active_version: Option<String>,
    },
    /// A generative image asset with version history
    GenerativeImage {
        /// Folder path relative to project root
        folder: PathBuf,
        /// Currently active version
        active_version: Option<String>,
    },
    /// A generative audio asset with version history
    GenerativeAudio {
        /// Folder path relative to project root
        folder: PathBuf,
        /// Currently active version
        active_version: Option<String>,
    },
}

#[allow(dead_code)]
impl AssetKind {
    /// Returns true if this is a generative asset
    pub fn is_generative(&self) -> bool {
        matches!(
            self,
            AssetKind::GenerativeVideo { .. }
                | AssetKind::GenerativeImage { .. }
                | AssetKind::GenerativeAudio { .. }
        )
    }

    /// Returns true if this asset produces video/image content (placeable on video tracks)
    pub fn is_visual(&self) -> bool {
        matches!(
            self,
            AssetKind::Video { .. }
                | AssetKind::Image { .. }
                | AssetKind::GenerativeVideo { .. }
                | AssetKind::GenerativeImage { .. }
        )
    }

    /// Returns true if this asset produces audio content (placeable on audio tracks)
    pub fn is_audio(&self) -> bool {
        matches!(
            self,
            AssetKind::Audio { .. } | AssetKind::GenerativeAudio { .. }
        )
    }
}

/// An asset in the project
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Asset {
    /// Unique identifier
    pub id: Uuid,
    /// User-facing display name
    pub name: String,
    /// The type and location of this asset
    pub kind: AssetKind,
}

#[allow(dead_code)]
impl Asset {
    /// Create a new video asset from an imported file
    pub fn new_video(name: impl Into<String>, path: PathBuf) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            kind: AssetKind::Video { path },
        }
    }

    /// Create a new image asset from an imported file
    pub fn new_image(name: impl Into<String>, path: PathBuf) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            kind: AssetKind::Image { path },
        }
    }

    /// Create a new audio asset from an imported file
    pub fn new_audio(name: impl Into<String>, path: PathBuf) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            kind: AssetKind::Audio { path },
        }
    }

    /// Create a new generative video asset (starts hollow)
    pub fn new_generative_video(name: impl Into<String>, folder: PathBuf) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            kind: AssetKind::GenerativeVideo {
                folder,
                active_version: None,
            },
        }
    }

    /// Create a new generative image asset (starts hollow)
    pub fn new_generative_image(name: impl Into<String>, folder: PathBuf) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            kind: AssetKind::GenerativeImage {
                folder,
                active_version: None,
            },
        }
    }

    /// Create a new generative audio asset (starts hollow)
    pub fn new_generative_audio(name: impl Into<String>, folder: PathBuf) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            kind: AssetKind::GenerativeAudio {
                folder,
                active_version: None,
            },
        }
    }

    /// Check if this is a generative asset
    pub fn is_generative(&self) -> bool {
        self.kind.is_generative()
    }

    /// Check if this is a video asset (including generative video)
    pub fn is_video(&self) -> bool {
        matches!(self.kind, AssetKind::Video { .. } | AssetKind::GenerativeVideo { .. })
    }

    /// Check if this is an image asset (including generative image)  
    pub fn is_image(&self) -> bool {
        matches!(self.kind, AssetKind::Image { .. } | AssetKind::GenerativeImage { .. })
    }

    /// Check if this asset can be placed on a video track
    pub fn is_visual(&self) -> bool {
        self.kind.is_visual()
    }

    /// Check if this asset can be placed on an audio track
    pub fn is_audio(&self) -> bool {
        self.kind.is_audio()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_creation() {
        let video = Asset::new_video("My Video", PathBuf::from("video/clip.mp4"));
        assert!(video.is_visual());
        assert!(!video.is_audio());
        assert!(!video.is_generative());

        let gen_video = Asset::new_generative_video("Gen Video", PathBuf::from("generated/video/gen_001"));
        assert!(gen_video.is_visual());
        assert!(!gen_video.is_audio());
        assert!(gen_video.is_generative());
    }

    #[test]
    fn test_asset_serialization() {
        let asset = Asset::new_image("Test Image", PathBuf::from("images/test.png"));
        let json = serde_json::to_string_pretty(&asset).unwrap();
        let parsed: Asset = serde_json::from_str(&json).unwrap();
        assert_eq!(asset.id, parsed.id);
        assert_eq!(asset.name, parsed.name);
    }
}
