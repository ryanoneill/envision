use super::*;

// Test row type
#[derive(Clone, Debug, PartialEq)]
struct TestRow {
    name: String,
    value: String,
}

impl TestRow {
    fn new(name: &str, value: &str) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

impl TableRow for TestRow {
    fn cells(&self) -> Vec<crate::component::cell::Cell> {
        use crate::component::cell::Cell;
        vec![Cell::new(&self.name), Cell::new(&self.value)]
    }
}

fn test_columns() -> Vec<Column> {
    vec![
        Column::new("Name", Constraint::Length(10)).sortable(),
        Column::new("Value", Constraint::Length(10)).sortable(),
    ]
}

fn test_rows() -> Vec<TestRow> {
    vec![
        TestRow::new("Charlie", "30"),
        TestRow::new("Alice", "10"),
        TestRow::new("Bob", "20"),
    ]
}

// ========== Multi-Column Sort Tests ==========

#[test]
fn test_sort_columns_initially_empty() {
    let state = TableState::new(test_rows(), test_columns());
    assert!(state.sort_columns().is_empty());
    assert!(state.sort().is_none());
}

#[test]
fn test_sort_asc_sets_single_column() {
    let mut state = TableState::new(test_rows(), test_columns());
    Table::<TestRow>::update(&mut state, TableMessage::SortAsc(0));
    assert_eq!(state.sort_columns().len(), 1);
    assert_eq!(state.sort_columns()[0], (0, SortDirection::Ascending));
    // backward compat: sort() returns primary
    assert_eq!(state.sort(), Some((0, SortDirection::Ascending)));
}

#[test]
fn test_add_sort_asc_creates_multi_column_sort() {
    let mut state = TableState::new(test_rows(), test_columns());

    // Primary sort by name
    Table::<TestRow>::update(&mut state, TableMessage::SortAsc(0));
    assert_eq!(state.sort_columns().len(), 1);

    // Add secondary sort by value
    let output = Table::<TestRow>::update(&mut state, TableMessage::AddSortAsc(1));
    assert_eq!(
        output,
        Some(TableOutput::Sorted {
            column: 1,
            direction: SortDirection::Ascending,
        })
    );
    assert_eq!(state.sort_columns().len(), 2);
    assert_eq!(state.sort_columns()[0], (0, SortDirection::Ascending));
    assert_eq!(state.sort_columns()[1], (1, SortDirection::Ascending));
}

#[test]
fn test_add_sort_toggle_flips_existing() {
    let mut state = TableState::new(test_rows(), test_columns());

    // Primary sort by name ascending
    Table::<TestRow>::update(&mut state, TableMessage::SortAsc(0));

    // AddSortToggle on the same column flips direction in place
    let output = Table::<TestRow>::update(&mut state, TableMessage::AddSortToggle(0));
    assert_eq!(
        output,
        Some(TableOutput::Sorted {
            column: 0,
            direction: SortDirection::Descending,
        })
    );
    assert_eq!(state.sort_columns().len(), 1);
    assert_eq!(state.sort_columns()[0], (0, SortDirection::Descending));
}

#[test]
fn test_add_sort_unsortable_column() {
    let columns = vec![
        Column::new("Name", Constraint::Length(10)).sortable(),
        Column::new("Value", Constraint::Length(10)), // not sortable
    ];
    let mut state = TableState::new(test_rows(), columns);
    Table::<TestRow>::update(&mut state, TableMessage::SortAsc(0));

    let output = Table::<TestRow>::update(&mut state, TableMessage::AddSortAsc(1));
    assert_eq!(output, None);
    assert_eq!(state.sort_columns().len(), 1);
}

#[test]
fn test_multi_column_sort_order() {
    // Create rows where primary sort has ties that secondary sort resolves
    let rows = vec![
        TestRow::new("Bob", "30"),
        TestRow::new("Alice", "20"),
        TestRow::new("Alice", "10"),
        TestRow::new("Charlie", "10"),
    ];
    let mut state = TableState::new(rows, test_columns());

    // Primary sort by name (ascending)
    Table::<TestRow>::update(&mut state, TableMessage::SortAsc(0));
    // Then add secondary sort by value (ascending)
    Table::<TestRow>::update(&mut state, TableMessage::AddSortAsc(1));

    // Expect: Alice 10, Alice 20, Bob 30, Charlie 10
    assert_eq!(state.rows()[state.display_order[0]].name, "Alice");
    assert_eq!(state.rows()[state.display_order[0]].value, "10");
    assert_eq!(state.rows()[state.display_order[1]].name, "Alice");
    assert_eq!(state.rows()[state.display_order[1]].value, "20");
    assert_eq!(state.rows()[state.display_order[2]].name, "Bob");
    assert_eq!(state.rows()[state.display_order[3]].name, "Charlie");
}

#[test]
fn test_sort_asc_replaces_multi_column_sort() {
    let mut state = TableState::new(test_rows(), test_columns());

    // Set up multi-column sort
    Table::<TestRow>::update(&mut state, TableMessage::SortAsc(0));
    Table::<TestRow>::update(&mut state, TableMessage::AddSortAsc(1));
    assert_eq!(state.sort_columns().len(), 2);

    // SortAsc on a different column replaces all
    Table::<TestRow>::update(&mut state, TableMessage::SortAsc(1));
    assert_eq!(state.sort_columns().len(), 1);
    assert_eq!(state.sort_columns()[0], (1, SortDirection::Ascending));
}

#[test]
fn test_sort_clear_clears_all_columns() {
    let mut state = TableState::new(test_rows(), test_columns());
    Table::<TestRow>::update(&mut state, TableMessage::SortAsc(0));
    Table::<TestRow>::update(&mut state, TableMessage::AddSortAsc(1));
    assert_eq!(state.sort_columns().len(), 2);

    let output = Table::<TestRow>::update(&mut state, TableMessage::SortClear);
    assert_eq!(output, Some(TableOutput::SortCleared));
    assert!(state.sort_columns().is_empty());
}

#[test]
fn test_multi_sort_preserves_selection() {
    let rows = vec![
        TestRow::new("Bob", "30"),
        TestRow::new("Alice", "20"),
        TestRow::new("Alice", "10"),
    ];
    let mut state = TableState::with_selected(rows, test_columns(), 2);
    // Selected: Alice 10

    Table::<TestRow>::update(&mut state, TableMessage::SortAsc(0));
    Table::<TestRow>::update(&mut state, TableMessage::AddSortAsc(1));

    let selected = state.selected_row().unwrap();
    assert_eq!(selected.name, "Alice");
    assert_eq!(selected.value, "10");
}

// NOTE: Custom Comparator Tests were removed when Column::with_comparator
// and the SortComparator/numeric_comparator/date_comparator API were
// dropped (Phase 2 Task 15). Replacement coverage for SortKey-driven
// sorting lands in Phase 3 Task 28.

// ========== Idempotency Tests ==========
//
// Spec invariant: a sort message that produces no observable change in
// the primary sort or in the stack contents must return `None`. Only an
// observable mutation emits `Sorted { .. }` or `SortCleared`.

/// SortAsc on a column already (Asc) primary must be a no-op (returns None).
#[test]
fn sort_asc_idempotent_when_already_primary_asc() {
    let mut state = TableState::new(test_rows(), test_columns());
    let first = Table::<TestRow>::update(&mut state, TableMessage::SortAsc(0));
    assert_eq!(
        first,
        Some(TableOutput::Sorted {
            column: 0,
            direction: SortDirection::Ascending,
        }),
    );
    let second = Table::<TestRow>::update(&mut state, TableMessage::SortAsc(0));
    assert_eq!(
        second, None,
        "SortAsc on already-Asc primary must return None"
    );
    // Stack unchanged.
    assert_eq!(state.sort_columns().len(), 1);
    assert_eq!(state.sort_columns()[0], (0, SortDirection::Ascending));
}

/// SortDesc on a column already (Desc) primary must be a no-op (returns None).
#[test]
fn sort_desc_idempotent_when_already_primary_desc() {
    let mut state = TableState::new(test_rows(), test_columns());
    let first = Table::<TestRow>::update(&mut state, TableMessage::SortDesc(0));
    assert_eq!(
        first,
        Some(TableOutput::Sorted {
            column: 0,
            direction: SortDirection::Descending,
        }),
    );
    let second = Table::<TestRow>::update(&mut state, TableMessage::SortDesc(0));
    assert_eq!(
        second, None,
        "SortDesc on already-Desc primary must return None"
    );
    assert_eq!(state.sort_columns().len(), 1);
    assert_eq!(state.sort_columns()[0], (0, SortDirection::Descending));
}

/// AddSortAsc on a column already in stack with Asc must be a no-op (returns None).
#[test]
fn add_sort_asc_idempotent_when_already_asc_in_stack() {
    let mut state = TableState::new(test_rows(), test_columns());
    // Build a 2-column stack: primary (0, Asc), tiebreaker (1, Asc).
    Table::<TestRow>::update(&mut state, TableMessage::SortAsc(0));
    let added = Table::<TestRow>::update(&mut state, TableMessage::AddSortAsc(1));
    assert_eq!(
        added,
        Some(TableOutput::Sorted {
            column: 1,
            direction: SortDirection::Ascending,
        }),
    );
    // Re-issue AddSortAsc on the same tiebreaker — must be a no-op.
    let again = Table::<TestRow>::update(&mut state, TableMessage::AddSortAsc(1));
    assert_eq!(
        again, None,
        "AddSortAsc on already-Asc entry must return None"
    );
    // Stack contents unchanged.
    assert_eq!(state.sort_columns().len(), 2);
    assert_eq!(state.sort_columns()[0], (0, SortDirection::Ascending));
    assert_eq!(state.sort_columns()[1], (1, SortDirection::Ascending));
}

/// AddSortDesc on a column already in stack with Desc must be a no-op (returns None).
#[test]
fn add_sort_desc_idempotent_when_already_desc_in_stack() {
    let mut state = TableState::new(test_rows(), test_columns());
    Table::<TestRow>::update(&mut state, TableMessage::SortAsc(0));
    let added = Table::<TestRow>::update(&mut state, TableMessage::AddSortDesc(1));
    assert_eq!(
        added,
        Some(TableOutput::Sorted {
            column: 1,
            direction: SortDirection::Descending,
        }),
    );
    let again = Table::<TestRow>::update(&mut state, TableMessage::AddSortDesc(1));
    assert_eq!(
        again, None,
        "AddSortDesc on already-Desc entry must return None"
    );
    assert_eq!(state.sort_columns().len(), 2);
    assert_eq!(state.sort_columns()[0], (0, SortDirection::Ascending));
    assert_eq!(state.sort_columns()[1], (1, SortDirection::Descending));
}

/// AddSortAsc on a column already in stack with Desc must flip direction in
/// place (preserve stack position) and emit Sorted.
#[test]
fn add_sort_asc_flips_existing_desc_in_place() {
    let mut state = TableState::new(test_rows(), test_columns());
    Table::<TestRow>::update(&mut state, TableMessage::SortAsc(0));
    Table::<TestRow>::update(&mut state, TableMessage::AddSortDesc(1));
    // Now flip the tiebreaker from Desc to Asc.
    let flipped = Table::<TestRow>::update(&mut state, TableMessage::AddSortAsc(1));
    assert_eq!(
        flipped,
        Some(TableOutput::Sorted {
            column: 1,
            direction: SortDirection::Ascending,
        }),
    );
    // Position preserved: (0, Asc) primary, (1, Asc) tiebreaker.
    assert_eq!(state.sort_columns().len(), 2);
    assert_eq!(state.sort_columns()[0], (0, SortDirection::Ascending));
    assert_eq!(state.sort_columns()[1], (1, SortDirection::Ascending));
}

/// RemoveSort on a tiebreaker (non-primary) entry must return None — the
/// primary is unchanged, so there is no observable sort change.
#[test]
fn remove_sort_tiebreaker_returns_none_with_unchanged_primary() {
    let mut state = TableState::new(test_rows(), test_columns());
    Table::<TestRow>::update(&mut state, TableMessage::SortAsc(0)); // primary
    Table::<TestRow>::update(&mut state, TableMessage::AddSortAsc(1)); // tiebreaker
    assert_eq!(state.sort_columns().len(), 2);

    let result = Table::<TestRow>::update(&mut state, TableMessage::RemoveSort(1));
    assert_eq!(
        result, None,
        "Removing a tiebreaker, primary unchanged → None",
    );
    // Primary still in place.
    assert_eq!(state.sort_columns().len(), 1);
    assert_eq!(state.sort(), Some((0, SortDirection::Ascending)));
}

/// RemoveSort on the primary while tiebreakers remain must promote the next
/// stack entry to primary and emit Sorted with the new primary's column/dir.
#[test]
fn remove_sort_primary_promotes_next_emits_sorted_with_new_primary() {
    let mut state = TableState::new(test_rows(), test_columns());
    Table::<TestRow>::update(&mut state, TableMessage::SortAsc(0)); // primary
    Table::<TestRow>::update(&mut state, TableMessage::AddSortDesc(1)); // tiebreaker
    assert_eq!(state.sort_columns().len(), 2);

    // Remove the primary (col 0); col 1 (Desc) must promote.
    let result = Table::<TestRow>::update(&mut state, TableMessage::RemoveSort(0));
    assert_eq!(
        result,
        Some(TableOutput::Sorted {
            column: 1,
            direction: SortDirection::Descending,
        }),
        "Removing primary with tiebreakers remaining must emit Sorted with the new primary",
    );
    assert_eq!(state.sort_columns().len(), 1);
    assert_eq!(state.sort(), Some((1, SortDirection::Descending)));
}

// ========== Initial Sort Builder Tests ==========

#[test]
fn with_initial_sort_renders_sorted_on_frame_1() {
    use crate::component::cell::Cell;
    use crate::component::{Column, SortDirection, TableRow, TableState};
    use ratatui::layout::Constraint;

    #[derive(Clone)]
    struct R(u8);
    impl TableRow for R {
        fn cells(&self) -> Vec<Cell> {
            vec![Cell::int(self.0 as i64)]
        }
    }
    let columns = vec![Column::new("V", Constraint::Length(5)).sortable()];
    let state = TableState::new(vec![R(3), R(1), R(2)], columns)
        .with_initial_sort(0, SortDirection::Ascending);
    // Without any update() calls, sort_columns is set
    assert_eq!(state.sort(), Some((0, SortDirection::Ascending)));
    // And display_order should reflect sorted order: row 1 (val 1), row 2 (val 2), row 0 (val 3)
    assert_eq!(state.display_order, vec![1, 2, 0]);
}

#[test]
fn with_initial_sort_non_sortable_col_still_sets_sort() {
    use crate::component::cell::Cell;
    use crate::component::{Column, SortDirection, TableRow, TableState};
    use ratatui::layout::Constraint;

    #[derive(Clone)]
    struct R(u8);
    impl TableRow for R {
        fn cells(&self) -> Vec<Cell> {
            vec![Cell::int(self.0 as i64)]
        }
    }
    let columns = vec![Column::new("V", Constraint::Length(5))]; // NOT sortable
    let state =
        TableState::new(vec![R(1)], columns).with_initial_sort(0, SortDirection::Descending);
    // Initial sort declarative — set anyway
    assert_eq!(state.sort(), Some((0, SortDirection::Descending)));
}

#[test]
fn with_initial_sorts_multi_column() {
    use crate::component::cell::Cell;
    use crate::component::{Column, InitialSort, SortDirection, TableRow, TableState};
    use ratatui::layout::Constraint;

    #[derive(Clone)]
    struct R(u8, u8);
    impl TableRow for R {
        fn cells(&self) -> Vec<Cell> {
            vec![Cell::int(self.0 as i64), Cell::int(self.1 as i64)]
        }
    }
    let columns = vec![
        Column::new("A", Constraint::Length(5)).sortable(),
        Column::new("B", Constraint::Length(5)).sortable(),
    ];
    let state = TableState::new(vec![R(1, 2), R(2, 1), R(1, 1)], columns).with_initial_sorts(vec![
        InitialSort {
            column: 0,
            direction: SortDirection::Ascending,
        },
        InitialSort {
            column: 1,
            direction: SortDirection::Ascending,
        },
    ]);
    assert_eq!(state.sort_columns().len(), 2);
    // R(1,1) before R(1,2) (tiebreaker on col 1 ascending), then R(2,1)
    assert_eq!(state.display_order, vec![2, 0, 1]);
}

// ========== Spec #14: SortKey::None preserves insertion order under stable sort ==========

/// End-to-end pin that proves `slice::sort_by` (stable) — not
/// `sort_unstable_by` — is in use. With three consecutive `SortKey::None`
/// rows interleaved among real values, ascending sort must place real
/// values first (`SortKey::None` sorts last in ascending) and the Nones
/// must remain in their original insertion order.
#[test]
fn sort_key_none_rows_preserve_insertion_order_under_stable_sort() {
    use crate::component::cell::{Cell, SortKey};

    #[derive(Clone, PartialEq, Debug)]
    struct R {
        id: u8,
        key: SortKey,
    }
    impl TableRow for R {
        fn cells(&self) -> Vec<Cell> {
            vec![Cell::new(format!("{}", self.id)).with_sort_key(self.key.clone())]
        }
    }
    let rows = vec![
        R {
            id: 1,
            key: SortKey::I64(1),
        },
        R {
            id: 2,
            key: SortKey::None,
        }, // a
        R {
            id: 3,
            key: SortKey::None,
        }, // b
        R {
            id: 4,
            key: SortKey::None,
        }, // c
        R {
            id: 5,
            key: SortKey::I64(2),
        },
    ];
    let columns = vec![Column::new("V", Constraint::Length(5)).sortable()];
    let mut state = TableState::new(rows.clone(), columns);
    let _ = Table::<R>::update(&mut state, TableMessage::SortAsc(0));
    // Real values first (1, 5), then Nones in original order (2, 3, 4)
    let order: Vec<u8> = state.display_order.iter().map(|&i| rows[i].id).collect();
    assert_eq!(
        order,
        vec![1, 5, 2, 3, 4],
        "Real values first, then Nones in insertion order — proves stable sort"
    );
}

// ========== Spec #15a: originating leadline bug pin ==========

/// The originating leadline bug: dispatching `SortToggle` 10× must always
/// leave `state.sort()` = `Some(_)`, never `None`. The pre-redesign
/// `SortBy(col)` 3-cycle (Asc → Desc → cleared) had this bug; `SortToggle`
/// is now a strict 2-cycle Asc ↔ Desc that never clears.
#[test]
fn sort_toggle_state_persists_on_repeated_press() {
    use crate::component::cell::Cell;

    #[derive(Clone)]
    struct R(u8);
    impl TableRow for R {
        fn cells(&self) -> Vec<Cell> {
            vec![Cell::int(self.0 as i64)]
        }
    }
    let rows = vec![R(1), R(2), R(3)];
    let columns = vec![Column::new("V", Constraint::Length(5)).sortable()];
    let mut state = TableState::new(rows, columns);

    for i in 0..10 {
        let _ = Table::<R>::update(&mut state, TableMessage::SortToggle(0));
        assert!(
            state.sort().is_some(),
            "SortToggle press {} cleared the sort — bug regressed",
            i + 1,
        );
    }
}

// ========== Spec #16: SortToggle column-switch honors new column's default_sort ==========

/// When `SortToggle` switches to a column not currently primary, it must
/// activate that column at its declared `default_sort` (not always Asc).
#[test]
fn sort_toggle_column_switch_honors_new_column_default_sort() {
    use crate::component::cell::Cell;

    #[derive(Clone)]
    struct R(u8, u8);
    impl TableRow for R {
        fn cells(&self) -> Vec<Cell> {
            vec![Cell::int(self.0 as i64), Cell::int(self.1 as i64)]
        }
    }
    let columns = vec![
        Column::new("A", Constraint::Length(5)).sortable(),
        Column::new("B", Constraint::Length(5))
            .sortable()
            .with_default_sort(SortDirection::Descending),
    ];
    let mut state = TableState::new(vec![R(1, 1), R(2, 2)], columns);
    // Stack starts (col 0, Asc)
    let _ = Table::<R>::update(&mut state, TableMessage::SortAsc(0));
    assert_eq!(state.sort(), Some((0, SortDirection::Ascending)));
    // Toggle col 1: should activate using col 1's default_sort = Descending
    let _ = Table::<R>::update(&mut state, TableMessage::SortToggle(1));
    assert_eq!(state.sort(), Some((1, SortDirection::Descending)));
}

// ========== Spec #17: AddSort* position preservation on existing entries ==========

/// AddSortAsc on a column already in the stack must replace its direction
/// in place — the column's stack position must not change.
#[test]
fn add_sort_asc_replaces_direction_in_place_no_reorder() {
    // Stack: [(0, Asc), (1, Desc), (2, Asc)]; dispatch AddSortAsc(1)
    // Expect: [(0, Asc), (1, Asc), (2, Asc)] — col 1 stays at position 1
    use crate::component::cell::Cell;

    #[derive(Clone)]
    struct R(u8, u8, u8);
    impl TableRow for R {
        fn cells(&self) -> Vec<Cell> {
            vec![
                Cell::int(self.0 as i64),
                Cell::int(self.1 as i64),
                Cell::int(self.2 as i64),
            ]
        }
    }
    let columns = vec![
        Column::new("A", Constraint::Length(5)).sortable(),
        Column::new("B", Constraint::Length(5)).sortable(),
        Column::new("C", Constraint::Length(5)).sortable(),
    ];
    let mut state = TableState::new(vec![R(1, 1, 1), R(2, 2, 2)], columns);
    // Build stack [(0, Asc), (1, Desc), (2, Asc)]
    let _ = Table::<R>::update(&mut state, TableMessage::SortAsc(0));
    let _ = Table::<R>::update(&mut state, TableMessage::AddSortDesc(1));
    let _ = Table::<R>::update(&mut state, TableMessage::AddSortAsc(2));
    assert_eq!(
        state.sort_columns(),
        &[
            (0, SortDirection::Ascending),
            (1, SortDirection::Descending),
            (2, SortDirection::Ascending),
        ]
    );
    // Flip col 1 from Desc to Asc
    let _ = Table::<R>::update(&mut state, TableMessage::AddSortAsc(1));
    // Position must be unchanged — col 1 stays at index 1
    assert_eq!(
        state.sort_columns(),
        &[
            (0, SortDirection::Ascending),
            (1, SortDirection::Ascending),
            (2, SortDirection::Ascending),
        ]
    );
}

/// AddSortToggle on a column already in the stack must flip its direction
/// in place — the column's stack position must not change.
#[test]
fn add_sort_toggle_replaces_direction_in_place_no_reorder() {
    use crate::component::cell::Cell;

    #[derive(Clone)]
    struct R(u8, u8, u8);
    impl TableRow for R {
        fn cells(&self) -> Vec<Cell> {
            vec![
                Cell::int(self.0 as i64),
                Cell::int(self.1 as i64),
                Cell::int(self.2 as i64),
            ]
        }
    }
    let columns = vec![
        Column::new("A", Constraint::Length(5)).sortable(),
        Column::new("B", Constraint::Length(5)).sortable(),
        Column::new("C", Constraint::Length(5)).sortable(),
    ];
    let mut state = TableState::new(vec![R(1, 1, 1), R(2, 2, 2)], columns);
    let _ = Table::<R>::update(&mut state, TableMessage::SortAsc(0));
    let _ = Table::<R>::update(&mut state, TableMessage::AddSortDesc(1));
    let _ = Table::<R>::update(&mut state, TableMessage::AddSortAsc(2));
    // Toggle col 1 from Desc to Asc — position-preserving
    let _ = Table::<R>::update(&mut state, TableMessage::AddSortToggle(1));
    assert_eq!(
        state.sort_columns(),
        &[
            (0, SortDirection::Ascending),
            (1, SortDirection::Ascending), // toggled from Desc, position 1 preserved
            (2, SortDirection::Ascending),
        ]
    );
}

// ========== Spec #13 expanded: non-sortable column silent no-op for ALL variants ==========

/// Every column-taking sort variant must silently no-op (return `None` and
/// leave `sort()` unchanged) when dispatched against a non-sortable column.
/// `SortClear` is excluded because it does not take a column index.
#[test]
fn all_sort_variants_silent_noop_on_nonsortable_column() {
    use crate::component::cell::Cell;

    #[derive(Clone)]
    struct R(u8);
    impl TableRow for R {
        fn cells(&self) -> Vec<Cell> {
            vec![Cell::int(self.0 as i64)]
        }
    }
    let columns = vec![Column::new("V", Constraint::Length(5))]; // NOT sortable
    let mut state = TableState::new(vec![R(1)], columns);

    let variants = [
        TableMessage::SortAsc(0),
        TableMessage::SortDesc(0),
        TableMessage::SortToggle(0),
        TableMessage::RemoveSort(0),
        TableMessage::AddSortAsc(0),
        TableMessage::AddSortDesc(0),
        TableMessage::AddSortToggle(0),
    ];
    for variant in variants {
        let result = Table::<R>::update(&mut state, variant.clone());
        assert!(
            result.is_none(),
            "{:?} on non-sortable col must return None",
            variant,
        );
        assert!(
            state.sort().is_none(),
            "{:?} must not change sort state",
            variant,
        );
    }
}

// ========== RemoveSort completeness ==========

/// `RemoveSort(col)` removing the only sort entry must clear the stack
/// and emit `SortCleared`.
#[test]
fn remove_sort_only_entry_emits_sort_cleared() {
    let mut state = TableState::new(test_rows(), test_columns());
    Table::<TestRow>::update(&mut state, TableMessage::SortAsc(0));
    assert_eq!(state.sort_columns().len(), 1);

    let result = Table::<TestRow>::update(&mut state, TableMessage::RemoveSort(0));
    assert_eq!(
        result,
        Some(TableOutput::SortCleared),
        "Removing the sole sort entry must emit SortCleared"
    );
    assert!(state.sort_columns().is_empty());
    assert_eq!(state.sort(), None);
}

/// `RemoveSort(col)` where `col` is not in the stack must return `None`
/// and leave the sort stack unchanged.
#[test]
fn remove_sort_column_not_in_stack_returns_none() {
    let mut state = TableState::new(test_rows(), test_columns());
    Table::<TestRow>::update(&mut state, TableMessage::SortAsc(0));
    assert_eq!(state.sort_columns().len(), 1);

    // Col 1 is sortable but not in the stack.
    let result = Table::<TestRow>::update(&mut state, TableMessage::RemoveSort(1));
    assert_eq!(
        result, None,
        "RemoveSort on a column not in the stack must return None"
    );
    assert_eq!(state.sort_columns().len(), 1);
    assert_eq!(state.sort(), Some((0, SortDirection::Ascending)));
}
