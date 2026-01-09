use dioxus::prelude::*;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::Rc;

use crate::components::common::{NumericField, ProviderTextField};
use super::generative_controls::render_generative_controls;
use super::provider_inputs::render_provider_inputs;
use crate::constants::*;
use crate::core::generation::{next_version_label, resolve_provider_inputs};
use crate::providers::comfyui;
use crate::state::{
    asset_display_name,
    delete_generative_version_files,
    generative_info_for_clip,
    parse_version_index,
    ProviderConnection,
    ProviderEntry,
    ProviderOutputType,
};

#[component]
pub fn AttributesPanelContent(
    project: Signal<crate::state::Project>,
    selection: Signal<crate::state::SelectionState>,
    preview_dirty: Signal<bool>,
    providers: Signal<Vec<ProviderEntry>>,
    previewer: Signal<std::sync::Arc<crate::core::preview::PreviewRenderer>>,
    thumbnailer: std::sync::Arc<crate::core::thumbnailer::Thumbnailer>,
    thumbnail_cache_buster: Signal<u64>,
) -> Element {
    let gen_config = use_signal(|| None::<crate::state::GenerativeConfig>);
    let mut gen_status = use_signal(|| None::<String>);
    let mut gen_busy = use_signal(|| false);
    let mut last_clip_id = use_signal(|| None::<uuid::Uuid>);

    use_effect(move || {
        let mut gen_config = gen_config.clone();
        let selection_state = selection.read();
        let selected_count = selection_state.clip_ids.len();
        let selected_clip_id = selection_state.primary_clip();
        drop(selection_state);

        if selected_count != 1 {
            gen_config.set(None);
            return;
        }

        let Some(clip_id) = selected_clip_id else {
            gen_config.set(None);
            return;
        };

        let project_read = project.read();
        let Some((folder, _output)) = generative_info_for_clip(&project_read, clip_id) else {
            gen_config.set(None);
            return;
        };
        let Some(project_root) = project_read.project_path.clone() else {
            gen_config.set(None);
            return;
        };
        let folder_path = project_root.join(folder);
        drop(project_read);

        let config = crate::state::GenerativeConfig::load(&folder_path).unwrap_or_default();
        gen_config.set(Some(config));
    });

    let selection_state = selection.read();
    let selected_count = selection_state.clip_ids.len();
    let selected_clip_id = selection_state.primary_clip();
    drop(selection_state);

    use_effect(move || {
        if last_clip_id() != selected_clip_id {
            last_clip_id.set(selected_clip_id);
            gen_status.set(None);
            gen_busy.set(false);
        }
    });

    if selected_count == 0 {
        return rsx! {
            div {
                style: "padding: 12px;",
                div {
                    style: "
                        display: flex; align-items: center; justify-content: center;
                        height: 80px; border: 1px dashed {BORDER_DEFAULT}; border-radius: 6px;
                        color: {TEXT_DIM}; font-size: 12px;
                    ",
                    "No selection"
                }
            }
        };
    }

    if selected_count > 1 {
        return rsx! {
            div {
                style: "padding: 12px;",
                div {
                    style: "
                        display: flex; align-items: center; justify-content: center;
                        height: 80px; border: 1px dashed {BORDER_DEFAULT}; border-radius: 6px;
                        color: {TEXT_DIM}; font-size: 12px;
                    ",
                    "{selected_count} items selected"
                }
            }
        };
    }

    let Some(clip_id) = selected_clip_id else {
        return rsx! {
            div {
                style: "padding: 12px;",
                div {
                    style: "
                        display: flex; align-items: center; justify-content: center;
                        height: 80px; border: 1px dashed {BORDER_DEFAULT}; border-radius: 6px;
                        color: {TEXT_DIM}; font-size: 12px;
                    ",
                    "Selection missing"
                }
            }
        };
    };

    let project_read = project.read();
    let clip = match project_read.clips.iter().find(|c| c.id == clip_id) {
        Some(clip) => clip.clone(),
        None => {
            drop(project_read);
            return rsx! {
                div {
                    style: "padding: 12px;",
                    div {
                        style: "
                            display: flex; align-items: center; justify-content: center;
                            height: 80px; border: 1px dashed {BORDER_DEFAULT}; border-radius: 6px;
                            color: {TEXT_DIM}; font-size: 12px;
                        ",
                        "Selection missing"
                    }
                }
            };
        }
    };

    let asset = project_read.find_asset(clip.asset_id).cloned();
    let asset_display = asset
        .as_ref()
        .map(asset_display_name)
        .unwrap_or_else(|| "Unknown".to_string());
    let project_root = project_read.project_path.clone();
    let generative_info = asset.as_ref().and_then(|asset| match &asset.kind {
        crate::state::AssetKind::GenerativeVideo { folder, .. } => {
            Some((folder.clone(), ProviderOutputType::Video))
        }
        crate::state::AssetKind::GenerativeImage { folder, .. } => {
            Some((folder.clone(), ProviderOutputType::Image))
        }
        crate::state::AssetKind::GenerativeAudio { folder, .. } => {
            Some((folder.clone(), ProviderOutputType::Audio))
        }
        _ => None,
    });
    drop(project_read);

    let gen_output = generative_info.as_ref().map(|(_, output)| *output);
    let gen_folder_path = generative_info.as_ref().and_then(|(folder, _)| {
        project_root.as_ref().map(|root| root.join(folder))
    });
    let providers_list = providers.read().clone();
    let compatible_providers: Vec<ProviderEntry> = match gen_output {
        Some(output) => providers_list
            .iter()
            .filter(|entry| entry.output_type == output)
            .cloned()
            .collect(),
        None => Vec::new(),
    };
    let config_snapshot = gen_config().unwrap_or_default();
    let selected_provider_id = config_snapshot.provider_id;
    let selected_provider = selected_provider_id.and_then(|id| {
        compatible_providers
            .iter()
            .find(|entry| entry.id == id)
            .cloned()
    });
    let selected_provider_value = selected_provider_id
        .map(|id| id.to_string())
        .unwrap_or_default();
    let show_missing_provider = selected_provider_id.is_some() && selected_provider.is_none();
    let providers_path_label = crate::core::provider_store::global_providers_root()
        .display()
        .to_string();
    let mut version_options: Vec<String> = config_snapshot
        .versions
        .iter()
        .map(|record| record.version.clone())
        .collect();
    if let Some(active_version) = config_snapshot.active_version.clone() {
        if !version_options.contains(&active_version) {
            version_options.push(active_version);
        }
    }
    version_options.sort_by(|a, b| match (parse_version_index(a), parse_version_index(b)) {
        (Some(a_num), Some(b_num)) => b_num.cmp(&a_num),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => b.cmp(a),
    });
    version_options.dedup();
    let selected_version_value = config_snapshot
        .active_version
        .clone()
        .unwrap_or_default();
    let confirm_delete_version = use_signal(|| false);
    let can_delete_version = !selected_version_value.trim().is_empty();
    let on_provider_change = {
        let mut gen_config = gen_config.clone();
        let gen_folder_path = gen_folder_path.clone();
        Rc::new(RefCell::new(move |e: FormEvent| {
            let value = e.value();
            let provider_id = value
                .trim()
                .parse::<uuid::Uuid>()
                .ok();
            let mut config = gen_config().unwrap_or_default();
            config.provider_id = provider_id;
            if let Some(folder_path) = gen_folder_path.as_ref() {
                if let Err(err) = config.save(folder_path) {
                    println!("Failed to save generative config: {}", err);
                }
            }
            gen_config.set(Some(config));
        }))
    };
    let on_version_change = {
        let mut gen_config = gen_config.clone();
        let gen_folder_path = gen_folder_path.clone();
        let asset_id = clip.asset_id;
        let mut project = project.clone();
        let mut preview_dirty = preview_dirty.clone();
        let thumbnailer = thumbnailer.clone();
        let thumbnail_cache_buster = thumbnail_cache_buster.clone();
        let mut confirm_delete_version = confirm_delete_version.clone();
        Rc::new(RefCell::new(move |e: FormEvent| {
            let value = e.value();
            let trimmed = value.trim();
            let next_version = if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            };
            let mut config = gen_config().unwrap_or_default();
            config.active_version = next_version.clone();
            if let Some(version) = next_version.as_ref() {
                if let Some(record) = config.versions.iter().find(|record| record.version == *version) {
                    config.inputs = record.inputs_snapshot.clone();
                    config.provider_id = Some(record.provider_id);
                }
            }
            if let Some(folder_path) = gen_folder_path.as_ref() {
                if let Err(err) = config.save(folder_path) {
                    println!("Failed to save generative config: {}", err);
                }
            }
            gen_config.set(Some(config));
            project
                .write()
                .set_generative_active_version(asset_id, next_version);
            preview_dirty.set(true);
            if let Some(asset) = project.read().find_asset(asset_id).cloned() {
                let thumbs = thumbnailer.clone();
                let mut thumbnail_cache_buster = thumbnail_cache_buster.clone();
                spawn(async move {
                    thumbs.generate(&asset, true).await;
                    thumbnail_cache_buster.set(thumbnail_cache_buster() + 1);
                });
            }
            confirm_delete_version.set(false);
        }))
    };
    let on_delete_version = {
        let gen_config = gen_config.clone();
        let gen_folder_path = gen_folder_path.clone();
        let asset_id = clip.asset_id;
        let project = project.clone();
        let preview_dirty = preview_dirty.clone();
        let previewer = previewer.clone();
        let thumbnailer = thumbnailer.clone();
        let thumbnail_cache_buster = thumbnail_cache_buster.clone();
        let gen_status = gen_status.clone();
        let confirm_delete_version = confirm_delete_version.clone();
        let version_options = version_options.clone();
        let selected_version_value = selected_version_value.clone();
        Rc::new(RefCell::new(move || {
            let mut gen_config = gen_config.clone();
            let gen_folder_path = gen_folder_path.clone();
            let mut project = project.clone();
            let mut preview_dirty = preview_dirty.clone();
            let previewer = previewer.clone();
            let thumbnailer = thumbnailer.clone();
            let thumbnail_cache_buster = thumbnail_cache_buster.clone();
            let mut gen_status = gen_status.clone();
            let mut confirm_delete_version = confirm_delete_version.clone();
            let version_options = version_options.clone();
            let selected_version_value = selected_version_value.clone();

            let version = selected_version_value.trim().to_string();
            if version.is_empty() {
                return;
            }
            let Some(folder_path) = gen_folder_path.as_ref() else {
                gen_status.set(Some("Missing generative folder.".to_string()));
                return;
            };

            let mut remaining = version_options.clone();
            let deleted_index = match remaining.iter().position(|item| item == &version) {
                Some(index) => {
                    remaining.remove(index);
                    index
                }
                None => 0,
            };
            let next_active = if remaining.is_empty() {
                None
            } else {
                let next_index = deleted_index.min(remaining.len() - 1);
                remaining.get(next_index).cloned()
            };

            let folder_path = folder_path.clone();
            let delete_folder = folder_path.clone();
            let version_clone = version.clone();
            let next_active_clone = next_active.clone();
            spawn(async move {
                let deletion = tokio::task::spawn_blocking(move || {
                    delete_generative_version_files(&delete_folder, &version_clone)
                })
                .await
                .ok()
                .unwrap_or_else(|| Err("Failed to delete version files.".to_string()));

                if let Err(err) = deletion {
                    gen_status.set(Some(format!("Delete failed: {}", err)));
                    confirm_delete_version.set(false);
                    return;
                }
                previewer.read().invalidate_folder(&folder_path);

                let mut config = gen_config().unwrap_or_default();
                config.versions.retain(|record| record.version != version);
                config.active_version = next_active_clone.clone();
                if let Err(err) = config.save(&folder_path) {
                    gen_status.set(Some(format!("Failed to save config: {}", err)));
                    confirm_delete_version.set(false);
                    return;
                }
                gen_config.set(Some(config));

                project
                    .write()
                    .set_generative_active_version(asset_id, next_active_clone.clone());
                preview_dirty.set(true);

                if let Some(asset) = project.read().find_asset(asset_id).cloned() {
                    let thumbs = thumbnailer.clone();
                    let mut thumbnail_cache_buster = thumbnail_cache_buster.clone();
                    spawn(async move {
                        thumbs.generate(&asset, true).await;
                        thumbnail_cache_buster.set(thumbnail_cache_buster() + 1);
                    });
                }

                confirm_delete_version.set(false);
                gen_status.set(Some(format!("Deleted {}", version)));
            });
        }))
    };

    let set_input_value = {
        let mut gen_config = gen_config.clone();
        let gen_folder_path = gen_folder_path.clone();
        Rc::new(RefCell::new(move |name: String, value: serde_json::Value| {
            let mut config = gen_config().unwrap_or_default();
            config.inputs.insert(
                name,
                crate::state::InputValue::Literal { value },
            );
            if let Some(folder_path) = gen_folder_path.as_ref() {
                if let Err(err) = config.save(folder_path) {
                    println!("Failed to save generative config: {}", err);
                }
            }
            gen_config.set(Some(config));
        }))
    };

    let on_generate = {
        let project = project.clone();
        let gen_config = gen_config.clone();
        let gen_folder_path = gen_folder_path.clone();
        let gen_status = gen_status.clone();
        let gen_busy = gen_busy.clone();
        let preview_dirty = preview_dirty.clone();
        let previewer = previewer.clone();
        let thumbnailer = thumbnailer.clone();
        let thumbnail_cache_buster = thumbnail_cache_buster.clone();
        let selected_provider = selected_provider.clone();
        let asset_id = clip.asset_id;
        Rc::new(RefCell::new(move |_evt: MouseEvent| {
            let mut project = project.clone();
            let mut gen_config = gen_config.clone();
            let gen_folder_path = gen_folder_path.clone();
            let mut gen_status = gen_status.clone();
            let mut gen_busy = gen_busy.clone();
            let mut preview_dirty = preview_dirty.clone();
            let previewer = previewer.clone();
            let thumbnailer = thumbnailer.clone();
            let thumbnail_cache_buster = thumbnail_cache_buster.clone();
            let selected_provider = selected_provider.clone();

            if gen_busy() {
                return;
            }

            let Some(provider) = selected_provider.clone() else {
                gen_status.set(Some("Select a provider first.".to_string()));
                return;
            };
            let Some(folder_path) = gen_folder_path.clone() else {
                gen_status.set(Some("Missing generative folder.".to_string()));
                return;
            };

            let mut config = gen_config().unwrap_or_default();
            if config.provider_id.is_none() {
                config.provider_id = Some(provider.id);
            }

            let resolved = resolve_provider_inputs(&provider, &config);
            if !resolved.missing_required.is_empty() {
                gen_status.set(Some(format!(
                    "Missing inputs: {}",
                    resolved.missing_required.join(", ")
                )));
                return;
            }

            let resolved_inputs = resolved.values.clone();
            let input_snapshot = resolved.snapshot.clone();

            gen_status.set(Some("Queued...".to_string()));
            gen_busy.set(true);

            spawn(async move {
                let result = match provider.connection.clone() {
                    ProviderConnection::ComfyUi {
                        base_url,
                        workflow_path,
                        manifest_path,
                        ..
                    } => {
                        let workflow_path =
                            comfyui::resolve_workflow_path(workflow_path.as_deref());
                        let manifest_path =
                            comfyui::resolve_manifest_path(manifest_path.as_deref());
                        comfyui::generate_image(
                            &base_url,
                            &workflow_path,
                            &resolved_inputs,
                            manifest_path.as_deref(),
                        )
                            .await
                    }
                    _ => Err("Provider connection not supported yet.".to_string()),
                };

                match result {
                    Ok(image) => {
                        let mut config = gen_config().unwrap_or_default();
                        let version = next_version_label(&config);
                        let _ = std::fs::create_dir_all(&folder_path);
                        let output_path = folder_path.join(format!(
                            "{}.{}",
                            version, image.extension
                        ));
                        if let Err(err) = std::fs::write(&output_path, &image.bytes) {
                            gen_status.set(Some(format!(
                                "Failed to save output: {}",
                                err
                            )));
                            gen_busy.set(false);
                            return;
                        }
                        previewer.read().invalidate_folder(&folder_path);

                        config.provider_id = Some(provider.id);
                        config.active_version = Some(version.clone());
                        config.inputs = input_snapshot.clone();
                        config.versions.push(crate::state::GenerationRecord {
                            version: version.clone(),
                            timestamp: chrono::Utc::now(),
                            provider_id: provider.id,
                            inputs_snapshot: input_snapshot.clone(),
                        });
                        if let Err(err) = config.save(&folder_path) {
                            gen_status.set(Some(format!(
                                "Failed to save config: {}",
                                err
                            )));
                            gen_busy.set(false);
                            return;
                        }
                        gen_config.set(Some(config));

                        project
                            .write()
                            .set_generative_active_version(asset_id, Some(version.clone()));
                        preview_dirty.set(true);

                        let maybe_asset = project.read().find_asset(asset_id).cloned();
                        if let Some(asset) = maybe_asset {
                            let thumbs = thumbnailer.clone();
                            let mut thumbnail_cache_buster = thumbnail_cache_buster.clone();
                            spawn(async move {
                                thumbs.generate(&asset, true).await;
                                thumbnail_cache_buster.set(thumbnail_cache_buster() + 1);
                            });
                        }

                        gen_status.set(Some(format!("Generated {}", version)));
                    }
                    Err(err) => {
                        gen_status.set(Some(format!("Generation failed: {}", err)));
                    }
                }

                gen_busy.set(false);
            });
        }))
    };

    let transform = clip.transform;
    let clip_id = clip.id;
    let clip_label = clip.label.clone().unwrap_or_default();
    let generate_label = if gen_busy() { "Generating..." } else { "Generate" };
    let generate_opacity = if gen_busy() { "0.6" } else { "1.0" };

    rsx! {
        div {
            style: "padding: 12px; display: flex; flex-direction: column; gap: 12px;",
            div {
                style: "display: flex; flex-direction: column; gap: 8px;",
                span { style: "font-size: 11px; color: {TEXT_MUTED}; text-transform: uppercase; letter-spacing: 0.5px;", "Clip" }
                div {
                    style: "display: flex; flex-direction: column; gap: 4px;",
                    span { style: "font-size: 10px; color: {TEXT_MUTED};", "Asset" }
                    span { style: "font-size: 12px; color: {TEXT_PRIMARY};", "{asset_display}" }
                }
                ProviderTextField {
                    label: "Clip Name".to_string(),
                    value: clip_label.clone(),
                    on_commit: move |next: String| {
                        let trimmed = next.trim();
                        let label = if trimmed.is_empty() {
                            None
                        } else {
                            Some(trimmed.to_string())
                        };
                        project.write().set_clip_label(clip_id, label);
                    }
                }
            }

            div {
                style: "
                    display: flex; flex-direction: column; gap: 10px;
                    padding: 10px; background-color: {BG_SURFACE};
                    border: 1px solid {BORDER_SUBTLE}; border-radius: 6px;
                ",
                div {
                    style: "font-size: 10px; color: {TEXT_DIM}; text-transform: uppercase; letter-spacing: 0.5px;",
                    "Transform"
                }
                div {
                    style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(70px, 1fr)); gap: 8px;",
                    NumericField {
                        key: "{clip_id}-position-x",
                        label: "Position X",
                        value: transform.position_x,
                        step: "1",
                        clamp_min: None,
                        clamp_max: None,
                        on_commit: move |value| {
                            update_clip_transform(project, clip_id, |transform| {
                                transform.position_x = value;
                            });
                            preview_dirty.set(true);
                        }
                    }
                    NumericField {
                        key: "{clip_id}-position-y",
                        label: "Position Y",
                        value: transform.position_y,
                        step: "1",
                        clamp_min: None,
                        clamp_max: None,
                        on_commit: move |value| {
                            update_clip_transform(project, clip_id, |transform| {
                                transform.position_y = value;
                            });
                            preview_dirty.set(true);
                        }
                    }
                    NumericField {
                        key: "{clip_id}-scale-x",
                        label: "Scale X",
                        value: transform.scale_x,
                        step: "0.01",
                        clamp_min: Some(0.01),
                        clamp_max: None,
                        on_commit: move |value| {
                            update_clip_transform(project, clip_id, |transform| {
                                transform.scale_x = value;
                            });
                            preview_dirty.set(true);
                        }
                    }
                    NumericField {
                        key: "{clip_id}-scale-y",
                        label: "Scale Y",
                        value: transform.scale_y,
                        step: "0.01",
                        clamp_min: Some(0.01),
                        clamp_max: None,
                        on_commit: move |value| {
                            update_clip_transform(project, clip_id, |transform| {
                                transform.scale_y = value;
                            });
                            preview_dirty.set(true);
                        }
                    }
                    NumericField {
                        key: "{clip_id}-rotation",
                        label: "Rotation",
                        value: transform.rotation_deg,
                        step: "1",
                        clamp_min: None,
                        clamp_max: None,
                        on_commit: move |value| {
                            update_clip_transform(project, clip_id, |transform| {
                                transform.rotation_deg = value;
                            });
                            preview_dirty.set(true);
                        }
                    }
                    NumericField {
                        key: "{clip_id}-opacity",
                        label: "Opacity",
                        value: transform.opacity,
                        step: "0.05",
                        clamp_min: Some(0.0),
                        clamp_max: Some(1.0),
                        on_commit: move |value| {
                            update_clip_transform(project, clip_id, |transform| {
                                transform.opacity = value;
                            });
                            preview_dirty.set(true);
                        }
                    }
                }
            }

            if gen_output.is_some() {
                {render_generative_controls(
                    &version_options,
                    &selected_version_value,
                    confirm_delete_version,
                    can_delete_version,
                    on_version_change,
                    on_delete_version.clone(),
                    &selected_provider_value,
                    &compatible_providers,
                    on_provider_change,
                    show_missing_provider,
                    &providers_path_label,
                    on_generate,
                    gen_status,
                    generate_label,
                    generate_opacity,
                )}
                {render_provider_inputs(
                    selected_provider,
                    show_missing_provider,
                    &config_snapshot,
                    &selected_version_value,
                    set_input_value.clone(),
                )}
            }

        }
    }
}

fn update_clip_transform(
    mut project: Signal<crate::state::Project>,
    clip_id: uuid::Uuid,
    update: impl FnOnce(&mut crate::state::ClipTransform),
) {
    if let Some(clip) = project.write().clips.iter_mut().find(|clip| clip.id == clip_id) {
        update(&mut clip.transform);
    }
}

