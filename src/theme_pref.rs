//! The editor's light/dark preference: follow the system, or force light or dark.
//! Only the editor chrome changes; exported banners never do.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ThemePref {
    #[default]
    System,
    Light,
    Dark,
}

impl ThemePref {
    /// The value stored in localStorage and written to `<html data-theme>` (empty for System).
    pub fn key(self) -> &'static str {
        match self {
            ThemePref::System => "",
            ThemePref::Light => "light",
            ThemePref::Dark => "dark",
        }
    }

    /// Parses a stored value; anything unknown follows the system.
    pub fn from_key(key: &str) -> Self {
        match key {
            "light" => ThemePref::Light,
            "dark" => ThemePref::Dark,
            _ => ThemePref::System,
        }
    }

    /// The next preference when the toggle is clicked: System, Light, Dark, then back.
    pub fn next(self) -> Self {
        match self {
            ThemePref::System => ThemePref::Light,
            ThemePref::Light => ThemePref::Dark,
            ThemePref::Dark => ThemePref::System,
        }
    }

    /// The toggle's label: what the editor does now.
    pub fn label(self) -> &'static str {
        match self {
            ThemePref::System => "Theme: system",
            ThemePref::Light => "Theme: light",
            ThemePref::Dark => "Theme: dark",
        }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
const KEY: &str = "bannerbay.theme";

/// The remembered preference (System when nothing is saved or storage is unavailable).
#[cfg(target_arch = "wasm32")]
pub fn load() -> ThemePref {
    let stored = web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(KEY).ok().flatten());
    ThemePref::from_key(stored.as_deref().unwrap_or(""))
}

/// Remembers the preference and applies it to `<html data-theme>`; System removes the attribute
/// so the CSS follows `prefers-color-scheme`.
#[cfg(target_arch = "wasm32")]
pub fn apply(pref: ThemePref) {
    let Some(window) = web_sys::window() else { return };
    if let Ok(Some(store)) = window.local_storage() {
        let _ = match pref {
            ThemePref::System => store.remove_item(KEY),
            _ => store.set_item(KEY, pref.key()),
        };
    }
    if let Some(root) = window.document().and_then(|d| d.document_element()) {
        let _ = match pref {
            ThemePref::System => root.remove_attribute("data-theme"),
            _ => root.set_attribute("data-theme", pref.key()),
        };
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load() -> ThemePref {
    ThemePref::System
}

#[cfg(not(target_arch = "wasm32"))]
pub fn apply(_pref: ThemePref) {}

#[cfg(test)]
mod tests {
    use super::ThemePref::{self, *};

    #[test]
    fn keys_round_trip() {
        for pref in [System, Light, Dark] {
            assert_eq!(ThemePref::from_key(pref.key()), pref);
        }
        assert_eq!(Light.key(), "light");
        assert_eq!(Dark.key(), "dark");
        assert_eq!(System.key(), "");
    }

    #[test]
    fn unknown_values_follow_the_system() {
        for junk in ["", "sepia", "LIGHT ", "null"] {
            assert_eq!(ThemePref::from_key(junk), System, "{junk:?}");
        }
    }

    #[test]
    fn clicking_cycles_system_light_dark() {
        assert_eq!(System.next(), Light);
        assert_eq!(Light.next(), Dark);
        assert_eq!(Dark.next(), System);
    }
}
