//! Browser-side file handling, done with small JS snippets run through `document::eval`.
//! Nothing leaves the browser: the SVG is read from the live preview and saved locally.

use dioxus::prelude::*;

/// Builds `xml`, the SVG to save, from the live preview. Every bundled font in use is fetched and
/// embedded as a base64 `@font-face`, because a saved SVG, or one drawn into a canvas, cannot see the
/// fonts the page loaded.
const PREPARE: &str = r#"
const fonts = __FONTS__;
const live = document.querySelector('#preview svg');
const w = Number(live.getAttribute('width')), h = Number(live.getAttribute('height'));
const svg = live.cloneNode(true);
let css = '';
for (const f of fonts) {
  try {
    const bytes = new Uint8Array(await (await fetch(f.url)).arrayBuffer());
    let bin = '';
    for (let i = 0; i < bytes.length; i += 0x8000) {
      bin += String.fromCharCode.apply(null, bytes.subarray(i, i + 0x8000));
    }
    css += "@font-face{font-family:'" + f.family + "';src:url(data:font/woff2;base64," + btoa(bin) +
      ") format('woff2');font-weight:" + f.w0 + " " + f.w1 + ";font-style:normal}";
  } catch (e) {
    console.warn('Bannerbay: could not embed ' + f.family + '; the export will use a fallback font', e);
  }
}
if (css) {
  const style = document.createElementNS('http://www.w3.org/2000/svg', 'style');
  style.textContent = css;
  svg.insertBefore(style, svg.firstChild);
}
const xml = new XMLSerializer().serializeToString(svg);
"#;

const SAVE_PNG: &str = r#"
const img = new Image();
img.onload = () => {
  const c = document.createElement('canvas');
  c.width = w * __SCALE__; c.height = h * __SCALE__;
  const ctx = c.getContext('2d');
  ctx.scale(__SCALE__, __SCALE__);
  ctx.drawImage(img, 0, 0, w, h);
  c.toBlob(b => {
    const a = document.createElement('a');
    a.href = URL.createObjectURL(b);
    a.download = '__NAME__.png';
    document.body.appendChild(a); a.click(); a.remove();
    setTimeout(() => URL.revokeObjectURL(a.href), 1000);
  }, 'image/png');
};
img.src = 'data:image/svg+xml;charset=utf-8,' + encodeURIComponent(xml);
"#;

const SAVE_SVG: &str = r#"
const a = document.createElement('a');
a.href = URL.createObjectURL(new Blob([xml], { type: 'image/svg+xml' }));
a.download = '__NAME__.svg';
document.body.appendChild(a); a.click(); a.remove();
setTimeout(() => URL.revokeObjectURL(a.href), 1000);
"#;

const READ_LOGO: &str = r#"
const f = document.getElementById('logo-file').files[0];
if (!f) { dioxus.send(''); }
else { const r = new FileReader(); r.onload = () => dioxus.send(r.result); r.readAsDataURL(f); }
"#;

/// The script that prepares the SVG (with fonts embedded) and then runs `action`.
fn script(action: &str, name: &str, faces: &str) -> String {
    let body = PREPARE.replace("__FONTS__", faces) + action;
    format!("(async () => {{ {} }})();", body.replace("__NAME__", name))
}

/// The bundled fonts used by `heading` and `body` as JSON for the export script, each once.
/// System fonts need no embedding.
pub fn faces_json(heading: &str, body: &str) -> String {
    let mut keys = vec![heading, body];
    keys.dedup();
    let faces: Vec<_> = keys
        .into_iter()
        .filter_map(crate::fonts::face)
        .map(|f| {
            serde_json::json!({
                "family": crate::model::font_info(f.key).family,
                "url": f.file.to_string(),
                "w0": crate::model::font_info(f.key).weights.0,
                "w1": crate::model::font_info(f.key).weights.1,
            })
        })
        .collect();
    serde_json::to_string(&faces).unwrap_or_else(|_| "[]".into())
}

/// `name` must already be a safe file stem (see `Config::file_stem`). `faces` is `faces_json`.
pub fn save_png(name: &str, scale: u32, faces: &str) {
    let action = SAVE_PNG.replace("__SCALE__", &scale.to_string());
    let _ = document::eval(&script(&action, name, faces));
}

pub fn save_svg(name: &str, faces: &str) {
    let _ = document::eval(&script(SAVE_SVG, name, faces));
}

/// Reads the file chosen in the logo input as a `data:` URL; `None` if nothing is selected.
pub async fn read_logo() -> Option<String> {
    let mut eval = document::eval(READ_LOGO);
    match eval.recv::<String>().await {
        Ok(s) if !s.is_empty() => Some(s),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_fonts_embed_nothing() {
        assert_eq!(faces_json("system-mono", "system-sans"), "[]");
        assert_eq!(faces_json("unknown-font", "system-mono"), "[]");
    }

    #[test]
    fn each_bundled_font_in_use_is_listed_once() {
        let both: serde_json::Value = serde_json::from_str(&faces_json("inter", "manrope")).unwrap();
        let names: Vec<_> = both.as_array().unwrap().iter().map(|f| f["family"].as_str().unwrap()).collect();
        assert_eq!(names, ["Inter", "Manrope"]);
        let same: serde_json::Value = serde_json::from_str(&faces_json("inter", "inter")).unwrap();
        assert_eq!(same.as_array().unwrap().len(), 1, "one font used twice is embedded once");
        let mixed: serde_json::Value = serde_json::from_str(&faces_json("system-mono", "dm-sans")).unwrap();
        assert_eq!(mixed.as_array().unwrap().len(), 1);
        let f = &mixed[0];
        assert!(f["url"].as_str().unwrap().contains("dm-sans") && f["url"].as_str().unwrap().ends_with("woff2"));
        assert!(f["w0"].as_u64().unwrap() < f["w1"].as_u64().unwrap());
        // A single-weight font has a one-weight range.
        let display: serde_json::Value = serde_json::from_str(&faces_json("bebas-neue", "bebas-neue")).unwrap();
        assert_eq!((display[0]["w0"].as_u64(), display[0]["w1"].as_u64()), (Some(400), Some(400)));
    }

    #[test]
    fn the_export_script_is_filled_in_and_wrapped() {
        let png = script(&SAVE_PNG.replace("__SCALE__", "2"), "my-banner", "[]");
        assert!(png.starts_with("(async () => {") && png.ends_with("})();"));
        assert!(png.contains("my-banner.png") && png.contains("const fonts = [];"));
        assert!(!png.contains("__"), "every placeholder is replaced: {png}");
        let svg = script(SAVE_SVG, "my-banner", r#"[{"family":"Inter"}]"#);
        assert!(svg.contains("my-banner.svg") && svg.contains(r#"[{"family":"Inter"}]"#) && !svg.contains("__"));
    }
}
