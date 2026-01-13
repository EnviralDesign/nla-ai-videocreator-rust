use dioxus::prelude::*;
use std::collections::HashMap;

use crate::constants::{
    BG_ELEVATED, BG_SURFACE,
    BORDER_DEFAULT, BORDER_SUBTLE,
    TEXT_DIM, TEXT_MUTED,
    ACCENT_AUDIO, ACCENT_MARKER, ACCENT_VIDEO,
};
use crate::state::{Track, TrackType};
use crate::core::timeline_snap::{snap_time_to_frame, SnapTarget};

use super::playback_controls::PlaybackBtn;
use super::ruler::TimeRuler;
use super::track_label::TrackLabel;
use super::track_row::TrackRow;

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
    project_root: Option<std::path::PathBuf>,
    audio_waveform_cache_buster: Signal<u64>,
    // Timeline state
    current_time: f64,
    duration: f64,
    fps: f64,
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
    selected_tracks: Vec<uuid::Uuid>,
    on_track_select: EventHandler<uuid::Uuid>,
    // Clip operations
    on_clip_delete: EventHandler<uuid::Uuid>,
    on_clip_move: EventHandler<(uuid::Uuid, f64)>,  // (clip_id, new_start_time)
    on_clip_resize: EventHandler<(uuid::Uuid, f64, f64)>,  // (clip_id, new_start, new_duration)
    on_clip_move_track: EventHandler<(uuid::Uuid, i32)>, // (clip_id, direction)
    selected_clips: Vec<uuid::Uuid>,
    on_clip_select: EventHandler<uuid::Uuid>,
    snap_targets: std::sync::Arc<Vec<SnapTarget>>,
    // Asset Drag & Drop
    dragged_asset: Option<uuid::Uuid>,
    on_asset_drop: EventHandler<(uuid::Uuid, f64, uuid::Uuid)>, // (track_id, time, asset_id)
    // Selection
    on_deselect_all: EventHandler<MouseEvent>,
) -> Element {
    let _ = thumbnail_refresh_tick;
    let fps = fps.max(1.0);
    let fps_i = fps.round().max(1.0) as u64;
    let mut snap_indicator_time = use_signal(|| None::<f64>);
    let icon = if collapsed { "▲" } else { "▼" };
    let play_icon = if is_playing { "⏸" } else { "▶" };
    
    // Only apply transition when NOT resizing
    let transition = if is_resizing { "none" } else { "height 0.2s ease, min-height 0.2s ease" };
    
    // Cursor for collapsed header
    let header_cursor = if collapsed { "pointer" } else { "default" };
    let header_class = if collapsed { "collapsed-rail" } else { "" };
    
    // Format time as HH:MM:SS:FF using project fps.
    let format_time = |t: f64| -> String {
        let total_frames = (t * fps).round().max(0.0) as u64;
        let frames = total_frames % fps_i.max(1);
        let total_seconds = total_frames / fps_i.max(1);
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
    let playhead_time = snap_time_to_frame(current_time, fps);
    let playhead_pos = (playhead_time * zoom).min(content_width_f - 1.0).max(0.0);
    let snap_indicator_pos = snap_indicator_time().map(|snap_time| {
        let snap_time = snap_time_to_frame(snap_time, fps);
        (snap_time * zoom).min(content_width_f - 1.0).max(0.0)
    });
    
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
                                                selected: selected_tracks.contains(&tid),
                                                on_select: move |id| on_track_select.call(id),
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
                                    let snapped = snap_time_to_frame(t, fps).clamp(0.0, duration);
                                    on_seek.call(snapped);
                                    // Start drag mode so continued mouse movement continues seeking
                                    on_seek_start.call(e);
                                },
                                
                                // Ruler ticks and labels (positioned in scroll space)
                                TimeRuler {
                                    duration: duration,
                                    zoom: zoom,
                                    scroll_offset: 0.0,  // No offset - we're in scroll space
                                    fps: fps,
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
                                if let Some(snap_pos) = snap_indicator_pos {
                                    div {
                                        style: "
                                            position: absolute;
                                            left: {snap_pos}px;
                                            top: 0;
                                            width: 1px;
                                            height: 100%;
                                            background-color: rgba(250, 204, 21, 0.5);
                                            pointer-events: none;
                                        ",
                                    }
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
                                        project_root: project_root.clone(),
                                        audio_waveform_cache_buster: audio_waveform_cache_buster,
                                        zoom: zoom,
                                        fps: fps,
                                        on_clip_delete: move |id| on_clip_delete.call(id),
                                        on_clip_move: move |(id, time)| on_clip_move.call((id, time)),
                                        on_clip_resize: move |(id, start, dur)| on_clip_resize.call((id, start, dur)),
                                        on_clip_move_track: move |(id, direction)| on_clip_move_track.call((id, direction)),
                                        selected_clips: selected_clips.clone(),
                                        on_clip_select: move |id| on_clip_select.call(id),
                                        on_snap_preview: move |time| snap_indicator_time.set(time),
                                        snap_targets: snap_targets.clone(),
                                        dragged_asset: dragged_asset,
                                        on_asset_drop: move |(tid, t, aid)| on_asset_drop.call((tid, t, aid)),
                                        on_deselect_all: move |e| on_deselect_all.call(e),
                                    }
                                }
                                
                                if let Some(snap_pos) = snap_indicator_pos {
                                    div {
                                        style: "
                                            position: absolute;
                                            left: {snap_pos}px;
                                            top: 0;
                                            width: 1px;
                                            height: 100%;
                                            background-color: rgba(250, 204, 21, 0.5);
                                            pointer-events: none;
                                            z-index: 9;
                                        ",
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

