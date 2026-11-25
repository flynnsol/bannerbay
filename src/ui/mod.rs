use dioxus::prelude::*;

use crate::export;
use crate::model::*;
use crate::measure;
use crate::render::{render_with_metrics, Metrics};
use crate::storage::{self, Saved};

mod fields;
mod swatches;
mod tabs;
mod theme_gallery;
mod topbar;

use fields::*;
use atelier_ui::widgets::{Accordion, TabPanel, Tabs};
use tabs::PanelTab;
use swatches::{swatch_field, ACCENTS, BACKGROUNDS, TEXTS};
use theme_gallery::theme_gallery;
use topbar::top_bar;

#[component]
pub fn App() -> Element {
    // Restore the last session synchronously, so there is no flash of the defaults.
    let mut cfg = use_signal(|| storage::load().unwrap_or_default());
    let mut save_state = use_signal(|| Saved::Full);
    let mut confirm_reset = use_signal(|| false);
    // Bumped when the bundled fonts in use have finished loading, to re-measure and re-render.
    let mut font_tick = use_signal(|| 0u32);
    let font_keys = use_memo(move || (cfg.read().heading_font.clone(), cfg.read().body_font.clone()));
    use_effect(move || {
        let (heading, body) = font_keys();
        if let Some(script) = measure::load_script(&heading, &body) {
            spawn(async move {
                let mut eval = document::eval(&script);
                if eval.recv::<bool>().await.is_ok() {
                    font_tick += 1;
                }
            });
        }
    });
    // Autosave whenever any setting changes.
    use_effect(move || {
        let c = cfg();
        save_state.set(storage::save(&c));
    });
    let png_scale = use_signal(|| 2u32);
    // The tab open last time; read once, as the first tab.
    let first_tab = use_hook(|| tabs::load().index());
    let c = cfg();
    // Real text widths, once the fonts in use have loaded. Until then (or off the web) the estimate
    // is used, and a re-render follows as soon as `font_tick` changes.
    let _ = font_tick();
    let measure_text = |text: &str, font: &FontInfo, weight: u32, size: f64| measure::width(text, font, weight, size);
    let metrics = if measure::available() { Metrics::measured(&measure_text) } else { Metrics::ESTIMATE };
    let rendered = render_with_metrics(&c, "bb", &metrics);
    let stem = c.file_stem();
    let faces = export::faces_json(&c.heading_font, &c.body_font);
    let (w, h) = c.preset.size();

    rsx! {
        div { class: "app",
            {top_bar(stem, faces, png_scale)}
            aside {
                class: "editor-panel",
                // Tabs reports no changes, so remember the selected tab after any click or key.
                onclick: move |_| tabs::remember(),
                onkeyup: move |_| tabs::remember(),
                Tabs { labels: PanelTab::labels(), initial: first_tab,
                TabPanel { index: 0,
                    section {
                        {theme_gallery(&c, &metrics, cfg)}
                    }
                }
                TabPanel { index: 1,
                    section {
                        {segmented("Size", c.preset, move |v| cfg.write().preset = v)}
                        {segmented("Background", c.bg_kind, move |v| cfg.write().bg_kind = v)}
                        {swatch_field("Base", c.bg1.clone(), BACKGROUNDS, move |v| cfg.write().bg1 = v)}
                        if c.bg_kind == BgKind::Gradient {
                            {swatch_field("To", c.bg2.clone(), BACKGROUNDS, move |v| cfg.write().bg2 = v)}
                        }
                        {swatch_field("Text", c.text.clone(), TEXTS, move |v| cfg.write().text = v)}
                        {swatch_field("Accent", c.accent.clone(), ACCENTS, move |v| cfg.write().accent = v)}
                        if c.bg_kind == BgKind::Gradient {
                            {range_field("Angle", c.angle, 360, move |v| cfg.write().angle = v)}
                        }
                        {range_field("Corner radius", c.radius, 120, move |v| cfg.write().radius = v)}
                        {check_field("Accent glow", c.glow, move |v| cfg.write().glow = v)}
                        Accordion { title: "Fonts", initially_open: true,
                            {font_select("Heading font", c.heading_font.clone(), move |v| cfg.write().heading_font = v)}
                            {font_select("Body font", c.body_font.clone(), move |v| cfg.write().body_font = v)}
                            if c.heading_font != DEFAULT_FONT || c.body_font != DEFAULT_FONT {
                                button {
                                    class: "small reset-fonts",
                                    onclick: move |_| {
                                        let mut cfg = cfg.write();
                                        cfg.heading_font = DEFAULT_FONT.into();
                                        cfg.body_font = DEFAULT_FONT.into();
                                    },
                                    "Use default fonts"
                                }
                            }
                        }
                    }
                }

                TabPanel { index: 2,
                    section {
                        {select_field("Mark", c.mark, move |v| cfg.write().mark = v)}
                        label { class: "field",
                            span { "Upload your own (PNG, SVG, JPG)" }
                            input {
                                id: "logo-file",
                                r#type: "file",
                                accept: "image/*",
                                onchange: move |_| {
                                    spawn(async move {
                                        let logo = export::read_logo().await;
                                        let mut cfg = cfg.write();
                                        if logo.is_some() {
                                            cfg.mark = Mark::Image;
                                        }
                                        cfg.logo = logo;
                                    });
                                },
                            }
                        }
                        {style_group(
                            rsx! {
                                {slider("Mark size", c.style.mark.size, (16.0, 120.0, 1.0), move |v| cfg.write().style.mark.size = v)}
                                {slider("Mark to name gap", c.style.mark.gap, (0.0, 60.0, 1.0), move |v| cfg.write().style.mark.gap = v)}
                                {slider("Name size", c.style.mark.name_size, (14.0, 72.0, 1.0), move |v| cfg.write().style.mark.name_size = v)}
                                {slider("Name weight", c.style.mark.name_weight, (400.0, 900.0, 100.0), move |v| cfg.write().style.mark.name_weight = v)}
                                {slider("Space below", c.style.mark.space_after, (0.0, 120.0, 1.0), move |v| cfg.write().style.mark.space_after = v)}
                            },
                            move || cfg.write().style.mark = MarkStyle::default(),
                        )}
                    }
                }

                TabPanel { index: 3,
                    section {
                        {text_field("Name", c.name.clone(), move |v| cfg.write().name = v)}
                        {text_field("Tag", c.tag.clone(), move |v| cfg.write().tag = v)}
                        {style_group(
                            rsx! {
                                {slider("Text size", c.style.tag.size, (10.0, 40.0, 1.0), move |v| cfg.write().style.tag.size = v)}
                                {slider("Letter spacing", c.style.tag.letter_spacing, (0.0, 12.0, 0.5), move |v| cfg.write().style.tag.letter_spacing = v)}
                                {slider("Space after", c.style.tag.space_after, (0.0, 80.0, 1.0), move |v| cfg.write().style.tag.space_after = v)}
                            },
                            move || cfg.write().style.tag = TagStyle::default(),
                        )}
                        {area_field("Header", c.header.clone(), 2, move |v| cfg.write().header = v)}
                        p { class: "hint", "Wrap words in *asterisks* to highlight them. Use \\* for a literal asterisk." }
                        {style_group(
                            rsx! {
                                div { class: "highlight-row",
                                    {swatch_field("Highlight colour", c.highlight_color().to_string(), ACCENTS, move |v| cfg.write().highlight = Some(v))}
                                    if c.highlight.is_some() {
                                        button { class: "small", onclick: move |_| cfg.write().highlight = None, "Use accent" }
                                    }
                                }
                                {slider("Text size", c.style.header.size, (20.0, 120.0, 1.0), move |v| cfg.write().style.header.size = v)}
                                {slider("Weight", c.style.header.weight, (400.0, 900.0, 100.0), move |v| cfg.write().style.header.weight = v)}
                                {slider("Line height", c.style.header.line_height, (0.9, 1.8, 0.05), move |v| cfg.write().style.header.line_height = v)}
                                {slider("Space after", c.style.header.space_after, (0.0, 100.0, 1.0), move |v| cfg.write().style.header.space_after = v)}
                            },
                            move || {
                                let mut cfg = cfg.write();
                                cfg.style.header = HeaderStyle::default();
                                cfg.highlight = None;
                            },
                        )}
                        {area_field("Description", c.description.clone(), 4, move |v| cfg.write().description = v)}
                        {style_group(
                            rsx! {
                                {slider("Text size", c.style.desc.size, (12.0, 60.0, 1.0), move |v| cfg.write().style.desc.size = v)}
                                {slider("Weight", c.style.desc.weight, (300.0, 900.0, 100.0), move |v| cfg.write().style.desc.weight = v)}
                                {slider("Line height", c.style.desc.line_height, (1.0, 2.4, 0.05), move |v| cfg.write().style.desc.line_height = v)}
                                {slider("Opacity", c.style.desc.opacity, (0.2, 1.0, 0.01), move |v| cfg.write().style.desc.opacity = v)}
                                {slider("Space after", c.style.desc.space_after, (0.0, 120.0, 1.0), move |v| cfg.write().style.desc.space_after = v)}
                            },
                            move || cfg.write().style.desc = DescStyle::default(),
                        )}
                        {text_field("Badge (top right)", c.badge.clone(), move |v| cfg.write().badge = v)}
                        {style_group(
                            rsx! {
                                {slider("Corner radius", c.style.badge.pill.radius, (0.0, 40.0, 1.0), move |v| cfg.write().style.badge.pill.radius = v)}
                                {slider("Text size", c.style.badge.pill.font_size, (10.0, 32.0, 1.0), move |v| cfg.write().style.badge.pill.font_size = v)}
                                {slider("Padding", c.style.badge.pill.pad_x, (6.0, 40.0, 1.0), move |v| cfg.write().style.badge.pill.pad_x = v)}
                                {slider("Height", c.style.badge.pill.height, (20.0, 70.0, 1.0), move |v| cfg.write().style.badge.pill.height = v)}
                                {slider("Border", c.style.badge.pill.border, (0.0, 6.0, 0.5), move |v| cfg.write().style.badge.pill.border = v)}
                                {slider("Fill", c.style.badge.pill.fill, (0.0, 0.5, 0.01), move |v| cfg.write().style.badge.pill.fill = v)}
                                {slider("Offset from right", c.style.badge.offset_x, (-40.0, 80.0, 1.0), move |v| cfg.write().style.badge.offset_x = v)}
                                {slider("Offset from top", c.style.badge.offset_y, (-40.0, 80.0, 1.0), move |v| cfg.write().style.badge.offset_y = v)}
                            },
                            move || cfg.write().style.badge = BadgeStyle::default(),
                        )}
                    }
                }

                TabPanel { index: 4,
                    section {
                        {area_field("One per line", c.features.clone(), 4, move |v| cfg.write().features = v)}
                        {segmented("Layout", c.layout, move |v| cfg.write().layout = v)}
                        {style_group(
                            rsx! {
                                {slider("Corner radius", c.style.pill.radius, (0.0, 40.0, 1.0), move |v| cfg.write().style.pill.radius = v)}
                                {slider("Text size", c.style.pill.font_size, (12.0, 36.0, 1.0), move |v| cfg.write().style.pill.font_size = v)}
                                {slider("Padding", c.style.pill.pad_x, (6.0, 40.0, 1.0), move |v| cfg.write().style.pill.pad_x = v)}
                                {slider("Height", c.style.pill.height, (24.0, 80.0, 1.0), move |v| cfg.write().style.pill.height = v)}
                                {slider("Gap", c.style.pill.gap, (4.0, 40.0, 1.0), move |v| cfg.write().style.pill.gap = v)}
                                {slider("Border", c.style.pill.border, (0.0, 6.0, 0.5), move |v| cfg.write().style.pill.border = v)}
                                {slider("Fill", c.style.pill.fill, (0.0, 0.4, 0.01), move |v| cfg.write().style.pill.fill = v)}
                            },
                            move || cfg.write().style.pill = PillStyle::default(),
                        )}
                    }
                }
                }
                section { class: "saved-note",
                    h2 { "Saved in this browser" }
                    p { class: "hint", "Your banner is saved automatically and never leaves this browser." }
                    button {
                        class: if confirm_reset() { "reset danger" } else { "reset" },
                        onmouseleave: move |_| confirm_reset.set(false),
                        onclick: move |_| {
                            if confirm_reset() {
                                cfg.set(Config::default());
                                confirm_reset.set(false);
                            } else {
                                confirm_reset.set(true);
                            }
                        },
                        if confirm_reset() { "Click again to erase everything" } else { "Reset everything" }
                    }
                }
            }

            main { class: "stage",
                div { class: "toolbar",
                    span { class: "dims", "{w} × {h}" }
                    match save_state() {
                        Saved::Full => rsx! {},
                        Saved::WithoutLogo => rsx! {
                            span { class: "warn", "The logo is too big to autosave, so it will be gone after a reload." }
                        },
                        Saved::Failed => rsx! {
                            span { class: "warn", "Autosave is unavailable in this browser." }
                        },
                    }
                    if rendered.overflow {
                        span { class: "warn", "Content runs past the bottom, so shorten the text or pick a taller size." }
                    }
                }
                div { class: "checker",
                    div { id: "preview", dangerous_inner_html: "{rendered.svg}" }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_tab_panel_stays_mounted_so_open_sections_survive_a_tab_switch() {
        let mut dom = VirtualDom::new(|| rsx! { atelier_ui::theme::ThemeProvider { App {} } });
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert_eq!(html.matches(r#"role="tabpanel""#).count(), PanelTab::ALL.len(), "all panels are in the page");
        assert_eq!(html.matches(r#"class="tab-panel active""#).count(), 1, "one is shown");
        assert_eq!(html.matches(r#"role="tab""#).count(), PanelTab::ALL.len());
        assert!(html.contains(r#"class="accordion open""#), "Fonts starts open: {html}");
    }
}
