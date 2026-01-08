use dioxus::prelude::*;
use crate::constants::*;

#[component]
pub fn SidePanel(
    title: &'static str,
    width: f64,
    collapsed: bool,
    side: &'static str,
    is_resizing: bool,
    on_toggle: EventHandler<MouseEvent>,
    on_resize_start: EventHandler<MouseEvent>,
    children: Element,
) -> Element {
    let is_left = side == "left";
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

    let transition = if is_resizing { "none" } else { "width 0.2s ease, min-width 0.2s ease" };
    let rail_cursor = if collapsed { "pointer" } else { "default" };
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
            onclick: move |e| {
                if collapsed {
                    on_toggle.call(e);
                }
            },

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

                if collapsed {
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
                    div {
                        style: "
                            display: flex; align-items: center;
                            height: 32px; padding: 0 8px;
                            background-color: {BG_SURFACE}; border-bottom: 1px solid {BORDER_DEFAULT};
                            flex-shrink: 0;
                        ",
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

                if !collapsed {
                    div {
                        style: "flex: 1; overflow-y: auto;",
                        {children}
                    }
                }
            }

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
