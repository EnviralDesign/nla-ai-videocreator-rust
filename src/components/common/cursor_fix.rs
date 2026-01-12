/// Cursor position preservation for Dioxus inputs
/// 
/// Dioxus controlled inputs reset cursor to end on every re-render.
/// This module provides utilities to work around that using RefCell.

use dioxus::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

/// A text input that preserves cursor position during typing.
/// Uses a RefCell to avoid triggering re-renders during input,
/// only commits on blur or Enter key.
#[component]
pub fn StableTextInput(
    id: String,
    value: String,
    placeholder: Option<String>,
    style: Option<String>,
    on_change: EventHandler<String>,
) -> Element {
    // Use RefCell to store draft without triggering re-renders
    let draft = use_hook(|| Rc::new(RefCell::new(value.clone())));
    let mut last_external_value = use_signal(|| value.clone());
    
    // Sync external value changes (but not during editing)
    let mut is_focused = use_signal(|| false);
    {
        let draft = draft.clone();
        let value = value.clone();
        use_effect(move || {
            if !is_focused() && value != last_external_value() {
                *draft.borrow_mut() = value.clone();
                last_external_value.set(value.clone());
            }
        });
    }
    
    let draft_for_input = draft.clone();
    let draft_for_blur = draft.clone();
    let draft_for_key = draft.clone();
    
    let default_style = "
        width: 100%; box-sizing: border-box;
        padding: 6px 8px; font-size: 12px;
        background-color: #1e1e1e; color: #e0e0e0;
        border: 1px solid #3a3a3a; border-radius: 4px;
        outline: none; user-select: text;
    ";
    
    let final_style = style.unwrap_or_else(|| default_style.to_string());
    let placeholder_text = placeholder.unwrap_or_default();
    
    rsx! {
        input {
            id: "{id}",
            r#type: "text",
            value: "{draft.borrow().clone()}",
            placeholder: "{placeholder_text}",
            style: "{final_style}",
            onfocus: move |_| is_focused.set(true),
            oninput: move |e| {
                // Update RefCell directly - no re-render
                *draft_for_input.borrow_mut() = e.value();
            },
            onblur: move |_| {
                is_focused.set(false);
                let final_value = draft_for_blur.borrow().clone();
                on_change.call(final_value);
            },
            onkeydown: move |e: KeyboardEvent| {
                if e.key() == Key::Enter {
                    let final_value = draft_for_key.borrow().clone();
                    on_change.call(final_value);
                }
            },
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
    on_change: EventHandler<String>,
) -> Element {
    let draft = use_hook(|| Rc::new(RefCell::new(value.clone())));
    let mut last_external_value = use_signal(|| value.clone());
    let mut is_focused = use_signal(|| false);
    
    {
        let draft = draft.clone();
        let value = value.clone();
        use_effect(move || {
            if !is_focused() && value != last_external_value() {
                *draft.borrow_mut() = value.clone();
                last_external_value.set(value.clone());
            }
        });
    }
    
    let draft_for_input = draft.clone();
    let draft_for_blur = draft.clone();
    
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
    
    rsx! {
        textarea {
            id: "{id}",
            value: "{draft.borrow().clone()}",
            placeholder: "{placeholder_text}",
            style: "{final_style}",
            onfocus: move |_| is_focused.set(true),
            oninput: move |e| {
                *draft_for_input.borrow_mut() = e.value();
            },
            onblur: move |_| {
                is_focused.set(false);
                let final_value = draft_for_blur.borrow().clone();
                on_change.call(final_value);
            },
        }
    }
}

