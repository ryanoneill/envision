//! Unified cell type for tabular components.
//!
//! `Cell` is the single cell representation used by `Table` and any
//! tabular component built on `TableRow`. A cell carries display text,
//! optional `CellStyle`, and an optional `SortKey` for typed sorting.
//!
//! Constructors and full usage examples live on the individual types as
//! they're built up across Phase 1 tasks.

#![allow(dead_code)] // Placeholder during Phase 1; tasks 2–9 fill this in.

use compact_str::CompactString;

/// A typed sort key carried by a `Cell` for typed comparison.
///
/// **DO NOT REORDER VARIANTS** — discriminant order is part of the
/// cross-variant fallback contract used by [`SortKey::compare`].
///
/// Same-variant comparisons use the natural ordering for the type
/// (with `f64::total_cmp` for `F64`). Cross-variant comparisons
/// (which shouldn't happen in well-formed code) fall back to
/// discriminant order.
#[derive(Clone, Debug, PartialEq)]
pub enum SortKey {
    /// Lexicographic string sort.
    String(CompactString),
    /// Signed-integer sort.
    I64(i64),
    /// Unsigned-integer sort.
    U64(u64),
    /// Float sort using `f64::total_cmp`. NaN sorts after `+∞`
    /// (and `SortKey::None` sorts after NaN). Real values first,
    /// then NaN, then absent.
    F64(f64),
    /// `false` sorts before `true`.
    Bool(bool),
    /// Duration sort.
    Duration(std::time::Duration),
    /// Older sorts before newer.
    DateTime(std::time::SystemTime),
    /// Absent value. Sorts last in ascending, first in descending —
    /// always at the bottom of the visible list ("nulls last").
    None,
}

impl SortKey {
    /// Compares two sort keys per the documented rules.
    ///
    /// Same-variant comparisons use the natural ordering for the type
    /// (with `f64::total_cmp` for `F64`). Cross-variant comparisons
    /// (which shouldn't happen in well-formed code) fall back to
    /// discriminant order and emit a `tracing::warn!` once per
    /// `(render_pass, column_index)` (deduped at the call site, not here).
    pub fn compare(_a: &Self, _b: &Self) -> std::cmp::Ordering {
        unimplemented!("filled in next task (Task 3)")
    }
}

#[cfg(test)]
mod sort_key_tests {
    use super::*;
    use std::cmp::Ordering;
    use std::time::{Duration, SystemTime};

    #[test]
    fn string_compare_lexicographic() {
        let a = SortKey::String("alice".into());
        let b = SortKey::String("bob".into());
        assert_eq!(SortKey::compare(&a, &b), Ordering::Less);
    }

    #[test]
    fn i64_compare_numeric() {
        let a = SortKey::I64(7);
        let b = SortKey::I64(11);
        assert_eq!(SortKey::compare(&a, &b), Ordering::Less);
    }

    #[test]
    fn u64_compare_numeric() {
        let a = SortKey::U64(7);
        let b = SortKey::U64(11);
        assert_eq!(SortKey::compare(&a, &b), Ordering::Less);
    }

    #[test]
    fn f64_compare_total_order() {
        // total_cmp: 3.5 < 7.0
        let a = SortKey::F64(3.5);
        let b = SortKey::F64(7.0);
        assert_eq!(SortKey::compare(&a, &b), Ordering::Less);
    }

    #[test]
    fn f64_nan_sorts_after_positive_infinity() {
        let inf = SortKey::F64(f64::INFINITY);
        let nan = SortKey::F64(f64::NAN);
        // total_cmp: +∞ < NaN
        assert_eq!(SortKey::compare(&inf, &nan), Ordering::Less);
    }

    #[test]
    fn bool_false_lt_true() {
        let f = SortKey::Bool(false);
        let t = SortKey::Bool(true);
        assert_eq!(SortKey::compare(&f, &t), Ordering::Less);
    }

    #[test]
    fn duration_compare() {
        let a = SortKey::Duration(Duration::from_secs(1));
        let b = SortKey::Duration(Duration::from_secs(2));
        assert_eq!(SortKey::compare(&a, &b), Ordering::Less);
    }

    #[test]
    fn datetime_older_lt_newer() {
        let older = SystemTime::UNIX_EPOCH;
        let newer = older + Duration::from_secs(1);
        assert_eq!(
            SortKey::compare(&SortKey::DateTime(older), &SortKey::DateTime(newer)),
            Ordering::Less,
        );
    }

    #[test]
    fn none_eq_none() {
        assert_eq!(
            SortKey::compare(&SortKey::None, &SortKey::None),
            Ordering::Equal
        );
    }
}
