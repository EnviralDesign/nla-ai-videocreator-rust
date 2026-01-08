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
mod providers;
mod generative;

pub use project::*;
pub use asset::*;
pub use selection::*;
#[allow(unused_imports)]
pub use providers::*;
#[allow(unused_imports)]
pub use generative::*;
