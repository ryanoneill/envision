//! Smart number formatting for chart labels.
//!
//! Provides human-readable formatting with SI suffixes (K, M, B),
//! scientific notation for very small values, and intelligent decimal
//! precision detection.

/// Formats a value with smart number formatting.
///
/// - Integers are formatted without decimals.
/// - Values >= 1,000,000,000 use "B" suffix.
/// - Values >= 1,000,000 use "M" suffix.
/// - Values >= 1,000 use "K" suffix.
/// - Values with absolute value < 0.001 (but non-zero) use scientific notation.
/// - Other values use the given precision (default 2).
///
/// # Example
///
/// ```ignore
/// use envision::component::chart::format::smart_format;
///
/// assert_eq!(smart_format(1_500_000.0, None), "1.5M");
/// assert_eq!(smart_format(42.0, None), "42");
/// assert_eq!(smart_format(0.0001, None), "1.0e-4");
/// ```
pub fn smart_format(value: f64, precision: Option<usize>) -> String {
    if value.is_nan() {
        return "NaN".to_string();
    }
    if value.is_infinite() {
        return if value.is_sign_positive() {
            "∞".to_string()
        } else {
            "-∞".to_string()
        };
    }

    let abs = value.abs();

    // Check if it's effectively an integer
    if is_integer(value) {
        return format_si(value);
    }

    // Very small non-zero values: use scientific notation
    if abs > 0.0 && abs < 0.001 {
        return format_scientific(value);
    }

    // Large values with SI suffixes
    if abs >= 1_000.0 {
        return format_with_suffix(value);
    }

    // Moderate decimal values
    let prec = precision.unwrap_or(2);
    let formatted = format!("{:.prec$}", value);
    // Trim trailing zeros after decimal point
    let trimmed = trim_trailing_zeros(&formatted);

    // If the value is non-zero but formatted to "0", use scientific notation
    // to avoid losing information
    if value != 0.0 && trimmed.trim_start_matches('-') == "0" {
        return format_scientific(value);
    }

    trimmed
}

/// Formats a value using SI suffixes (K, M, B).
fn format_with_suffix(value: f64) -> String {
    let abs = value.abs();
    let sign = if value < 0.0 { "-" } else { "" };

    if abs >= 1_000_000_000.0 {
        let scaled = abs / 1_000_000_000.0;
        let formatted = format!("{}{:.1}B", sign, scaled);
        trim_trailing_zeros_before_suffix(&formatted)
    } else if abs >= 1_000_000.0 {
        let scaled = abs / 1_000_000.0;
        let formatted = format!("{}{:.1}M", sign, scaled);
        trim_trailing_zeros_before_suffix(&formatted)
    } else {
        let scaled = abs / 1_000.0;
        let formatted = format!("{}{:.1}K", sign, scaled);
        trim_trailing_zeros_before_suffix(&formatted)
    }
}

/// Formats an integer value with SI suffixes if large enough.
fn format_si(value: f64) -> String {
    let abs = value.abs();
    if abs >= 1_000.0 {
        format_with_suffix(value)
    } else {
        format!("{}", value as i64)
    }
}

/// Formats very small values in scientific notation.
fn format_scientific(value: f64) -> String {
    let abs = value.abs();
    let sign = if value < 0.0 { "-" } else { "" };
    let exp = abs.log10().floor() as i32;
    let mantissa = abs / 10f64.powi(exp);
    let formatted = format!("{:.1}", mantissa);
    let trimmed = trim_trailing_zeros(&formatted);
    format!("{}{}e{}", sign, trimmed, exp)
}

/// Returns true if the value is effectively an integer.
fn is_integer(value: f64) -> bool {
    let abs = value.abs();
    // Check if the value is close enough to its rounded version
    (abs - abs.round()).abs() < 1e-9
}

/// Trims trailing zeros from a decimal string.
fn trim_trailing_zeros(s: &str) -> String {
    if let Some(dot_pos) = s.find('.') {
        let trimmed = s.trim_end_matches('0');
        if trimmed.ends_with('.') {
            trimmed[..dot_pos].to_string()
        } else {
            trimmed.to_string()
        }
    } else {
        s.to_string()
    }
}

/// Trims trailing zeros from a decimal string that ends with a suffix letter.
fn trim_trailing_zeros_before_suffix(s: &str) -> String {
    if s.len() < 2 {
        return s.to_string();
    }
    let suffix = &s[s.len() - 1..];
    let numeric = &s[..s.len() - 1];
    if let Some(dot_pos) = numeric.find('.') {
        let trimmed = numeric.trim_end_matches('0');
        if trimmed.ends_with('.') {
            format!("{}{}", &trimmed[..dot_pos], suffix)
        } else {
            format!("{}{}", trimmed, suffix)
        }
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Integer formatting
    // =========================================================================

    #[test]
    fn test_integer_zero() {
        assert_eq!(smart_format(0.0, None), "0");
    }

    #[test]
    fn test_integer_positive() {
        assert_eq!(smart_format(42.0, None), "42");
    }

    #[test]
    fn test_integer_negative() {
        assert_eq!(smart_format(-7.0, None), "-7");
    }

    #[test]
    fn test_integer_large_below_si() {
        assert_eq!(smart_format(999.0, None), "999");
    }

    // =========================================================================
    // SI suffix formatting
    // =========================================================================

    #[test]
    fn test_si_thousands() {
        assert_eq!(smart_format(1_000.0, None), "1K");
    }

    #[test]
    fn test_si_thousands_fractional() {
        assert_eq!(smart_format(15_500.0, None), "15.5K");
    }

    #[test]
    fn test_si_millions() {
        assert_eq!(smart_format(1_000_000.0, None), "1M");
    }

    #[test]
    fn test_si_millions_fractional() {
        assert_eq!(smart_format(1_500_000.0, None), "1.5M");
    }

    #[test]
    fn test_si_billions() {
        assert_eq!(smart_format(1_000_000_000.0, None), "1B");
    }

    #[test]
    fn test_si_billions_fractional() {
        assert_eq!(smart_format(2_500_000_000.0, None), "2.5B");
    }

    #[test]
    fn test_si_negative_thousands() {
        assert_eq!(smart_format(-50_000.0, None), "-50K");
    }

    // =========================================================================
    // Scientific notation
    // =========================================================================

    #[test]
    fn test_scientific_small() {
        assert_eq!(smart_format(0.0001, None), "1e-4");
    }

    #[test]
    fn test_scientific_very_small() {
        assert_eq!(smart_format(0.000005, None), "5e-6");
    }

    #[test]
    fn test_scientific_negative_small() {
        assert_eq!(smart_format(-0.0001, None), "-1e-4");
    }

    // =========================================================================
    // Moderate decimal values
    // =========================================================================

    #[test]
    fn test_moderate_decimal() {
        assert_eq!(smart_format(3.17, None), "3.17");
    }

    #[test]
    fn test_moderate_decimal_trailing_zeros() {
        assert_eq!(smart_format(2.10, None), "2.1");
    }

    #[test]
    fn test_moderate_decimal_with_precision() {
        assert_eq!(smart_format(1.23456, Some(4)), "1.2346");
    }

    // =========================================================================
    // Edge cases
    // =========================================================================

    #[test]
    fn test_nan() {
        assert_eq!(smart_format(f64::NAN, None), "NaN");
    }

    #[test]
    fn test_infinity() {
        assert_eq!(smart_format(f64::INFINITY, None), "∞");
    }

    #[test]
    fn test_negative_infinity() {
        assert_eq!(smart_format(f64::NEG_INFINITY, None), "-∞");
    }

    #[test]
    fn test_one() {
        assert_eq!(smart_format(1.0, None), "1");
    }

    #[test]
    fn test_negative_one() {
        assert_eq!(smart_format(-1.0, None), "-1");
    }

    #[test]
    fn test_just_above_threshold() {
        // 0.001 would format to "0" with precision 2, so use scientific notation
        assert_eq!(smart_format(0.001, None), "1e-3");
    }

    #[test]
    fn test_just_below_si_threshold() {
        assert_eq!(smart_format(999.99, None), "999.99");
    }
}
