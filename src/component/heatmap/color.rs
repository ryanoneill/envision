//! Color scale definitions and value-to-color mapping for the Heatmap component.

use ratatui::style::Color;

/// Color scale for the heatmap.
///
/// Determines how values are mapped to colors. Each variant defines a
/// different color gradient from low to high values.
///
/// # Example
///
/// ```rust
/// use envision::component::HeatmapColorScale;
/// use ratatui::style::Color;
///
/// let scale = HeatmapColorScale::GreenToRed;
/// assert_eq!(scale, HeatmapColorScale::default());
///
/// let custom = HeatmapColorScale::Intensity(Color::Cyan);
/// assert_ne!(custom, HeatmapColorScale::GreenToRed);
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum HeatmapColorScale {
    /// Green (low) to Red (high), passing through yellow.
    #[default]
    GreenToRed,
    /// Blue (low) to Red (high), passing through magenta.
    BlueToRed,
    /// Cool blue (low) to warm yellow (high), passing through gray.
    CoolToWarm,
    /// Single color with varying intensity (dim to bright).
    Intensity(Color),
    /// Perceptually uniform Viridis color scale (purple to yellow).
    ///
    /// Based on the matplotlib Viridis colormap, this scale provides
    /// perceptually uniform color mapping ideal for scientific visualization.
    /// The colors remain distinguishable under common forms of color blindness.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{HeatmapColorScale, value_to_color};
    /// use ratatui::style::Color;
    ///
    /// let color = value_to_color(0.0, 0.0, 1.0, &HeatmapColorScale::Viridis);
    /// assert_eq!(color, Color::Rgb(68, 1, 84));
    ///
    /// let color = value_to_color(1.0, 0.0, 1.0, &HeatmapColorScale::Viridis);
    /// assert_eq!(color, Color::Rgb(253, 231, 37));
    /// ```
    Viridis,
    /// Perceptually uniform Inferno color scale (black to yellow).
    ///
    /// Based on the matplotlib Inferno colormap, this scale ranges from
    /// near-black through reds and oranges to bright yellow. Perceptually
    /// uniform and colorblind-friendly.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{HeatmapColorScale, value_to_color};
    /// use ratatui::style::Color;
    ///
    /// let color = value_to_color(0.0, 0.0, 1.0, &HeatmapColorScale::Inferno);
    /// assert_eq!(color, Color::Rgb(0, 0, 4));
    ///
    /// let color = value_to_color(1.0, 0.0, 1.0, &HeatmapColorScale::Inferno);
    /// assert_eq!(color, Color::Rgb(252, 255, 164));
    /// ```
    Inferno,
    /// Perceptually uniform Plasma color scale (purple to yellow).
    ///
    /// Based on the matplotlib Plasma colormap, this scale ranges from
    /// deep purple through pinks and oranges to bright yellow. Perceptually
    /// uniform and colorblind-friendly.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{HeatmapColorScale, value_to_color};
    /// use ratatui::style::Color;
    ///
    /// let color = value_to_color(0.0, 0.0, 1.0, &HeatmapColorScale::Plasma);
    /// assert_eq!(color, Color::Rgb(13, 8, 135));
    ///
    /// let color = value_to_color(1.0, 0.0, 1.0, &HeatmapColorScale::Plasma);
    /// assert_eq!(color, Color::Rgb(240, 249, 33));
    /// ```
    Plasma,
    /// Blue (negative) to White (zero) to Red (positive) diverging scale.
    ///
    /// Ideal for ML visualizations such as attention maps, weight matrices,
    /// and correlation matrices where the midpoint represents zero or neutral.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{HeatmapColorScale, value_to_color};
    /// use ratatui::style::Color;
    ///
    /// // Negative value maps to blue
    /// let color = value_to_color(-1.0, -1.0, 1.0, &HeatmapColorScale::BlueWhiteRed);
    /// assert_eq!(color, Color::Rgb(0, 0, 255));
    ///
    /// // Zero maps to white
    /// let color = value_to_color(0.0, -1.0, 1.0, &HeatmapColorScale::BlueWhiteRed);
    /// assert_eq!(color, Color::Rgb(255, 255, 255));
    ///
    /// // Positive value maps to red
    /// let color = value_to_color(1.0, -1.0, 1.0, &HeatmapColorScale::BlueWhiteRed);
    /// assert_eq!(color, Color::Rgb(255, 0, 0));
    /// ```
    BlueWhiteRed,
    /// Red (negative) to White (zero) to Blue (positive) diverging scale.
    ///
    /// The reverse of [`BlueWhiteRed`](Self::BlueWhiteRed), useful when the
    /// convention is red-negative / blue-positive (e.g., temperature anomalies).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{HeatmapColorScale, value_to_color};
    /// use ratatui::style::Color;
    ///
    /// // Negative value maps to red
    /// let color = value_to_color(-1.0, -1.0, 1.0, &HeatmapColorScale::RedWhiteBlue);
    /// assert_eq!(color, Color::Rgb(255, 0, 0));
    ///
    /// // Zero maps to white
    /// let color = value_to_color(0.0, -1.0, 1.0, &HeatmapColorScale::RedWhiteBlue);
    /// assert_eq!(color, Color::Rgb(255, 255, 255));
    ///
    /// // Positive value maps to blue
    /// let color = value_to_color(1.0, -1.0, 1.0, &HeatmapColorScale::RedWhiteBlue);
    /// assert_eq!(color, Color::Rgb(0, 0, 255));
    /// ```
    RedWhiteBlue,
}

/// Maps a value to a color based on the heatmap color scale.
///
/// The value is normalized to the range [0.0, 1.0] based on the given
/// min and max, then mapped to an RGB color according to the scale.
///
/// # Example
///
/// ```rust
/// use envision::component::{HeatmapColorScale, value_to_color};
/// use ratatui::style::Color;
///
/// let color = value_to_color(0.0, 0.0, 1.0, &HeatmapColorScale::GreenToRed);
/// assert_eq!(color, Color::Rgb(0, 255, 0));
///
/// let color = value_to_color(1.0, 0.0, 1.0, &HeatmapColorScale::GreenToRed);
/// assert_eq!(color, Color::Rgb(255, 0, 0));
/// ```
pub fn value_to_color(value: f64, min: f64, max: f64, scale: &HeatmapColorScale) -> Color {
    let t = if (max - min).abs() < f64::EPSILON {
        0.5
    } else {
        ((value - min) / (max - min)).clamp(0.0, 1.0)
    };

    match scale {
        HeatmapColorScale::GreenToRed => {
            // Green -> Yellow -> Red
            if t <= 0.5 {
                let s = t * 2.0; // 0..1 within first half
                let r = (255.0 * s) as u8;
                let g = 255u8;
                Color::Rgb(r, g, 0)
            } else {
                let s = (t - 0.5) * 2.0; // 0..1 within second half
                let r = 255u8;
                let g = (255.0 * (1.0 - s)) as u8;
                Color::Rgb(r, g, 0)
            }
        }
        HeatmapColorScale::BlueToRed => {
            // Blue -> Magenta -> Red
            if t <= 0.5 {
                let s = t * 2.0;
                let r = (255.0 * s) as u8;
                let b = 255u8;
                Color::Rgb(r, 0, b)
            } else {
                let s = (t - 0.5) * 2.0;
                let r = 255u8;
                let b = (255.0 * (1.0 - s)) as u8;
                Color::Rgb(r, 0, b)
            }
        }
        HeatmapColorScale::CoolToWarm => {
            // Blue(0,0,200) -> Gray(200,200,200) -> Yellow(200,200,0)
            if t <= 0.5 {
                let s = t * 2.0;
                let r = (200.0 * s) as u8;
                let g = (200.0 * s) as u8;
                let b = 200u8;
                Color::Rgb(r, g, b)
            } else {
                let s = (t - 0.5) * 2.0;
                let r = 200u8;
                let g = 200u8;
                let b = (200.0 * (1.0 - s)) as u8;
                Color::Rgb(r, g, b)
            }
        }
        HeatmapColorScale::Intensity(base_color) => {
            // Extract the base RGB and scale brightness
            let (br, bg, bb) = match base_color {
                Color::Rgb(r, g, b) => (*r, *g, *b),
                Color::Red => (255, 0, 0),
                Color::Green => (0, 255, 0),
                Color::Blue => (0, 0, 255),
                Color::Yellow => (255, 255, 0),
                Color::Cyan => (0, 255, 255),
                Color::Magenta => (255, 0, 255),
                Color::White => (255, 255, 255),
                _ => (128, 128, 128),
            };
            // Scale from dim (t=0) to full brightness (t=1)
            // Minimum brightness of ~20% so cells are always visible
            let factor = 0.2 + 0.8 * t;
            let r = (br as f64 * factor) as u8;
            let g = (bg as f64 * factor) as u8;
            let b = (bb as f64 * factor) as u8;
            Color::Rgb(r, g, b)
        }
        HeatmapColorScale::Viridis => lookup_color(&VIRIDIS_LUT, t),
        HeatmapColorScale::Inferno => lookup_color(&INFERNO_LUT, t),
        HeatmapColorScale::Plasma => lookup_color(&PLASMA_LUT, t),
        HeatmapColorScale::BlueWhiteRed => diverging_color(t, 0, 0, 255, 255, 0, 0),
        HeatmapColorScale::RedWhiteBlue => diverging_color(t, 255, 0, 0, 0, 0, 255),
    }
}

// =============================================================================
// Perceptually uniform color scale lookup tables
// =============================================================================

/// Viridis colormap: 16 evenly spaced samples from the matplotlib Viridis scale.
const VIRIDIS_LUT: [(u8, u8, u8); 16] = [
    (68, 1, 84),
    (72, 26, 108),
    (71, 47, 126),
    (65, 68, 135),
    (57, 86, 140),
    (48, 103, 141),
    (39, 119, 142),
    (31, 135, 141),
    (30, 150, 138),
    (44, 166, 130),
    (73, 181, 117),
    (110, 196, 98),
    (155, 208, 72),
    (199, 217, 46),
    (238, 224, 29),
    (253, 231, 37),
];

/// Inferno colormap: 16 evenly spaced samples from the matplotlib Inferno scale.
const INFERNO_LUT: [(u8, u8, u8); 16] = [
    (0, 0, 4),
    (11, 7, 36),
    (32, 12, 74),
    (59, 15, 99),
    (87, 16, 110),
    (114, 17, 112),
    (140, 25, 101),
    (165, 44, 81),
    (187, 65, 58),
    (205, 92, 35),
    (219, 122, 12),
    (230, 155, 0),
    (237, 189, 12),
    (239, 222, 52),
    (237, 249, 121),
    (252, 255, 164),
];

/// Plasma colormap: 16 evenly spaced samples from the matplotlib Plasma scale.
const PLASMA_LUT: [(u8, u8, u8); 16] = [
    (13, 8, 135),
    (49, 4, 150),
    (80, 2, 162),
    (108, 1, 168),
    (134, 2, 166),
    (156, 23, 158),
    (177, 42, 144),
    (195, 63, 126),
    (210, 84, 107),
    (222, 107, 87),
    (231, 131, 67),
    (238, 157, 46),
    (242, 183, 28),
    (243, 210, 22),
    (238, 236, 38),
    (240, 249, 33),
];

/// Looks up a color from a lookup table with linear interpolation between entries.
///
/// `t` is a normalized value in [0.0, 1.0]. Values outside this range are clamped.
/// The function finds the two nearest LUT entries and linearly interpolates each
/// RGB channel between them.
fn lookup_color(lut: &[(u8, u8, u8)], t: f64) -> Color {
    let t = t.clamp(0.0, 1.0);
    let idx = t * (lut.len() - 1) as f64;
    let lo = idx.floor() as usize;
    let hi = (lo + 1).min(lut.len() - 1);
    let frac = idx - lo as f64;
    let (r1, g1, b1) = lut[lo];
    let (r2, g2, b2) = lut[hi];
    Color::Rgb(
        lerp_u8(r1, r2, frac),
        lerp_u8(g1, g2, frac),
        lerp_u8(b1, b2, frac),
    )
}

/// Linearly interpolates between two `u8` values.
///
/// `t` is the interpolation fraction in [0.0, 1.0].
fn lerp_u8(a: u8, b: u8, t: f64) -> u8 {
    (a as f64 + (b as f64 - a as f64) * t).round() as u8
}

/// Computes a diverging color for a normalized value `t` in [0.0, 1.0].
///
/// The scale interpolates from `(lo_r, lo_g, lo_b)` at `t=0.0` through
/// white `(255, 255, 255)` at `t=0.5` to `(hi_r, hi_g, hi_b)` at `t=1.0`.
fn diverging_color(t: f64, lo_r: u8, lo_g: u8, lo_b: u8, hi_r: u8, hi_g: u8, hi_b: u8) -> Color {
    if t <= 0.5 {
        // Interpolate from low color to white
        let s = t * 2.0; // 0.0..1.0 within first half
        let r = lo_r as f64 + (255.0 - lo_r as f64) * s;
        let g = lo_g as f64 + (255.0 - lo_g as f64) * s;
        let b = lo_b as f64 + (255.0 - lo_b as f64) * s;
        Color::Rgb(r as u8, g as u8, b as u8)
    } else {
        // Interpolate from white to high color
        let s = (t - 0.5) * 2.0; // 0.0..1.0 within second half
        let r = 255.0 + (hi_r as f64 - 255.0) * s;
        let g = 255.0 + (hi_g as f64 - 255.0) * s;
        let b = 255.0 + (hi_b as f64 - 255.0) * s;
        Color::Rgb(r as u8, g as u8, b as u8)
    }
}
