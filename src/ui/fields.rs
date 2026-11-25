//! Form controls shared by every panel: labelled inputs, sliders, selects and the Style group.
use atelier_ui::widgets::Accordion;
use atelier_ui::elements::{Checkbox, Field, FieldControl, Select, SelectOption, Slider, ToggleGroup};
use dioxus::prelude::*;

use crate::model::*;


pub fn text_field(label: &'static str, value: String, on_input: impl FnMut(String) + 'static) -> Element {
    rsx! { Field { label, id: field_id(label), value, oninput: on_input } }
}

pub fn area_field(label: &'static str, value: String, rows: u32, on_input: impl FnMut(String) + 'static) -> Element {
    rsx! { Field { label, id: field_id(label), control: FieldControl::Textarea, rows, value, oninput: on_input } }
}

/// A whole-number slider from 0 to `max`.
pub fn range_field(label: &'static str, value: u32, max: u32, mut on_input: impl FnMut(u32) + 'static) -> Element {
    rsx! {
        Slider {
            label,
            value: value as f64,
            max: max as f64,
            onchange: move |v: f64| on_input(whole(v, max)),
        }
    }
}

/// A slider value as a whole number within 0..=max.
fn whole(v: f64, max: u32) -> u32 {
    v.round().clamp(0.0, max as f64) as u32
}

pub fn check_field(label: &'static str, value: bool, on_input: impl FnMut(bool) + 'static) -> Element {
    rsx! { Checkbox { label, checked: value, onchange: on_input } }
}

/// The font picker's options: every `FONTS` key, in picker category order, each described by its
/// category (the list has no groups). Values are the keys, so saved choices survive the list changing.
pub fn font_options() -> Vec<SelectOption> {
    FontCategory::ALL
        .iter()
        .flat_map(|&category| FONTS.iter().filter(move |f| f.category == category))
        .map(|f| SelectOption::new(f.key, f.name).describe(f.category.label()))
        .collect()
}

/// The options for a keyed choice, in `ALL` order.
pub fn keyed_options<T: Keyed>() -> Vec<SelectOption> {
    T::ALL.iter().map(|o| SelectOption::new(o.key(), o.label())).collect()
}

/// The element id for a labelled control: "Heading font" becomes "field-heading-font".
fn field_id(label: &str) -> String {
    let slug: String = label.chars().map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '-' }).collect();
    format!("field-{slug}")
}

/// A font picker (atelier-ui `Select`): each option shows "Aa" in its own typeface, and a sample
/// line below shows the chosen one.
pub fn font_select(label: &'static str, current: String, on_change: impl FnMut(String) + 'static) -> Element {
    let chosen = font_info(&current);
    let options = font_options();
    let stacks: Vec<&'static str> = options.iter().map(|o| font_info(&o.value).stack).collect();
    let id = field_id(label);
    rsx! {
        div { class: "field",
            label { r#for: "{id}", "{label}" }
            Select {
                id: id.clone(),
                options,
                value: current.clone(),
                leading: rsx! { span { class: "font-aa", style: "font-family: {chosen.stack}", "Aa" } },
                option_leading: Callback::new(move |i: usize| {
                    let stack = stacks.get(i).copied().unwrap_or_default();
                    rsx! { span { class: "font-aa", style: "font-family: {stack}", "Aa" } }
                }),
                oninput: on_change,
            }
            span { class: "font-sample", style: "font-family: {chosen.stack}", "Ship your launch banner" }
        }
    }
}

/// A drop-down (atelier-ui `Select`) for a keyed choice; unknown values are ignored.
pub fn select_field<T: Keyed>(label: &'static str, current: T, mut on_change: impl FnMut(T) + 'static) -> Element {
    let id = field_id(label);
    rsx! {
        div { class: "field",
            label { r#for: "{id}", "{label}" }
            Select {
                id: id.clone(),
                options: keyed_options::<T>(),
                value: current.key().to_string(),
                oninput: move |v: String| {
                    if let Some(v) = T::from_key(&v) {
                        on_change(v)
                    }
                },
            }
        }
    }
}

pub fn slider(label: &'static str, value: f64, (min, max, step): (f64, f64, f64), on_input: impl FnMut(f64) + 'static) -> Element {
    rsx! { Slider { label, value, min, max, step, onchange: on_input } }
}

/// A collapsed "Style" group (atelier-ui `Accordion`) with a Reset button at the end.
pub fn style_group(children: Element, mut on_reset: impl FnMut() + 'static) -> Element {
    rsx! {
        Accordion { title: "Style",
            {children}
            button { class: "reset", onclick: move |_| on_reset(), "Reset to default" }
        }
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

/// The choice a single-choice ToggleGroup reports: its one value, if it is a known key. Pressing the
/// already-pressed toggle reports nothing, which leaves the choice as it was.
pub fn chosen<T: Keyed>(values: &[String]) -> Option<T> {
    values.first().and_then(|v| T::from_key(v))
}

/// A labelled single-choice ToggleGroup (atelier-ui) for a small option set. When the short labels
/// leave something out (sizes, for example), the full label of the current choice shows below.
pub fn segmented<T: Segmented>(label: &'static str, current: T, mut on_change: impl FnMut(T) + 'static) -> Element {
    let options: Vec<SelectOption> = T::ALL.iter().map(|o| SelectOption::new(o.key(), o.short())).collect();
    rsx! {
        div { class: "field segmented-field",
            span { aria_hidden: "true", "{label}" }
            ToggleGroup {
                label,
                options,
                value: vec![current.key().to_string()],
                onchange: move |values: Vec<String>| {
                    if let Some(v) = chosen::<T>(&values) {
                        on_change(v)
                    }
                },
            }
            if current.short() != current.label() {
                span { class: "hint", "{current.label()}" }
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
    fn font_options_list_every_font_by_category_with_keys_as_values() {
        let options = font_options();
        assert_eq!(options.len(), FONTS.len());
        let order = |key: &str| FontCategory::ALL.iter().position(|&c| c == font_info(key).category).unwrap();
        assert!(options.windows(2).all(|w| order(&w[0].value) <= order(&w[1].value)), "grouped by category");
        for o in &options {
            let font = font_info(&o.value);
            assert_eq!(font.key, o.value);
            assert_eq!(o.label, font.name);
            assert_eq!(o.description.as_deref(), Some(font.category.label()));
        }
    }

    #[test]
    fn keyed_options_follow_all() {
        let options = keyed_options::<Mark>();
        assert_eq!(options.len(), Mark::ALL.len());
        for (o, m) in options.iter().zip(Mark::ALL) {
            assert_eq!((o.value.as_str(), o.label.as_str()), (m.key(), m.label()));
        }
    }

    fn render(app: fn() -> Element) -> String {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    #[test]
    fn dropdowns_are_atelier_selects_not_native_ones() {
        fn app() -> Element {
            rsx! {
                {font_select("Heading font", "inter".into(), |_| {})}
                {select_field("Mark", Mark::ALL[0], |_| {})}
            }
        }
        let html = render(app);
        assert!(!html.contains("<select"), "{html}");
        assert_eq!(html.matches(r#"role="combobox""#).count(), 2, "{html}");
        for id in ["field-heading-font", "field-mark"] {
            assert!(html.contains(&format!(r#"for="{id}""#)) && html.contains(&format!(r#"id="{id}""#)), "{id} is labelled");
        }
    }

    #[test]
    fn whole_number_sliders_round_and_stay_in_range() {
        assert_eq!(whole(44.6, 360), 45);
        assert_eq!(whole(-3.0, 120), 0);
        assert_eq!(whole(500.0, 120), 120);
    }

    #[test]
    fn inputs_sliders_and_checkboxes_are_atelier_components() {
        fn app() -> Element {
            rsx! {
                {text_field("Name", "Bannerbay".into(), |_| {})}
                {area_field("Header", "Ship it".into(), 2, |_| {})}
                {range_field("Angle", 90, 360, |_| {})}
                {slider("Line height", 1.2, (0.9, 1.8, 0.05), |_| {})}
                {check_field("Accent glow", true, |_| {})}
            }
        }
        let html = render(app);
        assert!(html.contains(r#"for="field-name""#) && html.contains(r#"id="field-name""#), "{html}");
        assert!(html.contains("input--textarea"), "{html}");
        assert_eq!(html.matches(r#"class="slider-input""#).count(), 2, "{html}");
        assert!(html.contains(r#"role="checkbox""#) && html.contains(r#"aria-checked="true""#), "{html}");
        assert!(!html.contains(r#"type="checkbox""#), "no native checkbox: {html}");
        assert!(!html.contains("<span>Name</span>"), "old hand-rolled label markup: {html}");
    }

    #[test]
    fn a_toggle_group_choice_maps_back_and_an_unpress_changes_nothing() {
        assert_eq!(chosen::<Preset>(&["square".into()]), Some(Preset::Square));
        assert_eq!(chosen::<BgKind>(&["dots".into()]), Some(BgKind::Dots));
        assert_eq!(chosen::<FeatureLayout>(&["vertical".into()]), Some(FeatureLayout::Vertical));
        assert_eq!(chosen::<Preset>(&[]), None);
        assert_eq!(chosen::<Preset>(&["huge".into()]), None);
    }

    #[test]
    fn segmented_is_a_toggle_group_with_the_full_label_as_a_hint() {
        fn app() -> Element {
            rsx! { {segmented("Size", Preset::Social, |_| {})} }
        }
        let html = render(app);
        assert!(html.contains(r#"class="toggle-group""#) && html.contains(r#"aria-label="Size""#), "{html}");
        assert_eq!(html.matches(r#"aria-pressed="true""#).count(), 1, "{html}");
        assert!(html.contains("Social / OG card (1200×630)"), "{html}");
        assert!(!html.contains("seg on"), "{html}");
    }

    #[test]
    fn style_groups_are_closed_accordions_with_a_reset() {
        fn app() -> Element {
            rsx! { {style_group(rsx! { p { "inside" } }, || {})} }
        }
        let html = render(app);
        assert!(html.contains(r#"class="accordion""#) && html.contains(r#"aria-expanded="false""#), "{html}");
        assert!(html.contains("Reset to default") && html.contains("inside"), "{html}");
        assert!(!html.contains("collapsible"), "{html}");
    }

    #[test]
    fn segmented_option_sets_are_small_with_short_distinct_labels() {
        fits::<Preset>();
        fits::<BgKind>();
        fits::<FeatureLayout>();
    }
}
