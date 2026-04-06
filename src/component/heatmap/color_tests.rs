use super::*;
use ratatui::style::Color;

// =============================================================================
// Color mapping: GreenToRed
// =============================================================================

#[test]
fn test_green_to_red_at_zero() {
    let color = value_to_color(0.0, 0.0, 1.0, &HeatmapColorScale::GreenToRed);
    assert_eq!(color, Color::Rgb(0, 255, 0));
}

#[test]
fn test_green_to_red_at_half() {
    let color = value_to_color(0.5, 0.0, 1.0, &HeatmapColorScale::GreenToRed);
    assert_eq!(color, Color::Rgb(255, 255, 0)); // yellow
}

#[test]
fn test_green_to_red_at_one() {
    let color = value_to_color(1.0, 0.0, 1.0, &HeatmapColorScale::GreenToRed);
    assert_eq!(color, Color::Rgb(255, 0, 0));
}

// =============================================================================
// Color mapping: BlueToRed
// =============================================================================

#[test]
fn test_blue_to_red_at_zero() {
    let color = value_to_color(0.0, 0.0, 1.0, &HeatmapColorScale::BlueToRed);
    assert_eq!(color, Color::Rgb(0, 0, 255));
}

#[test]
fn test_blue_to_red_at_half() {
    let color = value_to_color(0.5, 0.0, 1.0, &HeatmapColorScale::BlueToRed);
    assert_eq!(color, Color::Rgb(255, 0, 255)); // magenta
}

#[test]
fn test_blue_to_red_at_one() {
    let color = value_to_color(1.0, 0.0, 1.0, &HeatmapColorScale::BlueToRed);
    assert_eq!(color, Color::Rgb(255, 0, 0));
}

// =============================================================================
// Color mapping: CoolToWarm
// =============================================================================

#[test]
fn test_cool_to_warm_at_zero() {
    let color = value_to_color(0.0, 0.0, 1.0, &HeatmapColorScale::CoolToWarm);
    assert_eq!(color, Color::Rgb(0, 0, 200));
}

#[test]
fn test_cool_to_warm_at_half() {
    let color = value_to_color(0.5, 0.0, 1.0, &HeatmapColorScale::CoolToWarm);
    assert_eq!(color, Color::Rgb(200, 200, 200)); // gray
}

#[test]
fn test_cool_to_warm_at_one() {
    let color = value_to_color(1.0, 0.0, 1.0, &HeatmapColorScale::CoolToWarm);
    assert_eq!(color, Color::Rgb(200, 200, 0));
}

// =============================================================================
// Color mapping: Intensity
// =============================================================================

#[test]
fn test_intensity_at_zero() {
    let color = value_to_color(0.0, 0.0, 1.0, &HeatmapColorScale::Intensity(Color::Cyan));
    // Cyan = (0, 255, 255) at 20% brightness
    assert_eq!(color, Color::Rgb(0, 51, 51));
}

#[test]
fn test_intensity_at_one() {
    let color = value_to_color(1.0, 0.0, 1.0, &HeatmapColorScale::Intensity(Color::Cyan));
    // Cyan = (0, 255, 255) at full brightness
    assert_eq!(color, Color::Rgb(0, 255, 255));
}

#[test]
fn test_intensity_with_rgb_color() {
    let color = value_to_color(
        1.0,
        0.0,
        1.0,
        &HeatmapColorScale::Intensity(Color::Rgb(100, 200, 50)),
    );
    assert_eq!(color, Color::Rgb(100, 200, 50));
}

// =============================================================================
// Color mapping: edge cases
// =============================================================================

#[test]
fn test_value_to_color_equal_min_max() {
    // When min == max, t should be 0.5
    let color = value_to_color(5.0, 5.0, 5.0, &HeatmapColorScale::GreenToRed);
    // t=0.5 => yellow
    assert_eq!(color, Color::Rgb(255, 255, 0));
}

#[test]
fn test_value_to_color_clamped_above() {
    let color = value_to_color(2.0, 0.0, 1.0, &HeatmapColorScale::GreenToRed);
    // Value above max is clamped to 1.0 => red
    assert_eq!(color, Color::Rgb(255, 0, 0));
}

#[test]
fn test_value_to_color_clamped_below() {
    let color = value_to_color(-1.0, 0.0, 1.0, &HeatmapColorScale::GreenToRed);
    // Value below min is clamped to 0.0 => green
    assert_eq!(color, Color::Rgb(0, 255, 0));
}

// =============================================================================
// Diverging color scales: BlueWhiteRed
// =============================================================================

#[test]
fn test_blue_white_red_negative_is_blue() {
    let color = value_to_color(-1.0, -1.0, 1.0, &HeatmapColorScale::BlueWhiteRed);
    assert_eq!(color, Color::Rgb(0, 0, 255));
}

#[test]
fn test_blue_white_red_zero_is_white() {
    let color = value_to_color(0.0, -1.0, 1.0, &HeatmapColorScale::BlueWhiteRed);
    assert_eq!(color, Color::Rgb(255, 255, 255));
}

#[test]
fn test_blue_white_red_positive_is_red() {
    let color = value_to_color(1.0, -1.0, 1.0, &HeatmapColorScale::BlueWhiteRed);
    assert_eq!(color, Color::Rgb(255, 0, 0));
}

#[test]
fn test_blue_white_red_quarter_negative() {
    // t = 0.25 => halfway between blue and white
    let color = value_to_color(-0.5, -1.0, 1.0, &HeatmapColorScale::BlueWhiteRed);
    assert_eq!(color, Color::Rgb(127, 127, 255));
}

#[test]
fn test_blue_white_red_quarter_positive() {
    // t = 0.75 => halfway between white and red
    let color = value_to_color(0.5, -1.0, 1.0, &HeatmapColorScale::BlueWhiteRed);
    assert_eq!(color, Color::Rgb(255, 127, 127));
}

// =============================================================================
// Diverging color scales: RedWhiteBlue
// =============================================================================

#[test]
fn test_red_white_blue_negative_is_red() {
    let color = value_to_color(-1.0, -1.0, 1.0, &HeatmapColorScale::RedWhiteBlue);
    assert_eq!(color, Color::Rgb(255, 0, 0));
}

#[test]
fn test_red_white_blue_zero_is_white() {
    let color = value_to_color(0.0, -1.0, 1.0, &HeatmapColorScale::RedWhiteBlue);
    assert_eq!(color, Color::Rgb(255, 255, 255));
}

#[test]
fn test_red_white_blue_positive_is_blue() {
    let color = value_to_color(1.0, -1.0, 1.0, &HeatmapColorScale::RedWhiteBlue);
    assert_eq!(color, Color::Rgb(0, 0, 255));
}

#[test]
fn test_red_white_blue_quarter_negative() {
    // t = 0.25 => halfway between red and white
    let color = value_to_color(-0.5, -1.0, 1.0, &HeatmapColorScale::RedWhiteBlue);
    assert_eq!(color, Color::Rgb(255, 127, 127));
}

#[test]
fn test_red_white_blue_quarter_positive() {
    // t = 0.75 => halfway between white and blue
    let color = value_to_color(0.5, -1.0, 1.0, &HeatmapColorScale::RedWhiteBlue);
    assert_eq!(color, Color::Rgb(127, 127, 255));
}

// =============================================================================
// Diverging color scales: asymmetric range
// =============================================================================

#[test]
fn test_blue_white_red_asymmetric_range_min() {
    // Range: -0.5 to 2.0, value at min => pure blue
    let color = value_to_color(-0.5, -0.5, 2.0, &HeatmapColorScale::BlueWhiteRed);
    assert_eq!(color, Color::Rgb(0, 0, 255));
}

#[test]
fn test_blue_white_red_asymmetric_range_max() {
    // Range: -0.5 to 2.0, value at max => pure red
    let color = value_to_color(2.0, -0.5, 2.0, &HeatmapColorScale::BlueWhiteRed);
    assert_eq!(color, Color::Rgb(255, 0, 0));
}

#[test]
fn test_blue_white_red_asymmetric_range_midpoint() {
    // Range: -0.5 to 2.0, midpoint is 0.75 => white
    let color = value_to_color(0.75, -0.5, 2.0, &HeatmapColorScale::BlueWhiteRed);
    assert_eq!(color, Color::Rgb(255, 255, 255));
}

#[test]
fn test_red_white_blue_asymmetric_range_min() {
    let color = value_to_color(-0.5, -0.5, 2.0, &HeatmapColorScale::RedWhiteBlue);
    assert_eq!(color, Color::Rgb(255, 0, 0));
}

#[test]
fn test_red_white_blue_asymmetric_range_max() {
    let color = value_to_color(2.0, -0.5, 2.0, &HeatmapColorScale::RedWhiteBlue);
    assert_eq!(color, Color::Rgb(0, 0, 255));
}

// =============================================================================
// Diverging color scales: equal min/max
// =============================================================================

#[test]
fn test_blue_white_red_equal_range() {
    // When min == max, t should be 0.5 => white
    let color = value_to_color(5.0, 5.0, 5.0, &HeatmapColorScale::BlueWhiteRed);
    assert_eq!(color, Color::Rgb(255, 255, 255));
}

#[test]
fn test_red_white_blue_equal_range() {
    let color = value_to_color(5.0, 5.0, 5.0, &HeatmapColorScale::RedWhiteBlue);
    assert_eq!(color, Color::Rgb(255, 255, 255));
}

// =============================================================================
// Diverging color scales: clamping
// =============================================================================

#[test]
fn test_blue_white_red_clamped_below() {
    let color = value_to_color(-5.0, -1.0, 1.0, &HeatmapColorScale::BlueWhiteRed);
    assert_eq!(color, Color::Rgb(0, 0, 255));
}

#[test]
fn test_blue_white_red_clamped_above() {
    let color = value_to_color(5.0, -1.0, 1.0, &HeatmapColorScale::BlueWhiteRed);
    assert_eq!(color, Color::Rgb(255, 0, 0));
}

// =============================================================================
// Color scale equality
// =============================================================================

#[test]
fn test_color_scale_default_is_green_to_red() {
    assert_eq!(HeatmapColorScale::default(), HeatmapColorScale::GreenToRed);
}

#[test]
fn test_color_scale_partial_eq() {
    assert_eq!(HeatmapColorScale::BlueToRed, HeatmapColorScale::BlueToRed);
    assert_ne!(HeatmapColorScale::BlueToRed, HeatmapColorScale::GreenToRed);
    assert_eq!(
        HeatmapColorScale::Intensity(Color::Red),
        HeatmapColorScale::Intensity(Color::Red)
    );
    assert_ne!(
        HeatmapColorScale::Intensity(Color::Red),
        HeatmapColorScale::Intensity(Color::Blue)
    );
}

#[test]
fn test_color_scale_diverging_eq() {
    assert_eq!(
        HeatmapColorScale::BlueWhiteRed,
        HeatmapColorScale::BlueWhiteRed
    );
    assert_eq!(
        HeatmapColorScale::RedWhiteBlue,
        HeatmapColorScale::RedWhiteBlue
    );
    assert_ne!(
        HeatmapColorScale::BlueWhiteRed,
        HeatmapColorScale::RedWhiteBlue
    );
}
