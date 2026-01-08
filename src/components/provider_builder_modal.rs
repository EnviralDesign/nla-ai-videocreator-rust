
use dioxus::prelude::*;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::constants::*;
use crate::core::comfyui_workflow::ComfyWorkflowNode;
use crate::core::provider_store::{provider_path_for_entry, write_provider_file};
use crate::state::{
    ComfyOutputSelector, ComfyWorkflowRef, InputBinding, ManifestInput, NodeSelector,
    ProviderConnection, ProviderEntry, ProviderInputField, ProviderInputType, ProviderManifest,
    ProviderOutputType,
};

#[derive(Clone)]
pub struct ProviderBuilderSaved {
    pub provider_path: PathBuf,
}

#[derive(Clone)]
pub enum ProviderBuilderSeed {
    New,
    Edit {
        provider_path: PathBuf,
        provider_entry: ProviderEntry,
        manifest_path: Option<PathBuf>,
        manifest: Option<ProviderManifest>,
        error: Option<String>,
    },
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
    selector: NodeSelectorDraft,
}

#[component]
pub fn ProviderBuilderModal(
    show: Signal<bool>,
    seed: Signal<ProviderBuilderSeed>,
    on_saved: EventHandler<ProviderBuilderSaved>,
) -> Element {
    let mut workflow_path = use_signal(|| None::<PathBuf>);
    let mut workflow_nodes = use_signal(Vec::<ComfyWorkflowNode>::new);
    let mut workflow_error = use_signal(|| None::<String>);
    let mut workflow_search = use_signal(String::new);
    let mut selected_node_id = use_signal(|| None::<String>);

    let mut provider_name = use_signal(|| "New Provider".to_string());
    let mut base_url = use_signal(|| "http://127.0.0.1:8188".to_string());
    let mut output_type = use_signal(|| ProviderOutputType::Image);
    let mut output_key = use_signal(|| "images".to_string());
    let output_tag = use_signal(String::new);
    let output_node = use_signal(|| None::<OutputNodeDraft>);

    let exposed_inputs = use_signal(Vec::<BuilderInput>::new);
    let builder_error = use_signal(|| None::<String>);
    let editing_provider_id = use_signal(|| None::<Uuid>);
    let editing_provider_path = use_signal(|| None::<PathBuf>);
    let editing_manifest_path = use_signal(|| None::<PathBuf>);
    let initial_workflow_path = use_signal(|| None::<PathBuf>);
    let initialized = use_signal(|| false);

    let show_effect = show.clone();
    let seed_effect = seed.clone();
    let mut workflow_path_effect = workflow_path.clone();
    let mut workflow_nodes_effect = workflow_nodes.clone();
    let mut workflow_error_effect = workflow_error.clone();
    let mut workflow_search_effect = workflow_search.clone();
    let mut selected_node_id_effect = selected_node_id.clone();
    let mut provider_name_effect = provider_name.clone();
    let mut base_url_effect = base_url.clone();
    let mut output_type_effect = output_type.clone();
    let mut output_key_effect = output_key.clone();
    let mut output_tag_effect = output_tag.clone();
    let mut output_node_effect = output_node.clone();
    let mut exposed_inputs_effect = exposed_inputs.clone();
    let mut builder_error_effect = builder_error.clone();
    let mut editing_provider_id_effect = editing_provider_id.clone();
    let mut editing_provider_path_effect = editing_provider_path.clone();
    let mut editing_manifest_path_effect = editing_manifest_path.clone();
    let mut initial_workflow_path_effect = initial_workflow_path.clone();
    let mut initialized_effect = initialized.clone();

    use_effect(move || {
        if !show_effect() {
            initialized_effect.set(false);
            return;
        }
        if initialized_effect() {
            return;
        }

        workflow_path_effect.set(None);
        workflow_nodes_effect.set(Vec::new());
        workflow_error_effect.set(None);
        workflow_search_effect.set(String::new());
        selected_node_id_effect.set(None);
        provider_name_effect.set("New Provider".to_string());
        base_url_effect.set("http://127.0.0.1:8188".to_string());
        output_type_effect.set(ProviderOutputType::Image);
        output_key_effect.set("images".to_string());
        output_tag_effect.set(String::new());
        output_node_effect.set(None);
        exposed_inputs_effect.set(Vec::new());
        builder_error_effect.set(None);
        editing_provider_id_effect.set(None);
        editing_provider_path_effect.set(None);
        editing_manifest_path_effect.set(None);
        initial_workflow_path_effect.set(None);

        match seed_effect() {
            ProviderBuilderSeed::New => {}
            ProviderBuilderSeed::Edit {
                provider_path,
                provider_entry,
                manifest_path,
                manifest,
                error,
            } => {
                if let Some(error) = error {
                    builder_error_effect.set(Some(error));
                }

                editing_provider_id_effect.set(Some(provider_entry.id));
                editing_provider_path_effect.set(Some(provider_path));
                editing_manifest_path_effect.set(manifest_path);

                let (base_url, workflow_path_from_connection) = match &provider_entry.connection {
                    ProviderConnection::ComfyUi {
                        base_url,
                        workflow_path,
                        ..
                    } => (base_url.clone(), workflow_path.clone()),
                    _ => {
                        builder_error_effect.set(Some(
                            "Provider Builder only supports ComfyUI providers.".to_string(),
                        ));
                        initialized_effect.set(true);
                        return;
                    }
                };

                provider_name_effect.set(provider_entry.name.clone());
                base_url_effect.set(base_url);
                output_type_effect.set(provider_entry.output_type);

                let mut workflow_path_value = workflow_path_from_connection
                    .as_ref()
                    .map(|path| PathBuf::from(path));

                if let Some(ProviderManifest::ComfyUi {
                    name,
                    output_type,
                    workflow,
                    inputs,
                    output,
                    ..
                }) = manifest
                {
                    if let Some(name) = name {
                        provider_name_effect.set(name);
                    }
                    output_type_effect.set(output_type);
                    workflow_path_value = Some(PathBuf::from(workflow.workflow_path));

                    let mut next_inputs = Vec::new();
                    for input in inputs.into_iter() {
                        let (input_type_key, enum_options) =
                            input_type_to_key(&input.input_type);
                        let default_text = default_value_to_text(input.default.as_ref());
                        next_inputs.push(BuilderInput {
                            name: input.name,
                            label: input.label,
                            input_type_key,
                            required: input.required,
                            default_text,
                            enum_options,
                            tag: input.bind.selector.tag.unwrap_or_default(),
                            selector: NodeSelectorDraft {
                                class_type: input.bind.selector.class_type,
                                input_key: input.bind.selector.input_key,
                                title: input.bind.selector.title,
                            },
                        });
                    }
                    exposed_inputs_effect.set(next_inputs);

                    let output_key = output.selector.input_key;
                    let output_key = if output_key.trim().is_empty() {
                        "images".to_string()
                    } else {
                        output_key
                    };
                    output_key_effect.set(output_key);
                    output_tag_effect.set(output.selector.tag.unwrap_or_default());
                    output_node_effect.set(Some(OutputNodeDraft {
                        class_type: output.selector.class_type,
                        title: output.selector.title,
                    }));
                } else if manifest.is_some() {
                    builder_error_effect.set(Some(
                        "Selected manifest is not a ComfyUI manifest.".to_string(),
                    ));
                }

                if let Some(path) = workflow_path_value.clone() {
                    initial_workflow_path_effect.set(Some(path.clone()));
                    match crate::core::comfyui_workflow::load_workflow_nodes(&path) {
                        Ok(nodes) => {
                            workflow_path_effect.set(Some(path.clone()));
                            workflow_nodes_effect.set(nodes);
                            workflow_error_effect.set(None);
                        }
                        Err(err) => {
                            workflow_path_effect.set(Some(path));
                            workflow_nodes_effect.set(Vec::new());
                            workflow_error_effect.set(Some(err));
                        }
                    }
                }
            }
        }

        initialized_effect.set(true);
    });

    let mut editing_manifest_path_for_pick = editing_manifest_path.clone();
    let mut initial_workflow_path_for_pick = initial_workflow_path.clone();
    let pick_workflow = move |_| {
        let mut dialog = rfd::FileDialog::new();
        if let Ok(root) = std::env::current_dir() {
            let workflows_dir = root.join("workflows");
            if workflows_dir.exists() {
                dialog = dialog.set_directory(workflows_dir);
            }
        }
        if let Some(path) = dialog
            .add_filter("ComfyUI API Workflow", &["json"])
            .set_title("Select ComfyUI Workflow")
            .pick_file()
        {
            match crate::core::comfyui_workflow::load_workflow_nodes(&path) {
                Ok(nodes) => {
                    if let (Some(previous), Some(current_manifest)) = (
                        initial_workflow_path_for_pick(),
                        editing_manifest_path_for_pick(),
                    ) {
                        if current_manifest == derive_manifest_path(&previous) {
                            editing_manifest_path_for_pick
                                .set(Some(derive_manifest_path(&path)));
                        }
                    }
                    workflow_path.set(Some(path.clone()));
                    workflow_nodes.set(nodes);
                    workflow_error.set(None);
                    selected_node_id.set(None);
                    initial_workflow_path_for_pick.set(Some(path));
                }
                Err(err) => {
                    workflow_error.set(Some(err));
                    workflow_nodes.set(Vec::new());
                    selected_node_id.set(None);
                }
            }
        }
    };

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

    let mut add_input_from_node = {
        let mut exposed_inputs = exposed_inputs.clone();
        let mut builder_error = builder_error.clone();
        move |node: &ComfyWorkflowNode, input_key: &str| {
            if exposed_inputs()
                .iter()
                .any(|input| input.name == input_key)
            {
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
    let save_builder = {
        let mut builder_error = builder_error.clone();
        let workflow_path = workflow_path.clone();
        let provider_name = provider_name.clone();
        let base_url = base_url.clone();
        let output_type = output_type.clone();
        let output_key = output_key.clone();
        let output_tag = output_tag.clone();
        let output_node = output_node.clone();
        let editing_provider_id = editing_provider_id.clone();
        let editing_provider_path = editing_provider_path.clone();
        let editing_manifest_path = editing_manifest_path.clone();
        let exposed_inputs = exposed_inputs.clone();
        let mut show = show.clone();
        move |_| {
            builder_error.set(None);

            let workflow_path = match workflow_path() {
                Some(path) => path,
                None => {
                    builder_error.set(Some("Select a workflow first.".to_string()));
                    return;
                }
            };

            let name = provider_name().trim().to_string();
            if name.is_empty() {
                builder_error.set(Some("Provider name is required.".to_string()));
                return;
            }

            let output_node = match output_node() {
                Some(node) => node,
                None => {
                    builder_error.set(Some("Select an output node.".to_string()));
                    return;
                }
            };

            let output_key = output_key().trim().to_string();
            if output_key.is_empty() {
                builder_error.set(Some("Output key is required.".to_string()));
                return;
            }

            let workflow_path_str = workflow_path.to_string_lossy().to_string();
            let manifest_path = editing_manifest_path()
                .unwrap_or_else(|| derive_manifest_path(&workflow_path));
            let manifest_path_str = manifest_path.to_string_lossy().to_string();

            let mut manifest_inputs = Vec::new();
            let mut provider_inputs = Vec::new();
            for input in exposed_inputs().iter() {
                let input_type = match parse_input_type(input) {
                    Ok(input_type) => input_type,
                    Err(err) => {
                        builder_error.set(Some(err));
                        return;
                    }
                };
                let default_value = match parse_default_value(&input_type, &input.default_text) {
                    Ok(value) => value,
                    Err(err) => {
                        builder_error.set(Some(err));
                        return;
                    }
                };
                let tag = input.tag.trim();
                let selector = NodeSelector {
                    tag: if tag.is_empty() {
                        None
                    } else {
                        Some(tag.to_string())
                    },
                    class_type: input.selector.class_type.clone(),
                    input_key: input.selector.input_key.clone(),
                    title: input.selector.title.clone(),
                };
                manifest_inputs.push(ManifestInput {
                    name: input.name.clone(),
                    label: input.label.clone(),
                    input_type: input_type.clone(),
                    required: input.required,
                    default: default_value.clone(),
                    ui: None,
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
                });
            }

            let output_tag_value = output_tag();
            let output_tag = output_tag_value.trim();
            let output_selector = NodeSelector {
                tag: if output_tag.is_empty() {
                    None
                } else {
                    Some(output_tag.to_string())
                },
                class_type: output_node.class_type.clone(),
                input_key: output_key.clone(),
                title: output_node.title.clone(),
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

            let provider_id = editing_provider_id().unwrap_or_else(Uuid::new_v4);
            let mut entry = ProviderEntry {
                id: provider_id,
                name,
                output_type: output_type(),
                inputs: Vec::new(),
                connection: ProviderConnection::ComfyUi {
                    base_url: base_url(),
                    workflow_path: Some(workflow_path_str),
                    manifest_path: Some(manifest_path_str),
                },
            };
            entry.inputs = provider_inputs;

            let manifest_json = match serde_json::to_string_pretty(&manifest) {
                Ok(json) => json,
                Err(err) => {
                    builder_error.set(Some(format!("Failed to serialize manifest: {}", err)));
                    return;
                }
            };
            if let Some(parent) = manifest_path.parent() {
                if let Err(err) = std::fs::create_dir_all(parent) {
                    builder_error.set(Some(format!(
                        "Failed to create manifest folder: {}",
                        err
                    )));
                    return;
                }
            }
            if let Err(err) = std::fs::write(&manifest_path, manifest_json) {
                builder_error.set(Some(format!("Failed to write manifest: {}", err)));
                return;
            }
            let provider_json = match serde_json::to_string_pretty(&entry) {
                Ok(json) => json,
                Err(err) => {
                    builder_error.set(Some(format!("Failed to serialize provider: {}", err)));
                    return;
                }
            };
            let provider_path =
                editing_provider_path().unwrap_or_else(|| provider_path_for_entry(&entry));
            if let Err(err) = write_provider_file(&provider_path, &provider_json) {
                builder_error.set(Some(format!("Failed to save provider: {}", err)));
                return;
            }

            on_saved.call(ProviderBuilderSaved { provider_path });
            show.set(false);
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
                                    if editing_provider_path().is_some() {
                                        "Mode: Edit"
                                    } else {
                                        "Mode: New"
                                    }
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

                    if let Some(error) = workflow_error() {
                        div {
                            style: "padding: 8px 18px; font-size: 11px; color: #f97316;",
                            "{error}"
                        }
                    }
                    if let Some(error) = builder_error() {
                        div {
                            style: "padding: 8px 18px; font-size: 11px; color: #f97316;",
                            "{error}"
                        }
                    }
                    div {
                        style: "flex: 1; display: flex; min-height: 0;",
                        // Left: Node list
                        div {
                            style: "
                                width: 320px; padding: 12px;
                                border-right: 1px solid {BORDER_SUBTLE};
                                background-color: {BG_BASE};
                                display: flex; flex-direction: column; gap: 8px;
                            ",
                            input {
                                style: "
                                    width: 100%; padding: 6px 8px; font-size: 11px;
                                    background-color: {BG_SURFACE}; color: {TEXT_PRIMARY};
                                    border: 1px solid {BORDER_DEFAULT}; border-radius: 6px;
                                    outline: none;
                                ",
                                value: "{workflow_search()}",
                                placeholder: "Search nodes or inputs...",
                                oninput: move |e| workflow_search.set(e.value()),
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
                                        "No nodes found"
                                    }
                                } else {
                                    for node in filtered_nodes.into_iter() {
                                        {
                                            let selected = selected_node_id()
                                                .as_ref()
                                                .map(|id| id == &node.id)
                                                .unwrap_or(false);
                                            let item_bg =
                                                if selected { BG_HOVER } else { "transparent" };
                                            let item_border =
                                                if selected { BORDER_ACCENT } else { BORDER_SUBTLE };
                                            let node_id = node.id.clone();
                                            let title = node
                                                .title
                                                .clone()
                                                .unwrap_or_else(|| "Untitled".to_string());
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
                                                    onclick: move |_| {
                                                        selected_node_id.set(Some(node_id.clone()));
                                                    },
                                                    span { style: "font-weight: 600;", "{title}" }
                                                    span { style: "font-size: 10px; color: {TEXT_DIM};", "{node.class_type}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        // Right: Builder details
                        div {
                            style: "flex: 1; padding: 12px; display: flex; flex-direction: column; gap: 12px; min-width: 0; overflow-y: auto;",
                            div {
                                style: "
                                    padding: 10px; border: 1px solid {BORDER_SUBTLE};
                                    background-color: {BG_SURFACE}; border-radius: 6px;
                                    display: flex; flex-direction: column; gap: 8px;
                                ",
                                div { style: "font-size: 10px; color: {TEXT_DIM}; text-transform: uppercase; letter-spacing: 0.5px;", "Provider Settings" }
                                div {
                                    style: "display: flex; gap: 8px;",
                                    input {
                                        style: "
                                            flex: 1; padding: 6px 8px; font-size: 11px;
                                            background-color: {BG_ELEVATED}; color: {TEXT_PRIMARY};
                                            border: 1px solid {BORDER_DEFAULT}; border-radius: 6px;
                                        ",
                                        value: "{provider_name()}",
                                        placeholder: "Provider name",
                                        oninput: move |e| provider_name.set(e.value()),
                                    }
                                    select {
                                        value: "{output_type_key(output_type())}",
                                        style: "
                                            width: 140px; padding: 6px 8px; font-size: 11px;
                                            background-color: {BG_ELEVATED}; color: {TEXT_PRIMARY};
                                            border: 1px solid {BORDER_DEFAULT}; border-radius: 6px;
                                        ",
                                        onchange: move |e| output_type.set(parse_output_type(&e.value())),
                                        option { value: "image", "Image" }
                                        option { value: "video", "Video" }
                                        option { value: "audio", "Audio" }
                                    }
                                }
                                input {
                                    style: "
                                        width: 100%; padding: 6px 8px; font-size: 11px;
                                        background-color: {BG_ELEVATED}; color: {TEXT_PRIMARY};
                                        border: 1px solid {BORDER_DEFAULT}; border-radius: 6px;
                                    ",
                                    value: "{base_url()}",
                                    placeholder: "ComfyUI base URL (http://127.0.0.1:8188)",
                                    oninput: move |e| base_url.set(e.value()),
                                }
                            }

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
                                div {
                                    style: "
                                        padding: 10px; border: 1px solid {BORDER_SUBTLE};
                                        background-color: {BG_SURFACE}; border-radius: 6px;
                                        display: flex; flex-direction: column; gap: 6px;
                                    ",
                                    div { style: "font-size: 10px; color: {TEXT_DIM}; text-transform: uppercase; letter-spacing: 0.5px;", "Output" }
                                    div {
                                        style: "display: flex; gap: 8px;",
                                        input {
                                            style: "
                                                flex: 1; padding: 6px 8px; font-size: 11px;
                                                background-color: {BG_ELEVATED}; color: {TEXT_PRIMARY};
                                                border: 1px solid {BORDER_DEFAULT}; border-radius: 6px;
                                            ",
                                            value: "{output_key()}",
                                            placeholder: "Output key (images)",
                                            oninput: move |e| output_key.set(e.value()),
                                        }
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
                                            "Use Node"
                                        }
                                    }
                                    div {
                                        style: "font-size: 10px; color: {TEXT_DIM};",
                                        if let Some(output) = output_node() {
                                            "{output.class_type} {output.title.clone().unwrap_or_else(|| \"\".to_string())}"
                                        } else {
                                            "No output node selected."
                                        }
                                    }
                                }
                            } else {
                                div {
                                    style: "font-size: 11px; color: {TEXT_DIM};",
                                    "Select a node to inspect inputs and set output."
                                }
                            }
                            div {
                                style: "
                                    padding: 10px; border: 1px solid {BORDER_SUBTLE};
                                    background-color: {BG_SURFACE}; border-radius: 6px;
                                    display: flex; flex-direction: column; gap: 8px;
                                ",
                                div { style: "font-size: 10px; color: {TEXT_DIM}; text-transform: uppercase; letter-spacing: 0.5px;", "Exposed Inputs" }
                                if exposed_inputs().is_empty() {
                                    div { style: "font-size: 11px; color: {TEXT_DIM};", "Expose inputs from a node to build the provider UI." }
                                } else {
                                    for (index, input) in exposed_inputs().iter().enumerate() {
                                        {
                                            let mut exposed_inputs = exposed_inputs.clone();
                                            let input_clone = input.clone();
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
                                                        input {
                                                            style: "
                                                                flex: 1; padding: 6px 8px; font-size: 11px;
                                                                background-color: {BG_SURFACE}; color: {TEXT_PRIMARY};
                                                                border: 1px solid {BORDER_DEFAULT}; border-radius: 6px;
                                                            ",
                                                            value: "{input.name}",
                                                            placeholder: "name",
                                                            oninput: move |e| {
                                                                let mut next = exposed_inputs();
                                                                if let Some(target) = next.get_mut(index) {
                                                                    target.name = e.value();
                                                                }
                                                                exposed_inputs.set(next);
                                                            },
                                                        }
                                                        input {
                                                            style: "
                                                                flex: 1; padding: 6px 8px; font-size: 11px;
                                                                background-color: {BG_SURFACE}; color: {TEXT_PRIMARY};
                                                                border: 1px solid {BORDER_DEFAULT}; border-radius: 6px;
                                                            ",
                                                            value: "{input.label}",
                                                            placeholder: "label",
                                                            oninput: move |e| {
                                                                let mut next = exposed_inputs();
                                                                if let Some(target) = next.get_mut(index) {
                                                                    target.label = e.value();
                                                                }
                                                                exposed_inputs.set(next);
                                                            },
                                                        }
                                                        button {
                                                            class: "collapse-btn",
                                                            style: "
                                                                padding: 6px 8px; font-size: 11px;
                                                                background-color: transparent;
                                                                border: 1px solid {BORDER_DEFAULT};
                                                                border-radius: 6px; color: #ef4444;
                                                                cursor: pointer;
                                                            ",
                                                            onclick: move |_| {
                                                                let mut next = exposed_inputs();
                                                                if index < next.len() {
                                                                    next.remove(index);
                                                                }
                                                                exposed_inputs.set(next);
                                                            },
                                                            "Remove"
                                                        }
                                                    }
                                                    div {
                                                        style: "display: flex; gap: 6px; align-items: center;",
                                                        select {
                                                            value: "{input.input_type_key}",
                                                            style: "
                                                                width: 140px; padding: 6px 8px; font-size: 11px;
                                                                background-color: {BG_SURFACE}; color: {TEXT_PRIMARY};
                                                                border: 1px solid {BORDER_DEFAULT}; border-radius: 6px;
                                                            ",
                                                            onchange: move |e| {
                                                                let mut next = exposed_inputs();
                                                                if let Some(target) = next.get_mut(index) {
                                                                    target.input_type_key = e.value();
                                                                }
                                                                exposed_inputs.set(next);
                                                            },
                                                            option { value: "text", "Text" }
                                                            option { value: "number", "Number" }
                                                            option { value: "integer", "Integer" }
                                                            option { value: "boolean", "Boolean" }
                                                            option { value: "enum", "Enum" }
                                                            option { value: "image", "Image" }
                                                            option { value: "video", "Video" }
                                                            option { value: "audio", "Audio" }
                                                        }
                                                        input {
                                                            style: "
                                                                flex: 1; padding: 6px 8px; font-size: 11px;
                                                                background-color: {BG_SURFACE}; color: {TEXT_PRIMARY};
                                                                border: 1px solid {BORDER_DEFAULT}; border-radius: 6px;
                                                            ",
                                                            value: "{input.default_text}",
                                                            placeholder: "default (optional)",
                                                            oninput: move |e| {
                                                                let mut next = exposed_inputs();
                                                                if let Some(target) = next.get_mut(index) {
                                                                    target.default_text = e.value();
                                                                }
                                                                exposed_inputs.set(next);
                                                            },
                                                        }
                                                        label {
                                                            style: "font-size: 10px; color: {TEXT_DIM}; display: flex; gap: 6px; align-items: center;",
                                                            input {
                                                                r#type: "checkbox",
                                                                checked: input.required,
                                                                onchange: move |_| {
                                                                    let mut next = exposed_inputs();
                                                                    if let Some(target) = next.get_mut(index) {
                                                                        target.required = !target.required;
                                                                    }
                                                                    exposed_inputs.set(next);
                                                                },
                                                            }
                                                            "Required"
                                                        }
                                                    }
                                                    if input.input_type_key == "enum" {
                                                        input {
                                                            style: "
                                                                width: 100%; padding: 6px 8px; font-size: 11px;
                                                                background-color: {BG_SURFACE}; color: {TEXT_PRIMARY};
                                                                border: 1px solid {BORDER_DEFAULT}; border-radius: 6px;
                                                            ",
                                                            value: "{input.enum_options}",
                                                            placeholder: "Enum options (comma-separated)",
                                                            oninput: move |e| {
                                                                let mut next = exposed_inputs();
                                                                if let Some(target) = next.get_mut(index) {
                                                                    target.enum_options = e.value();
                                                                }
                                                                exposed_inputs.set(next);
                                                            },
                                                        }
                                                    }
                                                    div {
                                                        style: "font-size: 10px; color: {TEXT_DIM};",
                                                        "Bind: {input_clone.selector.class_type} / {input_clone.selector.input_key}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    div {
                        style: "
                            padding: 12px 18px;
                            border-top: 1px solid {BORDER_DEFAULT};
                            background-color: {BG_SURFACE};
                            display: flex; align-items: center; justify-content: flex-end; gap: 8px;
                        ",
                        button {
                            class: "collapse-btn",
                            style: "
                                padding: 6px 12px;
                                background-color: transparent;
                                border: 1px solid {BORDER_DEFAULT};
                                border-radius: 6px; color: {TEXT_SECONDARY};
                                font-size: 11px; cursor: pointer;
                            ",
                            onclick: move |_| show.set(false),
                            "Cancel"
                        }
                        button {
                            class: "collapse-btn",
                            style: "
                                padding: 6px 12px;
                                background-color: {BG_ELEVATED};
                                border: 1px solid {BORDER_DEFAULT};
                                border-radius: 6px; color: {TEXT_PRIMARY};
                                font-size: 11px; cursor: pointer;
                            ",
                            onclick: save_builder,
                            "Save Provider"
                        }
                    }
                }
            }
        }
    }
}
fn derive_manifest_path(path: &Path) -> PathBuf {
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("workflow");
    let file_name = format!("{}_manifest.json", stem);
    path.with_file_name(file_name)
}

fn friendly_label(name: &str) -> String {
    let mut out = String::new();
    let mut uppercase_next = true;
    for ch in name.chars() {
        if ch == '_' || ch == '-' {
            out.push(' ');
            uppercase_next = true;
            continue;
        }
        if uppercase_next {
            out.extend(ch.to_uppercase());
            uppercase_next = false;
        } else {
            out.push(ch);
        }
    }
    out
}
fn parse_output_type(value: &str) -> ProviderOutputType {
    match value {
        "video" => ProviderOutputType::Video,
        "audio" => ProviderOutputType::Audio,
        _ => ProviderOutputType::Image,
    }
}

fn output_type_key(value: ProviderOutputType) -> &'static str {
    match value {
        ProviderOutputType::Image => "image",
        ProviderOutputType::Video => "video",
        ProviderOutputType::Audio => "audio",
    }
}

fn input_type_to_key(input_type: &ProviderInputType) -> (String, String) {
    match input_type {
        ProviderInputType::Text => ("text".to_string(), String::new()),
        ProviderInputType::Number => ("number".to_string(), String::new()),
        ProviderInputType::Integer => ("integer".to_string(), String::new()),
        ProviderInputType::Boolean => ("boolean".to_string(), String::new()),
        ProviderInputType::Image => ("image".to_string(), String::new()),
        ProviderInputType::Video => ("video".to_string(), String::new()),
        ProviderInputType::Audio => ("audio".to_string(), String::new()),
        ProviderInputType::Enum { options } => ("enum".to_string(), options.join(", ")),
    }
}

fn default_value_to_text(value: Option<&serde_json::Value>) -> String {
    match value {
        Some(serde_json::Value::String(text)) => text.clone(),
        Some(serde_json::Value::Number(number)) => number.to_string(),
        Some(serde_json::Value::Bool(flag)) => flag.to_string(),
        Some(other) => other.to_string(),
        None => String::new(),
    }
}

fn parse_input_type(input: &BuilderInput) -> Result<ProviderInputType, String> {
    let key = input.input_type_key.as_str();
    match key {
        "text" => Ok(ProviderInputType::Text),
        "number" => Ok(ProviderInputType::Number),
        "integer" => Ok(ProviderInputType::Integer),
        "boolean" => Ok(ProviderInputType::Boolean),
        "image" => Ok(ProviderInputType::Image),
        "video" => Ok(ProviderInputType::Video),
        "audio" => Ok(ProviderInputType::Audio),
        "enum" => {
            let options: Vec<String> = input
                .enum_options
                .split(',')
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .collect();
            if options.is_empty() {
                return Err(format!(
                    "Enum input '{}' needs at least one option.",
                    input.name
                ));
            }
            Ok(ProviderInputType::Enum { options })
        }
        _ => Err(format!(
            "Unsupported input type '{}' for {}.",
            input.input_type_key, input.name
        )),
    }
}

fn parse_default_value(
    input_type: &ProviderInputType,
    text: &str,
) -> Result<Option<serde_json::Value>, String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let value = match input_type {
        ProviderInputType::Text => serde_json::Value::String(trimmed.to_string()),
        ProviderInputType::Number => {
            let parsed = trimmed
                .parse::<f64>()
                .map_err(|_| format!("Invalid number default '{}'.", trimmed))?;
            let number = serde_json::Number::from_f64(parsed)
                .ok_or_else(|| format!("Invalid number default '{}'.", trimmed))?;
            serde_json::Value::Number(number)
        }
        ProviderInputType::Integer => {
            let parsed = trimmed
                .parse::<i64>()
                .map_err(|_| format!("Invalid integer default '{}'.", trimmed))?;
            serde_json::Value::Number(parsed.into())
        }
        ProviderInputType::Boolean => {
            let parsed = trimmed
                .parse::<bool>()
                .map_err(|_| format!("Invalid boolean default '{}'.", trimmed))?;
            serde_json::Value::Bool(parsed)
        }
        ProviderInputType::Enum { .. } => serde_json::Value::String(trimmed.to_string()),
        ProviderInputType::Image
        | ProviderInputType::Video
        | ProviderInputType::Audio => return Ok(None),
    };
    Ok(Some(value))
}
