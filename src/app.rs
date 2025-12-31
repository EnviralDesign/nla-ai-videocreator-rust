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
pub const ACCENT_KEYFRAME: &str = "#a855f7";
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
    // Panel state
    let mut left_width = use_signal(|| PANEL_DEFAULT_WIDTH);
    let mut left_collapsed = use_signal(|| false);
    let mut right_width = use_signal(|| PANEL_DEFAULT_WIDTH);
    let mut right_collapsed = use_signal(|| false);
    let mut timeline_height = use_signal(|| TIMELINE_DEFAULT_HEIGHT);
    let mut timeline_collapsed = use_signal(|| false);
    
    // Timeline playback state
    let mut current_time = use_signal(|| 0.0_f64);        // Current time in seconds
    let mut duration = use_signal(|| 60.0_f64);           // Total duration in seconds
    let mut zoom = use_signal(|| 100.0_f64);              // Pixels per second
    let mut is_playing = use_signal(|| false);            // Playback state
    let mut scroll_offset = use_signal(|| 0.0_f64);       // Horizontal scroll position
    
    // Drag state
    let mut dragging = use_signal(|| None::<&'static str>);
    let mut drag_start_pos = use_signal(|| 0.0);
    let mut drag_start_size = use_signal(|| 0.0);

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
                            let new_time = (drag_start_size() + delta_time).clamp(0.0, duration());
                            current_time.set(new_time);
                        }
                        _ => {}
                    }
                }
            },
            onmouseup: move |_| dragging.set(None),
            // Note: We intentionally don't clear drag on mouseleave so drag continues
            // if the user moves outside the window and back in while still holding mouse button

            TitleBar {}

            // Main content
            div {
                class: "main-content",
                style: "display: flex; flex: 1; overflow: hidden;",

                // Left panel
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
                        // Timeline state
                        current_time: current_time(),
                        duration: duration(),
                        zoom: zoom(),
                        is_playing: is_playing(),
                        scroll_offset: scroll_offset(),
                        // Callbacks
                        on_seek: move |t: f64| current_time.set(t.clamp(0.0, duration())),
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
                }
            }

            StatusBar {}
        }
    }
}

#[component]
fn TitleBar() -> Element {
    rsx! {
        div {
            style: "
                display: flex; align-items: center; justify-content: space-between;
                height: 40px; padding: 0 16px;
                background-color: {BG_SURFACE}; border-bottom: 1px solid {BORDER_DEFAULT};
                user-select: none;
            ",
            span { style: "font-size: 13px; font-weight: 600; color: {TEXT_SECONDARY};", "NLA AI Video Creator" }
            span { style: "font-size: 13px; color: {TEXT_MUTED};", "Untitled Project" }
            div {}
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
                        style: "flex: 1; padding: 12px; overflow-y: auto;",
                        div {
                            style: "
                                display: flex; align-items: center; justify-content: center;
                                height: 80px; border: 1px dashed {BORDER_DEFAULT}; border-radius: 6px;
                                color: {TEXT_DIM}; font-size: 12px;
                            ",
                            "{title}"
                        }
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
