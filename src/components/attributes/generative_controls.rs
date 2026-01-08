use std::cell::RefCell;
use std::rc::Rc;

use dioxus::prelude::*;

use crate::constants::*;
use crate::state::ProviderEntry;

pub(super) fn render_generative_controls(
    version_options: &[String],
    selected_version_value: &str,
    mut confirm_delete_version: Signal<bool>,
    can_delete_version: bool,
    on_version_change: Rc<RefCell<dyn FnMut(FormEvent)>>,
    on_delete_version: Rc<RefCell<dyn FnMut()>>,
    selected_provider_value: &str,
    compatible_providers: &[ProviderEntry],
    on_provider_change: Rc<RefCell<dyn FnMut(FormEvent)>>,
    show_missing_provider: bool,
    providers_path_label: &str,
    on_generate: Rc<RefCell<dyn FnMut(MouseEvent)>>,
    gen_status: Signal<Option<String>>,
    generate_label: &str,
    generate_opacity: &str,
) -> Element {
    rsx! {
        div {
            style: "
                display: flex; flex-direction: column; gap: 10px;
                padding: 10px; background-color: {BG_SURFACE};
                border: 1px solid {BORDER_SUBTLE}; border-radius: 6px;
            ",
            div {
                style: "font-size: 10px; color: {TEXT_DIM}; text-transform: uppercase; letter-spacing: 0.5px;",
                "Generative"
            }
            div {
                style: "display: flex; flex-direction: column; gap: 6px;",
                span { style: "font-size: 10px; color: {TEXT_MUTED};", "Version" }
                select {
                    value: "{selected_version_value}",
                    disabled: version_options.is_empty(),
                    style: "
                        width: 100%; padding: 6px 8px; font-size: 12px;
                        background-color: {BG_SURFACE}; color: {TEXT_PRIMARY};
                        border: 1px solid {BORDER_DEFAULT}; border-radius: 4px;
                        outline: none;
                    ",
                    onchange: {
                        let on_version_change = on_version_change.clone();
                        move |e| on_version_change.borrow_mut()(e)
                    },
                    if version_options.is_empty() {
                        option { value: "", "No versions yet" }
                    } else {
                        for version in version_options.iter() {
                            option { value: "{version}", "{version}" }
                        }
                    }
                }
            }
            if !version_options.is_empty() && can_delete_version {
                div {
                    style: "display: flex; gap: 8px; align-items: center;",
                    if confirm_delete_version() {
                        button {
                            style: "
                                flex: 1; padding: 6px 8px;
                                background-color: #b91c1c;
                                border: 1px solid #991b1b;
                                border-radius: 6px; color: white; font-size: 11px;
                                cursor: pointer;
                            ",
                            onclick: {
                                let on_delete_version = on_delete_version.clone();
                                move |_| on_delete_version.borrow_mut()()
                            },
                            "Confirm Delete"
                        }
                        button {
                            class: "collapse-btn",
                            style: "
                                padding: 6px 10px;
                                background-color: {BG_SURFACE};
                                border: 1px solid {BORDER_DEFAULT};
                                border-radius: 6px; color: {TEXT_PRIMARY}; font-size: 11px;
                                cursor: pointer;
                            ",
                            onclick: move |_| confirm_delete_version.set(false),
                            "Cancel"
                        }
                    } else {
                        button {
                            class: "collapse-btn",
                            style: "
                                padding: 6px 10px;
                                background-color: {BG_SURFACE};
                                border: 1px solid #7f1d1d;
                                border-radius: 6px; color: #fecaca; font-size: 11px;
                                cursor: pointer;
                            ",
                            onclick: move |_| confirm_delete_version.set(true),
                            "Delete Version"
                        }
                    }
                }
            }
            div {
                style: "display: flex; flex-direction: column; gap: 6px;",
                span { style: "font-size: 10px; color: {TEXT_MUTED};", "Provider" }
                select {
                    value: "{selected_provider_value}",
                    style: "
                        width: 100%; padding: 6px 8px; font-size: 12px;
                        background-color: {BG_SURFACE}; color: {TEXT_PRIMARY};
                        border: 1px solid {BORDER_DEFAULT}; border-radius: 4px;
                        outline: none;
                    ",
                    onchange: {
                        let on_provider_change = on_provider_change.clone();
                        move |e| on_provider_change.borrow_mut()(e)
                    },
                    option { value: "", "None selected" }
                    for provider in compatible_providers.iter() {
                        option { value: "{provider.id}", "{provider.name}" }
                    }
                }
            }
            if show_missing_provider {
                div {
                    style: "font-size: 11px; color: #f97316;",
                    "Selected provider missing from global providers."
                }
            }
            if compatible_providers.is_empty() {
                div {
                    style: "font-size: 11px; color: {TEXT_DIM};",
                    "No providers configured. Add JSON files under {providers_path_label}."
                }
            }
            div {
                style: "display: flex; flex-direction: column; gap: 6px;",
                button {
                    class: "collapse-btn",
                    style: "
                        width: 100%; padding: 8px 10px;
                        background-color: {ACCENT_VIDEO};
                        border: none; border-radius: 6px;
                        color: white; font-size: 12px; cursor: pointer;
                        opacity: {generate_opacity};
                    ",
                    onclick: {
                        let on_generate = on_generate.clone();
                        move |e| on_generate.borrow_mut()(e)
                    },
                    "{generate_label}"
                }
                if let Some(status) = gen_status() {
                    div { style: "font-size: 11px; color: {TEXT_DIM};", "{status}" }
                }
            }
        }
    }
}
