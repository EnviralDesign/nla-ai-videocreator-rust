use dioxus::prelude::*;
use std::path::PathBuf;
use crate::constants::*;
use crate::state::ProjectSettings;

#[derive(Clone, Copy, PartialEq)]
pub enum StartupModalMode {
    Create,
    Edit,
}

#[component]
pub fn StartupModal(
    mode: StartupModalMode,
    initial_name: Option<String>,
    initial_settings: Option<ProjectSettings>,
    initial_folder: Option<PathBuf>,
    on_create: EventHandler<(PathBuf, String, ProjectSettings)>,
    on_open: EventHandler<PathBuf>,
    on_update: EventHandler<ProjectSettings>,
    on_close: EventHandler<MouseEvent>,
) -> Element {
    let is_edit = mode == StartupModalMode::Edit;
    let seed_name = initial_name.unwrap_or_else(|| "My New Project".to_string());
    let seed_settings = initial_settings.unwrap_or_default();
    let width_default = seed_settings.width;
    let height_default = seed_settings.height;
    let fps_default = seed_settings.fps;
    let duration_default = seed_settings.duration_seconds;
    let preview_default_width = seed_settings.preview_max_width;
    let preview_default_height = seed_settings.preview_max_height;
    let mut name = use_signal(|| seed_name.clone());
    let mut width = use_signal(|| seed_settings.width.to_string());
    let mut height = use_signal(|| seed_settings.height.to_string());
    let mut fps = use_signal(|| seed_settings.fps.to_string());
    let mut duration = use_signal(|| seed_settings.duration_seconds.to_string());
    let mut preview_max_width = use_signal(|| seed_settings.preview_max_width.to_string());
    let mut preview_max_height = use_signal(|| seed_settings.preview_max_height.to_string());
    let header_title = if is_edit {
        "Project Settings"
    } else {
        "NLA AI Video Creator"
    };
    let header_subtitle = if is_edit {
        "Update resolution, timing, and preview performance."
    } else {
        "Create a new project or open an existing one"
    };
    let section_title = if is_edit {
        "Edit Project"
    } else {
        "Create New Project"
    };
    let name_input_bg = if is_edit { BG_SURFACE } else { BG_BASE };
    let left_panel_border = if is_edit {
        "border-right: none;"
    } else {
        "border-right: 1px solid {BORDER_DEFAULT};"
    };
    
    // Default projects folder
    let projects_folder = std::env::current_dir().unwrap_or_default().join("projects");
    let projects_folder_clone = projects_folder.clone();
    let projects_folder_for_browse = projects_folder.clone();
    let projects_folder_for_open = projects_folder.clone();
    let projects_folder_for_scan = projects_folder.clone();
    
    // Use `Option<PathBuf>` to store the selected parent directory
    let initial_parent = initial_folder.unwrap_or_else(|| projects_folder_clone.clone());
    let mut parent_dir = use_signal(move || initial_parent.clone());
    
    // Refresh counter - increment to force re-scan of projects
    let mut refresh_counter = use_signal(|| 0u32);
    
    // Context menu state: Option<(x, y, project_path, project_name)>
    let mut context_menu: Signal<Option<(f64, f64, std::path::PathBuf, String)>> = use_signal(|| None);

    fn parse_u32(value: &str, default: u32, min: u32) -> u32 {
        value
            .trim()
            .parse::<u32>()
            .ok()
            .filter(|v| *v >= min)
            .unwrap_or(default)
    }

    fn parse_f64(value: &str, default: f64, min: f64) -> f64 {
        value
            .trim()
            .parse::<f64>()
            .ok()
            .filter(|v| *v >= min)
            .unwrap_or(default)
    }
    
    // Scan for existing projects (folders containing project.json)
    // Re-runs when refresh_counter changes
    let _ = refresh_counter(); // Subscribe to changes
    let existing_projects: Vec<(String, std::path::PathBuf)> = if projects_folder_for_scan.exists() {
        std::fs::read_dir(&projects_folder_for_scan)
            .map(|entries| {
                entries
                    .filter_map(|entry| entry.ok())
                    .filter(|entry| entry.path().is_dir())
                    .filter(|entry| entry.path().join("project.json").exists())
                    .map(|entry| {
                        let path = entry.path();
                        let name = path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("Unknown")
                            .to_string();
                        (name, path)
                    })
                    .collect()
            })
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    rsx! {
        div {
            style: "
                position: fixed; top: 0; left: 0; right: 0; bottom: 0;
                background-color: {BG_BASE}; z-index: 9999;
                display: flex; align-items: center; justify-content: center;
            ",
            
            div {
                style: "
                    width: 720px; display: flex; flex-direction: column; 
                    background-color: {BG_ELEVATED}; border: 1px solid {BORDER_DEFAULT};
                    border-radius: 12px; overflow: hidden; 
                    box-shadow: 0 25px 60px rgba(0,0,0,0.6), 0 0 0 1px rgba(255,255,255,0.03);
                ",
                
                // Header
                div {
                    style: "
                        padding: 28px 32px 24px; 
                        border-bottom: 1px solid {BORDER_DEFAULT}; 
                        background: linear-gradient(180deg, {BG_SURFACE} 0%, {BG_ELEVATED} 100%);
                    ",
                    h1 {
                        style: "
                            margin: 0; font-size: 22px; font-weight: 600;
                            color: {TEXT_PRIMARY}; letter-spacing: -0.3px;
                        ",
                        "{header_title}"
                    }
                    p {
                        style: "margin: 6px 0 0; font-size: 13px; color: {TEXT_MUTED};",
                        "{header_subtitle}"
                    }
                }
                
                // Main content area
                div {
                    style: "display: flex; min-height: 400px;",
                    
                    // ═══════════════════════════════════════════════════════════
                    // LEFT: Create New Project
                    // ═══════════════════════════════════════════════════════════
                    div {
                        style: "
                            flex: 1.2; padding: 24px 28px; 
                            {left_panel_border}
                            display: flex; flex-direction: column;
                        ",
                        
                        // Section header
                        div {
                            style: "display: flex; align-items: center; gap: 10px; margin-bottom: 20px;",
                            div {
                                style: "
                                    width: 32px; height: 32px; border-radius: 8px;
                                    background: linear-gradient(135deg, {ACCENT_VIDEO}22 0%, {ACCENT_VIDEO}11 100%);
                                    border: 1px solid {ACCENT_VIDEO}33;
                                    display: flex; align-items: center; justify-content: center;
                                    font-size: 14px;
                                ",
                                "✨"
                            }
                            h2 {
                                style: "margin: 0; font-size: 16px; font-weight: 600; color: {TEXT_PRIMARY}; letter-spacing: 0.3px;",
                                "{section_title}"
                            }
                        }
                        
                        // Form content
                        div {
                            style: "flex: 1; display: flex; flex-direction: column; gap: 18px;",
                            
                            // Project Name
                            div {
                                label { 
                                    style: "
                                        display: block; font-size: 11px; font-weight: 500;
                                        color: {TEXT_MUTED}; margin-bottom: 8px;
                                        text-transform: uppercase; letter-spacing: 0.5px;
                                    ", 
                                    "Project Name" 
                                }
                                crate::components::common::StableTextInput {
                                    id: "project-name-input".to_string(),
                                    value: name(),
                                    placeholder: Some("Enter project name...".to_string()),
                                    style: Some(format!("
                                        width: 100%; padding: 10px 14px; 
                                        background: {name_input_bg}; border: 1px solid {BORDER_DEFAULT}; 
                                        border-radius: 6px; color: {TEXT_PRIMARY}; 
                                        font-size: 13px; outline: none;
                                        transition: border-color 0.15s ease;
                                        user-select: text;
                                    ")),
                                    on_change: move |new_value: String| name.set(new_value),
                                }
                            }

                            // Resolution section
                            div {
                                label { 
                                    style: "
                                        display: block; font-size: 11px; font-weight: 500;
                                        color: {TEXT_MUTED}; margin-bottom: 8px;
                                        text-transform: uppercase; letter-spacing: 0.5px;
                                    ", 
                                    "Resolution" 
                                }
                                
                                // Preset buttons
                                div {
                                    style: "display: flex; gap: 6px; margin-bottom: 10px;",
                                    
                                    // 1080p preset
                                    {
                                        let is_active = width() == "1920" && height() == "1080";
                                        let border_color = if is_active { ACCENT_VIDEO } else { BORDER_DEFAULT };
                                        rsx! {
                                            button {
                                                style: "
                                                    padding: 6px 12px; border-radius: 6px; font-size: 11px;
                                                    border: 1px solid {border_color}; cursor: pointer;
                                                    background: {BG_SURFACE}; color: {TEXT_SECONDARY};
                                                    transition: all 0.15s ease;
                                                ",
                                                onclick: move |_| {
                                                    width.set("1920".to_string());
                                                    height.set("1080".to_string());
                                                },
                                                "1080p"
                                            }
                                        }
                                    }
                                    
                                    // 4K preset
                                    {
                                        let is_active = width() == "3840" && height() == "2160";
                                        let border_color = if is_active { ACCENT_VIDEO } else { BORDER_DEFAULT };
                                        rsx! {
                                            button {
                                                style: "
                                                    padding: 6px 12px; border-radius: 6px; font-size: 11px;
                                                    border: 1px solid {border_color}; cursor: pointer;
                                                    background: {BG_SURFACE}; color: {TEXT_SECONDARY};
                                                    transition: all 0.15s ease;
                                                ",
                                                onclick: move |_| {
                                                    width.set("3840".to_string());
                                                    height.set("2160".to_string());
                                                },
                                                "4K"
                                            }
                                        }
                                    }
                                    
                                    // Vertical (9:16) preset
                                    {
                                        let is_active = width() == "1080" && height() == "1920";
                                        let border_color = if is_active { ACCENT_VIDEO } else { BORDER_DEFAULT };
                                        rsx! {
                                            button {
                                                style: "
                                                    padding: 6px 12px; border-radius: 6px; font-size: 11px;
                                                    border: 1px solid {border_color}; cursor: pointer;
                                                    background: {BG_SURFACE}; color: {TEXT_SECONDARY};
                                                    transition: all 0.15s ease;
                                                ",
                                                onclick: move |_| {
                                                    width.set("1080".to_string());
                                                    height.set("1920".to_string());
                                                },
                                                "9:16"
                                            }
                                        }
                                    }
                                    
                                    // Square preset
                                    {
                                        let is_active = width() == "1080" && height() == "1080";
                                        let border_color = if is_active { ACCENT_VIDEO } else { BORDER_DEFAULT };
                                        rsx! {
                                            button {
                                                style: "
                                                    padding: 6px 12px; border-radius: 6px; font-size: 11px;
                                                    border: 1px solid {border_color}; cursor: pointer;
                                                    background: {BG_SURFACE}; color: {TEXT_SECONDARY};
                                                    transition: all 0.15s ease;
                                                ",
                                                onclick: move |_| {
                                                    width.set("1080".to_string());
                                                    height.set("1080".to_string());
                                                },
                                                "1:1"
                                            }
                                        }
                                    }
                                }
                                
                                // Custom resolution inputs
                                div {
                                    style: "display: flex; gap: 8px; align-items: center;",
                                    crate::components::common::StableNumberInput {
                                        id: "width-input".to_string(),
                                        value: width(),
                                        placeholder: None,
                                        style: Some(format!("
                                            flex: 1; padding: 10px 12px; background: {};
                                            border: 1px solid {}; border-radius: 6px;
                                            color: {}; font-size: 13px; outline: none;
                                            text-align: center; transition: border-color 0.15s ease;
                                            user-select: text;
                                        ", BG_BASE, BORDER_DEFAULT, TEXT_PRIMARY)),
                                        min: Some("1".to_string()),
                                        max: None,
                                        step: Some("1".to_string()),
                                        on_change: move |v: String| width.set(v),
                                    }
                                    span { 
                                        style: "color: {TEXT_DIM}; font-size: 12px; font-weight: 500;", 
                                        "×" 
                                    }
                                    crate::components::common::StableNumberInput {
                                        id: "height-input".to_string(),
                                        value: height(),
                                        placeholder: None,
                                        style: Some(format!("
                                            flex: 1; padding: 10px 12px; background: {};
                                            border: 1px solid {}; border-radius: 6px;
                                            color: {}; font-size: 13px; outline: none;
                                            text-align: center; transition: border-color 0.15s ease;
                                            user-select: text;
                                        ", BG_BASE, BORDER_DEFAULT, TEXT_PRIMARY)),
                                        min: Some("1".to_string()),
                                        max: None,
                                        step: Some("1".to_string()),
                                        on_change: move |v: String| height.set(v),
                                    }
                                }
                            }

                            // Preview downsample section
                            div {
                                div {
                                    style: "display: flex; align-items: center; gap: 6px; margin-bottom: 8px;",
                                    label {
                                        style: "
                                            display: block; font-size: 11px; font-weight: 500;
                                            color: {TEXT_MUTED};
                                            text-transform: uppercase; letter-spacing: 0.5px;
                                        ",
                                        "Preview Downsample"
                                    }
                                    div {
                                        class: "info-tooltip",
                                        style: "
                                            position: relative; width: 14px; height: 14px;
                                            border-radius: 50%; border: 1px solid {TEXT_DIM};
                                            display: flex; align-items: center; justify-content: center;
                                            font-size: 9px; color: {TEXT_DIM}; cursor: help;
                                        ",
                                        "!"
                                        // Tooltip on hover
                                        div {
                                            style: "
                                                position: absolute; left: 20px; top: -8px;
                                                background: {BG_ELEVATED}; border: 1px solid {BORDER_DEFAULT};
                                                border-radius: 6px; padding: 8px 12px;
                                                font-size: 11px; color: {TEXT_SECONDARY};
                                                white-space: nowrap; pointer-events: none;
                                                opacity: 0; transition: opacity 0.2s ease;
                                                box-shadow: 0 4px 12px rgba(0,0,0,0.3);
                                                z-index: 1000;
                                            ",
                                            class: "tooltip-content",
                                            "Caps the realtime preview size for smoother playback."
                                        }
                                    }
                                }
                                div {
                                    style: "display: flex; gap: 8px; align-items: center;",
                                    crate::components::common::StableNumberInput {
                                        id: "preview-max-width-input".to_string(),
                                        value: preview_max_width(),
                                        placeholder: None,
                                        style: Some(format!("
                                            flex: 1; padding: 10px 12px; background: {};
                                            border: 1px solid {}; border-radius: 6px;
                                            color: {}; font-size: 13px; outline: none;
                                            text-align: center; transition: border-color 0.15s ease;
                                            user-select: text;
                                        ", BG_BASE, BORDER_DEFAULT, TEXT_PRIMARY)),
                                        min: Some("1".to_string()),
                                        max: None,
                                        step: Some("1".to_string()),
                                        on_change: move |v: String| preview_max_width.set(v),
                                    }
                                    span {
                                        style: "color: {TEXT_DIM}; font-size: 12px; font-weight: 500;",
                                        "×"
                                    }
                                    crate::components::common::StableNumberInput {
                                        id: "preview-max-height-input".to_string(),
                                        value: preview_max_height(),
                                        placeholder: None,
                                        style: Some(format!("
                                            flex: 1; padding: 10px 12px; background: {};
                                            border: 1px solid {}; border-radius: 6px;
                                            color: {}; font-size: 13px; outline: none;
                                            text-align: center; transition: border-color 0.15s ease;
                                            user-select: text;
                                        ", BG_BASE, BORDER_DEFAULT, TEXT_PRIMARY)),
                                        min: Some("1".to_string()),
                                        max: None,
                                        step: Some("1".to_string()),
                                        on_change: move |v: String| preview_max_height.set(v),
                                    }
                                }
                            }

                            // FPS & Duration row
                            div {
                                style: "display: flex; gap: 20px;",
                                div {
                                    style: "flex: 1;",
                                    label { 
                                        style: "
                                            display: block; font-size: 11px; font-weight: 500;
                                            color: {TEXT_MUTED}; margin-bottom: 8px;
                                            text-transform: uppercase; letter-spacing: 0.5px;
                                        ", 
                                        "Frame Rate" 
                                    }
                                    div {
                                        style: "position: relative; flex: 1;",
                                        crate::components::common::StableNumberInput {
                                            id: "fps-input".to_string(),
                                            value: fps(),
                                            placeholder: None,
                                            style: Some(format!("
                                                width: 100%; padding: 10px 12px; padding-right: 40px; background: {};
                                                border: 1px solid {}; border-radius: 6px;
                                                color: {}; font-size: 13px; outline: none;
                                                transition: border-color 0.15s ease;
                                                user-select: text;
                                            ", BG_BASE, BORDER_DEFAULT, TEXT_PRIMARY)),
                                            min: Some("1".to_string()),
                                            max: None,
                                            step: Some("1".to_string()),
                                            on_change: move |v: String| fps.set(v),
                                        }
                                        span {
                                            style: "
                                                position: absolute; right: 12px; top: 50%; transform: translateY(-50%);
                                                color: {TEXT_DIM}; font-size: 11px; pointer-events: none;
                                            ",
                                            "fps"
                                        }
                                    }
                                }
                                div {
                                    style: "flex: 1;",
                                    label { 
                                        style: "
                                            display: block; font-size: 11px; font-weight: 500;
                                            color: {TEXT_MUTED}; margin-bottom: 8px;
                                            text-transform: uppercase; letter-spacing: 0.5px;
                                        ", 
                                        "Duration" 
                                    }
                                    div {
                                        style: "position: relative; flex: 1;",
                                        crate::components::common::StableNumberInput {
                                            id: "duration-input".to_string(),
                                            value: duration(),
                                            placeholder: None,
                                            style: Some(format!("
                                                width: 100%; padding: 10px 12px; padding-right: 40px; background: {};
                                                border: 1px solid {}; border-radius: 6px;
                                                color: {}; font-size: 13px; outline: none;
                                                transition: border-color 0.15s ease;
                                                user-select: text;
                                            ", BG_BASE, BORDER_DEFAULT, TEXT_PRIMARY)),
                                            min: Some("1".to_string()),
                                            max: None,
                                            step: Some("1".to_string()),
                                            on_change: move |v: String| duration.set(v),
                                        }
                                        span {
                                            style: "
                                                position: absolute; right: 12px; top: 50%; transform: translateY(-50%);
                                                color: {TEXT_DIM}; font-size: 11px; pointer-events: none;
                                            ",
                                            "sec"
                                        }
                                    }
                                }
                            }

                            // Divider
                            div { 
                                style: "height: 1px; background: linear-gradient(90deg, {BORDER_SUBTLE} 0%, transparent 100%); margin: 8px 0;" 
                            }
                            
                            // Location
                            if is_edit {
                                div {
                                    label {
                                        style: "
                                            display: block; font-size: 11px; font-weight: 500;
                                            color: {TEXT_MUTED}; margin-bottom: 8px;
                                            text-transform: uppercase; letter-spacing: 0.5px;
                                        ",
                                        "Project Folder"
                                    }
                                    div {
                                        style: "
                                            padding: 8px 12px; background: {BG_BASE};
                                            border: 1px solid {BORDER_DEFAULT}; border-radius: 6px;
                                            color: {TEXT_DIM}; font-size: 12px;
                                            overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
                                        ",
                                        "{parent_dir().to_string_lossy()}"
                                    }
                                }
                            } else {
                                div {
                                    label { 
                                        style: "
                                            display: block; font-size: 11px; font-weight: 500;
                                            color: {TEXT_MUTED}; margin-bottom: 8px;
                                            text-transform: uppercase; letter-spacing: 0.5px;
                                        ", 
                                        "Save Location" 
                                    }
                                    div {
                                        style: "display: flex; gap: 8px;",
                                        div {
                                            style: "
                                                flex: 1; padding: 8px 12px; background: {BG_BASE};
                                                border: 1px solid {BORDER_DEFAULT}; border-radius: 6px;
                                                color: {TEXT_DIM}; font-size: 12px;
                                                overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
                                            ",
                                            "{parent_dir().to_string_lossy()}"
                                        }
                                        button {
                                            class: "collapse-btn",
                                            style: "
                                                padding: 8px 14px; background: {BG_SURFACE}; 
                                                border: 1px solid {BORDER_DEFAULT}; border-radius: 6px; 
                                                color: {TEXT_SECONDARY}; font-size: 12px; cursor: pointer;
                                                transition: all 0.15s ease;
                                            ",
                                            onclick: move |_| {
                                                let start_dir = projects_folder_for_browse.clone();
                                                if let Some(path) = rfd::FileDialog::new()
                                                    .set_directory(&start_dir)
                                                    .pick_folder() 
                                                {
                                                    parent_dir.set(path);
                                                }
                                            },
                                            "Browse"
                                        }
                                    }
                                }
                            }
                        }
                        
                        // Create/edit actions
                        if is_edit {
                            div {
                                style: "display: flex; gap: 12px; margin-top: 20px;",
                                button {
                                    class: "collapse-btn",
                                    style: "
                                        flex: 1; padding: 12px; border-radius: 8px;
                                        background: {BG_SURFACE}; border: 1px solid {BORDER_DEFAULT};
                                        color: {TEXT_SECONDARY}; font-size: 12px; font-weight: 600;
                                        cursor: pointer; transition: all 0.2s ease;
                                    ",
                                    onclick: move |e| {
                                        on_close.call(e);
                                    },
                                    "Cancel"
                                }
                                button {
                                    class: "collapse-btn",
                                    style: "
                                        flex: 1; padding: 12px;
                                        background: linear-gradient(180deg, {ACCENT_VIDEO} 0%, #1ea34b 100%);
                                        border: none; border-radius: 8px;
                                        color: white; font-size: 13px; font-weight: 600;
                                        cursor: pointer; transition: all 0.2s ease;
                                        box-shadow: 0 2px 8px rgba(34, 197, 94, 0.3);
                                    ",
                                    onclick: move |e| {
                                        let settings = crate::state::ProjectSettings {
                                            width: parse_u32(&width(), width_default, 1),
                                            height: parse_u32(&height(), height_default, 1),
                                            fps: parse_f64(&fps(), fps_default, 1.0),
                                            duration_seconds: parse_f64(&duration(), duration_default, 1.0),
                                            preview_max_width: parse_u32(
                                                &preview_max_width(),
                                                preview_default_width,
                                                1,
                                            ),
                                            preview_max_height: parse_u32(
                                                &preview_max_height(),
                                                preview_default_height,
                                                1,
                                            ),
                                        };
                                        on_update.call(settings);
                                        on_close.call(e);
                                    },
                                    "Save Changes"
                                }
                            }
                        } else {
                            button {
                                class: "collapse-btn",
                                style: "
                                    width: 100%; padding: 12px; margin-top: 20px;
                                    background: linear-gradient(180deg, {ACCENT_VIDEO} 0%, #1ea34b 100%);
                                    border: none; border-radius: 8px;
                                    color: white; font-size: 13px; font-weight: 600; 
                                    cursor: pointer; transition: all 0.2s ease;
                                    box-shadow: 0 2px 8px rgba(34, 197, 94, 0.3);
                                ",
                                onclick: move |_| {
                                    let n = name();
                                    if !n.trim().is_empty() {
                                        let settings = crate::state::ProjectSettings {
                                            width: parse_u32(&width(), width_default, 1),
                                            height: parse_u32(&height(), height_default, 1),
                                            fps: parse_f64(&fps(), fps_default, 1.0),
                                            duration_seconds: parse_f64(&duration(), duration_default, 1.0),
                                            preview_max_width: parse_u32(
                                                &preview_max_width(),
                                                preview_default_width,
                                                1,
                                            ),
                                            preview_max_height: parse_u32(
                                                &preview_max_height(),
                                                preview_default_height,
                                                1,
                                            ),
                                        };
                                        on_create.call((parent_dir(), n, settings));
                                    }
                                },
                                "Create Project"
                            }
                        }
                    }
                    
                    // ═══════════════════════════════════════════════════════════
                    // RIGHT: Open Existing Project
                    // ═══════════════════════════════════════════════════════════
                    if !is_edit {
                        div {
                            style: "
                                flex: 0.8; padding: 24px 28px;
                                display: flex; flex-direction: column;
                                background-color: {BG_BASE};
                                min-width: 0; overflow: hidden;
                            ",

                            // Section header
                            div {
                                style: "display: flex; align-items: center; gap: 10px; margin-bottom: 16px;",
                                div {
                                    style: "
                                        width: 32px; height: 32px; border-radius: 8px;
                                        background: linear-gradient(135deg, {ACCENT_AUDIO}22 0%, {ACCENT_AUDIO}11 100%);
                                        border: 1px solid {ACCENT_AUDIO}33;
                                        display: flex; align-items: center; justify-content: center;
                                        font-size: 14px;
                                    ",
                                    "📂"
                                }
                                h2 {
                                    style: "margin: 0; font-size: 16px; font-weight: 600; color: {TEXT_PRIMARY}; letter-spacing: 0.3px;",
                                    "Recent Projects"
                                }
                            }

                            // Project list or empty state
                            if existing_projects.is_empty() {
                                div {
                                    style: "
                                        flex: 1; display: flex; flex-direction: column;
                                        align-items: center; justify-content: center;
                                        border: 1px dashed {BORDER_DEFAULT}; border-radius: 8px;
                                        padding: 32px;
                                    ",
                                    div {
                                        style: "font-size: 40px; opacity: 0.3; margin-bottom: 12px;",
                                        "📁"
                                    }
                                    p {
                                        style: "margin: 0; font-size: 13px; color: {TEXT_DIM}; text-align: center;",
                                        "No projects yet"
                                    }
                                    p {
                                        style: "margin: 6px 0 0; font-size: 11px; color: {TEXT_DIM}; text-align: center;",
                                        "Create one to get started"
                                    }
                                }
                            } else {
                                div {
                                    style: "
                                        flex: 1; overflow-y: auto; overflow-x: hidden;
                                        border: 1px solid {BORDER_SUBTLE}; border-radius: 8px;
                                        background-color: {BG_ELEVATED};
                                        min-height: 0;
                                    ",
                                    for (proj_name, proj_path) in existing_projects.iter() {
                                        {
                                            let path_clone = proj_path.clone();
                                            let path_for_menu = proj_path.clone();
                                            let name_for_menu = proj_name.clone();
                                            let on_open_clone = on_open.clone();
                                            rsx! {
                                                div {
                                                    class: "collapse-btn",
                                                    key: "{proj_path.display()}",
                                                    style: "
                                                        padding: 12px 14px; cursor: pointer;
                                                        border-bottom: 1px solid {BORDER_SUBTLE};
                                                        transition: background-color 0.15s ease;
                                                    ",
                                                    onclick: move |_| {
                                                        on_open_clone.call(path_clone.clone());
                                                    },
                                                    oncontextmenu: move |e| {
                                                        e.prevent_default();
                                                        context_menu.set(Some((
                                                            e.client_coordinates().x,
                                                            e.client_coordinates().y,
                                                            path_for_menu.clone(),
                                                            name_for_menu.clone()
                                                        )));
                                                    },
                                                    div {
                                                        style: "display: flex; align-items: center; gap: 10px; min-width: 0;",
                                                        div {
                                                            style: "
                                                                width: 28px; height: 28px; border-radius: 6px;
                                                                background: {BG_SURFACE}; border: 1px solid {BORDER_SUBTLE};
                                                                display: flex; align-items: center; justify-content: center;
                                                                font-size: 12px; flex-shrink: 0;
                                                            ",
                                                            "🎬"
                                                        }
                                                        div {
                                                            style: "flex: 1; min-width: 0; overflow: hidden;",
                                                            div {
                                                                style: "
                                                                    font-size: 13px; font-weight: 500; color: {TEXT_PRIMARY};
                                                                    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
                                                                ",
                                                                "{proj_name}"
                                                            }
                                                        }
                                                        // Arrow indicator
                                                        span {
                                                            style: "color: {TEXT_DIM}; font-size: 10px;",
                                                            "→"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // Browse button
                            button {
                                class: "collapse-btn",
                                style: "
                                    width: 100%; padding: 10px; margin-top: 16px; flex-shrink: 0;
                                    background-color: {BG_SURFACE}; border: 1px solid {BORDER_DEFAULT};
                                    border-radius: 8px; color: {TEXT_SECONDARY};
                                    font-size: 12px; font-weight: 500; cursor: pointer;
                                    transition: all 0.15s ease;
                                    display: flex; align-items: center; justify-content: center; gap: 6px;
                                ",
                                onclick: move |_| {
                                    let start_dir = projects_folder_for_open.clone();
                                    if let Some(path) = rfd::FileDialog::new()
                                        .set_directory(&start_dir)
                                        .set_title("Open Project")
                                        .pick_folder()
                                    {
                                        on_open.call(path);
                                    }
                                },
                                span { style: "font-size: 11px;", "📁" }
                                "Browse for Project..."
                            }
                        }
                    }
                }
            }
            
            // Context menu overlay for project deletion
            if let Some((x, y, proj_path, proj_name)) = context_menu() {
                // Backdrop to catch clicks outside menu
                div {
                    style: "
                        position: fixed; top: 0; left: 0; right: 0; bottom: 0;
                        z-index: 10000;
                    ",
                    onclick: move |_| context_menu.set(None),
                }
                // The actual menu
                div {
                    style: "
                        position: fixed; 
                        left: min({x}px, calc(100vw - 160px)); 
                        top: min({y}px, calc(100vh - 60px));
                        background-color: {BG_ELEVATED}; border: 1px solid {BORDER_DEFAULT};
                        border-radius: 8px; padding: 4px 0; min-width: 160px;
                        box-shadow: 0 8px 24px rgba(0,0,0,0.4);
                        z-index: 10001; font-size: 12px;
                    ",
                    div {
                        class: "collapse-btn",
                        style: "
                            padding: 8px 14px; color: #ef4444; cursor: pointer;
                            display: flex; align-items: center; gap: 8px;
                            transition: background-color 0.1s ease;
                        ",
                        onclick: move |_| {
                            // Delete the project folder
                            if let Err(e) = std::fs::remove_dir_all(&proj_path) {
                                println!("Failed to delete project {:?}: {}", proj_path, e);
                            } else {
                                println!("Deleted project: {:?}", proj_path);
                            }
                            // Close menu and refresh list
                            context_menu.set(None);
                            refresh_counter.set(refresh_counter() + 1);
                        },
                        span { "🗑" }
                        "Delete \"{proj_name}\""
                    }
                }
            }
        }
    }
}



