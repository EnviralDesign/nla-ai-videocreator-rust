use dioxus::prelude::*;
use crate::constants::{BG_HOVER, TEXT_MUTED};

/// Playback button
#[component]
pub(crate) fn PlaybackBtn(
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

