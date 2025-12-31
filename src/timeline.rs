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
    ACCENT_AUDIO, ACCENT_MARKER, ACCENT_KEYFRAME, ACCENT_VIDEO,
};

/// Main timeline panel component
#[component]
pub fn TimelinePanel(
    height: f64, 
    collapsed: bool, 
    is_resizing: bool, 
    on_toggle: EventHandler<MouseEvent>,
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
    is_seeking: bool,  // True when playhead is being dragged
) -> Element {
    let icon = if collapsed { "▲" } else { "▼" };
    let play_icon = if is_playing { "⏸" } else { "▶" };
    
    // Only apply transition when NOT resizing
    let transition = if is_resizing { "none" } else { "height 0.2s ease, min-height 0.2s ease" };
    
    // Cursor for collapsed header
    let header_cursor = if collapsed { "pointer" } else { "default" };
    let header_class = if collapsed { "collapsed-rail" } else { "" };
    
    // Format time as HH:MM:SS:FF (assuming 60 fps for now)
    let format_time = |t: f64| -> String {
        let total_frames = (t * 60.0) as u32;
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
    
    // Calculate playhead position
    let playhead_x = (current_time * zoom) - scroll_offset;
    
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

            // Timeline content area
            if !collapsed {
                div {
                    style: "flex: 1; display: flex; flex-direction: column; overflow: hidden;",
                    
                    // Ruler row
                    div {
                        style: "display: flex; height: {ruler_height}px; flex-shrink: 0; border-bottom: 1px solid {BORDER_DEFAULT};",
                        
                        // Empty corner above track labels
                        div {
                            style: "width: {track_label_width}px; min-width: {track_label_width}px; background-color: {BG_ELEVATED}; border-right: 1px solid {BORDER_DEFAULT};",
                        }
                        
                        // Ruler area (clickable to seek)
                        div {
                            style: "flex: 1; position: relative; background-color: {BG_SURFACE}; overflow: hidden; cursor: pointer;",
                            onclick: move |e| {
                                // Don't seek if we were dragging the playhead
                                if !is_seeking {
                                    let x = e.element_coordinates().x + scroll_offset;
                                    let t = x / zoom;
                                    on_seek.call(t);
                                }
                            },
                            
                            // Ruler ticks and labels
                            TimeRuler {
                                duration: duration,
                                zoom: zoom,
                                scroll_offset: scroll_offset,
                            }
                            
                            // Playhead indicator on ruler
                            if playhead_x >= 0.0 {
                                div {
                                    style: "
                                        position: absolute;
                                        left: {playhead_x}px;
                                        top: 0;
                                        width: 1px;
                                        height: 100%;
                                        background-color: #ef4444;
                                        pointer-events: none;
                                    ",
                                }
                                // Playhead handle (triangle)
                                div {
                                    style: "
                                        position: absolute;
                                        left: {playhead_x - 5.0}px;
                                        top: 0;
                                        width: 0;
                                        height: 0;
                                        border-left: 6px solid transparent;
                                        border-right: 6px solid transparent;
                                        border-top: 8px solid #ef4444;
                                        cursor: ew-resize;
                                    ",
                                    onmousedown: move |e| {
                                        e.prevent_default();
                                        e.stop_propagation();
                                        on_seek_start.call(e);
                                    },
                                    onclick: move |e| e.stop_propagation(),
                                }
                            }
                        }
                    }
                    
                    // Tracks area - single scroll container for synced scrolling
                    div {
                        style: "flex: 1; display: flex; overflow: auto;",
                        
                        // Inner wrapper to keep labels and content side by side
                        div {
                            style: "display: flex; min-width: {content_width}px;",
                            
                            // Track labels (inside scroll container so they scroll vertically together)
                            div {
                                style: "
                                    width: {track_label_width}px; 
                                    min-width: {track_label_width}px; 
                                    background-color: {BG_ELEVATED}; 
                                    border-right: 1px solid {BORDER_DEFAULT};
                                    position: sticky;
                                    left: 0;
                                    z-index: 5;
                                ",
                                TrackLabel { name: "Audio", color: ACCENT_AUDIO }
                                TrackLabel { name: "Markers", color: ACCENT_MARKER }
                                TrackLabel { name: "Keyframes", color: ACCENT_KEYFRAME }
                                TrackLabel { name: "Video 1", color: ACCENT_VIDEO }
                            }
                            
                            // Track content area with playhead
                            div {
                                style: "flex: 1; position: relative; display: flex; flex-direction: column; background-color: {BG_BASE};",
                                
                                // Track rows container
                                div {
                                    style: "display: flex; flex-direction: column;",
                                    TrackRow { width: content_width }
                                    TrackRow { width: content_width }
                                    TrackRow { width: content_width }
                                    TrackRow { width: content_width }
                                }
                                
                                // Playhead line overlaying tracks (use large min-height to always extend full height)
                                if playhead_x >= 0.0 {
                                    div {
                                        style: "
                                            position: absolute;
                                            left: {playhead_x}px;
                                            top: 0;
                                            width: 1px;
                                            min-height: 1000px;
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
}

/// Time ruler with tick marks and labels
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
    
    // Generate tick positions
    let num_ticks = (duration / seconds_per_major_tick).ceil() as i32 + 1;
    
    rsx! {
        div {
            style: "position: absolute; left: 0; top: 0; width: 100%; height: 100%;",
            
            for i in 0..num_ticks {
                {
                    let t = i as f64 * seconds_per_major_tick;
                    let x = (t * zoom) - scroll_offset;
                    let minutes = t as i32 / 60;
                    let seconds = t as i32 % 60;
                    let label = format!("{}:{:02}", minutes, seconds);
                    
                    if x >= -50.0 && x <= 2000.0 {  // Only render visible ticks
                        rsx! {
                            // Major tick
                            div {
                                key: "tick-{i}",
                                style: "
                                    position: absolute;
                                    left: {x}px;
                                    bottom: 0;
                                    width: 1px;
                                    height: 10px;
                                    background-color: {BORDER_STRONG};
                                ",
                            }
                            // Label
                            div {
                                key: "label-{i}",
                                style: "
                                    position: absolute;
                                    left: {x + 4.0}px;
                                    top: 3px;
                                    font-size: 9px;
                                    color: {TEXT_DIM};
                                    font-family: 'SF Mono', Consolas, monospace;
                                    user-select: none;
                                ",
                                "{label}"
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
pub fn TrackLabel(name: &'static str, color: &'static str) -> Element {
    rsx! {
        div {
            style: "display: flex; align-items: center; gap: 10px; height: 36px; padding: 0 12px; border-bottom: 1px solid {BORDER_SUBTLE}; font-size: 12px; color: {TEXT_SECONDARY};",
            div { style: "width: 3px; height: 16px; border-radius: 2px; background-color: {color};" }
            span { "{name}" }
        }
    }
}

/// Track row content area
#[component]
pub fn TrackRow(width: i32) -> Element {
    rsx! {
        div { 
            style: "height: 36px; min-width: {width}px; border-bottom: 1px solid {BORDER_SUBTLE}; background-color: {BG_BASE};" 
        }
    }
}
