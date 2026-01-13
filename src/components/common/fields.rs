use dioxus::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use crate::components::common::{StableNumberInput, StableTextArea, StableTextInput};
use crate::constants::*;
use crate::utils::{parse_f32_input, parse_f64_input, parse_i64_input};

#[component]
pub fn NumericField(
    label: &'static str,
    value: f32,
    step: &'static str,
    clamp_min: Option<f32>,
    clamp_max: Option<f32>,
    on_commit: EventHandler<f32>,
    #[props(default = None)] on_change: Option<EventHandler<f32>>,
) -> Element {
    let mut text = use_signal(|| format!("{:.2}", value));
    let mut last_prop_value = use_signal(|| value);

    use_effect(move || {
        let v = value;
        if (v - last_prop_value()).abs() > 0.0001 {
            text.set(format!("{:.2}", v));
            last_prop_value.set(v);
        }
    });

    let make_commit = || {
        let mut text = text.clone();
        let mut last_prop_value = last_prop_value.clone();
        let on_commit = on_commit.clone();
        move || {
            let mut parsed = parse_f32_input(&text(), value);
            if let Some(min) = clamp_min {
                parsed = parsed.max(min);
            }
            if let Some(max) = clamp_max {
                parsed = parsed.min(max);
            }
            on_commit.call(parsed);
            text.set(format!("{:.2}", parsed));
            last_prop_value.set(parsed);
        }
    };

    let mut commit_on_blur = make_commit();
    let mut commit_on_key = make_commit();

    let on_blur = move |_| {
        commit_on_blur();
    };

    let on_keydown = move |e: KeyboardEvent| {
        if e.key() == Key::Enter {
            commit_on_key();
        }
    };
    let on_change_handler = on_change.clone();
    let on_change = move |next_value: String| {
        text.set(next_value.clone());
        if let Some(handler) = on_change_handler.as_ref() {
            let mut parsed = parse_f32_input(&next_value, last_prop_value());
            if let Some(min) = clamp_min {
                parsed = parsed.max(min);
            }
            if let Some(max) = clamp_max {
                parsed = parsed.min(max);
            }
            handler.call(parsed);
        }
    };

    let text_value = text();
    let input_id = format!("numeric-field-{}", label.replace(' ', "-"));
    let input_style = format!(
        "
            width: 100%; min-width: 0; box-sizing: border-box;
            padding: 6px 8px; font-size: 12px;
            background-color: {}; color: {};
            border: 1px solid {}; border-radius: 4px;
            outline: none;
            user-select: text;
        ",
        BG_SURFACE, TEXT_PRIMARY, BORDER_DEFAULT
    );

    rsx! {
        div {
            style: "display: flex; flex-direction: column; gap: 4px; min-width: 0;",
            span { style: "font-size: 10px; color: {TEXT_MUTED};", "{label}" }
            StableNumberInput {
                id: input_id,
                value: text_value,
                placeholder: None,
                style: Some(input_style),
                min: clamp_min.map(|v| v.to_string()),
                max: clamp_max.map(|v| v.to_string()),
                step: Some(step.to_string()),
                on_change: on_change,
                on_blur: on_blur,
                on_keydown: on_keydown,
            }
        }
    }
}

#[component]
pub fn ProviderTextField(
    label: String,
    value: String,
    on_commit: EventHandler<String>,
) -> Element {
    let mut text = use_signal(|| value.clone());
    let mut last_prop_value = use_signal(|| value.clone());

    use_effect(move || {
        let v = value.clone();
        if v != last_prop_value() {
            text.set(v.clone());
            last_prop_value.set(v);
        }
    });

    let make_commit = || {
        let text = text.clone();
        let mut last_prop_value = last_prop_value.clone();
        let on_commit = on_commit.clone();
        move || {
            let next = text();
            on_commit.call(next.clone());
            last_prop_value.set(next);
        }
    };

    let mut commit_on_blur = make_commit();
    let mut commit_on_key = make_commit();
    let input_id = format!("provider-text-field-{}", label.replace(' ', "-").to_lowercase());
    let input_style = format!(
        "
            width: 100%; min-width: 0; box-sizing: border-box;
            padding: 6px 8px; font-size: 12px;
            background-color: {}; color: {};
            border: 1px solid {}; border-radius: 4px;
            outline: none;
            user-select: text;
        ",
        BG_SURFACE, TEXT_PRIMARY, BORDER_DEFAULT
    );
    let text_value = text();

    rsx! {
        div {
            style: "display: flex; flex-direction: column; gap: 4px; min-width: 0;",
            span { style: "font-size: 10px; color: {TEXT_MUTED};", "{label}" }
            StableTextInput {
                id: input_id,
                value: text_value,
                placeholder: None,
                style: Some(input_style),
                on_change: move |v| text.set(v),
                on_blur: move |_| commit_on_blur(),
                on_keydown: move |e: KeyboardEvent| {
                    if e.key() == Key::Enter {
                        commit_on_key();
                    }
                },
                autofocus: false,
            }
        }
    }
}

#[component]
pub fn ProviderTextAreaField(
    label: String,
    value: String,
    rows: u32,
    on_commit: EventHandler<String>,
) -> Element {
    let draft = use_hook(|| Rc::new(RefCell::new(value.clone())));
    let draft_dirty = use_hook(|| Rc::new(Cell::new(false)));
    let mut is_focused = use_signal(|| false);

    {
        let draft = draft.clone();
        let draft_dirty = draft_dirty.clone();
        let is_focused = is_focused.clone();
        let value = value.clone();
        use_effect(move || {
            if is_focused() {
                return;
            }
            let mut draft_value = draft.borrow_mut();
            if !draft_dirty.get() && *draft_value != value {
                *draft_value = value.clone();
            } else if draft_dirty.get() && *draft_value == value {
                draft_dirty.set(false);
            }
        });
    }

    let draft_oninput = draft.clone();
    let draft_onblur = draft.clone();
    let draft_dirty_oninput = draft_dirty.clone();
    let input_id = format!("provider-text-area-field-{}", label.replace(' ', "-").to_lowercase());
    let input_style = format!(
        "
            width: 100%; min-width: 0; box-sizing: border-box;
            padding: 6px 8px; font-size: 12px; line-height: 1.4;
            background-color: {}; color: {};
            border: 1px solid {}; border-radius: 4px;
            outline: none;
            resize: vertical;
            user-select: text;
        ",
        BG_SURFACE, TEXT_PRIMARY, BORDER_DEFAULT
    );
    let draft_value = draft.borrow().clone();

    rsx! {
        div {
            style: "display: flex; flex-direction: column; gap: 4px; min-width: 0;",
            span { style: "font-size: 10px; color: {TEXT_MUTED};", "{label}" }
            StableTextArea {
                id: input_id,
                value: draft_value,
                placeholder: None,
                style: Some(input_style),
                rows: Some(rows),
                on_change: move |v| {
                    *draft_oninput.borrow_mut() = v;
                    draft_dirty_oninput.set(true);
                },
                on_focus: move |_| is_focused.set(true),
                on_blur: move |_| {
                    is_focused.set(false);
                    on_commit.call(draft_onblur.borrow().clone());
                },
            }
        }
    }
}

#[component]
pub fn ProviderFloatField(
    label: String,
    value: f64,
    step: &'static str,
    on_commit: EventHandler<f64>,
) -> Element {
    let mut text = use_signal(|| format!("{:.2}", value));
    let mut last_prop_value = use_signal(|| value);

    use_effect(move || {
        let v = value;
        if (v - last_prop_value()).abs() > 0.0001 {
            text.set(format!("{:.2}", v));
            last_prop_value.set(v);
        }
    });

    let make_commit = || {
        let mut text = text.clone();
        let mut last_prop_value = last_prop_value.clone();
        let on_commit = on_commit.clone();
        move || {
            let next = parse_f64_input(&text(), value);
            on_commit.call(next);
            text.set(format!("{:.2}", next));
            last_prop_value.set(next);
        }
    };

    let mut commit_on_blur = make_commit();
    let mut commit_on_key = make_commit();
    let input_id = format!("provider-float-field-{}", label.replace(' ', "-").to_lowercase());
    let input_style = format!(
        "
            width: 100%; min-width: 0; box-sizing: border-box;
            padding: 6px 8px; font-size: 12px;
            background-color: {}; color: {};
            border: 1px solid {}; border-radius: 4px;
            outline: none;
            user-select: text;
        ",
        BG_SURFACE, TEXT_PRIMARY, BORDER_DEFAULT
    );
    let text_value = text();

    rsx! {
        div {
            style: "display: flex; flex-direction: column; gap: 4px; min-width: 0;",
            span { style: "font-size: 10px; color: {TEXT_MUTED};", "{label}" }
            StableNumberInput {
                id: input_id,
                value: text_value,
                placeholder: None,
                style: Some(input_style),
                min: None,
                max: None,
                step: Some(step.to_string()),
                on_change: move |v| text.set(v),
                on_blur: move |_| commit_on_blur(),
                on_keydown: move |e: KeyboardEvent| {
                    if e.key() == Key::Enter {
                        commit_on_key();
                    }
                },
            }
        }
    }
}

#[component]
pub fn ProviderIntegerField(
    label: String,
    value: i64,
    on_commit: EventHandler<i64>,
) -> Element {
    let mut text = use_signal(|| value.to_string());
    let mut last_prop_value = use_signal(|| value);

    use_effect(move || {
        let v = value;
        if v != last_prop_value() {
            text.set(v.to_string());
            last_prop_value.set(v);
        }
    });

    let make_commit = || {
        let mut text = text.clone();
        let mut last_prop_value = last_prop_value.clone();
        let on_commit = on_commit.clone();
        move || {
            let next = parse_i64_input(&text(), value);
            on_commit.call(next);
            text.set(next.to_string());
            last_prop_value.set(next);
        }
    };

    let mut commit_on_blur = make_commit();
    let mut commit_on_key = make_commit();
    let input_id = format!("provider-integer-field-{}", label.replace(' ', "-").to_lowercase());
    let input_style = format!(
        "
            width: 100%; min-width: 0; box-sizing: border-box;
            padding: 6px 8px; font-size: 12px;
            background-color: {}; color: {};
            border: 1px solid {}; border-radius: 4px;
            outline: none;
            user-select: text;
        ",
        BG_SURFACE, TEXT_PRIMARY, BORDER_DEFAULT
    );
    let text_value = text();

    rsx! {
        div {
            style: "display: flex; flex-direction: column; gap: 4px; min-width: 0;",
            span { style: "font-size: 10px; color: {TEXT_MUTED};", "{label}" }
            StableNumberInput {
                id: input_id,
                value: text_value,
                placeholder: None,
                style: Some(input_style),
                min: None,
                max: None,
                step: Some("1".to_string()),
                on_change: move |v| text.set(v),
                on_blur: move |_| commit_on_blur(),
                on_keydown: move |e: KeyboardEvent| {
                    if e.key() == Key::Enter {
                        commit_on_key();
                    }
                },
            }
        }
    }
}
