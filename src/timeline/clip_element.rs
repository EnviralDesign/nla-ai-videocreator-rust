use dioxus::prelude::*;
use std::collections::HashMap;

use crate::constants::{ACCENT_VIDEO, BG_ELEVATED, BORDER_ACCENT, BORDER_DEFAULT, BORDER_SUBTLE, TEXT_PRIMARY};

use super::{MAX_THUMB_TILES, MIN_CLIP_WIDTH_FLOOR_PX, MIN_CLIP_WIDTH_PX, MIN_CLIP_WIDTH_SCALE, THUMB_TILE_WIDTH_PX};

/// Interactive clip element with drag, resize, and context menu support
#[component]
pub(crate) fn ClipElement(
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

