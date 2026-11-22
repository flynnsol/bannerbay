//! The top bar: brand on the left; PNG scale, downloads and the theme toggle on the right.
use dioxus::prelude::*;

use crate::export;
use crate::theme_pref::{self, ThemePref};

const BRAND_MARK: Asset = asset!("/assets/mark-dark-bg.svg");

/// A small icon for each theme preference: half-filled circle, sun, moon.
fn theme_icon(pref: ThemePref) -> Element {
    match pref {
        ThemePref::System => rsx! {
            svg { view_box: "0 0 24 24", "aria-hidden": "true",
                circle { cx: "12", cy: "12", r: "8", fill: "none", stroke: "currentColor", stroke_width: "2" }
                path { d: "M12 4a8 8 0 0 1 0 16z", fill: "currentColor" }
            }
        },
        ThemePref::Light => rsx! {
            svg { view_box: "0 0 24 24", "aria-hidden": "true",
                circle { cx: "12", cy: "12", r: "4", fill: "currentColor" }
                path {
                    d: "M12 2v2M12 20v2M2 12h2M20 12h2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4",
                    stroke: "currentColor",
                    stroke_width: "2",
                    stroke_linecap: "round",
                }
            }
        },
        ThemePref::Dark => rsx! {
            svg { view_box: "0 0 24 24", "aria-hidden": "true",
                path { d: "M20 14.5A8 8 0 1 1 9.5 4a6.5 6.5 0 0 0 10.5 10.5z", fill: "currentColor" }
            }
        },
    }
}

pub fn top_bar(stem: String, faces: String, mut png_scale: Signal<u32>, mut theme: Signal<ThemePref>) -> Element {
    let pref = theme();
    rsx! {
        header { class: "topbar",
            div { class: "brand",
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
                    "Download PNG"
                }
                button {
                    onclick: move |_| export::save_svg(&stem, &faces),
                    "Download SVG"
                }
                button {
                    class: "icon theme-toggle",
                    title: "{pref.label()} (click to change)",
                    aria_label: "{pref.label()}. Click to change.",
                    onclick: move |_| {
                        let next = theme().next();
                        theme_pref::apply(next);
                        theme.set(next);
                    },
                    {theme_icon(pref)}
                }
            }
        }
    }
}
