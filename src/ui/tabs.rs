//! The side panel's tabs, and how the keyboard moves between them (WAI-ARIA tabs pattern).
use dioxus::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PanelTab {
    #[default]
    Theme,
    Canvas,
    Logo,
    Text,
    Features,
}

impl PanelTab {
    pub const ALL: [PanelTab; 5] = [PanelTab::Theme, PanelTab::Canvas, PanelTab::Logo, PanelTab::Text, PanelTab::Features];

    pub fn key(self) -> &'static str {
        match self {
            PanelTab::Theme => "theme",
            PanelTab::Canvas => "canvas",
            PanelTab::Logo => "logo",
            PanelTab::Text => "text",
            PanelTab::Features => "features",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            PanelTab::Theme => "Theme",
            PanelTab::Canvas => "Canvas",
            PanelTab::Logo => "Logo",
            PanelTab::Text => "Text",
            PanelTab::Features => "Features",
        }
    }

    fn index(self) -> usize {
        Self::ALL.iter().position(|&t| t == self).unwrap_or(0)
    }

    /// Parses a stored tab; anything unknown opens Theme.
    pub fn from_key(key: &str) -> Self {
        Self::ALL.into_iter().find(|t| t.key() == key).unwrap_or_default()
    }

    /// The tab a key press moves to, or None when the key is not a tab key.
    /// Arrows move and wrap at both ends; Home and End jump to the first and last tab.
    pub fn after_key(self, key: &str) -> Option<Self> {
        let n = Self::ALL.len();
        let i = match key {
            "ArrowRight" => (self.index() + 1) % n,
            "ArrowLeft" => (self.index() + n - 1) % n,
            "Home" => 0,
            "End" => n - 1,
            _ => return None,
        };
        Some(Self::ALL[i])
    }
}

const KEY: &str = "bannerbay.tab";

/// The tab open last time in this browser.
#[cfg(target_arch = "wasm32")]
pub fn load() -> PanelTab {
    let stored = web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(KEY).ok().flatten());
    PanelTab::from_key(stored.as_deref().unwrap_or(""))
}

#[cfg(target_arch = "wasm32")]
pub fn save(tab: PanelTab) {
    if let Some(store) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        let _ = store.set_item(KEY, tab.key());
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load() -> PanelTab {
    let _ = KEY;
    PanelTab::Theme
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save(_tab: PanelTab) {}


fn select(mut tab: Signal<PanelTab>, next: PanelTab) {
    save(next);
    tab.set(next);
}

/// The tab strip. Only the selected tab is in the Tab order; arrows, Home and End move between
/// tabs and take focus with them, as the WAI-ARIA tabs pattern expects.
pub fn tab_list(tab: Signal<PanelTab>) -> Element {
    let current = tab();
    rsx! {
        div { class: "tabs", role: "tablist", aria_label: "Banner settings",
            for t in PanelTab::ALL {
                button {
                    id: "tab-{t.key()}",
                    class: if t == current { "tab on" } else { "tab" },
                    role: "tab",
                    aria_selected: t == current,
                    aria_controls: "panel-{t.key()}",
                    tabindex: if t == current { "0" } else { "-1" },
                    onclick: move |_| select(tab, t),
                    onkeydown: move |e: KeyboardEvent| {
                        // Leave modified keys alone: Alt+Arrow is browser back and forward.
                        if !e.modifiers().is_empty() {
                            return;
                        }
                        if let Some(next) = t.after_key(&e.key().to_string()) {
                            e.prevent_default();
                            select(tab, next);
                            document::eval(&format!("document.getElementById('tab-{}')?.focus()", next.key()));
                        }
                    },
                    "{t.label()}"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PanelTab::{self, *};

    #[test]
    fn keys_round_trip_and_unknown_opens_theme() {
        for tab in PanelTab::ALL {
            assert_eq!(PanelTab::from_key(tab.key()), tab);
        }
        for junk in ["", "colours", "THEME"] {
            assert_eq!(PanelTab::from_key(junk), Theme, "{junk:?}");
        }
    }

    #[test]
    fn labels_are_short_and_distinct() {
        let labels: Vec<_> = PanelTab::ALL.iter().map(|t| t.label()).collect();
        assert_eq!(labels, ["Theme", "Canvas", "Logo", "Text", "Features"]);
    }

    #[test]
    fn arrows_move_and_wrap() {
        assert_eq!(Canvas.after_key("ArrowRight"), Some(Logo));
        assert_eq!(Canvas.after_key("ArrowLeft"), Some(Theme));
        assert_eq!(Features.after_key("ArrowRight"), Some(Theme));
        assert_eq!(Theme.after_key("ArrowLeft"), Some(Features));
    }

    #[test]
    fn home_and_end_jump() {
        assert_eq!(Text.after_key("Home"), Some(Theme));
        assert_eq!(Logo.after_key("End"), Some(Features));
    }

    #[test]
    fn other_keys_are_ignored() {
        for key in ["Enter", " ", "a", "Tab", "ArrowUp", "ArrowDown"] {
            assert_eq!(Logo.after_key(key), None, "{key:?}");
        }
    }
}
