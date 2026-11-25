//! Prepares atelier-ui's theme before the app starts: the editor always shows the Ember Night
//! palette, and a light/dark choice saved by the old Bannerbay toggle carries over.
//! `ThemeProvider` reads both from localStorage on its first run, so they are written first.
//! Only the editor chrome changes; exported banners never do.

use atelier_ui::theme::{Palette, Theme};
#[cfg(target_arch = "wasm32")]
use atelier_ui::theme::{PALETTE_STORAGE_KEY, STORAGE_KEY};

/// The one palette the editor uses.
pub const PALETTE: Palette = Palette::EmberNight;

/// Where the Bannerbay toggle used to keep "light" or "dark" (absent meant follow the system).
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
const OLD_KEY: &str = "bannerbay.theme";

/// The value to write to atelier-ui's theme key, given the old Bannerbay value and atelier-ui's
/// own. An existing atelier-ui choice wins; an old light or dark choice is carried over; anything
/// else writes nothing, so the editor follows the system.
pub fn migrated_theme(old: Option<&str>, current: Option<&str>) -> Option<&'static str> {
    if current.is_some() {
        return None;
    }
    match Theme::parse(old?) {
        Theme::System => None,
        theme => Some(theme.as_str()),
    }
}

/// Writes the Ember Night palette and any carried-over theme to localStorage, then removes the
/// old key. Storage may be missing (private windows); then the defaults apply.
#[cfg(target_arch = "wasm32")]
pub fn prepare() {
    let Some(window) = web_sys::window() else { return };
    let Ok(Some(store)) = window.local_storage() else { return };
    let old = store.get_item(OLD_KEY).ok().flatten();
    let current = store.get_item(STORAGE_KEY).ok().flatten();
    if let Some(theme) = migrated_theme(old.as_deref(), current.as_deref()) {
        let _ = store.set_item(STORAGE_KEY, theme);
    }
    if old.is_some() {
        let _ = store.remove_item(OLD_KEY);
    }
    let _ = store.set_item(PALETTE_STORAGE_KEY, PALETTE.as_str());
}

#[cfg(not(target_arch = "wasm32"))]
pub fn prepare() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_old_light_or_dark_choice_carries_over() {
        assert_eq!(migrated_theme(Some("light"), None), Some("light"));
        assert_eq!(migrated_theme(Some("dark"), None), Some("dark"));
    }

    #[test]
    fn an_existing_atelier_choice_wins() {
        assert_eq!(migrated_theme(Some("dark"), Some("light")), None);
        assert_eq!(migrated_theme(None, Some("dark")), None);
    }

    #[test]
    fn nothing_or_junk_follows_the_system() {
        for old in [None, Some(""), Some("sepia"), Some("system")] {
            assert_eq!(migrated_theme(old, None), None, "{old:?}");
        }
    }

    #[test]
    fn the_palette_is_ember_night() {
        assert_eq!(PALETTE.as_str(), "ember-night");
    }

    const TOKENS: &str = include_str!("../assets/tokens.css");

    #[test]
    fn tokens_leave_the_palette_to_atelier_ui() {
        for name in ["--color-bg:", "--color-text:", "--color-accent:", "--color-surface:"] {
            assert!(!TOKENS.contains(name), "tokens.css still sets {name}");
        }
    }

    #[test]
    fn tokens_do_not_redefine_library_tokens_with_another_meaning() {
        // atelier-ui uses --blur-glass as a length inside blur(); a filter list there breaks it.
        assert!(!TOKENS.contains("--blur-glass:"));
    }

    /// Class names main.css styles that atelier-ui's stylesheet also styles, so its rules leak into
    /// ours (its `.panel` has `margin: 60vh auto`). Listed ones are replaced by library components
    /// in later tickets; anything else is a new clash.
    #[test]
    fn main_css_classes_do_not_clash_with_atelier_ui() {
        const PENDING: [&str; 6] = ["tab", "tabs", "field", "open", "active", "theme-toggle"];
        // Library classes main.css names on purpose: to keep its generic button rules off them, or
        // to lay library components out in the panel.
        const EXCLUDED: [&str; 5] = ["btn", "input", "checkbox-box", "toggle", "toggle-group"];
        let library = atelier_ui::styles::COMPONENTS_SRC;
        let ours = include_str!("../assets/main.css");
        let names = |css: &str| -> std::collections::BTreeSet<String> {
            let mut code = String::new();
            for (i, part) in css.split("/*").enumerate() {
                code.push_str(if i == 0 { part } else { part.split_once("*/").map_or("", |(_, after)| after) });
            }
            code.split('.')
                .skip(1)
                .map(|rest| rest.chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '-').collect::<String>())
                .filter(|n| n.starts_with(|c: char| c.is_ascii_alphabetic()))
                .collect::<std::collections::BTreeSet<_>>()
        };
        let library = names(library);
        let clashes: Vec<_> = names(ours)
            .into_iter()
            .filter(|n| library.contains(n) && !PENDING.contains(&n.as_str()) && !EXCLUDED.contains(&n.as_str()))
            .collect();
        assert!(clashes.is_empty(), "main.css classes also styled by atelier-ui: {clashes:?}");
    }
}
