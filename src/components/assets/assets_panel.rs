use dioxus::prelude::*;
use crate::components::assets::AssetItem;
use crate::constants::*;
use crate::state::next_generative_index;

#[component]
pub fn AssetsPanelContent(
    assets: Vec<crate::state::Asset>,
    thumbnailer: std::sync::Arc<crate::core::thumbnailer::Thumbnailer>,
    thumbnail_cache_buster: u64,
    thumbnail_refresh_tick: u64,
    panel_width: f64,
    on_import: EventHandler<crate::state::Asset>,
    on_import_file: EventHandler<std::path::PathBuf>,
    on_rename: EventHandler<(uuid::Uuid, String)>,
    on_delete: EventHandler<uuid::Uuid>,
    on_regenerate_thumbnails: EventHandler<uuid::Uuid>,
    on_add_to_timeline: EventHandler<uuid::Uuid>,
    on_drag_start: EventHandler<uuid::Uuid>,
) -> Element {
    let _ = thumbnail_refresh_tick;
    let next_video_index = next_generative_index(
        &assets,
        "Gen Video",
        |kind| matches!(kind, crate::state::AssetKind::GenerativeVideo { .. }),
    );
    let next_image_index = next_generative_index(
        &assets,
        "Gen Image",
        |kind| matches!(kind, crate::state::AssetKind::GenerativeImage { .. }),
    );
    let next_audio_index = next_generative_index(
        &assets,
        "Gen Audio",
        |kind| matches!(kind, crate::state::AssetKind::GenerativeAudio { .. }),
    );
    rsx! {
        div {
            style: "display: flex; flex-direction: column; height: 100%; padding: 8px;",
            
            // Import button
            button {
                style: "
                    width: 100%; padding: 8px 12px; margin-bottom: 8px;
                    background-color: {BG_SURFACE}; border: 1px dashed {BORDER_DEFAULT};
                    border-radius: 6px; color: {TEXT_SECONDARY}; font-size: 12px;
                    cursor: pointer; transition: all 0.15s ease;
                ",
                onclick: move |_| {
                    // Use rfd for native file dialog
                    if let Some(paths) = rfd::FileDialog::new()
                        .add_filter("Media Files", &["mp4", "mov", "avi", "mp3", "wav", "png", "jpg", "jpeg", "gif", "webp"])
                        .add_filter("Video", &["mp4", "mov", "avi", "mkv", "webm"])
                        .add_filter("Audio", &["mp3", "wav", "ogg", "flac"])
                        .add_filter("Images", &["png", "jpg", "jpeg", "gif", "webp"])
                        .set_title("Import Assets")
                        .pick_files()
                    {
                        for path in paths {
                            on_import_file.call(path);
                        }
                    }
                },
                "📁 Import Files..."
            }
            
            // Generative asset buttons
            div {
                style: "
                    display: flex; flex-direction: column; gap: 4px; margin-bottom: 12px;
                    padding: 8px; background-color: {BG_SURFACE}; border-radius: 6px;
                    border: 1px solid {BORDER_SUBTLE};
                ",
                div { 
                    style: "font-size: 10px; color: {TEXT_DIM}; text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 4px;",
                    "✨ New Generative"
                }
                div {
                    style: "display: flex; gap: 4px;",
                    
                    // Generative Video button
                    button {
                        style: "
                            flex: 1; padding: 6px 8px;
                            background: transparent; border: 1px dashed {ACCENT_VIDEO};
                            border-radius: 4px; color: {ACCENT_VIDEO}; font-size: 11px;
                            cursor: pointer; transition: all 0.15s ease;
                        ",
                        onclick: {
                            let on_import = on_import.clone();
                            move |_| {
                                let id = uuid::Uuid::new_v4();
                                let folder = std::path::PathBuf::from(format!("generated/video/{}", id));
                                let asset = crate::state::Asset::new_generative_video(
                                    format!("Gen Video {}", next_video_index),
                                    folder
                                );
                                on_import.call(asset);
                            }
                        },
                        "🎬 Video"
                    }
                    
                    // Generative Image button
                    button {
                        style: "
                            flex: 1; padding: 6px 8px;
                            background: transparent; border: 1px dashed {ACCENT_VIDEO};
                            border-radius: 4px; color: {ACCENT_VIDEO}; font-size: 11px;
                            cursor: pointer; transition: all 0.15s ease;
                        ",
                        onclick: {
                            let on_import = on_import.clone();
                            move |_| {
                                let id = uuid::Uuid::new_v4();
                                let folder = std::path::PathBuf::from(format!("generated/image/{}", id));
                                let asset = crate::state::Asset::new_generative_image(
                                    format!("Gen Image {}", next_image_index),
                                    folder
                                );
                                on_import.call(asset);
                            }
                        },
                        "🖼️ Image"
                    }
                    
                    // Generative Audio button
                    button {
                        style: "
                            flex: 1; padding: 6px 8px;
                            background: transparent; border: 1px dashed {ACCENT_AUDIO};
                            border-radius: 4px; color: {ACCENT_AUDIO}; font-size: 11px;
                            cursor: pointer; transition: all 0.15s ease;
                        ",
                        onclick: {
                            let on_import = on_import.clone();
                            move |_| {
                                let id = uuid::Uuid::new_v4();
                                let folder = std::path::PathBuf::from(format!("generated/audio/{}", id));
                                let asset = crate::state::Asset::new_generative_audio(
                                    format!("Gen Audio {}", next_audio_index),
                                    folder
                                );
                                on_import.call(asset);
                            }
                        },
                        "🔊 Audio"
                    }
                }
            }
            // Asset list
            div {
                style: "flex: 1; overflow-y: auto;",
                
                if assets.is_empty() {
                    div {
                        style: "
                            display: flex; flex-direction: column; align-items: center; justify-content: center;
                            height: 120px; border: 1px dashed {BORDER_DEFAULT}; border-radius: 6px;
                            color: {TEXT_DIM}; font-size: 12px; text-align: center; padding: 12px;
                        ",
                        div { style: "font-size: 24px; margin-bottom: 8px;", "📂" }
                        "No assets yet"
                        div { style: "font-size: 10px; color: {TEXT_DIM}; margin-top: 4px;", "Import files or create generative assets" }
                    }
                } else {
                    for asset in assets.iter() {
                        AssetItem { 
                            asset: asset.clone(),
                            thumbnailer: thumbnailer.clone(),
                            thumbnail_cache_buster: thumbnail_cache_buster,
                            panel_width: panel_width,
                            on_rename: move |payload| on_rename.call(payload),
                            on_delete: move |id| on_delete.call(id),
                            on_regenerate_thumbnails: move |id| on_regenerate_thumbnails.call(id),
                            on_add_to_timeline: move |id| on_add_to_timeline.call(id),
                            on_drag_start: move |id| on_drag_start.call(id),
                        }
                    }
                }
            }
        }
    }
}


