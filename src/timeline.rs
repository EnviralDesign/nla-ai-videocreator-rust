//! Timeline components
//! 
//! This module contains the timeline panel and related components:
//! - TimelinePanel: Main timeline container with header and tracks
//! - TimeRuler: Time ruler with tick marks
//! - TrackLabel: Track label in the sidebar
//! - TrackRow: Track row content area

use dioxus::prelude::*;
use std::collections::HashMap;

// Re-export colors from app (we'll move these to a shared module later)
use crate::constants::{
    BG_BASE, BG_ELEVATED, BG_HOVER, BG_SURFACE,
    BORDER_ACCENT, BORDER_DEFAULT, BORDER_STRONG, BORDER_SUBTLE,
    TEXT_DIM, TEXT_MUTED, TEXT_SECONDARY, TEXT_PRIMARY,
    ACCENT_AUDIO, ACCENT_MARKER, ACCENT_VIDEO,
    TIMELINE_MAX_PX_PER_FRAME, TIMELINE_MIN_ZOOM_FLOOR,
};
use crate::state::{Track, TrackType};

const THUMB_TILE_WIDTH_PX: f64 = 60.0;
const MAX_THUMB_TILES: usize = 120;
const MIN_CLIP_WIDTH_PX: f64 = 20.0;
const MIN_CLIP_WIDTH_FLOOR_PX: f64 = 2.0;
const MIN_CLIP_WIDTH_SCALE: f64 = 0.2;

pub fn timeline_zoom_bounds(duration: f64, viewport_width: Option<f64>, fps: f64) -> (f64, f64) {
    let duration = duration.max(0.01);
    let viewport_width = viewport_width.unwrap_or(600.0).max(1.0);
    let min_zoom = (viewport_width / duration).max(TIMELINE_MIN_ZOOM_FLOOR);
    let max_zoom = (fps.max(1.0) * TIMELINE_MAX_PX_PER_FRAME).max(min_zoom);
    (min_zoom, max_zoom)
}

/// Main timeline panel component
#[component]
pub fn TimelinePanel(
    height: f64, 
    collapsed: bool, 
    is_resizing: bool, 
    on_toggle: EventHandler<MouseEvent>,
    // Project data
    tracks: Vec<Track>,
    clips: Vec<crate::state::Clip>,
    assets: Vec<crate::state::Asset>,
    thumbnailer: std::sync::Arc<crate::core::thumbnailer::Thumbnailer>,
    thumbnail_cache_buster: u64,
    thumbnail_refresh_tick: u64,
    clip_cache_buckets: std::sync::Arc<HashMap<uuid::Uuid, Vec<bool>>>,
    // Timeline state
    current_time: f64,
    duration: f64,
    zoom: f64,
    min_zoom: f64,
    max_zoom: f64,
    is_playing: bool,
    scroll_offset: f64,
    // Callbacks
    on_seek: EventHandler<f64>,
    on_zoom_change: EventHandler<f64>,
    on_play_pause: EventHandler<MouseEvent>,
    on_scroll: EventHandler<f64>,
    on_seek_start: EventHandler<MouseEvent>,
    on_seek_end: EventHandler<MouseEvent>,
    is_seeking: bool,
    // Track management
    on_add_video_track: EventHandler<MouseEvent>,
    on_add_audio_track: EventHandler<MouseEvent>,
    on_track_context_menu: EventHandler<(f64, f64, uuid::Uuid)>,  // (x, y, track_id)
    // Clip operations
    on_clip_delete: EventHandler<uuid::Uuid>,
    on_clip_move: EventHandler<(uuid::Uuid, f64)>,  // (clip_id, new_start_time)
    on_clip_resize: EventHandler<(uuid::Uuid, f64, f64)>,  // (clip_id, new_start, new_duration)
    on_clip_move_track: EventHandler<(uuid::Uuid, i32)>, // (clip_id, direction)
    selected_clips: Vec<uuid::Uuid>,
    on_clip_select: EventHandler<uuid::Uuid>,
    // Asset Drag & Drop
    dragged_asset: Option<uuid::Uuid>,
    on_asset_drop: EventHandler<(uuid::Uuid, f64, uuid::Uuid)>, // (track_id, time, asset_id)
    // Selection
    on_deselect_all: EventHandler<MouseEvent>,
) -> Element {
    let _ = thumbnail_refresh_tick;
    let icon = if collapsed { "▲" } else { "▼" };
    let play_icon = if is_playing { "⏸" } else { "▶" };
    
    // Only apply transition when NOT resizing
    let transition = if is_resizing { "none" } else { "height 0.2s ease, min-height 0.2s ease" };
    
    // Cursor for collapsed header
    let header_cursor = if collapsed { "pointer" } else { "default" };
    let header_class = if collapsed { "collapsed-rail" } else { "" };
    
    // Frame rate constant
    const FPS: f64 = 60.0;
    
    // Format time as HH:MM:SS:FF (assuming 60 fps for now)
    let format_time = |t: f64| -> String {
        let total_frames = (t * FPS) as u32;
        let frames = total_frames % 60;
        let total_seconds = total_frames / 60;
        let seconds = total_seconds % 60;
        let total_minutes = total_seconds / 60;
        let minutes = total_minutes % 60;
        let hours = total_minutes / 60;
        format!("{:02}:{:02}:{:02}:{:02}", hours, minutes, seconds, frames)
    };
    
    let timecode = format_time(current_time);
    let zoom_label = if (zoom - min_zoom).abs() <= 0.5 {
        "Fit".to_string()
    } else if (zoom - max_zoom).abs() <= 0.5 {
        "Frames".to_string()
    } else {
        format!("{:.0}px/s", zoom)
    };
    
    // Calculate timeline content width based on duration and zoom
    let content_width = (duration * zoom) as i32;
    
    // Calculate playhead position in scroll space (snapped to frame for visual alignment)
    // Clamp to content_width - 1 so playhead line/triangle don't extend past content and cause scroll expansion
    let content_width_f = content_width as f64;
    let playhead_pos = (((current_time * FPS).round() / FPS) * zoom).min(content_width_f - 1.0).max(0.0);
    
    // Constants
    let ruler_height = 24;
    let track_label_width = 140;

    rsx! {
        {
            let _ = thumbnail_refresh_tick;
        }
        div {
            style: "
                display: flex; flex-direction: column;
                height: {height}px; min-height: {height}px;
                background-color: {BG_ELEVATED};
                transition: {transition};
                overflow: hidden;
            ",

            // Header
            div {
                class: "{header_class}",
                style: "
                    display: flex; align-items: center; justify-content: space-between;
                    height: 32px; padding: 0 14px;
                    background-color: {BG_SURFACE}; border-bottom: 1px solid {BORDER_DEFAULT};
                    flex-shrink: 0;
                    cursor: {header_cursor};
                ",
                onclick: move |e| {
                    if collapsed {
                        on_toggle.call(e);
                    }
                },
                
                // Left: Timeline label + zoom controls
                div {
                    style: "display: flex; align-items: center; gap: 12px;",
                    onclick: move |e| e.stop_propagation(),
                    span { style: "font-size: 11px; font-weight: 500; color: {TEXT_MUTED}; text-transform: uppercase; letter-spacing: 0.5px;", "Timeline" }
                    
                    // Zoom controls
                    div {
                        style: "display: flex; align-items: center; gap: 4px;",
                        button {
                            class: "collapse-btn",
                            style: "width: 20px; height: 20px; border: none; border-radius: 3px; background: transparent; color: {TEXT_MUTED}; font-size: 12px; cursor: pointer; display: flex; align-items: center; justify-content: center;",
                            onclick: move |_| on_zoom_change.call(zoom * 0.8),
                            "−"
                        }
                        span { 
                            style: "font-size: 10px; color: {TEXT_DIM}; min-width: 40px; text-align: center;", 
                            "{zoom_label}" 
                        }
                        button {
                            class: "collapse-btn",
                            style: "width: 20px; height: 20px; border: none; border-radius: 3px; background: transparent; color: {TEXT_MUTED}; font-size: 12px; cursor: pointer; display: flex; align-items: center; justify-content: center;",
                            onclick: move |_| on_zoom_change.call(zoom * 1.25),
                            "+"
                        }
                        button {
                            class: "collapse-btn",
                            style: "padding: 0 6px; height: 20px; border: none; border-radius: 3px; background: transparent; color: {TEXT_MUTED}; font-size: 10px; cursor: pointer; display: flex; align-items: center; justify-content: center;",
                            onclick: move |_| on_zoom_change.call(min_zoom),
                            "Fit"
                        }
                        button {
                            class: "collapse-btn",
                            style: "padding: 0 6px; height: 20px; border: none; border-radius: 3px; background: transparent; color: {TEXT_MUTED}; font-size: 10px; cursor: pointer; display: flex; align-items: center; justify-content: center;",
                            onclick: move |_| on_zoom_change.call(max_zoom),
                            "Frames"
                        }
                    }
                }
                
                // Center: Playback controls
                div {
                    style: "display: flex; align-items: center; gap: 4px;",
                    onclick: move |e| e.stop_propagation(),
                    PlaybackBtn { 
                        icon: "⏮",
                        on_click: move |_| on_seek.call(0.0),
                    }
                    PlaybackBtn { 
                        icon: "|◀",
                        on_click: move |_| {
                            // Snap to previous round second
                            let t = (current_time - 0.01).floor().max(0.0);
                            on_seek.call(t);
                        },
                    }
                    PlaybackBtn { 
                        icon: play_icon, 
                        primary: true,
                        on_click: move |e| on_play_pause.call(e),
                    }
                    PlaybackBtn { 
                        icon: "▶|",
                        on_click: move |_| {
                            // Snap to next round second
                            let t = (current_time.floor() + 1.0).min(duration);
                            on_seek.call(t);
                        },
                    }
                    PlaybackBtn { 
                        icon: "⏭",
                        on_click: move |_| on_seek.call(duration),
                    }
                }

                // Right: Timecode + collapse button
                div {
                    style: "display: flex; align-items: center; gap: 12px;",
                    span { 
                        style: "font-family: 'SF Mono', Consolas, monospace; font-size: 11px; color: {TEXT_DIM};", 
                        "{timecode}" 
                    }
                    button {
                        class: "collapse-btn",
                        style: "width: 24px; height: 24px; border: none; border-radius: 4px; background: transparent; color: {TEXT_MUTED}; font-size: 10px; cursor: pointer; display: flex; align-items: center; justify-content: center;",
                        onclick: move |e| {
                            e.stop_propagation();
                            on_toggle.call(e);
                        },
                        "{icon}"
                    }
                }
            }

            // Timeline content area - Robust hierarchical structure:
            // ┌─────────────────────────────────────────────────────┐
            // │ [Fixed Corner] │ [Scrollable Ruler + Playhead Head] │ <- ruler_height
            // ├────────────────┼────────────────────────────────────┤
            // │ [Fixed Labels] │ [Scrollable Tracks + Playhead Line]│ <- flex: 1
            // │                │ ↔ horizontal scroll                │
            // └────────────────┴────────────────────────────────────┘
            //                  ^ scrollbar only here
            if !collapsed {
                div {
                    style: "flex: 1; display: flex; overflow: hidden;",
                    
                    // ═══════════════════════════════════════════════════════════════
                    // LEFT COLUMN - Fixed width, never scrolls horizontally
                    // ═══════════════════════════════════════════════════════════════
                    div {
                        style: "
                            width: {track_label_width}px; 
                            min-width: {track_label_width}px;
                            flex-shrink: 0;
                            display: flex;
                            flex-direction: column;
                            background-color: {BG_ELEVATED};
                            border-right: 1px solid {BORDER_DEFAULT};
                            z-index: 20;
                        ",
                        
                        // Corner cell above track labels
                        div {
                            style: "
                                height: {ruler_height}px;
                                flex-shrink: 0;
                                border-bottom: 1px solid {BORDER_DEFAULT};
                                background-color: {BG_ELEVATED};
                            ",
                        }
                        
                        // Track labels - scrolls vertically with tracks (via overflow: auto on this container if needed)
                        div {
                            style: "flex: 1; overflow-y: hidden; overflow-x: hidden; display: flex; flex-direction: column;",
                            
                            // Existing track labels
                            div {
                                style: "flex: 1;",
                                for track in tracks.iter() {
                                    {
                                        let color = match track.track_type {
                                            TrackType::Video => ACCENT_VIDEO,
                                            TrackType::Audio => ACCENT_AUDIO,
                                            TrackType::Marker => ACCENT_MARKER,
                                        };
                                        let tid = track.id;
                                        rsx! {
                                            TrackLabel { 
                                                key: "{track.id}",
                                                name: track.name.clone(), 
                                                color: color,
                                                track_id: tid,
                                                on_context_menu: move |data| on_track_context_menu.call(data),
                                            }
                                        }
                                    }
                                }
                            }
                            
                            // Add track buttons
                            div {
                                style: "
                                    display: flex; gap: 4px; padding: 8px 12px;
                                    border-top: 1px solid {BORDER_SUBTLE};
                                ",
                                button {
                                    class: "collapse-btn",
                                    style: "
                                        flex: 1; height: 24px; border: 1px dashed {BORDER_DEFAULT}; 
                                        border-radius: 4px; background: transparent; 
                                        color: {TEXT_DIM}; font-size: 10px; cursor: pointer;
                                        display: flex; align-items: center; justify-content: center;
                                        gap: 4px;
                                    ",
                                    onclick: move |e| on_add_video_track.call(e),
                                    span { style: "color: {ACCENT_VIDEO};", "+" }
                                    "Video"
                                }
                                button {
                                    class: "collapse-btn",
                                    style: "
                                        flex: 1; height: 24px; border: 1px dashed {BORDER_DEFAULT}; 
                                        border-radius: 4px; background: transparent; 
                                        color: {TEXT_DIM}; font-size: 10px; cursor: pointer;
                                        display: flex; align-items: center; justify-content: center;
                                        gap: 4px;
                                    ",
                                    onclick: move |e| on_add_audio_track.call(e),
                                    span { style: "color: {ACCENT_AUDIO};", "+" }
                                    "Audio"
                                }
                            }
                        }
                    }
                    
                    // ═══════════════════════════════════════════════════════════════
                    // RIGHT COLUMN - Single scrollable container for ruler + tracks
                    // The ruler is sticky at top, everything scrolls horizontally together
                    // ═══════════════════════════════════════════════════════════════
                    div {
                        id: "timeline-scroll-host",
                        style: "
                            flex: 1;
                            overflow-x: auto;
                            overflow-y: auto;
                            position: relative;
                        ",
                        
                        // Inner content wrapper - sets the scrollable width
                        div {
                            style: "
                                min-width: {content_width}px;
                                display: flex;
                                flex-direction: column;
                                position: relative;
                            ",
                            
                            // Ruler row - sticky at top, scrolls horizontally with content
                            div {
                                style: "
                                    height: {ruler_height}px;
                                    min-height: {ruler_height}px;
                                    position: sticky;
                                    top: 0;
                                    z-index: 15;
                                    background-color: {BG_SURFACE};
                                    border-bottom: 1px solid {BORDER_DEFAULT};
                                    cursor: pointer;
                                    overflow: hidden;
                                ",
                                // Click anywhere on ruler to seek AND start dragging
                                onmousedown: move |e| {
                                    e.prevent_default();
                                    // element_coordinates gives position relative to this ruler element
                                    // which is in scroll space (content coordinates)
                                    let x = e.element_coordinates().x;
                                    let t = (x / zoom).clamp(0.0, duration);
                                    // Snap to frame and seek immediately
                                    let snapped = ((t * 60.0).round() / 60.0).clamp(0.0, duration);
                                    on_seek.call(snapped);
                                    // Start drag mode so continued mouse movement continues seeking
                                    on_seek_start.call(e);
                                },
                                
                                // Ruler ticks and labels (positioned in scroll space)
                                TimeRuler {
                                    duration: duration,
                                    zoom: zoom,
                                    scroll_offset: 0.0,  // No offset - we're in scroll space
                                }
                                
                                // Playhead indicator on ruler (in scroll space)
                                div {
                                    style: "
                                        position: absolute;
                                        left: {playhead_pos}px;
                                        top: 0;
                                        width: 1px;
                                        height: 100%;
                                        background-color: #ef4444;
                                        pointer-events: none;
                                    ",
                                }
                                // Playhead handle (triangle) - purely visual
                                div {
                                    style: "
                                        position: absolute;
                                        left: {playhead_pos - 5.0}px;
                                        top: 0;
                                        width: 0;
                                        height: 0;
                                        border-left: 6px solid transparent;
                                        border-right: 6px solid transparent;
                                        border-top: 8px solid #ef4444;
                                        pointer-events: none;
                                    ",
                                }
                            }
                            
                            // Track rows container
                            div {
                                style: "
                                    display: flex;
                                    flex-direction: column;
                                    position: relative;
                                ",
                                
                                for track in tracks.iter() {
                                    TrackRow { 
                                        key: "{track.id}",
                                        width: content_width,
                                        track_id: track.id,
                                        track_type: track.track_type.clone(),
                                        clips: clips.clone(),
                                        assets: assets.clone(),
                                        thumbnailer: thumbnailer.clone(),
                                        thumbnail_cache_buster: thumbnail_cache_buster,
                                        clip_cache_buckets: clip_cache_buckets.clone(),
                                        zoom: zoom,
                                        on_clip_delete: move |id| on_clip_delete.call(id),
                                        on_clip_move: move |(id, time)| on_clip_move.call((id, time)),
                                        on_clip_resize: move |(id, start, dur)| on_clip_resize.call((id, start, dur)),
                                        on_clip_move_track: move |(id, direction)| on_clip_move_track.call((id, direction)),
                                        selected_clips: selected_clips.clone(),
                                        on_clip_select: move |id| on_clip_select.call(id),
                                        dragged_asset: dragged_asset,
                                        on_asset_drop: move |(tid, t, aid)| on_asset_drop.call((tid, t, aid)),
                                        on_deselect_all: move |e| on_deselect_all.call(e),
                                    }
                                }
                                
                                // Playhead line overlaying tracks (in scroll space)
                                div {
                                    style: "
                                        position: absolute;
                                        left: {playhead_pos}px;
                                        top: 0;
                                        width: 1px;
                                        height: 100%;
                                        background-color: #ef4444;
                                        pointer-events: none;
                                        z-index: 10;
                                    ",
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Time ruler with tick marks and labels
/// All elements here use pointer-events: none so clicks pass through to parent
#[component]
fn TimeRuler(duration: f64, zoom: f64, scroll_offset: f64) -> Element {
    let _ = scroll_offset;
    // Calculate tick spacing based on zoom level
    let seconds_per_major_tick = if zoom < 30.0 {
        10.0
    } else if zoom < 60.0 {
        5.0
    } else if zoom < 120.0 {
        2.0
    } else {
        1.0
    };
    
    // Frame rate for frame ticks
    const FPS: f64 = 60.0;
    
    // Show frame ticks only at high zoom levels (when there's enough space)
    // At 100px/s zoom, each frame is ~1.67px apart - too dense
    // At 300px/s zoom, each frame is 5px apart - usable
    // At 500px/s zoom, each frame is ~8.3px apart - comfortable
    let show_frame_ticks = zoom >= 240.0;
    
    // Generate tick positions
    let num_ticks = (duration / seconds_per_major_tick).ceil() as i32 + 1;
    
    let content_width = duration * zoom;
    let visible_start_time = 0.0;
    let visible_end_time = duration;
    
    rsx! {
        // Entire ruler container ignores pointer events - clicks pass through
        div {
            style: "position: absolute; left: 0; top: 0; width: 100%; height: 100%; pointer-events: none;",
            
            // Frame ticks (subtle, only at high zoom)
            if show_frame_ticks {
                {
                    let start_frame = (visible_start_time * FPS).floor() as i32;
                    let end_frame = (visible_end_time * FPS).ceil() as i32;
                    
                    rsx! {
                        for frame in start_frame..=end_frame {
                            {
                                let frame_time = frame as f64 / FPS;
                                let x = frame_time * zoom;
                                // Skip frame ticks that land on second boundaries
                                let is_on_second = frame % 60 == 0;
                                
                                if !is_on_second && x <= content_width + 10.0 {
                                    rsx! {
                                        div {
                                            key: "frame-{frame}",
                                            style: "
                                                position: absolute;
                                                left: {x}px;
                                                bottom: 0;
                                                width: 1px;
                                                height: 4px;
                                                background-color: {BORDER_SUBTLE};
                                                pointer-events: none;
                                            ",
                                        }
                                    }
                                } else {
                                    rsx! {}
                                }
                            }
                        }
                    }
                }
            }
            
            // Second/major ticks and labels
            for i in 0..num_ticks {
                {
                    let t = i as f64 * seconds_per_major_tick;
                    let x = t * zoom;
                    let minutes = t as i32 / 60;
                    let seconds = t as i32 % 60;
                    let label = format!("{}:{:02}", minutes, seconds);
                    
                    if x <= content_width + 50.0 {
                        rsx! {
                            // Container for tick + label (key must be on first node)
                            div {
                                key: "tick-group-{i}",
                                // Major tick (second boundary)
                                div {
                                    style: "
                                        position: absolute;
                                        left: {x}px;
                                        bottom: 0;
                                        width: 1px;
                                        height: 10px;
                                        background-color: {BORDER_STRONG};
                                        pointer-events: none;
                                    ",
                                }
                                // Label - right-align last tick to prevent overflow
                                {
                                    // Check if this is the last visible tick
                                    let is_last_tick = i == num_ticks - 1;
                                    let next_tick_x = (i as f64 + 1.0) * seconds_per_major_tick * zoom;
                                    let is_near_end = next_tick_x > content_width;
                                    let should_right_align = is_last_tick || is_near_end;
                                    
                                    // For last label, use transform to shift text left of anchor point
                                    let label_style = if should_right_align {
                                        format!(
                                            "position: absolute; left: {}px; top: 3px; font-size: 9px; color: {}; font-family: 'SF Mono', Consolas, monospace; user-select: none; pointer-events: none; transform: translateX(-100%);",
                                            x - 4.0, TEXT_DIM
                                        )
                                    } else {
                                        format!(
                                            "position: absolute; left: {}px; top: 3px; font-size: 9px; color: {}; font-family: 'SF Mono', Consolas, monospace; user-select: none; pointer-events: none;",
                                            x + 4.0, TEXT_DIM
                                        )
                                    };
                                    rsx! {
                                        div {
                                            style: "{label_style}",
                                            "{label}"
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        rsx! {}
                    }
                }
            }
        }
    }
}

/// Playback button
#[component]
fn PlaybackBtn(
    icon: &'static str, 
    #[props(default = false)] primary: bool,
    on_click: EventHandler<MouseEvent>,
) -> Element {
    let bg = if primary { BG_HOVER } else { "transparent" };
    rsx! {
        button {
            class: "collapse-btn",
            style: "width: 26px; height: 26px; border: none; border-radius: 4px; background-color: {bg}; color: {TEXT_MUTED}; font-size: 10px; cursor: pointer; display: flex; align-items: center; justify-content: center; transition: all 0.12s ease;",
            onclick: move |e| on_click.call(e),
            "{icon}"
        }
    }
}

/// Track label in the sidebar
#[component]
pub fn TrackLabel(
    name: String, 
    color: &'static str,
    track_id: uuid::Uuid,
    on_context_menu: EventHandler<(f64, f64, uuid::Uuid)>,
) -> Element {
    rsx! {
        div {
            style: "
                display: flex; align-items: center; gap: 10px; height: 36px; 
                padding: 0 12px; border-bottom: 1px solid {BORDER_SUBTLE}; 
                font-size: 12px; color: {TEXT_SECONDARY}; cursor: context-menu;
            ",
            oncontextmenu: move |e| {
                e.prevent_default();
                let coords = e.client_coordinates();
                on_context_menu.call((coords.x, coords.y, track_id));
            },
            div { style: "width: 3px; height: 16px; border-radius: 2px; background-color: {color};" }
            span { "{name}" }
        }
    }
}
/// Track row content area
#[component]
pub fn TrackRow(
    width: i32,
    track_id: uuid::Uuid,
    track_type: TrackType,
    clips: Vec<crate::state::Clip>,
    assets: Vec<crate::state::Asset>,
    thumbnailer: std::sync::Arc<crate::core::thumbnailer::Thumbnailer>,
    thumbnail_cache_buster: u64,
    clip_cache_buckets: std::sync::Arc<HashMap<uuid::Uuid, Vec<bool>>>,
    zoom: f64,  // pixels per second
    on_clip_delete: EventHandler<uuid::Uuid>,
    on_clip_move: EventHandler<(uuid::Uuid, f64)>,  // (clip_id, new_start_time)
    on_clip_resize: EventHandler<(uuid::Uuid, f64, f64)>,  // (clip_id, new_start, new_duration)
    on_clip_move_track: EventHandler<(uuid::Uuid, i32)>,
    selected_clips: Vec<uuid::Uuid>,
    on_clip_select: EventHandler<uuid::Uuid>,
    dragged_asset: Option<uuid::Uuid>,
    on_asset_drop: EventHandler<(uuid::Uuid, f64, uuid::Uuid)>,
    on_deselect_all: EventHandler<MouseEvent>,
) -> Element {
    // Filter clips for this track
    let track_clips: Vec<_> = clips.iter()
        .filter(|c| c.track_id == track_id)
        .collect();
    
    // Color based on track type
    let clip_color = match track_type {
        TrackType::Video => ACCENT_VIDEO,
        TrackType::Audio => ACCENT_AUDIO,
        TrackType::Marker => ACCENT_MARKER,
    };
    
    // Check compatibility for drop
    let can_drop = if let Some(asset_id) = dragged_asset {
        assets.iter().find(|a| a.id == asset_id).map(|a| {
            match track_type {
                TrackType::Video => a.is_visual(),
                TrackType::Audio => a.is_audio(),
                TrackType::Marker => false,
            }
        }).unwrap_or(false)
    } else { false };
    
    let bg_color = if can_drop { BG_HOVER } else { BG_BASE };
    
    rsx! {
        div { 
            style: "
                height: 36px; min-width: {width}px; 
                border-bottom: 1px solid {BORDER_SUBTLE}; 
                background-color: {bg_color};
                position: relative;
                transition: background-color 0.2s;
            ",
            oncontextmenu: move |e| e.prevent_default(),
            onmousedown: move |e| {
                // Click on empty track area deselects all clips
                if let Some(btn) = e.trigger_button() {
                    if format!("{:?}", btn) == "Primary" {
                        on_deselect_all.call(e);
                    }
                }
            },
            onmouseup: move |e| {
                if let Some(asset_id) = dragged_asset {
                    if can_drop {
                        e.prevent_default();
                        // Calculate time from drop position
                        let x = e.element_coordinates().x;
                        let time = (x / zoom).max(0.0);
                        let snapped = (time * 60.0).round() / 60.0;
                        on_asset_drop.call((track_id, snapped, asset_id));
                    }
                }
            },
            
            // Render each clip
            for clip in track_clips.iter() {
                ClipElement {
                    key: "{clip.id}",
                    clip: (*clip).clone(),
                    assets: assets.clone(),
                    thumbnailer: thumbnailer.clone(),
                    thumbnail_cache_buster: thumbnail_cache_buster,
                    clip_cache_buckets: clip_cache_buckets.clone(),
                    zoom: zoom,
                    clip_color: clip_color,
                    on_delete: move |id| on_clip_delete.call(id),
                    on_move: move |(id, time)| on_clip_move.call((id, time)),
                    on_resize: move |(id, start, dur)| on_clip_resize.call((id, start, dur)),
                    on_move_track: move |(id, direction)| on_clip_move_track.call((id, direction)),
                    is_selected: selected_clips.contains(&clip.id),
                    on_select: move |id| on_clip_select.call(id),
                }
            }
        }
    }
}

/// Interactive clip element with drag, resize, and context menu support
#[component]
fn ClipElement(
    clip: crate::state::Clip,
    assets: Vec<crate::state::Asset>,
    thumbnailer: std::sync::Arc<crate::core::thumbnailer::Thumbnailer>,
    thumbnail_cache_buster: u64,
    clip_cache_buckets: std::sync::Arc<HashMap<uuid::Uuid, Vec<bool>>>,
    zoom: f64,
    clip_color: &'static str,
    on_delete: EventHandler<uuid::Uuid>,
    on_move: EventHandler<(uuid::Uuid, f64)>,
    on_resize: EventHandler<(uuid::Uuid, f64, f64)>,  // (id, new_start, new_duration)
    on_move_track: EventHandler<(uuid::Uuid, i32)>,
    is_selected: bool,
    on_select: EventHandler<uuid::Uuid>,
) -> Element {
    let mut show_menu = use_signal(|| false);
    let mut menu_pos = use_signal(|| (0.0, 0.0));
    let mut drag_mode = use_signal(|| None::<&'static str>);  // None, "move", "resize-left", "resize-right"
    let mut drag_start_x = use_signal(|| 0.0);
    let mut drag_start_time = use_signal(|| 0.0);
    let mut drag_start_duration = use_signal(|| 0.0);
    let mut drag_start_end_time = use_signal(|| 0.0);

    let left = (clip.start_time * zoom) as i32;
    let min_clip_width = (zoom * MIN_CLIP_WIDTH_SCALE)
        .clamp(MIN_CLIP_WIDTH_FLOOR_PX, MIN_CLIP_WIDTH_PX);
    let clip_width = (clip.duration * zoom).max(min_clip_width) as i32;
    let clip_width_f = clip_width as f64;
    let clip_id = clip.id;
    let cache_buckets = clip_cache_buckets
        .get(&clip.id)
        .cloned()
        .unwrap_or_default();
    let cache_bucket_width = if cache_buckets.is_empty() {
        0.0
    } else {
        clip_width_f / cache_buckets.len() as f64
    };
    
    let asset = assets.iter().find(|a| a.id == clip.asset_id);
    let asset_name = asset
        .map(|a| a.name.clone())
        .unwrap_or_else(|| "Unknown".to_string());
    let base_name = clip
        .label
        .as_ref()
        .map(|label| label.trim())
        .filter(|label| !label.is_empty())
        .map(|label| label.to_string())
        .unwrap_or_else(|| asset_name.clone());
    let display_name = match asset.and_then(|asset| asset.active_version()) {
        Some(version) => format!("{} ({})", base_name, version),
        None => base_name,
    };
    let is_generative = asset.map(|a| a.is_generative()).unwrap_or(false);
    let is_visual = asset.map(|a| a.is_visual()).unwrap_or(false);
    let trim_in_seconds = clip.trim_in_seconds.max(0.0);
    let max_duration = asset.and_then(|a| {
        if a.is_video() || a.is_audio() {
            a.duration_seconds.filter(|duration| *duration > 0.0)
        } else {
            None
        }
    });
    let available_duration = max_duration.map(|duration| (duration - trim_in_seconds).max(0.0));
    
    let first_thumb_url = if is_visual {
        thumbnailer.get_thumbnail_path(clip.asset_id, trim_in_seconds).map(|p| {
            let url = crate::utils::get_local_file_url(&p);
            format!("{}?v={}", url, thumbnail_cache_buster)
        })
    } else {
        None
    };
    
    let mut thumb_tiles: Vec<String> = Vec::new();
    let mut tile_width = THUMB_TILE_WIDTH_PX;
    
    if let Some(fallback_url) = first_thumb_url.clone() {
        if clip_width > 40 {
            let estimated_tiles = (clip_width_f / tile_width).ceil() as usize;
            if estimated_tiles > MAX_THUMB_TILES {
                tile_width = (clip_width_f / MAX_THUMB_TILES as f64).ceil();
            }
            let tile_count = (clip_width_f / tile_width).ceil() as usize;
            let tile_count = tile_count.max(1);
            let tile_time = tile_width / zoom;
            
            for i in 0..tile_count {
                let time_in_clip = (i as f64 * tile_time).min(clip.duration.max(0.0));
                let time = trim_in_seconds + time_in_clip;
                let url = thumbnailer
                    .get_thumbnail_path(clip.asset_id, time)
                    .map(|p| {
                        let url = crate::utils::get_local_file_url(&p);
                        format!("{}?v={}", url, thumbnail_cache_buster)
                    })
                    .unwrap_or_else(|| fallback_url.clone());
                thumb_tiles.push(url);
            }
        }
    }
    
    let border_style = if is_generative {
        format!("1px dashed {}", clip_color)
    } else {
        format!("1px solid {}", clip_color)
    };
    let selection_ring = if is_selected {
        format!("0 0 0 1px {}", BORDER_ACCENT)
    } else {
        "none".to_string()
    };

    let current_start = clip.start_time;
    let current_duration = clip.duration;
    let current_end = current_start + current_duration;
    
    let is_active = drag_mode().is_some();
    let cursor_style = match drag_mode() {
        Some("resize-left") | Some("resize-right") => "ew-resize",
        Some("move") => "grabbing",
        _ => "grab",
    };
    let z_index = if is_active { "100" } else { "1" };
    
    rsx! {
        // Main clip element
        div {
            style: "
                position: absolute;
                left: {left}px;
                top: 2px;
                width: {clip_width}px;
                height: 32px;
                background-color: {BG_ELEVATED};
                border: {border_style};
                box-shadow: {selection_ring};
                border-radius: 4px;
                display: flex;
                align-items: center;
                overflow: visible;
                cursor: {cursor_style};
                user-select: none;
                z-index: {z_index};
            ",
            oncontextmenu: move |e| {
                e.prevent_default();
                e.stop_propagation();
                let coords = e.client_coordinates();
                menu_pos.set((coords.x, coords.y));
                show_menu.set(true);
            },

            // Thumbnails sub-layer (absolute, clipped to clip bounds)
            if !thumb_tiles.is_empty() {
                div {
                    style: "
                        position: absolute; left: 0; right: 0; top: 0; bottom: 0;
                        display: flex; overflow: hidden; opacity: 0.5;
                        pointer-events: none; z-index: 0; border-radius: 4px;
                    ",
                    for (idx, src_url) in thumb_tiles.iter().enumerate() {
                        img {
                            key: "thumb-{clip_id}-{idx}",
                            src: "{src_url}",
                            style: "height: 100%; width: {tile_width}px; object-fit: cover; flex: 0 0 {tile_width}px;",
                            draggable: "false",
                        }
                    }
                }
            }

            if !cache_buckets.is_empty() {
                div {
                    style: "
                        position: absolute; left: 0; right: 0; bottom: 0;
                        height: 3px; display: flex; pointer-events: none;
                        z-index: 2; opacity: 0.8;
                    ",
                    for (idx, cached) in cache_buckets.iter().enumerate() {
                        {
                            let color = if *cached { ACCENT_VIDEO } else { "transparent" };
                            rsx! {
                                div {
                                    key: "cache-{clip_id}-{idx}",
                                    style: "
                                        flex: 0 0 {cache_bucket_width}px;
                                        height: 100%;
                                        background-color: {color};
                                    ",
                                }
                            }
                        }
                    }
                }
            }
            
            // Left resize handle
            div {
                class: "resize-handle-left",
                style: "
                    position: absolute; left: -4px; top: 0; bottom: 0; width: 10px;
                    cursor: ew-resize; z-index: 10;
                    border-radius: 4px 0 0 4px;
                ",
                onmousedown: move |e| {
                    if let Some(btn) = e.trigger_button() {
                        if format!("{:?}", btn) == "Primary" {
                            e.prevent_default();
                            e.stop_propagation();
                            on_select.call(clip_id);
                            drag_mode.set(Some("resize-left"));
                            drag_start_x.set(e.client_coordinates().x);
                            drag_start_time.set(current_start);
                            drag_start_duration.set(current_duration);
                            drag_start_end_time.set(current_end);
                        }
                    }
                },
                oncontextmenu: move |e| {
                     e.prevent_default();
                     e.stop_propagation();
                     let coords = e.client_coordinates();
                     menu_pos.set((coords.x, coords.y));
                     show_menu.set(true);
                },
                // Visual bar on hover (simulated with CSS below)
                 div {
                    style: "
                        position: absolute; left: 3px; top: 6px; bottom: 6px; width: 4px;
                        background-color: rgba(255, 255, 255, 0.2); 
                        border-radius: 2px;
                        pointer-events: none;
                        opacity: 0;
                        transition: opacity 0.1s;
                    ",
                }
            }
            
            // Center drag area (the main clip body)
            div {
                style: "
                    flex: 1; height: 100%; display: flex; align-items: center;
                    padding: 0 10px; overflow: visible; position: relative; z-index: 1;
                ",
                onmousedown: move |e| {
                    if let Some(btn) = e.trigger_button() {
                        if format!("{:?}", btn) == "Primary" {
                            e.prevent_default();
                            e.stop_propagation();
                            on_select.call(clip_id);
                            drag_mode.set(Some("move"));
                            drag_start_x.set(e.client_coordinates().x);
                            drag_start_time.set(current_start);
                        }
                    }
                },
                oncontextmenu: move |e| {
                     e.prevent_default();
                     e.stop_propagation();
                     let coords = e.client_coordinates();
                     menu_pos.set((coords.x, coords.y));
                     show_menu.set(true);
                },
                
                // Foreground Content Container (Text + Indicator)
                div {
                    style: "
                        display: flex; align-items: center; width: 100%;
                        min-width: 0; overflow: hidden;
                        z-index: 1; position: relative;
                    ",
                    // Color indicator bar
                    div {
                        style: "width: 3px; height: 20px; border-radius: 2px; background-color: {clip_color}; flex-shrink: 0; margin-right: 6px;",
                    }
                    // Clip name with text shadow for readability over image
                    span {
                        style: "
                            font-size: 10px; color: {TEXT_PRIMARY}; 
                            white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
                            flex: 1; min-width: 0;
                            text-shadow: 0 1px 2px rgba(0,0,0,0.8);
                        ",
                        if is_generative { "✨ " } else { "" }
                        "{display_name}"
                    }
                }
            }
            
            // Right resize handle
            div {
                class: "resize-handle-right",
                style: "
                    position: absolute; right: -4px; top: 0; bottom: 0; width: 10px;
                    cursor: ew-resize; z-index: 10;
                    border-radius: 0 4px 4px 0;
                ",
                onmousedown: move |e| {
                    if let Some(btn) = e.trigger_button() {
                        if format!("{:?}", btn) == "Primary" {
                            e.prevent_default();
                            e.stop_propagation();
                            on_select.call(clip_id);
                            drag_mode.set(Some("resize-right"));
                            drag_start_x.set(e.client_coordinates().x);
                            drag_start_time.set(current_start);
                            drag_start_duration.set(current_duration);
                        }
                    }
                },
                oncontextmenu: move |e| {
                     e.prevent_default();
                     e.stop_propagation();
                     let coords = e.client_coordinates();
                     menu_pos.set((coords.x, coords.y));
                     show_menu.set(true);
                },
                // Visual bar
                div {
                    style: "
                        position: absolute; right: 3px; top: 6px; bottom: 6px; width: 4px;
                        background-color: rgba(255, 255, 255, 0.2); 
                        border-radius: 2px;
                        pointer-events: none;
                        opacity: 0;
                        transition: opacity 0.1s;
                    ",
                }
            }
        }
        
        // Global drag/resize overlay - captures all mouse events when active
        if drag_mode().is_some() {
            div {
                style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; z-index: 9999; cursor: {cursor_style};",
                oncontextmenu: move |e| e.prevent_default(),
                onmousemove: move |e| {
                    let delta_x = e.client_coordinates().x - drag_start_x();
                    let delta_time = delta_x / zoom;
                    
                    match drag_mode() {
                        Some("move") => {
                            let new_time = (drag_start_time() + delta_time).max(0.0);
                            let snapped = (new_time * 60.0).round() / 60.0;
                            on_move.call((clip_id, snapped));
                        }
                        Some("resize-left") => {
                            // Moving left edge: keep right edge fixed while clamping to source duration
                            let end_time = drag_start_end_time();
                            let min_start = (current_start - trim_in_seconds).max(0.0);
                            let mut new_start = (drag_start_time() + delta_time).max(min_start);
                            let mut new_duration = end_time - new_start;
                            if let Some(max_duration) = max_duration {
                                if new_duration > max_duration {
                                    new_duration = max_duration;
                                    new_start = (end_time - new_duration).max(0.0);
                                }
                            }
                            if new_duration < 0.1 {
                                new_duration = 0.1;
                                new_start = (end_time - new_duration).max(0.0);
                            }
                            let snapped_start = (new_start * 60.0).round() / 60.0;
                            let snapped_dur = (end_time - snapped_start).max(0.1);
                            on_resize.call((clip_id, snapped_start, snapped_dur));
                        }
                        Some("resize-right") => {
                            // Moving right edge: only changes duration, clamped to source duration
                            let mut new_duration = (drag_start_duration() + delta_time).max(0.1);
                            if let Some(available_duration) = available_duration {
                                new_duration = new_duration.min(available_duration);
                            }
                            let snapped_dur = (new_duration * 60.0).round() / 60.0;
                            on_resize.call((clip_id, current_start, snapped_dur));
                        }
                        _ => {}
                    }
                },
                onmouseup: move |_| {
                    drag_mode.set(None);
                },
            }
        }
        
        // Context menu overlay
        if show_menu() {
            // Backdrop to close menu
            div {
                style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; z-index: 9998;",
                onclick: move |_| show_menu.set(false),
                oncontextmenu: move |e| {
                    e.prevent_default();
                    show_menu.set(false);
                },
            }
            // Menu popup
            div {
                style: "
                    position: fixed; 
                    left: {menu_pos().0}px; 
                    top: {menu_pos().1}px;
                    background-color: {BG_ELEVATED}; border: 1px solid {BORDER_DEFAULT};
                    border-radius: 6px; padding: 4px 0; min-width: 120px;
                    box-shadow: 0 4px 12px rgba(0,0,0,0.3);
                    z-index: 9999; font-size: 12px;
                ",
                oncontextmenu: move |e| e.prevent_default(),
                div {
                    style: "
                        padding: 6px 12px; color: {TEXT_PRIMARY}; cursor: pointer;
                        transition: background-color 0.1s ease;
                    ",
                    onclick: move |_| {
                        on_move_track.call((clip_id, -1));
                        show_menu.set(false);
                    },
                    "Move Up"
                }
                div {
                    style: "
                        padding: 6px 12px; color: {TEXT_PRIMARY}; cursor: pointer;
                        transition: background-color 0.1s ease;
                    ",
                    onclick: move |_| {
                        on_move_track.call((clip_id, 1));
                        show_menu.set(false);
                    },
                    "Move Down"
                }
                div {
                    style: "height: 1px; background-color: {BORDER_SUBTLE}; margin: 4px 0;",
                }
                div {
                    style: "
                        padding: 6px 12px; color: #ef4444; cursor: pointer;
                        transition: background-color 0.1s ease;
                    ",
                    onclick: move |_| {
                        on_delete.call(clip_id);
                        show_menu.set(false);
                    },
                    "🗑 Delete Clip"
                }
            }
        }
    }
}
