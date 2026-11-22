//! The theme gallery: your own banner drawn in every theme, one click to apply.
use dioxus::prelude::*;

use crate::model::Config;
use crate::render::{render_with_metrics, Metrics};
use crate::themes::THEMES;

pub fn theme_gallery(c: &Config, metrics: &Metrics, mut cfg: Signal<Config>) -> Element {
    // The uploaded logo is left out so six copies of a large image never end up in the page.
    let mut base = c.clone();
    base.logo = None;
    let thumbs = THEMES.iter().enumerate().map(|(i, theme)| {
        let mut themed = base.clone();
        theme.apply(&mut themed);
        // A prefix per thumbnail: ids are page-global and must not clash with the preview.
        (theme, render_with_metrics(&themed, &format!("th{i}"), metrics).svg)
    });
    rsx! {
        div { class: "field",
            span { "Theme" }
            div { class: "themes",
                for (theme , thumb) in thumbs {
                    button {
                        class: if theme.is_active(c) { "theme-card active" } else { "theme-card" },
                        title: "Apply the {theme.name} theme",
                        aria_pressed: theme.is_active(c),
                        onclick: move |_| theme.apply(&mut cfg.write()),
                        div { class: "theme-thumb", dangerous_inner_html: "{thumb}" }
                        span { class: "theme-name", "{theme.name}" }
                    }
                }
            }
        }
    }
}
