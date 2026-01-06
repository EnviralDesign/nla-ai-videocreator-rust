//! Root application component
//! 
//! This defines the main App component and the overall layout structure.

use dioxus::prelude::*;
use crate::timeline::TimelinePanel;

// =============================================================================
// COLOR SCHEME - Charcoal Monochrome with Functional Accents
// =============================================================================
pub const BG_DEEPEST: &str = "#09090b";
pub const BG_BASE: &str = "#0a0a0b";
pub const BG_ELEVATED: &str = "#141414";
pub const BG_SURFACE: &str = "#1a1a1a";
pub const BG_HOVER: &str = "#262626";

pub const BORDER_SUBTLE: &str = "#1f1f1f";
pub const BORDER_DEFAULT: &str = "#27272a";
pub const BORDER_STRONG: &str = "#3f3f46";
pub const BORDER_ACCENT: &str = "#3b82f6";

pub const TEXT_PRIMARY: &str = "#fafafa";
pub const TEXT_SECONDARY: &str = "#a1a1aa";
pub const TEXT_MUTED: &str = "#71717a";
pub const TEXT_DIM: &str = "#52525b";

pub const ACCENT_AUDIO: &str = "#3b82f6";
pub const ACCENT_MARKER: &str = "#f97316";

pub const ACCENT_VIDEO: &str = "#22c55e";

// Panel dimensions
const PANEL_MIN_WIDTH: f64 = 180.0;
const PANEL_MAX_WIDTH: f64 = 400.0;
const PANEL_DEFAULT_WIDTH: f64 = 250.0;
const PANEL_COLLAPSED_WIDTH: f64 = 40.0;
const TIMELINE_MIN_HEIGHT: f64 = 100.0;
const TIMELINE_MAX_HEIGHT: f64 = 500.0;
const TIMELINE_DEFAULT_HEIGHT: f64 = 220.0;
const TIMELINE_COLLAPSED_HEIGHT: f64 = 32.0;  // Must match header height exactly

/// Main application component
#[component]
pub fn App() -> Element {
    // Project state - the core data model
    let mut project = use_signal(|| crate::state::Project::default());
    
    // Panel state
    let mut left_width = use_signal(|| PANEL_DEFAULT_WIDTH);
    let mut left_collapsed = use_signal(|| false);
    let mut right_width = use_signal(|| PANEL_DEFAULT_WIDTH);
    let mut right_collapsed = use_signal(|| false);
    let mut timeline_height = use_signal(|| TIMELINE_DEFAULT_HEIGHT);
    let mut timeline_collapsed = use_signal(|| false);
    
    // Timeline playback state
    let mut current_time = use_signal(|| 0.0_f64);        // Current time in seconds
    let mut zoom = use_signal(|| 100.0_f64);              // Pixels per second
    let mut is_playing = use_signal(|| false);            // Playback state
    let mut scroll_offset = use_signal(|| 0.0_f64);       // Horizontal scroll position
    
    // Derive duration from project
    let duration = project.read().duration();
    
    // Drag state
    let mut dragging = use_signal(|| None::<&'static str>);
    let mut drag_start_pos = use_signal(|| 0.0);
    let mut drag_start_size = use_signal(|| 0.0);
    
    // Asset Drag & Drop state
    let mut dragged_asset = use_signal(|| None::<uuid::Uuid>);
    let mut mouse_pos = use_signal(|| (0.0, 0.0));
    
    // Context menu state: (x, y, track_id) - None means no menu shown
    let mut context_menu = use_signal(|| None::<(f64, f64, uuid::Uuid)>);

    // Dialog state
    let mut show_new_project_dialog = use_signal(|| false); // Kept for "File > New" inside app
    
    // Startup Modal state - check if we have a valid project path on load
    // For MVP, we start with a dummy project, so we check if project_path is None
    let mut startup_done = use_signal(|| false); 
    
    // On first load, if project has no path effectively, treat as "No Project Loaded"
    // But since we initialize with default(), we need a flag to block interaction until New/Open
    // We'll use specific "show_startup_modal" derived state
    
    let show_startup = project.read().project_path.is_none() && !startup_done();

    // Read current values
    let left_w = if left_collapsed() { PANEL_COLLAPSED_WIDTH } else { left_width() };
    let right_w = if right_collapsed() { PANEL_COLLAPSED_WIDTH } else { right_width() };
    let timeline_h = if timeline_collapsed() { TIMELINE_COLLAPSED_HEIGHT } else { timeline_height() };
    
    // Is currently dragging? (for cursor and user-select styling)
    let is_dragging = dragging().is_some();
    let user_select_style = if is_dragging { "none" } else { "auto" };
    let drag_cursor = match dragging() {
        Some("left") | Some("right") => "ew-resize",
        Some("timeline") => "ns-resize",
        Some("playhead") => "ew-resize",
        _ => "default",
    };
    
    // Ghost asset for drag and drop
    let drag_ghost_asset = if let Some(id) = dragged_asset() {
        project.read().assets.iter().find(|a| a.id == id).cloned()
    } else {
        None
    };
    
    // Which panel is currently being resized? (to disable transitions during drag)
    let left_resizing = dragging() == Some("left");
    let right_resizing = dragging() == Some("right");
    let timeline_resizing = dragging() == Some("timeline");

    rsx! {
        // Global CSS with drag state handling
        style {
            r#"
            *, *::before, *::after {{ box-sizing: border-box; }}
            html, body {{ margin: 0; padding: 0; overflow: hidden; background-color: {BG_BASE}; }}
            body {{ -webkit-font-smoothing: antialiased; }}
            ::-webkit-scrollbar {{ width: 6px; height: 6px; }}
            ::-webkit-scrollbar-track {{ background: transparent; }}
            ::-webkit-scrollbar-thumb {{ background: {BORDER_DEFAULT}; border-radius: 3px; }}
            ::-webkit-scrollbar-thumb:hover {{ background: {BORDER_STRONG}; }}
            .collapse-btn {{ opacity: 0.6; transition: opacity 0.15s ease, background-color 0.15s ease; }}
            .collapse-btn:hover {{ opacity: 1; background-color: {BG_HOVER} !important; }}
            .resize-handle {{ transition: background-color 0.15s ease; }}
            .resize-handle:hover {{ background-color: {BORDER_ACCENT} !important; }}
            .resize-handle:active {{ background-color: {BORDER_ACCENT} !important; }}
            .collapsed-rail {{ transition: background-color 0.15s ease; }}
            .collapsed-rail:hover {{ background-color: {BG_HOVER} !important; }}
            .resize-handle-left:hover > div, .resize-handle-right:hover > div {{ opacity: 1 !important; }}
            "#
        }

        // Main app container
        div {
            class: "app-container",
            style: "
                display: flex; flex-direction: column;
                width: 100vw; height: 100vh;
                background-color: {BG_BASE}; color: {TEXT_PRIMARY};
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
                overflow: hidden; position: fixed; top: 0; left: 0;
                user-select: {user_select_style};
                cursor: {drag_cursor};
            ",
            
            onmousemove: move |e| {
                if dragged_asset().is_some() {
                    mouse_pos.set((e.client_coordinates().x, e.client_coordinates().y));
                }
                
                if let Some(target) = dragging() {
                    e.prevent_default();
                    match target {
                        "left" => {
                            let delta = e.client_coordinates().x - drag_start_pos();
                            let new_w = (drag_start_size() + delta).clamp(PANEL_MIN_WIDTH, PANEL_MAX_WIDTH);
                            left_width.set(new_w);
                        }
                        "right" => {
                            let delta = drag_start_pos() - e.client_coordinates().x;
                            let new_w = (drag_start_size() + delta).clamp(PANEL_MIN_WIDTH, PANEL_MAX_WIDTH);
                            right_width.set(new_w);
                        }
                        "timeline" => {
                            let delta = drag_start_pos() - e.client_coordinates().y;
                            let new_h = (drag_start_size() + delta).clamp(TIMELINE_MIN_HEIGHT, TIMELINE_MAX_HEIGHT);
                            timeline_height.set(new_h);
                        }
                        "playhead" => {
                            // Convert mouse x delta to time delta using zoom factor
                            let delta_px = e.client_coordinates().x - drag_start_pos();
                            let delta_time = delta_px / zoom();
                            let raw_time = (drag_start_size() + delta_time).clamp(0.0, duration);
                            // Snap to frame boundary (60fps)
                            let snapped_time = (raw_time * 60.0).round() / 60.0;
                            current_time.set(snapped_time);
                        }
                        _ => {}
                    }
                }
            },
            onmouseup: move |_| {
                dragging.set(None);
                dragged_asset.set(None);
            },
            // Suppress the browser's default context menu - we'll use custom menus
            oncontextmenu: move |e| e.prevent_default(),
            // Note: We intentionally don't clear drag on mouseleave so drag continues
            // if the user moves outside the window and back in while still holding mouse button

            // Drag Ghost
            if let Some(asset) = drag_ghost_asset {
                div {
                    style: "
                        position: fixed; left: {mouse_pos().0 + 15.0}px; top: {mouse_pos().1 + 15.0}px;
                        background-color: {BG_ELEVATED}; border: 1px solid {ACCENT_VIDEO};
                        border-radius: 4px; padding: 6px 10px; font-size: 12px; pointer-events: none;
                        z-index: 10000; box-shadow: 0 4px 12px rgba(0,0,0,0.3); opacity: 0.9;
                        color: {TEXT_PRIMARY}; display: flex; align-items: center; gap: 6px;
                    ",
                    span { "📄" } // Generic icon for now
                    "{asset.name}"
                }
            }

            TitleBar { 
                project_name: project.read().name.clone(),
                on_new_project: move |_| {
                    show_new_project_dialog.set(true);
                }
            }

            // Main content
            div {
                class: "main-content",
                style: "display: flex; flex: 1; overflow: hidden;",

                // Left panel - Assets
                SidePanel {
                    title: "Assets",
                    width: left_w,
                    collapsed: left_collapsed(),
                    side: "left",
                    is_resizing: left_resizing,
                    on_toggle: move |_| left_collapsed.set(!left_collapsed()),
                    on_resize_start: move |e: MouseEvent| {
                        e.prevent_default();
                        dragging.set(Some("left"));
                        drag_start_pos.set(e.client_coordinates().x);
                        drag_start_size.set(left_width());
                    },
                    
                    // Assets panel content
                    AssetsPanelContent {
                        assets: project.read().assets.clone(),
                        on_import: move |asset| {
                            project.write().add_asset(asset);
                        },
                        on_import_file: move |path: std::path::PathBuf| {
                            // Implicit Copy: Always import directly designated by the strict folder policy
                            // We assume a project exists because the startup modal blocks everything else
                            if let Err(e) = project.write().import_file(&path) {
                                println!("Failed to import file {:?}: {}", path, e);
                            }
                        },
                        on_delete: move |id| {
                            project.write().remove_asset(id);
                        },
                        on_add_to_timeline: move |asset_id| {
                            // Add clip at current playhead position with default 2-second duration
                            let time = current_time();
                            project.write().add_clip_from_asset(asset_id, time, 2.0);
                        },
                        on_drag_start: move |id| dragged_asset.set(Some(id)),
                    }
                }

                // Center
                div {
                    class: "center-area",
                    style: "display: flex; flex-direction: column; flex: 1; overflow: hidden;",

                    PreviewPanel {}

                    // Timeline resize handle
                    div {
                        class: "resize-handle",
                        style: "height: 4px; background-color: {BORDER_DEFAULT}; cursor: ns-resize; flex-shrink: 0;",
                        onmousedown: move |e| {
                            if !timeline_collapsed() {
                                e.prevent_default();
                                dragging.set(Some("timeline"));
                                drag_start_pos.set(e.client_coordinates().y);
                                drag_start_size.set(timeline_height());
                            }
                        },
                    }

                    TimelinePanel {
                        height: timeline_h,
                        collapsed: timeline_collapsed(),
                        is_resizing: timeline_resizing,
                        on_toggle: move |_| timeline_collapsed.set(!timeline_collapsed()),
                        // Project data
                        tracks: project.read().tracks.clone(),
                        clips: project.read().clips.clone(),
                        assets: project.read().assets.clone(),
                        // Timeline state
                        current_time: current_time(),
                        duration: duration,
                        zoom: zoom(),
                        is_playing: is_playing(),
                        scroll_offset: scroll_offset(),
                        // Callbacks
                        on_seek: move |t: f64| {
                            // Snap to frame boundary (60fps) and clamp to duration
                            let snapped = ((t * 60.0).round() / 60.0).clamp(0.0, duration);
                            current_time.set(snapped);
                        },
                        on_zoom_change: move |z: f64| zoom.set(z.clamp(20.0, 500.0)),
                        on_play_pause: move |_| is_playing.set(!is_playing()),
                        on_scroll: move |offset: f64| scroll_offset.set(offset),
                        on_seek_start: move |e: MouseEvent| {
                            dragging.set(Some("playhead"));
                            drag_start_pos.set(e.client_coordinates().x);
                            drag_start_size.set(current_time());
                        },
                        on_seek_end: move |_| dragging.set(None),
                        is_seeking: dragging() == Some("playhead"),
                        // Track management
                        on_add_video_track: move |_| {
                            project.write().add_video_track();
                        },
                        on_add_audio_track: move |_| {
                            project.write().add_audio_track();
                        },
                        on_track_context_menu: move |(x, y, track_id)| {
                            context_menu.set(Some((x, y, track_id)));
                        },
                        // Clip operations
                        on_clip_delete: move |clip_id| {
                            project.write().remove_clip(clip_id);
                        },
                        on_clip_move: move |(clip_id, new_start)| {
                            project.write().move_clip(clip_id, new_start);
                        },
                        on_clip_resize: move |(clip_id, new_start, new_duration)| {
                            project.write().resize_clip(clip_id, new_start, new_duration);
                        },
                        // Asset Drag & Drop
                        dragged_asset: dragged_asset(),
                        on_asset_drop: move |(track_id, time, asset_id)| {
                            let clip = crate::state::Clip::new(asset_id, track_id, time, 2.0);
                            project.write().add_clip(clip);
                        },
                    }
                }

                // Right panel
                SidePanel {
                    title: "Attributes",  
                    width: right_w,
                    collapsed: right_collapsed(),
                    side: "right",
                    is_resizing: right_resizing,
                    on_toggle: move |_| right_collapsed.set(!right_collapsed()),
                    on_resize_start: move |e: MouseEvent| {
                        e.prevent_default();
                        dragging.set(Some("right"));
                        drag_start_pos.set(e.client_coordinates().x);
                        drag_start_size.set(right_width());
                    },
                    
                    // Attributes panel placeholder content
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
                }
            }

            StatusBar {}
            
            // Context menu overlay
            if let Some((x, y, track_id)) = context_menu() {
                // Backdrop to catch clicks outside menu
                div {
                    style: "
                        position: fixed; top: 0; left: 0; right: 0; bottom: 0;
                        z-index: 999;
                    ",
                    onclick: move |_| context_menu.set(None),
                }
                // The actual menu
                div {
                    style: "
                        position: fixed; 
                        left: min({x}px, calc(100vw - 150px)); 
                        top: min({y}px, calc(100vh - 120px));
                        background-color: {BG_ELEVATED}; border: 1px solid {BORDER_DEFAULT};
                        border-radius: 6px; padding: 4px 0; min-width: 140px;
                        box-shadow: 0 4px 12px rgba(0,0,0,0.3);
                        z-index: 1000; font-size: 12px;
                    ",
                    // Check if this is the Markers track (can't delete)
                    {
                        let is_markers = project.read().find_track(track_id)
                            .map(|t| t.track_type == crate::state::TrackType::Marker)
                            .unwrap_or(false);
                        let track_name = project.read().find_track(track_id)
                            .map(|t| t.name.clone())
                            .unwrap_or_default();
                        
                        if is_markers {
                            rsx! {
                                div {
                                    style: "
                                        padding: 6px 12px; color: {TEXT_DIM};
                                        cursor: not-allowed;
                                    ",
                                    "Cannot delete Markers track"
                                }
                            }
                        } else {
                            rsx! {
                                div {
                                    style: "
                                        padding: 6px 12px; color: #ef4444; cursor: pointer;
                                        transition: background-color 0.1s ease;
                                    ",
                                    onmouseenter: move |_| {
                                        // Would need state for hover, skipping for now
                                    },
                                    onclick: move |_| {
                                        project.write().remove_track(track_id);
                                        context_menu.set(None);
                                    },
                                    "🗑 Delete \"{track_name}\""
                                }

                                div {
                                    style: "height: 1px; background-color: {BORDER_SUBTLE}; margin: 2px 0;",
                                }

                                div {
                                    style: "
                                        padding: 6px 12px; color: {TEXT_PRIMARY}; cursor: pointer;
                                        transition: background-color 0.1s ease;
                                    ",
                                    onmouseenter: move |_| {},
                                    onclick: move |_| {
                                        project.write().move_track_up(track_id);
                                        context_menu.set(None);
                                    },
                                    "↑ Move Up"
                                }

                                div {
                                    style: "
                                        padding: 6px 12px; color: {TEXT_PRIMARY}; cursor: pointer;
                                        transition: background-color 0.1s ease;
                                    ",
                                    onmouseenter: move |_| {},
                                    onclick: move |_| {
                                        project.write().move_track_down(track_id);
                                        context_menu.set(None);
                                    },
                                    "↓ Move Down"
                                }
                            }
                        }
                    }
                }
            }

            // Startup Modal (Blocks everything until Project is created/loaded)
            if show_startup {
                StartupModal {
                    on_create: move |(parent_dir, name): (std::path::PathBuf, String)| {
                        // Create full path: parent_dir/name
                        let project_dir = parent_dir.join(&name);
                        match crate::state::Project::create_in(&project_dir, &name) {
                            Ok(new_proj) => {
                                project.set(new_proj);
                                startup_done.set(true);
                            },
                            Err(e) => println!("Error creating project: {}", e),
                        }
                    },
                    on_open: move |path: std::path::PathBuf| {
                         match crate::state::Project::load(&path) { // path is the project folder
                            Ok(loaded_proj) => {
                                project.set(loaded_proj);
                                startup_done.set(true);
                            },
                            Err(e) => println!("Error loading project: {}", e),
                        }
                    }
                }
            }

            // New Project Modal (File > New Project)
            if show_new_project_dialog() {
                // ... reusing the existing modal structure but purely for NEW projects
                // We reuse the startup modal logic essentially, but let's keep the existing simple one for now
                // just adapted to immediate creation
                div {
                    style: "
                        position: fixed; top: 0; left: 0; right: 0; bottom: 0;
                        background-color: rgba(0, 0, 0, 0.5);
                        display: flex; align-items: center; justify-content: center;
                        z-index: 2000;
                    ",
                    onclick: move |_| show_new_project_dialog.set(false),
                    
                    div {
                         style: "
                            width: 400px; background-color: {BG_ELEVATED};
                            border: 1px solid {BORDER_DEFAULT}; border-radius: 8px;
                            padding: 24px; box-shadow: 0 10px 25px rgba(0,0,0,0.5);
                        ",
                         onclick: move |e| e.stop_propagation(),
                         
                         h3 { style: "margin: 0 0 16px 0; font-size: 16px; color: {TEXT_PRIMARY};", "New Project" }
                         // ... (keep existing simple input for now) ...
                         // NOTE: We should probably unify this with StartupModal eventually
                         div {
                            style: "margin-bottom: 20px;",
                             button {
                                style: "width: 100%; padding: 10px; background: {ACCENT_VIDEO}; border: none; border-radius: 4px; color: white; cursor: pointer;",
                                onclick: move |_| {
                                     // Quick hack: just reset to startup modal for now to force the flow
                                    project.set(crate::state::Project::default()); // Reset to untitled
                                    startup_done.set(false); // Trigger startup modal
                                    show_new_project_dialog.set(false);
                                },
                                "Go to Project Wizard"
                             }
                         }
                    }
                }
            }
        }
    }
}

/// Modal shown on app launch
#[component]
fn StartupModal(
    on_create: EventHandler<(std::path::PathBuf, String)>,
    on_open: EventHandler<std::path::PathBuf>
) -> Element {
    let mut name = use_signal(|| "My New Project".to_string());
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
                    width: 600px; display: flex; flex-direction: column; 
                    background-color: {BG_ELEVATED}; border: 1px solid {BORDER_DEFAULT};
                    border-radius: 8px; overflow: hidden; box-shadow: 0 20px 50px rgba(0,0,0,0.5);
                ",
                
                // Header
                div {
                    style: "padding: 24px; border-bottom: 1px solid {BORDER_DEFAULT}; background-color: {BG_SURFACE};",
                    h1 { style: "margin: 0; font-size: 20px; font-weight: 600; color: {TEXT_PRIMARY};", "NLA AI Video Creator" }
                    p { style: "margin: 4px 0 0; font-size: 13px; color: {TEXT_SECONDARY};", "Select an action to get started" }
                }
                
                div {
                    style: "display: flex; height: 320px;",
                    
                    // Left: New Project
                    div {
                        style: "flex: 1; padding: 24px; border-right: 1px solid {BORDER_DEFAULT}; display: flex; flex-direction: column;",
                        h2 { style: "margin: 0 0 16px 0; font-size: 16px; color: {TEXT_PRIMARY};", "Create New Project" },
                        
                        div {
                            style: "flex: 1;",
                            label { style: "display: block; font-size: 12px; color: {TEXT_SECONDARY}; margin-bottom: 6px;", "Project Name" }
                            input {
                                style: "
                                    width: 100%; padding: 8px 12px; background: {BG_BASE};
                                    border: 1px solid {BORDER_DEFAULT}; border-radius: 4px;
                                    color: {TEXT_PRIMARY}; outline: none; margin-bottom: 16px;
                                ",
                                value: "{name}",
                                oninput: move |e| name.set(e.value()),
                            }
                            
                            label { style: "display: block; font-size: 12px; color: {TEXT_SECONDARY}; margin-bottom: 6px;", "Location" }
                            div {
                                style: "display: flex; gap: 8px; margin-bottom: 8px;",
                                input {
                                    style: "
                                        flex: 1; padding: 8px 12px; background: {BG_BASE};
                                        border: 1px solid {BORDER_DEFAULT}; border-radius: 4px;
                                        color: {TEXT_DIM}; outline: none;
                                    ",
                                    disabled: true,
                                    value: "{parent_dir().to_string_lossy()}",
                                }
                                button {
                                    style: "padding: 0 12px; background: {BG_SURFACE}; border: 1px solid {BORDER_DEFAULT}; border-radius: 4px; color: {TEXT_PRIMARY}; cursor: pointer;",
                                    onclick: move |_| {
                                        let start_dir = projects_folder_for_browse.clone();
                                        if let Some(path) = rfd::FileDialog::new()
                                            .set_directory(&start_dir)
                                            .pick_folder() 
                                        {
                                            parent_dir.set(path);
                                        }
                                    },
                                    "Browse..."
                                }
                            }
                            
                            div {
                                style: "font-size: 11px; color: {TEXT_MUTED};",
                                "Project will be created at: "
                                span { style: "color: {TEXT_SECONDARY};", "{parent_dir().join(name()).to_string_lossy()}" }
                            }
                        }
                        
                        button {
                            style: "
                                width: 100%; padding: 10px; margin-top: 16px;
                                background-color: {ACCENT_VIDEO}; border: none; border-radius: 4px;
                                color: white; font-weight: 500; cursor: pointer;
                                transition: opacity 0.2s;
                            ",
                            onclick: move |_| {
                                let n = name();
                                if !n.trim().is_empty() {
                                    on_create.call((parent_dir(), n));
                                }
                            },
                            "Create Project"
                        }
                    }
                    
                    // Right: Open Project
                    div {
                        style: "flex: 1; padding: 24px; display: flex; flex-direction: column; background-color: {BG_BASE}; min-width: 0; overflow: hidden;",
                        h2 { style: "margin: 0 0 12px 0; font-size: 16px; color: {TEXT_PRIMARY};", "Open Existing" },
                        
                        // Project list or empty state
                        if existing_projects.is_empty() {
                            div {
                                style: "flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; color: {TEXT_DIM}; gap: 12px;",
                                div { style: "font-size: 32px; opacity: 0.5;", "📂" }
                                p { style: "margin: 0; font-size: 13px; text-align: center;", "No projects found" }
                            }
                        } else {
                            div {
                                style: "
                                    flex: 1; overflow-y: auto; overflow-x: hidden;
                                    border: 1px solid {BORDER_SUBTLE}; border-radius: 6px;
                                    background-color: {BG_SURFACE};
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
                                                key: "{proj_path.display()}",
                                                style: "
                                                    padding: 8px 10px; cursor: pointer;
                                                    border-bottom: 1px solid {BORDER_SUBTLE};
                                                    transition: background-color 0.15s ease;
                                                ",
                                                onmouseenter: move |_| {},
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
                                                    style: "display: flex; align-items: center; gap: 8px; min-width: 0;",
                                                    span { style: "font-size: 14px; flex-shrink: 0;", "📁" }
                                                    div {
                                                        style: "flex: 1; min-width: 0; overflow: hidden;",
                                                        div { 
                                                            style: "font-size: 12px; font-weight: 500; color: {TEXT_PRIMARY}; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;", 
                                                            "{proj_name}" 
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        
                        button {
                            style: "
                                width: 100%; padding: 10px; margin-top: 12px; flex-shrink: 0;
                                background-color: {BG_SURFACE}; border: 1px solid {BORDER_DEFAULT};
                                border-radius: 4px; color: {TEXT_PRIMARY}; font-weight: 500; cursor: pointer;
                                transition: background 0.2s;
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
                        border-radius: 6px; padding: 4px 0; min-width: 140px;
                        box-shadow: 0 4px 12px rgba(0,0,0,0.3);
                        z-index: 10001; font-size: 12px;
                    ",
                    div {
                        style: "
                            padding: 6px 12px; color: #ef4444; cursor: pointer;
                            transition: background-color 0.1s ease;
                        ",
                        onmouseenter: move |_| {},
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
                        "🗑 Delete \"{proj_name}\""
                    }
                }
            }
        }
    }
}

#[component]

fn TitleBar(project_name: String, on_new_project: EventHandler<MouseEvent>) -> Element {
    rsx! {
        div {
            style: "
                display: flex; align-items: center; justify-content: space-between;
                height: 40px; padding: 0 16px;
                background-color: {BG_SURFACE}; border-bottom: 1px solid {BORDER_DEFAULT};
                user-select: none;
            ",
            div {
                style: "display: flex; align-items: center; gap: 20px;",
                span { style: "font-size: 13px; font-weight: 600; color: {TEXT_SECONDARY};", "NLA AI Video Creator" }
                
                // Simple File > New Project button for now (can expand to full menu later)
                button {
                    class: "collapse-btn", // Reusing hover style
                    style: "
                        background: transparent; border: none; color: {TEXT_PRIMARY}; 
                        font-size: 12px; cursor: pointer; padding: 4px 8px; border-radius: 4px;
                    ",
                    onclick: move |e| on_new_project.call(e),
                    "New Project"
                }
            }
            
            span { style: "font-size: 13px; color: {TEXT_MUTED};", "{project_name}" }
            
            // Right side spacer (balance)
            div { style: "width: 150px;" } 
        }
    }
}

#[component]
fn SidePanel(
    title: &'static str,
    width: f64,
    collapsed: bool,
    side: &'static str,
    is_resizing: bool,
    on_toggle: EventHandler<MouseEvent>,
    on_resize_start: EventHandler<MouseEvent>,
    children: Element,  // Custom content for this panel
) -> Element {
    let is_left = side == "left";
    // Arrow points toward the resize edge when expanded, away when collapsed
    let icon = if collapsed { 
        if is_left { "▶" } else { "◀" } 
    } else { 
        if is_left { "◀" } else { "▶" } 
    };
    let border = if is_left { 
        format!("border-right: 1px solid {BORDER_DEFAULT};") 
    } else { 
        format!("border-left: 1px solid {BORDER_DEFAULT};") 
    };
    
    // Only apply transition when NOT resizing
    let transition = if is_resizing { "none" } else { "width 0.2s ease, min-width 0.2s ease" };
    
    // Cursor for collapsed state (entire rail is clickable)
    let rail_cursor = if collapsed { "pointer" } else { "default" };

    // Class for hover effect when collapsed
    let panel_class = if collapsed { "collapsed-rail" } else { "" };

    rsx! {
        div {
            class: "{panel_class}",
            style: "
                display: flex; flex-direction: row;
                width: {width}px; min-width: {width}px;
                background-color: {BG_ELEVATED}; {border}
                transition: {transition};
                overflow: hidden;
                cursor: {rail_cursor};
            ",
            // Make entire collapsed panel clickable
            onclick: move |e| {
                if collapsed {
                    on_toggle.call(e);
                }
            },

            // Resize handle (left edge for right panel)
            if !collapsed && !is_left {
                div {
                    class: "resize-handle",
                    style: "width: 4px; height: 100%; background-color: {BORDER_DEFAULT}; cursor: ew-resize; flex-shrink: 0;",
                    onmousedown: move |e| {
                        e.prevent_default();
                        e.stop_propagation();
                        on_resize_start.call(e);
                    },
                }
            }

            div {
                style: "display: flex; flex-direction: column; flex: 1; overflow: hidden;",

                // Header - different layout when collapsed vs expanded
                if collapsed {
                    // Collapsed: just the arrow button centered
                    div {
                        style: "
                            display: flex; align-items: center; justify-content: center;
                            height: 32px;
                            background-color: {BG_SURFACE}; border-bottom: 1px solid {BORDER_DEFAULT};
                            flex-shrink: 0;
                        ",
                        button {
                            class: "collapse-btn",
                            style: "
                                width: 24px; height: 24px; border: none; border-radius: 4px;
                                background: transparent; color: {TEXT_MUTED}; font-size: 10px;
                                cursor: pointer; display: flex; align-items: center; justify-content: center;
                            ",
                            onclick: move |e| {
                                e.stop_propagation();
                                on_toggle.call(e);
                            },
                            "{icon}"
                        }
                    }
                } else {
                    // Expanded: title on left, arrow on the side nearest resize bar
                    div {
                        style: "
                            display: flex; align-items: center;
                            height: 32px; padding: 0 8px;
                            background-color: {BG_SURFACE}; border-bottom: 1px solid {BORDER_DEFAULT};
                            flex-shrink: 0;
                        ",
                        // For left panel: [title ... arrow] (arrow near right edge/resize bar)
                        // For right panel: [arrow ... title] (arrow near left edge/resize bar)
                        if !is_left {
                            button {
                                class: "collapse-btn",
                                style: "
                                    width: 24px; height: 24px; border: none; border-radius: 4px;
                                    background: transparent; color: {TEXT_MUTED}; font-size: 10px;
                                    cursor: pointer; display: flex; align-items: center; justify-content: center;
                                    margin-right: 8px;
                                ",
                                onclick: move |e| on_toggle.call(e),
                                "{icon}"
                            }
                        }
                        
                        span { 
                            style: "font-size: 11px; font-weight: 500; color: {TEXT_MUTED}; text-transform: uppercase; letter-spacing: 0.5px; flex: 1;", 
                            "{title}" 
                        }
                        
                        if is_left {
                            button {
                                class: "collapse-btn",
                                style: "
                                    width: 24px; height: 24px; border: none; border-radius: 4px;
                                    background: transparent; color: {TEXT_MUTED}; font-size: 10px;
                                    cursor: pointer; display: flex; align-items: center; justify-content: center;
                                ",
                                onclick: move |e| on_toggle.call(e),
                                "{icon}"
                            }
                        }
                    }
                }

                // Content (only when expanded)
                if !collapsed {
                    div {
                        style: "flex: 1; overflow-y: auto;",
                        {children}
                    }
                }
            }

            // Resize handle (right edge for left panel)
            if !collapsed && is_left {
                div {
                    class: "resize-handle",
                    style: "width: 4px; height: 100%; background-color: {BORDER_DEFAULT}; cursor: ew-resize; flex-shrink: 0;",
                    onmousedown: move |e| {
                        e.prevent_default();
                        e.stop_propagation();
                        on_resize_start.call(e);
                    },
                }
            }
        }
    }
}

#[component]
fn PreviewPanel() -> Element {
    rsx! {
        div {
            style: "display: flex; flex-direction: column; flex: 1; min-height: 200px; background-color: {BG_DEEPEST};",

            div {
                style: "
                    display: flex; align-items: center; justify-content: space-between;
                    height: 32px; padding: 0 14px;
                    background-color: {BG_SURFACE}; border-bottom: 1px solid {BORDER_DEFAULT};
                ",
                span { style: "font-size: 11px; font-weight: 500; color: {TEXT_MUTED}; text-transform: uppercase; letter-spacing: 0.5px;", "Preview" }
                div {
                    style: "display: flex; align-items: center; gap: 6px; font-family: 'SF Mono', Consolas, monospace; font-size: 11px; color: {TEXT_DIM};",
                    span { "1920 × 1080" }
                    span { style: "color: {TEXT_MUTED};", "@" }
                    span { "60" }
                }
            }

            div {
                style: "flex: 1; display: flex; align-items: center; justify-content: center; background-color: {BG_DEEPEST};",
                div {
                    style: "display: flex; flex-direction: column; align-items: center; gap: 12px; color: {TEXT_DIM};",
                    div {
                        style: "width: 48px; height: 48px; border: 1px solid {BORDER_DEFAULT}; border-radius: 50%; display: flex; align-items: center; justify-content: center; font-size: 14px;",
                        "▶"
                    }
                    span { style: "font-size: 12px;", "No preview" }
                }
            }
        }
    }
}

// TimelinePanel, PlaybackBtn, TrackLabel, TrackRow moved to src/timeline.rs

#[component]
fn StatusBar() -> Element {
    rsx! {
        div {
            style: "display: flex; align-items: center; justify-content: space-between; height: 22px; padding: 0 14px; background-color: {BG_SURFACE}; border-top: 1px solid {BORDER_DEFAULT}; font-size: 11px; color: {TEXT_DIM};",
            span { "Ready" }
            div {
                style: "display: flex; gap: 16px; font-family: 'SF Mono', Consolas, monospace;",
                span { "60 fps" }
                span { "00:00 / 00:00" }
            }
        }
    }
}

/// Assets panel content - displays project assets and import functionality
#[component]
fn AssetsPanelContent(
    assets: Vec<crate::state::Asset>,
    on_import: EventHandler<crate::state::Asset>,
    on_import_file: EventHandler<std::path::PathBuf>,
    on_delete: EventHandler<uuid::Uuid>,
    on_add_to_timeline: EventHandler<uuid::Uuid>,
    on_drag_start: EventHandler<uuid::Uuid>,
) -> Element {
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
                                // Generate unique ID for this generative asset
                                let id = uuid::Uuid::new_v4();
                                let folder = std::path::PathBuf::from(format!("generated/video/{}", id));
                                let asset = crate::state::Asset::new_generative_video(
                                    format!("Gen Video {}", &id.to_string()[..8]),
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
                                    format!("Gen Image {}", &id.to_string()[..8]),
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
                                    format!("Gen Audio {}", &id.to_string()[..8]),
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
                            on_delete: move |id| on_delete.call(id),
                            on_add_to_timeline: move |id| on_add_to_timeline.call(id),
                            on_drag_start: move |id| on_drag_start.call(id),
                        }
                    }
                }
            }
        }
    }
}

/// Individual asset item in the list
#[component]
fn AssetItem(
    asset: crate::state::Asset,
    on_delete: EventHandler<uuid::Uuid>,
    on_add_to_timeline: EventHandler<uuid::Uuid>,
    on_drag_start: EventHandler<uuid::Uuid>,
) -> Element {
    let mut show_menu = use_signal(|| false);
    let mut menu_pos = use_signal(|| (0.0, 0.0));

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
    
    // Generative assets have a subtle dashed border
    let border_style = if asset.is_generative() {
        format!("1px dashed {}", BORDER_DEFAULT)  // Subtle dashed, not accent-colored
    } else {
        format!("1px solid {}", BORDER_SUBTLE)
    };

    let asset_id = asset.id;
    let asset_name = asset.name.clone();
    
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
                // Icon
                span { style: "font-size: 14px;", "{icon}" }
                // Name
                span { 
                    style: "flex: 1; font-size: 12px; color: {TEXT_PRIMARY}; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                    "{asset_name}" 
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
                                left: min({x}px, calc(100vw - 140px)); 
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
