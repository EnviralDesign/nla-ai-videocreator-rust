use dioxus::prelude::*;

use crate::constants::{ACCENT_MARKER, BG_SURFACE, BORDER_DEFAULT, TEXT_DIM, TEXT_PRIMARY, TIMELINE_SNAP_THRESHOLD_PX};
use crate::core::timeline_snap::{best_snap_delta_frames, frames_from_seconds, seconds_from_frames, SnapTarget};

#[component]
pub fn MarkerElement(
    marker: crate::state::Marker,
    width: i32,
    zoom: f64,
    fps: f64,
    duration: f64,
    is_selected: bool,
    on_select: EventHandler<uuid::Uuid>,
    on_move: EventHandler<(uuid::Uuid, f64)>,
    on_delete: EventHandler<uuid::Uuid>,
    on_snap_preview: EventHandler<Option<f64>>,
    snap_targets: std::sync::Arc<Vec<SnapTarget>>,
) -> Element {
    let fps = fps.max(1.0);
    let width_f = width.max(1) as f64;
    let marker_color = marker
        .color
        .as_deref()
        .unwrap_or(ACCENT_MARKER);
    let marker_id = marker.id;
    let marker_time = marker.time;
    let position = (marker_time * zoom).min(width_f - 1.0).max(0.0);
    let line_width = if is_selected { 2.0 } else { 1.0 };

    let mut drag_active = use_signal(|| false);
    let mut drag_start_x = use_signal(|| 0.0);
    let mut drag_start_time = use_signal(|| marker_time);
    let mut show_menu = use_signal(|| false);
    let mut menu_pos = use_signal(|| (0.0, 0.0));

    let filtered_snap_targets: Vec<SnapTarget> = snap_targets
        .iter()
        .copied()
        .filter(|target| target.marker_id != Some(marker_id))
        .collect();

    rsx! {
        div {
            style: "
                position: absolute;
                left: {position}px;
                top: 0;
                height: 100%;
                transform: translateX(-0.5px);
                cursor: ew-resize;
            ",
            onmousedown: move |e| {
                if let Some(btn) = e.trigger_button() {
                    if format!("{:?}", btn) == "Primary" {
                        e.prevent_default();
                        e.stop_propagation();
                        on_select.call(marker_id);
                        drag_active.set(true);
                        drag_start_x.set(e.client_coordinates().x);
                        drag_start_time.set(marker_time);
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

            // Marker line
            div {
                style: "
                    position: absolute;
                    left: {-(line_width / 2.0)}px;
                    top: 0;
                    width: {line_width}px;
                    height: 100%;
                    background-color: {marker_color};
                    box-shadow: 0 0 0 1px rgba(0,0,0,0.2);
                ",
            }
            // Marker head (triangle)
            div {
                style: "
                    position: absolute;
                    left: -4px;
                    bottom: 0;
                    width: 0;
                    height: 0;
                    border-left: 4px solid transparent;
                    border-right: 4px solid transparent;
                    border-bottom: 6px solid {marker_color};
                ",
            }
            if let Some(label) = marker.label.as_ref().filter(|label| !label.is_empty()) {
                div {
                    style: "
                        position: absolute;
                        left: 6px;
                        top: 2px;
                        max-width: 120px;
                        padding: 2px 6px;
                        font-size: 10px;
                        border-radius: 6px;
                        background-color: {BG_SURFACE};
                        color: {TEXT_PRIMARY};
                        border: 1px solid {BORDER_DEFAULT};
                        white-space: nowrap;
                        overflow: hidden;
                        text-overflow: ellipsis;
                        opacity: 0.9;
                    ",
                    "{label}"
                }
            } else {
                div {
                    style: "
                        position: absolute;
                        left: 6px;
                        top: 3px;
                        font-size: 9px;
                        color: {TEXT_DIM};
                        opacity: 0.6;
                        pointer-events: none;
                    ",
                    ""
                }
            }
        }

        if drag_active() {
            div {
                style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; z-index: 9999; cursor: ew-resize;",
                oncontextmenu: move |e| e.prevent_default(),
                onmousemove: move |e| {
                    let delta_x = e.client_coordinates().x - drag_start_x();
                    let delta_frames = (delta_x / zoom) * fps;
                    let start_frames = frames_from_seconds(drag_start_time(), fps).round();
                    let mut new_frames = start_frames + delta_frames;
                    let snap_enabled = !e.modifiers().alt();
                    let snap_threshold_frames = if zoom > 0.0 {
                        (TIMELINE_SNAP_THRESHOLD_PX / zoom) * fps
                    } else {
                        0.0
                    };
                    if snap_enabled {
                        if let Some(hit) = best_snap_delta_frames(
                            &[new_frames],
                            &filtered_snap_targets,
                            snap_threshold_frames,
                        ) {
                            new_frames += hit.delta_frames;
                            on_snap_preview.call(Some(seconds_from_frames(hit.target.frame, fps)));
                        } else {
                            on_snap_preview.call(None);
                        }
                    } else {
                        on_snap_preview.call(None);
                    }
                    let max_frames = frames_from_seconds(duration, fps).round();
                    let snapped_frames = new_frames.round().clamp(0.0, max_frames);
                    let snapped_time = seconds_from_frames(snapped_frames, fps);
                    on_move.call((marker_id, snapped_time));
                },
                onmouseup: move |_| {
                    drag_active.set(false);
                    on_snap_preview.call(None);
                },
            }
        }

        if show_menu() {
            div {
                style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; z-index: 9998;",
                onclick: move |_| show_menu.set(false),
                oncontextmenu: move |e| {
                    e.prevent_default();
                    show_menu.set(false);
                },
            }
            div {
                style: "
                    position: fixed;
                    left: {menu_pos().0}px;
                    top: {menu_pos().1}px;
                    background-color: {BG_SURFACE};
                    border: 1px solid {BORDER_DEFAULT};
                    border-radius: 6px;
                    padding: 4px 0;
                    min-width: 140px;
                    box-shadow: 0 4px 12px rgba(0,0,0,0.3);
                    z-index: 9999;
                    font-size: 12px;
                ",
                oncontextmenu: move |e| e.prevent_default(),
                div {
                    style: "
                        padding: 6px 12px;
                        color: #ef4444;
                        cursor: pointer;
                        transition: background-color 0.1s ease;
                    ",
                    onclick: move |_| {
                        on_delete.call(marker_id);
                        show_menu.set(false);
                    },
                    "Delete Marker"
                }
            }
        }
    }
}
