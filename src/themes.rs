//! One-click looks. A theme sets the background, colours and logo mark. It never touches your
//! words, size, layout, uploaded logo or style sliders, so you can switch themes freely.

use crate::model::{BgKind, Config, Mark};

pub struct Theme {
    pub name: &'static str,
    pub bg_kind: BgKind,
    pub bg1: &'static str,
    pub bg2: &'static str,
    pub angle: u32,
    pub glow: bool,
    pub text: &'static str,
    pub accent: &'static str,
    /// `None` makes emphasised header words follow the accent.
    pub highlight: Option<&'static str>,
    pub mark: Mark,
}

pub const THEMES: &[Theme] = &[
    Theme {
        name: "Midnight",
        bg_kind: BgKind::Grid,
        bg1: "#0a0d14",
        bg2: "#1b2433",
        angle: 135,
        glow: true,
        text: "#f4f1ea",
        accent: "#d6a24b",
        highlight: None,
        mark: Mark::Sail,
    },
    Theme {
        name: "Paper",
        bg_kind: BgKind::Dots,
        bg1: "#f6f1e7",
        bg2: "#e9dfcb",
        angle: 135,
        glow: false,
        text: "#1f2430",
        accent: "#b4430a",
        highlight: None,
        mark: Mark::Sail,
    },
    Theme {
        name: "Neon",
        bg_kind: BgKind::Grid,
        bg1: "#07060f",
        bg2: "#1a0b2e",
        angle: 135,
        glow: true,
        text: "#f5f3ff",
        accent: "#ff2bd6",
        highlight: Some("#22e6ff"),
        mark: Mark::Rings,
    },
    Theme {
        name: "Ocean",
        bg_kind: BgKind::Gradient,
        bg1: "#062a36",
        bg2: "#0b5a6b",
        angle: 160,
        glow: true,
        text: "#eaf7f7",
        accent: "#2dd4bf",
        highlight: None,
        mark: Mark::Sail,
    },
    Theme {
        name: "Sunset",
        bg_kind: BgKind::Gradient,
        bg1: "#2a0d3a",
        bg2: "#a3162f",
        angle: 135,
        glow: false,
        text: "#fff7ed",
        accent: "#fbbf24",
        highlight: None,
        mark: Mark::Rings,
    },
    Theme {
        name: "Forest",
        bg_kind: BgKind::Dots,
        bg1: "#0f1f17",
        bg2: "#17301f",
        angle: 135,
        glow: true,
        text: "#ecf4ea",
        accent: "#86d17a",
        highlight: None,
        mark: Mark::Blocks,
    },
];

impl Theme {
    /// True when `cfg` currently shows this theme's look (the fields `apply` sets).
    pub fn is_active(&self, cfg: &Config) -> bool {
        let gradient = self.bg_kind == BgKind::Gradient;
        cfg.bg_kind == self.bg_kind
            && cfg.bg1 == self.bg1
            && (!gradient || (cfg.bg2 == self.bg2 && cfg.angle == self.angle))
            && cfg.glow == self.glow
            && cfg.text == self.text
            && cfg.accent == self.accent
            && cfg.highlight.as_deref() == self.highlight
            // A user's uploaded logo replaces the built-in mark, whatever the theme says.
            && ((cfg.mark == Mark::Image && cfg.logo.is_some()) || cfg.mark == self.mark)
    }

    pub fn apply(&self, cfg: &mut Config) {
        cfg.bg_kind = self.bg_kind;
        cfg.bg1 = self.bg1.into();
        cfg.bg2 = self.bg2.into();
        cfg.angle = self.angle;
        cfg.glow = self.glow;
        cfg.text = self.text.into();
        cfg.accent = self.accent.into();
        cfg.highlight = self.highlight.map(String::from);
        // An uploaded logo is the user's own; a theme must not swap it for a built-in mark.
        if !(cfg.mark == Mark::Image && cfg.logo.is_some()) {
            cfg.mark = self.mark;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{FeatureLayout, Keyed, Preset};
    use crate::render::render;

    fn luminance(hex: &str) -> f64 {
        let c = |i: usize| {
            let v = u8::from_str_radix(&hex[1 + i * 2..3 + i * 2], 16).unwrap() as f64 / 255.0;
            if v <= 0.03928 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
        };
        0.2126 * c(0) + 0.7152 * c(1) + 0.0722 * c(2)
    }

    /// WCAG contrast ratio between two `#rrggbb` colours.
    fn contrast(a: &str, b: &str) -> f64 {
        let (la, lb) = (luminance(a), luminance(b));
        (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
    }

    #[test]
    fn midnight_is_the_default_look() {
        let mut cfg = Config::default();
        THEMES[0].apply(&mut cfg);
        assert_eq!(cfg, Config::default());
    }

    #[test]
    fn theme_names_are_unique() {
        let mut names: Vec<_> = THEMES.iter().map(|t| t.name).collect();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), THEMES.len());
    }

    #[test]
    fn applying_a_theme_changes_colours_but_keeps_everything_else() {
        let mut cfg = Config {
            preset: Preset::Wide,
            layout: FeatureLayout::Vertical,
            header: "Mine *only*".into(),
            features: "One\nTwo".into(),
            radius: 20,
            ..Config::default()
        };
        cfg.style.pill.radius = 3.0;
        let before = cfg.clone();
        THEMES[2].apply(&mut cfg); // Neon
        assert_eq!(cfg.accent, "#ff2bd6");
        assert_eq!(cfg.highlight.as_deref(), Some("#22e6ff"));
        assert_eq!(cfg.mark, Mark::Rings);
        assert_ne!(cfg.bg1, before.bg1);
        assert_eq!((cfg.preset, cfg.layout, cfg.radius), (before.preset, before.layout, before.radius));
        assert_eq!((&cfg.header, &cfg.features, &cfg.name), (&before.header, &before.features, &before.name));
        assert_eq!(cfg.style, before.style, "style sliders are not touched");
    }

    #[test]
    fn an_uploaded_logo_survives_a_theme() {
        let mut cfg = Config { mark: Mark::Image, logo: Some("data:image/png;base64,AAAA".into()), ..Config::default() };
        THEMES[1].apply(&mut cfg);
        assert_eq!(cfg.mark, Mark::Image);
        assert!(cfg.logo.is_some());
        // But an image mark with no image does take the theme's mark.
        let mut empty = Config { mark: Mark::Image, logo: None, ..Config::default() };
        THEMES[1].apply(&mut empty);
        assert_eq!(empty.mark, Mark::Sail);
    }

    #[test]
    fn a_theme_without_a_highlight_follows_its_accent() {
        let mut cfg = Config { highlight: Some("#123456".into()), ..Config::default() };
        THEMES[3].apply(&mut cfg); // Ocean
        assert_eq!(cfg.highlight, None);
        assert_eq!(cfg.highlight_color(), "#2dd4bf");
    }

    #[test]
    fn every_theme_is_readable() {
        for t in THEMES {
            let mut backgrounds = vec![t.bg1];
            if t.bg_kind == BgKind::Gradient {
                backgrounds.push(t.bg2);
            }
            for bg in backgrounds {
                assert!(contrast(t.text, bg) >= 4.5, "{}: text on {bg} is {:.1}", t.name, contrast(t.text, bg));
                assert!(contrast(t.accent, bg) >= 3.0, "{}: accent on {bg} is {:.1}", t.name, contrast(t.accent, bg));
                if let Some(h) = t.highlight {
                    assert!(contrast(h, bg) >= 3.0, "{}: highlight on {bg} is {:.1}", t.name, contrast(h, bg));
                }
            }
        }
    }

    #[test]
    fn every_theme_renders_the_default_content_on_every_size() {
        for t in THEMES {
            for preset in Preset::ALL {
                let mut cfg = Config { preset: *preset, ..Config::default() };
                t.apply(&mut cfg);
                let r = render(&cfg);
                assert!(r.svg.starts_with("<svg"), "{}", t.name);
                if *preset != Preset::Banner {
                    assert!(!r.overflow, "{} overflows {preset:?}", t.name);
                }
            }
        }
    }

    #[test]
    fn a_theme_is_active_right_after_it_is_applied() {
        for t in THEMES {
            let mut cfg = Config::default();
            t.apply(&mut cfg);
            assert!(t.is_active(&cfg), "{} should be active after apply", t.name);
            let others = THEMES.iter().filter(|o| o.name != t.name && o.is_active(&cfg)).count();
            assert_eq!(others, 0, "only {} should be active", t.name);
        }
    }

    #[test]
    fn default_config_shows_midnight_and_a_tweak_clears_it() {
        let mut cfg = Config::default();
        assert!(THEMES[0].is_active(&cfg));
        cfg.accent = "#123456".into();
        assert!(THEMES.iter().all(|t| !t.is_active(&cfg)));
    }

    #[test]
    fn content_and_size_do_not_affect_which_theme_is_active() {
        let mut cfg = Config { preset: Preset::Square, header: "Anything".into(), radius: 30, ..Config::default() };
        THEMES[3].apply(&mut cfg);
        assert!(THEMES[3].is_active(&cfg));
        cfg.logo = Some("data:image/png;base64,AAAA".into());
        cfg.mark = Mark::Image;
        assert!(THEMES[3].is_active(&cfg), "an uploaded logo does not count as a different theme");
    }
}
