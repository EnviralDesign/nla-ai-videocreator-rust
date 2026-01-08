use dioxus::prelude::*;
use crate::constants::*;

#[component]
pub fn StatusBar() -> Element {
    rsx! {
        div {
            style: "display: flex; align-items: center; justify-content: space-between; height: 22px; padding: 0 14px; background-color: {BG_SURFACE}; border-top: 1px solid {BORDER_DEFAULT}; font-size: 11px; color: {TEXT_DIM};",
            span { "Ready" }
            div {
                style: "display: flex; gap: 16px; font-family: 'SF Mono', Consolas, monospace;",
                span { "60 fps" }
                span { "00:00 / 00:00" }
            }
        }
    }
}
