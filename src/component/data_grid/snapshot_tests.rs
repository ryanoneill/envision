use super::*;
use crate::component::test_utils;

#[derive(Clone, Debug, PartialEq)]
struct Person {
    name: String,
    age: String,
}

impl TableRow for Person {
    fn cells(&self) -> Vec<crate::component::cell::Cell> {
        use crate::component::cell::Cell;
        vec![Cell::new(&self.name), Cell::new(&self.age)]
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
            DataGrid::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
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
            DataGrid::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_focused() {
    let state = DataGridState::new(sample_rows(), sample_columns());
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            DataGrid::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_focused_second_column() {
    let mut state = DataGridState::new(sample_rows(), sample_columns());
    DataGrid::update(&mut state, DataGridMessage::Right);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            DataGrid::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_editing() {
    let mut state = DataGridState::new(sample_rows(), sample_columns());
    DataGrid::update(&mut state, DataGridMessage::Enter);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            DataGrid::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn view_chrome_owned_no_outer_border() {
    let state = DataGridState::new(sample_rows(), sample_columns());
    let (mut terminal, theme) = test_utils::setup_render(40, 8);
    terminal
        .draw(|frame| {
            DataGrid::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).chrome_owned(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}
