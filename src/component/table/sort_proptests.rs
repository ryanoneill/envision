//! Property-based tests for the sort path.
//!
//! Three invariants pinned via proptest:
//!   1. `SortKey::compare` is a total order (antisymmetry + transitivity).
//!   2. Sort is stable across permutations of equal-key inputs.
//!   3. Multi-column sort respects column priority.

use super::*;
use crate::component::cell::{Cell, SortKey};
use proptest::prelude::*;
use std::cmp::Ordering;
use std::time::{Duration, SystemTime};

/// Generator for any `SortKey` variant. Covers every variant of the enum so
/// the totality properties exercise both same-variant and cross-variant
/// comparisons.
fn arb_sort_key() -> impl Strategy<Value = SortKey> {
    prop_oneof![
        any::<String>().prop_map(|s| SortKey::String(s.into())),
        any::<i64>().prop_map(SortKey::I64),
        any::<u64>().prop_map(SortKey::U64),
        any::<f64>().prop_map(SortKey::F64),
        any::<bool>().prop_map(SortKey::Bool),
        any::<u32>().prop_map(|n| SortKey::Duration(Duration::from_secs(u64::from(n)))),
        any::<u32>().prop_map(|n| SortKey::DateTime(
            SystemTime::UNIX_EPOCH + Duration::from_secs(u64::from(n))
        )),
        Just(SortKey::None),
    ]
}

proptest! {
    /// Property #1a: antisymmetry — `compare(a, b) == compare(b, a).reverse()`.
    /// Holds for any pair, including cross-variant pairs (which fall back to
    /// discriminant order) and `None` (which sorts last in ascending).
    #[test]
    fn sort_key_compare_antisymmetric(a in arb_sort_key(), b in arb_sort_key()) {
        let cmp_ab = SortKey::compare(&a, &b);
        let cmp_ba = SortKey::compare(&b, &a);
        prop_assert_eq!(cmp_ab, cmp_ba.reverse());
    }

    /// Property #1b: transitivity — if `a <= b` and `b <= c`, then `a <= c`.
    /// Combined with antisymmetry this establishes a total order over
    /// `SortKey`.
    #[test]
    fn sort_key_compare_transitive(
        a in arb_sort_key(),
        b in arb_sort_key(),
        c in arb_sort_key(),
    ) {
        let ab = SortKey::compare(&a, &b);
        let bc = SortKey::compare(&b, &c);
        let ac = SortKey::compare(&a, &c);
        if matches!(ab, Ordering::Less | Ordering::Equal)
            && matches!(bc, Ordering::Less | Ordering::Equal)
        {
            prop_assert!(matches!(ac, Ordering::Less | Ordering::Equal));
        }
    }
}

/// Property-test row carrying two integer-valued sort columns and a unique
/// id column. The id column lets stability assertions reconstruct the
/// original input order from the sorted display order.
#[derive(Clone, Debug, PartialEq)]
struct PropRow {
    id: u32,
    a: i64,
    b: i64,
}

impl TableRow for PropRow {
    fn cells(&self) -> Vec<Cell> {
        vec![
            Cell::int(self.a),
            Cell::int(self.b),
            Cell::uint(u64::from(self.id)),
        ]
    }
}

fn prop_columns() -> Vec<Column> {
    vec![
        Column::new("A", Constraint::Length(5)).sortable(),
        Column::new("B", Constraint::Length(5)).sortable(),
        Column::new("ID", Constraint::Length(5)).sortable(),
    ]
}

proptest! {
    /// Property #2: sort is stable — equal-key elements preserve input order.
    ///
    /// Build rows with id == input position, sort by column 0 (the `a` key).
    /// For any two consecutive rows in the sorted output that tie on `a`,
    /// the lower id must come first.
    #[test]
    fn sort_is_stable(
        keys in proptest::collection::vec(any::<i64>(), 1..50)
    ) {
        let rows: Vec<PropRow> = keys.iter().enumerate()
            .map(|(i, &k)| PropRow { id: i as u32, a: k, b: 0 })
            .collect();
        let mut state = TableState::new(rows.clone(), prop_columns());
        let _ = Table::<PropRow>::update(&mut state, TableMessage::SortAsc(0));
        let order: Vec<PropRow> = state.display_order.iter()
            .map(|&i| rows[i].clone())
            .collect();
        for window in order.windows(2) {
            let (l, r) = (&window[0], &window[1]);
            if l.a == r.a {
                prop_assert!(
                    l.id < r.id,
                    "Stable sort violated at {:?} vs {:?}", l, r,
                );
            }
        }
    }

    /// Property #3: multi-column sort respects priority.
    ///
    /// Sort stack: [(0, Asc), (1, Desc)]. For any consecutive pair in
    /// the sorted output: column 0 (`a`) must be non-decreasing; within
    /// ties on `a`, column 1 (`b`) must be non-increasing.
    #[test]
    fn multi_column_sort_respects_priority(
        rows_in in proptest::collection::vec((any::<i8>(), any::<i8>()), 2..40)
    ) {
        let rows: Vec<PropRow> = rows_in.iter().enumerate()
            .map(|(i, &(a, b))| PropRow {
                id: i as u32,
                a: i64::from(a),
                b: i64::from(b),
            })
            .collect();
        let mut state = TableState::new(rows.clone(), prop_columns());
        let _ = Table::<PropRow>::update(&mut state, TableMessage::SortAsc(0));
        let _ = Table::<PropRow>::update(&mut state, TableMessage::AddSortDesc(1));
        let order: Vec<PropRow> = state.display_order.iter()
            .map(|&i| rows[i].clone())
            .collect();
        for window in order.windows(2) {
            let (l, r) = (&window[0], &window[1]);
            prop_assert!(
                l.a <= r.a,
                "Primary order violated: {:?} -> {:?}", l, r,
            );
            if l.a == r.a {
                prop_assert!(
                    l.b >= r.b,
                    "Tiebreaker desc violated: {:?} -> {:?}", l, r,
                );
            }
        }
    }
}
