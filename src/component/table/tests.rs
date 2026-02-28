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

// TableRow Trait Tests

#[test]
fn test_tablerow_impl() {
    let row = TestRow::new("Test", "123");
    assert_eq!(row.cells(), vec!["Test", "123"]);
}

#[test]
fn test_tablerow_empty_cells() {
    #[derive(Clone)]
    struct EmptyRow;

    impl TableRow for EmptyRow {
        fn cells(&self) -> Vec<String> {
            vec![]
        }
    }

    let row = EmptyRow;
    assert!(row.cells().is_empty());
}

// Column Tests

#[test]
fn test_column_new() {
    let col = Column::new("Header", Constraint::Length(15));
    assert_eq!(col.header(), "Header");
    assert!(!col.is_sortable());
}

#[test]
fn test_column_sortable() {
    let col = Column::new("Header", Constraint::Length(15)).sortable();
    assert!(col.is_sortable());
}

#[test]
fn test_column_clone() {
    let col = Column::new("Header", Constraint::Length(15)).sortable();
    let cloned = col.clone();
    assert_eq!(cloned.header(), "Header");
    assert!(cloned.is_sortable());
}

#[test]
fn test_column_width() {
    let col = Column::new("Header", Constraint::Percentage(50));
    assert_eq!(col.width(), Constraint::Percentage(50));
}

// SortDirection Tests

#[test]
fn test_sort_direction_toggle() {
    assert_eq!(SortDirection::Ascending.toggle(), SortDirection::Descending);
    assert_eq!(SortDirection::Descending.toggle(), SortDirection::Ascending);
}

#[test]
fn test_sort_direction_default() {
    let dir: SortDirection = Default::default();
    assert_eq!(dir, SortDirection::Ascending);
}

// State Creation Tests

#[test]
fn test_new() {
    let state = TableState::new(test_rows(), test_columns());
    assert_eq!(state.len(), 3);
    assert_eq!(state.selected_index(), Some(0));
    assert!(state.sort().is_none());
}

#[test]
fn test_new_empty() {
    let state: TableState<TestRow> = TableState::new(vec![], test_columns());
    assert!(state.is_empty());
    assert_eq!(state.selected_index(), None);
}

#[test]
fn test_with_selected() {
    let state = TableState::with_selected(test_rows(), test_columns(), 2);
    assert_eq!(state.selected_index(), Some(2));
}

#[test]
fn test_with_selected_clamps() {
    let state = TableState::with_selected(test_rows(), test_columns(), 100);
    assert_eq!(state.selected_index(), Some(2)); // Clamped to last
}

#[test]
fn test_default() {
    let state: TableState<TestRow> = TableState::default();
    assert!(state.is_empty());
    assert_eq!(state.selected_index(), None);
    assert!(state.columns().is_empty());
}

// Accessors Tests

#[test]
fn test_rows_accessor() {
    let state = TableState::new(test_rows(), test_columns());
    assert_eq!(state.rows().len(), 3);
}

#[test]
fn test_columns_accessor() {
    let state = TableState::new(test_rows(), test_columns());
    assert_eq!(state.columns().len(), 2);
}

#[test]
fn test_selected_index() {
    let state = TableState::with_selected(test_rows(), test_columns(), 1);
    assert_eq!(state.selected_index(), Some(1));
}

#[test]
fn test_selected_row() {
    let state = TableState::with_selected(test_rows(), test_columns(), 1);
    let row = state.selected_row().unwrap();
    assert_eq!(row.name, "Alice");
}

#[test]
fn test_sort() {
    let state = TableState::new(test_rows(), test_columns());
    assert!(state.sort().is_none());
}

#[test]
fn test_len() {
    let state = TableState::new(test_rows(), test_columns());
    assert_eq!(state.len(), 3);
}

#[test]
fn test_is_empty() {
    let empty: TableState<TestRow> = TableState::new(vec![], vec![]);
    assert!(empty.is_empty());

    let not_empty = TableState::new(test_rows(), test_columns());
    assert!(!not_empty.is_empty());
}

// Mutators Tests

#[test]
fn test_set_rows() {
    let mut state = TableState::new(test_rows(), test_columns());
    state.set_rows(vec![TestRow::new("New", "1")]);
    assert_eq!(state.len(), 1);
    assert_eq!(state.selected_index(), Some(0));
}

#[test]
fn test_set_rows_preserves_selection() {
    let mut state = TableState::with_selected(test_rows(), test_columns(), 1);
    state.set_rows(vec![
        TestRow::new("A", "1"),
        TestRow::new("B", "2"),
        TestRow::new("C", "3"),
    ]);
    assert_eq!(state.selected_index(), Some(1));
}

#[test]
fn test_set_rows_clamps_selection() {
    let mut state = TableState::with_selected(test_rows(), test_columns(), 2);
    state.set_rows(vec![TestRow::new("A", "1")]);
    assert_eq!(state.selected_index(), Some(0)); // Clamped
}

#[test]
fn test_set_selected() {
    let mut state = TableState::new(test_rows(), test_columns());
    state.set_selected(Some(2));
    assert_eq!(state.selected_index(), Some(2));

    state.set_selected(None);
    assert_eq!(state.selected_index(), None);
}

// Navigation Tests

#[test]
fn test_down() {
    let mut state = TableState::new(test_rows(), test_columns());
    let output = Table::<TestRow>::update(&mut state, TableMessage::Down);
    assert_eq!(output, Some(TableOutput::SelectionChanged(1)));
    assert_eq!(state.selected_index(), Some(1));
}

#[test]
fn test_down_at_last() {
    let mut state = TableState::with_selected(test_rows(), test_columns(), 2);
    let output = Table::<TestRow>::update(&mut state, TableMessage::Down);
    assert_eq!(output, None);
    assert_eq!(state.selected_index(), Some(2));
}

#[test]
fn test_up() {
    let mut state = TableState::with_selected(test_rows(), test_columns(), 1);
    let output = Table::<TestRow>::update(&mut state, TableMessage::Up);
    assert_eq!(output, Some(TableOutput::SelectionChanged(0)));
    assert_eq!(state.selected_index(), Some(0));
}

#[test]
fn test_up_at_first() {
    let mut state = TableState::new(test_rows(), test_columns());
    let output = Table::<TestRow>::update(&mut state, TableMessage::Up);
    assert_eq!(output, None);
    assert_eq!(state.selected_index(), Some(0));
}

#[test]
fn test_first() {
    let mut state = TableState::with_selected(test_rows(), test_columns(), 2);
    let output = Table::<TestRow>::update(&mut state, TableMessage::First);
    assert_eq!(output, Some(TableOutput::SelectionChanged(0)));
    assert_eq!(state.selected_index(), Some(0));
}

#[test]
fn test_first_already_first() {
    let mut state = TableState::new(test_rows(), test_columns());
    let output = Table::<TestRow>::update(&mut state, TableMessage::First);
    assert_eq!(output, None);
}

#[test]
fn test_last() {
    let mut state = TableState::new(test_rows(), test_columns());
    let output = Table::<TestRow>::update(&mut state, TableMessage::Last);
    assert_eq!(output, Some(TableOutput::SelectionChanged(2)));
    assert_eq!(state.selected_index(), Some(2));
}

#[test]
fn test_last_already_last() {
    let mut state = TableState::with_selected(test_rows(), test_columns(), 2);
    let output = Table::<TestRow>::update(&mut state, TableMessage::Last);
    assert_eq!(output, None);
}

#[test]
fn test_page_down() {
    let mut state = TableState::new(test_rows(), test_columns());
    let output = Table::<TestRow>::update(&mut state, TableMessage::PageDown(2));
    assert_eq!(output, Some(TableOutput::SelectionChanged(2)));
}

#[test]
fn test_page_up() {
    let mut state = TableState::with_selected(test_rows(), test_columns(), 2);
    let output = Table::<TestRow>::update(&mut state, TableMessage::PageUp(2));
    assert_eq!(output, Some(TableOutput::SelectionChanged(0)));
}

#[test]
fn test_select() {
    let mut state = TableState::with_selected(test_rows(), test_columns(), 1);
    let output = Table::<TestRow>::update(&mut state, TableMessage::Select);
    assert_eq!(
        output,
        Some(TableOutput::Selected(TestRow::new("Alice", "10")))
    );
}

#[test]
fn test_empty_navigation() {
    let mut state: TableState<TestRow> = TableState::new(vec![], test_columns());

    assert_eq!(
        Table::<TestRow>::update(&mut state, TableMessage::Down),
        None
    );
    assert_eq!(Table::<TestRow>::update(&mut state, TableMessage::Up), None);
    assert_eq!(
        Table::<TestRow>::update(&mut state, TableMessage::Select),
        None
    );
}

// Sorting Tests

#[test]
fn test_sort_ascending() {
    let mut state = TableState::new(test_rows(), test_columns());
    let output = Table::<TestRow>::update(&mut state, TableMessage::SortBy(0));
    assert_eq!(
        output,
        Some(TableOutput::Sorted {
            column: 0,
            direction: SortDirection::Ascending,
        })
    );

    // Check order: Alice, Bob, Charlie
    assert_eq!(state.rows()[state.display_order[0]].name, "Alice");
    assert_eq!(state.rows()[state.display_order[1]].name, "Bob");
    assert_eq!(state.rows()[state.display_order[2]].name, "Charlie");
}

#[test]
fn test_sort_descending() {
    let mut state = TableState::new(test_rows(), test_columns());
    Table::<TestRow>::update(&mut state, TableMessage::SortBy(0)); // Ascending
    let output = Table::<TestRow>::update(&mut state, TableMessage::SortBy(0)); // Descending
    assert_eq!(
        output,
        Some(TableOutput::Sorted {
            column: 0,
            direction: SortDirection::Descending,
        })
    );

    // Check order: Charlie, Bob, Alice
    assert_eq!(state.rows()[state.display_order[0]].name, "Charlie");
    assert_eq!(state.rows()[state.display_order[1]].name, "Bob");
    assert_eq!(state.rows()[state.display_order[2]].name, "Alice");
}

#[test]
fn test_sort_clear() {
    let mut state = TableState::new(test_rows(), test_columns());
    Table::<TestRow>::update(&mut state, TableMessage::SortBy(0)); // Ascending
    Table::<TestRow>::update(&mut state, TableMessage::SortBy(0)); // Descending
    let output = Table::<TestRow>::update(&mut state, TableMessage::SortBy(0)); // Clear
    assert_eq!(output, Some(TableOutput::SortCleared));
    assert!(state.sort().is_none());

    // Back to original order: Charlie, Alice, Bob
    assert_eq!(state.rows()[state.display_order[0]].name, "Charlie");
    assert_eq!(state.rows()[state.display_order[1]].name, "Alice");
    assert_eq!(state.rows()[state.display_order[2]].name, "Bob");
}

#[test]
fn test_sort_unsortable_column() {
    let columns = vec![
        Column::new("Name", Constraint::Length(10)), // Not sortable
        Column::new("Value", Constraint::Length(10)).sortable(),
    ];
    let mut state = TableState::new(test_rows(), columns);
    let output = Table::<TestRow>::update(&mut state, TableMessage::SortBy(0));
    assert_eq!(output, None);
}

#[test]
fn test_sort_preserves_selection() {
    let mut state = TableState::with_selected(test_rows(), test_columns(), 1);
    // Initially selected: Alice (index 1 in original order)

    Table::<TestRow>::update(&mut state, TableMessage::SortBy(0)); // Sort ascending

    // After sort, Alice should still be selected but at a different display index
    let selected = state.selected_row().unwrap();
    assert_eq!(selected.name, "Alice");
}

#[test]
fn test_sort_numeric_strings() {
    // Numeric strings sort lexicographically, not numerically
    let rows = vec![
        TestRow::new("Item", "9"),
        TestRow::new("Item", "10"),
        TestRow::new("Item", "2"),
    ];
    let columns = vec![
        Column::new("Name", Constraint::Length(10)),
        Column::new("Value", Constraint::Length(10)).sortable(),
    ];
    let mut state = TableState::new(rows, columns);

    Table::<TestRow>::update(&mut state, TableMessage::SortBy(1));

    // Lexicographic: "10" < "2" < "9"
    assert_eq!(state.rows()[state.display_order[0]].value, "10");
    assert_eq!(state.rows()[state.display_order[1]].value, "2");
    assert_eq!(state.rows()[state.display_order[2]].value, "9");
}

#[test]
fn test_clear_sort() {
    let mut state = TableState::new(test_rows(), test_columns());
    Table::<TestRow>::update(&mut state, TableMessage::SortBy(0));
    assert!(state.sort().is_some());

    let output = Table::<TestRow>::update(&mut state, TableMessage::ClearSort);
    assert_eq!(output, Some(TableOutput::SortCleared));
    assert!(state.sort().is_none());
}

#[test]
fn test_clear_sort_when_not_sorted() {
    let mut state = TableState::new(test_rows(), test_columns());
    let output = Table::<TestRow>::update(&mut state, TableMessage::ClearSort);
    assert_eq!(output, None);
}

#[test]
fn test_sort_different_column() {
    let mut state = TableState::new(test_rows(), test_columns());

    // Sort by column 0
    Table::<TestRow>::update(&mut state, TableMessage::SortBy(0));
    assert_eq!(state.sort(), Some((0, SortDirection::Ascending)));

    // Sort by column 1 - should reset to ascending on new column
    let output = Table::<TestRow>::update(&mut state, TableMessage::SortBy(1));
    assert_eq!(
        output,
        Some(TableOutput::Sorted {
            column: 1,
            direction: SortDirection::Ascending,
        })
    );
}

// Disabled State Tests

#[test]
fn test_disabled() {
    let mut state = TableState::new(test_rows(), test_columns());
    state.set_disabled(true);

    assert_eq!(
        Table::<TestRow>::update(&mut state, TableMessage::Down),
        None
    );
    assert_eq!(Table::<TestRow>::update(&mut state, TableMessage::Up), None);
    assert_eq!(
        Table::<TestRow>::update(&mut state, TableMessage::Select),
        None
    );
    assert_eq!(
        Table::<TestRow>::update(&mut state, TableMessage::SortBy(0)),
        None
    );
}

// View Tests

#[test]
fn test_view_renders() {
    let state = TableState::new(test_rows(), test_columns());

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            Table::<TestRow>::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_with_header() {
    let state = TableState::new(test_rows(), test_columns());

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            Table::<TestRow>::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_with_sort_indicator() {
    let mut state = TableState::new(test_rows(), test_columns());
    Table::<TestRow>::update(&mut state, TableMessage::SortBy(0));

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            Table::<TestRow>::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_focused() {
    let mut state = TableState::new(test_rows(), test_columns());
    state.focused = true;

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            Table::<TestRow>::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_disabled() {
    let mut state = TableState::new(test_rows(), test_columns());
    state.disabled = true;

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            Table::<TestRow>::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_empty() {
    let state: TableState<TestRow> = TableState::new(vec![], test_columns());

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            Table::<TestRow>::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

// Integration Tests

#[test]
fn test_init() {
    let state: TableState<TestRow> = Table::<TestRow>::init();
    assert!(state.is_empty());
    assert!(!state.focused);
    assert!(!state.disabled);
}

#[test]
fn test_full_workflow() {
    let mut state = TableState::new(test_rows(), test_columns());
    Table::<TestRow>::set_focused(&mut state, true);

    // Navigate
    Table::<TestRow>::update(&mut state, TableMessage::Down);
    Table::<TestRow>::update(&mut state, TableMessage::Down);
    assert_eq!(state.selected_index(), Some(2));

    // Sort
    Table::<TestRow>::update(&mut state, TableMessage::SortBy(0));
    // Selection should follow the row, not the position

    // Navigate after sort
    Table::<TestRow>::update(&mut state, TableMessage::First);
    assert_eq!(state.selected_row().unwrap().name, "Alice");

    // Select
    let output = Table::<TestRow>::update(&mut state, TableMessage::Select);
    assert_eq!(
        output,
        Some(TableOutput::Selected(TestRow::new("Alice", "10")))
    );
}

#[test]
fn test_navigation_with_sort() {
    let mut state = TableState::new(test_rows(), test_columns());

    // Initially selected: Charlie (position 0 in original order)

    // Sort ascending by name
    Table::<TestRow>::update(&mut state, TableMessage::SortBy(0));

    // Now display order is: Alice, Bob, Charlie
    // But selection is preserved on the same ROW (Charlie), now at position 2
    assert_eq!(state.selected_row().unwrap().name, "Charlie");
    assert_eq!(state.selected_index(), Some(2));

    // Navigate to first to get to Alice
    Table::<TestRow>::update(&mut state, TableMessage::First);
    assert_eq!(state.selected_row().unwrap().name, "Alice");

    Table::<TestRow>::update(&mut state, TableMessage::Down);
    assert_eq!(state.selected_row().unwrap().name, "Bob");

    Table::<TestRow>::update(&mut state, TableMessage::Down);
    assert_eq!(state.selected_row().unwrap().name, "Charlie");
}

#[test]
fn test_sort_out_of_bounds_column() {
    let mut state = TableState::new(test_rows(), test_columns());
    let output = Table::<TestRow>::update(&mut state, TableMessage::SortBy(99));
    assert_eq!(output, None);
}

#[test]
fn test_page_navigation_bounds() {
    let mut state = TableState::new(test_rows(), test_columns());

    // PageDown beyond end
    let output = Table::<TestRow>::update(&mut state, TableMessage::PageDown(100));
    assert_eq!(output, Some(TableOutput::SelectionChanged(2)));

    // PageUp beyond start
    let output = Table::<TestRow>::update(&mut state, TableMessage::PageUp(100));
    assert_eq!(output, Some(TableOutput::SelectionChanged(0)));
}

#[test]
fn test_with_selected_empty() {
    let state: TableState<TestRow> = TableState::with_selected(vec![], test_columns(), 5);
    assert!(state.is_empty());
    assert_eq!(state.selected_index(), None);
}

#[test]
fn test_set_rows_to_empty() {
    let mut state = TableState::new(test_rows(), test_columns());
    assert_eq!(state.selected_index(), Some(0));

    state.set_rows(vec![]);
    assert!(state.is_empty());
    assert_eq!(state.selected_index(), None);
}

#[test]
fn test_set_rows_with_no_prior_selection() {
    let mut state = TableState::new(test_rows(), test_columns());
    state.set_selected(None);
    assert_eq!(state.selected_index(), None);

    state.set_rows(vec![TestRow::new("New", "1")]);
    // Should set selection to 0 when none was set
    assert_eq!(state.selected_index(), Some(0));
}

#[test]
fn test_set_selected_out_of_bounds() {
    let mut state = TableState::new(test_rows(), test_columns());
    // Try to set selection out of bounds
    state.set_selected(Some(100));
    // Should be ignored, selection unchanged
    assert_eq!(state.selected_index(), Some(0));
}

#[test]
fn test_view_descending_sort_indicator() {
    let mut state = TableState::new(test_rows(), test_columns());
    // Sort ascending first, then descending
    Table::<TestRow>::update(&mut state, TableMessage::SortBy(0));
    Table::<TestRow>::update(&mut state, TableMessage::SortBy(0));

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            Table::<TestRow>::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_clear_sort_preserves_selection() {
    let mut state = TableState::with_selected(test_rows(), test_columns(), 1);
    // Initially selected: Alice (index 1 in original order)

    // Sort ascending by name
    Table::<TestRow>::update(&mut state, TableMessage::SortBy(0));
    // Alice is now at display position 0

    // Clear sort
    Table::<TestRow>::update(&mut state, TableMessage::ClearSort);

    // Selection should still point to Alice (back at index 1)
    let selected = state.selected_row().unwrap();
    assert_eq!(selected.name, "Alice");
}

#[test]
fn test_view_unfocused() {
    let mut state = TableState::new(test_rows(), test_columns());
    state.focused = false;

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            Table::<TestRow>::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_page_up_at_first() {
    let mut state = TableState::new(test_rows(), test_columns());
    // Already at first, PageUp should return None
    let output = Table::<TestRow>::update(&mut state, TableMessage::PageUp(2));
    assert_eq!(output, None);
}

#[test]
fn test_page_down_at_last() {
    let mut state = TableState::with_selected(test_rows(), test_columns(), 2);
    // Already at last, PageDown should return None
    let output = Table::<TestRow>::update(&mut state, TableMessage::PageDown(2));
    assert_eq!(output, None);
}

// Row Mutation Edge Case Tests

#[test]
fn test_set_rows_preserves_valid_selection_after_navigation() {
    let mut state = TableState::new(test_rows(), test_columns());

    // Navigate to index 1
    Table::<TestRow>::update(&mut state, TableMessage::Down);
    assert_eq!(state.selected_index(), Some(1));

    // Replace with 5 rows - selection at 1 should be preserved
    let new_rows = vec![
        TestRow::new("X", "1"),
        TestRow::new("Y", "2"),
        TestRow::new("Z", "3"),
        TestRow::new("W", "4"),
        TestRow::new("V", "5"),
    ];
    state.set_rows(new_rows);
    assert_eq!(state.selected_index(), Some(1));
}

#[test]
fn test_set_rows_clamps_selection_after_navigation() {
    let rows: Vec<TestRow> = (0..10)
        .map(|i| TestRow::new(&format!("Row {}", i), &format!("{}", i)))
        .collect();
    let mut state = TableState::new(rows, test_columns());

    // Navigate to index 8
    for _ in 0..8 {
        Table::<TestRow>::update(&mut state, TableMessage::Down);
    }
    assert_eq!(state.selected_index(), Some(8));

    // Replace with only 3 rows - selection should clamp to last valid index
    let new_rows = vec![
        TestRow::new("A", "1"),
        TestRow::new("B", "2"),
        TestRow::new("C", "3"),
    ];
    state.set_rows(new_rows);
    assert_eq!(state.selected_index(), Some(2));
}

#[test]
fn test_set_rows_to_empty_clears_selection() {
    let mut state = TableState::new(test_rows(), test_columns());
    assert_eq!(state.selected_index(), Some(0));

    state.set_rows(vec![]);
    assert_eq!(state.selected_index(), None);
}

#[test]
fn test_sort_after_row_mutation() {
    let mut state = TableState::new(test_rows(), test_columns());

    // Sort by first column (ascending)
    Table::<TestRow>::update(&mut state, TableMessage::SortBy(0));
    assert!(state.sort().is_some());

    // Now mutate rows - set_rows resets sort
    let new_rows = vec![
        TestRow::new("Zebra", "1"),
        TestRow::new("Alpha", "2"),
    ];
    state.set_rows(new_rows);

    // Sort is cleared by set_rows
    assert!(state.sort().is_none());

    // Selection was at display index 2 (after sort), clamped to last valid (1)
    assert_eq!(state.selected_index(), Some(1));

    // Table should still work after mutation - navigate back to first
    Table::<TestRow>::update(&mut state, TableMessage::First);
    assert_eq!(state.selected_index(), Some(0));
    Table::<TestRow>::update(&mut state, TableMessage::Down);
    assert_eq!(state.selected_index(), Some(1));
}

#[test]
fn test_large_table_navigation() {
    let columns = vec![
        Column::new("ID", Constraint::Length(10)).sortable(),
        Column::new("Name", Constraint::Length(10)).sortable(),
    ];
    let rows: Vec<TestRow> = (0..1000)
        .map(|i| TestRow::new(&format!("{}", i), &format!("Row {}", i)))
        .collect();
    let mut state = TableState::new(rows, columns);

    // Navigate to middle
    for _ in 0..500 {
        Table::<TestRow>::update(&mut state, TableMessage::Down);
    }
    assert_eq!(state.selected_index(), Some(500));

    // First/Last
    Table::<TestRow>::update(&mut state, TableMessage::First);
    assert_eq!(state.selected_index(), Some(0));

    Table::<TestRow>::update(&mut state, TableMessage::Last);
    assert_eq!(state.selected_index(), Some(999));

    // PageUp/PageDown
    Table::<TestRow>::update(&mut state, TableMessage::PageUp(100));
    assert_eq!(state.selected_index(), Some(899));

    Table::<TestRow>::update(&mut state, TableMessage::PageDown(100));
    assert_eq!(state.selected_index(), Some(999));
}

#[test]
fn test_unicode_cell_content() {
    let columns = vec![
        Column::new("名前", Constraint::Length(15)),
        Column::new("説明", Constraint::Length(15)),
    ];
    let rows = vec![
        TestRow::new("田中太郎", "エンジニア"),
        TestRow::new("Москва", "город"),
    ];
    let mut state = TableState::new(rows, columns);

    Table::<TestRow>::update(&mut state, TableMessage::Down);
    assert_eq!(state.selected_index(), Some(1));
}
