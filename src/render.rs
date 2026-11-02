//! Turns a `Config` into an SVG string. The preview and every export use this
//! same string, so what you see is exactly what you download.

use crate::emphasis::{self, Span};
use crate::model::*;
use std::fmt::Write;

pub struct Rendered {
    pub svg: String,
    /// True when the content runs past the bottom padding of the canvas.
    pub overflow: bool,
}

pub fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

/// Greedy word wrap that also honours explicit newlines.
pub fn wrap(text: &str, max_chars: usize) -> Vec<String> {
    let max_chars = max_chars.max(8);
    let mut lines = Vec::new();
    for para in text.lines() {
        let mut cur = String::new();
        for word in para.split_whitespace() {
            if cur.is_empty() {
                cur = word.to_string();
            } else if cur.chars().count() + 1 + word.chars().count() <= max_chars {
                cur.push(' ');
                cur.push_str(word);
            } else {
                lines.push(std::mem::take(&mut cur));
                cur = word.to_string();
            }
        }
        if !cur.is_empty() {
            lines.push(cur);
        }
    }
    lines
}

fn mark_svg(cfg: &Config, x: f64, y: f64, size: f64) -> Option<String> {
    let k = size / 48.0;
    let t = &cfg.text;
    let a = &cfg.accent;
    let body = match cfg.mark {
        Mark::None => return None,
        Mark::Image => {
            let data = cfg.logo.as_ref()?;
            return Some(format!(
                r#"<image href="{}" x="{x:.1}" y="{y:.1}" width="{size:.1}" height="{size:.1}" preserveAspectRatio="xMidYMid meet"/>"#,
                escape(data)
            ));
        }
        Mark::Blocks => {
            // Three columns climbing like a staircase; the last cell is filled with the accent.
            // Outlined cells are 12 wide plus a 3 stroke = 15 outer, on a 16.5 step: 3 cells fill 48.
            let mut s = String::new();
            for (cx, cy) in [(0.0, 0.0), (0.0, 1.0), (0.0, 2.0), (1.0, 1.0), (1.0, 2.0)] {
                s.push_str(&format!(
                    r#"<rect x="{:.1}" y="{:.1}" width="12" height="12" rx="4" fill="none" stroke="{t}" stroke-width="3"/>"#,
                    cx * 16.5 + 1.5,
                    cy * 16.5 + 1.5
                ));
            }
            s.push_str(&format!(r#"<rect x="33" y="33" width="15" height="15" rx="4" fill="{a}"/>"#));
            s
        }
        Mark::Rings => format!(
            r#"<circle cx="24" cy="24" r="6" fill="{a}"/><path d="M24 4 A20 20 0 0 1 44 24" fill="none" stroke="{t}" stroke-width="3.5" stroke-linecap="round"/><path d="M24 44 A20 20 0 0 1 4 24" fill="none" stroke="{t}" stroke-width="3.5" stroke-linecap="round"/><path d="M24 12 A12 12 0 1 1 12 24" fill="none" stroke="{t}" stroke-width="3.5" stroke-linecap="round"/>"#
        ),
        Mark::Sail => format!(
            r#"<path d="M22 5 L22 31 L7 31 Z" fill="{t}"/><path d="M26 11 L26 31 L40 31 Z" fill="{a}"/><path d="M5 35 H43 L38 44 H10 Z" fill="{t}"/>"#
        ),
    };
    Some(format!(r#"<g transform="translate({x:.1} {y:.1}) scale({k:.4})">{body}</g>"#))
}

fn text_el(x: f64, y: f64, size: f64, weight: u32, fill: &str, opacity: f64, extra: &str, s: &str) -> String {
    text_el_raw(x, y, size, weight, fill, opacity, extra, &escape(s))
}

/// Like `text_el`, but `inner` is already-escaped markup (text and `<tspan>`s).
fn text_el_raw(x: f64, y: f64, size: f64, weight: u32, fill: &str, opacity: f64, extra: &str, inner: &str) -> String {
    format!(
        r#"<text x="{x:.1}" y="{y:.1}" font-size="{size:.1}" font-weight="{weight}" fill="{fill}" fill-opacity="{opacity}" {extra}>{inner}</text>"#
    )
}

/// One line of spans: emphasised runs become coloured `<tspan>`s, plain runs stay bare text.
fn spans_markup(spans: &[Span], highlight: &str) -> String {
    spans
        .iter()
        .map(|sp| {
            if sp.emphasis {
                format!(r#"<tspan fill="{highlight}">{}</tspan>"#, escape(&sp.text))
            } else {
                escape(&sp.text)
            }
        })
        .collect()
}

/// The pill in the top-right corner, with its top edge at `top`. `None` when the badge is empty.
fn badge_svg(cfg: &Config, u: f64, wf: f64, p: f64, top: f64) -> Option<String> {
    let badge = cfg.badge.trim();
    if badge.is_empty() {
        return None;
    }
    let a = &cfg.accent;
    let st = &cfg.style.badge;
    let pill = Pill::new(&st.pill, u);
    let bw = badge.chars().count() as f64 * pill.fs * cfg.font.char_width() + 2.0 * pill.padx;
    let bx = wf - p - bw - st.offset_x * u;
    let mut s = format!(
        r#"<rect x="{bx:.1}" y="{top:.1}" width="{bw:.1}" height="{:.1}" rx="{:.1}" fill="{a}" fill-opacity="{}" stroke="{a}" stroke-opacity="0.7" stroke-width="{:.1}"/>"#,
        pill.h, pill.radius, pill.fill, pill.border
    );
    s.push_str(&text_el(bx + bw / 2.0, top + pill.h / 2.0 + pill.fs * 0.35, pill.fs, 700, a, 1.0, r#"text-anchor="middle""#, badge));
    Some(s)
}

/// Renders with the default id prefix (used by the preview and exports).
pub fn render(cfg: &Config) -> Rendered {
    render_with_id(cfg, "bb")
}

/// Renders with `id` as the prefix of every internal SVG id (`<id>-grad`, `<id>-clip`, ...).
/// SVGs placed in the same page must use different prefixes: ids are page-global, so a clash makes
/// every SVG silently use the first one's gradients and patterns.
pub fn render_with_id(cfg: &Config, id: &str) -> Rendered {
    let (w, h) = cfg.preset.size();
    let (wf, hf) = (w as f64, h as f64);
    // Everything scales from one unit so every preset keeps the same proportions.
    let u = (hf / 630.0).min(wf / 1200.0);
    let p = 64.0 * u;
    let cw = cfg.font.char_width();
    let t = &cfg.text;
    let a = &cfg.accent;
    let radius = cfg.radius as f64;

    let mut s = String::new();
    let _ = write!(
        s,
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}" font-family="{}">"#,
        cfg.font.stack()
    );

    // ---- background -------------------------------------------------------
    let grid = 40.0 * u;
    let (dx, dy) = {
        let r = (cfg.angle as f64).to_radians();
        (r.sin(), -r.cos())
    };
    let _ = write!(s, r#"<defs><clipPath id="{id}-clip"><rect width="{w}" height="{h}" rx="{radius}"/></clipPath>"#);
    let _ = write!(
        s,
        r#"<linearGradient id="{id}-grad" x1="{:.3}" y1="{:.3}" x2="{:.3}" y2="{:.3}"><stop offset="0" stop-color="{}"/><stop offset="1" stop-color="{}"/></linearGradient>"#,
        0.5 - dx / 2.0,
        0.5 - dy / 2.0,
        0.5 + dx / 2.0,
        0.5 + dy / 2.0,
        cfg.bg1,
        cfg.bg2
    );
    let _ = write!(
        s,
        r#"<radialGradient id="{id}-glow" cx="0.85" cy="0.1" r="0.7"><stop offset="0" stop-color="{a}" stop-opacity="0.28"/><stop offset="1" stop-color="{a}" stop-opacity="0"/></radialGradient>"#
    );
    let _ = write!(
        s,
        r#"<pattern id="{id}-grid" width="{grid:.1}" height="{grid:.1}" patternUnits="userSpaceOnUse"><path d="M {grid:.1} 0 L 0 0 0 {grid:.1}" fill="none" stroke="{t}" stroke-opacity="0.07" stroke-width="1"/></pattern>"#
    );
    let _ = write!(
        s,
        r#"<pattern id="{id}-dots" width="{grid:.1}" height="{grid:.1}" patternUnits="userSpaceOnUse"><circle cx="{:.1}" cy="{:.1}" r="{:.1}" fill="{t}" fill-opacity="0.14"/></pattern></defs>"#,
        grid / 2.0,
        grid / 2.0,
        (1.6 * u).max(1.0)
    );
    let base = match cfg.bg_kind {
        BgKind::Gradient => format!("url(#{id}-grad)"),
        _ => cfg.bg1.clone(),
    };
    let _ = write!(s, r#"<g clip-path="url(#{id}-clip)"><rect width="{w}" height="{h}" fill="{base}"/>"#);
    match cfg.bg_kind {
        BgKind::Grid => {
            let _ = write!(s, r#"<rect width="{w}" height="{h}" fill="url(#{id}-grid)"/>"#);
        }
        BgKind::Dots => {
            let _ = write!(s, r#"<rect width="{w}" height="{h}" fill="url(#{id}-dots)"/>"#);
        }
        _ => {}
    }
    if cfg.glow {
        let _ = write!(s, r#"<rect width="{w}" height="{h}" fill="url(#{id}-glow)"/>"#);
    }
    s.push_str("</g>");

    // ---- lockup: just a mark and a name, scaled up and centred -------------
    let lockup = cfg.header.trim().is_empty()
        && cfg.description.trim().is_empty()
        && cfg.tag.trim().is_empty()
        && cfg.feature_list().is_empty()
        && !cfg.name.trim().is_empty();
    if lockup {
        let name = cfg.name.trim();
        let name_w = name.chars().count() as f64 * cw; // in units of font size
        let has_mark = mark_svg(cfg, 0.0, 0.0, 1.0).is_some();
        // Style values are multipliers here: 1x at the defaults.
        let ms = &cfg.style.mark;
        let (m, g, n) = (ms.size / 48.0, ms.gap / 16.0, ms.name_size / 28.0);
        let extra = if has_mark { 1.2 * m + 0.36 * g } else { 0.0 }; // mark + gap, in font sizes
        let fs = (hf * 0.25 * n).min(wf * 0.78 / (extra + name_w));
        let total = fs * (extra + name_w);
        let mut x = (wf - total) / 2.0;
        let mark_size = fs * 1.2 * m;
        if let Some(mk) = mark_svg(cfg, x, hf / 2.0 - mark_size / 2.0, mark_size) {
            s.push_str(&mk);
            x += mark_size + fs * 0.36 * g;
        }
        let weight = (ms.name_weight as u32 + 100).min(900);
        s.push_str(&text_el(x, hf / 2.0 + fs * 0.35, fs, weight, t, 1.0, "", name));
        if let Some(b) = badge_svg(cfg, u, wf, p, p + cfg.style.badge.offset_y * u) {
            s.push_str(&b);
        }
        s.push_str("</svg>");
        return Rendered { svg: s, overflow: false };
    }

    // ---- top row: mark + name, badge on the right -------------------------
    let ms = &cfg.style.mark;
    let logo_size = ms.size * u;
    let name_fs = ms.name_size * u;
    // The row is as tall as its taller part; the shorter one is centred in it.
    let row_h = logo_size.max(name_fs * 1.2);
    let mark = mark_svg(cfg, p, p + (row_h - logo_size) / 2.0, logo_size);
    let has_name = !cfg.name.trim().is_empty();
    let has_row = mark.is_some() || has_name;
    let mut y = p;
    let mut bottom = p;

    if has_row {
        let mut x = p;
        if let Some(m) = mark {
            s.push_str(&m);
            x += logo_size + ms.gap * u;
        }
        if has_name {
            s.push_str(&text_el(x, p + row_h / 2.0 + name_fs * 0.35, name_fs, ms.name_weight as u32, t, 1.0, "", cfg.name.trim()));
        }
        bottom = p + row_h;
        y = bottom + ms.space_after * u;
    }

    let badge_h = Pill::new(&cfg.style.badge.pill, u).h;
    let badge_top = if has_row { p + row_h / 2.0 - badge_h / 2.0 } else { p } + cfg.style.badge.offset_y * u;
    if let Some(b) = badge_svg(cfg, u, wf, p, badge_top) {
        s.push_str(&b);
        if !has_row {
            bottom = badge_top + badge_h;
            y = bottom + ms.space_after * u;
        }
    }

    // ---- tag (eyebrow) ----------------------------------------------------
    let tag = cfg.tag.trim();
    if !tag.is_empty() {
        let ts = &cfg.style.tag;
        let fs = ts.size * u;
        let extra = format!(r#"letter-spacing="{:.1}""#, ts.letter_spacing * u);
        s.push_str(&text_el(p, y + fs, fs, 700, a, 1.0, &extra, &tag.to_uppercase()));
        bottom = y + fs;
        y = bottom + ts.space_after * u;
    }

    // ---- feature pill geometry (shared by both layouts) -------------------
    let features = cfg.feature_list();
    let pill = Pill::new(&cfg.style.pill, u);
    let column = cfg.layout == FeatureLayout::Vertical && !features.is_empty();
    // Vertical layout: equal-width pills in a column on the right, centred top to bottom.
    let col_w = features.iter().map(|f| pill.width(f, cw)).fold(0.0_f64, f64::max);
    let col_x = wf - p - col_w;
    let col_h = features.len() as f64 * (pill.h + pill.gap) - pill.gap;
    let col_y = (hf - col_h) / 2.0;
    let text_gap = 40.0 * u;
    let min_text_w = 160.0 * u;

    // ---- header -----------------------------------------------------------
    let content_w = wf - 2.0 * p;
    // The text keeps clear of the pill column.
    let text_w = if column { (col_x - text_gap - p).max(min_text_w) } else { content_w };
    let hs = &cfg.style.header;
    let header_fs = hs.size * u;
    let header_spans = emphasis::parse(cfg.header.trim());
    let lines = emphasis::wrap(&header_spans, (text_w / (header_fs * cw * 1.04)) as usize);
    let lh = header_fs * hs.line_height;
    let highlight = cfg.highlight_color();
    for (i, line) in lines.iter().enumerate() {
        let inner = spans_markup(line, highlight);
        s.push_str(&text_el_raw(p, y + header_fs * 0.85 + i as f64 * lh, header_fs, hs.weight as u32, t, 1.0, "", &inner));
    }
    if !lines.is_empty() {
        bottom = y + lines.len() as f64 * lh;
        y = bottom + hs.space_after * u;
    }

    // ---- description ------------------------------------------------------
    let ds = &cfg.style.desc;
    let desc_fs = ds.size * u;
    let desc_w = text_w.min(840.0 * u);
    let lines = wrap(cfg.description.trim(), (desc_w / (desc_fs * cw)) as usize);
    let lh = desc_fs * ds.line_height;
    for (i, line) in lines.iter().enumerate() {
        s.push_str(&text_el(p, y + desc_fs * 0.95 + i as f64 * lh, desc_fs, ds.weight as u32, t, ds.opacity.clamp(0.0, 1.0), "", line));
    }
    if !lines.is_empty() {
        bottom = y + lines.len() as f64 * lh;
        y = bottom + ds.space_after * u;
    }

    // ---- features ---------------------------------------------------------
    let mut overflow = false;
    if column {
        for (i, f) in features.iter().enumerate() {
            s.push_str(&pill_svg(cfg, &pill, col_x, col_y + i as f64 * (pill.h + pill.gap), col_w, f));
        }
        // The column must fit inside the padding, clear of the badge, and leave room for text.
        let badge_bottom = if cfg.badge.trim().is_empty() { 0.0 } else { badge_top + badge_h + 12.0 * u };
        overflow = col_y < p - 1.0
            || col_y + col_h > hf - p + 1.0
            || col_y < badge_bottom
            || col_x - text_gap - p < min_text_w;
    } else if !features.is_empty() {
        let (mut x, mut row_y) = (p, y);
        for f in &features {
            let pw = pill.width(f, cw);
            if x + pw > wf - p && x > p {
                x = p;
                row_y += pill.h + pill.gap;
            }
            s.push_str(&pill_svg(cfg, &pill, x, row_y, pw, f));
            x += pw + pill.gap;
        }
        bottom = row_y + pill.h;
    }

    s.push_str("</svg>");
    Rendered { svg: s, overflow: overflow || bottom > hf - p + 1.0 }
}

/// A feature pill's dimensions in pixels (the style scaled to the canvas).
struct Pill {
    fs: f64,
    padx: f64,
    h: f64,
    gap: f64,
    dot: f64,
    radius: f64,
    border: f64,
    fill: f64,
}

impl Pill {
    fn new(style: &PillStyle, u: f64) -> Self {
        let h = (style.height * u).max(1.0);
        Self {
            fs: style.font_size * u,
            padx: style.pad_x * u,
            h,
            gap: style.gap * u,
            dot: 14.0 * u,
            radius: (style.radius * u).clamp(0.0, h / 2.0),
            // Thin borders stay visible on small canvases; 0 means no border at all.
            border: if style.border <= 0.0 { 0.0 } else { (style.border * u).max(1.0) },
            fill: style.fill.clamp(0.0, 1.0),
        }
    }

    fn width(&self, label: &str, char_width: f64) -> f64 {
        label.chars().count() as f64 * self.fs * char_width + 2.0 * self.padx + self.dot
    }
}

/// One outlined pill with an accent dot and its label, left-aligned inside a box of width `w`.
fn pill_svg(cfg: &Config, pill: &Pill, x: f64, y: f64, w: f64, label: &str) -> String {
    let (t, a) = (&cfg.text, &cfg.accent);
    let mut s = format!(
        r#"<rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{:.1}" rx="{:.1}" fill="{t}" fill-opacity="{}" stroke="{t}" stroke-opacity="0.22" stroke-width="{:.1}"/>"#,
        pill.h, pill.radius, pill.fill, pill.border
    );
    let _ = write!(
        s,
        r#"<circle cx="{:.1}" cy="{:.1}" r="{:.1}" fill="{a}"/>"#,
        x + pill.padx,
        y + pill.h / 2.0,
        pill.dot * 4.0 / 14.0
    );
    s.push_str(&text_el(x + pill.padx + pill.dot, y + pill.h / 2.0 + pill.fs * 0.35, pill.fs, 500, t, 0.9, "", label));
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_markup() {
        assert_eq!(escape(r#"<a & "b">"#), "&lt;a &amp; &quot;b&quot;&gt;");
    }

    #[test]
    fn wraps_on_words_and_newlines() {
        assert_eq!(wrap("aaa bbb ccc", 8), vec!["aaa bbb", "ccc"]);
        assert_eq!(wrap("one\ntwo", 20), vec!["one", "two"]);
        assert!(wrap("   ", 20).is_empty());
    }

    #[test]
    fn long_word_is_kept_whole() {
        assert_eq!(wrap("supercalifragilistic", 8), vec!["supercalifragilistic"]);
    }

    #[test]
    fn default_fits_every_preset() {
        for preset in Preset::ALL {
            let cfg = Config { preset: *preset, ..Config::default() };
            let (w, h) = preset.size();
            let r = render(&cfg);
            assert!(r.svg.contains(&format!(r#"width="{w}" height="{h}""#)));
            if *preset != Preset::Banner {
                assert!(!r.overflow, "default content overflows {preset:?}");
            }
        }
    }

    #[test]
    fn huge_text_reports_overflow() {
        let cfg = Config { description: "word ".repeat(400), ..Config::default() };
        assert!(render(&cfg).overflow);
    }

    #[test]
    fn user_text_is_escaped_in_svg() {
        let cfg = Config { header: "</text><script>x</script>".into(), ..Config::default() };
        let svg = render(&cfg).svg;
        assert!(!svg.contains("<script>"));
    }

    #[test]
    fn svg_is_well_formed_for_all_variants() {
        for bg in BgKind::ALL {
            for mark in Mark::ALL {
                for layout in FeatureLayout::ALL {
                    let cfg = Config { bg_kind: *bg, mark: *mark, layout: *layout, ..Config::default() };
                    let svg = render(&cfg).svg;
                    assert!(svg.starts_with("<svg") && svg.ends_with("</svg>"));
                }
            }
        }
    }

    #[test]
    fn lockup_when_only_a_name_is_set() {
        let cfg = Config {
            preset: Preset::Banner,
            header: "".into(),
            description: "".into(),
            tag: "".into(),
            features: "".into(),
            ..Config::default()
        };
        let r = render(&cfg);
        assert!(!r.overflow);
        assert!(r.svg.contains("bannerbay"));
        // Lockup text is far larger than the 28-unit name in the normal layout.
        assert!(r.svg.contains(r#"font-size="100.0""#));
    }

    #[test]
    fn file_stem_is_sanitised() {
        let cfg = Config { name: "My Cool/App!".into(), ..Config::default() };
        assert_eq!(cfg.file_stem(), "my-cool-app");
        assert_eq!(Config { name: "".into(), ..Config::default() }.file_stem(), "banner");
    }

    fn nums(svg: &str, attr: &str) -> Vec<f64> {
        let key = format!(r#"{attr}=""#);
        svg.match_indices(&key)
            .filter_map(|(i, _)| svg[i + key.len()..].split('"').next()?.parse().ok())
            .collect()
    }

    #[test]
    fn vertical_features_form_a_right_hand_column_centred_on_the_card() {
        let cfg = Config { layout: FeatureLayout::Vertical, badge: "".into(), ..Config::default() };
        let (w, h) = cfg.preset.size();
        let svg = render(&cfg).svg;
        // Pill rects are the ones with a stroke-opacity of 0.22.
        let pills: Vec<&str> = svg.split("<rect ").filter(|r| r.contains(r#"stroke-opacity="0.22""#)).collect();
        assert_eq!(pills.len(), 3);
        let xs: Vec<f64> = pills.iter().map(|r| nums(&format!("<rect {r}"), "x")[0]).collect();
        let ys: Vec<f64> = pills.iter().map(|r| nums(&format!("<rect {r}"), "y")[0]).collect();
        let ws: Vec<f64> = pills.iter().map(|r| nums(&format!("<rect {r}"), "width")[0]).collect();
        assert!(xs.iter().all(|x| (*x - xs[0]).abs() < 0.2), "same left edge: {xs:?}");
        assert!(ws.iter().all(|x| (*x - ws[0]).abs() < 0.2), "equal widths: {ws:?}");
        assert!((xs[0] + ws[0] - (w as f64 - 64.0 * (h as f64 / 630.0).min(w as f64 / 1200.0))).abs() < 0.2, "right edge on padding");
        assert!(xs[0] > w as f64 / 2.0, "column is on the right half");
        let (top, bottom) = (ys[0], ys[2] + 44.0 * (h as f64 / 630.0).min(w as f64 / 1200.0));
        assert!(((top + bottom) / 2.0 - h as f64 / 2.0).abs() < 0.5, "centred vertically");
    }

    #[test]
    fn text_wraps_left_of_the_column_in_vertical_layout() {
        let long = Config {
            description: "word ".repeat(60),
            features: "A much longer feature label here".into(),
            ..Config::default()
        };
        let vert = Config { layout: FeatureLayout::Vertical, ..long.clone() };
        let horiz = Config { layout: FeatureLayout::Horizontal, ..long };
        let widest = |svg: &str| {
            svg.split("<text ")
                .filter(|t| t.contains(r#"fill-opacity="0.72""#))
                .map(|t| t.split('>').nth(1).unwrap_or("").chars().count())
                .max()
                .unwrap_or(0)
        };
        assert!(widest(&render(&vert).svg) < widest(&render(&horiz).svg));
    }

    #[test]
    fn column_that_does_not_fit_reports_overflow() {
        let cfg = Config {
            preset: Preset::Banner,
            layout: FeatureLayout::Vertical,
            features: (1..=9).map(|i| format!("Feature {i}")).collect::<Vec<_>>().join("\n"),
            ..Config::default()
        };
        assert!(render(&cfg).overflow);
    }

    #[test]
    fn default_vertical_column_fits_every_preset() {
        for preset in Preset::ALL {
            let cfg = Config { preset: *preset, layout: FeatureLayout::Vertical, ..Config::default() };
            assert!(!render(&cfg).overflow, "column overflows {preset:?}");
        }
    }

    /// Compares `cfg`'s SVG with `tests/golden/<name>.svg`. Regenerate deliberately with
    /// `BB_UPDATE_GOLDEN=1 cargo test` when the default look is meant to change.
    fn golden(name: &str, cfg: &Config) {
        let path = format!("{}/tests/golden/{name}.svg", env!("CARGO_MANIFEST_DIR"));
        let svg = render(cfg).svg;
        if std::env::var("BB_UPDATE_GOLDEN").is_ok() {
            std::fs::create_dir_all(format!("{}/tests/golden", env!("CARGO_MANIFEST_DIR"))).unwrap();
            std::fs::write(&path, &svg).unwrap();
            return;
        }
        let want = std::fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("missing golden file {path}; run with BB_UPDATE_GOLDEN=1"));
        assert!(svg == want, "default render changed for {name}; if intended, rerun with BB_UPDATE_GOLDEN=1");
    }

    #[test]
    fn default_style_renders_identically() {
        golden("social_horizontal", &Config::default());
        golden("social_vertical", &Config { layout: FeatureLayout::Vertical, ..Config::default() });
        golden("banner_horizontal", &Config { preset: Preset::Banner, ..Config::default() });
        golden("wide_gradient_vertical", &Config { preset: Preset::Wide, bg_kind: BgKind::Gradient, layout: FeatureLayout::Vertical, ..Config::default() });
        golden("lockup", &Config { header: "".into(), description: "".into(), tag: "".into(), features: "".into(), ..Config::default() });
    }

    fn pill_rects(svg: &str) -> Vec<String> {
        svg.split("<rect ").filter(|r| r.contains(r#"stroke-opacity="0.22""#)).map(|r| r.to_string()).collect()
    }

    #[test]
    fn pill_style_changes_the_output() {
        let base = render(&Config::default()).svg;
        let styled = |f: fn(&mut PillStyle)| {
            let mut c = Config::default();
            f(&mut c.style.pill);
            render(&c).svg
        };
        let square = styled(|p| p.radius = 0.0);
        assert!(pill_rects(&square).iter().all(|r| r.contains(r#"rx="0.0""#)));
        assert!(styled(|p| p.border = 0.0).contains(r#"stroke-width="0.0""#));
        assert!(styled(|p| p.fill = 0.3).contains(r#"fill-opacity="0.3""#));
        assert_ne!(styled(|p| p.pad_x = 30.0), base);
        assert_ne!(styled(|p| p.font_size = 28.0), base);
        assert_ne!(styled(|p| p.gap = 30.0), base);
        assert_ne!(styled(|p| p.height = 60.0), base);
    }

    #[test]
    fn radius_is_capped_at_a_capsule() {
        let mut cfg = Config::default();
        cfg.style.pill.radius = 500.0;
        let (_, h) = cfg.preset.size();
        let u = h as f64 / 630.0;
        let cap = format!(r#"rx="{:.1}""#, 44.0 * u / 2.0);
        assert!(pill_rects(&render(&cfg).svg).iter().all(|r| r.contains(&cap)), "expected {cap}");
    }

    #[test]
    fn taller_pills_make_a_taller_column() {
        let tall = |height: f64| {
            let mut cfg = Config { layout: FeatureLayout::Vertical, badge: "".into(), ..Config::default() };
            cfg.style.pill.height = height;
            let svg = render(&cfg).svg;
            let ys: Vec<f64> = pill_rects(&svg).iter().map(|r| nums(&format!("<rect {r}"), "y")[0]).collect();
            ys[2] - ys[0]
        };
        assert!(tall(70.0) > tall(44.0));
    }

    #[test]
    fn badge_style_changes_the_badge_only() {
        let badge_rect = |svg: &str| svg.split("<rect ").find(|r| r.contains(r#"stroke-opacity="0.7""#)).unwrap().to_string();
        let base = badge_rect(&render(&Config::default()).svg);
        let styled = |f: fn(&mut BadgeStyle)| {
            let mut c = Config::default();
            f(&mut c.style.badge);
            render(&c).svg
        };
        assert!(badge_rect(&styled(|b| b.pill.radius = 0.0)).contains(r#"rx="0.0""#));
        assert!(badge_rect(&styled(|b| b.pill.fill = 0.4)).contains(r#"fill-opacity="0.4""#));
        assert_ne!(badge_rect(&styled(|b| b.pill.height = 60.0)), base);
        assert_ne!(badge_rect(&styled(|b| b.pill.pad_x = 30.0)), base);
        // Feature pills are untouched by badge style.
        assert_eq!(pill_rects(&styled(|b| b.pill.radius = 0.0)), pill_rects(&render(&Config::default()).svg));
    }

    #[test]
    fn badge_offsets_move_it_inward_and_down() {
        let pos = |svg: &str| {
            let r = svg.split("<rect ").find(|r| r.contains(r#"stroke-opacity="0.7""#)).unwrap().to_string();
            let r = format!("<rect {r}");
            (nums(&r, "x")[0], nums(&r, "y")[0], nums(&r, "width")[0])
        };
        let (x0, y0, w0) = pos(&render(&Config::default()).svg);
        let mut c = Config::default();
        c.style.badge.offset_x = 40.0;
        c.style.badge.offset_y = 20.0;
        let (x1, y1, w1) = pos(&render(&c).svg);
        assert!(x1 < x0 && y1 > y0 && (w1 - w0).abs() < 0.01, "({x0},{y0}) -> ({x1},{y1})");
    }

    #[test]
    fn badge_offset_applies_in_lockup_mode_too() {
        let mut c = Config { header: "".into(), description: "".into(), tag: "".into(), features: "".into(), ..Config::default() };
        let y = |c: &Config| {
            let svg = render(c).svg;
            let r = svg.split("<rect ").find(|r| r.contains(r#"stroke-opacity="0.7""#)).unwrap().to_string();
            nums(&format!("<rect {r}"), "y")[0]
        };
        let base = y(&c);
        c.style.badge.offset_y = 30.0;
        assert!(y(&c) > base);
    }

    #[test]
    fn mark_and_name_style_apply_in_the_top_row() {
        let name_text = |svg: &str| svg.split("<text ").find(|t| t.contains(">bannerbay<")).unwrap().to_string();
        let base = name_text(&render(&Config::default()).svg);
        let styled = |f: fn(&mut MarkStyle)| {
            let mut c = Config::default();
            f(&mut c.style.mark);
            render(&c).svg
        };
        assert!(name_text(&styled(|m| m.name_size = 40.0)).contains(r#"font-size="40.0""#));
        assert!(name_text(&styled(|m| m.name_weight = 400.0)).contains(r#"font-weight="400""#));
        // A wider gap pushes the name right.
        let x = |t: &str| nums(&format!("<text {t}"), "x")[0];
        assert!(x(&name_text(&styled(|m| m.gap = 40.0))) > x(&base));
        // A bigger mark pushes the name right and makes the row taller, so the tag drops.
        assert!(x(&name_text(&styled(|m| m.size = 80.0))) > x(&base));
    }

    #[test]
    fn lockup_responds_to_mark_and_name_style() {
        let lock = |f: fn(&mut MarkStyle)| {
            let mut c = Config { header: "".into(), description: "".into(), tag: "".into(), features: "".into(), badge: "".into(), ..Config::default() };
            f(&mut c.style.mark);
            let svg = render(&c).svg;
            let t = svg.split("<text ").find(|t| t.contains(">bannerbay<")).unwrap().to_string();
            (nums(&format!("<text {t}"), "font-size")[0], nums(&format!("<text {t}"), "x")[0])
        };
        let (fs0, x0) = lock(|_| {});
        let (fs1, _) = lock(|m| m.name_size = 20.0);
        assert!(fs1 < fs0, "smaller name size shrinks the lockup text");
        let (_, x2) = lock(|m| m.size = 24.0);
        assert!(x2 != x0);
    }

    #[test]
    fn text_block_styles_apply() {
        let find = |svg: &str, needle: &str| svg.split("<text ").find(|t| t.contains(needle)).unwrap().to_string();
        let styled = |f: fn(&mut Style)| {
            let mut c = Config::default();
            f(&mut c.style);
            render(&c).svg
        };
        let svg = styled(|s| { s.header.size = 40.0; s.header.weight = 500.0; });
        let h = find(&svg, "Ship your");
        assert!(h.contains(r#"font-size="40.0""#) && h.contains(r#"font-weight="500""#));
        let svg = styled(|s| { s.desc.opacity = 0.4; s.desc.size = 30.0; s.desc.weight = 700.0; });
        let d = find(&svg, "Design README");
        assert!(d.contains(r#"fill-opacity="0.4""#) && d.contains(r#"font-size="30.0""#) && d.contains(r#"font-weight="700""#));
        let svg = styled(|s| { s.tag.size = 22.0; s.tag.letter_spacing = 6.0; });
        let t = find(&svg, "OPEN SOURCE");
        assert!(t.contains(r#"font-size="22.0""#) && t.contains(r#"letter-spacing="6.0""#));
    }

    #[test]
    fn spacing_and_line_height_move_the_blocks() {
        let y_of = |svg: &str, needle: &str| {
            let t = svg.split("<text ").find(|t| t.contains(needle)).unwrap().to_string();
            nums(&format!("<text {t}"), "y")[0]
        };
        let base = render(&Config::default()).svg;
        let styled = |f: fn(&mut Style)| {
            let mut c = Config::default();
            f(&mut c.style);
            render(&c).svg
        };
        // More space below the logo row pushes the tag down.
        assert!(y_of(&styled(|s| s.mark.space_after = 90.0), "OPEN SOURCE") > y_of(&base, "OPEN SOURCE"));
        // More space after the tag pushes the header down.
        assert!(y_of(&styled(|s| s.tag.space_after = 50.0), "Ship your") > y_of(&base, "Ship your"));
        // More space after the header pushes the description down.
        assert!(y_of(&styled(|s| s.header.space_after = 60.0), "Design README") > y_of(&base, "Design README"));
        // A taller description line height spreads its lines.
        let second = |svg: &str| y_of(svg, "Pick a background") - y_of(svg, "Design README");
        assert!(second(&styled(|s| s.desc.line_height = 2.0)) > second(&base));
        // More space after the description pushes the pills down.
        let pill_y = |svg: &str| nums(&format!("<rect {}", svg.split("<rect ").find(|r| r.contains(r#"stroke-opacity="0.22""#)).unwrap()), "y")[0];
        assert!(pill_y(&styled(|s| s.desc.space_after = 80.0)) > pill_y(&base));
    }

    #[test]
    fn marked_header_words_use_the_highlight_colour() {
        let cfg = Config { header: "Ship your *launch banner* in seconds".into(), ..Config::default() };
        let svg = render(&cfg).svg;
        // Highlight follows the accent by default.
        assert!(svg.contains(r##"<tspan fill="#d6a24b">launch banner</tspan>"##));
        assert!(!svg.contains('*'), "markers must not be drawn");
        let custom = Config { highlight: Some("#ff3366".into()), ..cfg.clone() };
        assert!(render(&custom).svg.contains(r##"<tspan fill="#ff3366">launch banner</tspan>"##));
        // Following the accent: change the accent and the highlight follows.
        let accent = Config { accent: "#00ff88".into(), ..cfg };
        assert!(render(&accent).svg.contains(r##"<tspan fill="#00ff88">launch banner</tspan>"##));
    }

    #[test]
    fn header_without_markers_has_no_tspans() {
        let plain = Config { header: "Ship your launch banner in seconds".into(), ..Config::default() };
        assert!(!render(&plain).svg.contains("<tspan"));
        let escaped = Config { header: r"2 \* 3 and 5 * 4".into(), ..Config::default() };
        let svg = render(&escaped).svg;
        assert!(!svg.contains("<tspan") && svg.contains("2 * 3 and 5 * 4"));
    }

    #[test]
    fn highlighted_text_is_escaped_and_survives_wrapping() {
        let cfg = Config { header: "a *<b> & c* d e f g h i j k l m n o p".into(), ..Config::default() };
        let svg = render(&cfg).svg;
        assert!(svg.contains("&lt;b&gt; &amp; c</tspan>"));
        assert!(!svg.contains("<b>"));
        // Wrapped across lines inside the column layout too.
        let col = Config { layout: FeatureLayout::Vertical, header: "Ship *your launch banner in* seconds, please, today".into(), ..Config::default() };
        let svg = render(&col).svg;
        assert!(svg.matches("<tspan").count() >= 2, "a highlighted phrase split across lines is emphasised on each");
    }

    fn ids_and_refs(svg: &str) -> (Vec<String>, Vec<String>) {
        let grab = |marker: &str, end: char| -> Vec<String> {
            svg.match_indices(marker).map(|(i, _)| svg[i + marker.len()..].split(end).next().unwrap().to_string()).collect()
        };
        (grab(" id=\"", '"'), grab("url(#", ')'))
    }

    #[test]
    fn svgs_on_one_page_never_share_ids() {
        // Every background kind, plus glow, so every kind of def is present.
        let mut all_ids: Vec<String> = Vec::new();
        for (i, kind) in BgKind::ALL.iter().enumerate() {
            let cfg = Config { bg_kind: *kind, glow: true, ..Config::default() };
            let svg = render_with_id(&cfg, &format!("th{i}")).svg;
            let (ids, refs) = ids_and_refs(&svg);
            assert!(!ids.is_empty());
            // Every reference points at an id this very SVG defines, under its own prefix.
            for r in &refs {
                assert!(ids.contains(r), "{r} is not defined in its own svg");
                assert!(r.starts_with(&format!("th{i}-")), "{r} ignores the prefix");
            }
            all_ids.extend(ids);
        }
        let preview = ids_and_refs(&render(&Config::default()).svg).0;
        all_ids.extend(preview);
        let mut unique = all_ids.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), all_ids.len(), "duplicate ids: {all_ids:?}");
    }

    #[test]
    fn default_prefix_is_bb() {
        let (ids, _) = ids_and_refs(&render(&Config::default()).svg);
        assert!(ids.iter().all(|i| i.starts_with("bb-")), "{ids:?}");
    }
}
