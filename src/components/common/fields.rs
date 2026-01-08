use dioxus::prelude::*;
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

    let text_value = text();

    rsx! {
        div {
            style: "display: flex; flex-direction: column; gap: 4px; min-width: 0;",
            span { style: "font-size: 10px; color: {TEXT_MUTED};", "{label}" }
            input {
                r#type: "number",
                step: "{step}",
                value: "{text_value}",
                style: "
                    width: 100%; min-width: 0; box-sizing: border-box;
                    padding: 6px 8px; font-size: 12px;
                    background-color: {BG_SURFACE}; color: {TEXT_PRIMARY};
                    border: 1px solid {BORDER_DEFAULT}; border-radius: 4px;
                    outline: none;
                    user-select: text;
                ",
                oninput: move |e| text.set(e.value()),
                onblur: on_blur,
                onkeydown: on_keydown,
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

    rsx! {
        div {
            style: "display: flex; flex-direction: column; gap: 4px; min-width: 0;",
            span { style: "font-size: 10px; color: {TEXT_MUTED};", "{label}" }
            input {
                r#type: "text",
                value: "{text()}",
                style: "
                    width: 100%; min-width: 0; box-sizing: border-box;
                    padding: 6px 8px; font-size: 12px;
                    background-color: {BG_SURFACE}; color: {TEXT_PRIMARY};
                    border: 1px solid {BORDER_DEFAULT}; border-radius: 4px;
                    outline: none;
                    user-select: text;
                ",
                oninput: move |e| text.set(e.value()),
                onblur: move |_| commit_on_blur(),
                onkeydown: move |e: KeyboardEvent| {
                    if e.key() == Key::Enter {
                        commit_on_key();
                    }
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

    rsx! {
        div {
            style: "display: flex; flex-direction: column; gap: 4px; min-width: 0;",
            span { style: "font-size: 10px; color: {TEXT_MUTED};", "{label}" }
            input {
                r#type: "number",
                step: "{step}",
                value: "{text()}",
                style: "
                    width: 100%; min-width: 0; box-sizing: border-box;
                    padding: 6px 8px; font-size: 12px;
                    background-color: {BG_SURFACE}; color: {TEXT_PRIMARY};
                    border: 1px solid {BORDER_DEFAULT}; border-radius: 4px;
                    outline: none;
                    user-select: text;
                ",
                oninput: move |e| text.set(e.value()),
                onblur: move |_| commit_on_blur(),
                onkeydown: move |e: KeyboardEvent| {
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

    rsx! {
        div {
            style: "display: flex; flex-direction: column; gap: 4px; min-width: 0;",
            span { style: "font-size: 10px; color: {TEXT_MUTED};", "{label}" }
            input {
                r#type: "number",
                step: "1",
                value: "{text()}",
                style: "
                    width: 100%; min-width: 0; box-sizing: border-box;
                    padding: 6px 8px; font-size: 12px;
                    background-color: {BG_SURFACE}; color: {TEXT_PRIMARY};
                    border: 1px solid {BORDER_DEFAULT}; border-radius: 4px;
                    outline: none;
                    user-select: text;
                ",
                oninput: move |e| text.set(e.value()),
                onblur: move |_| commit_on_blur(),
                onkeydown: move |e: KeyboardEvent| {
                    if e.key() == Key::Enter {
                        commit_on_key();
                    }
                },
            }
        }
    }
}
