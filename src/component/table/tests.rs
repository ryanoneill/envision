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

#[test]
fn test_disabled_accessors() {
    let mut state = TableState::new(test_rows(), test_columns());
    assert!(!state.is_disabled());

    state.set_disabled(true);
    assert!(state.is_disabled());

    state.set_disabled(false);
    assert!(!state.is_disabled());
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

// Focus Tests

#[test]
fn test_focusable() {
    let mut state = TableState::new(test_rows(), test_columns());
    assert!(!Table::<TestRow>::is_focused(&state));

    Table::<TestRow>::set_focused(&mut state, true);
    assert!(Table::<TestRow>::is_focused(&state));

    Table::<TestRow>::blur(&mut state);
    assert!(!Table::<TestRow>::is_focused(&state));

    Table::<TestRow>::focus(&mut state);
    assert!(Table::<TestRow>::is_focused(&state));
}

// View Tests

#[test]
fn test_view_renders() {
    use crate::backend::CaptureBackend;
    use crate::theme::Theme;
    use ratatui::Terminal;

    let state = TableState::new(test_rows(), test_columns());

    let backend = CaptureBackend::new(40, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            Table::<TestRow>::view(&state, frame, frame.area(), &Theme::default());
        })
        .unwrap();

    let output = terminal.backend().to_string();
    assert!(output.contains("Name"));
    assert!(output.contains("Value"));
    assert!(output.contains("Charlie"));
    assert!(output.contains("Alice"));
    assert!(output.contains("Bob"));
}

#[test]
fn test_view_with_header() {
    use crate::backend::CaptureBackend;
    use crate::theme::Theme;
    use ratatui::Terminal;

    let state = TableState::new(test_rows(), test_columns());

    let backend = CaptureBackend::new(40, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            Table::<TestRow>::view(&state, frame, frame.area(), &Theme::default());
        })
        .unwrap();

    let output = terminal.backend().to_string();
    assert!(output.contains("Name"));
    assert!(output.contains("Value"));
}

#[test]
fn test_view_with_sort_indicator() {
    use crate::backend::CaptureBackend;
    use crate::theme::Theme;
    use ratatui::Terminal;

    let mut state = TableState::new(test_rows(), test_columns());
    Table::<TestRow>::update(&mut state, TableMessage::SortBy(0));

    let backend = CaptureBackend::new(40, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            Table::<TestRow>::view(&state, frame, frame.area(), &Theme::default());
        })
        .unwrap();

    let output = terminal.backend().to_string();
    assert!(output.contains("↑")); // Ascending indicator
}

#[test]
fn test_view_focused() {
    use crate::backend::CaptureBackend;
    use crate::theme::Theme;
    use ratatui::Terminal;

    let mut state = TableState::new(test_rows(), test_columns());
    state.focused = true;

    let backend = CaptureBackend::new(40, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            Table::<TestRow>::view(&state, frame, frame.area(), &Theme::default());
        })
        .unwrap();

    // Should render without panicking
    let _output = terminal.backend().to_string();
}

#[test]
fn test_view_disabled() {
    use crate::backend::CaptureBackend;
    use crate::theme::Theme;
    use ratatui::Terminal;

    let mut state = TableState::new(test_rows(), test_columns());
    state.disabled = true;

    let backend = CaptureBackend::new(40, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            Table::<TestRow>::view(&state, frame, frame.area(), &Theme::default());
        })
        .unwrap();

    // Should render without panicking
    let _output = terminal.backend().to_string();
}

#[test]
fn test_view_empty() {
    use crate::backend::CaptureBackend;
    use crate::theme::Theme;
    use ratatui::Terminal;

    let state: TableState<TestRow> = TableState::new(vec![], test_columns());

    let backend = CaptureBackend::new(40, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            Table::<TestRow>::view(&state, frame, frame.area(), &Theme::default());
        })
        .unwrap();

    // Should render without panicking
    let output = terminal.backend().to_string();
    assert!(output.contains("Name")); // Headers still shown
}

// Integration Tests

#[test]
fn test_clone() {
    let mut state = TableState::with_selected(test_rows(), test_columns(), 1);
    state.focused = true;
    Table::<TestRow>::update(&mut state, TableMessage::SortBy(0));

    let cloned = state.clone();
    assert_eq!(cloned.selected_index(), Some(0)); // Alice is now at position 0 after sort
    assert!(cloned.focused);
    assert!(cloned.sort().is_some());
}

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
    use crate::backend::CaptureBackend;
    use crate::theme::Theme;
    use ratatui::Terminal;

    let mut state = TableState::new(test_rows(), test_columns());
    // Sort ascending first, then descending
    Table::<TestRow>::update(&mut state, TableMessage::SortBy(0));
    Table::<TestRow>::update(&mut state, TableMessage::SortBy(0));

    let backend = CaptureBackend::new(40, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            Table::<TestRow>::view(&state, frame, frame.area(), &Theme::default());
        })
        .unwrap();

    let output = terminal.backend().to_string();
    assert!(output.contains("↓")); // Descending indicator
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
    use crate::backend::CaptureBackend;
    use crate::theme::Theme;
    use ratatui::Terminal;

    let mut state = TableState::new(test_rows(), test_columns());
    state.focused = false;

    let backend = CaptureBackend::new(40, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            Table::<TestRow>::view(&state, frame, frame.area(), &Theme::default());
        })
        .unwrap();

    let output = terminal.backend().to_string();
    assert!(output.contains("Charlie"));
}

#[test]
fn test_table_message_debug() {
    let msg = TableMessage::SortBy(0);
    let debug = format!("{:?}", msg);
    assert!(debug.contains("SortBy"));
}

#[test]
fn test_table_message_eq() {
    assert_eq!(TableMessage::Up, TableMessage::Up);
    assert_eq!(TableMessage::Down, TableMessage::Down);
    assert_eq!(TableMessage::First, TableMessage::First);
    assert_eq!(TableMessage::Last, TableMessage::Last);
    assert_eq!(TableMessage::PageUp(5), TableMessage::PageUp(5));
    assert_eq!(TableMessage::PageDown(10), TableMessage::PageDown(10));
    assert_eq!(TableMessage::Select, TableMessage::Select);
    assert_eq!(TableMessage::SortBy(0), TableMessage::SortBy(0));
    assert_eq!(TableMessage::ClearSort, TableMessage::ClearSort);
}

#[test]
fn test_table_output_debug() {
    let out: TableOutput<TestRow> = TableOutput::SelectionChanged(1);
    let debug = format!("{:?}", out);
    assert!(debug.contains("SelectionChanged"));
}

#[test]
fn test_table_output_eq() {
    let out1: TableOutput<TestRow> = TableOutput::SelectionChanged(1);
    let out2: TableOutput<TestRow> = TableOutput::SelectionChanged(1);
    assert_eq!(out1, out2);

    let out3: TableOutput<TestRow> = TableOutput::SortCleared;
    let out4: TableOutput<TestRow> = TableOutput::SortCleared;
    assert_eq!(out3, out4);
}

#[test]
fn test_column_debug() {
    let col = Column::new("Header", Constraint::Length(10));
    let debug = format!("{:?}", col);
    assert!(debug.contains("Column"));
}

#[test]
fn test_state_debug() {
    let state = TableState::new(test_rows(), test_columns());
    let debug = format!("{:?}", state);
    assert!(debug.contains("TableState"));
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
