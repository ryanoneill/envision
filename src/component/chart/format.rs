//! Smart number formatting for chart axis labels and value display.
//!
//! Provides context-aware formatting that chooses the most readable
//! representation based on magnitude: SI suffixes for large numbers,
//! scientific notation for very small numbers, and integer display
//! when values are whole numbers.

/// Formats a numeric value for display on a chart axis or label.
///
/// Chooses the most readable format based on the value's magnitude:
/// - Integers: `"100"` not `"100.0"` when the value is whole
/// - SI suffixes: `"25K"`, `"1.5M"`, `"3.2B"` for large values
/// - Scientific notation: `"1.2e-4"` for very small values
/// - Decimal: `"3.14"` for moderate fractional values
///
/// The `precision` parameter controls decimal places for the fractional
/// part. When `None`, a default precision is chosen based on magnitude.
pub fn smart_format(value: f64, precision: Option<usize>) -> String {
    if !value.is_finite() {
        return format!("{value}");
    }

    let abs = value.abs();

    // Integer detection: if value is effectively a whole number
    if abs < 1e15 && (value - value.round()).abs() < 1e-9 {
        let int_val = value.round() as i64;
        return format_with_suffix(int_val as f64, 0);
    }

    // Very small non-zero values: use scientific notation
    if abs > 0.0 && abs < 0.001 {
        let prec = precision.unwrap_or(2);
        return format!("{:.prec$e}", value);
    }

    // Moderate fractional values: use decimal
    let prec = precision.unwrap_or(2);
    format_with_suffix(value, prec)
}

/// Formats a value with SI suffix if large enough, otherwise plain decimal.
fn format_with_suffix(value: f64, precision: usize) -> String {
    let abs = value.abs();

    if abs >= 1_000_000_000.0 {
        format_si(value, 1_000_000_000.0, "B", precision)
    } else if abs >= 1_000_000.0 {
        format_si(value, 1_000_000.0, "M", precision)
    } else if abs >= 10_000.0 {
        format_si(value, 1_000.0, "K", precision)
    } else if precision == 0 {
        format!("{}", value as i64)
    } else {
        format!("{:.prec$}", value, prec = precision)
    }
}

/// Formats a value divided by a scale factor with a suffix.
fn format_si(value: f64, divisor: f64, suffix: &str, precision: usize) -> String {
    let scaled = value / divisor;
    // Use integer format if the scaled value is whole
    if (scaled - scaled.round()).abs() < 0.05 && precision == 0 {
        format!("{}{}", scaled.round() as i64, suffix)
    } else {
        let prec = if precision == 0 { 1 } else { precision };
        let formatted = format!("{:.prec$}", scaled, prec = prec);
        // Trim trailing zeros after decimal point for cleanliness
        let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
        format!("{}{}", trimmed, suffix)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =============================================================================
    // Integer detection
    // =============================================================================

    #[test]
    fn test_integer_values() {
        assert_eq!(smart_format(0.0, None), "0");
        assert_eq!(smart_format(100.0, None), "100");
        assert_eq!(smart_format(-40.0, None), "-40");
        assert_eq!(smart_format(1.0, None), "1");
    }

    #[test]
    fn test_integer_from_float() {
        // Values that are effectively integers despite being f64
        assert_eq!(smart_format(5.0000000001, None), "5");
        assert_eq!(smart_format(99.9999999999, None), "100");
    }

    // =============================================================================
    // SI suffixes
    // =============================================================================

    #[test]
    fn test_thousands() {
        assert_eq!(smart_format(10000.0, None), "10K");
        assert_eq!(smart_format(25000.0, None), "25K");
        assert_eq!(smart_format(250000.0, None), "250K");
    }

    #[test]
    fn test_thousands_fractional() {
        assert_eq!(smart_format(15500.0, None), "15.5K");
    }

    #[test]
    fn test_millions() {
        assert_eq!(smart_format(1000000.0, None), "1M");
        assert_eq!(smart_format(2500000.0, None), "2.5M");
    }

    #[test]
    fn test_billions() {
        assert_eq!(smart_format(1000000000.0, None), "1B");
        assert_eq!(smart_format(7500000000.0, None), "7.5B");
    }

    #[test]
    fn test_negative_large() {
        assert_eq!(smart_format(-50000.0, None), "-50K");
        assert_eq!(smart_format(-2000000.0, None), "-2M");
    }

    // =============================================================================
    // Scientific notation for small values
    // =============================================================================

    #[test]
    fn test_small_values() {
        assert_eq!(smart_format(0.0001, None), "1.00e-4");
        assert_eq!(smart_format(0.00012, None), "1.20e-4");
    }

    #[test]
    fn test_very_small_values() {
        assert_eq!(smart_format(0.000001, None), "1.00e-6");
    }

    #[test]
    fn test_small_negative() {
        assert_eq!(smart_format(-0.0005, None), "-5.00e-4");
    }

    // =============================================================================
    // Moderate fractional values
    // =============================================================================

    #[test]
    fn test_moderate_decimals() {
        assert_eq!(smart_format(2.73, None), "2.73");
        assert_eq!(smart_format(0.5, None), "0.50");
    }

    #[test]
    fn test_moderate_with_precision() {
        assert_eq!(smart_format(1.23456, Some(3)), "1.235");
        assert_eq!(smart_format(1.23456, Some(1)), "1.2");
    }

    // =============================================================================
    // Edge cases
    // =============================================================================

    #[test]
    fn test_zero() {
        assert_eq!(smart_format(0.0, None), "0");
    }

    #[test]
    fn test_nan() {
        assert_eq!(smart_format(f64::NAN, None), "NaN");
    }

    #[test]
    fn test_infinity() {
        assert_eq!(smart_format(f64::INFINITY, None), "inf");
        assert_eq!(smart_format(f64::NEG_INFINITY, None), "-inf");
    }

    #[test]
    fn test_threshold_boundary_10000() {
        // 10000 is the threshold where K suffix kicks in
        assert_eq!(smart_format(10000.0, None), "10K");
        assert_eq!(smart_format(9999.0, None), "9999");
    }

    #[test]
    fn test_values_below_si_threshold() {
        assert_eq!(smart_format(5000.0, None), "5000");
        assert_eq!(smart_format(9000.0, None), "9000");
    }

    #[test]
    fn test_one_decimal_not_small() {
        // 0.1 is >= 0.001, so it should use decimal format, not scientific
        assert_eq!(smart_format(0.1, None), "0.10");
        assert_eq!(smart_format(0.01, None), "0.01");
    }
}
