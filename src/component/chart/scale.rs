//! Logarithmic and linear scale transforms for chart axes.
//!
//! Provides [`Scale`] enum with `Linear`, `Log10`, and `SymLog` variants
//! for transforming Y-axis data values during rendering.

use super::format::smart_format;

/// Scale transformation for a chart axis.
///
/// # Example
///
/// ```rust
/// use envision::component::Scale;
///
/// let scale = Scale::Log10;
/// assert!(scale.is_logarithmic());
/// let transformed = scale.transform(100.0);
/// assert!((transformed - 2.0).abs() < 1e-10);
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum Scale {
    /// Linear scale (identity transform).
    #[default]
    Linear,
    /// Base-10 logarithmic scale.
    ///
    /// Values <= 0 are clamped to a small positive value.
    Log10,
    /// Symmetric logarithmic scale.
    ///
    /// Handles negative values by applying `sign(x) * log10(|x| + 1)`.
    SymLog,
}

impl Scale {
    /// Transforms a value according to this scale.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::Scale;
    ///
    /// assert_eq!(Scale::Linear.transform(42.0), 42.0);
    /// assert!((Scale::Log10.transform(1000.0) - 3.0).abs() < 1e-10);
    /// ```
    pub fn transform(&self, value: f64) -> f64 {
        match self {
            Scale::Linear => value,
            Scale::Log10 => {
                if value <= 0.0 {
                    // Clamp to a small positive value
                    f64::MIN_POSITIVE.log10()
                } else {
                    value.log10()
                }
            }
            Scale::SymLog => {
                let sign = value.signum();
                sign * (value.abs() + 1.0).log10()
            }
        }
    }

    /// Computes the inverse transform (from scaled space back to data space).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::Scale;
    ///
    /// assert_eq!(Scale::Linear.inverse(42.0), 42.0);
    /// assert!((Scale::Log10.inverse(2.0) - 100.0).abs() < 1e-10);
    /// ```
    pub fn inverse(&self, value: f64) -> f64 {
        match self {
            Scale::Linear => value,
            Scale::Log10 => 10f64.powf(value),
            Scale::SymLog => {
                let sign = value.signum();
                sign * (10f64.powf(value.abs()) - 1.0)
            }
        }
    }

    /// Returns true if this scale is logarithmic (Log10 or SymLog).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::Scale;
    ///
    /// assert!(!Scale::Linear.is_logarithmic());
    /// assert!(Scale::Log10.is_logarithmic());
    /// assert!(Scale::SymLog.is_logarithmic());
    /// ```
    pub fn is_logarithmic(&self) -> bool {
        matches!(self, Scale::Log10 | Scale::SymLog)
    }
}

/// Generates tick positions for a logarithmic axis.
///
/// Produces ticks at powers of 10 within the given range,
/// limited to `max_ticks`.
///
/// # Example
///
/// ```ignore
/// use envision::component::chart::scale::log_ticks;
///
/// let ticks = log_ticks(1.0, 10000.0, 5);
/// assert!(ticks.contains(&1.0));
/// assert!(ticks.contains(&10.0));
/// assert!(ticks.contains(&100.0));
/// assert!(ticks.contains(&1000.0));
/// assert!(ticks.contains(&10000.0));
/// ```
pub fn log_ticks(min: f64, max: f64, max_ticks: usize) -> Vec<f64> {
    if max_ticks < 2 || min <= 0.0 || max <= 0.0 || min.is_nan() || max.is_nan() {
        return vec![min.max(f64::MIN_POSITIVE), max.max(f64::MIN_POSITIVE)];
    }

    let min_exp = min.log10().floor() as i32;
    let max_exp = max.log10().ceil() as i32;

    let mut ticks: Vec<f64> = (min_exp..=max_exp).map(|e| 10f64.powi(e)).collect();

    // If too many ticks, skip some exponents
    while ticks.len() > max_ticks {
        let step = (ticks.len() / max_ticks).max(2);
        ticks = ticks.into_iter().step_by(step).collect();
    }

    ticks
}

/// Formats a tick value on a logarithmic axis.
///
/// # Example
///
/// ```ignore
/// use envision::component::chart::scale::format_log_tick;
///
/// assert_eq!(format_log_tick(1000.0), "1K");
/// assert_eq!(format_log_tick(1.0), "1");
/// ```
pub fn format_log_tick(value: f64) -> String {
    smart_format(value, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Scale::Linear
    // =========================================================================

    #[test]
    fn test_linear_transform() {
        assert_eq!(Scale::Linear.transform(42.0), 42.0);
    }

    #[test]
    fn test_linear_inverse() {
        assert_eq!(Scale::Linear.inverse(42.0), 42.0);
    }

    #[test]
    fn test_linear_roundtrip() {
        let value = 123.456;
        let result = Scale::Linear.inverse(Scale::Linear.transform(value));
        assert!((result - value).abs() < 1e-10);
    }

    #[test]
    fn test_linear_not_logarithmic() {
        assert!(!Scale::Linear.is_logarithmic());
    }

    // =========================================================================
    // Scale::Log10
    // =========================================================================

    #[test]
    fn test_log10_transform() {
        assert!((Scale::Log10.transform(100.0) - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_log10_transform_one() {
        assert!((Scale::Log10.transform(1.0)).abs() < 1e-10);
    }

    #[test]
    fn test_log10_transform_ten() {
        assert!((Scale::Log10.transform(10.0) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_log10_transform_zero_clamped() {
        // Zero should be clamped to a small positive value
        let result = Scale::Log10.transform(0.0);
        assert!(result < 0.0); // log10 of tiny value is very negative
    }

    #[test]
    fn test_log10_transform_negative_clamped() {
        let result = Scale::Log10.transform(-5.0);
        assert!(result < 0.0);
    }

    #[test]
    fn test_log10_inverse() {
        assert!((Scale::Log10.inverse(2.0) - 100.0).abs() < 1e-10);
    }

    #[test]
    fn test_log10_roundtrip() {
        let value = 500.0;
        let result = Scale::Log10.inverse(Scale::Log10.transform(value));
        assert!((result - value).abs() < 1e-6);
    }

    #[test]
    fn test_log10_is_logarithmic() {
        assert!(Scale::Log10.is_logarithmic());
    }

    // =========================================================================
    // Scale::SymLog
    // =========================================================================

    #[test]
    fn test_symlog_transform_positive() {
        let result = Scale::SymLog.transform(9.0);
        assert!((result - 1.0).abs() < 1e-10); // log10(9 + 1) = log10(10) = 1
    }

    #[test]
    fn test_symlog_transform_zero() {
        assert!((Scale::SymLog.transform(0.0)).abs() < 1e-10); // sign(0)*log10(1) = 0
    }

    #[test]
    fn test_symlog_transform_negative() {
        let result = Scale::SymLog.transform(-9.0);
        assert!((result + 1.0).abs() < 1e-10); // -1 * log10(10) = -1
    }

    #[test]
    fn test_symlog_symmetry() {
        let pos = Scale::SymLog.transform(100.0);
        let neg = Scale::SymLog.transform(-100.0);
        assert!((pos + neg).abs() < 1e-10);
    }

    #[test]
    fn test_symlog_inverse_positive() {
        let value = 50.0;
        let result = Scale::SymLog.inverse(Scale::SymLog.transform(value));
        assert!((result - value).abs() < 1e-6);
    }

    #[test]
    fn test_symlog_inverse_negative() {
        let value = -50.0;
        let result = Scale::SymLog.inverse(Scale::SymLog.transform(value));
        assert!((result - value).abs() < 1e-6);
    }

    #[test]
    fn test_symlog_is_logarithmic() {
        assert!(Scale::SymLog.is_logarithmic());
    }

    // =========================================================================
    // log_ticks
    // =========================================================================

    #[test]
    fn test_log_ticks_basic() {
        let ticks = log_ticks(1.0, 10000.0, 5);
        assert!(ticks.contains(&1.0));
        assert!(ticks.contains(&10.0));
        assert!(ticks.contains(&100.0));
        assert!(ticks.contains(&1000.0));
        assert!(ticks.contains(&10000.0));
    }

    #[test]
    fn test_log_ticks_limited() {
        let ticks = log_ticks(1.0, 1_000_000_000.0, 3);
        assert!(ticks.len() <= 3);
    }

    #[test]
    fn test_log_ticks_small_range() {
        let ticks = log_ticks(1.0, 10.0, 5);
        assert!(!ticks.is_empty());
    }

    #[test]
    fn test_log_ticks_zero_min() {
        let ticks = log_ticks(0.0, 100.0, 5);
        assert!(!ticks.is_empty());
    }

    // =========================================================================
    // format_log_tick
    // =========================================================================

    #[test]
    fn test_format_log_tick_one() {
        assert_eq!(format_log_tick(1.0), "1");
    }

    #[test]
    fn test_format_log_tick_thousand() {
        assert_eq!(format_log_tick(1000.0), "1K");
    }

    #[test]
    fn test_format_log_tick_large() {
        assert_eq!(format_log_tick(1_000_000.0), "1M");
    }

    // =========================================================================
    // Default
    // =========================================================================

    #[test]
    fn test_default_is_linear() {
        assert_eq!(Scale::default(), Scale::Linear);
    }

    // =========================================================================
    // Clone and Debug
    // =========================================================================

    #[test]
    fn test_clone() {
        let scale = Scale::Log10;
        let cloned = scale.clone();
        assert_eq!(scale, cloned);
    }

    #[test]
    fn test_debug() {
        let debug = format!("{:?}", Scale::SymLog);
        assert_eq!(debug, "SymLog");
    }
}
