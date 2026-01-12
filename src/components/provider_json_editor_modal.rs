use dioxus::prelude::*;
use std::path::PathBuf;

use crate::constants::*;
use crate::core::provider_store::{read_provider_file, write_provider_file};

#[component]
pub fn ProviderJsonEditorModal(
    show: Signal<bool>,
    provider_path: Signal<Option<PathBuf>>,
    on_saved: EventHandler<PathBuf>,
) -> Element {
    // Simple: just one signal for the text content
    let mut json_text = use_signal(String::new);
    let mut error = use_signal(|| None::<String>);
    let mut loaded_path = use_signal(|| None::<PathBuf>); // Track what we loaded
    let mut initial_load_done = use_signal(|| false); // Track if we did initial load
    
    // Load from file DIRECTLY - no use_effect!
    // When modal opens with a path different from what we loaded, load it now.
    let should_set_initial = if show() {
        let current_path = provider_path();
        let already_loaded = loaded_path();
        
        // Check if we need to load (path changed or first open)
        let need_load = match (&current_path, &already_loaded) {
            (Some(curr), Some(loaded)) => curr != loaded,
            (Some(_), None) => true,
            (None, Some(_)) => true, // Clear if path removed
            (None, None) => false,
        };
        
        if need_load {
            if let Some(path) = &current_path {
                println!("[DEBUG] JSON Editor loading file: {:?}", path);
                if let Some(contents) = read_provider_file(path) {
                    json_text.set(contents);
                    error.set(None);
                } else {
                    error.set(Some(format!("Failed to read: {}", path.display())));
                }
                loaded_path.set(Some(path.clone()));
                initial_load_done.set(true);
                true // We just loaded, need to set initial value
            } else {
                // No path = clear for new
                json_text.set(String::new());
                error.set(None);
                loaded_path.set(None);
                initial_load_done.set(true);
                true
            }
        } else {
            false
        }
    } else {
        // When hidden, reset so we reload next time
        if loaded_path().is_some() {
            loaded_path.set(None);
            initial_load_done.set(false);
        }
        false
    };
    
    // Use different value binding depending on whether we just loaded
    let current_value = json_text();
    
    let save_handler = move |_| async move {
        let Some(path) = provider_path() else {
            error.set(Some("No provider file selected".to_string()));
            return;
        };
        
        // Read directly from textarea via JS to get current value
        let eval_result = document::eval(r#"document.getElementById('json-editor-textarea')?.value || ''"#).await;
        let text = match eval_result {
            Ok(val) => val.as_str().unwrap_or_default().to_string(),
            Err(_) => json_text(), // Fallback to signal
        };
        
        // Validate JSON before saving
        if let Err(e) = serde_json::from_str::<serde_json::Value>(&text) {
            error.set(Some(format!("Invalid JSON: {}", e)));
            return;
        }
        
        if let Err(e) = write_provider_file(&path, &text) {
            error.set(Some(format!("Failed to save: {}", e)));
            return;
        }
        
        error.set(None);
        on_saved.call(path);
    };
    
    let file_name = provider_path()
        .and_then(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "provider.json".to_string());
    
    rsx! {
        if !show() {
            div {}
        } else {
            // Backdrop
            div {
                style: "
                    position: fixed; top: 0; left: 0; right: 0; bottom: 0;
                    background-color: rgba(0, 0, 0, 0.6);
                    z-index: 3200;
                ",
                onclick: move |_| show.set(false),
            }
            
            // Modal
            div {
                style: "
                    position: fixed; top: 0; left: 0; right: 0; bottom: 0;
                    display: flex; align-items: center; justify-content: center;
                    z-index: 3201;
                ",
                onclick: move |e| e.stop_propagation(),
                
                div {
                    style: "
                        width: 800px; height: 720px;
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
                                "Edit Provider JSON" 
                            }
                            span { 
                                style: "font-size: 10px; color: {TEXT_DIM};", 
                                "{file_name}" 
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
                    
                    // Error message
                    if let Some(err) = error() {
                        div {
                            style: "padding: 8px 18px; font-size: 11px; color: #f97316;",
                            "{err}"
                        }
                    }
                    
                    // Editor
                    div {
                        style: "flex: 1; padding: 12px; display: flex; flex-direction: column; gap: 8px;",
                        textarea {
                            id: "json-editor-textarea",
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
                            value: "{json_text()}",
                            // No oninput binding - save reads directly from DOM
                        }
                        
                        // Save button
                        div {
                            style: "display: flex; justify-content: flex-end;",
                            button {
                                class: "collapse-btn",
                                style: "
                                    padding: 6px 12px;
                                    background-color: {BG_SURFACE};
                                    border: 1px solid {BORDER_DEFAULT};
                                    border-radius: 6px;
                                    color: {TEXT_PRIMARY}; font-size: 11px; cursor: pointer;
                                ",
                                onclick: save_handler,
                                "Save"
                            }
                        }
                    }
                }
            }
        }
    }
}
