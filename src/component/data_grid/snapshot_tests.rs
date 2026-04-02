use super::*;
use crate::component::test_utils;

#[derive(Clone, Debug, PartialEq)]
struct Person {
    name: String,
    age: String,
}

impl TableRow for Person {
    fn cells(&self) -> Vec<String> {
        vec![self.name.clone(), self.age.clone()]
    }
}

fn sample_rows() -> Vec<Person> {
    vec![
        Person {
            name: "Alice".into(),
            age: "30".into(),
        },
        Person {
            name: "Bob".into(),
            age: "25".into(),
        },
        Person {
            name: "Charlie".into(),
            age: "35".into(),
        },
    ]
}

fn sample_columns() -> Vec<Column> {
    vec![
        Column::new("Name", Constraint::Min(10)),
        Column::new("Age", Constraint::Min(5)),
    ]
}

// =============================================================================
// Snapshot tests
// =============================================================================

#[test]
fn test_snapshot_default_empty() {
    let state = DataGridState::<Person>::new(vec![], sample_columns());
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            DataGrid::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_populated() {
    let state = DataGridState::new(sample_rows(), sample_columns());
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            DataGrid::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_focused() {
    let mut state = DataGridState::new(sample_rows(), sample_columns());
    DataGrid::set_focused(&mut state, true);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            DataGrid::view(
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
fn test_snapshot_focused_second_column() {
    let mut state = DataGridState::new(sample_rows(), sample_columns());
    DataGrid::set_focused(&mut state, true);
    DataGrid::update(&mut state, DataGridMessage::Right);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            DataGrid::view(
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
fn test_snapshot_editing() {
    let mut state = DataGridState::new(sample_rows(), sample_columns());
    DataGrid::set_focused(&mut state, true);
    DataGrid::update(&mut state, DataGridMessage::Enter);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            DataGrid::view(
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
