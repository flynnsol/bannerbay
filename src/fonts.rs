//! The bundled Google fonts: which file backs each font, and the `@font-face` rules that load
//! them. Variable fonts, Latin subset, SIL Open Font License (licences in `assets/fonts/licenses`).

use dioxus::prelude::*;

pub struct Face {
    /// Matches a `FONTS` key in `model.rs`.
    pub key: &'static str,
    pub file: Asset,
}

pub const FACES: &[Face] = &[
    Face { key: "inter", file: asset!("/assets/fonts/inter.woff2") },
    Face { key: "dm-sans", file: asset!("/assets/fonts/dm-sans.woff2") },
    Face { key: "space-grotesk", file: asset!("/assets/fonts/space-grotesk.woff2") },
    Face { key: "manrope", file: asset!("/assets/fonts/manrope.woff2") },
    Face { key: "plus-jakarta-sans", file: asset!("/assets/fonts/plus-jakarta-sans.woff2") },
    Face { key: "playfair-display", file: asset!("/assets/fonts/playfair-display.woff2") },
    Face { key: "fraunces", file: asset!("/assets/fonts/fraunces.woff2") },
    Face { key: "dm-serif-display", file: asset!("/assets/fonts/dm-serif-display.woff2") },
    Face { key: "bebas-neue", file: asset!("/assets/fonts/bebas-neue.woff2") },
    Face { key: "archivo-black", file: asset!("/assets/fonts/archivo-black.woff2") },
    Face { key: "jetbrains-mono", file: asset!("/assets/fonts/jetbrains-mono.woff2") },
    Face { key: "fira-code", file: asset!("/assets/fonts/fira-code.woff2") },
    Face { key: "geist-mono", file: asset!("/assets/fonts/geist-mono.woff2") },
    Face { key: "red-hat-mono", file: asset!("/assets/fonts/red-hat-mono.woff2") },
];

pub fn face(key: &str) -> Option<&'static Face> {
    FACES.iter().find(|f| f.key == key)
}

/// One `@font-face` rule per bundled font, for a `<style>` in the page.
pub fn font_face_css() -> String {
    FACES
        .iter()
        .map(|f| {
            let info = crate::model::font_info(f.key);
            format!(
                "@font-face{{font-family:'{}';src:url('{}') format('woff2');font-weight:{} {};font-style:normal;font-display:swap;}}",
                info.family, f.file, info.weights.0, info.weights.1
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::FONTS;

    #[test]
    fn every_bundled_font_has_a_face_and_every_face_a_font() {
        for font in FONTS.iter().filter(|f| !f.family.is_empty()) {
            assert!(face(font.key).is_some(), "{} has no bundled file", font.key);
            assert!(font.stack.contains(font.family), "{}'s stack must start from its own family", font.key);
        }
        for f in FACES {
            let info = crate::model::font_info(f.key);
            assert_eq!(info.key, f.key, "face {} is not in FONTS", f.key);
            assert!(!info.family.is_empty());
        }
    }

    #[test]
    fn system_fonts_never_name_a_bundled_family() {
        for sys in FONTS.iter().filter(|f| f.family.is_empty()) {
            for bundled in FONTS.iter().filter(|f| !f.family.is_empty()) {
                assert!(!sys.stack.contains(bundled.family), "{} names {}", sys.key, bundled.family);
            }
        }
    }

    #[test]
    fn font_face_css_declares_each_family_once() {
        let css = font_face_css();
        for font in FONTS.iter().filter(|f| !f.family.is_empty()) {
            assert_eq!(css.matches(&format!("font-family:'{}'", font.family)).count(), 1, "{}", font.family);
        }
        assert!(css.contains("format('woff2')") && css.contains("font-weight:"));
    }

    #[test]
    fn licences_ship_with_every_font_file() {
        let root = env!("CARGO_MANIFEST_DIR");
        for f in FACES {
            let font = format!("{root}/assets/fonts/{}.woff2", f.key);
            let lic = format!("{root}/assets/fonts/licenses/{}-OFL.txt", f.key);
            assert!(std::fs::metadata(&font).unwrap().len() > 1000, "{font}");
            assert!(std::fs::read_to_string(&lic).unwrap().contains("SIL OPEN FONT LICENSE"), "{lic}");
        }
    }
}
