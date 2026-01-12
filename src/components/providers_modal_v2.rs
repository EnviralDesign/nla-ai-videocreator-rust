use dioxus::prelude::*;
use std::path::PathBuf;

use crate::constants::*;
use crate::core::provider_store::read_provider_file;

#[component]
pub fn ProvidersModalV2(
    show: Signal<bool>,
    provider_files: Signal<Vec<PathBuf>>,
    on_new: EventHandler<()>,
    on_reload: EventHandler<()>,
    on_delete: EventHandler<PathBuf>,
    on_edit_builder: EventHandler<PathBuf>,
    on_edit_json: EventHandler<PathBuf>,
) -> Element {
    let mut selected_provider = use_signal(|| None::<PathBuf>);
    
    let providers_root = crate::core::provider_store::global_providers_root()
        .display()
        .to_string();
    
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
                        width: 700px; height: 520px;
                        background-color: {BG_ELEVATED};
                        border: 1px solid {BORDER_DEFAULT};
                        border-radius: 10px;
                        box-shadow: 0 20px 50px rgba(0,0,0,0.6);
                        display: flex; flex-direction: column;
                        overflow: hidden;
                    ",
                    
                    // Header
                    div {
                        style: "
                            display: flex; align-items: center; justify-content: space-between;
                            padding: 14px 18px;
                            background-color: {BG_SURFACE};
                            border-bottom: 1px solid {BORDER_DEFAULT};
                        ",
                        div {
                            style: "display: flex; flex-direction: column; gap: 4px;",
                            span { 
                                style: "font-size: 13px; font-weight: 600; color: {TEXT_PRIMARY};", 
                                "AI Providers (Global)" 
                            }
                            span { 
                                style: "font-size: 10px; color: {TEXT_DIM};", 
                                "{providers_root}" 
                            }
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
                    
                    // Body
                    div {
                        style: "flex: 1; display: flex; min-height: 0;",
                        
                        // Left: Provider list
                        div {
                            style: "
                                width: 280px; padding: 12px;
                                border-right: 1px solid {BORDER_SUBTLE};
                                background-color: {BG_BASE};
                                display: flex; flex-direction: column; gap: 8px;
                            ",
                            
                            // Top buttons: New, Reload
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
                                    onclick: move |_| on_new.call(()),
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
                                    onclick: move |_| on_reload.call(()),
                                    "Reload"
                                }
                            }
                            
                            // Provider list
                            div {
                                style: "
                                    flex: 1; overflow-y: auto;
                                    border: 1px solid {BORDER_SUBTLE};
                                    border-radius: 6px;
                                    background-color: {BG_ELEVATED};
                                    padding: 6px;
                                ",
                                
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
                                                .and_then(|n| n.to_str())
                                                .unwrap_or("provider.json");
                                            let path_clone = path.clone();
                                            let is_selected = selected_provider()
                                                .as_ref()
                                                .map(|s| s == path)
                                                .unwrap_or(false);
                                            let item_bg = if is_selected { BG_HOVER } else { "transparent" };
                                            let item_border = if is_selected { BORDER_ACCENT } else { BORDER_SUBTLE };
                                            
                                            // Try to get provider name from JSON
                                            let provider_name = read_provider_file(&path_clone)
                                                .and_then(|json| serde_json::from_str::<serde_json::Value>(&json).ok())
                                                .and_then(|v| v.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()))
                                                .unwrap_or_else(|| "Unnamed".to_string());
                                            
                                            rsx! {
                                                div {
                                                    key: "{path.display()}",
                                                    class: "collapse-btn",
                                                    style: "
                                                        padding: 8px; margin-bottom: 6px;
                                                        border: 1px solid {item_border};
                                                        background-color: {item_bg};
                                                        border-radius: 6px;
                                                        cursor: pointer;
                                                        display: flex; flex-direction: column; gap: 2px;
                                                    ",
                                                    onclick: move |_| selected_provider.set(Some(path_clone.clone())),
                                                    
                                                    span { 
                                                        style: "font-size: 11px; font-weight: 600; color: {TEXT_PRIMARY};", 
                                                        "{provider_name}" 
                                                    }
                                                    span { 
                                                        style: "font-size: 9px; color: {TEXT_DIM};", 
                                                        "{file_name}" 
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            
                            // Delete button at bottom (only if selected)
                            if selected_provider().is_some() {
                                button {
                                    class: "collapse-btn",
                                    style: "
                                        width: 100%; padding: 6px 8px;
                                        background-color: transparent;
                                        border: 1px solid {BORDER_DEFAULT};
                                        border-radius: 6px;
                                        color: #ef4444; font-size: 11px; cursor: pointer;
                                    ",
                                    onclick: move |_| {
                                        if let Some(path) = selected_provider() {
                                            on_delete.call(path.clone());
                                            selected_provider.set(None);
                                        }
                                    },
                                    "Delete"
                                }
                            }
                        }
                        
                        // Right: Actions when provider selected
                        div {
                            style: "
                                flex: 1; padding: 24px;
                                display: flex; flex-direction: column;
                                align-items: center; justify-content: center;
                                gap: 16px;
                            ",
                            
                            if let Some(path) = selected_provider() {
                                // Get provider info
                                {
                                    let provider_name = read_provider_file(&path)
                                        .and_then(|json| serde_json::from_str::<serde_json::Value>(&json).ok())
                                        .and_then(|v| v.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()))
                                        .unwrap_or_else(|| "Unnamed Provider".to_string());
                                    
                                    rsx! {
                                        div {
                                            style: "text-align: center; margin-bottom: 12px;",
                                            div { 
                                                style: "font-size: 14px; font-weight: 600; color: {TEXT_PRIMARY}; margin-bottom: 4px;", 
                                                "{provider_name}" 
                                            }
                                            div { 
                                                style: "font-size: 10px; color: {TEXT_DIM};", 
                                                "Select an editor:" 
                                            }
                                        }
                                        
                                        {
                                            let path_for_builder = path.clone();
                                            let path_for_json = path.clone();
                                            rsx! {
                                                button {
                                                    class: "collapse-btn",
                                                    style: "
                                                        width: 240px; padding: 12px 16px;
                                                        background-color: {BG_SURFACE};
                                                        border: 1px solid {BORDER_DEFAULT};
                                                        border-radius: 8px;
                                                        color: {TEXT_PRIMARY}; font-size: 12px; font-weight: 600;
                                                        cursor: pointer;
                                                    ",
                                                    onclick: move |_| on_edit_builder.call(path_for_builder.clone()),
                                                    "Edit in Builder"
                                                }
                                                
                                                button {
                                                    class: "collapse-btn",
                                                    style: "
                                                        width: 240px; padding: 12px 16px;
                                                        background-color: {BG_SURFACE};
                                                        border: 1px solid {BORDER_DEFAULT};
                                                        border-radius: 8px;
                                                        color: {TEXT_PRIMARY}; font-size: 12px; font-weight: 600;
                                                        cursor: pointer;
                                                    ",
                                                    onclick: move |_| on_edit_json.call(path_for_json.clone()),
                                                    "Edit as JSON"
                                                }
                                            }
                                        }
                                    }
                                }
                            } else {
                                div {
                                    style: "font-size: 11px; color: {TEXT_DIM}; text-align: center;",
                                    "Select a provider from the list"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
