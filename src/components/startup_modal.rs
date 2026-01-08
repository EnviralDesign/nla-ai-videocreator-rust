use dioxus::prelude::*;
use std::path::PathBuf;
use crate::constants::*;
use crate::state::ProjectSettings;

#[component]
pub fn StartupModal(
    on_create: EventHandler<(PathBuf, String, ProjectSettings)>,
    on_open: EventHandler<PathBuf>,
) -> Element {
    let mut name = use_signal(|| "My New Project".to_string());
    let mut width = use_signal(|| "1920".to_string());
    let mut height = use_signal(|| "1080".to_string());
    let mut fps = use_signal(|| "60".to_string());
    let mut duration = use_signal(|| "60".to_string());
    
    // Default projects folder
    let projects_folder = std::env::current_dir().unwrap_or_default().join("projects");
    let projects_folder_clone = projects_folder.clone();
    let projects_folder_for_browse = projects_folder.clone();
    let projects_folder_for_open = projects_folder.clone();
    let projects_folder_for_scan = projects_folder.clone();
    
    // Use `Option<PathBuf>` to store the selected parent directory
    let mut parent_dir = use_signal(move || projects_folder_clone.clone());
    
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
                        "NLA AI Video Creator" 
                    }
                    p { 
                        style: "margin: 6px 0 0; font-size: 13px; color: {TEXT_MUTED};", 
                        "Create a new project or open an existing one" 
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
                            border-right: 1px solid {BORDER_DEFAULT}; 
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
                                style: "margin: 0; font-size: 15px; font-weight: 600; color: {TEXT_PRIMARY};", 
                                "Create New Project" 
                            }
                        }
                        
                        // Form content
                        div {
                            style: "flex: 1; display: flex; flex-direction: column; gap: 16px;",
                            
                            // Project Name
                            div {
                                label { 
                                    style: "
                                        display: block; font-size: 11px; font-weight: 500;
                                        color: {TEXT_MUTED}; margin-bottom: 6px;
                                        text-transform: uppercase; letter-spacing: 0.5px;
                                    ", 
                                    "Project Name" 
                                }
                                input {
                                    style: "
                                        width: 100%; padding: 10px 14px; 
                                        background: {BG_BASE}; border: 1px solid {BORDER_DEFAULT}; 
                                        border-radius: 6px; color: {TEXT_PRIMARY}; 
                                        font-size: 13px; outline: none;
                                        transition: border-color 0.15s ease;
                                        user-select: text;
                                    ",
                                    value: "{name}",
                                    placeholder: "Enter project name...",
                                    oninput: move |e| name.set(e.value()),
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
                                    button {
                                        style: "
                                            padding: 6px 12px; border-radius: 6px; font-size: 11px;
                                            border: 1px solid {BORDER_DEFAULT}; cursor: pointer;
                                            background: {BG_SURFACE}; color: {TEXT_SECONDARY};
                                            transition: all 0.15s ease;
                                        ",
                                        onclick: move |_| {
                                            width.set("1920".to_string());
                                            height.set("1080".to_string());
                                        },
                                        "1080p"
                                    }
                                    
                                    // 4K preset
                                    button {
                                        style: "
                                            padding: 6px 12px; border-radius: 6px; font-size: 11px;
                                            border: 1px solid {BORDER_DEFAULT}; cursor: pointer;
                                            background: {BG_SURFACE}; color: {TEXT_SECONDARY};
                                            transition: all 0.15s ease;
                                        ",
                                        onclick: move |_| {
                                            width.set("3840".to_string());
                                            height.set("2160".to_string());
                                        },
                                        "4K"
                                    }
                                    
                                    // Vertical (9:16) preset
                                    button {
                                        style: "
                                            padding: 6px 12px; border-radius: 6px; font-size: 11px;
                                            border: 1px solid {BORDER_DEFAULT}; cursor: pointer;
                                            background: {BG_SURFACE}; color: {TEXT_SECONDARY};
                                            transition: all 0.15s ease;
                                        ",
                                        onclick: move |_| {
                                            width.set("1080".to_string());
                                            height.set("1920".to_string());
                                        },
                                        "9:16"
                                    }
                                    
                                    // Square preset
                                    button {
                                        style: "
                                            padding: 6px 12px; border-radius: 6px; font-size: 11px;
                                            border: 1px solid {BORDER_DEFAULT}; cursor: pointer;
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
                                
                                // Custom resolution inputs
                                div {
                                    style: "display: flex; gap: 8px; align-items: center;",
                                    input {
                                        style: "
                                            flex: 1; padding: 8px 12px; background: {BG_BASE};
                                            border: 1px solid {BORDER_DEFAULT}; border-radius: 6px;
                                            color: {TEXT_PRIMARY}; font-size: 13px; outline: none;
                                            text-align: center;
                                            user-select: text;
                                        ",
                                        r#type: "number",
                                        min: "1",
                                        step: "1",
                                        value: "{width}",
                                        oninput: move |e| width.set(e.value()),
                                    }
                                    span { 
                                        style: "color: {TEXT_DIM}; font-size: 12px; font-weight: 500;", 
                                        "×" 
                                    }
                                    input {
                                        style: "
                                            flex: 1; padding: 8px 12px; background: {BG_BASE};
                                            border: 1px solid {BORDER_DEFAULT}; border-radius: 6px;
                                            color: {TEXT_PRIMARY}; font-size: 13px; outline: none;
                                            text-align: center;
                                            user-select: text;
                                        ",
                                        r#type: "number",
                                        min: "1",
                                        step: "1",
                                        value: "{height}",
                                        oninput: move |e| height.set(e.value()),
                                    }
                                }
                            }

                            // FPS & Duration row
                            div {
                                style: "display: flex; gap: 16px;",
                                div {
                                    style: "flex: 1;",
                                    label { 
                                        style: "
                                            display: block; font-size: 11px; font-weight: 500;
                                            color: {TEXT_MUTED}; margin-bottom: 6px;
                                            text-transform: uppercase; letter-spacing: 0.5px;
                                        ", 
                                        "Frame Rate" 
                                    }
                                    div {
                                        style: "display: flex; align-items: center; gap: 6px;",
                                        input {
                                            style: "
                                                flex: 1; padding: 8px 12px; background: {BG_BASE};
                                                border: 1px solid {BORDER_DEFAULT}; border-radius: 6px;
                                                color: {TEXT_PRIMARY}; font-size: 13px; outline: none;
                                                user-select: text;
                                            ",
                                            r#type: "number",
                                            min: "1",
                                            step: "1",
                                            value: "{fps}",
                                            oninput: move |e| fps.set(e.value()),
                                        }
                                        span { 
                                            style: "color: {TEXT_DIM}; font-size: 11px;", 
                                            "fps" 
                                        }
                                    }
                                }
                                div {
                                    style: "flex: 1;",
                                    label { 
                                        style: "
                                            display: block; font-size: 11px; font-weight: 500;
                                            color: {TEXT_MUTED}; margin-bottom: 6px;
                                            text-transform: uppercase; letter-spacing: 0.5px;
                                        ", 
                                        "Duration" 
                                    }
                                    div {
                                        style: "display: flex; align-items: center; gap: 6px;",
                                        input {
                                            style: "
                                                flex: 1; padding: 8px 12px; background: {BG_BASE};
                                                border: 1px solid {BORDER_DEFAULT}; border-radius: 6px;
                                                color: {TEXT_PRIMARY}; font-size: 13px; outline: none;
                                                user-select: text;
                                            ",
                                            r#type: "number",
                                            min: "1",
                                            step: "1",
                                            value: "{duration}",
                                            oninput: move |e| duration.set(e.value()),
                                        }
                                        span { 
                                            style: "color: {TEXT_DIM}; font-size: 11px;", 
                                            "sec" 
                                        }
                                    }
                                }
                            }

                            // Divider
                            div { 
                                style: "height: 1px; background: linear-gradient(90deg, {BORDER_SUBTLE} 0%, transparent 100%); margin: 4px 0;" 
                            }
                            
                            // Location
                            div {
                                label { 
                                    style: "
                                        display: block; font-size: 11px; font-weight: 500;
                                        color: {TEXT_MUTED}; margin-bottom: 6px;
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
                        
                        // Create button
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
                                        width: parse_u32(&width(), 1920, 1),
                                        height: parse_u32(&height(), 1080, 1),
                                        fps: parse_f64(&fps(), 60.0, 1.0),
                                        duration_seconds: parse_f64(&duration(), 60.0, 1.0),
                                    };
                                    on_create.call((parent_dir(), n, settings));
                                }
                            },
                            "Create Project"
                        }
                    }
                    
                    // ═══════════════════════════════════════════════════════════
                    // RIGHT: Open Existing Project
                    // ═══════════════════════════════════════════════════════════
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
                                style: "margin: 0; font-size: 15px; font-weight: 600; color: {TEXT_PRIMARY};", 
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



