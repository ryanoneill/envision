//! Converts distribution snapshots into heatmap-compatible data.
//!
//! [`DistributionMap`] takes time-series distribution snapshots and bins them
//! into a 2D grid (bins x steps) suitable for [`HeatmapState`]. This is useful
//! for visualizing how a distribution evolves over time, such as gradient
//! histograms or weight distributions across training steps.
//!
//! # Example
//!
//! ```rust
//! use envision::component::{DistributionMap, HeatmapColorScale};
//!
//! let state = DistributionMap::new()
//!     .add_snapshot("Step 0", &[-1.0, -0.5, 0.0, 0.5, 1.0])
//!     .add_snapshot("Step 1", &[-0.8, -0.3, 0.1, 0.4, 0.9])
//!     .add_snapshot("Step 2", &[-0.5, -0.1, 0.2, 0.3, 0.6])
//!     .with_bins(10)
//!     .to_heatmap();
//!
//! assert_eq!(state.cols(), 3);   // 3 snapshots
//! assert_eq!(state.rows(), 10);  // 10 bins
//! assert_eq!(state.col_labels().len(), 3);
//! assert_eq!(state.row_labels().len(), 10);
//! assert_eq!(state.color_scale(), &HeatmapColorScale::Inferno);
//! ```

use super::{HeatmapColorScale, HeatmapState};

/// Default number of bins when none is specified.
const DEFAULT_BIN_COUNT: usize = 50;

/// Converts distribution snapshots into heatmap-compatible data.
///
/// Each snapshot is a collection of values at a specific time step.
/// The helper bins the values into a histogram and assembles the
/// results into a 2D grid (bins x steps) suitable for [`HeatmapState`].
///
/// The resulting heatmap has:
/// - **Rows** = bin indices (Y axis, from lowest to highest value range)
/// - **Columns** = snapshots (X axis, in insertion order)
/// - **Cell values** = count of values falling in each bin for each snapshot
///
/// # Example
///
/// ```rust
/// use envision::component::DistributionMap;
///
/// // Track gradient magnitudes across training steps
/// let state = DistributionMap::new()
///     .add_snapshot("Epoch 1", &[0.01, 0.02, 0.015, 0.03, 0.025])
///     .add_snapshot("Epoch 5", &[0.005, 0.01, 0.008, 0.012, 0.009])
///     .add_snapshot("Epoch 10", &[0.001, 0.002, 0.0015, 0.003, 0.002])
///     .with_bins(20)
///     .to_heatmap();
///
/// assert_eq!(state.rows(), 20);
/// assert_eq!(state.cols(), 3);
/// ```
pub struct DistributionMap {
    snapshots: Vec<(String, Vec<f64>)>,
    bin_count: usize,
    range: Option<(f64, f64)>,
}

impl DistributionMap {
    /// Creates a new empty `DistributionMap` with default settings.
    ///
    /// The default bin count is 50 and the value range is automatically
    /// determined from the data.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DistributionMap;
    ///
    /// let dm = DistributionMap::new();
    /// let state = dm.to_heatmap();
    /// assert_eq!(state.rows(), 0);
    /// assert_eq!(state.cols(), 0);
    /// ```
    pub fn new() -> Self {
        Self {
            snapshots: Vec::new(),
            bin_count: DEFAULT_BIN_COUNT,
            range: None,
        }
    }

    /// Adds a snapshot of values at a labeled time step.
    ///
    /// Snapshots appear as columns in the resulting heatmap, in the
    /// order they are added. Empty value slices are accepted and will
    /// produce a column of zeros.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DistributionMap;
    ///
    /// let state = DistributionMap::new()
    ///     .add_snapshot("T=0", &[1.0, 2.0, 3.0])
    ///     .add_snapshot("T=1", &[2.0, 3.0, 4.0])
    ///     .with_bins(5)
    ///     .to_heatmap();
    ///
    /// assert_eq!(state.cols(), 2);
    /// assert_eq!(state.rows(), 5);
    /// ```
    pub fn add_snapshot(mut self, label: impl Into<String>, values: &[f64]) -> Self {
        self.snapshots.push((label.into(), values.to_vec()));
        self
    }

    /// Sets the number of bins for histogramming.
    ///
    /// The default is 50 bins. The bin count must be at least 1; a value
    /// of 0 is clamped to 1.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DistributionMap;
    ///
    /// let state = DistributionMap::new()
    ///     .add_snapshot("Step 0", &[0.0, 1.0, 2.0, 3.0])
    ///     .with_bins(4)
    ///     .to_heatmap();
    ///
    /// assert_eq!(state.rows(), 4);
    /// ```
    pub fn with_bins(mut self, bins: usize) -> Self {
        self.bin_count = bins.max(1);
        self
    }

    /// Sets a fixed value range across all snapshots.
    ///
    /// By default, the range is computed from the global minimum and
    /// maximum across all snapshot values. Setting a fixed range is
    /// useful when comparing distribution maps or when the expected
    /// data range is known in advance.
    ///
    /// If `min >= max`, the range is treated as a single-point range
    /// and all values will fall into the first bin.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DistributionMap;
    ///
    /// let state = DistributionMap::new()
    ///     .add_snapshot("A", &[0.5, 1.5])
    ///     .with_bins(10)
    ///     .with_range(0.0, 10.0)
    ///     .to_heatmap();
    ///
    /// assert_eq!(state.rows(), 10);
    /// // Row labels reflect the fixed 0.0..10.0 range
    /// ```
    pub fn with_range(mut self, min: f64, max: f64) -> Self {
        self.range = Some((min, max));
        self
    }

    /// Converts the accumulated snapshots to a [`HeatmapState`] ready for rendering.
    ///
    /// The resulting heatmap uses the [`Inferno`](HeatmapColorScale::Inferno)
    /// color scale by default, which provides good contrast for density
    /// visualizations.
    ///
    /// # Layout
    ///
    /// - Rows correspond to bin indices (low values at the top, high at the bottom
    ///   is reversed so that the Y axis reads bottom-to-top in the visual).
    /// - Columns correspond to snapshots in insertion order.
    /// - Cell values are the count of values falling in each bin.
    ///
    /// Returns an empty `HeatmapState` if no snapshots have been added.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{DistributionMap, HeatmapColorScale};
    ///
    /// let state = DistributionMap::new()
    ///     .add_snapshot("Init", &[-1.0, 0.0, 1.0])
    ///     .add_snapshot("Trained", &[-0.1, 0.0, 0.1])
    ///     .with_bins(5)
    ///     .to_heatmap();
    ///
    /// assert_eq!(state.rows(), 5);
    /// assert_eq!(state.cols(), 2);
    /// assert_eq!(state.color_scale(), &HeatmapColorScale::Inferno);
    /// ```
    pub fn to_heatmap(&self) -> HeatmapState {
        if self.snapshots.is_empty() {
            return HeatmapState::default();
        }

        let (global_min, global_max) = self.compute_range();
        let bin_count = self.bin_count;

        // Build the 2D grid: rows = bin indices, columns = snapshots.
        // We reverse the row order so that higher values appear at the top
        // of the heatmap (visually bottom-to-top Y axis).
        let mut grid = vec![vec![0.0; self.snapshots.len()]; bin_count];

        for (col_idx, (_label, values)) in self.snapshots.iter().enumerate() {
            let histogram = bin_values(values, bin_count, global_min, global_max);
            for (bin_idx, &count) in histogram.iter().enumerate() {
                // Reverse: bin 0 (lowest range) goes to the last row,
                // bin N-1 (highest range) goes to the first row.
                let row_idx = bin_count - 1 - bin_idx;
                grid[row_idx][col_idx] = count as f64;
            }
        }

        // Build row labels (bin range strings, reversed to match grid).
        let row_labels = build_bin_labels(bin_count, global_min, global_max);

        // Build column labels.
        let col_labels: Vec<String> = self
            .snapshots
            .iter()
            .map(|(label, _)| label.clone())
            .collect();

        HeatmapState::with_data(grid)
            .with_row_labels(row_labels)
            .with_col_labels(col_labels)
            .with_color_scale(HeatmapColorScale::Inferno)
            .with_title("Distribution Map")
    }

    /// Computes the global (min, max) range across all snapshot values.
    ///
    /// If a fixed range was set via [`with_range`](Self::with_range), that
    /// is returned. Otherwise, the range is derived from the data. If all
    /// snapshots are empty, returns (0.0, 0.0).
    fn compute_range(&self) -> (f64, f64) {
        if let Some((min, max)) = self.range {
            return (min, max);
        }

        let mut global_min = f64::INFINITY;
        let mut global_max = f64::NEG_INFINITY;

        for (_label, values) in &self.snapshots {
            for &v in values {
                if v < global_min {
                    global_min = v;
                }
                if v > global_max {
                    global_max = v;
                }
            }
        }

        if global_min.is_infinite() {
            // All snapshots are empty.
            (0.0, 0.0)
        } else {
            (global_min, global_max)
        }
    }
}

impl Default for DistributionMap {
    fn default() -> Self {
        Self::new()
    }
}

/// Bins a slice of values into `bin_count` histogram bins.
///
/// Values are distributed into bins spanning `[min, max]`. Values exactly
/// equal to `max` are placed in the last bin. Returns a vector of counts
/// with length `bin_count`.
fn bin_values(values: &[f64], bin_count: usize, min: f64, max: f64) -> Vec<usize> {
    let mut bins = vec![0usize; bin_count];
    let range = max - min;

    if range.abs() < f64::EPSILON {
        // All values are effectively the same; put everything in bin 0.
        for &v in values {
            if (v - min).abs() < f64::EPSILON || (range.abs() < f64::EPSILON) {
                bins[0] += 1;
            }
        }
        return bins;
    }

    for &v in values {
        let normalized = (v - min) / range;
        let bin_idx = (normalized * bin_count as f64).floor() as usize;
        // Clamp: values at exactly max go into the last bin.
        let bin_idx = bin_idx.min(bin_count - 1);
        bins[bin_idx] += 1;
    }

    bins
}

/// Builds row labels for the bin ranges, reversed so that the highest range
/// appears first (matching the reversed grid layout).
fn build_bin_labels(bin_count: usize, min: f64, max: f64) -> Vec<String> {
    let range = max - min;
    let bin_width = if range.abs() < f64::EPSILON {
        0.0
    } else {
        range / bin_count as f64
    };

    let mut labels = Vec::with_capacity(bin_count);
    for i in 0..bin_count {
        let lo = min + i as f64 * bin_width;
        let hi = lo + bin_width;
        labels.push(format_range(lo, hi));
    }

    // Reverse to match the reversed grid rows.
    labels.reverse();
    labels
}

/// Formats a bin range label, choosing an appropriate precision
/// based on the magnitude and range width.
fn format_range(lo: f64, hi: f64) -> String {
    // Use enough decimal places to distinguish bins.
    let width = hi - lo;
    let precision = if width >= 1.0 || width.abs() < f64::EPSILON {
        1
    } else if width >= 0.1 {
        2
    } else if width >= 0.01 {
        3
    } else {
        4
    };

    format!("{lo:.precision$}..{hi:.precision$}")
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Basic usage
    // =========================================================================

    #[test]
    fn test_basic_usage_with_three_snapshots() {
        let state = DistributionMap::new()
            .add_snapshot("Step 0", &[-1.0, -0.5, 0.0, 0.5, 1.0])
            .add_snapshot("Step 1", &[-0.8, -0.3, 0.1, 0.4, 0.9])
            .add_snapshot("Step 2", &[-0.5, -0.1, 0.2, 0.3, 0.6])
            .with_bins(10)
            .to_heatmap();

        assert_eq!(state.rows(), 10);
        assert_eq!(state.cols(), 3);
        assert_eq!(state.col_labels().len(), 3);
        assert_eq!(state.row_labels().len(), 10);
        assert_eq!(state.col_labels()[0], "Step 0");
        assert_eq!(state.col_labels()[1], "Step 1");
        assert_eq!(state.col_labels()[2], "Step 2");
    }

    #[test]
    fn test_correct_heatmap_dimensions() {
        let state = DistributionMap::new()
            .add_snapshot("A", &[1.0, 2.0, 3.0, 4.0, 5.0])
            .add_snapshot("B", &[2.0, 3.0, 4.0, 5.0, 6.0])
            .with_bins(20)
            .to_heatmap();

        assert_eq!(state.rows(), 20);
        assert_eq!(state.cols(), 2);
    }

    #[test]
    fn test_default_color_scale_is_inferno() {
        let state = DistributionMap::new()
            .add_snapshot("X", &[0.0, 1.0])
            .with_bins(5)
            .to_heatmap();

        assert_eq!(state.color_scale(), &HeatmapColorScale::Inferno);
    }

    #[test]
    fn test_default_title_is_distribution_map() {
        let state = DistributionMap::new()
            .add_snapshot("X", &[0.0, 1.0])
            .with_bins(5)
            .to_heatmap();

        assert_eq!(state.title(), Some("Distribution Map"));
    }

    // =========================================================================
    // Custom bin count
    // =========================================================================

    #[test]
    fn test_custom_bin_count() {
        let state = DistributionMap::new()
            .add_snapshot("T=0", &[0.0, 0.25, 0.5, 0.75, 1.0])
            .with_bins(4)
            .to_heatmap();

        assert_eq!(state.rows(), 4);
        assert_eq!(state.cols(), 1);
    }

    #[test]
    fn test_single_bin() {
        let state = DistributionMap::new()
            .add_snapshot("All", &[1.0, 2.0, 3.0])
            .with_bins(1)
            .to_heatmap();

        assert_eq!(state.rows(), 1);
        assert_eq!(state.get(0, 0), Some(3.0));
    }

    #[test]
    fn test_bin_count_zero_clamped_to_one() {
        let state = DistributionMap::new()
            .add_snapshot("X", &[1.0, 2.0])
            .with_bins(0)
            .to_heatmap();

        assert_eq!(state.rows(), 1);
    }

    #[test]
    fn test_default_bin_count_is_fifty() {
        let state = DistributionMap::new()
            .add_snapshot("X", &[0.0, 100.0])
            .to_heatmap();

        assert_eq!(state.rows(), DEFAULT_BIN_COUNT);
    }

    // =========================================================================
    // Fixed range
    // =========================================================================

    #[test]
    fn test_fixed_range() {
        let state = DistributionMap::new()
            .add_snapshot("A", &[2.0, 3.0])
            .with_bins(10)
            .with_range(0.0, 10.0)
            .to_heatmap();

        assert_eq!(state.rows(), 10);
        assert_eq!(state.cols(), 1);

        let labels = state.row_labels();
        assert!(labels.last().unwrap().starts_with("0.0"));
        assert!(labels.first().unwrap().contains("10.0"));
    }

    #[test]
    fn test_fixed_range_values_outside_range_clamped() {
        let state = DistributionMap::new()
            .add_snapshot("A", &[-5.0, 15.0])
            .with_bins(5)
            .with_range(0.0, 10.0)
            .to_heatmap();

        assert_eq!(state.rows(), 5);

        let total: f64 = (0..5).map(|r| state.get(r, 0).unwrap_or(0.0)).sum();
        assert_eq!(total, 2.0);
    }

    // =========================================================================
    // Empty snapshots
    // =========================================================================

    #[test]
    fn test_no_snapshots_returns_empty_heatmap() {
        let state = DistributionMap::new().to_heatmap();

        assert_eq!(state.rows(), 0);
        assert_eq!(state.cols(), 0);
    }

    #[test]
    fn test_empty_values_snapshot() {
        let state = DistributionMap::new()
            .add_snapshot("Empty", &[])
            .with_bins(5)
            .to_heatmap();

        assert_eq!(state.rows(), 5);
        assert_eq!(state.cols(), 1);

        for r in 0..5 {
            assert_eq!(state.get(r, 0), Some(0.0));
        }
    }

    #[test]
    fn test_mix_of_empty_and_nonempty_snapshots() {
        let state = DistributionMap::new()
            .add_snapshot("Has Data", &[1.0, 2.0, 3.0])
            .add_snapshot("Empty", &[])
            .with_bins(5)
            .to_heatmap();

        assert_eq!(state.rows(), 5);
        assert_eq!(state.cols(), 2);

        for r in 0..5 {
            assert_eq!(state.get(r, 1), Some(0.0));
        }
    }

    // =========================================================================
    // Bin distribution correctness
    // =========================================================================

    #[test]
    fn test_values_distributed_across_bins() {
        let state = DistributionMap::new()
            .add_snapshot("T", &[0.0, 1.0, 2.0, 3.0, 4.0, 5.0])
            .with_bins(5)
            .with_range(0.0, 5.0)
            .to_heatmap();

        let total: f64 = (0..5).map(|r| state.get(r, 0).unwrap_or(0.0)).sum();
        assert_eq!(total, 6.0);
    }

    #[test]
    fn test_all_same_values() {
        let state = DistributionMap::new()
            .add_snapshot("Same", &[5.0, 5.0, 5.0])
            .with_bins(3)
            .to_heatmap();

        assert_eq!(state.rows(), 3);

        let total: f64 = (0..3).map(|r| state.get(r, 0).unwrap_or(0.0)).sum();
        assert_eq!(total, 3.0);
    }

    #[test]
    fn test_single_value_snapshot() {
        let state = DistributionMap::new()
            .add_snapshot("One", &[42.0])
            .with_bins(10)
            .to_heatmap();

        assert_eq!(state.rows(), 10);

        let total: f64 = (0..10).map(|r| state.get(r, 0).unwrap_or(0.0)).sum();
        assert_eq!(total, 1.0);
    }

    // =========================================================================
    // Row labels
    // =========================================================================

    #[test]
    fn test_row_labels_are_reversed_high_to_low() {
        let state = DistributionMap::new()
            .add_snapshot("X", &[0.0, 10.0])
            .with_bins(5)
            .with_range(0.0, 10.0)
            .to_heatmap();

        let labels = state.row_labels();
        assert_eq!(labels.len(), 5);

        assert!(labels[0].contains("10.0"));
        assert!(labels[4].starts_with("0.0"));
    }

    // =========================================================================
    // Column labels
    // =========================================================================

    #[test]
    fn test_column_labels_match_snapshot_order() {
        let state = DistributionMap::new()
            .add_snapshot("First", &[1.0])
            .add_snapshot("Second", &[2.0])
            .add_snapshot("Third", &[3.0])
            .with_bins(5)
            .to_heatmap();

        assert_eq!(state.col_labels(), &["First", "Second", "Third"]);
    }

    // =========================================================================
    // Builder patterns
    // =========================================================================

    #[test]
    fn test_default_creates_empty() {
        let dm = DistributionMap::default();
        let state = dm.to_heatmap();
        assert_eq!(state.rows(), 0);
        assert_eq!(state.cols(), 0);
    }

    #[test]
    fn test_builder_chaining() {
        let state = DistributionMap::new()
            .with_bins(8)
            .with_range(-1.0, 1.0)
            .add_snapshot("A", &[0.0])
            .add_snapshot("B", &[0.5])
            .to_heatmap();

        assert_eq!(state.rows(), 8);
        assert_eq!(state.cols(), 2);
    }

    // =========================================================================
    // Internal helpers
    // =========================================================================

    #[test]
    fn test_bin_values_even_distribution() {
        let bins = bin_values(&[0.0, 1.0, 2.0, 3.0], 4, 0.0, 4.0);
        assert_eq!(bins, vec![1, 1, 1, 1]);
    }

    #[test]
    fn test_bin_values_max_value_in_last_bin() {
        let bins = bin_values(&[10.0], 5, 0.0, 10.0);
        assert_eq!(bins[4], 1);
        assert_eq!(bins.iter().sum::<usize>(), 1);
    }

    #[test]
    fn test_bin_values_all_same() {
        let bins = bin_values(&[5.0, 5.0, 5.0], 3, 5.0, 5.0);
        assert_eq!(bins[0], 3);
    }

    #[test]
    fn test_bin_values_empty_input() {
        let bins = bin_values(&[], 5, 0.0, 10.0);
        assert_eq!(bins, vec![0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_format_range_large_bins() {
        let label = format_range(0.0, 2.0);
        assert_eq!(label, "0.0..2.0");
    }

    #[test]
    fn test_format_range_small_bins() {
        let label = format_range(0.0, 0.5);
        assert_eq!(label, "0.00..0.50");
    }

    #[test]
    fn test_format_range_tiny_bins() {
        let label = format_range(0.0, 0.05);
        assert_eq!(label, "0.000..0.050");
    }

    #[test]
    fn test_format_range_very_tiny_bins() {
        let label = format_range(0.0, 0.005);
        assert_eq!(label, "0.0000..0.0050");
    }

    // =========================================================================
    // Selection state
    // =========================================================================

    #[test]
    fn test_heatmap_has_selection_set() {
        let state = DistributionMap::new()
            .add_snapshot("X", &[0.0, 1.0])
            .with_bins(5)
            .to_heatmap();

        assert_eq!(state.selected(), Some((0, 0)));
    }

    // =========================================================================
    // Gradient distribution use case (integration test)
    // =========================================================================

    #[test]
    fn test_gradient_distribution_use_case() {
        let epoch_1: Vec<f64> = (-50..=50).map(|i| i as f64 * 0.02).collect();
        let epoch_5: Vec<f64> = (-50..=50).map(|i| i as f64 * 0.01).collect();
        let epoch_10: Vec<f64> = (-50..=50).map(|i| i as f64 * 0.005).collect();

        let state = DistributionMap::new()
            .add_snapshot("Epoch 1", &epoch_1)
            .add_snapshot("Epoch 5", &epoch_5)
            .add_snapshot("Epoch 10", &epoch_10)
            .with_bins(25)
            .with_range(-1.0, 1.0)
            .to_heatmap();

        assert_eq!(state.rows(), 25);
        assert_eq!(state.cols(), 3);
        assert_eq!(state.col_labels(), &["Epoch 1", "Epoch 5", "Epoch 10"]);
        assert_eq!(state.color_scale(), &HeatmapColorScale::Inferno);
        assert_eq!(state.title(), Some("Distribution Map"));

        for col in 0..3 {
            let total: f64 = (0..25).map(|r| state.get(r, col).unwrap_or(0.0)).sum();
            assert_eq!(total, 101.0);
        }
    }
}
