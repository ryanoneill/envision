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
        HeatmapColorScale::BlueWhiteRed => diverging_color(t, 0, 0, 255, 255, 0, 0),
        HeatmapColorScale::RedWhiteBlue => diverging_color(t, 255, 0, 0, 0, 0, 255),
    }
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
