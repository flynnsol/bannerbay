//! Real text widths, measured with a hidden canvas in the browser.
//!
//! The renderer is plain Rust and takes widths through `render::Metrics`. In the browser this
//! module supplies exact widths for the font, weight and size being drawn (so capitals, bold text
//! and unusual fonts all fit their labels). Everywhere else, including native tests, widths fall
//! back to the per-font estimate.

use crate::model::FontInfo;

/// True where real measuring is available (the browser).
pub fn available() -> bool {
    cfg!(target_arch = "wasm32")
}

/// Width in pixels of `text` set in `font` at `weight` and `size`; the estimate if measuring is
/// unavailable or fails.
pub fn width(text: &str, font: &FontInfo, weight: u32, size: f64) -> f64 {
    imp::measure(text, font, weight, size).unwrap_or_else(|| text.chars().count() as f64 * size * font.char_width)
}

/// JavaScript that loads the bundled fonts in use, so the page can measure and draw them. `None`
/// when only system fonts are in use. It sends `true` back to Rust when loading has finished (or
/// failed: the app then simply measures the fallback font).
pub fn load_script(heading: &str, body: &str) -> Option<String> {
    let mut families: Vec<&str> = [heading, body]
        .into_iter()
        .map(|key| crate::model::font_info(key).family)
        .filter(|family| !family.is_empty())
        .collect();
    families.dedup();
    if families.is_empty() {
        return None;
    }
    let list = serde_json::to_string(&families).ok()?;
    Some(format!(
        r#"(async () => {{
  try {{
    const families = {list};
    await Promise.all(families.flatMap(f => [400, 500, 700, 800].map(w => document.fonts.load(w + " 20px '" + f + "'"))));
    await document.fonts.ready;
  }} catch (e) {{
    console.warn('Bannerbay: could not load fonts for measuring', e);
  }}
  dioxus.send(true);
}})();"#
    ))
}

#[cfg(target_arch = "wasm32")]
mod imp {
    use super::FontInfo;
    use std::cell::RefCell;
    use web_sys::wasm_bindgen::JsCast;
    use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

    thread_local! {
        static CONTEXT: RefCell<Option<CanvasRenderingContext2d>> = const { RefCell::new(None) };
    }

    fn make_context() -> Option<CanvasRenderingContext2d> {
        let canvas: HtmlCanvasElement =
            web_sys::window()?.document()?.create_element("canvas").ok()?.dyn_into().ok()?;
        canvas.get_context("2d").ok()??.dyn_into().ok()
    }

    pub fn measure(text: &str, font: &FontInfo, weight: u32, size: f64) -> Option<f64> {
        CONTEXT.with(|slot| {
            let mut slot = slot.borrow_mut();
            if slot.is_none() {
                *slot = make_context();
            }
            let ctx = slot.as_ref()?;
            ctx.set_font(&format!("{weight} {size}px {}", font.stack));
            ctx.measure_text(text).ok().map(|m| m.width())
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod imp {
    use super::FontInfo;

    pub fn measure(_text: &str, _font: &FontInfo, _weight: u32, _size: f64) -> Option<f64> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::font_info;

    #[test]
    fn outside_the_browser_widths_are_the_estimate() {
        assert!(!available());
        let f = font_info("manrope");
        assert_eq!(width("hello", f, 400, 20.0), 5.0 * 20.0 * f.char_width);
        assert_eq!(width("", f, 400, 20.0), 0.0);
    }

    #[test]
    fn system_fonts_need_no_loading() {
        assert!(load_script("system-mono", "system-sans").is_none());
    }

    #[test]
    fn bundled_fonts_in_use_are_loaded_once_each() {
        let one = load_script("inter", "inter").unwrap();
        assert_eq!(one.matches("\"Inter\"").count(), 1, "{one}");
        let two = load_script("inter", "playfair-display").unwrap();
        assert!(two.contains("\"Inter\"") && two.contains("\"Playfair Display\""));
        let mixed = load_script("system-mono", "dm-sans").unwrap();
        assert!(mixed.contains("[\"DM Sans\"]"));
        assert!(mixed.contains("document.fonts.load") && mixed.contains("dioxus.send(true)"));
    }
}
