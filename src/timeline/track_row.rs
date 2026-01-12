use dioxus::prelude::*;
use std::collections::HashMap;

use crate::constants::{ACCENT_AUDIO, ACCENT_MARKER, ACCENT_VIDEO, BG_BASE, BG_HOVER, BORDER_SUBTLE};
use crate::state::TrackType;

use super::clip_element::ClipElement;

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
    project_root: Option<std::path::PathBuf>,
    audio_waveform_cache_buster: Signal<u64>,
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
                    project_root: project_root.clone(),
                    audio_waveform_cache_buster: audio_waveform_cache_buster,
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

