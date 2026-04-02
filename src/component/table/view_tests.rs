use super::*;

// Test row type (shared with tests.rs)
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

#[test]
fn test_view_renders() {
    let state = TableState::new(test_rows(), test_columns());

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            Table::<TestRow>::view(&state, frame, frame.area(), &theme, &ViewContext::default());
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
            Table::<TestRow>::view(&state, frame, frame.area(), &theme, &ViewContext::default());
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
            Table::<TestRow>::view(&state, frame, frame.area(), &theme, &ViewContext::default());
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
            Table::<TestRow>::view(
                &state,
                frame,
                frame.area(),
                &theme,
                &ViewContext::new().focused(true),
            );
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
            Table::<TestRow>::view(
                &state,
                frame,
                frame.area(),
                &theme,
                &ViewContext::new().disabled(true),
            );
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
            Table::<TestRow>::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
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
            Table::<TestRow>::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_unfocused() {
    let mut state = TableState::new(test_rows(), test_columns());
    state.focused = false;

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            Table::<TestRow>::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_multi_column_sort_indicators() {
    let mut state = TableState::new(test_rows(), test_columns());
    // Sort by name ascending (primary), then add value ascending (secondary)
    Table::<TestRow>::update(&mut state, TableMessage::SortBy(0));
    Table::<TestRow>::update(&mut state, TableMessage::AddSort(1));

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            Table::<TestRow>::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}
