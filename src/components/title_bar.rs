use dioxus::prelude::*;
use crate::constants::*;

#[component]
pub fn TitleBar(
    project_name: String,
    on_new_project: EventHandler<MouseEvent>,
    on_save: EventHandler<MouseEvent>,
    on_open_providers: EventHandler<MouseEvent>,
    show_preview_stats: bool,
    on_toggle_preview_stats: EventHandler<MouseEvent>,
    use_hw_decode: bool,
    on_toggle_hw_decode: EventHandler<MouseEvent>,
) -> Element {
    let stats_toggle_bg = if show_preview_stats { BG_HOVER } else { BG_BASE };
    let hw_toggle_bg = if use_hw_decode { BG_HOVER } else { BG_BASE };
    rsx! {
        div {
            style: "
                display: flex; align-items: center; justify-content: space-between;
                height: 40px; padding: 0 16px;
                background-color: {BG_SURFACE}; border-bottom: 1px solid {BORDER_DEFAULT};
                user-select: none;
            ",
            div {
                style: "display: flex; align-items: center; gap: 20px;",
                span { style: "font-size: 13px; font-weight: 600; color: {TEXT_SECONDARY};", "NLA AI Video Creator" }
                button {
                    class: "collapse-btn",
                    style: "
                        background: transparent; border: none; color: {TEXT_PRIMARY};
                        font-size: 12px; cursor: pointer; padding: 4px 8px; border-radius: 4px;
                    ",
                    onclick: move |e| on_new_project.call(e),
                    "New Project"
                }
                button {
                    class: "collapse-btn",
                    style: "
                        background: transparent; border: none; color: {TEXT_PRIMARY};
                        font-size: 12px; cursor: pointer; padding: 4px 8px; border-radius: 4px;
                    ",
                    onclick: move |e| on_save.call(e),
                    "Save"
                }
                button {
                    class: "collapse-btn",
                    style: "
                        background: transparent; border: none; color: {TEXT_PRIMARY};
                        font-size: 12px; cursor: pointer; padding: 4px 8px; border-radius: 4px;
                    ",
                    onclick: move |e| on_open_providers.call(e),
                    "Providers"
                }
            }
            span { style: "font-size: 13px; color: {TEXT_MUTED};", "{project_name}" }
            div {
                style: "display: flex; align-items: center; justify-content: flex-end; gap: 12px; min-width: 220px;",
                div {
                    style: "display: flex; align-items: center; gap: 6px;",
                    span {
                        style: "font-size: 10px; color: {TEXT_DIM}; text-transform: uppercase; letter-spacing: 0.6px;",
                        "Stats"
                    }
                    button {
                        class: "collapse-btn",
                        style: "
                            background: {stats_toggle_bg};
                            border: 1px solid {BORDER_DEFAULT};
                            color: {TEXT_PRIMARY}; font-size: 11px; cursor: pointer;
                            padding: 4px 10px; border-radius: 999px;
                        ",
                        onclick: move |e| on_toggle_preview_stats.call(e),
                        if show_preview_stats { "On" } else { "Off" }
                    }
                }
                div {
                    style: "display: flex; align-items: center; gap: 6px;",
                    span {
                        style: "font-size: 10px; color: {TEXT_DIM}; text-transform: uppercase; letter-spacing: 0.6px;",
                        "HW Dec"
                    }
                    button {
                        class: "collapse-btn",
                        style: "
                            background: {hw_toggle_bg};
                            border: 1px solid {BORDER_DEFAULT};
                            color: {TEXT_PRIMARY}; font-size: 11px; cursor: pointer;
                            padding: 4px 10px; border-radius: 999px;
                        ",
                        onclick: move |e| on_toggle_hw_decode.call(e),
                        if use_hw_decode { "On" } else { "Off" }
                    }
                }
            }
        }
    }
}
