//! The settings a user edits. Pure data, no UI or rendering.

/// An enum whose variants can be shown in a `<select>` and parsed back.
pub trait Keyed: Copy + PartialEq + Sized + 'static {
    const ALL: &'static [Self];
    fn key(self) -> &'static str;
    fn label(self) -> &'static str;
    fn from_key(key: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|v| v.key() == key)
    }
}

macro_rules! keyed_enum {
    ($name:ident { $($var:ident => ($key:literal, $label:literal)),+ $(,)? }) => {
        #[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
        #[serde(rename_all = "snake_case")]
        pub enum $name { $($var),+ }
        impl Keyed for $name {
            const ALL: &'static [Self] = &[$($name::$var),+];
            fn key(self) -> &'static str { match self { $($name::$var => $key),+ } }
            fn label(self) -> &'static str { match self { $($name::$var => $label),+ } }
        }
    };
}

keyed_enum!(Preset {
    Banner => ("banner", "README banner (1520×400)"),
    Social => ("social", "Social / OG card (1200×630)"),
    Square => ("square", "Square (1080×1080)"),
    Wide => ("wide", "Wide 16:9 (1920×1080)"),
});

impl Preset {
    pub fn size(self) -> (u32, u32) {
        match self {
            Preset::Banner => (1520, 400),
            Preset::Social => (1200, 630),
            Preset::Square => (1080, 1080),
            Preset::Wide => (1920, 1080),
        }
    }
}

keyed_enum!(BgKind {
    Solid => ("solid", "Solid"),
    Gradient => ("gradient", "Gradient"),
    Grid => ("grid", "Grid"),
    Dots => ("dots", "Dots"),
});

keyed_enum!(FeatureLayout {
    Horizontal => ("horizontal", "Horizontal pills"),
    Vertical => ("vertical", "Vertical list"),
});

keyed_enum!(Mark {
    Blocks => ("blocks", "Blocks"),
    Rings => ("rings", "Rings"),
    Sail => ("sail", "Sail"),
    Image => ("image", "Uploaded image"),
    None => ("none", "None"),
});

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FontCategory {
    System,
    Sans,
    Serif,
    Display,
    Mono,
}

impl FontCategory {
    /// The order the picker lists them in.
    pub const ALL: [FontCategory; 5] = [
        FontCategory::Mono,
        FontCategory::Sans,
        FontCategory::Serif,
        FontCategory::Display,
        FontCategory::System,
    ];

    pub fn label(self) -> &'static str {
        match self {
            FontCategory::System => "System",
            FontCategory::Sans => "Sans-serif",
            FontCategory::Serif => "Serif",
            FontCategory::Display => "Display",
            FontCategory::Mono => "Monospace",
        }
    }
}

/// One selectable font. Settings store the `key`, so the list can grow or change without
/// breaking saved banners.
#[derive(Debug)]
pub struct FontInfo {
    pub key: &'static str,
    pub name: &'static str,
    pub category: FontCategory,
    /// The `@font-face` family for a bundled font; empty for system fonts.
    pub family: &'static str,
    /// The CSS `font-family` value, with fallbacks.
    pub stack: &'static str,
    /// Average glyph width as a fraction of the font size; used for wrapping and pill sizing
    /// until text is measured for real.
    pub char_width: f64,
    /// The weight range the font really has. Text is kept inside it, because asking for a weight
    /// outside it makes the browser fake a bold.
    pub weights: (u16, u16),
}

/// Both the heading and body fonts start here. It is bundled, so every visitor sees the same font
/// and exports can embed it.
pub const DEFAULT_FONT: &str = "jetbrains-mono";

/// Widths of bundled fonts were measured on a banner sentence and given a 5% margin (monospace
/// fonts are exactly 0.6). Variable fonts list their real weight range.
pub const FONTS: &[FontInfo] = &[
    FontInfo {
        key: "system-mono",
        name: "System monospace",
        category: FontCategory::System,
        family: "",
        stack: "ui-monospace, 'SF Mono', Menlo, Consolas, 'DejaVu Sans Mono', monospace",
        char_width: 0.6,
        weights: (100, 900),
    },
    FontInfo {
        key: "system-sans",
        name: "System sans-serif",
        category: FontCategory::System,
        family: "",
        stack: "system-ui, -apple-system, 'Segoe UI', 'Helvetica Neue', Arial, sans-serif",
        char_width: 0.56,
        weights: (100, 900),
    },
    FontInfo {
        key: "inter",
        name: "Inter",
        category: FontCategory::Sans,
        family: "Inter",
        stack: "'Inter', system-ui, 'Helvetica Neue', Arial, sans-serif",
        char_width: 0.515,
        weights: (100, 900),
    },
    FontInfo {
        key: "dm-sans",
        name: "DM Sans",
        category: FontCategory::Sans,
        family: "DM Sans",
        stack: "'DM Sans', system-ui, 'Helvetica Neue', Arial, sans-serif",
        char_width: 0.5,
        weights: (100, 900),
    },
    FontInfo {
        key: "space-grotesk",
        name: "Space Grotesk",
        category: FontCategory::Sans,
        family: "Space Grotesk",
        stack: "'Space Grotesk', system-ui, 'Helvetica Neue', Arial, sans-serif",
        char_width: 0.515,
        weights: (300, 700),
    },
    FontInfo {
        key: "manrope",
        name: "Manrope",
        category: FontCategory::Sans,
        family: "Manrope",
        stack: "'Manrope', system-ui, 'Helvetica Neue', Arial, sans-serif",
        char_width: 0.475,
        weights: (200, 800),
    },
    FontInfo {
        key: "plus-jakarta-sans",
        name: "Plus Jakarta Sans",
        category: FontCategory::Sans,
        family: "Plus Jakarta Sans",
        stack: "'Plus Jakarta Sans', system-ui, 'Helvetica Neue', Arial, sans-serif",
        char_width: 0.5,
        weights: (200, 800),
    },
    FontInfo {
        key: "playfair-display",
        name: "Playfair Display",
        category: FontCategory::Serif,
        family: "Playfair Display",
        stack: "'Playfair Display', Georgia, 'Times New Roman', serif",
        char_width: 0.485,
        weights: (400, 900),
    },
    FontInfo {
        key: "fraunces",
        name: "Fraunces",
        category: FontCategory::Serif,
        family: "Fraunces",
        stack: "'Fraunces', Georgia, 'Times New Roman', serif",
        char_width: 0.55,
        weights: (100, 900),
    },
    FontInfo {
        key: "dm-serif-display",
        name: "DM Serif Display",
        category: FontCategory::Display,
        family: "DM Serif Display",
        stack: "'DM Serif Display', Georgia, 'Times New Roman', serif",
        char_width: 0.475,
        weights: (400, 400),
    },
    FontInfo {
        key: "bebas-neue",
        name: "Bebas Neue",
        category: FontCategory::Display,
        family: "Bebas Neue",
        stack: "'Bebas Neue', Impact, 'Arial Narrow', sans-serif",
        char_width: 0.355,
        weights: (400, 400),
    },
    FontInfo {
        key: "archivo-black",
        name: "Archivo Black",
        category: FontCategory::Display,
        family: "Archivo Black",
        stack: "'Archivo Black', 'Arial Black', Impact, sans-serif",
        char_width: 0.59,
        weights: (400, 400),
    },
    FontInfo {
        key: "jetbrains-mono",
        name: "JetBrains Mono",
        category: FontCategory::Mono,
        family: "JetBrains Mono",
        stack: "'JetBrains Mono', ui-monospace, 'SF Mono', Menlo, Consolas, 'DejaVu Sans Mono', monospace",
        char_width: 0.6,
        weights: (100, 800),
    },
    FontInfo {
        key: "fira-code",
        name: "Fira Code",
        category: FontCategory::Mono,
        family: "Fira Code",
        stack: "'Fira Code', ui-monospace, 'SF Mono', Menlo, Consolas, 'DejaVu Sans Mono', monospace",
        char_width: 0.6,
        weights: (300, 700),
    },
    FontInfo {
        key: "geist-mono",
        name: "Geist Mono",
        category: FontCategory::Mono,
        family: "Geist Mono",
        stack: "'Geist Mono', ui-monospace, 'SF Mono', Menlo, Consolas, 'DejaVu Sans Mono', monospace",
        char_width: 0.6,
        weights: (100, 900),
    },
    FontInfo {
        key: "red-hat-mono",
        name: "Red Hat Mono",
        category: FontCategory::Mono,
        family: "Red Hat Mono",
        stack: "'Red Hat Mono', ui-monospace, 'SF Mono', Menlo, Consolas, 'DejaVu Sans Mono', monospace",
        char_width: 0.6,
        weights: (300, 700),
    },
];

/// The font for `key`, or the default if it is unknown (for example a font removed later).
pub fn font_info(key: &str) -> &'static FontInfo {
    FONTS
        .iter()
        .find(|f| f.key == key)
        .or_else(|| FONTS.iter().find(|f| f.key == DEFAULT_FONT))
        .expect("the default font is in FONTS")
}

/// How a pill looks. Sizes are in design units at the 1200x630 base; the renderer scales them
/// with the canvas, so one setting looks the same on every preset.
#[derive(Clone, Copy, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct PillStyle {
    /// Corner radius; anything above half the height is a full capsule.
    pub radius: f64,
    pub font_size: f64,
    /// Space between the pill edge and its dot, and its text and the right edge.
    pub pad_x: f64,
    pub height: f64,
    /// Space between neighbouring pills.
    pub gap: f64,
    pub border: f64,
    /// Strength of the tint inside the pill, 0 to 1.
    pub fill: f64,
}

impl Default for PillStyle {
    fn default() -> Self {
        Self { radius: 22.0, font_size: 20.0, pad_x: 18.0, height: 44.0, gap: 14.0, border: 1.5, fill: 0.06 }
    }
}

/// The top-right badge: a pill (its `gap` is unused) plus how far it is nudged from its corner.
#[derive(Clone, Copy, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct BadgeStyle {
    pub pill: PillStyle,
    /// Extra distance from the right edge's padding; negative moves it outward.
    pub offset_x: f64,
    /// Extra distance down from its default position; negative moves it up.
    pub offset_y: f64,
}

impl Default for BadgeStyle {
    fn default() -> Self {
        Self {
            pill: PillStyle { radius: 18.0, font_size: 16.0, pad_x: 16.0, height: 36.0, gap: 14.0, border: 1.5, fill: 0.15 },
            offset_x: 0.0,
            offset_y: 0.0,
        }
    }
}

/// The logo mark and the name beside it, in design units. In lockup mode these act as
/// multipliers on the automatic layout (defaults give 1x), and the name is one step bolder.
#[derive(Clone, Copy, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct MarkStyle {
    pub size: f64,
    /// Space between the mark and the name.
    pub gap: f64,
    pub name_size: f64,
    /// CSS font weight, 100 to 900.
    pub name_weight: f64,
    /// Space between the logo/name row and the next block.
    pub space_after: f64,
}

impl Default for MarkStyle {
    fn default() -> Self {
        Self { size: 48.0, gap: 16.0, name_size: 28.0, name_weight: 700.0, space_after: 48.0 }
    }
}

/// The small uppercase line above the header.
#[derive(Clone, Copy, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct TagStyle {
    pub size: f64,
    pub letter_spacing: f64,
    pub space_after: f64,
}

impl Default for TagStyle {
    fn default() -> Self {
        Self { size: 16.0, letter_spacing: 2.0, space_after: 20.0 }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct HeaderStyle {
    pub size: f64,
    pub weight: f64,
    /// Line height as a multiple of the font size.
    pub line_height: f64,
    pub space_after: f64,
}

impl Default for HeaderStyle {
    fn default() -> Self {
        Self { size: 56.0, weight: 800.0, line_height: 1.15, space_after: 20.0 }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct DescStyle {
    pub size: f64,
    pub weight: f64,
    pub line_height: f64,
    /// Text opacity, 0 to 1.
    pub opacity: f64,
    pub space_after: f64,
}

impl Default for DescStyle {
    fn default() -> Self {
        Self { size: 24.0, weight: 400.0, line_height: 1.5, opacity: 0.72, space_after: 36.0 }
    }
}

/// Per-element look. Defaults reproduce the original fixed design exactly.
#[derive(Clone, Copy, PartialEq, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Style {
    pub pill: PillStyle,
    pub badge: BadgeStyle,
    pub mark: MarkStyle,
    pub tag: TagStyle,
    pub header: HeaderStyle,
    pub desc: DescStyle,
}

#[derive(Clone, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Config {
    pub preset: Preset,
    pub bg_kind: BgKind,
    pub bg1: String,
    pub bg2: String,
    pub angle: u32,
    pub glow: bool,
    pub radius: u32,
    pub text: String,
    pub accent: String,
    /// Colour of `*emphasised*` header words; `None` follows the accent.
    pub highlight: Option<String>,
    /// Font for the name and header (a `FONTS` key).
    pub heading_font: String,
    /// Font for everything else (a `FONTS` key).
    pub body_font: String,

    pub mark: Mark,
    /// `data:` URL of an uploaded logo, used when `mark == Mark::Image`.
    pub logo: Option<String>,
    pub name: String,
    pub tag: String,
    pub header: String,
    pub description: String,
    pub badge: String,
    /// One feature per line.
    pub features: String,
    pub layout: FeatureLayout,
    pub style: Style,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            preset: Preset::Social,
            bg_kind: BgKind::Grid,
            bg1: "#0a0d14".into(),
            bg2: "#1b2433".into(),
            angle: 135,
            glow: true,
            radius: 0,
            text: "#f4f1ea".into(),
            accent: "#d6a24b".into(),
            highlight: None,
            heading_font: DEFAULT_FONT.into(),
            body_font: DEFAULT_FONT.into(),

            mark: Mark::Sail,
            logo: None,
            name: "bannerbay".into(),
            tag: "Open source".into(),
            header: "Ship your *launch banner* in seconds".into(),
            description: "Design README banners and social cards in the browser. Pick a background, add your logo and copy, then export a crisp PNG or SVG.".into(),
            badge: "v0.1".into(),
            features: "Live preview\nPNG & SVG export\nNo sign-up".into(),
            layout: FeatureLayout::Horizontal,
            style: Style::default(),
        }
    }
}

impl Config {
    pub fn heading(&self) -> &'static FontInfo {
        font_info(&self.heading_font)
    }

    pub fn body(&self) -> &'static FontInfo {
        font_info(&self.body_font)
    }

    /// The colour for emphasised header words.
    pub fn highlight_color(&self) -> &str {
        self.highlight.as_deref().unwrap_or(&self.accent)
    }

    pub fn feature_list(&self) -> Vec<&str> {
        self.features.lines().map(str::trim).filter(|l| !l.is_empty()).collect()
    }

    /// A safe file stem for downloads, derived from the name.
    pub fn file_stem(&self) -> String {
        let slug: String = self
            .name
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
            .collect();
        let slug = slug.split('-').filter(|s| !s.is_empty()).collect::<Vec<_>>().join("-");
        if slug.is_empty() { "banner".into() } else { slug }
    }
}
