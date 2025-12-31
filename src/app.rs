//! Root application component
//! 
//! This defines the main App component and the overall layout structure.

use dioxus::prelude::*;

// =============================================================================
// COLOR SCHEME - Charcoal Monochrome with Functional Accents
// =============================================================================
// Backgrounds (darkest to lightest)
const BG_DEEPEST: &str = "#09090b";      // Near black - preview viewport
const BG_BASE: &str = "#0a0a0b";          // Base background
const BG_ELEVATED: &str = "#141414";      // Panels, timeline
const BG_SURFACE: &str = "#1a1a1a";       // Headers, raised elements
const BG_HOVER: &str = "#262626";         // Hover states

// Borders
const BORDER_SUBTLE: &str = "#1f1f1f";    // Very subtle dividers
const BORDER_DEFAULT: &str = "#27272a";   // Normal borders
const BORDER_STRONG: &str = "#3f3f46";    // Emphasized borders

// Text
const TEXT_PRIMARY: &str = "#fafafa";     // Primary text
const TEXT_SECONDARY: &str = "#a1a1aa";   // Secondary text
const TEXT_MUTED: &str = "#71717a";       // Muted/disabled text
const TEXT_DIM: &str = "#52525b";         // Very dim text

// Accent colors (functional only)
const ACCENT_AUDIO: &str = "#3b82f6";     // Blue - audio tracks
const ACCENT_MARKER: &str = "#f97316";    // Orange - markers
const ACCENT_KEYFRAME: &str = "#a855f7";  // Purple - keyframes
const ACCENT_VIDEO: &str = "#22c55e";     // Green - video tracks

/// Main application component - the root of our UI tree
#[component]
pub fn App() -> Element {
    rsx! {
        // Global CSS reset and scrollbar hiding
        style {
            r#"
            *, *::before, *::after {{
                box-sizing: border-box;
            }}
            html, body {{
                margin: 0;
                padding: 0;
                overflow: hidden;
                background-color: {BG_BASE};
            }}
            body {{
                -webkit-font-smoothing: antialiased;
                -moz-osx-font-smoothing: grayscale;
            }}
            /* Hide scrollbars but keep functionality */
            ::-webkit-scrollbar {{
                width: 6px;
                height: 6px;
            }}
            ::-webkit-scrollbar-track {{
                background: transparent;
            }}
            ::-webkit-scrollbar-thumb {{
                background: {BORDER_DEFAULT};
                border-radius: 3px;
            }}
            ::-webkit-scrollbar-thumb:hover {{
                background: {BORDER_STRONG};
            }}
            "#
        }

        // Main app container - fills the window
        div {
            class: "app-container",
            style: "
                display: flex;
                flex-direction: column;
                width: 100vw;
                height: 100vh;
                background-color: {BG_BASE};
                color: {TEXT_PRIMARY};
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
                overflow: hidden;
                position: fixed;
                top: 0;
                left: 0;
            ",

            // Title bar
            TitleBar {}

            // Main content area (panels + timeline)
            div {
                class: "main-content",
                style: "
                    display: flex;
                    flex: 1;
                    overflow: hidden;
                ",

                // Left panel (Asset Browser)
                SidePanel { 
                    title: "Assets",
                    width: "250px",
                }

                // Center area (Preview + Timeline)
                div {
                    class: "center-area",
                    style: "
                        display: flex;
                        flex-direction: column;
                        flex: 1;
                        overflow: hidden;
                    ",

                    // Preview window
                    PreviewPanel {}

                    // Timeline
                    TimelinePanel {}
                }

                // Right panel (Attribute Editor)
                SidePanel {
                    title: "Attributes",
                    width: "280px",
                }
            }

            // Status bar
            StatusBar {}
        }
    }
}

/// Title bar component
#[component]
fn TitleBar() -> Element {
    rsx! {
        div {
            class: "title-bar",
            style: "
                display: flex;
                align-items: center;
                justify-content: space-between;
                height: 40px;
                padding: 0 16px;
                background-color: {BG_SURFACE};
                border-bottom: 1px solid {BORDER_DEFAULT};
                user-select: none;
            ",

            // Left side - Logo/App name
            div {
                style: "
                    display: flex;
                    align-items: center;
                    gap: 12px;
                ",
                span {
                    style: "
                        font-size: 13px;
                        font-weight: 600;
                        color: {TEXT_SECONDARY};
                        letter-spacing: 0.3px;
                    ",
                    "NLA AI Video Creator"
                }
            }

            // Center - Project name (placeholder)
            div {
                style: "
                    font-size: 13px;
                    color: {TEXT_MUTED};
                ",
                "Untitled Project"
            }

            // Right side - placeholder for window controls or menu
            div {
                style: "
                    display: flex;
                    gap: 8px;
                ",
                // We can add menu items here later
            }
        }
    }
}

/// Reusable side panel component
#[component]
fn SidePanel(title: &'static str, width: &'static str) -> Element {
    rsx! {
        div {
            class: "side-panel",
            style: "
                display: flex;
                flex-direction: column;
                width: {width};
                min-width: {width};
                background-color: {BG_ELEVATED};
                border-left: 1px solid {BORDER_DEFAULT};
                border-right: 1px solid {BORDER_DEFAULT};
            ",

            // Panel header
            div {
                class: "panel-header",
                style: "
                    display: flex;
                    align-items: center;
                    height: 32px;
                    padding: 0 14px;
                    background-color: {BG_SURFACE};
                    border-bottom: 1px solid {BORDER_DEFAULT};
                    font-size: 11px;
                    font-weight: 500;
                    color: {TEXT_MUTED};
                    text-transform: uppercase;
                    letter-spacing: 0.5px;
                ",
                "{title}"
            }

            // Panel content (placeholder)
            div {
                class: "panel-content",
                style: "
                    flex: 1;
                    padding: 12px;
                    overflow-y: auto;
                ",
                
                // Placeholder content
                div {
                    style: "
                        display: flex;
                        align-items: center;
                        justify-content: center;
                        height: 80px;
                        border: 1px dashed {BORDER_DEFAULT};
                        border-radius: 6px;
                        color: {TEXT_DIM};
                        font-size: 12px;
                    ",
                    "{title}"
                }
            }
        }
    }
}

/// Preview panel component
#[component]
fn PreviewPanel() -> Element {
    rsx! {
        div {
            class: "preview-panel",
            style: "
                display: flex;
                flex-direction: column;
                flex: 1;
                min-height: 300px;
                background-color: {BG_DEEPEST};
                border-bottom: 1px solid {BORDER_DEFAULT};
            ",

            // Preview header - matches side panel header height
            div {
                class: "preview-header",
                style: "
                    display: flex;
                    align-items: center;
                    justify-content: space-between;
                    height: 32px;
                    padding: 0 14px;
                    background-color: {BG_SURFACE};
                    border-bottom: 1px solid {BORDER_DEFAULT};
                ",

                span {
                    style: "
                        font-size: 11px;
                        font-weight: 500;
                        color: {TEXT_MUTED};
                        text-transform: uppercase;
                        letter-spacing: 0.5px;
                    ",
                    "Preview"
                }

                // Preview info - resolution @ fps
                div {
                    style: "
                        display: flex;
                        align-items: center;
                        gap: 6px;
                        font-family: 'SF Mono', 'Consolas', 'Monaco', monospace;
                        font-size: 11px;
                        color: {TEXT_DIM};
                    ",
                    span { "1920 × 1080" }
                    span { 
                        style: "color: {TEXT_MUTED};",
                        "@" 
                    }
                    span { "60" }
                }
            }

            // Preview viewport
            div {
                class: "preview-viewport",
                style: "
                    flex: 1;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    background-color: {BG_DEEPEST};
                ",

                // Placeholder
                div {
                    style: "
                        display: flex;
                        flex-direction: column;
                        align-items: center;
                        gap: 12px;
                        color: {TEXT_DIM};
                    ",
                    
                    // Play icon placeholder
                    div {
                        style: "
                            width: 48px;
                            height: 48px;
                            border: 1px solid {BORDER_DEFAULT};
                            border-radius: 50%;
                            display: flex;
                            align-items: center;
                            justify-content: center;
                            font-size: 14px;
                        ",
                        "▶"
                    }
                    
                    span {
                        style: "font-size: 12px;",
                        "No preview"
                    }
                }
            }
        }
    }
}

/// Timeline panel component
#[component]
fn TimelinePanel() -> Element {
    rsx! {
        div {
            class: "timeline-panel",
            style: "
                display: flex;
                flex-direction: column;
                height: 220px;
                min-height: 150px;
                background-color: {BG_ELEVATED};
                border-top: 1px solid {BORDER_DEFAULT};
            ",

            // Timeline header with controls
            div {
                class: "timeline-header",
                style: "
                    display: flex;
                    align-items: center;
                    justify-content: space-between;
                    height: 32px;
                    padding: 0 14px;
                    background-color: {BG_SURFACE};
                    border-bottom: 1px solid {BORDER_DEFAULT};
                ",

                // Left - Timeline label
                span {
                    style: "
                        font-size: 11px;
                        font-weight: 500;
                        color: {TEXT_MUTED};
                        text-transform: uppercase;
                        letter-spacing: 0.5px;
                    ",
                    "Timeline"
                }

                // Center - Playback controls
                div {
                    class: "playback-controls",
                    style: "
                        display: flex;
                        align-items: center;
                        gap: 4px;
                    ",

                    PlaybackButton { icon: "⏮", label: "Start" }
                    PlaybackButton { icon: "◀", label: "Back" }
                    PlaybackButton { icon: "▶", label: "Play", primary: true }
                    PlaybackButton { icon: "▶", label: "Forward" }
                    PlaybackButton { icon: "⏭", label: "End" }
                }

                // Right - Time display
                div {
                    style: "
                        font-family: 'SF Mono', 'Consolas', 'Monaco', monospace;
                        font-size: 11px;
                        color: {TEXT_DIM};
                        letter-spacing: 0.5px;
                    ",
                    "00:00:00:00"
                }
            }

            // Timeline tracks area
            div {
                class: "timeline-tracks",
                style: "
                    flex: 1;
                    display: flex;
                    overflow: hidden;
                ",

                // Track labels column
                div {
                    class: "track-labels",
                    style: "
                        width: 140px;
                        min-width: 140px;
                        background-color: {BG_ELEVATED};
                        border-right: 1px solid {BORDER_DEFAULT};
                    ",

                    TrackLabel { name: "Audio", color: ACCENT_AUDIO }
                    TrackLabel { name: "Markers", color: ACCENT_MARKER }
                    TrackLabel { name: "Keyframes", color: ACCENT_KEYFRAME }
                    TrackLabel { name: "Video 1", color: ACCENT_VIDEO }
                }

                // Timeline content area
                div {
                    class: "timeline-content",
                    style: "
                        flex: 1;
                        display: flex;
                        flex-direction: column;
                        background-color: {BG_BASE};
                        overflow-x: auto;
                    ",

                    // Placeholder tracks
                    TimelineTrack {}
                    TimelineTrack {}
                    TimelineTrack {}
                    TimelineTrack {}
                }
            }
        }
    }
}

/// Playback control button
#[component]
fn PlaybackButton(icon: &'static str, label: &'static str, #[props(default = false)] primary: bool) -> Element {
    let bg = if primary { BG_HOVER } else { "transparent" };
    
    rsx! {
        button {
            class: "playback-btn",
            title: "{label}",
            style: "
                width: 26px;
                height: 26px;
                border: none;
                border-radius: 4px;
                background-color: {bg};
                color: {TEXT_MUTED};
                font-size: 10px;
                cursor: pointer;
                display: flex;
                align-items: center;
                justify-content: center;
                transition: all 0.12s ease;
            ",
            "{icon}"
        }
    }
}

/// Track label in the timeline
#[component]
fn TrackLabel(name: &'static str, color: &'static str) -> Element {
    rsx! {
        div {
            class: "track-label",
            style: "
                display: flex;
                align-items: center;
                gap: 10px;
                height: 36px;
                padding: 0 12px;
                border-bottom: 1px solid {BORDER_SUBTLE};
                font-size: 12px;
                color: {TEXT_SECONDARY};
                cursor: default;
            ",

            // Color indicator
            div {
                style: "
                    width: 3px;
                    height: 16px;
                    border-radius: 2px;
                    background-color: {color};
                ",
            }

            span { "{name}" }
        }
    }
}

/// Timeline track row
#[component]
fn TimelineTrack() -> Element {
    rsx! {
        div {
            class: "timeline-track",
            style: "
                height: 36px;
                border-bottom: 1px solid {BORDER_SUBTLE};
                background-color: {BG_BASE};
            ",
            // Track content will go here
        }
    }
}

/// Status bar at the bottom
#[component]
fn StatusBar() -> Element {
    rsx! {
        div {
            class: "status-bar",
            style: "
                display: flex;
                align-items: center;
                justify-content: space-between;
                height: 22px;
                padding: 0 14px;
                background-color: {BG_SURFACE};
                border-top: 1px solid {BORDER_DEFAULT};
                font-size: 11px;
                color: {TEXT_DIM};
            ",

            // Left side - status message
            span {
                "Ready"
            }

            // Right side - info
            div {
                style: "
                    display: flex;
                    gap: 16px;
                    font-family: 'SF Mono', 'Consolas', 'Monaco', monospace;
                ",
                span { "60 fps" }
                span { "00:00 / 00:00" }
            }
        }
    }
}
