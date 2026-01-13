use dioxus::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::constants::{
    ACCENT_VIDEO,
    BG_ELEVATED,
    BORDER_ACCENT,
    BORDER_DEFAULT,
    BORDER_SUBTLE,
    TEXT_PRIMARY,
    TIMELINE_SNAP_THRESHOLD_PX,
};
use crate::core::timeline_snap::{best_snap_delta_frames, frames_from_seconds, seconds_from_frames, SnapTarget};
use crate::core::audio::cache::{cache_matches_source, load_peak_cache, peak_cache_path, PeakCache};
use crate::core::audio::waveform::{build_and_store_peak_cache, resolve_audio_source, PeakBuildConfig};

use image::codecs::bmp::BmpEncoder;
use image::{ColorType, ImageEncoder};

use super::{MAX_THUMB_TILES, MIN_CLIP_WIDTH_FLOOR_PX, MIN_CLIP_WIDTH_PX, MIN_CLIP_WIDTH_SCALE, THUMB_TILE_WIDTH_PX};

/// Interactive clip element with drag, resize, and context menu support
#[component]
pub(crate) fn ClipElement(
    clip: crate::state::Clip,
    assets: Vec<crate::state::Asset>,
    thumbnailer: std::sync::Arc<crate::core::thumbnailer::Thumbnailer>,
    thumbnail_cache_buster: u64,
    clip_cache_buckets: std::sync::Arc<HashMap<uuid::Uuid, Vec<bool>>>,
    project_root: Option<std::path::PathBuf>,
    audio_waveform_cache_buster: Signal<u64>,
    zoom: f64,
    fps: f64,
    clip_color: &'static str,
    on_delete: EventHandler<uuid::Uuid>,
    on_move: EventHandler<(uuid::Uuid, f64)>,
    on_resize: EventHandler<(uuid::Uuid, f64, f64)>,  // (id, new_start, new_duration)
    on_move_track: EventHandler<(uuid::Uuid, i32)>,
    is_selected: bool,
    on_select: EventHandler<uuid::Uuid>,
    on_snap_preview: EventHandler<Option<f64>>,
    snap_targets: std::sync::Arc<Vec<SnapTarget>>,
) -> Element {
    let mut show_menu = use_signal(|| false);
    let mut menu_pos = use_signal(|| (0.0, 0.0));
    let mut drag_mode = use_signal(|| None::<&'static str>);  // None, "move", "resize-left", "resize-right"
    let mut drag_start_x = use_signal(|| 0.0);
    let mut drag_start_time = use_signal(|| 0.0);
    let mut drag_start_duration = use_signal(|| 0.0);
    let mut drag_start_end_time = use_signal(|| 0.0);
    let mut drag_start_offset = use_signal(|| 0.0);
    let fps = fps.max(1.0);

    let left = (clip.start_time * zoom) as i32;
    let min_clip_width = (zoom * MIN_CLIP_WIDTH_SCALE)
        .clamp(MIN_CLIP_WIDTH_FLOOR_PX, MIN_CLIP_WIDTH_PX);
    let clip_width = (clip.duration * zoom).max(min_clip_width) as i32;
    let clip_width_f = clip_width as f64;
    let clip_id = clip.id;
    let filtered_snap_targets: Vec<SnapTarget> = snap_targets
        .iter()
        .copied()
        .filter(|target| target.clip_id != Some(clip_id))
        .collect();
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
    let is_audio = asset.map(|a| a.is_audio()).unwrap_or(false);
    let has_source_trim = asset
        .map(|a| a.is_video() || a.is_audio())
        .unwrap_or(false);
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

    let mut waveform_cache = use_signal(|| None::<PeakCache>);
    let mut waveform_building = use_signal(|| false);
    let waveform_cache_buster = audio_waveform_cache_buster;
    let mut waveform_last_buster = use_signal(|| 0_u64);
    let mut waveform_bitmap_cache = use_signal(|| None::<(WaveformKey, String)>);

    let waveform_buster_value = waveform_cache_buster();
    if is_audio {
        if let (Some(project_root), Some(asset)) = (project_root.clone(), asset.clone()) {
            let asset_id = asset.id;
            let cache_path = peak_cache_path(&project_root, asset_id);
            let source_path = resolve_audio_source(&project_root, &asset);

            if waveform_last_buster() != waveform_buster_value {
                let mut loaded = None;
                if let Some(source_path) = source_path.as_ref() {
                    if cache_path.exists() {
                        match load_peak_cache(&cache_path)
                            .and_then(|cache| {
                                if cache_matches_source(&cache, source_path)? {
                                    Ok(Some(cache))
                                } else {
                                    Ok(None)
                                }
                            }) {
                            Ok(cache) => {
                                loaded = cache;
                            }
                            Err(_err) => {}
                        }
                    }
                }
                waveform_cache.set(loaded);
                if waveform_bitmap_cache().is_some() {
                    waveform_bitmap_cache.set(None);
                }
                waveform_last_buster.set(waveform_buster_value);
            }

            if waveform_cache().is_none() && !waveform_building() {
                if let Some(source_path) = source_path {
                    waveform_building.set(true);
                    let mut waveform_cache = waveform_cache.clone();
                    let mut waveform_building = waveform_building.clone();
                    let mut waveform_cache_buster = waveform_cache_buster.clone();
                    let project_root_for_build = project_root.clone();
                    let source_path_for_build = source_path.clone();
                    spawn(async move {
                        let build_result = tokio::task::spawn_blocking(move || {
                            build_and_store_peak_cache(
                                &project_root_for_build,
                                asset_id,
                                &source_path_for_build,
                                PeakBuildConfig::default(),
                            )
                        })
                        .await
                        .ok()
                        .and_then(|res| res.ok());

                        waveform_building.set(false);
                        if let Some(cache_path) = build_result {
                            let load_path = cache_path.clone();
                            if let Ok(cache) =
                                tokio::task::spawn_blocking(move || load_peak_cache(&load_path))
                                    .await
                                    .ok()
                                    .unwrap_or_else(|| Err("Waveform cache load failed".to_string()))
                            {
                                waveform_cache.set(Some(cache));
                                waveform_cache_buster
                                    .set(waveform_cache_buster() + 1);
                                waveform_bitmap_cache.set(None);
                            }
                        }
                    });
                }
            }
        }
    } else if waveform_cache().is_some() {
        waveform_cache.set(None);
        if waveform_bitmap_cache().is_some() {
            waveform_bitmap_cache.set(None);
        }
    }

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
    let snap_targets = filtered_snap_targets.clone();
    
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

            if is_audio {
                {
                    let mut waveform_url = String::new();
                    if let Some(cache) = waveform_cache().as_ref() {
                        let base_block = cache
                            .levels
                            .first()
                            .map(|level| level.block_size.max(1))
                            .unwrap_or(1);
                        let max_columns = ((clip.duration.max(0.0) * cache.sample_rate as f64)
                            / base_block as f64)
                            .ceil() as usize;
                        let mut render_width = clip_width.max(1) as usize;
                        render_width = render_width.min(WAVEFORM_MAX_WIDTH_PX).min(max_columns.max(1));

                        let key = WaveformKey {
                            buster: waveform_buster_value,
                            width: render_width,
                            zoom_bits: zoom.to_bits(),
                            trim_bits: trim_in_seconds.to_bits(),
                            duration_bits: clip.duration.to_bits(),
                        };

                        let mut needs_rebuild = true;
                        if let Some((cached_key, cached_url)) = waveform_bitmap_cache().as_ref() {
                            if *cached_key == key {
                                waveform_url = cached_url.clone();
                                needs_rebuild = false;
                            }
                        }

                        if needs_rebuild {
                            if let Some(project_root) = project_root.as_ref() {
                                let bmp_path = waveform_bmp_cache_path(
                                    project_root,
                                    clip.asset_id,
                                    &key,
                                    WAVEFORM_BMP_HEIGHT_PX,
                                );
                                let bmp_url = crate::utils::get_local_file_url(&bmp_path);

                                if bmp_path.exists() {
                                    waveform_url = bmp_url.clone();
                                    waveform_bitmap_cache.set(Some((key, bmp_url)));
                                } else {
                                    let columns_start = Instant::now();
                                    let columns = waveform_columns_for_clip(
                                        cache,
                                        clip.duration,
                                        trim_in_seconds,
                                        render_width,
                                    );
                                    let columns_elapsed = columns_start.elapsed();

                                    let bitmap_start = Instant::now();
                                    let bitmap = waveform_bitmap_from_columns(
                                        &columns,
                                        render_width,
                                        WAVEFORM_BMP_HEIGHT_PX,
                                    );
                                    let bitmap_elapsed = bitmap_start.elapsed();

                                    match write_waveform_bmp(
                                        &bmp_path,
                                        clip.asset_id,
                                        render_width,
                                        WAVEFORM_BMP_HEIGHT_PX,
                                        &bitmap,
                                    ) {
                                        Ok((_encode_ms, _write_ms, _byte_len)) => {
                                            waveform_url = bmp_url.clone();
                                            waveform_bitmap_cache.set(Some((key, bmp_url)));
                                            let _ = (columns_elapsed, bitmap_elapsed);
                                        }
                                        Err(err) => {
                                            eprintln!(
                                                "[AUDIO WARN] Waveform bmp write failed: asset_id={} err={}",
                                                clip.asset_id, err
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if !waveform_url.is_empty() {
                        rsx! {
                            img {
                                style: "
                                    position: absolute; left: 0; right: 0; top: 0; bottom: 0;
                                    width: 100%; height: 100%;
                                    pointer-events: none; z-index: 0;
                                    opacity: 0.6;
                                ",
                                src: "{waveform_url}",
                                draggable: "false",
                            }
                        }
                    } else {
                        rsx! {}
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
                            drag_start_offset.set(e.element_coordinates().x);
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
                    let delta_frames = (delta_x / zoom) * fps;
                    let snap_enabled = !e.modifiers().alt();
                    let snap_threshold_frames = if zoom > 0.0 {
                        (TIMELINE_SNAP_THRESHOLD_PX / zoom) * fps
                    } else {
                        0.0
                    };
                    let min_duration_frames = (0.1 * fps).ceil().max(1.0);
                    
                    match drag_mode() {
                        Some("move") => {
                            let start_frames = frames_from_seconds(drag_start_time(), fps).round();
                            let duration_frames = frames_from_seconds(current_duration, fps).round();
                            let mut new_start_frames = start_frames + delta_frames;
                            let start_hit = if snap_enabled {
                                best_snap_delta_frames(
                                    &[new_start_frames],
                                    &snap_targets,
                                    snap_threshold_frames,
                                )
                            } else {
                                None
                            };
                            let end_hit = if snap_enabled {
                                best_snap_delta_frames(
                                    &[new_start_frames + duration_frames],
                                    &snap_targets,
                                    snap_threshold_frames,
                                )
                            } else {
                                None
                            };
                            let prefer_start = drag_start_offset() <= (clip_width_f / 2.0);
                            let epsilon = 1e-4;
                            let chosen_hit = match (start_hit, end_hit) {
                                (Some(start_hit), Some(end_hit)) => {
                                    let dist_start = start_hit.delta_frames.abs();
                                    let dist_end = end_hit.delta_frames.abs();
                                    let prio_start = start_hit.target.kind.priority();
                                    let prio_end = end_hit.target.kind.priority();
                                    if dist_start + epsilon < dist_end {
                                        Some(start_hit)
                                    } else if dist_end + epsilon < dist_start {
                                        Some(end_hit)
                                    } else if prio_start != prio_end {
                                        if prio_start > prio_end {
                                            Some(start_hit)
                                        } else {
                                            Some(end_hit)
                                        }
                                    } else if prefer_start {
                                        Some(start_hit)
                                    } else {
                                        Some(end_hit)
                                    }
                                }
                                (Some(hit), None) | (None, Some(hit)) => Some(hit),
                                _ => None,
                            };
                            let mut snap_target_frame = None;
                            if let Some(hit) = chosen_hit {
                                new_start_frames += hit.delta_frames;
                                snap_target_frame = Some(hit.target.frame);
                            }
                            new_start_frames = new_start_frames.max(0.0);
                            let snapped_start_frames = new_start_frames.round().max(0.0);
                            let snapped_start = seconds_from_frames(snapped_start_frames, fps);
                            on_move.call((clip_id, snapped_start));
                            let mut snap_preview = None;
                            if snap_enabled {
                                if let Some(target_frame) = snap_target_frame {
                                    let matches_start =
                                        (snapped_start_frames - target_frame).abs() <= 0.5;
                                    let matches_end = (snapped_start_frames + duration_frames
                                        - target_frame)
                                        .abs()
                                        <= 0.5;
                                    if matches_start || matches_end {
                                        snap_preview = Some(seconds_from_frames(target_frame, fps));
                                    }
                                }
                            }
                            if !snap_enabled {
                                snap_preview = None;
                            }
                            on_snap_preview.call(snap_preview);
                        }
                        Some("resize-left") => {
                            // Moving left edge: keep right edge fixed while clamping to source duration
                            let end_frames = frames_from_seconds(drag_start_end_time(), fps).round();
                            let min_start_frames = if has_source_trim {
                                frames_from_seconds(
                                    (current_start - trim_in_seconds).max(0.0),
                                    fps,
                                )
                                .round()
                            } else {
                                0.0
                            };
                            let mut new_start_frames =
                                frames_from_seconds(drag_start_time(), fps).round() + delta_frames;
                            let mut snap_target_frame = None;
                            if snap_enabled {
                                if let Some(hit) = best_snap_delta_frames(
                                    &[new_start_frames],
                                    &snap_targets,
                                    snap_threshold_frames,
                                ) {
                                    new_start_frames += hit.delta_frames;
                                    snap_target_frame = Some(hit.target.frame);
                                }
                            }
                            new_start_frames = new_start_frames.max(min_start_frames);
                            let mut new_duration_frames = end_frames - new_start_frames;
                            if let Some(max_duration) = max_duration {
                                let max_duration_frames =
                                    frames_from_seconds(max_duration, fps).round();
                                if new_duration_frames > max_duration_frames {
                                    new_duration_frames = max_duration_frames;
                                    new_start_frames = end_frames - new_duration_frames;
                                }
                            }
                            if new_duration_frames < min_duration_frames {
                                new_duration_frames = min_duration_frames;
                                new_start_frames = end_frames - new_duration_frames;
                            }
                            let snapped_start_frames = new_start_frames.round().max(0.0);
                            let snapped_start = seconds_from_frames(snapped_start_frames, fps);
                            let snapped_duration_frames =
                                (end_frames - snapped_start_frames).max(min_duration_frames);
                            let snapped_duration =
                                seconds_from_frames(snapped_duration_frames, fps);
                            on_resize.call((clip_id, snapped_start, snapped_duration));
                            let mut snap_preview = None;
                            if snap_enabled {
                                if let Some(target_frame) = snap_target_frame {
                                    if (snapped_start_frames - target_frame).abs() <= 0.5 {
                                        snap_preview = Some(seconds_from_frames(target_frame, fps));
                                    }
                                }
                            }
                            on_snap_preview.call(snap_preview);
                        }
                        Some("resize-right") => {
                            // Moving right edge: only changes duration, clamped to source duration
                            let start_frames = frames_from_seconds(drag_start_time(), fps).round();
                            let mut new_end_frames = start_frames
                                + frames_from_seconds(drag_start_duration(), fps).round()
                                + delta_frames;
                            let mut snap_target_frame = None;
                            if snap_enabled {
                                if let Some(hit) = best_snap_delta_frames(
                                    &[new_end_frames],
                                    &snap_targets,
                                    snap_threshold_frames,
                                ) {
                                    new_end_frames += hit.delta_frames;
                                    snap_target_frame = Some(hit.target.frame);
                                }
                            }
                            let mut new_duration_frames =
                                (new_end_frames - start_frames).max(min_duration_frames);
                            if let Some(available_duration) = available_duration {
                                let available_frames =
                                    frames_from_seconds(available_duration, fps).round();
                                if new_duration_frames > available_frames {
                                    new_duration_frames = available_frames;
                                }
                            }
                            let snapped_duration_frames =
                                new_duration_frames.round().max(min_duration_frames);
                            let snapped_duration =
                                seconds_from_frames(snapped_duration_frames, fps);
                            let snapped_start = seconds_from_frames(start_frames, fps);
                            on_resize.call((clip_id, snapped_start, snapped_duration));
                            let mut snap_preview = None;
                            if snap_enabled {
                                if let Some(target_frame) = snap_target_frame {
                                    let snapped_end_frames = start_frames + snapped_duration_frames;
                                    if (snapped_end_frames - target_frame).abs() <= 0.5 {
                                        snap_preview = Some(seconds_from_frames(target_frame, fps));
                                    }
                                }
                            }
                            on_snap_preview.call(snap_preview);
                        }
                        _ => {
                            on_snap_preview.call(None);
                        }
                    }
                },
                onmouseup: move |_| {
                    drag_mode.set(None);
                    on_snap_preview.call(None);
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

#[derive(Clone, Copy, Debug, PartialEq)]
struct WaveformKey {
    buster: u64,
    width: usize,
    zoom_bits: u64,
    trim_bits: u64,
    duration_bits: u64,
}

#[derive(Clone, Copy, Debug)]
struct WaveColumn {
    y_top: f32,
    y_bottom: f32,
}

const WAVEFORM_BMP_HEIGHT_PX: usize = 32;
const WAVEFORM_MAX_WIDTH_PX: usize = 60_000;
const WAVEFORM_PIXEL_VALUE: u8 = 160;

fn waveform_columns_for_clip(
    cache: &PeakCache,
    clip_duration: f64,
    trim_in_seconds: f64,
    width_px: usize,
) -> Vec<WaveColumn> {
    let levels = &cache.levels;
    if levels.is_empty() || width_px == 0 {
        return Vec::new();
    }

    let sample_rate = cache.sample_rate as f64;
    let level = &levels[0];

    let clip_duration = clip_duration.max(0.0);
    let trim_in_seconds = trim_in_seconds.max(0.0);
    let start_frame = (trim_in_seconds * sample_rate).floor() as usize;
    let end_frame = ((trim_in_seconds + clip_duration) * sample_rate).ceil() as usize;
    if level.block_size == 0 {
        return Vec::new();
    }
    let start_index = start_frame / level.block_size;
    let end_index = (end_frame / level.block_size).min(level.peaks.len());
    if start_index >= end_index {
        return Vec::new();
    }

    let slice = &level.peaks[start_index..end_index];
    let width = width_px.max(1);
    let step = slice.len() as f64 / width as f64;
    let height = 32.0_f32;
    let center = height / 2.0;
    let amp = (height - 6.0) / 2.0;

    let mut columns = Vec::with_capacity(width);
    for x in 0..width {
        let start = (x as f64 * step).floor() as usize;
        let end = ((x + 1) as f64 * step).ceil() as usize;
        if start >= slice.len() {
            continue;
        }
        let end = end.min(slice.len()).max(start + 1);
        let mut min = i16::MAX;
        let mut max = i16::MIN;
        for peak in &slice[start..end] {
            min = min.min(peak.min_l.min(peak.min_r));
            max = max.max(peak.max_l.max(peak.max_r));
        }
        let min = min as f32 / i16::MAX as f32;
        let max = max as f32 / i16::MAX as f32;
        columns.push(WaveColumn {
            y_top: center - max * amp,
            y_bottom: center - min * amp,
        });
    }

    columns
}

fn waveform_bitmap_from_columns(
    columns: &[WaveColumn],
    width: usize,
    height: usize,
) -> Vec<u8> {
    if columns.is_empty() || width == 0 || height == 0 {
        return Vec::new();
    }
    let mut buffer = vec![0_u8; width * height];
    let height_f = height as f32;
    let max_y = height.saturating_sub(1) as i32;

    for (x, column) in columns.iter().enumerate() {
        if x >= width {
            break;
        }
        let mut y_top = column.y_top.clamp(0.0, height_f - 1.0).round() as i32;
        let mut y_bottom = column.y_bottom.clamp(0.0, height_f - 1.0).round() as i32;
        if y_top > y_bottom {
            std::mem::swap(&mut y_top, &mut y_bottom);
        }
        y_top = y_top.clamp(0, max_y);
        y_bottom = y_bottom.clamp(0, max_y);
        let base = x;
        for y in y_top..=y_bottom {
            buffer[y as usize * width + base] = WAVEFORM_PIXEL_VALUE;
        }
    }

    buffer
}

fn waveform_bmp_cache_path(
    project_root: &Path,
    asset_id: uuid::Uuid,
    key: &WaveformKey,
    height: usize,
) -> PathBuf {
    let file_name = format!(
        "w{}_h{}_z{:x}_t{:x}_d{:x}_b{:x}.bmp",
        key.width, height, key.zoom_bits, key.trim_bits, key.duration_bits, key.buster
    );
    project_root
        .join(".cache")
        .join("audio")
        .join("waveform_strips")
        .join(asset_id.to_string())
        .join(file_name)
}

fn write_waveform_bmp(
    path: &Path,
    asset_id: uuid::Uuid,
    width: usize,
    height: usize,
    bitmap: &[u8],
) -> Result<(u128, u128, usize), String> {
    if bitmap.is_empty() || width == 0 || height == 0 {
        return Err("Empty bitmap.".to_string());
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }

    let mut bmp_bytes = Vec::new();
    let bmp_encode_start = Instant::now();
    let bmp_result = BmpEncoder::new(&mut bmp_bytes)
        .write_image(bitmap, width as u32, height as u32, ColorType::L8.into());
    let bmp_encode_ms = bmp_encode_start.elapsed().as_millis();

    if bmp_result.is_err() {
        eprintln!(
            "[AUDIO WARN] Waveform bmp encode failed: asset_id={}",
            asset_id
        );
        return Err("BMP encode failed.".to_string());
    }

    let bmp_write_start = Instant::now();
    let bmp_write_result = fs::write(path, &bmp_bytes);
    let bmp_write_ms = bmp_write_start.elapsed().as_millis();

    bmp_write_result.map_err(|err| err.to_string())?;
    Ok((bmp_encode_ms, bmp_write_ms, bmp_bytes.len()))
}

