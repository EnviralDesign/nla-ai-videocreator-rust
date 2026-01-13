use dioxus::prelude::*;

use crate::components::common::StableNumberInput;
use crate::constants::*;

#[component]
pub fn GenerativeVideoModal(
    open: bool,
    fps_value: String,
    frame_count_value: String,
    duration_label: String,
    error: Option<String>,
    on_change_fps: EventHandler<String>,
    on_change_frame_count: EventHandler<String>,
    on_cancel: EventHandler<MouseEvent>,
    on_create: EventHandler<MouseEvent>,
) -> Element {
    if !open {
        return rsx! {};
    }

    rsx! {
        div {
            style: "
                position: fixed; inset: 0;
                background: rgba(0, 0, 0, 0.45);
                backdrop-filter: blur(6px);
                -webkit-backdrop-filter: blur(6px);
                z-index: 140;
            ",
            onclick: move |e| on_cancel.call(e),
        }
        div {
            style: "
                position: fixed; top: 50%; left: 50%;
                transform: translate(-50%, -50%);
                width: 360px;
                padding: 14px;
                background-color: {BG_ELEVATED};
                border: 1px solid {BORDER_DEFAULT};
                border-radius: 10px;
                box-shadow: 0 14px 30px rgba(0,0,0,0.45);
                display: flex; flex-direction: column; gap: 12px;
                z-index: 141;
            ",
            onclick: move |e| e.stop_propagation(),
            div {
                style: "display: flex; flex-direction: column; gap: 4px;",
                span { style: "font-size: 13px; color: {TEXT_PRIMARY};", "New Generative Video" }
                span { style: "font-size: 10px; color: {TEXT_DIM};", "Define the target duration for this asset." }
            }
            div {
                style: "display: grid; grid-template-columns: 1fr 1fr; gap: 10px;",
                div {
                    style: "display: flex; flex-direction: column; gap: 6px;",
                    span { style: "font-size: 10px; color: {TEXT_MUTED};", "FPS" }
                    StableNumberInput {
                        id: "gen-video-fps".to_string(),
                        value: fps_value,
                        placeholder: Some("16".to_string()),
                        style: Some(format!("
                            width: 100%; padding: 6px 8px; font-size: 11px;
                            background-color: {}; color: {};
                            border: 1px solid {}; border-radius: 6px;
                        ", BG_SURFACE, TEXT_PRIMARY, BORDER_DEFAULT)),
                        min: Some("1".to_string()),
                        max: None,
                        step: Some("0.1".to_string()),
                        on_change: move |v: String| on_change_fps.call(v),
                        on_blur: move |_| {},
                        on_keydown: move |_| {},
                    }
                }
                div {
                    style: "display: flex; flex-direction: column; gap: 6px;",
                    span { style: "font-size: 10px; color: {TEXT_MUTED};", "Frame Count" }
                    StableNumberInput {
                        id: "gen-video-frames".to_string(),
                        value: frame_count_value,
                        placeholder: Some("81".to_string()),
                        style: Some(format!("
                            width: 100%; padding: 6px 8px; font-size: 11px;
                            background-color: {}; color: {};
                            border: 1px solid {}; border-radius: 6px;
                        ", BG_SURFACE, TEXT_PRIMARY, BORDER_DEFAULT)),
                        min: Some("1".to_string()),
                        max: None,
                        step: Some("1".to_string()),
                        on_change: move |v: String| on_change_frame_count.call(v),
                        on_blur: move |_| {},
                        on_keydown: move |_| {},
                    }
                }
            }
            div {
                style: "display: flex; align-items: center; justify-content: space-between;",
                span { style: "font-size: 10px; color: {TEXT_DIM};", "Duration" }
                span { style: "font-size: 11px; color: {TEXT_PRIMARY};", "{duration_label}" }
            }
            if let Some(message) = error.as_ref() {
                div { style: "font-size: 10px; color: #fca5a5;", "{message}" }
            }
            div {
                style: "display: flex; justify-content: flex-end; gap: 8px;",
                button {
                    class: "collapse-btn",
                    style: "
                        padding: 6px 10px; font-size: 11px;
                        background-color: {BG_SURFACE};
                        border: 1px solid {BORDER_DEFAULT};
                        border-radius: 6px; color: {TEXT_PRIMARY};
                        cursor: pointer;
                    ",
                    onclick: move |e| on_cancel.call(e),
                    "Cancel"
                }
                button {
                    class: "collapse-btn",
                    style: "
                        padding: 6px 12px; font-size: 11px;
                        background-color: {ACCENT_PRIMARY};
                        border: none; border-radius: 6px;
                        color: white; font-weight: 600;
                        cursor: pointer;
                    ",
                    onclick: move |e| on_create.call(e),
                    "Create"
                }
            }
        }
    }
}
