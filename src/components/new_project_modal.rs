use dioxus::prelude::*;

use crate::constants::*;

#[component]
pub fn NewProjectModal(
    show: Signal<bool>,
    on_go_to_wizard: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        if !show() {
            div {}
        } else {
        div {
            style: "
                position: fixed; top: 0; left: 0; right: 0; bottom: 0;
                background-color: rgba(0, 0, 0, 0.5);
                display: flex; align-items: center; justify-content: center;
                z-index: 2000;
            ",
            onclick: move |_| show.set(false),
            div {
                 style: "
                    width: 400px; background-color: {BG_ELEVATED};
                    border: 1px solid {BORDER_DEFAULT}; border-radius: 8px;
                    padding: 24px; box-shadow: 0 10px 25px rgba(0,0,0,0.5);
                ",
                 onclick: move |e| e.stop_propagation(),

                 h3 { style: "margin: 0 0 16px 0; font-size: 16px; color: {TEXT_PRIMARY};", "New Project" }
                 div {
                    style: "margin-bottom: 20px;",
                     button {
                        style: "width: 100%; padding: 10px; background: {ACCENT_VIDEO}; border: none; border-radius: 4px; color: white; cursor: pointer;",
                        onclick: on_go_to_wizard,
                        "Go to Project Wizard"
                     }
                 }
            }
        }
        }
    }
}
