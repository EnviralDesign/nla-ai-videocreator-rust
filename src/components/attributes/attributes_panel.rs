use dioxus::prelude::*;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::Rc;

use crate::components::common::{NumericField, ProviderTextField};
use super::generative_controls::render_generative_controls;
use super::provider_inputs::render_provider_inputs;
use crate::constants::*;
use crate::core::generation::{
    random_seed_i64, resolve_provider_inputs, resolve_seed_field, update_seed_inputs,
};
use crate::providers::comfyui;
use crate::state::{
    asset_display_name,
    delete_all_generative_version_files,
    delete_generative_version_files,
    input_value_as_i64,
    parse_version_index,
    GenerationJob,
    GenerationJobStatus,
    ProviderConnection,
    ProviderEntry,
    ProviderInputType,
    ProviderOutputType,
    SeedStrategy,
    TrackType,
};

const MAX_BATCH_COUNT: u32 = 50;

#[component]
pub fn AttributesPanelContent(
    project: Signal<crate::state::Project>,
    selection: Signal<crate::state::SelectionState>,
    preview_dirty: Signal<bool>,
    providers: Signal<Vec<ProviderEntry>>,
    on_enqueue_generation: EventHandler<GenerationJob>,
    on_audio_items_refresh: EventHandler<()>,
    previewer: Signal<std::sync::Arc<crate::core::preview::PreviewRenderer>>,
    thumbnailer: std::sync::Arc<crate::core::thumbnailer::Thumbnailer>,
    thumbnail_cache_buster: Signal<u64>,
) -> Element {
    let mut gen_status = use_signal(|| None::<String>);
    let mut last_clip_id = use_signal(|| None::<uuid::Uuid>);

    let selection_state = selection.read();
    let selected_clip_count = selection_state.clip_ids.len();
    let selected_track_count = selection_state.track_ids.len();
    let selected_clip_id = selection_state.primary_clip();
    let selected_track_id = selection_state.primary_track();
    drop(selection_state);

    use_effect(move || {
        if last_clip_id() != selected_clip_id {
            last_clip_id.set(selected_clip_id);
            gen_status.set(None);
        }
    });

    if selected_clip_count == 0 && selected_track_count == 0 {
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

    let total_selected = selected_clip_count + selected_track_count;
    if total_selected > 1 {
        return rsx! {
            div {
                style: "padding: 12px;",
                div {
                    style: "
                        display: flex; align-items: center; justify-content: center;
                        height: 80px; border: 1px dashed {BORDER_DEFAULT}; border-radius: 6px;
                        color: {TEXT_DIM}; font-size: 12px;
                    ",
                    "{total_selected} items selected"
                }
            }
        };
    }

    let Some(clip_id) = selected_clip_id else {
        if let Some(track_id) = selected_track_id {
            let project_read = project.read();
            let track = match project_read.tracks.iter().find(|track| track.id == track_id) {
                Some(track) => track.clone(),
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
            drop(project_read);

            let track_id = track.id;
            let track_label = match track.track_type {
                crate::state::TrackType::Audio => "Audio Track",
                crate::state::TrackType::Video => "Video Track",
                crate::state::TrackType::Marker => "Marker Track",
            };
            let track_type_label = match track.track_type {
                crate::state::TrackType::Audio => "Audio",
                crate::state::TrackType::Video => "Video",
                crate::state::TrackType::Marker => "Markers",
            };

            return rsx! {
                div {
                    style: "padding: 12px; display: flex; flex-direction: column; gap: 12px;",
                    div {
                        style: "display: flex; flex-direction: column; gap: 6px;",
                        span { style: "font-size: 11px; color: {TEXT_MUTED}; text-transform: uppercase; letter-spacing: 0.5px;", "{track_label}" }
                        div {
                            style: "display: flex; flex-direction: column; gap: 4px;",
                            span { style: "font-size: 10px; color: {TEXT_MUTED};", "Name" }
                            span { style: "font-size: 12px; color: {TEXT_PRIMARY};", "{track.name}" }
                        }
                        div {
                            style: "display: flex; flex-direction: column; gap: 4px;",
                            span { style: "font-size: 10px; color: {TEXT_MUTED};", "Type" }
                            span { style: "font-size: 12px; color: {TEXT_PRIMARY};", "{track_type_label}" }
                        }
                    }
                    if track.track_type != crate::state::TrackType::Marker {
                        div {
                            style: "
                                display: flex; flex-direction: column; gap: 10px;
                                padding: 10px; background-color: {BG_SURFACE};
                                border: 1px solid {BORDER_SUBTLE}; border-radius: 6px;
                            ",
                            div {
                                style: "font-size: 10px; color: {TEXT_DIM}; text-transform: uppercase; letter-spacing: 0.5px;",
                                "Audio"
                            }
                            NumericField {
                                key: "{track_id}-volume",
                                label: "Track Volume",
                                value: track.volume,
                                step: "0.05",
                                clamp_min: Some(0.0),
                                clamp_max: Some(2.0),
                                on_commit: move |value: f32| {
                                    if let Some(track) = project.write().tracks.iter_mut().find(|track| track.id == track_id) {
                                        track.volume = value.max(0.0);
                                    }
                                    on_audio_items_refresh.call(());
                                }
                            }
                        }
                    }
                }
            };
        }
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
    let config_snapshot = project_read
        .generative_config(clip.asset_id)
        .cloned()
        .unwrap_or_default();
    let asset_display = asset
        .as_ref()
        .map(asset_display_name)
        .unwrap_or_else(|| "Unknown".to_string());
    let asset_base_label = asset
        .as_ref()
        .map(|asset| asset.name.clone())
        .unwrap_or_else(|| "Unknown".to_string());
    let clip_has_audio = asset
        .as_ref()
        .map(|asset| asset.is_audio() || asset.is_video())
        .unwrap_or(false);
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
    let batch_settings = config_snapshot.batch.clone();
    let batch_count = batch_settings.count.max(1).min(MAX_BATCH_COUNT);
    let seed_strategy_value = batch_settings.seed_strategy.as_str();
    let seed_field_value = batch_settings.seed_field.clone().unwrap_or_default();
    let seed_field_options: Vec<(String, String)> = selected_provider
        .as_ref()
        .map(|provider| {
            provider
                .inputs
                .iter()
                .filter(|input| {
                    matches!(
                        input.input_type,
                        ProviderInputType::Integer | ProviderInputType::Number
                    )
                })
                .map(|input| {
                    let label = if input.label.trim().is_empty() || input.label == input.name {
                        input.name.clone()
                    } else {
                        format!("{} ({})", input.label, input.name)
                    };
                    (input.name.clone(), label)
                })
                .collect()
        })
        .unwrap_or_default();
    let seed_field_missing = batch_settings
        .seed_field
        .as_ref()
        .map(|field| !seed_field_options.iter().any(|(name, _)| name == field))
        .unwrap_or(false);
    let resolved_seed_field = selected_provider
        .as_ref()
        .and_then(|provider| resolve_seed_field(provider, batch_settings.seed_field.as_deref()));
    let seed_hint = if seed_field_missing {
        batch_settings
            .seed_field
            .as_ref()
            .map(|field| format!("Seed field '{}' not found in provider inputs.", field))
    } else if batch_settings.seed_field.is_none() && selected_provider.is_some() {
        Some(match resolved_seed_field.as_ref() {
            Some(field) => format!("Auto-detect: {}", field),
            None => "Auto-detect: none".to_string(),
        })
    } else {
        None
    };
    let batch_hint = if batch_count > 1 {
        match batch_settings.seed_strategy {
            SeedStrategy::Keep => Some(
                "Identical inputs can be cached by ComfyUI; use Increment or Random."
                    .to_string(),
            ),
            _ => {
                if resolved_seed_field.is_none() {
                    Some(
                        "No numeric seed field detected. Pick one to offset seeds."
                            .to_string(),
                    )
                } else {
                    None
                }
            }
        }
    } else {
        None
    };
    let selected_version_value = config_snapshot
        .active_version
        .clone()
        .unwrap_or_default();
    let mut version_options: Vec<String> = config_snapshot
        .versions
        .iter()
        .map(|record| record.version.clone())
        .collect();
    if !selected_version_value.trim().is_empty()
        && !version_options.contains(&selected_version_value)
    {
        version_options.push(selected_version_value.clone());
    }
    version_options.sort_by(|a, b| match (parse_version_index(a), parse_version_index(b)) {
        (Some(a_num), Some(b_num)) => b_num.cmp(&a_num),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => b.cmp(a),
    });
    version_options.dedup();
    let manage_versions_open = use_signal(|| false);
    let confirm_delete_current = use_signal(|| false);
    let confirm_delete_others = use_signal(|| false);
    let confirm_delete_all = use_signal(|| false);
    let can_delete_version = !selected_version_value.trim().is_empty();
    let on_provider_change = {
        let asset_id = clip.asset_id;
        let mut project = project.clone();
        Rc::new(RefCell::new(move |e: FormEvent| {
            let value = e.value();
            let provider_id = value
                .trim()
                .parse::<uuid::Uuid>()
                .ok();
            let mut project_write = project.write();
            project_write.set_generative_provider_id(asset_id, provider_id);
            let _ = project_write.save_generative_config(asset_id);
        }))
    };
    let on_version_change = {
        let asset_id = clip.asset_id;
        let mut project = project.clone();
        let mut preview_dirty = preview_dirty.clone();
        let thumbnailer = thumbnailer.clone();
        let thumbnail_cache_buster = thumbnail_cache_buster.clone();
        let mut manage_versions_open = manage_versions_open.clone();
        let mut confirm_delete_current = confirm_delete_current.clone();
        let mut confirm_delete_others = confirm_delete_others.clone();
        let mut confirm_delete_all = confirm_delete_all.clone();
        Rc::new(RefCell::new(move |e: FormEvent| {
            let value = e.value();
            let trimmed = value.trim();
            let next_version = if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            };
            {
                let mut project_write = project.write();
                project_write.update_generative_config(asset_id, |config| {
                    config.active_version = next_version.clone();
                    if let Some(version) = next_version.as_ref() {
                        if let Some(record) = config
                            .versions
                            .iter()
                            .find(|record| record.version == *version)
                        {
                            config.inputs = record.inputs_snapshot.clone();
                            config.provider_id = Some(record.provider_id);
                        }
                    }
                });
                let _ = project_write.save_generative_config(asset_id);
            }
            preview_dirty.set(true);
            if let Some(asset) = project.read().find_asset(asset_id).cloned() {
                let thumbs = thumbnailer.clone();
                let mut thumbnail_cache_buster = thumbnail_cache_buster.clone();
                spawn(async move {
                    thumbs.generate(&asset, true).await;
                    thumbnail_cache_buster.set(thumbnail_cache_buster() + 1);
                });
            }
            manage_versions_open.set(false);
            confirm_delete_current.set(false);
            confirm_delete_others.set(false);
            confirm_delete_all.set(false);
        }))
    };
    let on_delete_version = {
        let gen_folder_path = gen_folder_path.clone();
        let asset_id = clip.asset_id;
        let project = project.clone();
        let preview_dirty = preview_dirty.clone();
        let previewer = previewer.clone();
        let thumbnailer = thumbnailer.clone();
        let thumbnail_cache_buster = thumbnail_cache_buster.clone();
        let gen_status = gen_status.clone();
        let manage_versions_open = manage_versions_open.clone();
        let confirm_delete_current = confirm_delete_current.clone();
        let confirm_delete_others = confirm_delete_others.clone();
        let confirm_delete_all = confirm_delete_all.clone();
        let version_options = version_options.clone();
        let selected_version_value = selected_version_value.clone();
        Rc::new(RefCell::new(move || {
            let gen_folder_path = gen_folder_path.clone();
            let mut project = project.clone();
            let mut preview_dirty = preview_dirty.clone();
            let previewer = previewer.clone();
            let thumbnailer = thumbnailer.clone();
            let thumbnail_cache_buster = thumbnail_cache_buster.clone();
            let mut gen_status = gen_status.clone();
            let mut manage_versions_open = manage_versions_open.clone();
            let mut confirm_delete_current = confirm_delete_current.clone();
            let mut confirm_delete_others = confirm_delete_others.clone();
            let mut confirm_delete_all = confirm_delete_all.clone();
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
                    manage_versions_open.set(false);
                    confirm_delete_current.set(false);
                    confirm_delete_others.set(false);
                    confirm_delete_all.set(false);
                    return;
                }
                previewer.read().invalidate_folder(&folder_path);

                {
                    let mut project_write = project.write();
                    project_write.update_generative_config(asset_id, |config| {
                        config.versions.retain(|record| record.version != version);
                        config.active_version = next_active_clone.clone();
                        if let Some(next_active) = next_active_clone.as_ref() {
                            if let Some(record) = config
                                .versions
                                .iter()
                                .find(|record| record.version == *next_active)
                            {
                                config.inputs = record.inputs_snapshot.clone();
                                config.provider_id = Some(record.provider_id);
                            }
                        }
                    });
                    let _ = project_write.save_generative_config(asset_id);
                }

                preview_dirty.set(true);

                if let Some(asset) = project.read().find_asset(asset_id).cloned() {
                    let thumbs = thumbnailer.clone();
                    let mut thumbnail_cache_buster = thumbnail_cache_buster.clone();
                    spawn(async move {
                        thumbs.generate(&asset, true).await;
                        thumbnail_cache_buster.set(thumbnail_cache_buster() + 1);
                    });
                }

                manage_versions_open.set(false);
                confirm_delete_current.set(false);
                confirm_delete_others.set(false);
                confirm_delete_all.set(false);
                gen_status.set(Some(format!("Deleted {}", version)));
            });
        }))
    };
    let on_delete_all_versions = {
        let gen_folder_path = gen_folder_path.clone();
        let asset_id = clip.asset_id;
        let project = project.clone();
        let preview_dirty = preview_dirty.clone();
        let previewer = previewer.clone();
        let thumbnailer = thumbnailer.clone();
        let thumbnail_cache_buster = thumbnail_cache_buster.clone();
        let gen_status = gen_status.clone();
        let manage_versions_open = manage_versions_open.clone();
        let confirm_delete_current = confirm_delete_current.clone();
        let confirm_delete_others = confirm_delete_others.clone();
        let confirm_delete_all = confirm_delete_all.clone();
        let versions_to_delete: Vec<String> = config_snapshot
            .versions
            .iter()
            .map(|record| record.version.clone())
            .collect();
        Rc::new(RefCell::new(move || {
            let gen_folder_path = gen_folder_path.clone();
            let mut project = project.clone();
            let mut preview_dirty = preview_dirty.clone();
            let previewer = previewer.clone();
            let thumbnailer = thumbnailer.clone();
            let thumbnail_cache_buster = thumbnail_cache_buster.clone();
            let mut gen_status = gen_status.clone();
            let mut manage_versions_open = manage_versions_open.clone();
            let mut confirm_delete_current = confirm_delete_current.clone();
            let mut confirm_delete_others = confirm_delete_others.clone();
            let mut confirm_delete_all = confirm_delete_all.clone();
            let versions_to_delete = versions_to_delete.clone();

            if versions_to_delete.is_empty() {
                gen_status.set(Some("No versions to delete.".to_string()));
                return;
            }
            let Some(folder_path) = gen_folder_path.as_ref() else {
                gen_status.set(Some("Missing generative folder.".to_string()));
                return;
            };

            let folder_path = folder_path.clone();
            let delete_folder = folder_path.clone();
            spawn(async move {
                let deletion = tokio::task::spawn_blocking(move || {
                    delete_all_generative_version_files(&delete_folder, &versions_to_delete)
                })
                .await
                .ok()
                .unwrap_or_else(|| Err("Failed to delete version files.".to_string()));

                if let Err(err) = deletion {
                    gen_status.set(Some(format!("Delete failed: {}", err)));
                    manage_versions_open.set(false);
                    confirm_delete_current.set(false);
                    confirm_delete_others.set(false);
                    confirm_delete_all.set(false);
                    return;
                }
                previewer.read().invalidate_folder(&folder_path);

                {
                    let mut project_write = project.write();
                    project_write.update_generative_config(asset_id, |config| {
                        config.versions.clear();
                        config.active_version = None;
                    });
                    let _ = project_write.save_generative_config(asset_id);
                }

                preview_dirty.set(true);

                if let Some(asset) = project.read().find_asset(asset_id).cloned() {
                    let thumbs = thumbnailer.clone();
                    let mut thumbnail_cache_buster = thumbnail_cache_buster.clone();
                    spawn(async move {
                        thumbs.generate(&asset, true).await;
                        thumbnail_cache_buster.set(thumbnail_cache_buster() + 1);
                    });
                }

                manage_versions_open.set(false);
                confirm_delete_current.set(false);
                confirm_delete_others.set(false);
                confirm_delete_all.set(false);
                gen_status.set(Some("Deleted all versions".to_string()));
            });
        }))
    };
    let on_delete_other_versions = {
        let gen_folder_path = gen_folder_path.clone();
        let asset_id = clip.asset_id;
        let project = project.clone();
        let preview_dirty = preview_dirty.clone();
        let previewer = previewer.clone();
        let thumbnailer = thumbnailer.clone();
        let thumbnail_cache_buster = thumbnail_cache_buster.clone();
        let gen_status = gen_status.clone();
        let manage_versions_open = manage_versions_open.clone();
        let confirm_delete_current = confirm_delete_current.clone();
        let confirm_delete_others = confirm_delete_others.clone();
        let confirm_delete_all = confirm_delete_all.clone();
        let version_options = version_options.clone();
        let selected_version_value = selected_version_value.clone();
        Rc::new(RefCell::new(move || {
            let gen_folder_path = gen_folder_path.clone();
            let mut project = project.clone();
            let mut preview_dirty = preview_dirty.clone();
            let previewer = previewer.clone();
            let thumbnailer = thumbnailer.clone();
            let thumbnail_cache_buster = thumbnail_cache_buster.clone();
            let mut gen_status = gen_status.clone();
            let mut manage_versions_open = manage_versions_open.clone();
            let mut confirm_delete_current = confirm_delete_current.clone();
            let mut confirm_delete_others = confirm_delete_others.clone();
            let mut confirm_delete_all = confirm_delete_all.clone();
            let version_options = version_options.clone();
            let selected_version_value = selected_version_value.clone();

            let current_version = selected_version_value.trim().to_string();
            if current_version.is_empty() {
                gen_status.set(Some("No active version selected.".to_string()));
                return;
            }
            let versions_to_delete: Vec<String> = version_options
                .iter()
                .filter(|version| *version != &current_version)
                .cloned()
                .collect();
            if versions_to_delete.is_empty() {
                gen_status.set(Some("No other versions to delete.".to_string()));
                return;
            }
            let Some(folder_path) = gen_folder_path.as_ref() else {
                gen_status.set(Some("Missing generative folder.".to_string()));
                return;
            };

            let folder_path = folder_path.clone();
            let delete_folder = folder_path.clone();
            spawn(async move {
                let deletion = tokio::task::spawn_blocking(move || {
                    delete_all_generative_version_files(&delete_folder, &versions_to_delete)
                })
                .await
                .ok()
                .unwrap_or_else(|| Err("Failed to delete version files.".to_string()));

                if let Err(err) = deletion {
                    gen_status.set(Some(format!("Delete failed: {}", err)));
                    manage_versions_open.set(false);
                    confirm_delete_current.set(false);
                    confirm_delete_others.set(false);
                    confirm_delete_all.set(false);
                    return;
                }
                previewer.read().invalidate_folder(&folder_path);

                {
                    let mut project_write = project.write();
                    project_write.update_generative_config(asset_id, |config| {
                        config
                            .versions
                            .retain(|record| record.version == current_version);
                        config.active_version = Some(current_version.clone());
                    });
                    let _ = project_write.save_generative_config(asset_id);
                }

                preview_dirty.set(true);

                if let Some(asset) = project.read().find_asset(asset_id).cloned() {
                    let thumbs = thumbnailer.clone();
                    let mut thumbnail_cache_buster = thumbnail_cache_buster.clone();
                    spawn(async move {
                        thumbs.generate(&asset, true).await;
                        thumbnail_cache_buster.set(thumbnail_cache_buster() + 1);
                    });
                }

                manage_versions_open.set(false);
                confirm_delete_current.set(false);
                confirm_delete_others.set(false);
                confirm_delete_all.set(false);
                gen_status.set(Some("Deleted other versions".to_string()));
            });
        }))
    };

    let set_input_value = {
        let asset_id = clip.asset_id;
        let mut project = project.clone();
        Rc::new(RefCell::new(move |name: String, value: serde_json::Value| {
            let mut project_write = project.write();
            project_write.update_generative_config(asset_id, |config| {
                config.inputs.insert(
                    name,
                    crate::state::InputValue::Literal { value },
                );
            });
            let _ = project_write.save_generative_config(asset_id);
        }))
    };

    let on_batch_count_change = {
        let asset_id = clip.asset_id;
        let mut project = project.clone();
        Rc::new(RefCell::new(move |next: i64| {
            let clamped = next.clamp(1, MAX_BATCH_COUNT as i64) as u32;
            let mut project_write = project.write();
            project_write.update_generative_config(asset_id, |config| {
                config.batch.count = clamped;
            });
            let _ = project_write.save_generative_config(asset_id);
        }))
    };

    let on_seed_strategy_change = {
        let asset_id = clip.asset_id;
        let mut project = project.clone();
        Rc::new(RefCell::new(move |e: FormEvent| {
            let next = SeedStrategy::from_str(&e.value());
            let mut project_write = project.write();
            project_write.update_generative_config(asset_id, |config| {
                config.batch.seed_strategy = next;
            });
            let _ = project_write.save_generative_config(asset_id);
        }))
    };

    let on_seed_field_change = {
        let asset_id = clip.asset_id;
        let mut project = project.clone();
        Rc::new(RefCell::new(move |e: FormEvent| {
            let value = e.value();
            let trimmed = value.trim();
            let next = if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            };
            let mut project_write = project.write();
            project_write.update_generative_config(asset_id, |config| {
                config.batch.seed_field = next;
            });
            let _ = project_write.save_generative_config(asset_id);
        }))
    };

    let asset_label = asset_base_label.clone();
    let on_generate = {
        let gen_folder_path = gen_folder_path.clone();
        let gen_status = gen_status.clone();
        let selected_provider = selected_provider.clone();
        let asset_id = clip.asset_id;
        let clip_id = clip.id;
        let asset_label = asset_label.clone();
        let on_enqueue_generation = on_enqueue_generation.clone();
        let project = project.clone();
        Rc::new(RefCell::new(move |_evt: MouseEvent| {
            let gen_folder_path = gen_folder_path.clone();
            let mut gen_status = gen_status.clone();
            let selected_provider = selected_provider.clone();
            let on_enqueue_generation = on_enqueue_generation.clone();
            let mut project = project.clone();

            let Some(provider) = selected_provider.clone() else {
                gen_status.set(Some("Select a provider first.".to_string()));
                return;
            };
            let Some(folder_path) = gen_folder_path.clone() else {
                gen_status.set(Some("Missing generative folder.".to_string()));
                return;
            };

            let mut project_write = project.write();
            project_write.update_generative_config(asset_id, |config| {
                config.provider_id = Some(provider.id);
            });
            let config_snapshot = project_write
                .generative_config(asset_id)
                .cloned()
                .unwrap_or_default();
            let _ = project_write.save_generative_config(asset_id);

            let resolved = resolve_provider_inputs(&provider, &config_snapshot);
            if !resolved.missing_required.is_empty() {
                gen_status.set(Some(format!(
                    "Missing inputs: {}",
                    resolved.missing_required.join(", ")
                )));
                return;
            }

            let batch_settings = config_snapshot.batch.clone();
            let batch_count = batch_settings.count.max(1).min(MAX_BATCH_COUNT);
            let seed_field =
                resolve_seed_field(&provider, batch_settings.seed_field.as_deref());
            let mut seed_base = seed_field
                .as_ref()
                .and_then(|field| resolved.values.get(field))
                .and_then(input_value_as_i64);
            let mut seed_base_randomized = false;
            if seed_base.is_none()
                && seed_field.is_some()
                && batch_settings.seed_strategy == SeedStrategy::Increment
            {
                seed_base = Some(random_seed_i64());
                seed_base_randomized = true;
            }
            let seed_strategy = batch_settings.seed_strategy;
            let base_inputs = resolved.values.clone();
            let base_snapshot = resolved.snapshot.clone();
            let job_asset_label = asset_label.clone();

            gen_status.set(Some("Checking provider...".to_string()));

            spawn(async move {
                let health = match provider.connection.clone() {
                    ProviderConnection::ComfyUi { base_url, .. } => {
                        comfyui::check_health(&base_url).await
                    }
                    _ => Err("Provider health check not supported for this adapter yet.".to_string()),
                };

                if let Err(err) = health {
                    gen_status.set(Some(format!("Provider offline: {}", err)));
                    return;
                }

                let mut queued = 0u32;
                for index in 0..batch_count {
                    let (inputs, input_snapshot) = match (seed_strategy, seed_field.as_ref()) {
                        (SeedStrategy::Keep, _) | (_, None) => {
                            (base_inputs.clone(), base_snapshot.clone())
                        }
                        (SeedStrategy::Increment, Some(field)) => {
                            let seed = seed_base.unwrap_or(0) + index as i64;
                            update_seed_inputs(&base_inputs, &base_snapshot, field, seed)
                        }
                        (SeedStrategy::Random, Some(field)) => {
                            let seed = random_seed_i64();
                            update_seed_inputs(&base_inputs, &base_snapshot, field, seed)
                        }
                    };
                    let job = GenerationJob {
                        id: uuid::Uuid::new_v4(),
                        created_at: chrono::Utc::now(),
                        status: GenerationJobStatus::Queued,
                        progress_overall: None,
                        progress_node: None,
                        attempts: 0,
                        next_attempt_at: None,
                        provider: provider.clone(),
                        output_type: provider.output_type,
                        asset_id,
                        clip_id,
                        asset_label: job_asset_label.clone(),
                        folder_path: folder_path.clone(),
                        inputs,
                        inputs_snapshot: input_snapshot,
                        version: None,
                        error: None,
                    };

                    on_enqueue_generation.call(job);
                    queued += 1;
                }

                let mut status = if queued > 1 {
                    format!("Queued {} jobs", queued)
                } else {
                    "Queued".to_string()
                };
                if queued > 1 {
                    if seed_strategy == SeedStrategy::Keep {
                        status = format!("{} (identical inputs may be cached)", status);
                    } else if seed_field.is_none() {
                        status = format!("{} (no seed field detected)", status);
                    } else if seed_base_randomized {
                        status = format!("{} (seed missing, randomized base)", status);
                    }
                }
                gen_status.set(Some(status));
            });
        }))
    };

    let transform = clip.transform;
    let clip_id = clip.id;
    let clip_label = clip.label.clone().unwrap_or_default();
    let clip_track_type = project.read().find_track(clip.track_id).map(|track| track.track_type);
    let allow_clip_gain = clip_track_type == Some(TrackType::Audio)
        || clip_track_type == Some(TrackType::Video);
    let generate_label = if batch_count > 1 {
        format!("Generate x{}", batch_count)
    } else {
        "Generate".to_string()
    };
    let generate_opacity = "1.0";

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

            if clip_has_audio && allow_clip_gain {
                div {
                    style: "
                        display: flex; flex-direction: column; gap: 10px;
                        padding: 10px; background-color: {BG_SURFACE};
                        border: 1px solid {BORDER_SUBTLE}; border-radius: 6px;
                    ",
                    div {
                        style: "font-size: 10px; color: {TEXT_DIM}; text-transform: uppercase; letter-spacing: 0.5px;",
                        "Audio"
                    }
                    NumericField {
                        key: "{clip_id}-volume",
                        label: "Clip Volume",
                        value: clip.volume,
                        step: "0.05",
                        clamp_min: Some(0.0),
                        clamp_max: Some(2.0),
                        on_commit: move |value: f32| {
                            if let Some(clip) = project.write().clips.iter_mut().find(|clip| clip.id == clip_id) {
                                clip.volume = value.max(0.0);
                            }
                            on_audio_items_refresh.call(());
                        }
                    }
                }
            }

            if gen_output.is_some() {
                {render_generative_controls(
                    &version_options,
                    &selected_version_value,
                    manage_versions_open,
                    confirm_delete_current,
                    confirm_delete_others,
                    can_delete_version,
                    on_version_change,
                    on_delete_version.clone(),
                    on_delete_other_versions.clone(),
                    on_delete_all_versions.clone(),
                    &selected_provider_value,
                    &compatible_providers,
                    on_provider_change,
                    show_missing_provider,
                    &providers_path_label,
                    on_generate,
                    gen_status,
                    generate_label.as_str(),
                    generate_opacity,
                    batch_count,
                    on_batch_count_change,
                    seed_strategy_value,
                    on_seed_strategy_change,
                    seed_field_value.as_str(),
                    &seed_field_options,
                    on_seed_field_change,
                    seed_hint.clone(),
                    seed_field_missing,
                    batch_hint.clone(),
                    confirm_delete_all,
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

