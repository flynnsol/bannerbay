//! The top bar: brand, the export actions (a sticky bottom bar on phones) and the theme toggle.
use dioxus::prelude::*;

use crate::export;
use atelier_ui::elements::ThemeToggle;

const BRAND_MARK: Asset = asset!("/assets/mark-dark-bg.svg");

pub fn top_bar(stem: String, faces: String, mut png_scale: Signal<u32>) -> Element {
    rsx! {
        header { class: "topbar",
            div { class: "topbar-brand",
                img { class: "brand-mark", src: BRAND_MARK, alt: "" }
                h1 { "Bannerbay" }
                p { class: "tagline", "Banners and social cards, made in your browser." }
            }
            div { class: "actions",
                div { class: "segmented", role: "group", aria_label: "PNG scale",
                    for scale in [1u32, 2] {
                        button {
                            class: if png_scale() == scale { "seg on" } else { "seg" },
                            aria_pressed: png_scale() == scale,
                            onclick: move |_| png_scale.set(scale),
                            "{scale}×"
                        }
                    }
                }
                button {
                    class: "primary",
                    onclick: {
                        let (stem, faces) = (stem.clone(), faces.clone());
                        move |_| export::save_png(&stem, png_scale(), &faces)
                    },
                    aria_label: "Download PNG",
                    span { class: "wide-only", "Download " }
                    "PNG"
                }
                button {
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
    }
}
