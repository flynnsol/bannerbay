//! Swatch palette colour fields: a row of preset colours plus a custom picker.
use dioxus::prelude::*;

/// Banner backgrounds: dark and light bases that suit the themes.
pub const BACKGROUNDS: &[&str] = &["#0a0d14", "#111111", "#062a36", "#2a0d3a", "#0f1f17", "#f6f1e7", "#eef2f7", "#ffffff"];
/// Text colours: light and dark neutrals.
pub const TEXTS: &[&str] = &["#ffffff", "#f4f1ea", "#cbd5e1", "#94a3b8", "#1f2430", "#111111"];
/// Accents and highlights: one strong colour per hue.
pub const ACCENTS: &[&str] = &["#d6a24b", "#ffd400", "#b4430a", "#ff7a5e", "#ff2bd6", "#4a9bff", "#2dd4bf", "#86d17a"];

/// Which swatch, if any, is the current colour (case-insensitive). None means a custom colour.
pub fn active_swatch(value: &str, swatches: &[&str]) -> Option<usize> {
    swatches.iter().position(|s| s.eq_ignore_ascii_case(value))
}

/// Whether a value is a `#rrggbb` colour (any case), safe to put in a style attribute.
pub fn is_hex_colour(value: &str) -> bool {
    value.len() == 7 && value.starts_with('#') && value[1..].chars().all(|c| c.is_ascii_hexdigit())
}

/// The colour to write when a swatch is clicked, or None when it is already the current colour.
/// Re-picking the shown colour must not write: the Highlight field shows the accent while it
/// follows it, and writing would unlink the two.
pub fn pick(current: &str, chosen: &str) -> Option<String> {
    (!current.eq_ignore_ascii_case(chosen)).then(|| chosen.to_string())
}

/// A labelled swatch row: click a swatch to use it, or the last (rainbow) one for any colour.
/// The current colour's swatch is pressed; a colour outside the list presses Custom.
pub fn swatch_field(
    label: &'static str,
    value: String,
    swatches: &'static [&'static str],
    on_input: impl FnMut(String) + 'static,
) -> Element {
    let on_input = Callback::new(on_input);
    let active = active_swatch(&value, swatches);
    let current: std::rc::Rc<str> = value.as_str().into();
    rsx! {
        div { class: "field",
            span { "{label}" }
            div { class: "swatches", role: "group", aria_label: label,
                for (i , &colour) in swatches.iter().enumerate() {
                    button {
                        class: if active == Some(i) { "swatch on" } else { "swatch" },
                        style: "--swatch: {colour}",
                        title: "{colour}",
                        aria_label: "{label} {colour}",
                        aria_pressed: active == Some(i),
                        onclick: {
                            let current = current.clone();
                            move |_| {
                                if let Some(v) = pick(&current, colour) {
                                    on_input.call(v);
                                }
                            }
                        },
                    }
                }
                label {
                    class: if active.is_none() { "swatch custom on" } else { "swatch custom" },
                    // Saved colours are not validated on load, so only a real hex colour goes into the style.
                    style: if active.is_none() && is_hex_colour(&value) { "--swatch: {value}" } else { "" },
                    title: "Custom colour",
                    input {
                        r#type: "color",
                        aria_label: "{label}: custom colour",
                        value: "{value}",
                        oninput: move |e| {
                            if let Some(v) = pick(&current, &e.value()) {
                                on_input.call(v);
                            }
                        },
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_hex6(s: &str) -> bool {
        s.len() == 7 && s.starts_with('#') && s[1..].chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
    }

    #[test]
    fn swatches_are_valid_unique_lowercase_hex() {
        for list in [BACKGROUNDS, TEXTS, ACCENTS] {
            assert!((6..=8).contains(&list.len()), "{list:?}");
            for (i, s) in list.iter().enumerate() {
                assert!(is_hex6(s), "{s:?}");
                assert!(!list[..i].contains(s), "{s:?} repeats");
            }
        }
    }

    #[test]
    fn active_swatch_ignores_case() {
        assert_eq!(active_swatch(&ACCENTS[2].to_uppercase(), ACCENTS), Some(2));
        assert_eq!(active_swatch(ACCENTS[0], ACCENTS), Some(0));
    }

    #[test]
    fn only_hex_colours_count_as_colours() {
        for ok in ["#0a0d14", "#FFD400", "#abcdef"] {
            assert!(is_hex_colour(ok), "{ok:?}");
        }
        for bad in ["", "red", "#fff", "#12345g", "#123456; background:url(x)", "0a0d14x"] {
            assert!(!is_hex_colour(bad), "{bad:?}");
        }
    }

    #[test]
    fn picking_the_current_colour_writes_nothing() {
        assert_eq!(pick("#d6a24b", "#D6A24B"), None);
        assert_eq!(pick("#d6a24b", "#ffd400"), Some("#ffd400".to_string()));
    }

    #[test]
    fn other_colours_are_custom() {
        assert_eq!(active_swatch("#123456", ACCENTS), None);
        assert_eq!(active_swatch("", ACCENTS), None);
        assert_eq!(active_swatch("not a colour", BACKGROUNDS), None);
    }
}
