use dioxus::prelude::*;
use crate::constants::{BORDER_SUBTLE, TEXT_SECONDARY};

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
