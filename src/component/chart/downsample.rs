//! Data downsampling algorithms for chart components.
//!
//! Implements the Largest Triangle Three Buckets (LTTB) algorithm for
//! reducing the number of data points while preserving the visual shape
//! of the data. This is critical for rendering large datasets (25000+
//! points) at terminal resolution.

/// Downsamples data using the Largest Triangle Three Buckets algorithm.
///
/// Reduces `data` to at most `target` points while preserving the visual
/// shape. The first and last points are always kept. Interior points are
/// selected by dividing the data into buckets and choosing the point in
/// each bucket that forms the largest triangle with the selected points
/// in adjacent buckets.
///
/// Returns the original data if it has `target` or fewer points.
pub fn lttb(data: &[(f64, f64)], target: usize) -> Vec<(f64, f64)> {
    if data.len() <= target || target < 3 {
        return data.to_vec();
    }

    let mut result = Vec::with_capacity(target);

    // Always keep the first point
    result.push(data[0]);

    let bucket_size = (data.len() - 2) as f64 / (target - 2) as f64;

    let mut prev_selected = 0;

    for i in 0..(target - 2) {
        // Current bucket range
        let bucket_start = ((i as f64 * bucket_size) as usize + 1).min(data.len() - 1);
        let bucket_end = (((i + 1) as f64 * bucket_size) as usize + 1).min(data.len() - 1);

        // Next bucket range (for computing the average point)
        let next_start = bucket_end;
        let next_end = (((i + 2) as f64 * bucket_size) as usize + 1).min(data.len());

        // Average of the next bucket
        let (avg_x, avg_y) = if next_start < next_end {
            let count = (next_end - next_start) as f64;
            let sum_x: f64 = data[next_start..next_end].iter().map(|(x, _)| x).sum();
            let sum_y: f64 = data[next_start..next_end].iter().map(|(_, y)| y).sum();
            (sum_x / count, sum_y / count)
        } else {
            // Last bucket: use the last point
            data[data.len() - 1]
        };

        // Find the point in the current bucket that forms the largest
        // triangle with the previously selected point and the next average
        let (prev_x, prev_y) = data[prev_selected];
        let mut max_area = -1.0;
        let mut max_idx = bucket_start;

        for (j, &(cx, cy)) in data.iter().enumerate().take(bucket_end).skip(bucket_start) {
            // Triangle area (doubled, no division needed for comparison)
            let area =
                ((prev_x - avg_x) * (cy - prev_y) - (prev_x - cx) * (avg_y - prev_y)).abs();
            if area > max_area {
                max_area = area;
                max_idx = j;
            }
        }

        result.push(data[max_idx]);
        prev_selected = max_idx;
    }

    // Always keep the last point
    result.push(data[data.len() - 1]);

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lttb_preserves_endpoints() {
        let data: Vec<(f64, f64)> = (0..100).map(|i| (i as f64, (i as f64).sin())).collect();
        let result = lttb(&data, 10);
        assert_eq!(result.first(), Some(&data[0]));
        assert_eq!(result.last(), Some(&data[99]));
    }

    #[test]
    fn test_lttb_output_length() {
        let data: Vec<(f64, f64)> = (0..100).map(|i| (i as f64, i as f64)).collect();
        let result = lttb(&data, 10);
        assert_eq!(result.len(), 10);
    }

    #[test]
    fn test_lttb_fewer_than_target() {
        let data = vec![(0.0, 1.0), (1.0, 2.0), (2.0, 3.0)];
        let result = lttb(&data, 10);
        assert_eq!(result, data);
    }

    #[test]
    fn test_lttb_exact_target() {
        let data = vec![(0.0, 1.0), (1.0, 2.0), (2.0, 3.0)];
        let result = lttb(&data, 3);
        assert_eq!(result, data);
    }

    #[test]
    fn test_lttb_target_two() {
        // target < 3 returns original data
        let data: Vec<(f64, f64)> = (0..10).map(|i| (i as f64, i as f64)).collect();
        let result = lttb(&data, 2);
        assert_eq!(result, data);
    }

    #[test]
    fn test_lttb_empty() {
        let data: Vec<(f64, f64)> = vec![];
        let result = lttb(&data, 10);
        assert!(result.is_empty());
    }

    #[test]
    fn test_lttb_single_point() {
        let data = vec![(0.0, 42.0)];
        let result = lttb(&data, 10);
        assert_eq!(result, data);
    }

    #[test]
    fn test_lttb_preserves_extremes() {
        // Data with a spike: LTTB should preserve the peak
        let mut data: Vec<(f64, f64)> = (0..100).map(|i| (i as f64, 0.0)).collect();
        data[50] = (50.0, 100.0); // spike at index 50

        let result = lttb(&data, 10);

        // The spike should be preserved
        let max_y = result
            .iter()
            .map(|(_, y)| *y)
            .fold(f64::NEG_INFINITY, f64::max);
        assert_eq!(max_y, 100.0);
    }

    #[test]
    fn test_lttb_large_dataset() {
        let data: Vec<(f64, f64)> = (0..25000)
            .map(|i| {
                let x = i as f64;
                let y = (x / 100.0).sin() * 100.0;
                (x, y)
            })
            .collect();
        let result = lttb(&data, 500);
        assert_eq!(result.len(), 500);
        assert_eq!(result.first(), Some(&data[0]));
        assert_eq!(result.last(), Some(&data[24999]));
    }

    #[test]
    fn test_lttb_monotonic_increasing() {
        // For monotonically increasing data, all selected x values should be increasing
        let data: Vec<(f64, f64)> = (0..100).map(|i| (i as f64, i as f64)).collect();
        let result = lttb(&data, 10);
        for i in 1..result.len() {
            assert!(result[i].0 > result[i - 1].0);
        }
    }
}
