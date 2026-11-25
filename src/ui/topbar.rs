//! The top bar: brand, the export actions (a sticky bottom bar on phones) and the theme toggle.
use dioxus::prelude::*;

use crate::export;
use atelier_ui::elements::{Button, ButtonVariant, SelectOption, ThemeToggle, ToggleGroup};

const BRAND_MARK: Asset = asset!("/assets/mark-dark-bg.svg");

/// The PNG scale a ToggleGroup reports: 1 or 2, or nothing (an unpress or anything else).
fn scale_choice(values: &[String]) -> Option<u32> {
    values.first().and_then(|v| v.parse().ok()).filter(|s| matches!(s, 1 | 2))
}

pub fn top_bar(stem: String, faces: String, mut png_scale: Signal<u32>) -> Element {
    rsx! {
        header { class: "topbar",
            div { class: "topbar-brand",
                img { class: "brand-mark", src: BRAND_MARK, alt: "" }
                h1 { "Bannerbay" }
                p { class: "tagline", "Banners and social cards, made in your browser." }
            }
            div { class: "actions",
                ToggleGroup {
                    label: "PNG scale",
                    options: vec![SelectOption::new("1", "1×"), SelectOption::new("2", "2×")],
                    value: vec![png_scale().to_string()],
                    onchange: move |values: Vec<String>| {
                        if let Some(scale) = scale_choice(&values) {
                            png_scale.set(scale)
                        }
                    },
                }
                Button {
                    variant: ButtonVariant::Primary,
                    class: "btn--shine",
                    onclick: {
                        let (stem, faces) = (stem.clone(), faces.clone());
                        move |_| export::save_png(&stem, png_scale(), &faces)
                    },
                    aria_label: "Download PNG",
                    span { class: "wide-only", "Download " }
                    "PNG"
                }
                Button {
                    onclick: move |_| export::save_svg(&stem, &faces),
                    aria_label: "Download SVG",
                    span { class: "wide-only", "Download " }
                    "SVG"
                }
            }
            ThemeToggle {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use atelier_ui::theme::ThemeProvider;

    #[test]
    fn the_png_scale_is_one_or_two() {
        assert_eq!(scale_choice(&["1".into()]), Some(1));
        assert_eq!(scale_choice(&["2".into()]), Some(2));
        assert_eq!(scale_choice(&[]), None);
        assert_eq!(scale_choice(&["3".into()]), None);
    }

    #[test]
    fn the_theme_toggle_is_the_atelier_ui_one() {
        fn app() -> Element {
            let scale = use_signal(|| 2u32);
            rsx! { ThemeProvider { {top_bar("b".into(), "[]".into(), scale)} } }
        }
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains(r#"class="btn theme-toggle""#), "{html}");
        assert!(!html.contains("icon theme-toggle"), "the hand-rolled toggle is gone");
        assert!(html.contains(r#"aria-label="PNG scale""#) && html.contains(r#"class="toggle""#), "{html}");
        assert!(html.contains(r#"class="btn btn--primary btn--shine""#), "PNG is the library primary button: {html}");
        assert!(html.contains(r#"<button class="btn" type="button" aria-label="Download SVG""#), "{html}");
    }
}
