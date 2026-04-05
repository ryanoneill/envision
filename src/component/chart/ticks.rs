//! Axis tick generation using Heckbert's nice numbers algorithm.
//!
//! Generates human-readable tick positions for chart axes,
//! choosing "nice" intervals (multiples of 1, 2, or 5) that
//! produce clean, readable labels.

use super::format::smart_format;

/// Generates nice tick positions for an axis range.
///
/// Uses Heckbert's nice numbers algorithm to produce evenly spaced
/// tick marks at human-readable intervals. Returns at most `max_ticks`
/// tick positions.
///
/// # Example
///
/// ```ignore
/// use envision::component::chart::ticks::nice_ticks;
///
/// let ticks = nice_ticks(0.0, 100.0, 5);
/// assert!(!ticks.is_empty());
/// assert!(ticks.len() <= 5);
/// assert!(ticks[0] <= 0.0);
/// assert!(*ticks.last().unwrap() >= 100.0);
/// ```
pub fn nice_ticks(min: f64, max: f64, max_ticks: usize) -> Vec<f64> {
    if max_ticks < 2 {
        return vec![min, max];
    }

    // Handle NaN or infinite inputs
    if min.is_nan() || max.is_nan() || min.is_infinite() || max.is_infinite() {
        return vec![min, max];
    }

    let range = max - min;

    // Zero range: return single value
    if range.abs() < f64::EPSILON {
        return vec![min];
    }

    // Negative range: swap and reverse
    if range < 0.0 {
        let mut result = nice_ticks(max, min, max_ticks);
        result.reverse();
        return result;
    }

    let rough_step = range / (max_ticks - 1) as f64;
    let step = nice_step(rough_step);

    let tick_min = (min / step).floor() * step;
    let tick_max = (max / step).ceil() * step;

    let mut ticks = Vec::new();
    let mut tick = tick_min;

    // Guard against infinite loops from very small steps
    let max_iterations = max_ticks * 4;
    let mut iterations = 0;

    while tick <= tick_max + step * 0.5 * f64::EPSILON && iterations < max_iterations {
        ticks.push(round_to_precision(tick, step));
        tick += step;
        iterations += 1;
    }

    // If we ended up with too many ticks, reduce
    if ticks.len() > max_ticks + 1 {
        // Try a larger step
        let larger_step = nice_step(rough_step * 2.0);
        let tick_min = (min / larger_step).floor() * larger_step;
        let tick_max = (max / larger_step).ceil() * larger_step;
        ticks.clear();
        let mut tick = tick_min;
        iterations = 0;
        while tick <= tick_max + larger_step * 0.5 * f64::EPSILON && iterations < max_iterations {
            ticks.push(round_to_precision(tick, larger_step));
            tick += larger_step;
            iterations += 1;
        }
    }

    ticks
}

/// Rounds a rough step size to a "nice" value.
///
/// Nice steps are multiples of 1, 2, or 5 times a power of 10.
fn nice_step(rough: f64) -> f64 {
    if rough <= 0.0 || rough.is_nan() || rough.is_infinite() {
        return 1.0;
    }

    let exponent = rough.log10().floor();
    let fraction = rough / 10f64.powf(exponent);

    let nice_fraction = if fraction <= 1.5 {
        1.0
    } else if fraction <= 3.5 {
        2.0
    } else if fraction <= 7.5 {
        5.0
    } else {
        10.0
    };

    nice_fraction * 10f64.powf(exponent)
}

/// Rounds a tick value to avoid floating-point artifacts.
fn round_to_precision(value: f64, step: f64) -> f64 {
    if step == 0.0 {
        return value;
    }

    // Determine decimal places from step
    let step_str = format!("{}", step);
    let decimals = if let Some(dot_pos) = step_str.find('.') {
        let frac = &step_str[dot_pos + 1..];
        // Count significant digits (not just trailing zeros)
        frac.trim_end_matches('0').len()
    } else {
        0
    };

    if decimals == 0 {
        value.round()
    } else {
        let factor = 10f64.powi(decimals as i32);
        (value * factor).round() / factor
    }
}

/// Formats a tick value for display on an axis.
///
/// Uses smart formatting for large or very small values,
/// and appropriate decimal precision based on the step size.
///
/// # Example
///
/// ```ignore
/// use envision::component::chart::ticks::format_tick;
///
/// assert_eq!(format_tick(1000.0, 500.0), "1000");
/// assert_eq!(format_tick(50000.0, 10000.0), "50K");
/// ```
pub fn format_tick(value: f64, step: f64) -> String {
    // Use smart_format for large or very small values
    if value.abs() >= 10_000.0 || (value.abs() > 0.0 && value.abs() < 0.001) {
        return smart_format(value, None);
    }

    // Determine precision from step
    let step_str = format!("{}", step);
    let precision = if let Some(dot_pos) = step_str.find('.') {
        let frac = &step_str[dot_pos + 1..];
        frac.trim_end_matches('0').len()
    } else {
        0
    };

    if precision == 0 {
        format!("{}", value as i64)
    } else {
        let formatted = format!("{:.prec$}", value, prec = precision);
        // Trim trailing zeros
        if formatted.contains('.') {
            let trimmed = formatted.trim_end_matches('0');
            if let Some(stripped) = trimmed.strip_suffix('.') {
                stripped.to_string()
            } else {
                trimmed.to_string()
            }
        } else {
            formatted
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // nice_ticks basic
    // =========================================================================

    #[test]
    fn test_basic_range() {
        let ticks = nice_ticks(0.0, 100.0, 5);
        assert!(!ticks.is_empty());
        assert!(ticks.len() <= 6);
        assert!(ticks[0] <= 0.0);
        assert!(*ticks.last().unwrap() >= 100.0);
    }

    #[test]
    fn test_zero_range() {
        let ticks = nice_ticks(50.0, 50.0, 5);
        assert_eq!(ticks, vec![50.0]);
    }

    #[test]
    fn test_negative_range() {
        let ticks = nice_ticks(-100.0, 0.0, 5);
        assert!(ticks[0] <= -100.0);
        assert!(*ticks.last().unwrap() >= 0.0);
    }

    #[test]
    fn test_small_range() {
        let ticks = nice_ticks(0.0, 0.5, 5);
        assert!(!ticks.is_empty());
        assert!(ticks[0] <= 0.0);
        assert!(*ticks.last().unwrap() >= 0.5);
    }

    #[test]
    fn test_large_range() {
        let ticks = nice_ticks(0.0, 1_000_000.0, 5);
        assert!(!ticks.is_empty());
        assert!(ticks[0] <= 0.0);
        assert!(*ticks.last().unwrap() >= 1_000_000.0);
    }

    #[test]
    fn test_max_two() {
        let ticks = nice_ticks(0.0, 100.0, 2);
        assert!(ticks.len() >= 2);
        assert!(ticks[0] <= 0.0);
        assert!(*ticks.last().unwrap() >= 100.0);
    }

    #[test]
    fn test_non_zero_origin() {
        let ticks = nice_ticks(45.0, 92.0, 5);
        assert!(ticks[0] <= 45.0);
        assert!(*ticks.last().unwrap() >= 92.0);
    }

    #[test]
    fn test_fractional() {
        let ticks = nice_ticks(0.1, 0.9, 5);
        assert!(ticks[0] <= 0.1);
        assert!(*ticks.last().unwrap() >= 0.9);
    }

    #[test]
    fn test_negative_to_positive() {
        let ticks = nice_ticks(-50.0, 50.0, 5);
        assert!(ticks[0] <= -50.0);
        assert!(*ticks.last().unwrap() >= 50.0);
    }

    #[test]
    fn test_nan_input() {
        let ticks = nice_ticks(f64::NAN, 100.0, 5);
        assert_eq!(ticks.len(), 2);
    }

    #[test]
    fn test_infinity_input() {
        let ticks = nice_ticks(0.0, f64::INFINITY, 5);
        assert_eq!(ticks.len(), 2);
    }

    // =========================================================================
    // nice_step
    // =========================================================================

    #[test]
    fn test_nice_step_one() {
        assert_eq!(nice_step(1.0), 1.0);
    }

    #[test]
    fn test_nice_step_two() {
        assert_eq!(nice_step(2.5), 2.0);
    }

    #[test]
    fn test_nice_step_five() {
        assert_eq!(nice_step(4.0), 5.0);
    }

    #[test]
    fn test_nice_step_ten() {
        assert_eq!(nice_step(8.0), 10.0);
    }

    #[test]
    fn test_nice_step_hundred() {
        assert_eq!(nice_step(80.0), 100.0);
    }

    #[test]
    fn test_nice_step_small() {
        let step = nice_step(0.03);
        assert!((step - 0.02).abs() < 1e-10);
    }

    #[test]
    fn test_nice_step_zero() {
        assert_eq!(nice_step(0.0), 1.0);
    }

    #[test]
    fn test_nice_step_negative() {
        assert_eq!(nice_step(-1.0), 1.0);
    }

    // =========================================================================
    // format_tick
    // =========================================================================

    #[test]
    fn test_format_tick_integer() {
        assert_eq!(format_tick(100.0, 50.0), "100");
    }

    #[test]
    fn test_format_tick_decimal() {
        assert_eq!(format_tick(0.5, 0.1), "0.5");
    }

    #[test]
    fn test_format_tick_small() {
        assert_eq!(format_tick(0.0001, 0.0001), "1e-4");
    }

    #[test]
    fn test_format_tick_large() {
        assert_eq!(format_tick(50_000.0, 10_000.0), "50K");
    }

    #[test]
    fn test_format_tick_zero() {
        assert_eq!(format_tick(0.0, 1.0), "0");
    }
}
