use dioxus::prelude::*;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::constants::*;
use crate::core::comfyui_workflow::ComfyWorkflowNode;
use crate::core::provider_store::{provider_path_for_entry, read_provider_file, write_provider_file};
use crate::state::{
    ComfyOutputSelector, ComfyWorkflowRef, InputBinding, ManifestInput, NodeSelector,
    ProviderConnection, ProviderEntry, ProviderInputField, ProviderInputType, ProviderManifest,
    ProviderOutputType, InputUi,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum BuilderMode {
    Inputs,
    Output,
}

#[derive(Clone)]
struct NodeSelectorDraft {
    class_type: String,
    input_key: String,
    title: Option<String>,
}

#[derive(Clone)]
struct OutputNodeDraft {
    class_type: String,
    title: Option<String>,
}

#[derive(Clone)]
struct BuilderInput {
    name: String,
    label: String,
    input_type_key: String,
    required: bool,
    default_text: String,
    enum_options: String,
    tag: String,
    multiline: bool,
    selector: NodeSelectorDraft,
}

#[component]
pub fn ProviderBuilderModalV2(
    show: Signal<bool>,
    provider_path: Signal<Option<PathBuf>>,
    on_saved: EventHandler<PathBuf>,
) -> Element {
    // All the state signals - no "initialized" flag needed!
    let mut workflow_path = use_signal(|| None::<PathBuf>);
    let mut workflow_nodes = use_signal(Vec::<ComfyWorkflowNode>::new);
    let mut workflow_error = use_signal(|| None::<String>);
    let mut workflow_search = use_signal(String::new);
    let mut selected_node_id = use_signal(|| None::<String>);

    let mut provider_name = use_signal(|| "New Provider".to_string());
    let mut provider_id = use_signal(Uuid::new_v4);
    let mut base_url = use_signal(|| "http://127.0.0.1:8188".to_string());
    let mut output_type = use_signal(|| ProviderOutputType::Image);
    let mut output_key = use_signal(|| "images".to_string());
    let mut output_tag = use_signal(String::new);
    let mut output_node = use_signal(|| None::<OutputNodeDraft>);
    let mut builder_mode = use_signal(|| BuilderMode::Inputs);

    let mut exposed_inputs = use_signal(Vec::<BuilderInput>::new);
    let mut builder_error = use_signal(|| None::<String>);
    let mut manifest_path = use_signal(|| None::<PathBuf>);
    let mut loaded_path = use_signal(|| None::<PathBuf>); // Track what we loaded

    // Load provider DIRECTLY when modal opens - no use_effect!
    if show() {
        let current_path = provider_path();
        let already_loaded = loaded_path();
        
        // Check if we need to load (path changed or first open)
        let need_load = match (&current_path, &already_loaded) {
            (Some(curr), Some(loaded)) => curr != loaded,
            (Some(_), None) => true,
            (None, Some(_)) => true, // Reset if path removed (New clicked)
            (None, None) => false,
        };
        
        if need_load {
            println!("[DEBUG] ProviderBuilderV2: Loading from: {:?}", current_path);
            
            // Reset to defaults first
            provider_name.set("New Provider".to_string());
            provider_id.set(Uuid::new_v4());
            base_url.set("http://127.0.0.1:8188".to_string());
            output_type.set(ProviderOutputType::Image);
            output_key.set("images".to_string());
            output_tag.set(String::new());
            output_node.set(None);
            exposed_inputs.set(Vec::new());
            workflow_path.set(None);
            workflow_nodes.set(Vec::new());
            workflow_error.set(None);
            manifest_path.set(None);
            builder_error.set(None);
            
            if let Some(ref path) = current_path {
                // Load and parse provider JSON
                if let Some(json) = read_provider_file(path) {
                    if let Ok(entry) = serde_json::from_str::<ProviderEntry>(&json) {
                        println!("[DEBUG] Loaded provider: {}", entry.name);
                        
                        // PRESERVE EXISTING UUID!
                        provider_id.set(entry.id);
                        provider_name.set(entry.name.clone());
                        output_type.set(entry.output_type);
                        
                        if let ProviderConnection::ComfyUi {
                            base_url: url,
                            workflow_path: wf_path,
                            manifest_path: man_path,
                        } = &entry.connection {
                            base_url.set(url.clone());
                            
                            // Load workflow if present
                            if let Some(wf_path_str) = wf_path {
                                let wf_path = PathBuf::from(wf_path_str);
                                match crate::core::comfyui_workflow::load_workflow_nodes(&wf_path) {
                                    Ok(nodes) => {
                                        println!("[DEBUG] Loaded {} workflow nodes", nodes.len());
                                        workflow_path.set(Some(wf_path));
                                        workflow_nodes.set(nodes);
                                    }
                                    Err(err) => {
                                        workflow_error.set(Some(err));
                                    }
                                }
                            }
                            
                            // Load manifest if present
                            if let Some(man_path_str) = man_path {
                                let man_path_buf = PathBuf::from(man_path_str);
                                manifest_path.set(Some(man_path_buf.clone()));
                                
                                if let Ok(man_json) = std::fs::read_to_string(&man_path_buf) {
                                    if let Ok(manifest) = serde_json::from_str::<ProviderManifest>(&man_json) {
                                        if let ProviderManifest::ComfyUi { inputs, output, .. } = manifest {
                                            println!("[DEBUG] Loaded manifest with {} inputs", inputs.len());
                                            
                                            // Populate inputs from manifest
                                            let mut next_inputs = Vec::new();
                                            for input in inputs {
                                                let (input_type_key, enum_options) = input_type_to_key(&input.input_type);
                                                let default_text = default_value_to_text(input.default.as_ref());
                                                next_inputs.push(BuilderInput {
                                                    name: input.name,
                                                    label: input.label,
                                                    input_type_key,
                                                    required: input.required,
                                                    default_text,
                                                    enum_options,
                                                    tag: input.bind.selector.tag.unwrap_or_default(),
                                                    multiline: input.ui.as_ref().map(|ui| ui.multiline).unwrap_or(false),
                                                    selector: NodeSelectorDraft {
                                                        class_type: input.bind.selector.class_type,
                                                        input_key: input.bind.selector.input_key,
                                                        title: input.bind.selector.title,
                                                    },
                                                });
                                            }
                                            exposed_inputs.set(next_inputs);
                                            
                                            // Populate output from manifest
                                            let key = if output.selector.input_key.trim().is_empty() {
                                                "images".to_string()
                                            } else {
                                                output.selector.input_key
                                            };
                                            output_key.set(key);
                                            output_tag.set(output.selector.tag.unwrap_or_default());
                                            output_node.set(Some(OutputNodeDraft {
                                                class_type: output.selector.class_type,
                                                title: output.selector.title,
                                            }));
                                        }
                                    }
                                }
                            }
                        }
                        
                        println!("[DEBUG] Provider load complete");
                    } else {
                        builder_error.set(Some("Failed to parse provider JSON".to_string()));
                    }
                } else {
                    builder_error.set(Some(format!("Failed to read: {}", path.display())));
                }
                loaded_path.set(Some(path.clone()));
            } else {
                // New provider - already reset above
                loaded_path.set(None);
            }
        }
    } else {
        // When hidden, reset loaded_path so we reload next time
        if loaded_path().is_some() {
            loaded_path.set(None);
        }
    }

    let pick_workflow = move |_| {
        let mut dialog = rfd::FileDialog::new();
        if let Some(workflows_dir) = crate::core::paths::resource_dir("workflows") {
            dialog = dialog.set_directory(workflows_dir);
        }
        if let Some(path) = dialog
            .add_filter("ComfyUI API Workflow", &["json"])
            .set_title("Select ComfyUI Workflow")
            .pick_file()
        {
            match crate::core::comfyui_workflow::load_workflow_nodes(&path) {
                Ok(nodes) => {
                    workflow_path.set(Some(path));
                    workflow_nodes.set(nodes);
                    workflow_error.set(None);
                    selected_node_id.set(None);
                }
                Err(err) => {
                    workflow_error.set(Some(err));
                    workflow_nodes.set(Vec::new());
                    selected_node_id.set(None);
                }
            }
        }
    };

    // NOTE: Keeping the rest of the builder UI mostly the same,
    // just the initialization logic changed. Let me add the save handler
    // then render the rest of the UI...
    
    let save_provider = move |_| {
        println!("[DEBUG] Save provider clicked");
        
        let Some(wf_path) = workflow_path() else {
            builder_error.set(Some("Select a workflow first".to_string()));
            return;
        };
        
        let name = provider_name().trim().to_string();
        if name.is_empty() {
            builder_error.set(Some("Provider name required".to_string()));
            return;
        }
        
        let Some(out_node) = output_node() else {
            builder_error.set(Some("Select an output node".to_string()));
            return;
        };
        
        let out_key = output_key().trim().to_string();
        if out_key.is_empty() {
            builder_error.set(Some("Output key required".to_string()));
            return;
        }
        
        // Build manifest and provider entry
        let workflow_path_str = wf_path.to_string_lossy().to_string();
        let manifest_path_value = manifest_path()
            .unwrap_or_else(|| derive_manifest_path(&wf_path));
        let manifest_path_str = manifest_path_value.to_string_lossy().to_string();
        
        let mut manifest_inputs = Vec::new();
        let mut provider_inputs = Vec::new();
        
        for input in exposed_inputs().iter() {
            let input_type = match parse_input_type(input) {
                Ok(t) => t,
                Err(err) => {
                    builder_error.set(Some(err));
                    return;
                }
            };
            
            let default_value = match parse_default_value(&input_type, &input.default_text) {
                Ok(v) => v,
                Err(err) => {
                    builder_error.set(Some(err));
                    return;
                }
            };
            
            let tag = input.tag.trim();
            let selector = NodeSelector {
                tag: if tag.is_empty() { None } else { Some(tag.to_string()) },
                class_type: input.selector.class_type.clone(),
                input_key: input.selector.input_key.clone(),
                title: input.selector.title.clone(),
            };
            
            let input_ui = build_input_ui(input);
            
            manifest_inputs.push(ManifestInput {
                name: input.name.clone(),
                label: input.label.clone(),
                input_type: input_type.clone(),
                required: input.required,
                default: default_value.clone(),
                ui: input_ui.clone(),
                bind: InputBinding {
                    selector,
                    transform: None,
                },
            });
            
            provider_inputs.push(ProviderInputField {
                name: input.name.clone(),
                label: input.label.clone(),
                input_type,
                required: input.required,
                default: default_value,
                ui: input_ui,
            });
        }
        
        let output_tag_value = output_tag();
        let out_tag = output_tag_value.trim();
        let output_selector = NodeSelector {
            tag: if out_tag.is_empty() { None } else { Some(out_tag.to_string()) },
            class_type: out_node.class_type.clone(),
            input_key: out_key.clone(),
            title: out_node.title.clone(),
        };
        
        let manifest = ProviderManifest::ComfyUi {
            schema_version: 1,
            name: Some(name.clone()),
            output_type: output_type(),
            workflow: ComfyWorkflowRef {
                workflow_path: workflow_path_str.clone(),
                workflow_hash: None,
            },
            inputs: manifest_inputs,
            output: ComfyOutputSelector {
                selector: output_selector,
                index: None,
            },
        };
        
        // CRITICAL: Use existing provider_id if we loaded one, don't generate new!
        let entry = ProviderEntry {
            id: provider_id(), // ← PRESERVES UUID
            name,
            output_type: output_type(),
            inputs: provider_inputs,
            connection: ProviderConnection::ComfyUi {
                base_url: base_url(),
                workflow_path: Some(workflow_path_str),
                manifest_path: Some(manifest_path_str),
            },
        };
        
        // Write manifest
        let manifest_json = match serde_json::to_string_pretty(&manifest) {
            Ok(json) => json,
            Err(err) => {
                builder_error.set(Some(format!("Failed to serialize manifest: {}", err)));
                return;
            }
        };
        
        if let Some(parent) = manifest_path_value.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        
        if let Err(err) = std::fs::write(&manifest_path_value, manifest_json) {
            builder_error.set(Some(format!("Failed to write manifest: {}", err)));
            return;
        }
        
        // Write provider
        let provider_json = match serde_json::to_string_pretty(&entry) {
            Ok(json) => json,
            Err(err) => {
                builder_error.set(Some(format!("Failed to serialize provider: {}", err)));
                return;
            }
        };
        
        let save_path = provider_path()
            .unwrap_or_else(|| provider_path_for_entry(&entry));
        
        if let Err(err) = write_provider_file(&save_path, &provider_json) {
            builder_error.set(Some(format!("Failed to save provider: {}", err)));
            return;
        }
        
        println!("[DEBUG] Provider saved successfully to: {:?}", save_path);
        manifest_path.set(Some(manifest_path_value));
        on_saved.call(save_path);
    };

    // Shortened version of rest of UI - keeping interactive parts
    let query = workflow_search().trim().to_lowercase();
    let nodes = workflow_nodes();
    let filtered_nodes: Vec<ComfyWorkflowNode> = if query.is_empty() {
        nodes
    } else {
        nodes
            .into_iter()
            .filter(|node| {
                node.id.to_lowercase().contains(&query)
                    || node.class_type.to_lowercase().contains(&query)
                    || node
                        .title
                        .as_ref()
                        .map(|title| title.to_lowercase().contains(&query))
                        .unwrap_or(false)
                    || node
                        .inputs
                        .iter()
                        .any(|input| input.to_lowercase().contains(&query))
            })
            .collect()
    };

    let selected_node = selected_node_id().and_then(|id| {
        workflow_nodes()
            .into_iter()
            .find(|node| node.id == id)
    });

    // UI helper values
    let inputs_active = builder_mode() == BuilderMode::Inputs;
    let input_tab_bg = if inputs_active { BG_HOVER } else { BG_SURFACE };
    let input_tab_border = if inputs_active { BORDER_ACCENT } else { BORDER_DEFAULT };
    let input_tab_color = if inputs_active { TEXT_PRIMARY } else { TEXT_SECONDARY };
    let output_tab_bg = if !inputs_active { BG_HOVER } else { BG_SURFACE };
    let output_tab_border = if !inputs_active { BORDER_ACCENT } else { BORDER_DEFAULT };
    let output_tab_color = if !inputs_active { TEXT_PRIMARY } else { TEXT_SECONDARY };

    let output_status_label = if let Some(node) = output_node() {
        format!("Output: {} ({})", node.title.unwrap_or_else(|| "Untitled".to_string()), node.class_type)
    } else {
        "Output: Not set".to_string()
    };

    // Add input handler
    let mut add_input_from_node = {
        let mut exposed_inputs = exposed_inputs.clone();
        let mut builder_error = builder_error.clone();
        move |node: &ComfyWorkflowNode, input_key: &str| {
            if exposed_inputs().iter().any(|input| input.name == input_key) {
                builder_error.set(Some("Input already exposed.".to_string()));
                return;
            }
            let selector = NodeSelectorDraft {
                class_type: node.class_type.clone(),
                input_key: input_key.to_string(),
                title: node.title.clone(),
            };
            let input = BuilderInput {
                name: input_key.to_string(),
                label: friendly_label(input_key),
                input_type_key: "text".to_string(),
                required: false,
                default_text: String::new(),
                enum_options: String::new(),
                tag: String::new(),
                multiline: false,
                selector,
            };
            let mut next = exposed_inputs();
            next.push(input);
            exposed_inputs.set(next);
            builder_error.set(None);
        }
    };

    let mut set_output_from_node = {
        let mut output_node = output_node.clone();
        let mut output_tag = output_tag.clone();
        move |node: &ComfyWorkflowNode| {
            output_node.set(Some(OutputNodeDraft {
                class_type: node.class_type.clone(),
                title: node.title.clone(),
            }));
            output_tag.set(String::new());
        }
    };

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
                        width: 1060px; height: 720px;
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
                            span { style: "font-size: 13px; font-weight: 600; color: {TEXT_PRIMARY};", "Provider Builder (ComfyUI)" }
                            span {
                                style: "font-size: 10px; color: {TEXT_DIM};",
                                if provider_path().is_some() { "Mode: Edit" } else { "Mode: New" }
                            }
                            span {
                                style: "font-size: 10px; color: {TEXT_DIM};",
                                if let Some(path) = workflow_path() {
                                    "{path.display()}"
                                } else {
                                    "No workflow selected"
                                }
                            }
                        }
                        div {
                            style: "display: flex; gap: 8px; align-items: center;",
                            button {
                                class: "collapse-btn",
                                style: "
                                    background: {BG_SURFACE}; border: 1px solid {BORDER_DEFAULT};
                                    color: {TEXT_PRIMARY}; font-size: 11px; cursor: pointer;
                                    padding: 6px 10px; border-radius: 6px;
                                ",
                                onclick: pick_workflow,
                                "Choose Workflow..."
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
                    }

                    // Errors
                    if let Some(error) = workflow_error() {
                        div { style: "padding: 8px 18px; font-size: 11px; color: #f97316;", "{error}" }
                    }
                    if let Some(error) = builder_error() {
                        div { style: "padding: 8px 18px; font-size: 11px; color: #f97316;", "{error}" }
                    }

                    // Main content
                    div {
                        style: "flex: 1; display: flex; flex-direction: column; min-height: 0;",

                        // Tabs
                        div {
                            style: "
                                padding: 8px 18px;
                                border-bottom: 1px solid {BORDER_DEFAULT};
                                background-color: {BG_SURFACE};
                                display: flex; align-items: center; justify-content: space-between;
                            ",
                            div {
                                style: "display: flex; gap: 6px;",
                                button {
                                    class: "collapse-btn",
                                    style: "
                                        padding: 4px 10px; font-size: 11px;
                                        background-color: {input_tab_bg};
                                        border: 1px solid {input_tab_border};
                                        border-radius: 6px; color: {input_tab_color};
                                        cursor: pointer;
                                    ",
                                    onclick: move |_| builder_mode.set(BuilderMode::Inputs),
                                    "Inputs"
                                }
                                button {
                                    class: "collapse-btn",
                                    style: "
                                        padding: 4px 10px; font-size: 11px;
                                        background-color: {output_tab_bg};
                                        border: 1px solid {output_tab_border};
                                        border-radius: 6px; color: {output_tab_color};
                                        cursor: pointer;
                                    ",
                                    onclick: move |_| builder_mode.set(BuilderMode::Output),
                                    "Output"
                                }
                            }
                            div { style: "font-size: 10px; color: {TEXT_DIM};", "{output_status_label}" }
                        }

                        // 3-column layout
                        div {
                            style: "flex: 1; display: flex; min-height: 0;",

                            // Left: Node list
                            div {
                                style: "
                                    width: 280px; padding: 12px;
                                    border-right: 1px solid {BORDER_SUBTLE};
                                    background-color: {BG_BASE};
                                    display: flex; flex-direction: column; gap: 8px;
                                ",
                                crate::components::common::StableTextInput {
                                    id: "workflow-search-input".to_string(),
                                    value: workflow_search(),
                                    placeholder: Some("Search nodes or inputs...".to_string()),
                                    style: Some(format!("
                                        width: 100%; padding: 6px 8px; font-size: 11px;
                                        background-color: {}; color: {};
                                        border: 1px solid {}; border-radius: 6px;
                                        outline: none;
                                    ", BG_SURFACE, TEXT_PRIMARY, BORDER_DEFAULT)),
                                    on_change: move |v: String| workflow_search.set(v),
                                    on_blur: move |_| {},
                                    on_keydown: move |_| {},
                                    autofocus: false,
                                }
                                div {
                                    style: "
                                        flex: 1; overflow-y: auto;
                                        border: 1px solid {BORDER_SUBTLE};
                                        border-radius: 6px;
                                        background-color: {BG_ELEVATED};
                                        padding: 6px;
                                    ",
                                    if filtered_nodes.is_empty() {
                                        div {
                                            style: "padding: 10px; font-size: 11px; color: {TEXT_DIM}; text-align: center;",
                                            if workflow_nodes().is_empty() {
                                                "Choose a workflow to see nodes"
                                            } else {
                                                "No nodes match search"
                                            }
                                        }
                                    } else {
                                        for node in filtered_nodes.into_iter() {
                                            {
                                                let selected = selected_node_id()
                                                    .as_ref()
                                                    .map(|id| id == &node.id)
                                                    .unwrap_or(false);
                                                let item_bg = if selected { BG_HOVER } else { "transparent" };
                                                let item_border = if selected { BORDER_ACCENT } else { BORDER_SUBTLE };
                                                let node_id = node.id.clone();
                                                let title = node.title.clone().unwrap_or_else(|| "Untitled".to_string());
                                                rsx! {
                                                    div {
                                                        key: "{node_id}",
                                                        class: "collapse-btn",
                                                        style: "
                                                            padding: 6px 8px; margin-bottom: 6px;
                                                            border: 1px solid {item_border};
                                                            background-color: {item_bg};
                                                            border-radius: 6px;
                                                            font-size: 11px; color: {TEXT_PRIMARY};
                                                            cursor: pointer;
                                                            display: flex; flex-direction: column; gap: 2px;
                                                        ",
                                                        onclick: move |_| selected_node_id.set(Some(node_id.clone())),
                                                        span { style: "font-weight: 600;", "{title}" }
                                                        span { style: "font-size: 10px; color: {TEXT_DIM};", "{node.class_type}" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // Middle: Node details
                            div {
                                style: "
                                    width: 240px; padding: 12px;
                                    border-right: 1px solid {BORDER_SUBTLE};
                                    background-color: {BG_ELEVATED};
                                    display: flex; flex-direction: column; gap: 12px;
                                    overflow-y: auto;
                                ",
                                if let Some(node) = selected_node {
                                    div {
                                        style: "
                                            padding: 10px; border: 1px solid {BORDER_SUBTLE};
                                            background-color: {BG_SURFACE}; border-radius: 6px;
                                        ",
                                        div { style: "font-size: 12px; color: {TEXT_PRIMARY}; font-weight: 600;", "{node.title.clone().unwrap_or_else(|| \"Untitled\".to_string())}" }
                                        div { style: "font-size: 10px; color: {TEXT_DIM};", "Class: {node.class_type}" }
                                        div { style: "font-size: 10px; color: {TEXT_DIM};", "Node ID: {node.id}" }
                                    }
                                    if inputs_active {
                                        div {
                                            style: "
                                                padding: 10px; border: 1px solid {BORDER_SUBTLE};
                                                background-color: {BG_SURFACE}; border-radius: 6px;
                                                display: flex; flex-direction: column; gap: 6px;
                                            ",
                                            div { style: "font-size: 10px; color: {TEXT_DIM}; text-transform: uppercase; letter-spacing: 0.5px;", "Inputs" }
                                            if node.inputs.is_empty() {
                                                div { style: "font-size: 11px; color: {TEXT_DIM};", "No inputs found." }
                                            } else {
                                                for input in node.inputs.iter() {
                                                    {
                                                        let node_clone = node.clone();
                                                        let input_key = input.clone();
                                                        rsx! {
                                                            div {
                                                                key: "{input}",
                                                                style: "display: flex; align-items: center; justify-content: space-between;",
                                                                span { style: "font-size: 11px; color: {TEXT_PRIMARY};", "{input}" }
                                                                button {
                                                                    class: "collapse-btn",
                                                                    style: "
                                                                        padding: 2px 8px; font-size: 10px;
                                                                        background-color: {BG_ELEVATED};
                                                                        border: 1px solid {BORDER_DEFAULT};
                                                                        border-radius: 4px; color: {TEXT_PRIMARY};
                                                                        cursor: pointer;
                                                                    ",
                                                                    onclick: move |_| add_input_from_node(&node_clone, &input_key),
                                                                    "Expose"
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    } else {
                                        div {
                                            style: "
                                                padding: 10px; border: 1px solid {BORDER_SUBTLE};
                                                background-color: {BG_SURFACE}; border-radius: 6px;
                                                display: flex; flex-direction: column; gap: 6px;
                                            ",
                                            div { style: "font-size: 10px; color: {TEXT_DIM}; text-transform: uppercase; letter-spacing: 0.5px;", "Output Node" }
                                            button {
                                                class: "collapse-btn",
                                                style: "
                                                    padding: 6px 8px; font-size: 11px;
                                                    background-color: {BG_ELEVATED};
                                                    border: 1px solid {BORDER_DEFAULT};
                                                    border-radius: 6px; color: {TEXT_PRIMARY};
                                                    cursor: pointer;
                                                ",
                                                onclick: move |_| set_output_from_node(&node),
                                                "Use as Output"
                                            }
                                        }
                                    }
                                } else {
                                    div {
                                        style: "font-size: 11px; color: {TEXT_DIM};",
                                        if inputs_active {
                                            "Select a node to expose inputs."
                                        } else {
                                            "Select a node to set output."
                                        }
                                    }
                                }
                            }

                            // Right: Provider config
                            div {
                                style: "flex: 1; padding: 12px; display: flex; flex-direction: column; gap: 12px; min-width: 0; overflow-y: auto;",

                                // Settings
                                div {
                                    style: "
                                        padding: 10px; border: 1px solid {BORDER_SUBTLE};
                                        background-color: {BG_SURFACE}; border-radius: 6px;
                                        display: flex; flex-direction: column; gap: 8px;
                                    ",
                                    div { style: "font-size: 10px; color: {TEXT_DIM}; text-transform: uppercase; letter-spacing: 0.5px;", "Provider Settings" }
                                    div {
                                        style: "display: flex; gap: 8px;",
                                        crate::components::common::StableTextInput {
                                            id: "provider-name-input".to_string(),
                                            value: provider_name(),
                                            placeholder: Some("Provider name".to_string()),
                                            style: Some(format!("
                                                flex: 1; padding: 6px 8px; font-size: 11px;
                                                background-color: {}; color: {};
                                                border: 1px solid {}; border-radius: 6px;
                                            ", BG_ELEVATED, TEXT_PRIMARY, BORDER_DEFAULT)),
                                            on_change: move |v: String| provider_name.set(v),
                                            on_blur: move |_| {},
                                            on_keydown: move |_| {},
                                            autofocus: false,
                                        }
                                        select {
                                            value: "{output_type_key(output_type())}",
                                            style: "
                                                width: 100px; padding: 6px 8px; font-size: 11px;
                                                background-color: {BG_ELEVATED}; color: {TEXT_PRIMARY};
                                                border: 1px solid {BORDER_DEFAULT}; border-radius: 6px;
                                            ",
                                            onchange: move |e| output_type.set(parse_output_type(&e.value())),
                                            option { value: "image", "Image" }
                                            option { value: "video", "Video" }
                                            option { value: "audio", "Audio" }
                                        }
                                    }
                                    crate::components::common::StableTextInput {
                                        id: "base-url-input".to_string(),
                                        value: base_url(),
                                        placeholder: Some("ComfyUI URL (http://127.0.0.1:8188)".to_string()),
                                        style: Some(format!("
                                            width: 100%; padding: 6px 8px; font-size: 11px;
                                            background-color: {}; color: {};
                                            border: 1px solid {}; border-radius: 6px;
                                        ", BG_ELEVATED, TEXT_PRIMARY, BORDER_DEFAULT)),
                                        on_change: move |v: String| base_url.set(v),
                                        on_blur: move |_| {},
                                        on_keydown: move |_| {},
                                        autofocus: false,
                                    }
                                }

                                // Inputs or Output config
                                if inputs_active {
                                    div {
                                        style: "
                                            flex: 1; padding: 10px; border: 1px solid {BORDER_SUBTLE};
                                            background-color: {BG_SURFACE}; border-radius: 6px;
                                            display: flex; flex-direction: column; gap: 8px;
                                            overflow-y: auto;
                                        ",
                                        div { style: "font-size: 10px; color: {TEXT_DIM}; text-transform: uppercase; letter-spacing: 0.5px;", "Exposed Inputs ({exposed_inputs().len()})" }
                                        if exposed_inputs().is_empty() {
                                            div { style: "font-size: 11px; color: {TEXT_DIM};", "No inputs exposed. Select a node and click 'Expose' on its inputs." }
                                        } else {
                                            for (index, input) in exposed_inputs().iter().enumerate() {
                                                {
                                                    let mut exposed_inputs_clone = exposed_inputs.clone();
                                                    rsx! {
                                                        div {
                                                            key: "input-{index}",
                                                            style: "
                                                                display: flex; flex-direction: column; gap: 6px;
                                                                padding: 8px; border: 1px solid {BORDER_DEFAULT};
                                                                border-radius: 6px; background-color: {BG_ELEVATED};
                                                            ",
                                                            div {
                                                                style: "display: flex; gap: 6px;",
                                                                crate::components::common::StableTextInput {
                                                                    id: format!("input-name-{}", index),
                                                                    value: input.name.clone(),
                                                                    placeholder: Some("name".to_string()),
                                                                style: Some(format!("
                                                                    flex: 1; padding: 4px 6px; font-size: 11px;
                                                                    background-color: {}; color: {};
                                                                    border: 1px solid {}; border-radius: 4px;
                                                                ", BG_SURFACE, TEXT_PRIMARY, BORDER_DEFAULT)),
                                                                on_change: move |v: String| {
                                                                    let mut next = exposed_inputs_clone();
                                                                    if let Some(target) = next.get_mut(index) {
                                                                        target.name = v;
                                                                    }
                                                                    exposed_inputs_clone.set(next);
                                                                },
                                                                on_blur: move |_| {},
                                                                on_keydown: move |_| {},
                                                                autofocus: false,
                                                            }
                                                                crate::components::common::StableTextInput {
                                                                    id: format!("input-label-{}", index),
                                                                    value: input.label.clone(),
                                                                    placeholder: Some("label".to_string()),
                                                                style: Some(format!("
                                                                    flex: 1; padding: 4px 6px; font-size: 11px;
                                                                    background-color: {}; color: {};
                                                                    border: 1px solid {}; border-radius: 4px;
                                                                ", BG_SURFACE, TEXT_PRIMARY, BORDER_DEFAULT)),
                                                                on_change: move |v: String| {
                                                                    let mut next = exposed_inputs();
                                                                    if let Some(target) = next.get_mut(index) {
                                                                        target.label = v;
                                                                    }
                                                                    exposed_inputs.set(next);
                                                                },
                                                                on_blur: move |_| {},
                                                                on_keydown: move |_| {},
                                                                autofocus: false,
                                                            }
                                                                button {
                                                                    class: "collapse-btn",
                                                                    style: "
                                                                        padding: 4px 8px; font-size: 10px;
                                                                        background-color: transparent;
                                                                        border: 1px solid {BORDER_DEFAULT};
                                                                        border-radius: 4px; color: #ef4444;
                                                                        cursor: pointer;
                                                                    ",
                                                                    onclick: move |_| {
                                                                        let mut next = exposed_inputs();
                                                                        if index < next.len() {
                                                                            next.remove(index);
                                                                        }
                                                                        exposed_inputs.set(next);
                                                                    },
                                                                    "×"
                                                                }
                                                            }
                                                            div { style: "font-size: 9px; color: {TEXT_DIM};", "→ {input.selector.class_type}.{input.selector.input_key}" }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    div {
                                        style: "
                                            padding: 10px; border: 1px solid {BORDER_SUBTLE};
                                            background-color: {BG_SURFACE}; border-radius: 6px;
                                            display: flex; flex-direction: column; gap: 8px;
                                        ",
                                        div { style: "font-size: 10px; color: {TEXT_DIM}; text-transform: uppercase; letter-spacing: 0.5px;", "Output Configuration" }
                                        if let Some(out) = output_node() {
                                            div { style: "font-size: 11px; color: {TEXT_PRIMARY};", "Node: {out.title.clone().unwrap_or_else(|| out.class_type.clone())}" }
                                            crate::components::common::StableTextInput {
                                                id: "output-key-input".to_string(),
                                                value: output_key(),
                                                placeholder: Some("Output key (images, videos, etc)".to_string()),
                                                style: Some(format!("
                                                    width: 100%; padding: 6px 8px; font-size: 11px;
                                                    background-color: {}; color: {};
                                                    border: 1px solid {}; border-radius: 6px;
                                                ", BG_ELEVATED, TEXT_PRIMARY, BORDER_DEFAULT)),
                                                on_change: move |v: String| output_key.set(v),
                                                on_blur: move |_| {},
                                                on_keydown: move |_| {},
                                                autofocus: false,
                                            }
                                        } else {
                                            div { style: "font-size: 11px; color: {TEXT_DIM};", "Select a node and click 'Use as Output'." }
                                        }
                                    }
                                }

                                // Save button
                                div {
                                    style: "display: flex; justify-content: flex-end; gap: 8px;",
                                    button {
                                        class: "collapse-btn",
                                        style: "
                                            padding: 8px 16px; font-size: 12px;
                                            background-color: {ACCENT_PRIMARY};
                                            border: none; border-radius: 6px;
                                            color: white; font-weight: 600;
                                            cursor: pointer;
                                        ",
                                        onclick: save_provider,
                                        "Save Provider"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// Helper functions (same as V1)
fn derive_manifest_path(workflow_path: &Path) -> PathBuf {
    let mut path = workflow_path.to_path_buf();
    let old_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("workflow");
    let new_name = format!("{}_manifest.json", old_name);
    path.set_file_name(new_name);
    path
}

fn input_type_to_key(input_type: &ProviderInputType) -> (String, String) {
    match input_type {
        ProviderInputType::Text => ("text".to_string(), String::new()),
        ProviderInputType::Integer => ("integer".to_string(), String::new()),
        ProviderInputType::Number => ("number".to_string(), String::new()),
        ProviderInputType::Boolean => ("boolean".to_string(), String::new()),
        ProviderInputType::Enum { options } => {
            let opts = options.join(",");
            ("enum".to_string(), opts)
        }
        _ => ("text".to_string(), String::new()), // Image/Video/Audio fallback
    }
}

fn default_value_to_text(value: Option<&serde_json::Value>) -> String {
    value
        .map(|v| match v {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            _ => String::new(),
        })
        .unwrap_or_default()
}

fn build_input_ui(input: &BuilderInput) -> Option<InputUi> {
    if input.multiline {
        Some(InputUi {
            multiline: true,
            min: None,
            max: None,
            step: None,
            placeholder: None,
            group: None,
            advanced: false,
            unit: None,
        })
    } else {
        None
    }
}

fn parse_input_type(input: &BuilderInput) -> Result<ProviderInputType, String> {
    match input.input_type_key.as_str() {
        "text" => Ok(ProviderInputType::Text),
        "integer" => Ok(ProviderInputType::Integer),
        "number" => Ok(ProviderInputType::Number),
        "boolean" => Ok(ProviderInputType::Boolean),
        "enum" => {
            let options: Vec<String> = input
                .enum_options
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if options.is_empty() {
                Err("Enum must have at least one option".to_string())
            } else {
                Ok(ProviderInputType::Enum { options })
            }
        }
        _ => Err(format!("Unknown input type: {}", input.input_type_key)),
    }
}

fn parse_default_value(
    input_type: &ProviderInputType,
    text: &str,
) -> Result<Option<serde_json::Value>, String> {
    if text.trim().is_empty() {
        return Ok(None);
    }
    match input_type {
        ProviderInputType::Text => Ok(Some(serde_json::Value::String(text.to_string()))),
        ProviderInputType::Integer => text
            .parse::<i64>()
            .map(|n| Some(serde_json::Value::Number(n.into())))
            .map_err(|_| format!("Invalid integer: {}", text)),
        ProviderInputType::Number => text
            .parse::<f64>()
            .map_err(|_| format!("Invalid number: {}", text))
            .and_then(|f| {
                serde_json::Number::from_f64(f)
                    .ok_or_else(|| "Invalid number".to_string())
            })
            .map(|n| Some(serde_json::Value::Number(n))),
        ProviderInputType::Boolean => text
            .parse::<bool>()
            .map(|b| Some(serde_json::Value::Bool(b)))
            .map_err(|_| format!("Invalid boolean: {}", text)),
        ProviderInputType::Enum { .. } => Ok(Some(serde_json::Value::String(text.to_string()))),
        _ => Ok(Some(serde_json::Value::String(text.to_string()))), // Image/Video/Audio
    }
}

fn friendly_label(name: &str) -> String {
    name.replace('_', " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn output_type_key(value: ProviderOutputType) -> &'static str {
    match value {
        ProviderOutputType::Image => "image",
        ProviderOutputType::Video => "video",
        ProviderOutputType::Audio => "audio",
    }
}

fn parse_output_type(value: &str) -> ProviderOutputType {
    match value {
        "video" => ProviderOutputType::Video,
        "audio" => ProviderOutputType::Audio,
        _ => ProviderOutputType::Image,
    }
}
