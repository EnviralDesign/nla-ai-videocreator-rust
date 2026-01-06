//! State management module
//!
//! This module contains all the core data structures for the application:
//! - Project: The top-level container for a video project
//! - Track: Timeline tracks (Video, Audio, Markers)
//! - Clip: Media clips placed on tracks
//! - Asset: Project assets (imported files and generative assets)
//! - Marker: Point-in-time annotations

mod project;
mod asset;
mod selection;

pub use project::*;
pub use asset::*;
pub use selection::*;
