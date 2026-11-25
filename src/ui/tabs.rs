//! The side panel's tabs. atelier-ui's `Tabs` draws them and handles the keyboard; this module
//! names them and remembers the open one in this browser.
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

    /// The tab with this visible label.
    pub fn from_label(label: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|t| t.label() == label.trim())
    }

    /// The labels for atelier-ui `Tabs`, in order.
    pub fn labels() -> Vec<String> {
        Self::ALL.iter().map(|t| t.label().to_string()).collect()
    }

    pub fn index(self) -> usize {
        Self::ALL.iter().position(|&t| t == self).unwrap_or(0)
    }

    /// Parses a stored tab; anything unknown opens Theme.
    pub fn from_key(key: &str) -> Self {
        Self::ALL.into_iter().find(|t| t.key() == key).unwrap_or_default()
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


/// Remembers whichever tab is selected now. atelier-ui's `Tabs` reports no changes, so the panel
/// calls this after a click or key press and it reads the selected tab once the page has updated.
pub fn remember() {
    spawn(async move {
        let mut eval = document::eval(
            "requestAnimationFrame(() => setTimeout(() => dioxus.send(\
             document.querySelector('.editor-panel [role=tab][aria-selected=true]')?.textContent ?? '')))",
        );
        if let Ok(label) = eval.recv::<String>().await {
            if let Some(tab) = PanelTab::from_label(&label) {
                save(tab);
            }
        }
    });
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
    fn labels_map_back_to_tabs() {
        for tab in PanelTab::ALL {
            assert_eq!(PanelTab::from_label(tab.label()), Some(tab));
        }
        assert_eq!(PanelTab::from_label(" Logo "), Some(Logo), "whitespace around textContent");
        assert_eq!(PanelTab::from_label(""), None);
        assert_eq!(PanelTab::labels(), ["Theme", "Canvas", "Logo", "Text", "Features"]);
        assert_eq!(Text.index(), 3);
    }
}
