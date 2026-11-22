//! Form controls shared by every panel: labelled inputs, sliders, selects and the Style group.
use dioxus::prelude::*;

use crate::model::*;

pub fn text_field(label: &'static str, value: String, mut on_input: impl FnMut(String) + 'static) -> Element {
    rsx! {
        label { class: "field",
            span { "{label}" }
            input { r#type: "text", value: "{value}", oninput: move |e| on_input(e.value()) }
        }
    }
}

pub fn area_field(label: &'static str, value: String, rows: u32, mut on_input: impl FnMut(String) + 'static) -> Element {
    rsx! {
        label { class: "field",
            span { "{label}" }
            textarea { rows: "{rows}", value: "{value}", oninput: move |e| on_input(e.value()) }
        }
    }
}

pub fn color_field(label: &'static str, value: String, mut on_input: impl FnMut(String) + 'static) -> Element {
    rsx! {
        label { class: "field color",
            span { "{label}" }
            input { r#type: "color", value: "{value}", oninput: move |e| on_input(e.value()) }
        }
    }
}

pub fn range_field(label: &'static str, value: u32, max: u32, mut on_input: impl FnMut(u32) + 'static) -> Element {
    rsx! {
        label { class: "field",
            span { "{label}: {value}" }
            input {
                r#type: "range",
                min: "0",
                max: "{max}",
                value: "{value}",
                oninput: move |e| {
                    if let Ok(v) = e.value().parse() {
                        on_input(v)
                    }
                },
            }
        }
    }
}

pub fn check_field(label: &'static str, value: bool, mut on_input: impl FnMut(bool) + 'static) -> Element {
    rsx! {
        label { class: "field check",
            input { r#type: "checkbox", checked: value, onchange: move |e| on_input(e.checked()) }
            span { "{label}" }
        }
    }
}

/// A font picker: fonts grouped by category, each option drawn in its own typeface, and a sample
/// line in the chosen one. Options are the `FONTS` keys, so saved choices survive the list changing.
pub fn font_select(label: &'static str, current: String, mut on_change: impl FnMut(String) + 'static) -> Element {
    let chosen = font_info(&current);
    rsx! {
        label { class: "field",
            span { "{label}" }
            select { onchange: move |e| on_change(e.value()),
                for category in FontCategory::ALL {
                    optgroup { label: category.label(),
                        for font in FONTS.iter().filter(|f| f.category == category) {
                            option {
                                value: font.key,
                                selected: font.key == current,
                                style: "font-family: {font.stack}",
                                "{font.name}"
                            }
                        }
                    }
                }
            }
            span { class: "font-sample", style: "font-family: {chosen.stack}", "Ship your launch banner" }
        }
    }
}

pub fn select_field<T: Keyed>(label: &'static str, current: T, mut on_change: impl FnMut(T) + 'static) -> Element {
    rsx! {
        label { class: "field",
            span { "{label}" }
            select {
                onchange: move |e| {
                    if let Some(v) = T::from_key(&e.value()) {
                        on_change(v)
                    }
                },
                for opt in T::ALL.iter().copied() {
                    option { value: opt.key(), selected: opt == current, "{opt.label()}" }
                }
            }
        }
    }
}

pub fn slider(
    label: &'static str,
    value: f64,
    (min, max, step): (f64, f64, f64),
    mut on_input: impl FnMut(f64) + 'static,
) -> Element {
    rsx! {
        label { class: "field",
            span { "{label}: {value}" }
            input {
                r#type: "range",
                min: "{min}",
                max: "{max}",
                step: "{step}",
                value: "{value}",
                oninput: move |e| {
                    if let Ok(v) = e.value().parse() {
                        on_input(v)
                    }
                },
            }
        }
    }
}

/// A collapsed "Style" group (native `<details>`, so it needs no state) with a Reset button.
pub fn style_group(children: Element, mut on_reset: impl FnMut() + 'static) -> Element {
    rsx! {
        details { class: "style-group",
            summary { "Style" }
            div { class: "style-body",
                {children}
                button { class: "reset", onclick: move |_| on_reset(), "Reset to default" }
            }
        }
    }
}
