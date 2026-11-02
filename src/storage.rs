//! Autosave to the browser's localStorage.
//!
//! Settings are kept as small versioned JSON and never leave the browser (no cookies, no
//! server). The JSON handling is plain Rust so it is tested natively; only the few functions
//! that touch `window.localStorage` are browser-specific.

use crate::model::{Config, Mark};
use serde::{Deserialize, Serialize};

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
const KEY: &str = "bannerbay:settings";
/// Bump when a saved setting changes meaning. Missing fields do not need a bump: they fall back
/// to their defaults.
const VERSION: u32 = 1;

#[derive(Serialize)]
struct Save<'a> {
    v: u32,
    config: &'a Config,
}

#[derive(Deserialize)]
struct Loaded {
    v: u32,
    config: Config,
}

/// What happened when saving. The failure cases only arise in the browser.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Saved {
    Full,
    /// The browser refused the full save, so the uploaded logo was left out.
    WithoutLogo,
    Failed,
}

pub fn to_json(cfg: &Config) -> String {
    serde_json::to_string(&Save { v: VERSION, config: cfg }).unwrap_or_default()
}

/// `None` for unreadable data or data from a newer version of the app.
pub fn from_json(json: &str) -> Option<Config> {
    let loaded: Loaded = serde_json::from_str(json).ok()?;
    (loaded.v <= VERSION).then_some(loaded.config)
}

/// The same settings without the uploaded logo (which can be megabytes).
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub fn without_logo(cfg: &Config) -> Config {
    let mut c = cfg.clone();
    c.logo = None;
    if c.mark == Mark::Image {
        c.mark = Config::default().mark;
    }
    c
}

#[cfg(target_arch = "wasm32")]
fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

/// The saved settings, if there are any and they can be read.
#[cfg(target_arch = "wasm32")]
pub fn load() -> Option<Config> {
    from_json(&storage()?.get_item(KEY).ok()??)
}

#[cfg(target_arch = "wasm32")]
pub fn save(cfg: &Config) -> Saved {
    let Some(store) = storage() else { return Saved::Failed };
    if store.set_item(KEY, &to_json(cfg)).is_ok() {
        return Saved::Full;
    }
    if cfg.logo.is_some() && store.set_item(KEY, &to_json(&without_logo(cfg))).is_ok() {
        return Saved::WithoutLogo;
    }
    Saved::Failed
}

// Outside the browser (native tests) there is nothing to read or write.
#[cfg(not(target_arch = "wasm32"))]
pub fn load() -> Option<Config> {
    None
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save(_cfg: &Config) -> Saved {
    Saved::Full
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;

    fn customised() -> Config {
        let mut c = Config {
            preset: Preset::Wide,
            bg_kind: BgKind::Gradient,
            layout: FeatureLayout::Vertical,
            mark: Mark::Rings,
            header: "Make *it* pop".into(),
            highlight: Some("#ff3366".into()),
            logo: Some("data:image/png;base64,AAAA".into()),
            ..Config::default()
        };
        c.style.pill.radius = 4.0;
        c.style.badge.offset_x = 12.0;
        c.style.mark.name_weight = 900.0;
        c.style.header.line_height = 1.4;
        c.style.desc.opacity = 0.5;
        c.style.tag.letter_spacing = 5.0;
        c
    }

    #[test]
    fn default_and_customised_settings_round_trip() {
        for cfg in [Config::default(), customised()] {
            assert_eq!(from_json(&to_json(&cfg)), Some(cfg));
        }
    }

    #[test]
    fn saved_json_is_versioned() {
        let v: serde_json::Value = serde_json::from_str(&to_json(&Config::default())).unwrap();
        assert_eq!(v["v"], VERSION);
        assert!(v["config"]["header"].is_string());
    }

    #[test]
    fn missing_fields_fall_back_to_defaults() {
        // An older save that lacks every style, the highlight and the layout.
        let old = r##"{"v":1,"config":{"name":"Old","accent":"#112233"}}"##;
        let cfg = from_json(old).expect("older saves still load");
        assert_eq!(cfg.name, "Old");
        assert_eq!(cfg.accent, "#112233");
        assert_eq!(cfg.style, Style::default());
        assert_eq!(cfg.layout, Config::default().layout);
        assert_eq!(cfg.highlight, None);
    }

    #[test]
    fn nested_style_with_missing_fields_keeps_the_rest() {
        let old = r#"{"v":1,"config":{"style":{"pill":{"radius":3.0}}}}"#;
        let cfg = from_json(old).unwrap();
        assert_eq!(cfg.style.pill.radius, 3.0);
        assert_eq!(cfg.style.pill.height, PillStyle::default().height);
        assert_eq!(cfg.style.badge, BadgeStyle::default());
    }

    #[test]
    fn unreadable_or_newer_data_is_ignored() {
        assert_eq!(from_json(""), None);
        assert_eq!(from_json("not json"), None);
        assert_eq!(from_json(r#"{"v":1}"#), None);
        assert_eq!(from_json(r#"{"v":1,"config":{"preset":"no_such_size"}}"#), None);
        assert_eq!(from_json(&format!(r#"{{"v":{},"config":{{}}}}"#, VERSION + 1)), None);
    }

    #[test]
    fn dropping_the_logo_keeps_everything_else() {
        let mut cfg = customised();
        cfg.mark = Mark::Image;
        let slim = without_logo(&cfg);
        assert_eq!(slim.logo, None);
        assert_ne!(slim.mark, Mark::Image, "an image mark with no image would draw nothing");
        assert_eq!(slim.header, cfg.header);
        assert_eq!(slim.style, cfg.style);
        assert!(to_json(&slim).len() < to_json(&cfg).len());
    }

    #[test]
    fn markup_in_text_survives_a_round_trip_unchanged() {
        let cfg = Config { header: "</text><script>\"x\"</script> *a* \\* ünï".into(), ..Config::default() };
        assert_eq!(from_json(&to_json(&cfg)).unwrap().header, cfg.header);
    }
}
