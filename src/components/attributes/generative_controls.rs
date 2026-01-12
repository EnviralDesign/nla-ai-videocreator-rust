use std::cell::RefCell;
use std::rc::Rc;

use dioxus::prelude::*;

use crate::components::common::ProviderIntegerField;
use crate::constants::*;
use crate::state::ProviderEntry;

pub(super) fn render_generative_controls(
    version_options: &[String],
    selected_version_value: &str,
    mut manage_versions_open: Signal<bool>,
    mut confirm_delete_current: Signal<bool>,
    mut confirm_delete_others: Signal<bool>,
    can_delete_version: bool,
    on_version_change: Rc<RefCell<dyn FnMut(FormEvent)>>,
    on_delete_version: Rc<RefCell<dyn FnMut()>>,
    on_delete_other_versions: Rc<RefCell<dyn FnMut()>>,
    on_delete_all_versions: Rc<RefCell<dyn FnMut()>>,
    selected_provider_value: &str,
    compatible_providers: &[ProviderEntry],
    on_provider_change: Rc<RefCell<dyn FnMut(FormEvent)>>,
    show_missing_provider: bool,
    providers_path_label: &str,
    on_generate: Rc<RefCell<dyn FnMut(MouseEvent)>>,
    gen_status: Signal<Option<String>>,
    generate_label: &str,
    generate_opacity: &str,
    batch_count: u32,
    on_batch_count_change: Rc<RefCell<dyn FnMut(i64)>>,
    seed_strategy_value: &str,
    on_seed_strategy_change: Rc<RefCell<dyn FnMut(FormEvent)>>,
    seed_field_value: &str,
    seed_field_options: &[(String, String)],
    on_seed_field_change: Rc<RefCell<dyn FnMut(FormEvent)>>,
    seed_hint: Option<String>,
    seed_hint_is_warning: bool,
    batch_hint: Option<String>,
    mut confirm_delete_all: Signal<bool>,
) -> Element {
    let has_versions = !version_options.is_empty();
    let has_other_versions = can_delete_version
        && version_options
            .iter()
            .any(|version| version != selected_version_value);
    let can_delete_current = can_delete_version;
    let can_delete_all = has_versions;
    let manage_opacity = if has_versions { "0.8" } else { "0.4" };
    let current_opacity = if can_delete_current { "1.0" } else { "0.4" };
    let others_opacity = if has_other_versions { "1.0" } else { "0.4" };
    let all_opacity = if can_delete_all { "1.0" } else { "0.4" };
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
                div {
                    style: "display: flex; align-items: center; justify-content: space-between;",
                    span { style: "font-size: 10px; color: {TEXT_MUTED};", "Version" }
                    button {
                        class: "collapse-btn",
                        style: "
                            padding: 4px 8px; border-radius: 6px;
                            border: 1px solid {BORDER_DEFAULT};
                            background-color: {BG_SURFACE}; color: {TEXT_PRIMARY};
                            font-size: 11px; cursor: pointer;
                            opacity: {manage_opacity};
                        ",
                        disabled: !has_versions,
                        onclick: move |_| {
                            if manage_versions_open() {
                                manage_versions_open.set(false);
                                confirm_delete_current.set(false);
                                confirm_delete_others.set(false);
                                confirm_delete_all.set(false);
                            } else {
                                manage_versions_open.set(true);
                            }
                        },
                        "Manage"
                    }
                }
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
            if manage_versions_open() {
                div {
                    style: "
                        display: flex; flex-direction: column; gap: 8px;
                        padding: 8px; border: 1px solid {BORDER_DEFAULT};
                        border-radius: 8px; background-color: {BG_ELEVATED};
                    ",
                    if confirm_delete_current() {
                        div {
                            style: "display: flex; gap: 8px; align-items: center;",
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
                                "Confirm Delete Current"
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
                                onclick: move |_| confirm_delete_current.set(false),
                                "Cancel"
                            }
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
                                opacity: {current_opacity};
                            ",
                            disabled: !can_delete_current,
                            onclick: move |_| {
                                if can_delete_current {
                                    confirm_delete_current.set(true);
                                    confirm_delete_others.set(false);
                                    confirm_delete_all.set(false);
                                }
                            },
                            "Delete Current"
                        }
                    }
                    if confirm_delete_others() {
                        div {
                            style: "display: flex; gap: 8px; align-items: center;",
                            button {
                                style: "
                                    flex: 1; padding: 6px 8px;
                                    background-color: #b91c1c;
                                    border: 1px solid #991b1b;
                                    border-radius: 6px; color: white; font-size: 11px;
                                    cursor: pointer;
                                ",
                                onclick: {
                                    let on_delete_other_versions = on_delete_other_versions.clone();
                                    move |_| on_delete_other_versions.borrow_mut()()
                                },
                                "Confirm Delete Others"
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
                                onclick: move |_| confirm_delete_others.set(false),
                                "Cancel"
                            }
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
                                opacity: {others_opacity};
                            ",
                            disabled: !has_other_versions,
                            onclick: move |_| {
                                if has_other_versions {
                                    confirm_delete_others.set(true);
                                    confirm_delete_current.set(false);
                                    confirm_delete_all.set(false);
                                }
                            },
                            "Delete Others"
                        }
                    }
                    if confirm_delete_all() {
                        div {
                            style: "display: flex; gap: 8px; align-items: center;",
                            button {
                                style: "
                                    flex: 1; padding: 6px 8px;
                                    background-color: #b91c1c;
                                    border: 1px solid #991b1b;
                                    border-radius: 6px; color: white; font-size: 11px;
                                    cursor: pointer;
                                ",
                                onclick: {
                                    let on_delete_all_versions = on_delete_all_versions.clone();
                                    move |_| on_delete_all_versions.borrow_mut()()
                                },
                                "Confirm Delete All"
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
                                onclick: move |_| confirm_delete_all.set(false),
                                "Cancel"
                            }
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
                                opacity: {all_opacity};
                            ",
                            disabled: !can_delete_all,
                            onclick: move |_| {
                                if can_delete_all {
                                    confirm_delete_all.set(true);
                                    confirm_delete_current.set(false);
                                    confirm_delete_others.set(false);
                                }
                            },
                            "Delete All"
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
            div {
                style: "
                    display: flex; flex-direction: column; gap: 8px;
                    padding: 8px; border: 1px dashed {BORDER_SUBTLE};
                    border-radius: 6px; background-color: rgba(255, 255, 255, 0.02);
                ",
                div {
                    style: "font-size: 10px; color: {TEXT_DIM}; text-transform: uppercase; letter-spacing: 0.5px;",
                    "Batch"
                }
                ProviderIntegerField {
                    label: "Count".to_string(),
                    value: batch_count as i64,
                    on_commit: {
                        let on_batch_count_change = on_batch_count_change.clone();
                        move |next| on_batch_count_change.borrow_mut()(next)
                    }
                }
                div {
                    style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(120px, 1fr)); gap: 8px;",
                    div {
                        style: "display: flex; flex-direction: column; gap: 4px;",
                        span { style: "font-size: 10px; color: {TEXT_MUTED};", "Seed Strategy" }
                        select {
                            value: "{seed_strategy_value}",
                            style: "
                                width: 100%; padding: 6px 8px; font-size: 12px;
                                background-color: {BG_SURFACE}; color: {TEXT_PRIMARY};
                                border: 1px solid {BORDER_DEFAULT}; border-radius: 4px;
                                outline: none;
                            ",
                            onchange: {
                                let on_seed_strategy_change = on_seed_strategy_change.clone();
                                move |e| on_seed_strategy_change.borrow_mut()(e)
                            },
                            option { value: "increment", "Increment" }
                            option { value: "random", "Random" }
                            option { value: "keep", "Keep" }
                        }
                    }
                    div {
                        style: "display: flex; flex-direction: column; gap: 4px;",
                        span { style: "font-size: 10px; color: {TEXT_MUTED};", "Seed Field" }
                        select {
                            value: "{seed_field_value}",
                            style: "
                                width: 100%; padding: 6px 8px; font-size: 12px;
                                background-color: {BG_SURFACE}; color: {TEXT_PRIMARY};
                                border: 1px solid {BORDER_DEFAULT}; border-radius: 4px;
                                outline: none;
                            ",
                            onchange: {
                                let on_seed_field_change = on_seed_field_change.clone();
                                move |e| on_seed_field_change.borrow_mut()(e)
                            },
                            option { value: "", "Auto-detect" }
                            for (value, label) in seed_field_options.iter() {
                                option { value: "{value}", "{label}" }
                            }
                        }
                    }
                }
                if let Some(hint) = seed_hint.as_ref() {
                    if seed_hint_is_warning {
                        div { style: "font-size: 10px; color: #f97316;", "{hint}" }
                    } else {
                        div { style: "font-size: 10px; color: {TEXT_DIM};", "{hint}" }
                    }
                }
                if let Some(hint) = batch_hint.as_ref() {
                    div { style: "font-size: 10px; color: #f97316;", "{hint}" }
                }
            }
        }
    }
}
