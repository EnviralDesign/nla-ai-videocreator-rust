use dioxus::prelude::*;

use crate::constants::*;
use crate::core::provider_store::read_provider_file;

#[component]
pub fn ProvidersModal(
    show: Signal<bool>,
    provider_files: Signal<Vec<std::path::PathBuf>>,
    provider_editor_path: Signal<Option<std::path::PathBuf>>,
    provider_editor_text: Signal<String>,
    provider_editor_error: Signal<Option<String>>,
    provider_editor_dirty: Signal<bool>,
    providers_root_label: String,
    provider_save_label: String,
    provider_build_label: String,
    provider_selected_label: String,
    on_provider_new: EventHandler<MouseEvent>,
    on_provider_reload: EventHandler<MouseEvent>,
    on_provider_build: EventHandler<MouseEvent>,
    on_provider_save: EventHandler<MouseEvent>,
    on_provider_delete: EventHandler<MouseEvent>,
) -> Element {
    let mut editor_focused = use_signal(|| false);
    rsx! {
        if !show() {
            div {}
        } else {
        // Backdrop
        div {
            style: "
                position: fixed; top: 0; left: 0; right: 0; bottom: 0;
                background-color: rgba(0, 0, 0, 0.6);
                z-index: 3000;
            ",
            onclick: move |_| show.set(false),
        }
        // Modal
        div {
            style: "
                position: fixed; top: 0; left: 0; right: 0; bottom: 0;
                display: flex; align-items: center; justify-content: center;
                z-index: 3001;
            ",
            onclick: move |e| e.stop_propagation(),
            div {
                style: "
                    width: 920px; height: 620px;
                    background-color: {BG_ELEVATED};
                    border: 1px solid {BORDER_DEFAULT};
                    border-radius: 10px;
                    box-shadow: 0 20px 50px rgba(0,0,0,0.6);
                    display: flex; flex-direction: column;
                    overflow: hidden;
                ",
                div {
                    style: "
                        display: flex; align-items: center; justify-content: space-between;
                        padding: 14px 18px;
                        background-color: {BG_SURFACE};
                        border-bottom: 1px solid {BORDER_DEFAULT};
                    ",
                    div {
                        style: "display: flex; flex-direction: column; gap: 4px;",
                        span { style: "font-size: 13px; font-weight: 600; color: {TEXT_PRIMARY};", "Providers (Global)" }
                        span { style: "font-size: 10px; color: {TEXT_DIM};", "{providers_root_label}" }
                    }
                    button {
                        class: "collapse-btn",
                        style: "
                            background: transparent; border: none; color: {TEXT_SECONDARY};
                            font-size: 12px; cursor: pointer; padding: 4px 8px; border-radius: 4px;
                        ",
                        onclick: move |_| show.set(false),
                        "Close"
                    }
                }

                div {
                    style: "flex: 1; display: flex; min-height: 0;",
                    // Left list
                    div {
                        style: "
                            width: 240px; padding: 12px;
                            border-right: 1px solid {BORDER_SUBTLE};
                            background-color: {BG_BASE};
                            display: flex; flex-direction: column; gap: 8px;
                        ",
                        div {
                            style: "display: flex; gap: 6px;",
                            button {
                                class: "collapse-btn",
                                style: "
                                    flex: 1; padding: 6px 8px;
                                    background-color: {BG_SURFACE};
                                    border: 1px solid {BORDER_DEFAULT};
                                    border-radius: 6px;
                                    color: {TEXT_SECONDARY}; font-size: 11px; cursor: pointer;
                                ",
                                onclick: on_provider_build,
                                "{provider_build_label}"
                            }
                            button {
                                class: "collapse-btn",
                                style: "
                                    flex: 1; padding: 6px 8px;
                                    background-color: {BG_SURFACE};
                                    border: 1px solid {BORDER_DEFAULT};
                                    border-radius: 6px;
                                    color: {TEXT_SECONDARY}; font-size: 11px; cursor: pointer;
                                ",
                                onclick: on_provider_new,
                                "New"
                            }
                            button {
                                class: "collapse-btn",
                                style: "
                                    flex: 1; padding: 6px 8px;
                                    background-color: {BG_SURFACE};
                                    border: 1px solid {BORDER_DEFAULT};
                                    border-radius: 6px;
                                    color: {TEXT_SECONDARY}; font-size: 11px; cursor: pointer;
                                ",
                                onclick: on_provider_reload,
                                "Reload"
                            }
                        }
                        div {
                            style: "
                                flex: 1; overflow-y: auto;
                                border: 1px solid {BORDER_SUBTLE};
                                border-radius: 6px;
                                background-color: {BG_ELEVATED};
                                padding: 6px;
                            ",
                            onclick: move |_| {
                                provider_editor_path.set(None);
                                provider_editor_text.set(String::new());
                                provider_editor_error.set(None);
                                provider_editor_dirty.set(false);
                            },
                            if provider_files().is_empty() {
                                div {
                                    style: "
                                        padding: 10px; font-size: 11px; color: {TEXT_DIM};
                                        text-align: center;
                                    ",
                                    "No providers yet"
                                }
                            } else {
                                for path in provider_files().iter() {
                                    {
                                        let file_name = path
                                            .file_name()
                                            .and_then(|name| name.to_str())
                                            .unwrap_or("provider.json");
                                        let path_clone = path.clone();
                                        let selected = provider_editor_path()
                                            .as_ref()
                                            .map(|selected| selected == path)
                                            .unwrap_or(false);
                                        let item_bg = if selected { BG_HOVER } else { "transparent" };
                                        let item_border = if selected { BORDER_ACCENT } else { BORDER_SUBTLE };
                                        rsx! {
                                            div {
                                                key: "{path.display()}",
                                                class: "collapse-btn",
                                                style: "
                                                    padding: 6px 8px; margin-bottom: 6px;
                                                    border: 1px solid {item_border};
                                                    background-color: {item_bg};
                                                    border-radius: 6px;
                                                    font-size: 11px; color: {TEXT_PRIMARY};
                                                    cursor: pointer;
                                                    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
                                                ",
                                                onclick: move |evt: MouseEvent| {
                                                    evt.stop_propagation();
                                                    if selected {
                                                        provider_editor_path.set(None);
                                                        provider_editor_text.set(String::new());
                                                        provider_editor_error.set(None);
                                                        provider_editor_dirty.set(false);
                                                    } else {
                                                        provider_editor_path.set(Some(path_clone.clone()));
                                                        provider_editor_text.set(read_provider_file(&path_clone).unwrap_or_default());
                                                        provider_editor_error.set(None);
                                                        provider_editor_dirty.set(false);
                                                    }
                                                },
                                                "{file_name}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        button {
                            class: "collapse-btn",
                            style: "
                                width: 100%; padding: 6px 8px;
                                background-color: transparent;
                                border: 1px solid {BORDER_DEFAULT};
                                border-radius: 6px;
                                color: #ef4444; font-size: 11px; cursor: pointer;
                            ",
                            onclick: on_provider_delete,
                            "Delete"
                        }
                    }

                    // Right editor
                    div {
                        style: "flex: 1; padding: 12px; display: flex; flex-direction: column; gap: 8px; min-width: 0;",
                        if editor_focused() {
                            textarea {
                                style: "
                                    flex: 1; width: 100%;
                                    background-color: {BG_SURFACE};
                                    border: 1px solid {BORDER_DEFAULT};
                                    border-radius: 6px;
                                    color: {TEXT_PRIMARY};
                                    font-family: 'SF Mono', Consolas, monospace;
                                    font-size: 11px; line-height: 1.5;
                                    padding: 10px; resize: none;
                                    white-space: pre;
                                    user-select: text;
                                ",
                                oninput: move |e| {
                                    provider_editor_text.set(e.value());
                                    provider_editor_dirty.set(true);
                                    provider_editor_error.set(None);
                                },
                                onfocus: move |_| editor_focused.set(true),
                                onblur: move |_| editor_focused.set(false),
                            }
                        } else {
                            textarea {
                                style: "
                                    flex: 1; width: 100%;
                                    background-color: {BG_SURFACE};
                                    border: 1px solid {BORDER_DEFAULT};
                                    border-radius: 6px;
                                    color: {TEXT_PRIMARY};
                                    font-family: 'SF Mono', Consolas, monospace;
                                    font-size: 11px; line-height: 1.5;
                                    padding: 10px; resize: none;
                                    white-space: pre;
                                    user-select: text;
                                ",
                                value: "{provider_editor_text()}",
                                oninput: move |e| {
                                    provider_editor_text.set(e.value());
                                    provider_editor_dirty.set(true);
                                    provider_editor_error.set(None);
                                },
                                onfocus: move |_| editor_focused.set(true),
                                onblur: move |_| editor_focused.set(false),
                            }
                        }
                        if let Some(error) = provider_editor_error() {
                            div {
                                style: "font-size: 11px; color: #f97316;",
                                "{error}"
                            }
                        }
                        div {
                            style: "display: flex; align-items: center; justify-content: space-between;",
                            span { style: "font-size: 10px; color: {TEXT_DIM};", "File: {provider_selected_label}" }
                            button {
                                class: "collapse-btn",
                                style: "
                                    padding: 6px 12px;
                                    background-color: {BG_SURFACE};
                                    border: 1px solid {BORDER_DEFAULT};
                                    border-radius: 6px;
                                    color: {TEXT_PRIMARY}; font-size: 11px; cursor: pointer;
                                ",
                                onclick: on_provider_save,
                                "{provider_save_label}"
                            }
                        }
                    }
                }
            }
        }
        }
    }
}
