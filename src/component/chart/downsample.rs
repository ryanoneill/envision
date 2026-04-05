//! LTTB (Largest Triangle Three Buckets) downsampling algorithm.
//!
//! Reduces the number of data points while preserving the visual shape
//! of the data. This is particularly useful for rendering large datasets
//! within limited screen widths.

/// Downsamples data using the Largest Triangle Three Buckets algorithm.
///
/// Preserves the first and last points, and selects representative points
/// from each bucket that form the largest triangle with their neighbors.
///
/// Returns the original data unchanged if `target >= data.len()` or `target < 3`.
///
/// # Example
///
/// ```ignore
/// use envision::component::chart::downsample::lttb;
///
/// let data: Vec<(f64, f64)> = (0..100).map(|i| (i as f64, (i as f64).sin())).collect();
/// let result = lttb(&data, 20);
/// assert_eq!(result.len(), 20);
/// // First and last points are preserved
/// assert_eq!(result[0], data[0]);
/// assert_eq!(result[19], data[99]);
/// ```
pub fn lttb(data: &[(f64, f64)], target: usize) -> Vec<(f64, f64)> {
    let n = data.len();

    // If data is smaller than or equal to target, return as-is
    if n <= target || target < 3 {
        return data.to_vec();
    }

    let mut result = Vec::with_capacity(target);

    // Always include first point
    result.push(data[0]);

    // Number of buckets for intermediate points (excluding first and last)
    let bucket_count = target - 2;
    let bucket_size = (n - 2) as f64 / bucket_count as f64;

    let mut prev_selected = 0;

    for bucket_idx in 0..bucket_count {
        // Current bucket range
        let bucket_start = ((bucket_idx as f64 * bucket_size) as usize) + 1;
        let bucket_end = (((bucket_idx + 1) as f64 * bucket_size) as usize + 1).min(n - 1);

        // Next bucket average (for triangle area computation)
        let next_bucket_start = bucket_end;
        let next_bucket_end = if bucket_idx + 1 < bucket_count {
            (((bucket_idx + 2) as f64 * bucket_size) as usize + 1).min(n - 1)
        } else {
            n // Last bucket extends to end
        };

        let (avg_x, avg_y) = bucket_average(data, next_bucket_start, next_bucket_end);

        // Find the point in the current bucket that forms the largest triangle
        let prev_point = data[prev_selected];
        let mut best_area = -1.0;
        let mut best_idx = bucket_start;

        for (i, point) in data.iter().enumerate().take(bucket_end).skip(bucket_start) {
            let area = triangle_area(prev_point, *point, (avg_x, avg_y));
            if area > best_area {
                best_area = area;
                best_idx = i;
            }
        }

        result.push(data[best_idx]);
        prev_selected = best_idx;
    }

    // Always include last point
    result.push(data[n - 1]);

    result
}

/// Computes the average x and y of points in a range.
fn bucket_average(data: &[(f64, f64)], start: usize, end: usize) -> (f64, f64) {
    let end = end.min(data.len());
    if start >= end {
        return if start < data.len() {
            data[start]
        } else {
            (0.0, 0.0)
        };
    }

    let count = (end - start) as f64;
    let (sum_x, sum_y) = data[start..end]
        .iter()
        .fold((0.0, 0.0), |(sx, sy), (x, y)| (sx + x, sy + y));

    (sum_x / count, sum_y / count)
}

/// Computes the area of a triangle formed by three points.
fn triangle_area(p1: (f64, f64), p2: (f64, f64), p3: (f64, f64)) -> f64 {
    ((p1.0 - p3.0) * (p2.1 - p1.1) - (p1.0 - p2.0) * (p3.1 - p1.1)).abs() * 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Endpoint preservation
    // =========================================================================

    #[test]
    fn test_preserves_endpoints() {
        let data: Vec<(f64, f64)> = (0..50).map(|i| (i as f64, (i as f64) * 2.0)).collect();
        let result = lttb(&data, 10);
        assert_eq!(result[0], data[0]);
        assert_eq!(*result.last().unwrap(), *data.last().unwrap());
    }

    // =========================================================================
    // Output length
    // =========================================================================

    #[test]
    fn test_output_length() {
        let data: Vec<(f64, f64)> = (0..100).map(|i| (i as f64, (i as f64).sin())).collect();
        let result = lttb(&data, 20);
        assert_eq!(result.len(), 20);
    }

    #[test]
    fn test_output_length_large() {
        let data: Vec<(f64, f64)> = (0..1000).map(|i| (i as f64, (i as f64).sin())).collect();
        let result = lttb(&data, 50);
        assert_eq!(result.len(), 50);
    }

    // =========================================================================
    // Fewer than target
    // =========================================================================

    #[test]
    fn test_fewer_than_target() {
        let data = vec![(0.0, 1.0), (1.0, 2.0), (2.0, 3.0)];
        let result = lttb(&data, 10);
        assert_eq!(result.len(), 3);
        assert_eq!(result, data);
    }

    // =========================================================================
    // Exact target
    // =========================================================================

    #[test]
    fn test_exact_target() {
        let data = vec![(0.0, 1.0), (1.0, 2.0), (2.0, 3.0)];
        let result = lttb(&data, 3);
        assert_eq!(result.len(), 3);
    }

    // =========================================================================
    // Empty input
    // =========================================================================

    #[test]
    fn test_empty() {
        let data: Vec<(f64, f64)> = vec![];
        let result = lttb(&data, 10);
        assert!(result.is_empty());
    }

    // =========================================================================
    // Single point
    // =========================================================================

    #[test]
    fn test_single_point() {
        let data = vec![(0.0, 1.0)];
        let result = lttb(&data, 1);
        assert_eq!(result.len(), 1);
    }

    // =========================================================================
    // Spike preservation
    // =========================================================================

    #[test]
    fn test_spike_preserved() {
        // Create data with a prominent spike at position 25
        let mut data: Vec<(f64, f64)> = (0..50).map(|i| (i as f64, 1.0)).collect();
        data[25].1 = 100.0; // Large spike

        let result = lttb(&data, 10);

        // The spike should be preserved since it creates the largest triangle
        let max_y = result.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
        assert_eq!(max_y, 100.0);
    }

    // =========================================================================
    // Large dataset
    // =========================================================================

    #[test]
    fn test_large_dataset() {
        let data: Vec<(f64, f64)> = (0..10_000)
            .map(|i| {
                let x = i as f64;
                let y = (x * 0.01).sin() * 100.0;
                (x, y)
            })
            .collect();
        let result = lttb(&data, 100);
        assert_eq!(result.len(), 100);
        assert_eq!(result[0], data[0]);
        assert_eq!(*result.last().unwrap(), *data.last().unwrap());
    }

    // =========================================================================
    // Monotonic data
    // =========================================================================

    #[test]
    fn test_monotonic() {
        let data: Vec<(f64, f64)> = (0..100).map(|i| (i as f64, i as f64)).collect();
        let result = lttb(&data, 10);
        assert_eq!(result.len(), 10);

        // Result should be monotonically increasing
        for i in 1..result.len() {
            assert!(result[i].1 >= result[i - 1].1);
        }
    }

    // =========================================================================
    // Target less than 3
    // =========================================================================

    #[test]
    fn test_target_less_than_three() {
        let data: Vec<(f64, f64)> = (0..100).map(|i| (i as f64, i as f64)).collect();
        let result = lttb(&data, 2);
        // Returns original data when target < 3
        assert_eq!(result.len(), 100);
    }
}
