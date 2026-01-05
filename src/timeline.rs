//! Timeline components
//! 
//! This module contains the timeline panel and related components:
//! - TimelinePanel: Main timeline container with header and tracks
//! - TimeRuler: Time ruler with tick marks
//! - TrackLabel: Track label in the sidebar
//! - TrackRow: Track row content area

use dioxus::prelude::*;

// Re-export colors from app (we'll move these to a shared module later)
use crate::app::{
    BG_BASE, BG_ELEVATED, BG_HOVER, BG_SURFACE,
    BORDER_DEFAULT, BORDER_STRONG, BORDER_SUBTLE,
    TEXT_DIM, TEXT_MUTED, TEXT_SECONDARY,
    ACCENT_AUDIO, ACCENT_MARKER, ACCENT_VIDEO,
};
use crate::state::{Track, TrackType};

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
    // Timeline state
    current_time: f64,
    duration: f64,
    zoom: f64,
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
) -> Element {
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
    
    // Calculate timeline content width based on duration and zoom
    let content_width = (duration * zoom) as i32;
    
    // Calculate playhead position in scroll space (snapped to frame for visual alignment)
    let playhead_pos = ((current_time * FPS).round() / FPS) * zoom;
    
    // Constants
    let ruler_height = 24;
    let track_label_width = 140;

    rsx! {
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
                            "{zoom as i32}px/s" 
                        }
                        button {
                            class: "collapse-btn",
                            style: "width: 20px; height: 20px; border: none; border-radius: 3px; background: transparent; color: {TEXT_MUTED}; font-size: 12px; cursor: pointer; display: flex; align-items: center; justify-content: center;",
                            onclick: move |_| on_zoom_change.call(zoom * 1.25),
                            "+"
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
                                        zoom: zoom,
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
    
    // Calculate visible time range for frame ticks
    let visible_start_time = (scroll_offset / zoom).max(0.0);
    let visible_end_time = ((scroll_offset + 2000.0) / zoom).min(duration);
    
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
                                let x = (frame_time * zoom) - scroll_offset;
                                // Skip frame ticks that land on second boundaries
                                let is_on_second = frame % 60 == 0;
                                
                                if !is_on_second && x >= -10.0 && x <= 2010.0 {
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
                    let x = (t * zoom) - scroll_offset;
                    let minutes = t as i32 / 60;
                    let seconds = t as i32 % 60;
                    let label = format!("{}:{:02}", minutes, seconds);
                    
                    if x >= -50.0 && x <= 2000.0 {  // Only render visible ticks
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
                                // Label
                                div {
                                    style: "
                                        position: absolute;
                                        left: {x + 4.0}px;
                                        top: 3px;
                                        font-size: 9px;
                                        color: {TEXT_DIM};
                                        font-family: 'SF Mono', Consolas, monospace;
                                        user-select: none;
                                        pointer-events: none;
                                    ",
                                    "{label}"
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
    zoom: f64,  // pixels per second
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
    
    rsx! {
        div { 
            style: "
                height: 36px; min-width: {width}px; 
                border-bottom: 1px solid {BORDER_SUBTLE}; 
                background-color: {BG_BASE};
                position: relative;
            ",
            
            // Render each clip
            for clip in track_clips.iter() {
                {
                    let left = (clip.start_time * zoom) as i32;
                    let clip_width = (clip.duration * zoom).max(20.0) as i32;  // Min width 20px
                    
                    // Find asset name
                    let asset_name = assets.iter()
                        .find(|a| a.id == clip.asset_id)
                        .map(|a| a.name.clone())
                        .unwrap_or_else(|| "Unknown".to_string());
                    
                    // Check if asset is generative
                    let is_generative = assets.iter()
                        .find(|a| a.id == clip.asset_id)
                        .map(|a| a.is_generative())
                        .unwrap_or(false);
                    
                    let border_style = if is_generative {
                        format!("1px dashed {}", clip_color)
                    } else {
                        format!("1px solid {}", clip_color)
                    };
                    
                    rsx! {
                        div {
                            key: "{clip.id}",
                            style: "
                                position: absolute;
                                left: {left}px;
                                top: 2px;
                                width: {clip_width}px;
                                height: 32px;
                                background-color: {BG_ELEVATED};
                                border: {border_style};
                                border-radius: 4px;
                                display: flex;
                                align-items: center;
                                padding: 0 6px;
                                overflow: hidden;
                                cursor: grab;
                                user-select: none;
                            ",
                            // Color indicator bar
                            div {
                                style: "width: 3px; height: 20px; border-radius: 2px; background-color: {clip_color}; flex-shrink: 0; margin-right: 6px;",
                            }
                            // Clip name
                            span {
                                style: "font-size: 10px; color: {TEXT_SECONDARY}; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                                if is_generative { "✨ " } else { "" }
                                "{asset_name}"
                            }
                        }
                    }
                }
            }
        }
    }
}
