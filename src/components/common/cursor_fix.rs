/// Cursor position preservation for Dioxus inputs
/// 
/// Uses the "Uncontrolled" pattern: DON'T bind the value attribute at all.
/// Browser manages the DOM value and cursor natively.
/// We only force re-render via key change when external value changes.

use dioxus::prelude::*;

/// A text input that preserves cursor position during typing.
/// Truly uncontrolled - no value binding, browser manages cursor.
#[component]
pub fn StableTextInput(
    id: String,
    value: String,
    placeholder: Option<String>,
    style: Option<String>,
    on_change: EventHandler<String>,
    on_blur: EventHandler<FocusEvent>,
    on_keydown: EventHandler<KeyboardEvent>,
    autofocus: bool,
) -> Element {
    // Track the last prop value we received
    let mut last_prop_value = use_signal(|| value.clone());
    // Generation key - increment to force DOM element recreation
    let mut key_gen = use_signal(|| 0u32);
    // Internal text state (for tracking, not for binding)
    let mut text = use_signal(|| value.clone());
    
    // Detect external value changes (not from typing)
    if value != last_prop_value() {
        if value != text() {
            // External value changed - update state and force element recreation
            text.set(value.clone());
            key_gen.set(key_gen() + 1);
        }
        last_prop_value.set(value.clone());
    }
    
    let default_style = "
        width: 100%; box-sizing: border-box;
        padding: 6px 8px; font-size: 12px;
        background-color: #1e1e1e; color: #e0e0e0;
        border: 1px solid #3a3a3a; border-radius: 4px;
        outline: none; user-select: text;
    ";
    
    let final_style = style.unwrap_or_else(|| default_style.to_string());
    let placeholder_text = placeholder.unwrap_or_default();
    let current_key = key_gen();
    let initial_value = text();
    let id_for_mount = id.clone();
    let focus_on_mount = autofocus;

    rsx! {
        input {
            // Key forces element recreation when external value changes
            key: "{current_key}",
            id: "{id}",
            r#type: "text",
            // NO value binding - browser manages this
            placeholder: "{placeholder_text}",
            style: "{final_style}",
            // Set initial value via JS when mounted
            onmounted: move |_| {
                let mut js = format!(
                    r#"document.getElementById('{}').value = '{}';"#,
                    id_for_mount,
                    initial_value.replace("'", "\\'").replace("\n", "\\n")
                );
                if focus_on_mount {
                    js.push_str(&format!(
                        r#"document.getElementById('{}').focus();"#,
                        id_for_mount
                    ));
                }
                let _ = document::eval(&js);
            },
            oninput: move |e| {
                let new_val = e.value();
                text.set(new_val.clone());
                on_change.call(new_val);
            },
            onblur: move |e| on_blur.call(e),
            onkeydown: move |e| on_keydown.call(e),
        }
    }
}

/// A textarea that preserves cursor position during typing.
#[component]
pub fn StableTextArea(
    id: String,
    value: String,
    placeholder: Option<String>,
    style: Option<String>,
    rows: Option<u32>,
    on_change: EventHandler<String>,
    on_focus: EventHandler<FocusEvent>,
    on_blur: EventHandler<FocusEvent>,
) -> Element {
    let mut last_prop_value = use_signal(|| value.clone());
    let mut key_gen = use_signal(|| 0u32);
    let mut text = use_signal(|| value.clone());
    
    // Detect external value changes
    if value != last_prop_value() {
        if value != text() {
            text.set(value.clone());
            key_gen.set(key_gen() + 1);
        }
        last_prop_value.set(value.clone());
    }
    
    let default_style = "
        width: 100%; box-sizing: border-box;
        padding: 10px; font-size: 11px; line-height: 1.5;
        background-color: #1e1e1e; color: #e0e0e0;
        border: 1px solid #3a3a3a; border-radius: 6px;
        outline: none; resize: none; white-space: pre;
        user-select: text; font-family: 'SF Mono', Consolas, monospace;
    ";
    
    let final_style = style.unwrap_or_else(|| default_style.to_string());
    let placeholder_text = placeholder.unwrap_or_default();
    let current_key = key_gen();
    let initial_value = text();
    let id_for_mount = id.clone();

    if let Some(rows_value) = rows {
        rsx! {
            textarea {
                key: "{current_key}",
                id: "{id}",
                rows: "{rows_value}",
                // NO value binding - browser manages this
                placeholder: "{placeholder_text}",
                style: "{final_style}",
                // Set initial value via JS when mounted
                onmounted: move |_| {
                    let js = format!(
                        r#"document.getElementById('{}').value = {};"#,
                        id_for_mount,
                        serde_json::to_string(&initial_value).unwrap_or_else(|_| "''".to_string())
                    );
                    let _ = document::eval(&js);
                },
                oninput: move |e| {
                    let new_val = e.value();
                    text.set(new_val.clone());
                    on_change.call(new_val);
                },
                onfocus: move |e| on_focus.call(e),
                onblur: move |e| on_blur.call(e),
            }
        }
    } else {
        rsx! {
            textarea {
                key: "{current_key}",
                id: "{id}",
                // NO value binding - browser manages this
                placeholder: "{placeholder_text}",
                style: "{final_style}",
                // Set initial value via JS when mounted
                onmounted: move |_| {
                    let js = format!(
                        r#"document.getElementById('{}').value = {};"#,
                        id_for_mount,
                        serde_json::to_string(&initial_value).unwrap_or_else(|_| "''".to_string())
                    );
                    let _ = document::eval(&js);
                },
                oninput: move |e| {
                    let new_val = e.value();
                    text.set(new_val.clone());
                    on_change.call(new_val);
                },
                onfocus: move |e| on_focus.call(e),
                onblur: move |e| on_blur.call(e),
            }
        }
    }
}

/// A number input that preserves cursor position during typing.
/// Truly uncontrolled - no value binding, browser manages cursor.
#[component]
pub fn StableNumberInput(
    id: String,
    value: String,
    placeholder: Option<String>,
    style: Option<String>,
    min: Option<String>,
    max: Option<String>,
    step: Option<String>,
    on_change: EventHandler<String>,
    on_blur: EventHandler<FocusEvent>,
    on_keydown: EventHandler<KeyboardEvent>,
) -> Element {
    let mut last_prop_value = use_signal(|| value.clone());
    let mut key_gen = use_signal(|| 0u32);
    let mut text = use_signal(|| value.clone());
    
    // Detect external value changes (not from typing)
    if value != last_prop_value() {
        if value != text() {
            text.set(value.clone());
            key_gen.set(key_gen() + 1);
        }
        last_prop_value.set(value.clone());
    }
    
    let default_style = "
        width: 100%; box-sizing: border-box;
        padding: 6px 8px; font-size: 12px;
        background-color: #1e1e1e; color: #e0e0e0;
        border: 1px solid #3a3a3a; border-radius: 4px;
        outline: none; user-select: text;
    ";
    
    let final_style = style.unwrap_or_else(|| default_style.to_string());
    let placeholder_text = placeholder.unwrap_or_default();
    let min_val = min.unwrap_or_default();
    let max_val = max.unwrap_or_default();
    let step_val = step.unwrap_or_else(|| "1".to_string());
    let current_key = key_gen();
    let initial_value = text();
    let id_for_mount = id.clone();

    rsx! {
        input {
            key: "{current_key}",
            id: "{id}",
            r#type: "number",
            // NO value binding - browser manages this
            placeholder: "{placeholder_text}",
            style: "{final_style}",
            min: "{min_val}",
            max: "{max_val}",
            step: "{step_val}",
            // Set initial value via JS when mounted
            onmounted: move |_| {
                let js = format!(
                    r#"document.getElementById('{}').value = '{}';"#,
                    id_for_mount,
                    initial_value
                );
                let _ = document::eval(&js);
            },
            oninput: move |e| {
                let new_val = e.value();
                text.set(new_val.clone());
                on_change.call(new_val);
            },
            onblur: move |e| on_blur.call(e),
            onkeydown: move |e| on_keydown.call(e),
        }
    }
}
