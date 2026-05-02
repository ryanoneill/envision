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
fn test_sort_by_sets_single_column() {
    let mut state = TableState::new(test_rows(), test_columns());
    Table::<TestRow>::update(&mut state, TableMessage::SortBy(0));
    assert_eq!(state.sort_columns().len(), 1);
    assert_eq!(state.sort_columns()[0], (0, SortDirection::Ascending));
    // backward compat: sort() returns primary
    assert_eq!(state.sort(), Some((0, SortDirection::Ascending)));
}

#[test]
fn test_add_sort_creates_multi_column_sort() {
    let mut state = TableState::new(test_rows(), test_columns());

    // Primary sort by name
    Table::<TestRow>::update(&mut state, TableMessage::SortBy(0));
    assert_eq!(state.sort_columns().len(), 1);

    // Add secondary sort by value
    let output = Table::<TestRow>::update(&mut state, TableMessage::AddSort(1));
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
fn test_add_sort_toggles_existing() {
    let mut state = TableState::new(test_rows(), test_columns());

    // Primary sort by name ascending
    Table::<TestRow>::update(&mut state, TableMessage::SortBy(0));

    // Add sort on same column should toggle direction
    let output = Table::<TestRow>::update(&mut state, TableMessage::AddSort(0));
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
    Table::<TestRow>::update(&mut state, TableMessage::SortBy(0));

    let output = Table::<TestRow>::update(&mut state, TableMessage::AddSort(1));
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
    Table::<TestRow>::update(&mut state, TableMessage::SortBy(0));
    // Then add secondary sort by value (ascending)
    Table::<TestRow>::update(&mut state, TableMessage::AddSort(1));

    // Expect: Alice 10, Alice 20, Bob 30, Charlie 10
    assert_eq!(state.rows()[state.display_order[0]].name, "Alice");
    assert_eq!(state.rows()[state.display_order[0]].value, "10");
    assert_eq!(state.rows()[state.display_order[1]].name, "Alice");
    assert_eq!(state.rows()[state.display_order[1]].value, "20");
    assert_eq!(state.rows()[state.display_order[2]].name, "Bob");
    assert_eq!(state.rows()[state.display_order[3]].name, "Charlie");
}

#[test]
fn test_sort_by_replaces_multi_column_sort() {
    let mut state = TableState::new(test_rows(), test_columns());

    // Set up multi-column sort
    Table::<TestRow>::update(&mut state, TableMessage::SortBy(0));
    Table::<TestRow>::update(&mut state, TableMessage::AddSort(1));
    assert_eq!(state.sort_columns().len(), 2);

    // SortBy on a different column replaces all
    Table::<TestRow>::update(&mut state, TableMessage::SortBy(1));
    assert_eq!(state.sort_columns().len(), 1);
    assert_eq!(state.sort_columns()[0], (1, SortDirection::Ascending));
}

#[test]
fn test_clear_sort_clears_all_columns() {
    let mut state = TableState::new(test_rows(), test_columns());
    Table::<TestRow>::update(&mut state, TableMessage::SortBy(0));
    Table::<TestRow>::update(&mut state, TableMessage::AddSort(1));
    assert_eq!(state.sort_columns().len(), 2);

    let output = Table::<TestRow>::update(&mut state, TableMessage::ClearSort);
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

    Table::<TestRow>::update(&mut state, TableMessage::SortBy(0));
    Table::<TestRow>::update(&mut state, TableMessage::AddSort(1));

    let selected = state.selected_row().unwrap();
    assert_eq!(selected.name, "Alice");
    assert_eq!(selected.value, "10");
}

// NOTE: Custom Comparator Tests were removed when Column::with_comparator
// and the SortComparator/numeric_comparator/date_comparator API were
// dropped (Phase 2 Task 15). Replacement coverage for SortKey-driven
// sorting lands in Phase 3 Task 28.
