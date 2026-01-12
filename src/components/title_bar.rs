use dioxus::prelude::*;
use crate::constants::*;

/// Menu item with label and optional hotkey hint
#[derive(Clone, PartialEq)]
pub struct MenuItem {
    pub label: String,
    pub hotkey: Option<String>,
    pub enabled: bool,
    pub checked: Option<bool>,
}

impl MenuItem {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            hotkey: None,
            enabled: true,
            checked: None,
        }
    }

    pub fn with_hotkey(mut self, hotkey: &str) -> Self {
        self.hotkey = Some(hotkey.to_string());
        self
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = Some(checked);
        self
    }
}

#[component]
pub fn TitleBar(
    project_name: String,
    on_new_project: EventHandler<MouseEvent>,
    on_save: EventHandler<MouseEvent>,
    on_project_settings: EventHandler<MouseEvent>,
    on_open_providers: EventHandler<MouseEvent>,
    show_preview_stats: bool,
    on_toggle_preview_stats: EventHandler<MouseEvent>,
    use_hw_decode: bool,
    on_toggle_hw_decode: EventHandler<MouseEvent>,
    queue_count: usize,
    queue_open: bool,
    queue_running: bool,
    queue_paused: bool,
    project_loaded: bool,
    on_toggle_queue: EventHandler<MouseEvent>,
    on_menu_open: EventHandler<bool>,
) -> Element {
    // Track which menu is currently open (None = all closed)
    let mut active_menu = use_signal(|| None::<String>);
    let project_settings_item = if project_loaded {
        MenuItem::new("Project Settings...")
    } else {
        MenuItem::new("Project Settings...").disabled()
    };

    // Close menu on any click outside
    let close_menus = move |_: MouseEvent| {
        active_menu.set(None);
        on_menu_open.call(false);
    };

    rsx! {
        div {
            style: "
                display: flex; align-items: center; justify-content: space-between;
                height: 32px; padding: 0 8px;
                background-color: {BG_SURFACE}; border-bottom: 1px solid {BORDER_DEFAULT};
                user-select: none; font-size: 13px;
            ",

            // Left side: Menu bar
            div {
                style: "display: flex; align-items: center; gap: 0;",

                // File Menu
                MenuButton {
                    label: "File",
                    is_open: active_menu() == Some("file".to_string()),
                    on_toggle: move |_| {
                        if active_menu() == Some("file".to_string()) {
                            active_menu.set(None); on_menu_open.call(false);
                        } else {
                            active_menu.set(Some("file".to_string())); on_menu_open.call(true);
                        }
                    },
                    on_hover: move |_| {
                        if active_menu().is_some() {
                            active_menu.set(Some("file".to_string())); on_menu_open.call(true);
                        }
                    },
                    MenuDropdown {
                        MenuItemButton {
                            item: MenuItem::new("New Project...").with_hotkey("Ctrl+N"),
                            on_click: move |e| {
                                active_menu.set(None); on_menu_open.call(false);
                                on_new_project.call(e);
                            },
                        }
                        MenuItemButton {
                            item: MenuItem::new("Open Project...").with_hotkey("Ctrl+O").disabled(),
                            on_click: move |_| {},
                        }
                        MenuItemButton {
                            item: project_settings_item.clone(),
                            on_click: move |e| {
                                active_menu.set(None); on_menu_open.call(false);
                                on_project_settings.call(e);
                            },
                        }
                        MenuDivider {}
                        MenuItemButton {
                            item: MenuItem::new("Save").with_hotkey("Ctrl+S"),
                            on_click: move |e| {
                                active_menu.set(None); on_menu_open.call(false);
                                on_save.call(e);
                            },
                        }
                        MenuItemButton {
                            item: MenuItem::new("Save As...").with_hotkey("Ctrl+Shift+S").disabled(),
                            on_click: move |_| {},
                        }
                        MenuDivider {}
                        MenuItemButton {
                            item: MenuItem::new("Exit").with_hotkey("Alt+F4").disabled(),
                            on_click: move |_| {},
                        }
                    }
                }

                // Edit Menu
                MenuButton {
                    label: "Edit",
                    is_open: active_menu() == Some("edit".to_string()),
                    on_toggle: move |_| {
                        if active_menu() == Some("edit".to_string()) {
                            active_menu.set(None); on_menu_open.call(false);
                        } else {
                            active_menu.set(Some("edit".to_string())); on_menu_open.call(true);
                        }
                    },
                    on_hover: move |_| {
                        if active_menu().is_some() {
                            active_menu.set(Some("edit".to_string())); on_menu_open.call(true);
                        }
                    },
                    MenuDropdown {
                        MenuItemButton {
                            item: MenuItem::new("Undo").with_hotkey("Ctrl+Z").disabled(),
                            on_click: move |_| {},
                        }
                        MenuItemButton {
                            item: MenuItem::new("Redo").with_hotkey("Ctrl+Y").disabled(),
                            on_click: move |_| {},
                        }
                        MenuDivider {}
                        MenuItemButton {
                            item: MenuItem::new("Cut").with_hotkey("Ctrl+X").disabled(),
                            on_click: move |_| {},
                        }
                        MenuItemButton {
                            item: MenuItem::new("Copy").with_hotkey("Ctrl+C").disabled(),
                            on_click: move |_| {},
                        }
                        MenuItemButton {
                            item: MenuItem::new("Paste").with_hotkey("Ctrl+V").disabled(),
                            on_click: move |_| {},
                        }
                        MenuDivider {}
                        MenuItemButton {
                            item: MenuItem::new("Delete").with_hotkey("Del").disabled(),
                            on_click: move |_| {},
                        }
                        MenuItemButton {
                            item: MenuItem::new("Select All").with_hotkey("Ctrl+A").disabled(),
                            on_click: move |_| {},
                        }
                    }
                }

                // View Menu
                MenuButton {
                    label: "View",
                    is_open: active_menu() == Some("view".to_string()),
                    on_toggle: move |_| {
                        if active_menu() == Some("view".to_string()) {
                            active_menu.set(None); on_menu_open.call(false);
                        } else {
                            active_menu.set(Some("view".to_string())); on_menu_open.call(true);
                        }
                    },
                    on_hover: move |_| {
                        if active_menu().is_some() {
                            active_menu.set(Some("view".to_string())); on_menu_open.call(true);
                        }
                    },
                    MenuDropdown {
                        MenuItemButton {
                            item: MenuItem::new("Preview Statistics").checked(show_preview_stats),
                            on_click: move |e| {
                                active_menu.set(None); on_menu_open.call(false);
                                on_toggle_preview_stats.call(e);
                            },
                        }
                        MenuDivider {}
                        MenuItemButton {
                            item: MenuItem::new("Zoom In").with_hotkey("Num +").disabled(),
                            on_click: move |_| {},
                        }
                        MenuItemButton {
                            item: MenuItem::new("Zoom Out").with_hotkey("Num -").disabled(),
                            on_click: move |_| {},
                        }
                        MenuItemButton {
                            item: MenuItem::new("Zoom to Fit").with_hotkey("Ctrl+0").disabled(),
                            on_click: move |_| {},
                        }
                    }
                }

                // Settings Menu
                MenuButton {
                    label: "Settings",
                    is_open: active_menu() == Some("settings".to_string()),
                    on_toggle: move |_| {
                        if active_menu() == Some("settings".to_string()) {
                            active_menu.set(None); on_menu_open.call(false);
                        } else {
                            active_menu.set(Some("settings".to_string())); on_menu_open.call(true);
                        }
                    },
                    on_hover: move |_| {
                        if active_menu().is_some() {
                            active_menu.set(Some("settings".to_string())); on_menu_open.call(true);
                        }
                    },
                    MenuDropdown {
                        MenuItemButton {
                            item: MenuItem::new("AI Providers..."),
                            on_click: move |e| {
                                active_menu.set(None); on_menu_open.call(false);
                                on_open_providers.call(e);
                            },
                        }
                        MenuDivider {}
                        MenuItemButton {
                            item: MenuItem::new("Hardware Decoding").checked(use_hw_decode),
                            on_click: move |e| {
                                active_menu.set(None); on_menu_open.call(false);
                                on_toggle_hw_decode.call(e);
                            },
                        }
                        MenuDivider {}
                        MenuItemButton {
                            item: MenuItem::new("Preferences...").disabled(),
                            on_click: move |_| {},
                        }
                    }
                }

                // Help Menu
                MenuButton {
                    label: "Help",
                    is_open: active_menu() == Some("help".to_string()),
                    on_toggle: move |_| {
                        if active_menu() == Some("help".to_string()) {
                            active_menu.set(None); on_menu_open.call(false);
                        } else {
                            active_menu.set(Some("help".to_string())); on_menu_open.call(true);
                        }
                    },
                    on_hover: move |_| {
                        if active_menu().is_some() {
                            active_menu.set(Some("help".to_string())); on_menu_open.call(true);
                        }
                    },
                    MenuDropdown {
                        MenuItemButton {
                            item: MenuItem::new("Documentation").disabled(),
                            on_click: move |_| {},
                        }
                        MenuItemButton {
                            item: MenuItem::new("Keyboard Shortcuts").disabled(),
                            on_click: move |_| {},
                        }
                        MenuDivider {}
                        MenuItemButton {
                            item: MenuItem::new("About NLA AI Video Creator").disabled(),
                            on_click: move |_| {},
                        }
                    }
                }
            }

            // Center: Project name
            span { 
                style: "
                    font-size: 12px; color: {TEXT_MUTED};
                    position: absolute; left: 50%; transform: translateX(-50%);
                ", 
                "{project_name}" 
            }

            // Right side: Quick toggles (compact)
            div {
                style: "display: flex; align-items: center; gap: 8px;",
                QuickToggleBadge {
                    label: "QUE",
                    enabled: queue_open,
                    badge_count: queue_count,
                    running: queue_running,
                    paused: queue_paused,
                    on_toggle: on_toggle_queue,
                }
            }

            // Click-away backdrop when menu is open
            if active_menu().is_some() {
                div {
                    style: "
                        position: fixed; top: 32px; left: 0; right: 0; bottom: 0;
                        background: rgba(0, 0, 0, 0.34);
                        backdrop-filter: blur(5px);
                        -webkit-backdrop-filter: blur(5px);
                        z-index: 99;
                    ",
                    onclick: close_menus,
                }
            }
        }
    }
}

/// A menu bar button that opens a dropdown
#[component]
fn MenuButton(
    label: &'static str,
    is_open: bool,
    on_toggle: EventHandler<MouseEvent>,
    on_hover: EventHandler<MouseEvent>,
    children: Element,
) -> Element {
    let bg = if is_open { BG_HOVER } else { "transparent" };
    
    rsx! {
        div {
            style: "position: relative;",
            button {
                class: "menu-button",
                style: "
                    background: {bg}; border: none; color: {TEXT_PRIMARY};
                    font-size: 13px; cursor: pointer; padding: 6px 10px;
                    border-radius: 4px;
                ",
                onclick: move |e| on_toggle.call(e),
                onmouseenter: move |e| on_hover.call(e),
                "{label}"
            }
            if is_open {
                {children}
            }
        }
    }
}

/// Container for dropdown menu items
#[component]
fn MenuDropdown(children: Element) -> Element {
    rsx! {
        div {
            style: "
                position: absolute; top: 100%; left: 0; min-width: 200px;
                background-color: {BG_ELEVATED}; border: 1px solid {BORDER_DEFAULT};
                border-radius: 6px; box-shadow: 0 8px 24px rgba(0,0,0,0.4);
                padding: 4px 0; z-index: 100; margin-top: 2px;
            ",
            {children}
        }
    }
}

/// A single menu item button
#[component]
fn MenuItemButton(
    item: MenuItem,
    on_click: EventHandler<MouseEvent>,
) -> Element {
    let text_color = if item.enabled { TEXT_PRIMARY } else { TEXT_DIM };
    let cursor = if item.enabled { "pointer" } else { "default" };
    let hotkey = item.hotkey.clone().unwrap_or_default();
    let show_check = item.checked.is_some();
    let is_checked = item.checked.unwrap_or(false);

    rsx! {
        button {
            class: "menu-item",
            style: "
                display: flex; align-items: center; justify-content: space-between;
                width: 100%; background: transparent; border: none;
                color: {text_color}; font-size: 13px; cursor: {cursor};
                padding: 6px 12px; text-align: left;
            ",
            onclick: move |e| {
                if item.enabled {
                    on_click.call(e);
                }
            },
            div {
                style: "display: flex; align-items: center; gap: 8px;",
                // Checkmark area
                span {
                    style: "width: 16px; text-align: center; font-size: 12px;",
                    if show_check && is_checked { "✓" } else { "" }
                }
                span { "{item.label}" }
            }
            if !hotkey.is_empty() {
                span { 
                    style: "color: {TEXT_DIM}; font-size: 11px; margin-left: 16px;",
                    "{hotkey}" 
                }
            }
        }
    }
}

/// A divider line in the menu
#[component]
fn MenuDivider() -> Element {
    rsx! {
        div {
            style: "
                height: 1px; background-color: {BORDER_DEFAULT};
                margin: 4px 8px;
            ",
        }
    }
}

/// Quick toggle button with a small badge count.
#[component]
fn QuickToggleBadge(
    label: &'static str,
    enabled: bool,
    badge_count: usize,
    running: bool,
    paused: bool,
    on_toggle: EventHandler<MouseEvent>,
) -> Element {
    let bg = if enabled { ACCENT_PRIMARY } else { BG_BASE };
    let border = if enabled { ACCENT_PRIMARY } else { BORDER_DEFAULT };
    let text = if enabled { "#000" } else { TEXT_MUTED };
    let show_badge = badge_count > 0;
    let badge_bg = if paused { BORDER_STRONG } else { ACCENT_MARKER };
    let badge_text_color = if paused { TEXT_PRIMARY } else { "#111" };
    let badge_text = if badge_count > 99 {
        "99+".to_string()
    } else {
        badge_count.to_string()
    };
    let button_class = if running {
        "collapse-btn queue-running"
    } else {
        "collapse-btn"
    };

    rsx! {
        button {
            class: "{button_class}",
            style: "
                position: relative;
                background: {bg}; border: 1px solid {border};
                color: {text}; font-size: 10px; font-weight: 500;
                cursor: pointer; padding: 2px 8px; border-radius: 999px;
                text-transform: uppercase; letter-spacing: 0.5px;
            ",
            onclick: move |e| on_toggle.call(e),
            "{label}"
            if show_badge {
                span {
                    style: "
                        position: absolute; top: -6px; right: -4px;
                        min-width: 16px; height: 16px;
                        background-color: {badge_bg};
                        color: {badge_text_color}; font-size: 9px; font-weight: 700;
                        border-radius: 999px; padding: 0 4px;
                        display: inline-flex; align-items: center; justify-content: center;
                        border: 1px solid {BG_BASE};
                    ",
                    "{badge_text}"
                }
            }
        }
    }
}
