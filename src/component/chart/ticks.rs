//! Axis tick generation for chart components.
//!
//! Implements the "Nice Numbers for Graph Labels" algorithm (Heckbert, 1990)
//! to compute human-friendly axis tick values that adapt to available space.

/// Computes human-friendly tick values for an axis range.
///
/// Returns a vector of tick values between `min` and `max` (inclusive of
/// the nice-rounded bounds), with at most `max_ticks` values. The ticks
/// use "nice" step sizes (multiples of 1, 2, or 5 × 10^n).
pub fn nice_ticks(min: f64, max: f64, max_ticks: usize) -> Vec<f64> {
    if max_ticks < 2 {
        return vec![min, max];
    }

    let range = max - min;
    if range <= 0.0 || !range.is_finite() {
        return vec![min];
    }

    let rough_step = range / (max_ticks - 1) as f64;
    let step = nice_step(rough_step);

    let nice_min = (min / step).floor() * step;
    let nice_max = (max / step).ceil() * step;

    let mut ticks = Vec::new();
    let mut value = nice_min;

    // Guard against infinite loops from floating point edge cases
    let max_iterations = max_ticks * 2 + 2;
    let mut iterations = 0;

    while value <= nice_max + step * 0.001 && iterations < max_iterations {
        ticks.push(round_to_precision(value, step));
        value += step;
        iterations += 1;
    }

    ticks
}

/// Rounds a rough step size to a "nice" value.
///
/// Nice values are 1, 2, 5, or 10 × 10^n. This produces axis labels
/// that humans find easy to read.
fn nice_step(rough: f64) -> f64 {
    let exponent = rough.log10().floor();
    let fraction = rough / 10.0_f64.powf(exponent);

    let nice_fraction = if fraction <= 1.5 {
        1.0
    } else if fraction <= 3.5 {
        2.0
    } else if fraction <= 7.5 {
        5.0
    } else {
        10.0
    };

    nice_fraction * 10.0_f64.powf(exponent)
}

/// Rounds a value to the same decimal precision as the step size.
///
/// Eliminates floating-point artifacts like 0.30000000000000004.
fn round_to_precision(value: f64, step: f64) -> f64 {
    if step == 0.0 {
        return value;
    }
    let decimals = (-step.log10()).ceil().max(0.0) as i32;
    let factor = 10.0_f64.powi(decimals);
    (value * factor).round() / factor
}

/// Formats a tick value for display on an axis.
///
/// Uses smart formatting for large/small values (SI suffixes, scientific
/// notation) and step-aware precision for moderate values.
pub fn format_tick(value: f64, step: f64) -> String {
    let abs = value.abs();

    // For large values, use smart_format which provides SI suffixes
    if abs >= 10_000.0 || (abs > 0.0 && abs < 0.001) {
        return super::format::smart_format(value, None);
    }

    // For moderate values, use step-aware precision
    if step >= 1.0 && (value - value.round()).abs() < 1e-9 {
        format!("{}", value.round() as i64)
    } else {
        let decimals = (-step.log10()).ceil().max(0.0) as usize;
        format!("{:.prec$}", value, prec = decimals)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nice_ticks_basic() {
        let ticks = nice_ticks(0.0, 100.0, 6);
        assert_eq!(ticks, vec![0.0, 20.0, 40.0, 60.0, 80.0, 100.0]);
    }

    #[test]
    fn test_nice_ticks_five() {
        // With max_ticks=5, rough_step=25, nice_step rounds to 20
        let ticks = nice_ticks(0.0, 100.0, 5);
        assert_eq!(ticks, vec![0.0, 20.0, 40.0, 60.0, 80.0, 100.0]);
    }

    #[test]
    fn test_nice_ticks_zero_range() {
        let ticks = nice_ticks(5.0, 5.0, 5);
        assert_eq!(ticks, vec![5.0]);
    }

    #[test]
    fn test_nice_ticks_negative_range() {
        let ticks = nice_ticks(-100.0, 0.0, 6);
        assert_eq!(ticks, vec![-100.0, -80.0, -60.0, -40.0, -20.0, 0.0]);
    }

    #[test]
    fn test_nice_ticks_small_range() {
        let ticks = nice_ticks(0.0, 1.0, 6);
        assert_eq!(ticks, vec![0.0, 0.2, 0.4, 0.6, 0.8, 1.0]);
    }

    #[test]
    fn test_nice_ticks_large_range() {
        let ticks = nice_ticks(0.0, 10000.0, 6);
        assert_eq!(ticks, vec![0.0, 2000.0, 4000.0, 6000.0, 8000.0, 10000.0]);
    }

    #[test]
    fn test_nice_ticks_max_two() {
        let ticks = nice_ticks(0.0, 100.0, 2);
        // With max_ticks=2, step should be 100, giving [0, 100]
        assert_eq!(ticks, vec![0.0, 100.0]);
    }

    #[test]
    fn test_nice_ticks_one_returns_endpoints() {
        let ticks = nice_ticks(10.0, 50.0, 1);
        assert_eq!(ticks, vec![10.0, 50.0]);
    }

    #[test]
    fn test_nice_ticks_non_zero_origin() {
        let ticks = nice_ticks(3.0, 97.0, 6);
        // Should round to nice bounds
        assert!(ticks.first().unwrap() <= &3.0);
        assert!(ticks.last().unwrap() >= &97.0);
        // All ticks should be evenly spaced
        let step = ticks[1] - ticks[0];
        for i in 1..ticks.len() {
            let diff = ticks[i] - ticks[i - 1];
            assert!((diff - step).abs() < 1e-10);
        }
    }

    #[test]
    fn test_nice_ticks_fractional_data() {
        let ticks = nice_ticks(0.001, 0.009, 5);
        assert!(!ticks.is_empty());
        assert!(ticks.first().unwrap() <= &0.001);
        assert!(ticks.last().unwrap() >= &0.009);
    }

    #[test]
    fn test_nice_ticks_negative_to_positive() {
        let ticks = nice_ticks(-50.0, 50.0, 6);
        assert!(ticks.contains(&0.0));
    }

    #[test]
    fn test_nice_step_values() {
        // Step of 3 → nice step of 2
        assert_eq!(nice_step(3.0), 2.0);
        // Step of 7 → nice step of 5
        assert_eq!(nice_step(7.0), 5.0);
        // Step of 0.3 → nice step of 0.2
        assert!((nice_step(0.3) - 0.2).abs() < 1e-10);
        // Step of 15 → nice step of 10 (fraction=1.5 ≤ 1.5 → 1.0 × 10^1)
        assert_eq!(nice_step(15.0), 10.0);
    }

    #[test]
    fn test_format_tick_integer() {
        assert_eq!(format_tick(100.0, 20.0), "100");
        assert_eq!(format_tick(0.0, 50.0), "0");
        assert_eq!(format_tick(-40.0, 20.0), "-40");
    }

    #[test]
    fn test_format_tick_decimal() {
        assert_eq!(format_tick(0.2, 0.2), "0.2");
        assert_eq!(format_tick(0.4, 0.2), "0.4");
    }

    #[test]
    fn test_format_tick_small_values() {
        // Values >= 0.001 use step-aware decimal precision
        assert_eq!(format_tick(0.002, 0.002), "0.002");
        // Values < 0.001 use scientific notation via smart_format
        assert_eq!(format_tick(0.0002, 0.0001), "2.00e-4");
    }

    #[test]
    fn test_format_tick_large_values() {
        // Large values use SI suffixes via smart_format
        assert_eq!(format_tick(10000.0, 5000.0), "10K");
        assert_eq!(format_tick(25000.0, 5000.0), "25K");
        assert_eq!(format_tick(1000000.0, 500000.0), "1M");
    }

    #[test]
    fn test_round_to_precision_eliminates_artifacts() {
        // 0.1 + 0.2 = 0.30000000000000004 in floating point
        let value = 0.1 + 0.2;
        let rounded = round_to_precision(value, 0.1);
        assert_eq!(rounded, 0.3);
    }

    #[test]
    fn test_nice_ticks_nan_range() {
        let ticks = nice_ticks(0.0, f64::NAN, 5);
        assert_eq!(ticks, vec![0.0]);
    }

    #[test]
    fn test_nice_ticks_infinite_range() {
        let ticks = nice_ticks(0.0, f64::INFINITY, 5);
        assert_eq!(ticks, vec![0.0]);
    }
}
