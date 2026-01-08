use dioxus::prelude::*;
use crate::constants::*;
use crate::state::asset_display_name;

#[component]
pub fn AssetItem(
    asset: crate::state::Asset,
    thumbnailer: std::sync::Arc<crate::core::thumbnailer::Thumbnailer>,
    thumbnail_cache_buster: u64,
    panel_width: f64,
    on_rename: EventHandler<(uuid::Uuid, String)>,
    on_delete: EventHandler<uuid::Uuid>,
    on_regenerate_thumbnails: EventHandler<uuid::Uuid>,
    on_add_to_timeline: EventHandler<uuid::Uuid>,
    on_drag_start: EventHandler<uuid::Uuid>,
) -> Element {
    let mut show_menu = use_signal(|| false);
    let mut menu_pos = use_signal(|| (0.0, 0.0));
    let is_editing = use_signal(|| false);
    let asset_name = asset.name.clone();
    let asset_name_for_effect = asset_name.clone();
    let mut draft_name = use_signal(|| asset_name.clone());
    let mut draft_name_for_effect = draft_name.clone();
    let is_editing_for_effect = is_editing.clone();

    use_effect(move || {
        if !is_editing_for_effect() {
            draft_name_for_effect.set(asset_name_for_effect.clone());
        }
    });

    // Icon based on asset type
    let icon = match &asset.kind {
        crate::state::AssetKind::Video { .. } => "🎬",
        crate::state::AssetKind::Image { .. } => "🖼️",
        crate::state::AssetKind::Audio { .. } => "🔊",
        crate::state::AssetKind::GenerativeVideo { .. } => "✨🎬",
        crate::state::AssetKind::GenerativeImage { .. } => "✨🖼️",
        crate::state::AssetKind::GenerativeAudio { .. } => "✨🔊",
    };
    
    // Color accent based on type
    let accent = match &asset.kind {
        crate::state::AssetKind::Video { .. } | crate::state::AssetKind::GenerativeVideo { .. } => ACCENT_VIDEO,
        crate::state::AssetKind::Audio { .. } | crate::state::AssetKind::GenerativeAudio { .. } => ACCENT_AUDIO,
        crate::state::AssetKind::Image { .. } | crate::state::AssetKind::GenerativeImage { .. } => ACCENT_VIDEO,
    };
    
    let thumb_url = if asset.is_visual() {
        thumbnailer.get_thumbnail_path(asset.id, 0.0).map(|p| {
            let url = crate::utils::get_local_file_url(&p);
            format!("{}?v={}", url, thumbnail_cache_buster)
        })
    } else {
        None
    };
    
    // Generative assets have a subtle dashed border
    let border_style = if asset.is_generative() {
        format!("1px dashed {}", BORDER_DEFAULT)  // Subtle dashed, not accent-colored
    } else {
        format!("1px solid {}", BORDER_SUBTLE)
    };

    let asset_id = asset.id;
    let display_name = asset_display_name(&asset);
    let menu_max_x = (panel_width - 140.0).max(0.0);
    
    rsx! {
        div {
            style: "position: relative;",
            
            div {
                style: "
                    display: flex; align-items: center; gap: 8px;
                    padding: 8px; margin-bottom: 4px;
                    background-color: {BG_SURFACE}; border: {border_style}; border-radius: 4px;
                    cursor: grab; transition: background-color 0.1s ease;
                    user-select: none;
                ",
                oncontextmenu: move |e| {
                    e.prevent_default();
                    let coords = e.client_coordinates();
                    menu_pos.set((coords.x, coords.y));
                    show_menu.set(true);
                },
                onmousedown: move |e| {
                    // Left click starts drag
                    e.prevent_default(); // prevent browser default drag (we use our own)
                    on_drag_start.call(asset_id);
                },
                // Type indicator
                div {
                    style: "width: 3px; height: 24px; border-radius: 2px; background-color: {accent};",
                }
                // Thumbnail + icon
                div {
                    style: "
                        width: 36px; height: 24px; border-radius: 3px; overflow: hidden;
                        background-color: {BG_BASE}; border: 1px solid {BORDER_SUBTLE};
                        display: flex; align-items: center; justify-content: center;
                        position: relative; flex-shrink: 0;
                    ",
                    if let Some(src_url) = thumb_url.clone() {
                        img {
                            src: "{src_url}",
                            style: "width: 100%; height: 100%; object-fit: cover; pointer-events: none;",
                            draggable: "false",
                        }
                        span {
                            style: "
                                position: absolute; right: 2px; bottom: 2px;
                                font-size: 9px; color: {TEXT_PRIMARY};
                                background-color: rgba(0,0,0,0.6); padding: 1px 3px;
                                border-radius: 3px; pointer-events: none;
                            ",
                            "{icon}"
                        }
                    } else {
                        span { style: "font-size: 12px; color: {TEXT_MUTED}; pointer-events: none;", "{icon}" }
                    }
                }
                // Name
                if is_editing() {
                    input {
                        r#type: "text",
                        value: "{draft_name()}",
                        autofocus: "true",
                        style: "
                            flex: 1; min-width: 0;
                            font-size: 12px; color: {TEXT_PRIMARY};
                            background-color: {BG_BASE};
                            border: 1px solid {BORDER_DEFAULT};
                            border-radius: 4px;
                            padding: 4px 6px;
                        ",
                        oninput: move |e| draft_name.set(e.value()),
                        onblur: {
                            let asset_name = asset_name.clone();
                            let on_rename = on_rename.clone();
                            let asset_id = asset_id;
                            let mut is_editing = is_editing.clone();
                            let mut draft_name = draft_name.clone();
                            move |_| {
                                let next = draft_name().trim().to_string();
                                is_editing.set(false);
                                if !next.is_empty() && next != asset_name {
                                    on_rename.call((asset_id, next));
                                } else {
                                    draft_name.set(asset_name.clone());
                                }
                            }
                        },
                        onkeydown: {
                            let asset_name = asset_name.clone();
                            let on_rename = on_rename.clone();
                            let asset_id = asset_id;
                            let mut is_editing = is_editing.clone();
                            let mut draft_name = draft_name.clone();
                            move |e: KeyboardEvent| {
                                if e.key() == Key::Enter {
                                    let next = draft_name().trim().to_string();
                                    is_editing.set(false);
                                    if !next.is_empty() && next != asset_name {
                                        on_rename.call((asset_id, next));
                                    } else {
                                        draft_name.set(asset_name.clone());
                                    }
                                } else if e.key() == Key::Escape {
                                    is_editing.set(false);
                                    draft_name.set(asset_name.clone());
                                }
                            }
                        },
                        onmousedown: move |e| e.stop_propagation(),
                        oncontextmenu: move |e| e.stop_propagation(),
                    }
                } else {
                    span { 
                        style: "flex: 1; min-width: 0; font-size: 12px; color: {TEXT_PRIMARY}; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                        onmousedown: move |e| e.stop_propagation(),
                        ondoubleclick: {
                            let asset_name = asset_name.clone();
                            let mut draft_name = draft_name.clone();
                            let mut is_editing = is_editing.clone();
                            move |e| {
                                e.stop_propagation();
                                is_editing.set(true);
                                draft_name.set(asset_name.clone());
                            }
                        },
                        "{display_name}"
                    }
                }
            }
            
            // Context menu for this asset
            if show_menu() {
                // Backdrop
                div {
                    style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; z-index: 999;",
                    onclick: move |_| show_menu.set(false),
                }
                // Menu
                {
                    let (x, y) = menu_pos();
                    rsx! {
                        div {
                            style: "
                                position: fixed; 
                                left: min({x}px, {menu_max_x}px); 
                                top: min({y}px, calc(100vh - 50px));
                                background-color: {BG_ELEVATED}; border: 1px solid {BORDER_DEFAULT};
                                border-radius: 6px; padding: 4px 0; min-width: 120px;
                                box-shadow: 0 4px 12px rgba(0,0,0,0.3);
                                z-index: 1000; font-size: 12px;
                            ",
                            // Add to timeline option
                            div {
                                style: "
                                    padding: 6px 12px; color: {TEXT_PRIMARY}; cursor: pointer;
                                    transition: background-color 0.1s ease;
                                ",
                                onclick: move |_| {
                                    on_add_to_timeline.call(asset_id);
                                    show_menu.set(false);
                                },
                                "➕ Add to Timeline"
                            }
                             // Regenerate Thumbnails
                            div {
                                style: "
                                    padding: 6px 12px; color: {TEXT_PRIMARY}; cursor: pointer;
                                    transition: background-color 0.1s ease;
                                ",
                                onclick: move |_| {
                                    on_regenerate_thumbnails.call(asset_id);
                                    show_menu.set(false);
                                },
                                "🔄 Refresh Thumbnails"
                            }
                            // Divider
                            div {
                                style: "height: 1px; background-color: {BORDER_SUBTLE}; margin: 4px 0;",
                            }
                            // Delete option
                            div {
                                style: "
                                    padding: 6px 12px; color: #ef4444; cursor: pointer;
                                    transition: background-color 0.1s ease;
                                ",
                                onclick: move |_| {
                                    on_delete.call(asset_id);
                                    show_menu.set(false);
                                },
                                "🗑 Delete"
                            }
                        }
                    }
                }
            }
        }
    }
}





