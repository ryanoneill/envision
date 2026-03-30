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

// ========== Column Resizing Tests ==========

#[test]
fn test_increase_column_width() {
    let mut state = TableState::new(test_rows(), test_columns());
    let output = Table::<TestRow>::update(&mut state, TableMessage::IncreaseColumnWidth(0));
    assert_eq!(
        output,
        Some(TableOutput::ColumnResized {
            column: 0,
            width: 11,
        })
    );
    assert_eq!(state.columns()[0].width(), Constraint::Length(11));
}

#[test]
fn test_decrease_column_width() {
    let mut state = TableState::new(test_rows(), test_columns());
    let output = Table::<TestRow>::update(&mut state, TableMessage::DecreaseColumnWidth(0));
    assert_eq!(
        output,
        Some(TableOutput::ColumnResized {
            column: 0,
            width: 9,
        })
    );
    assert_eq!(state.columns()[0].width(), Constraint::Length(9));
}

#[test]
fn test_decrease_column_width_minimum() {
    let columns = vec![
        Column::new("Name", Constraint::Length(3)).sortable(),
        Column::new("Value", Constraint::Length(10)).sortable(),
    ];
    let mut state = TableState::new(test_rows(), columns);

    // Already at minimum width of 3
    let output = Table::<TestRow>::update(&mut state, TableMessage::DecreaseColumnWidth(0));
    assert_eq!(output, None);
    assert_eq!(state.columns()[0].width(), Constraint::Length(3));
}

#[test]
fn test_decrease_column_width_to_minimum() {
    let columns = vec![
        Column::new("Name", Constraint::Length(4)).sortable(),
        Column::new("Value", Constraint::Length(10)).sortable(),
    ];
    let mut state = TableState::new(test_rows(), columns);

    let output = Table::<TestRow>::update(&mut state, TableMessage::DecreaseColumnWidth(0));
    assert_eq!(
        output,
        Some(TableOutput::ColumnResized {
            column: 0,
            width: 3,
        })
    );
    assert_eq!(state.columns()[0].width(), Constraint::Length(3));
}

#[test]
fn test_resize_non_length_constraint() {
    let columns = vec![
        Column::new("Name", Constraint::Percentage(50)),
        Column::new("Value", Constraint::Length(10)),
    ];
    let mut state = TableState::new(test_rows(), columns);

    // Resize should not work on Percentage constraints
    let output = Table::<TestRow>::update(&mut state, TableMessage::IncreaseColumnWidth(0));
    assert_eq!(output, None);

    let output = Table::<TestRow>::update(&mut state, TableMessage::DecreaseColumnWidth(0));
    assert_eq!(output, None);
}

#[test]
fn test_resize_out_of_bounds_column() {
    let mut state = TableState::new(test_rows(), test_columns());
    let output = Table::<TestRow>::update(&mut state, TableMessage::IncreaseColumnWidth(99));
    assert_eq!(output, None);

    let output = Table::<TestRow>::update(&mut state, TableMessage::DecreaseColumnWidth(99));
    assert_eq!(output, None);
}

#[test]
fn test_resize_disabled() {
    let mut state = TableState::new(test_rows(), test_columns());
    state.set_disabled(true);
    let output = Table::<TestRow>::update(&mut state, TableMessage::IncreaseColumnWidth(0));
    assert_eq!(output, None);
}

// ========== Key Binding Tests for New Features ==========

mod key_binding_tests {
    use super::*;
    use crate::input::Event;

    #[test]
    fn test_plus_key_increases_column_width() {
        let mut state = TableState::new(test_rows(), test_columns());
        state.set_focused(true);

        let msg = Table::<TestRow>::handle_event(&state, &Event::char('+'));
        assert_eq!(msg, Some(TableMessage::IncreaseColumnWidth(0)));
    }

    #[test]
    fn test_minus_key_decreases_column_width() {
        let mut state = TableState::new(test_rows(), test_columns());
        state.set_focused(true);

        let msg = Table::<TestRow>::handle_event(&state, &Event::char('-'));
        assert_eq!(msg, Some(TableMessage::DecreaseColumnWidth(0)));
    }

    #[test]
    fn test_plus_key_targets_sort_column() {
        let mut state = TableState::new(test_rows(), test_columns());
        state.set_focused(true);

        // Sort by column 1
        Table::<TestRow>::update(&mut state, TableMessage::SortBy(1));

        let msg = Table::<TestRow>::handle_event(&state, &Event::char('+'));
        assert_eq!(msg, Some(TableMessage::IncreaseColumnWidth(1)));
    }

    #[test]
    fn test_resize_keys_ignored_when_unfocused() {
        let state = TableState::new(test_rows(), test_columns());

        let msg = Table::<TestRow>::handle_event(&state, &Event::char('+'));
        assert_eq!(msg, None);

        let msg = Table::<TestRow>::handle_event(&state, &Event::char('-'));
        assert_eq!(msg, None);
    }

    #[test]
    fn test_resize_keys_ignored_when_disabled() {
        let mut state = TableState::new(test_rows(), test_columns());
        state.set_focused(true);
        state.set_disabled(true);

        let msg = Table::<TestRow>::handle_event(&state, &Event::char('+'));
        assert_eq!(msg, None);
    }
}
