//! Form controls shared by every panel: labelled inputs, sliders, selects and the Style group.
use dioxus::prelude::*;

use crate::model::*;

use super::collapsible::Collapsible;

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

/// A collapsed "Style" group with a Reset button.
pub fn style_group(children: Element, on_reset: impl FnMut() + 'static) -> Element {
    let mut on_reset = on_reset;
    rsx! {
        Collapsible { title: "Style", on_reset: move |_| on_reset(), {children} }
    }
}

/// An option set short enough for a segmented control: 2 to 4 options with one-word labels.
pub trait Segmented: Keyed {
    /// The label on the segment; the full `label()` becomes its tooltip.
    fn short(self) -> &'static str;
}

impl Segmented for Preset {
    fn short(self) -> &'static str {
        match self {
            Preset::Banner => "Banner",
            Preset::Social => "Social",
            Preset::Square => "Square",
            Preset::Wide => "Wide",
        }
    }
}

impl Segmented for BgKind {
    fn short(self) -> &'static str {
        match self {
            BgKind::Solid => "Solid",
            BgKind::Gradient => "Gradient",
            BgKind::Grid => "Grid",
            BgKind::Dots => "Dots",
        }
    }
}

impl Segmented for FeatureLayout {
    fn short(self) -> &'static str {
        match self {
            FeatureLayout::Horizontal => "Row",
            FeatureLayout::Vertical => "Column",
        }
    }
}

/// A labelled row of buttons for a small option set; the chosen one is pressed.
pub fn segmented<T: Segmented>(label: &'static str, current: T, on_change: impl FnMut(T) + 'static) -> Element {
    // A Callback is Copy, so every segment's click handler can share it.
    let on_change = Callback::new(on_change);
    rsx! {
        div { class: "field",
            span { "{label}" }
            div { class: "segmented wide", role: "group", aria_label: label,
                for opt in T::ALL.iter().copied() {
                    button {
                        class: if opt == current { "seg on" } else { "seg" },
                        title: opt.label(),
                        aria_pressed: opt == current,
                        onclick: move |_| on_change.call(opt),
                        "{opt.short()}"
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fits<T: Segmented>() {
        assert!((2..=4).contains(&T::ALL.len()), "{} options", T::ALL.len());
        let mut seen = Vec::new();
        for &opt in T::ALL {
            let short = opt.short();
            assert!(!short.is_empty() && short.chars().count() <= 10, "{short:?} is too long");
            assert!(!seen.contains(&short), "{short:?} repeats");
            seen.push(short);
        }
    }

    #[test]
    fn segmented_option_sets_are_small_with_short_distinct_labels() {
        fits::<Preset>();
        fits::<BgKind>();
        fits::<FeatureLayout>();
    }
}
