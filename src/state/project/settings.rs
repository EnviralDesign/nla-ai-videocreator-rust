use serde::{Deserialize, Serialize};

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
