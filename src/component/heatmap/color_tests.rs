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
// Perceptual color scales: Viridis
// =============================================================================

#[test]
fn test_viridis_at_zero() {
    let color = value_to_color(0.0, 0.0, 1.0, &HeatmapColorScale::Viridis);
    assert_eq!(color, Color::Rgb(68, 1, 84));
}

#[test]
fn test_viridis_at_one() {
    let color = value_to_color(1.0, 0.0, 1.0, &HeatmapColorScale::Viridis);
    assert_eq!(color, Color::Rgb(253, 231, 37));
}

#[test]
fn test_viridis_at_midpoint() {
    // t=0.5, index=7.5 => lerp between LUT[7] and LUT[8]
    // LUT[7] = (31, 135, 141), LUT[8] = (30, 150, 138)
    // frac = 0.5
    // r = lerp(31, 30, 0.5) = 30.5 => 31
    // g = lerp(135, 150, 0.5) = 142.5 => 143
    // b = lerp(141, 138, 0.5) = 139.5 => 140
    let color = value_to_color(0.5, 0.0, 1.0, &HeatmapColorScale::Viridis);
    assert_eq!(color, Color::Rgb(31, 143, 140));
}

#[test]
fn test_viridis_interpolation_smoothness() {
    // Sample many points and verify no channel jumps more than a reasonable amount
    let scale = HeatmapColorScale::Viridis;
    let steps = 100;
    let mut prev: Option<(u8, u8, u8)> = None;

    for i in 0..=steps {
        let t = i as f64 / steps as f64;
        let color = value_to_color(t, 0.0, 1.0, &scale);
        if let Color::Rgb(r, g, b) = color {
            if let Some((pr, pg, pb)) = prev {
                let dr = (r as i16 - pr as i16).unsigned_abs();
                let dg = (g as i16 - pg as i16).unsigned_abs();
                let db = (b as i16 - pb as i16).unsigned_abs();
                // With 16 LUT entries and 100 steps, max per-step change should be small
                assert!(
                    dr <= 10,
                    "Viridis red channel jump too large at step {i}: {dr}"
                );
                assert!(
                    dg <= 10,
                    "Viridis green channel jump too large at step {i}: {dg}"
                );
                assert!(
                    db <= 10,
                    "Viridis blue channel jump too large at step {i}: {db}"
                );
            }
            prev = Some((r, g, b));
        } else {
            panic!("Expected Color::Rgb from Viridis scale");
        }
    }
}

#[test]
fn test_viridis_clamped_below() {
    let color = value_to_color(-5.0, 0.0, 1.0, &HeatmapColorScale::Viridis);
    assert_eq!(color, Color::Rgb(68, 1, 84));
}

#[test]
fn test_viridis_clamped_above() {
    let color = value_to_color(5.0, 0.0, 1.0, &HeatmapColorScale::Viridis);
    assert_eq!(color, Color::Rgb(253, 231, 37));
}

#[test]
fn test_viridis_equal_range() {
    // When min == max, t = 0.5
    let color = value_to_color(5.0, 5.0, 5.0, &HeatmapColorScale::Viridis);
    assert_eq!(color, Color::Rgb(31, 143, 140));
}

// =============================================================================
// Perceptual color scales: Inferno
// =============================================================================

#[test]
fn test_inferno_at_zero() {
    let color = value_to_color(0.0, 0.0, 1.0, &HeatmapColorScale::Inferno);
    assert_eq!(color, Color::Rgb(0, 0, 4));
}

#[test]
fn test_inferno_at_one() {
    let color = value_to_color(1.0, 0.0, 1.0, &HeatmapColorScale::Inferno);
    assert_eq!(color, Color::Rgb(252, 255, 164));
}

#[test]
fn test_inferno_at_midpoint() {
    // t=0.5, index=7.5 => lerp between LUT[7] and LUT[8]
    // LUT[7] = (165, 44, 81), LUT[8] = (187, 65, 58)
    // frac = 0.5
    // r = lerp(165, 187, 0.5) = 176
    // g = lerp(44, 65, 0.5) = 54.5 => 55
    // b = lerp(81, 58, 0.5) = 69.5 => 70
    let color = value_to_color(0.5, 0.0, 1.0, &HeatmapColorScale::Inferno);
    assert_eq!(color, Color::Rgb(176, 55, 70));
}

#[test]
fn test_inferno_interpolation_smoothness() {
    let scale = HeatmapColorScale::Inferno;
    let steps = 100;
    let mut prev: Option<(u8, u8, u8)> = None;
    // Inferno has steep gradients in the blue channel near the top of the range,
    // so we allow a per-step delta of up to 12 with a 16-entry LUT.
    let max_delta: u16 = 12;

    for i in 0..=steps {
        let t = i as f64 / steps as f64;
        let color = value_to_color(t, 0.0, 1.0, &scale);
        if let Color::Rgb(r, g, b) = color {
            if let Some((pr, pg, pb)) = prev {
                let dr = (r as i16 - pr as i16).unsigned_abs();
                let dg = (g as i16 - pg as i16).unsigned_abs();
                let db = (b as i16 - pb as i16).unsigned_abs();
                assert!(
                    dr <= max_delta,
                    "Inferno red channel jump too large at step {i}: {dr}"
                );
                assert!(
                    dg <= max_delta,
                    "Inferno green channel jump too large at step {i}: {dg}"
                );
                assert!(
                    db <= max_delta,
                    "Inferno blue channel jump too large at step {i}: {db}"
                );
            }
            prev = Some((r, g, b));
        } else {
            panic!("Expected Color::Rgb from Inferno scale");
        }
    }
}

#[test]
fn test_inferno_clamped_below() {
    let color = value_to_color(-5.0, 0.0, 1.0, &HeatmapColorScale::Inferno);
    assert_eq!(color, Color::Rgb(0, 0, 4));
}

#[test]
fn test_inferno_clamped_above() {
    let color = value_to_color(5.0, 0.0, 1.0, &HeatmapColorScale::Inferno);
    assert_eq!(color, Color::Rgb(252, 255, 164));
}

#[test]
fn test_inferno_equal_range() {
    let color = value_to_color(5.0, 5.0, 5.0, &HeatmapColorScale::Inferno);
    assert_eq!(color, Color::Rgb(176, 55, 70));
}

// =============================================================================
// Perceptual color scales: Plasma
// =============================================================================

#[test]
fn test_plasma_at_zero() {
    let color = value_to_color(0.0, 0.0, 1.0, &HeatmapColorScale::Plasma);
    assert_eq!(color, Color::Rgb(13, 8, 135));
}

#[test]
fn test_plasma_at_one() {
    let color = value_to_color(1.0, 0.0, 1.0, &HeatmapColorScale::Plasma);
    assert_eq!(color, Color::Rgb(240, 249, 33));
}

#[test]
fn test_plasma_at_midpoint() {
    // t=0.5, index=7.5 => lerp between LUT[7] and LUT[8]
    // LUT[7] = (195, 63, 126), LUT[8] = (210, 84, 107)
    // frac = 0.5
    // r = lerp(195, 210, 0.5) = 202.5 => 203
    // g = lerp(63, 84, 0.5) = 73.5 => 74
    // b = lerp(126, 107, 0.5) = 116.5 => 117
    let color = value_to_color(0.5, 0.0, 1.0, &HeatmapColorScale::Plasma);
    assert_eq!(color, Color::Rgb(203, 74, 117));
}

#[test]
fn test_plasma_interpolation_smoothness() {
    let scale = HeatmapColorScale::Plasma;
    let steps = 100;
    let mut prev: Option<(u8, u8, u8)> = None;

    for i in 0..=steps {
        let t = i as f64 / steps as f64;
        let color = value_to_color(t, 0.0, 1.0, &scale);
        if let Color::Rgb(r, g, b) = color {
            if let Some((pr, pg, pb)) = prev {
                let dr = (r as i16 - pr as i16).unsigned_abs();
                let dg = (g as i16 - pg as i16).unsigned_abs();
                let db = (b as i16 - pb as i16).unsigned_abs();
                assert!(
                    dr <= 10,
                    "Plasma red channel jump too large at step {i}: {dr}"
                );
                assert!(
                    dg <= 10,
                    "Plasma green channel jump too large at step {i}: {dg}"
                );
                assert!(
                    db <= 10,
                    "Plasma blue channel jump too large at step {i}: {db}"
                );
            }
            prev = Some((r, g, b));
        } else {
            panic!("Expected Color::Rgb from Plasma scale");
        }
    }
}

#[test]
fn test_plasma_clamped_below() {
    let color = value_to_color(-5.0, 0.0, 1.0, &HeatmapColorScale::Plasma);
    assert_eq!(color, Color::Rgb(13, 8, 135));
}

#[test]
fn test_plasma_clamped_above() {
    let color = value_to_color(5.0, 0.0, 1.0, &HeatmapColorScale::Plasma);
    assert_eq!(color, Color::Rgb(240, 249, 33));
}

#[test]
fn test_plasma_equal_range() {
    let color = value_to_color(5.0, 5.0, 5.0, &HeatmapColorScale::Plasma);
    assert_eq!(color, Color::Rgb(203, 74, 117));
}

// =============================================================================
// Perceptual color scales: LUT boundary tests
// =============================================================================

#[test]
fn test_viridis_at_exact_lut_entry() {
    // t = 1/15 should hit LUT[1] exactly
    let color = value_to_color(1.0 / 15.0, 0.0, 1.0, &HeatmapColorScale::Viridis);
    assert_eq!(color, Color::Rgb(72, 26, 108));
}

#[test]
fn test_inferno_at_exact_lut_entry() {
    // t = 1/15 should hit LUT[1] exactly
    let color = value_to_color(1.0 / 15.0, 0.0, 1.0, &HeatmapColorScale::Inferno);
    assert_eq!(color, Color::Rgb(11, 7, 36));
}

#[test]
fn test_plasma_at_exact_lut_entry() {
    // t = 1/15 should hit LUT[1] exactly
    let color = value_to_color(1.0 / 15.0, 0.0, 1.0, &HeatmapColorScale::Plasma);
    assert_eq!(color, Color::Rgb(49, 4, 150));
}

// =============================================================================
// Perceptual color scales: equality
// =============================================================================

#[test]
fn test_perceptual_scale_equality() {
    assert_eq!(HeatmapColorScale::Viridis, HeatmapColorScale::Viridis);
    assert_eq!(HeatmapColorScale::Inferno, HeatmapColorScale::Inferno);
    assert_eq!(HeatmapColorScale::Plasma, HeatmapColorScale::Plasma);
    assert_ne!(HeatmapColorScale::Viridis, HeatmapColorScale::Inferno);
    assert_ne!(HeatmapColorScale::Viridis, HeatmapColorScale::Plasma);
    assert_ne!(HeatmapColorScale::Inferno, HeatmapColorScale::Plasma);
    assert_ne!(HeatmapColorScale::Viridis, HeatmapColorScale::GreenToRed);
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
