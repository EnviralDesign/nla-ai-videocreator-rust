//! Timeline module
//!
//! Split from the original monolithic timeline.rs.

mod panel;
mod ruler;
mod playback_controls;
mod track_label;
mod track_row;
mod clip_element;

pub use panel::TimelinePanel;

use crate::constants::{TIMELINE_MAX_PX_PER_FRAME, TIMELINE_MIN_ZOOM_FLOOR};

pub(crate) const THUMB_TILE_WIDTH_PX: f64 = 60.0;
pub(crate) const MAX_THUMB_TILES: usize = 120;
pub(crate) const MIN_CLIP_WIDTH_PX: f64 = 20.0;
pub(crate) const MIN_CLIP_WIDTH_FLOOR_PX: f64 = 2.0;
pub(crate) const MIN_CLIP_WIDTH_SCALE: f64 = 0.2;

pub fn timeline_zoom_bounds(duration: f64, viewport_width: Option<f64>, fps: f64) -> (f64, f64) {
    let duration = duration.max(0.01);
    let viewport_width = viewport_width.unwrap_or(600.0).max(1.0);
    let min_zoom = (viewport_width / duration).max(TIMELINE_MIN_ZOOM_FLOOR);
    let max_zoom = (fps.max(1.0) * TIMELINE_MAX_PX_PER_FRAME).max(min_zoom);
    (min_zoom, max_zoom)
}
