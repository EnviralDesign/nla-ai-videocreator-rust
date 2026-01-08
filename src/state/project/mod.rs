//! Project data model
//!
//! This module contains the core data structures for a video project.

mod project;
mod track;
mod clip;
mod marker;
mod settings;
mod persistence;

pub use project::Project;
pub use track::{Track, TrackType};
pub use clip::{Clip, ClipTransform};
pub use marker::Marker;
pub use settings::ProjectSettings;
