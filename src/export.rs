//! Browser-side file handling, done with small JS snippets run through `document::eval`.
//! Nothing leaves the browser: the SVG is read from the live preview and saved locally.

use dioxus::prelude::*;

const SAVE_PNG: &str = r#"
const svg = document.querySelector('#preview svg');
const w = Number(svg.getAttribute('width')), h = Number(svg.getAttribute('height'));
const xml = new XMLSerializer().serializeToString(svg);
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
const svg = document.querySelector('#preview svg');
const xml = new XMLSerializer().serializeToString(svg);
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

/// `name` must already be a safe file stem (see `Config::file_stem`).
pub fn save_png(name: &str, scale: u32) {
    let js = SAVE_PNG.replace("__SCALE__", &scale.to_string()).replace("__NAME__", name);
    let _ = document::eval(&js);
}

pub fn save_svg(name: &str) {
    let _ = document::eval(&SAVE_SVG.replace("__NAME__", name));
}

/// Reads the file chosen in the logo input as a `data:` URL; `None` if nothing is selected.
pub async fn read_logo() -> Option<String> {
    let mut eval = document::eval(READ_LOGO);
    match eval.recv::<String>().await {
        Ok(s) if !s.is_empty() => Some(s),
        _ => None,
    }
}
