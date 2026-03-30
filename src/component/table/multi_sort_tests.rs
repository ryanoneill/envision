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
    fn cells(&self) -> Vec<String> {
        vec![self.name.clone(), self.value.clone()]
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

// ========== Custom Comparator Tests ==========

#[test]
fn test_numeric_comparator() {
    use super::types::numeric_comparator;
    use std::cmp::Ordering;

    let cmp = numeric_comparator();
    assert_eq!(cmp("2", "10"), Ordering::Less);
    assert_eq!(cmp("10", "2"), Ordering::Greater);
    assert_eq!(cmp("5", "5"), Ordering::Equal);
    assert_eq!(cmp("-1", "0"), Ordering::Less);
    assert_eq!(cmp("1.5", "2.5"), Ordering::Less);
    // Non-numeric values sort after numeric ones
    assert_eq!(cmp("abc", "10"), Ordering::Greater);
    assert_eq!(cmp("10", "abc"), Ordering::Less);
    // Two non-numeric values sort lexicographically
    assert_eq!(cmp("abc", "def"), Ordering::Less);
}

#[test]
fn test_date_comparator() {
    use super::types::date_comparator;
    use std::cmp::Ordering;

    let cmp = date_comparator();
    assert_eq!(cmp("2024-01-15", "2024-02-01"), Ordering::Less);
    assert_eq!(cmp("2024-02-01", "2024-01-15"), Ordering::Greater);
    assert_eq!(cmp("2024-01-01", "2024-01-01"), Ordering::Equal);
    assert_eq!(cmp("2023-12-31", "2024-01-01"), Ordering::Less);
    // Invalid dates sort after valid ones
    assert_eq!(cmp("not-a-date", "2024-01-01"), Ordering::Greater);
    assert_eq!(cmp("2024-01-01", "not-a-date"), Ordering::Less);
}

#[test]
fn test_column_with_comparator() {
    use super::types::numeric_comparator;

    let col = Column::fixed("Price", 10).with_comparator(numeric_comparator());
    assert!(col.is_sortable());
    assert!(col.comparator().is_some());
}

#[test]
fn test_column_without_comparator() {
    let col = Column::fixed("Name", 10).sortable();
    assert!(col.is_sortable());
    assert!(col.comparator().is_none());
}

#[test]
fn test_sort_with_numeric_comparator() {
    use super::types::numeric_comparator;

    let rows = vec![
        TestRow::new("Item", "9"),
        TestRow::new("Item", "10"),
        TestRow::new("Item", "2"),
    ];
    let columns = vec![
        Column::new("Name", Constraint::Length(10)),
        Column::new("Value", Constraint::Length(10)).with_comparator(numeric_comparator()),
    ];
    let mut state = TableState::new(rows, columns);

    Table::<TestRow>::update(&mut state, TableMessage::SortBy(1));

    // With numeric comparator: 2 < 9 < 10
    assert_eq!(state.rows()[state.display_order[0]].value, "2");
    assert_eq!(state.rows()[state.display_order[1]].value, "9");
    assert_eq!(state.rows()[state.display_order[2]].value, "10");
}

#[test]
fn test_sort_with_date_comparator() {
    use super::types::date_comparator;

    let rows = vec![
        TestRow::new("Event A", "2024-12-25"),
        TestRow::new("Event B", "2024-01-01"),
        TestRow::new("Event C", "2024-06-15"),
    ];
    let columns = vec![
        Column::new("Name", Constraint::Length(15)),
        Column::new("Date", Constraint::Length(15)).with_comparator(date_comparator()),
    ];
    let mut state = TableState::new(rows, columns);

    Table::<TestRow>::update(&mut state, TableMessage::SortBy(1));

    // Sorted by date ascending
    assert_eq!(state.rows()[state.display_order[0]].value, "2024-01-01");
    assert_eq!(state.rows()[state.display_order[1]].value, "2024-06-15");
    assert_eq!(state.rows()[state.display_order[2]].value, "2024-12-25");
}

#[test]
fn test_sort_descending_with_comparator() {
    use super::types::numeric_comparator;

    let rows = vec![
        TestRow::new("Item", "9"),
        TestRow::new("Item", "10"),
        TestRow::new("Item", "2"),
    ];
    let columns = vec![
        Column::new("Name", Constraint::Length(10)),
        Column::new("Value", Constraint::Length(10)).with_comparator(numeric_comparator()),
    ];
    let mut state = TableState::new(rows, columns);

    // Sort ascending first, then toggle to descending
    Table::<TestRow>::update(&mut state, TableMessage::SortBy(1));
    Table::<TestRow>::update(&mut state, TableMessage::SortBy(1));

    // With numeric comparator descending: 10 > 9 > 2
    assert_eq!(state.rows()[state.display_order[0]].value, "10");
    assert_eq!(state.rows()[state.display_order[1]].value, "9");
    assert_eq!(state.rows()[state.display_order[2]].value, "2");
}

#[test]
fn test_multi_sort_with_comparator() {
    use super::types::numeric_comparator;

    let rows = vec![
        TestRow::new("Alice", "30"),
        TestRow::new("Alice", "5"),
        TestRow::new("Bob", "20"),
        TestRow::new("Alice", "100"),
    ];
    let columns = vec![
        Column::new("Name", Constraint::Length(10)).sortable(),
        Column::new("Value", Constraint::Length(10)).with_comparator(numeric_comparator()),
    ];
    let mut state = TableState::new(rows, columns);

    // Primary: name (lexicographic), secondary: value (numeric)
    Table::<TestRow>::update(&mut state, TableMessage::SortBy(0));
    Table::<TestRow>::update(&mut state, TableMessage::AddSort(1));

    // Alice 5, Alice 30, Alice 100, Bob 20
    assert_eq!(state.rows()[state.display_order[0]].name, "Alice");
    assert_eq!(state.rows()[state.display_order[0]].value, "5");
    assert_eq!(state.rows()[state.display_order[1]].name, "Alice");
    assert_eq!(state.rows()[state.display_order[1]].value, "30");
    assert_eq!(state.rows()[state.display_order[2]].name, "Alice");
    assert_eq!(state.rows()[state.display_order[2]].value, "100");
    assert_eq!(state.rows()[state.display_order[3]].name, "Bob");
}
