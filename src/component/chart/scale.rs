//! Y-axis scale transformations for chart components.
//!
//! Provides linear, logarithmic (base 10), and symmetric log scales.
//! Log and symlog scales transform data internally before rendering,
//! then format axis labels in the original value space.

/// The Y-axis scale for a chart.
///
/// Controls how data values are mapped to vertical position on the chart.
///
/// # Example
///
/// ```rust
/// use envision::component::Scale;
///
/// let scale = Scale::Log10;
/// assert_ne!(scale, Scale::Linear);
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum Scale {
    /// Linear scale (default). Values are mapped directly.
    #[default]
    Linear,
    /// Logarithmic scale (base 10). Values must be positive.
    /// Zero and negative values are clamped to a small positive value.
    Log10,
    /// Symmetric log scale: `sign(x) * log10(1 + |x|)`.
    /// Handles zero and negative values gracefully.
    SymLog,
}

impl Scale {
    /// Transforms a value from data space to chart space.
    pub fn transform(&self, value: f64) -> f64 {
        match self {
            Scale::Linear => value,
            Scale::Log10 => {
                if value <= 0.0 {
                    // Clamp to a small positive value to avoid -inf
                    f64::MIN_POSITIVE.log10()
                } else {
                    value.log10()
                }
            }
            Scale::SymLog => {
                // sign(x) * log10(1 + |x|)
                value.signum() * (1.0 + value.abs()).log10()
            }
        }
    }

    /// Transforms a value from chart space back to data space.
    pub fn inverse(&self, value: f64) -> f64 {
        match self {
            Scale::Linear => value,
            Scale::Log10 => 10.0_f64.powf(value),
            Scale::SymLog => {
                // Inverse of sign(x) * log10(1 + |x|)
                value.signum() * (10.0_f64.powf(value.abs()) - 1.0)
            }
        }
    }

    /// Returns true if this is a non-linear scale.
    pub fn is_logarithmic(&self) -> bool {
        matches!(self, Scale::Log10 | Scale::SymLog)
    }
}

/// Generates tick values for a logarithmic scale.
///
/// Produces ticks at powers of 10 within the given range (in data space).
/// For example, range [0.01, 1000] produces ticks at 0.01, 0.1, 1, 10, 100, 1000.
pub fn log_ticks(min: f64, max: f64, max_ticks: usize) -> Vec<f64> {
    if max_ticks < 2 || min <= 0.0 || max <= 0.0 || min >= max {
        return vec![min, max];
    }

    let log_min = min.log10().floor() as i32;
    let log_max = max.log10().ceil() as i32;

    let mut ticks: Vec<f64> = (log_min..=log_max).map(|exp| 10.0_f64.powi(exp)).collect();

    // If we have too many ticks, skip some
    while ticks.len() > max_ticks {
        let step = (ticks.len() / max_ticks).max(2);
        ticks = ticks.into_iter().step_by(step).collect();
    }

    // Ensure we have at least the endpoints
    if ticks.is_empty() {
        ticks = vec![min, max];
    }

    ticks
}

/// Formats a tick value for a logarithmic scale.
///
/// Uses compact notation: "0.01", "0.1", "1", "10", "100", "1K", "10K", etc.
pub fn format_log_tick(value: f64) -> String {
    super::format::smart_format(value, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    // =============================================================================
    // Scale::Linear
    // =============================================================================

    #[test]
    fn test_linear_transform() {
        assert_eq!(Scale::Linear.transform(42.0), 42.0);
        assert_eq!(Scale::Linear.transform(-5.0), -5.0);
        assert_eq!(Scale::Linear.transform(0.0), 0.0);
    }

    #[test]
    fn test_linear_inverse() {
        assert_eq!(Scale::Linear.inverse(42.0), 42.0);
    }

    // =============================================================================
    // Scale::Log10
    // =============================================================================

    #[test]
    fn test_log10_transform() {
        assert!((Scale::Log10.transform(1.0) - 0.0).abs() < 1e-10);
        assert!((Scale::Log10.transform(10.0) - 1.0).abs() < 1e-10);
        assert!((Scale::Log10.transform(100.0) - 2.0).abs() < 1e-10);
        assert!((Scale::Log10.transform(0.1) - (-1.0)).abs() < 1e-10);
    }

    #[test]
    fn test_log10_transform_zero() {
        // Zero should clamp to a very small value (not -inf)
        let result = Scale::Log10.transform(0.0);
        assert!(result.is_finite());
        assert!(result < -300.0); // log10(MIN_POSITIVE) is around -308
    }

    #[test]
    fn test_log10_transform_negative() {
        // Negative values should clamp like zero
        let result = Scale::Log10.transform(-5.0);
        assert!(result.is_finite());
    }

    #[test]
    fn test_log10_inverse() {
        assert!((Scale::Log10.inverse(0.0) - 1.0).abs() < 1e-10);
        assert!((Scale::Log10.inverse(1.0) - 10.0).abs() < 1e-10);
        assert!((Scale::Log10.inverse(2.0) - 100.0).abs() < 1e-10);
    }

    #[test]
    fn test_log10_roundtrip() {
        for value in [0.001, 0.1, 1.0, 10.0, 100.0, 10000.0] {
            let transformed = Scale::Log10.transform(value);
            let recovered = Scale::Log10.inverse(transformed);
            assert!(
                (recovered - value).abs() / value < 1e-10,
                "roundtrip failed for {value}: got {recovered}"
            );
        }
    }

    // =============================================================================
    // Scale::SymLog
    // =============================================================================

    #[test]
    fn test_symlog_transform_positive() {
        // symlog(10) = log10(11) ≈ 1.0414
        let result = Scale::SymLog.transform(10.0);
        assert!((result - (11.0_f64).log10()).abs() < 1e-10);
    }

    #[test]
    fn test_symlog_transform_zero() {
        // symlog(0) = 0 * log10(1) = 0
        assert_eq!(Scale::SymLog.transform(0.0), 0.0);
    }

    #[test]
    fn test_symlog_transform_negative() {
        // symlog(-10) = -log10(11) ≈ -1.0414
        let result = Scale::SymLog.transform(-10.0);
        assert!((result + (11.0_f64).log10()).abs() < 1e-10);
    }

    #[test]
    fn test_symlog_symmetry() {
        // symlog(-x) = -symlog(x)
        for x in [1.0, 10.0, 100.0, 0.5] {
            let pos = Scale::SymLog.transform(x);
            let neg = Scale::SymLog.transform(-x);
            assert!(
                (pos + neg).abs() < 1e-10,
                "symmetry failed for {x}: {pos} vs {neg}"
            );
        }
    }

    #[test]
    fn test_symlog_roundtrip() {
        for value in [-100.0, -1.0, 0.0, 1.0, 100.0, 10000.0] {
            let transformed = Scale::SymLog.transform(value);
            let recovered = Scale::SymLog.inverse(transformed);
            if value == 0.0 {
                assert!(recovered.abs() < 1e-10);
            } else {
                assert!(
                    (recovered - value).abs() / value.abs() < 1e-10,
                    "roundtrip failed for {value}: got {recovered}"
                );
            }
        }
    }

    // =============================================================================
    // is_logarithmic
    // =============================================================================

    #[test]
    fn test_is_logarithmic() {
        assert!(!Scale::Linear.is_logarithmic());
        assert!(Scale::Log10.is_logarithmic());
        assert!(Scale::SymLog.is_logarithmic());
    }

    // =============================================================================
    // log_ticks
    // =============================================================================

    #[test]
    fn test_log_ticks_basic() {
        let ticks = log_ticks(1.0, 10000.0, 10);
        assert_eq!(ticks, vec![1.0, 10.0, 100.0, 1000.0, 10000.0]);
    }

    #[test]
    fn test_log_ticks_fractional_range() {
        let ticks = log_ticks(0.01, 100.0, 10);
        assert_eq!(ticks, vec![0.01, 0.1, 1.0, 10.0, 100.0]);
    }

    #[test]
    fn test_log_ticks_max_ticks_limits() {
        let ticks = log_ticks(0.001, 1000000.0, 5);
        // 10 powers of 10, but limited to 5
        assert!(ticks.len() <= 5);
    }

    #[test]
    fn test_log_ticks_invalid_range() {
        let ticks = log_ticks(-1.0, 100.0, 5);
        assert_eq!(ticks, vec![-1.0, 100.0]);
    }

    #[test]
    fn test_log_ticks_zero_min() {
        let ticks = log_ticks(0.0, 100.0, 5);
        assert_eq!(ticks, vec![0.0, 100.0]);
    }

    // =============================================================================
    // format_log_tick
    // =============================================================================

    #[test]
    fn test_format_log_tick() {
        assert_eq!(format_log_tick(1.0), "1");
        assert_eq!(format_log_tick(10.0), "10");
        assert_eq!(format_log_tick(100.0), "100");
        assert_eq!(format_log_tick(1000.0), "1000");
        assert_eq!(format_log_tick(10000.0), "10K");
        assert_eq!(format_log_tick(0.1), "0.10");
        assert_eq!(format_log_tick(0.01), "0.01");
    }

    // =============================================================================
    // Default
    // =============================================================================

    #[test]
    fn test_default_is_linear() {
        assert_eq!(Scale::default(), Scale::Linear);
    }
}
