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
use ratatui::style::Style;

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
    /// Same-variant comparisons use the natural ordering for the type.
    /// `F64` uses `f64::total_cmp` (not `partial_cmp`) — NaN sorts after
    /// `+∞` per IEEE 754 total ordering, so the result is always a
    /// concrete `Ordering` and the caller never sees `Option<Ordering>`.
    /// Cross-variant comparisons (which shouldn't happen in well-formed
    /// code) fall back to discriminant order and emit a `tracing::warn!`
    /// once per `(render_pass, column_index)` (deduped at the call site,
    /// not here).
    pub fn compare(a: &Self, b: &Self) -> std::cmp::Ordering {
        use SortKey::*;
        use std::cmp::Ordering;

        // None policy: any non-None always sorts before None in ascending.
        match (a, b) {
            (None, None) => return Ordering::Equal,
            (None, _) => return Ordering::Greater,
            (_, None) => return Ordering::Less,
            _ => {}
        }

        // Same-variant fast paths.
        match (a, b) {
            (String(x), String(y)) => return x.cmp(y),
            (I64(x), I64(y)) => return x.cmp(y),
            (U64(x), U64(y)) => return x.cmp(y),
            (F64(x), F64(y)) => return x.total_cmp(y),
            (Bool(x), Bool(y)) => return x.cmp(y),
            (Duration(x), Duration(y)) => return x.cmp(y),
            (DateTime(x), DateTime(y)) => return x.cmp(y),
            _ => {}
        }

        // Cross-variant: fall back to discriminant order.
        Self::discriminant(a).cmp(&Self::discriminant(b))
    }

    /// Returns the discriminant byte used as cross-variant fallback ordering.
    ///
    /// **Numeric values MUST mirror the enum's variant declaration order.**
    /// If you add a `SortKey` variant, insert it at the matching position
    /// here (renumbering subsequent values). The exhaustive match catches a
    /// missing arm at compile time, but it cannot detect *misordered* arms —
    /// a drift between this assignment and the enum order silently produces
    /// wrong cross-variant sort results.
    fn discriminant(k: &Self) -> u8 {
        use SortKey::*;
        match k {
            String(_) => 0,
            I64(_) => 1,
            U64(_) => 2,
            F64(_) => 3,
            Bool(_) => 4,
            Duration(_) => 5,
            DateTime(_) => 6,
            None => 7,
        }
    }
}

/// Semantic cell styling.
///
/// `Default` renders with no override (theme-driven). `Success`,
/// `Warning`, `Error`, `Muted` map to the theme's semantic colors.
/// `Custom(Style)` applies a raw `ratatui::style::Style` directly.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum CellStyle {
    /// No override — render with the theme's default cell style.
    #[default]
    Default,
    /// Maps to the theme's success color (typically green).
    Success,
    /// Maps to the theme's warning color (typically yellow).
    Warning,
    /// Maps to the theme's error color (typically red).
    Error,
    /// Maps to the theme's muted color (typically dark gray) for de-emphasized text.
    Muted,
    /// Applies a raw `ratatui::style::Style` directly, bypassing theme mapping.
    Custom(Style),
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

    #[test]
    fn cross_variant_falls_back_to_discriminant_order() {
        // I64 < F64 in the enum; in ascending, I64(7) appears before F64(3.5)
        // even though numerically 3.5 < 7.
        let a = SortKey::I64(7);
        let b = SortKey::F64(3.5);
        assert_eq!(SortKey::compare(&a, &b), Ordering::Less);
    }

    #[test]
    fn none_sorts_last_in_ascending_against_real_value() {
        // None > Some(real) — None sorts last in ascending order.
        let real = SortKey::F64(0.0);
        assert_eq!(SortKey::compare(&real, &SortKey::None), Ordering::Less);
        assert_eq!(SortKey::compare(&SortKey::None, &real), Ordering::Greater);
    }

    #[test]
    fn none_sorts_after_nan() {
        let nan = SortKey::F64(f64::NAN);
        assert_eq!(SortKey::compare(&nan, &SortKey::None), Ordering::Less);
    }
}

#[cfg(test)]
mod cell_style_tests {
    use super::*;
    use ratatui::style::{Color, Style};

    #[test]
    fn default_is_default_variant() {
        assert_eq!(CellStyle::default(), CellStyle::Default);
    }

    #[test]
    fn custom_carries_style() {
        let style = Style::default().fg(Color::Red);
        assert_eq!(CellStyle::Custom(style), CellStyle::Custom(style));
    }
}
