use std::cell::RefCell;
use std::rc::Rc;

use dioxus::prelude::*;

use crate::components::common::{
    ProviderFloatField, ProviderIntegerField, ProviderTextAreaField, ProviderTextField,
};
use crate::constants::*;
use crate::state::{
    input_value_as_bool, input_value_as_f64, input_value_as_i64, input_value_as_string,
    GenerativeConfig, ProviderEntry, ProviderInputType,
};

pub(super) fn render_provider_inputs(
    selected_provider: Option<ProviderEntry>,
    show_missing_provider: bool,
    config_snapshot: &GenerativeConfig,
    version_key: &str,
    set_input_value: Rc<RefCell<dyn FnMut(String, serde_json::Value)>>,
) -> Element {
    let version_key = if version_key.trim().is_empty() {
        "current"
    } else {
        version_key
    };
    rsx! {
        div {
            style: "
                display: flex; flex-direction: column; gap: 10px;
                padding: 10px; background-color: {BG_SURFACE};
                border: 1px solid {BORDER_SUBTLE}; border-radius: 6px;
            ",
            div {
                style: "font-size: 10px; color: {TEXT_DIM}; text-transform: uppercase; letter-spacing: 0.5px;",
                "Provider Inputs"
            }
            if show_missing_provider {
                span { style: "font-size: 11px; color: {TEXT_DIM};", "Select a valid provider to configure inputs." }
            } else if let Some(provider) = selected_provider {
                if provider.inputs.is_empty() {
                    span { style: "font-size: 11px; color: {TEXT_DIM};", "No inputs defined." }
                } else {
                    for input in provider.inputs.iter() {
                        {
                            let label = if input.required {
                                format!("{} *", input.label)
                            } else {
                                input.label.clone()
                            };
                            let stored_value = config_snapshot.inputs.get(&input.name).and_then(|input| {
                                if let crate::state::InputValue::Literal { value } = input {
                                    Some(value.clone())
                                } else {
                                    None
                                }
                            });
                            let current_value = stored_value.or_else(|| input.default.clone());
                            let input_name = input.name.clone();
                            let input_type = input.input_type.clone();
                            let field_key = format!("{}::{}", version_key, input.name);
                            let set_input_value = set_input_value.clone();
                            match input_type {
                                ProviderInputType::Text => {
                                    let value = current_value
                                        .as_ref()
                                        .and_then(input_value_as_string)
                                        .unwrap_or_default();
                                    let multiline = input
                                        .ui
                                        .as_ref()
                                        .map(|ui| ui.multiline)
                                        .unwrap_or(false);
                                    rsx! {
                                        if multiline {
                                            ProviderTextAreaField {
                                                key: "{field_key}",
                                                label: label.clone(),
                                                value: value.clone(),
                                                rows: 3,
                                                on_commit: move |next| {
                                                    set_input_value
                                                        .borrow_mut()(input_name.clone(), serde_json::Value::String(next));
                                                }
                                            }
                                        } else {
                                            ProviderTextField {
                                                key: "{field_key}",
                                                label: label.clone(),
                                                value: value.clone(),
                                                on_commit: move |next| {
                                                    set_input_value
                                                        .borrow_mut()(input_name.clone(), serde_json::Value::String(next));
                                                }
                                            }
                                        }
                                    }
                                }
                                ProviderInputType::Number => {
                                    let value = current_value
                                        .as_ref()
                                        .and_then(input_value_as_f64)
                                        .unwrap_or(0.0);
                                    rsx! {
                                        ProviderFloatField {
                                            key: "{field_key}",
                                            label: label.clone(),
                                            value,
                                            step: "0.1",
                                            on_commit: move |next| {
                                                if let Some(number) = serde_json::Number::from_f64(next) {
                                                    set_input_value
                                                        .borrow_mut()(input_name.clone(), serde_json::Value::Number(number));
                                                }
                                            }
                                        }
                                    }
                                }
                                ProviderInputType::Integer => {
                                    let value = current_value
                                        .as_ref()
                                        .and_then(input_value_as_i64)
                                        .unwrap_or(0);
                                    rsx! {
                                        ProviderIntegerField {
                                            key: "{field_key}",
                                            label: label.clone(),
                                            value,
                                            on_commit: move |next: i64| {
                                                set_input_value
                                                    .borrow_mut()(input_name.clone(), serde_json::Value::Number(next.into()));
                                            }
                                        }
                                    }
                                }
                                ProviderInputType::Boolean => {
                                    let enabled = current_value
                                        .as_ref()
                                        .and_then(input_value_as_bool)
                                        .unwrap_or(false);
                                    rsx! {
                                        div {
                                            key: "{field_key}",
                                            style: "display: flex; align-items: center; justify-content: space-between; gap: 8px;",
                                            span { style: "font-size: 10px; color: {TEXT_MUTED};", "{label}" }
                                            button {
                                                class: "collapse-btn",
                                                style: "
                                                    padding: 4px 10px;
                                                    background-color: {BG_SURFACE};
                                                    border: 1px solid {BORDER_DEFAULT};
                                                    border-radius: 999px;
                                                    color: {TEXT_PRIMARY}; font-size: 11px; cursor: pointer;
                                                ",
                                                onclick: move |_| {
                                                    set_input_value
                                                        .borrow_mut()(input_name.clone(), serde_json::Value::Bool(!enabled));
                                                },
                                                if enabled { "On" } else { "Off" }
                                            }
                                        }
                                    }
                                }
                                ProviderInputType::Enum { options } => {
                                    let current = current_value
                                        .as_ref()
                                        .and_then(input_value_as_string)
                                        .unwrap_or_default();
                                    rsx! {
                                        div {
                                            key: "{field_key}",
                                            style: "display: flex; flex-direction: column; gap: 4px;",
                                            span { style: "font-size: 10px; color: {TEXT_MUTED};", "{label}" }
                                            select {
                                                value: "{current}",
                                                style: "
                                                    width: 100%; padding: 6px 8px; font-size: 12px;
                                                    background-color: {BG_SURFACE}; color: {TEXT_PRIMARY};
                                                    border: 1px solid {BORDER_DEFAULT}; border-radius: 4px;
                                                    outline: none;
                                                ",
                                                onchange: move |e| {
                                                    set_input_value
                                                        .borrow_mut()(input_name.clone(), serde_json::Value::String(e.value()));
                                                },
                                                for option in options.iter() {
                                                    option { value: "{option}", "{option}" }
                                                }
                                            }
                                        }
                                    }
                                }
                                ProviderInputType::Image
                                | ProviderInputType::Video
                                | ProviderInputType::Audio => {
                                    rsx! {
                                        div {
                                            key: "{field_key}",
                                            style: "font-size: 10px; color: {TEXT_DIM};",
                                            "{label} (asset inputs not wired yet)"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                span { style: "font-size: 11px; color: {TEXT_DIM};", "Select a provider to configure inputs." }
            }
        }
    }
}
