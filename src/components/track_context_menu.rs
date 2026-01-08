use dioxus::prelude::*;

use crate::constants::*;
use crate::state::{Project, SelectionState, TrackType};

#[component]
pub fn TrackContextMenu(
    context_menu: Signal<Option<(f64, f64, uuid::Uuid)>>,
    project: Signal<Project>,
    selection: Signal<SelectionState>,
    preview_dirty: Signal<bool>,
) -> Element {
    rsx! {
        if let Some((x, y, track_id)) = context_menu() {
        // Backdrop to catch clicks outside menu
        div {
            style: "
                position: fixed; top: 0; left: 0; right: 0; bottom: 0;
                z-index: 999;
            ",
            onclick: move |_| context_menu.set(None),
        }
        // The actual menu
        div {
            style: "
                position: fixed;
                left: min({x}px, calc(100vw - 150px));
                top: min({y}px, calc(100vh - 120px));
                background-color: {BG_ELEVATED}; border: 1px solid {BORDER_DEFAULT};
                border-radius: 6px; padding: 4px 0; min-width: 140px;
                box-shadow: 0 4px 12px rgba(0,0,0,0.3);
                z-index: 1000; font-size: 12px;
            ",
            {
                let is_markers = project.read().find_track(track_id)
                    .map(|t| t.track_type == TrackType::Marker)
                    .unwrap_or(false);
                let track_name = project.read().find_track(track_id)
                    .map(|t| t.name.clone())
                    .unwrap_or_default();

                if is_markers {
                    rsx! {
                        div {
                            style: "
                                padding: 6px 12px; color: {TEXT_DIM};
                                cursor: not-allowed;
                            ",
                            "Cannot delete Markers track"
                        }
                    }
                } else {
                    rsx! {
                        div {
                            style: "
                                padding: 6px 12px; color: #ef4444; cursor: pointer;
                                transition: background-color 0.1s ease;
                            ",
                            onmouseenter: move |_| {},
                            onclick: move |_| {
                                project.write().remove_track(track_id);
                                selection.write().clear();
                                preview_dirty.set(true);
                                context_menu.set(None);
                            },
                            "🗑 Delete \"{track_name}\""
                        }

                        div {
                            style: "height: 1px; background-color: {BORDER_SUBTLE}; margin: 2px 0;",
                        }

                        div {
                            style: "
                                padding: 6px 12px; color: {TEXT_PRIMARY}; cursor: pointer;
                                transition: background-color 0.1s ease;
                            ",
                            onmouseenter: move |_| {},
                            onclick: move |_| {
                                project.write().move_track_up(track_id);
                                preview_dirty.set(true);
                                context_menu.set(None);
                            },
                            "↑ Move Up"
                        }

                        div {
                            style: "
                                padding: 6px 12px; color: {TEXT_PRIMARY}; cursor: pointer;
                                transition: background-color 0.1s ease;
                            ",
                            onmouseenter: move |_| {},
                            onclick: move |_| {
                                project.write().move_track_down(track_id);
                                preview_dirty.set(true);
                                context_menu.set(None);
                            },
                            "↓ Move Down"
                        }
                    }
                }
            }
        }
        }
    }
}
