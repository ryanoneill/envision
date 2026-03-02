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

fn focused_grid() -> DataGridState<Person> {
    let mut state = DataGridState::new(sample_rows(), sample_columns());
    DataGrid::set_focused(&mut state, true);
    state
}

// =============================================================================
// Construction
// =============================================================================

#[test]
fn test_new() {
    let state = DataGridState::new(sample_rows(), sample_columns());
    assert_eq!(state.row_count(), 3);
    assert_eq!(state.column_count(), 2);
    assert_eq!(state.selected_row_index(), Some(0));
    assert_eq!(state.selected_column(), 0);
    assert!(!state.is_editing());
}

#[test]
fn test_new_empty() {
    let state = DataGridState::<Person>::new(vec![], sample_columns());
    assert_eq!(state.selected_row_index(), None);
    assert!(state.is_empty());
}

#[test]
fn test_default() {
    let state = DataGridState::<Person>::default();
    assert!(state.is_empty());
    assert_eq!(state.column_count(), 0);
    assert!(!state.is_focused());
    assert!(!state.is_disabled());
}

// =============================================================================
// Row and cell accessors
// =============================================================================

#[test]
fn test_selected_row() {
    let state = DataGridState::new(sample_rows(), sample_columns());
    let row = state.selected_row().unwrap();
    assert_eq!(row.name, "Alice");
}

#[test]
fn test_selected_item() {
    let state = DataGridState::new(sample_rows(), sample_columns());
    assert_eq!(state.selected_item(), state.selected_row());
}

#[test]
fn test_current_cell_value() {
    let state = DataGridState::new(sample_rows(), sample_columns());
    assert_eq!(state.current_cell_value(), Some("Alice".to_string()));
}

#[test]
fn test_current_cell_value_second_column() {
    let mut state = focused_grid();
    DataGrid::update(&mut state, DataGridMessage::Right);
    assert_eq!(state.current_cell_value(), Some("30".to_string()));
}

// =============================================================================
// Row navigation
// =============================================================================

#[test]
fn test_down() {
    let mut state = focused_grid();
    let output = DataGrid::update(&mut state, DataGridMessage::Down);
    assert_eq!(state.selected_row_index(), Some(1));
    assert_eq!(output, Some(DataGridOutput::SelectionChanged(1)));
}

#[test]
fn test_up() {
    let mut state = focused_grid();
    DataGrid::update(&mut state, DataGridMessage::Down);
    let output = DataGrid::update(&mut state, DataGridMessage::Up);
    assert_eq!(state.selected_row_index(), Some(0));
    assert_eq!(output, Some(DataGridOutput::SelectionChanged(0)));
}

#[test]
fn test_up_at_top() {
    let mut state = focused_grid();
    let output = DataGrid::update(&mut state, DataGridMessage::Up);
    assert_eq!(output, None);
}

#[test]
fn test_down_at_bottom() {
    let mut state = focused_grid();
    DataGrid::update(&mut state, DataGridMessage::Down);
    DataGrid::update(&mut state, DataGridMessage::Down);
    let output = DataGrid::update(&mut state, DataGridMessage::Down);
    assert_eq!(output, None);
}

#[test]
fn test_first() {
    let mut state = focused_grid();
    DataGrid::update(&mut state, DataGridMessage::Down);
    DataGrid::update(&mut state, DataGridMessage::Down);
    let output = DataGrid::update(&mut state, DataGridMessage::First);
    assert_eq!(state.selected_row_index(), Some(0));
    assert_eq!(output, Some(DataGridOutput::SelectionChanged(0)));
}

#[test]
fn test_last() {
    let mut state = focused_grid();
    let output = DataGrid::update(&mut state, DataGridMessage::Last);
    assert_eq!(state.selected_row_index(), Some(2));
    assert_eq!(output, Some(DataGridOutput::SelectionChanged(2)));
}

// =============================================================================
// Column navigation
// =============================================================================

#[test]
fn test_right() {
    let mut state = focused_grid();
    let output = DataGrid::update(&mut state, DataGridMessage::Right);
    assert_eq!(state.selected_column(), 1);
    assert_eq!(output, Some(DataGridOutput::ColumnChanged(1)));
}

#[test]
fn test_left() {
    let mut state = focused_grid();
    DataGrid::update(&mut state, DataGridMessage::Right);
    let output = DataGrid::update(&mut state, DataGridMessage::Left);
    assert_eq!(state.selected_column(), 0);
    assert_eq!(output, Some(DataGridOutput::ColumnChanged(0)));
}

#[test]
fn test_right_at_last_column() {
    let mut state = focused_grid();
    DataGrid::update(&mut state, DataGridMessage::Right);
    let output = DataGrid::update(&mut state, DataGridMessage::Right);
    assert_eq!(state.selected_column(), 1);
    assert_eq!(output, None);
}

#[test]
fn test_left_at_first_column() {
    let mut state = focused_grid();
    let output = DataGrid::update(&mut state, DataGridMessage::Left);
    assert_eq!(state.selected_column(), 0);
    assert_eq!(output, None);
}

// =============================================================================
// Editing
// =============================================================================

#[test]
fn test_enter_starts_editing() {
    let mut state = focused_grid();
    DataGrid::update(&mut state, DataGridMessage::Enter);
    assert!(state.is_editing());
    assert_eq!(state.editor_value(), "Alice");
}

#[test]
fn test_type_while_editing() {
    let mut state = focused_grid();
    DataGrid::update(&mut state, DataGridMessage::Enter);
    DataGrid::update(&mut state, DataGridMessage::Input('!'));
    assert!(state.editor_value().contains('!'));
}

#[test]
fn test_confirm_edit() {
    let mut state = focused_grid();
    DataGrid::update(&mut state, DataGridMessage::Enter);
    // Clear and type new value
    DataGrid::update(&mut state, DataGridMessage::Home);
    // Select all and type over... simpler to just add to end
    DataGrid::update(&mut state, DataGridMessage::End);
    DataGrid::update(&mut state, DataGridMessage::Input('!'));

    let output = DataGrid::update(&mut state, DataGridMessage::Enter);
    assert!(!state.is_editing());
    assert_eq!(
        output,
        Some(DataGridOutput::CellEdited {
            row: 0,
            column: 0,
            value: "Alice!".into(),
        })
    );
}

#[test]
fn test_cancel_edit() {
    let mut state = focused_grid();
    DataGrid::update(&mut state, DataGridMessage::Enter);
    DataGrid::update(&mut state, DataGridMessage::Input('!'));
    let output = DataGrid::update(&mut state, DataGridMessage::Cancel);
    assert!(!state.is_editing());
    assert_eq!(output, Some(DataGridOutput::EditCancelled));
}

#[test]
fn test_backspace_while_editing() {
    let mut state = focused_grid();
    DataGrid::update(&mut state, DataGridMessage::Enter);
    DataGrid::update(&mut state, DataGridMessage::Backspace);
    assert_eq!(state.editor_value(), "Alic");
}

#[test]
fn test_delete_while_editing() {
    let mut state = focused_grid();
    DataGrid::update(&mut state, DataGridMessage::Enter);
    DataGrid::update(&mut state, DataGridMessage::Home);
    DataGrid::update(&mut state, DataGridMessage::Delete);
    assert_eq!(state.editor_value(), "lice");
}

#[test]
fn test_edit_second_column() {
    let mut state = focused_grid();
    DataGrid::update(&mut state, DataGridMessage::Right);
    DataGrid::update(&mut state, DataGridMessage::Enter);
    assert!(state.is_editing());
    assert_eq!(state.editor_value(), "30");

    let output = DataGrid::update(&mut state, DataGridMessage::Enter);
    assert_eq!(
        output,
        Some(DataGridOutput::CellEdited {
            row: 0,
            column: 1,
            value: "30".into(),
        })
    );
}

#[test]
fn test_edit_different_row() {
    let mut state = focused_grid();
    DataGrid::update(&mut state, DataGridMessage::Down);
    DataGrid::update(&mut state, DataGridMessage::Enter);
    assert_eq!(state.editor_value(), "Bob");

    let output = DataGrid::update(&mut state, DataGridMessage::Enter);
    assert_eq!(
        output,
        Some(DataGridOutput::CellEdited {
            row: 1,
            column: 0,
            value: "Bob".into(),
        })
    );
}

// =============================================================================
// Disabled state
// =============================================================================

#[test]
fn test_disabled_ignores_messages() {
    let mut state = focused_grid();
    state.set_disabled(true);
    let output = DataGrid::update(&mut state, DataGridMessage::Down);
    assert_eq!(output, None);
}

#[test]
fn test_disabled_ignores_events() {
    let mut state = focused_grid();
    state.set_disabled(true);
    let msg = DataGrid::handle_event(&state, &Event::key(KeyCode::Down));
    assert_eq!(msg, None);
}

#[test]
fn test_with_disabled() {
    let state = DataGridState::new(sample_rows(), sample_columns()).with_disabled(true);
    assert!(state.is_disabled());
}

// =============================================================================
// Unfocused state
// =============================================================================

#[test]
fn test_unfocused_ignores_events() {
    let state = DataGridState::new(sample_rows(), sample_columns());
    let msg = DataGrid::handle_event(&state, &Event::key(KeyCode::Down));
    assert_eq!(msg, None);
}

// =============================================================================
// Event mapping — navigation mode
// =============================================================================

#[test]
fn test_up_key_maps() {
    let state = focused_grid();
    assert_eq!(
        DataGrid::handle_event(&state, &Event::key(KeyCode::Up)),
        Some(DataGridMessage::Up)
    );
    assert_eq!(
        DataGrid::handle_event(&state, &Event::char('k')),
        Some(DataGridMessage::Up)
    );
}

#[test]
fn test_down_key_maps() {
    let state = focused_grid();
    assert_eq!(
        DataGrid::handle_event(&state, &Event::key(KeyCode::Down)),
        Some(DataGridMessage::Down)
    );
    assert_eq!(
        DataGrid::handle_event(&state, &Event::char('j')),
        Some(DataGridMessage::Down)
    );
}

#[test]
fn test_left_right_key_maps() {
    let state = focused_grid();
    assert_eq!(
        DataGrid::handle_event(&state, &Event::key(KeyCode::Left)),
        Some(DataGridMessage::Left)
    );
    assert_eq!(
        DataGrid::handle_event(&state, &Event::key(KeyCode::Right)),
        Some(DataGridMessage::Right)
    );
    assert_eq!(
        DataGrid::handle_event(&state, &Event::char('h')),
        Some(DataGridMessage::Left)
    );
    assert_eq!(
        DataGrid::handle_event(&state, &Event::char('l')),
        Some(DataGridMessage::Right)
    );
}

#[test]
fn test_home_end_key_maps() {
    let state = focused_grid();
    assert_eq!(
        DataGrid::handle_event(&state, &Event::key(KeyCode::Home)),
        Some(DataGridMessage::First)
    );
    assert_eq!(
        DataGrid::handle_event(&state, &Event::key(KeyCode::End)),
        Some(DataGridMessage::Last)
    );
}

#[test]
fn test_enter_key_maps() {
    let state = focused_grid();
    assert_eq!(
        DataGrid::handle_event(&state, &Event::key(KeyCode::Enter)),
        Some(DataGridMessage::Enter)
    );
}

// =============================================================================
// Event mapping — editing mode
// =============================================================================

#[test]
fn test_editing_char_maps_to_input() {
    let mut state = focused_grid();
    DataGrid::update(&mut state, DataGridMessage::Enter);
    assert!(state.is_editing());

    assert_eq!(
        DataGrid::handle_event(&state, &Event::char('x')),
        Some(DataGridMessage::Input('x'))
    );
}

#[test]
fn test_editing_enter_maps() {
    let mut state = focused_grid();
    DataGrid::update(&mut state, DataGridMessage::Enter);

    assert_eq!(
        DataGrid::handle_event(&state, &Event::key(KeyCode::Enter)),
        Some(DataGridMessage::Enter)
    );
}

#[test]
fn test_editing_esc_maps_to_cancel() {
    let mut state = focused_grid();
    DataGrid::update(&mut state, DataGridMessage::Enter);

    assert_eq!(
        DataGrid::handle_event(&state, &Event::key(KeyCode::Esc)),
        Some(DataGridMessage::Cancel)
    );
}

#[test]
fn test_editing_backspace_maps() {
    let mut state = focused_grid();
    DataGrid::update(&mut state, DataGridMessage::Enter);

    assert_eq!(
        DataGrid::handle_event(&state, &Event::key(KeyCode::Backspace)),
        Some(DataGridMessage::Backspace)
    );
}

// =============================================================================
// set_rows
// =============================================================================

#[test]
fn test_set_rows() {
    let mut state = focused_grid();
    state.set_rows(vec![Person {
        name: "New".into(),
        age: "1".into(),
    }]);
    assert_eq!(state.row_count(), 1);
    assert_eq!(state.selected_row_index(), Some(0));
}

#[test]
fn test_set_rows_cancels_edit() {
    let mut state = focused_grid();
    DataGrid::update(&mut state, DataGridMessage::Enter);
    assert!(state.is_editing());

    state.set_rows(sample_rows());
    assert!(!state.is_editing());
}

#[test]
fn test_set_rows_clamps_selection() {
    let mut state = focused_grid();
    DataGrid::update(&mut state, DataGridMessage::Last);
    assert_eq!(state.selected_row_index(), Some(2));

    state.set_rows(vec![Person {
        name: "Only".into(),
        age: "1".into(),
    }]);
    assert_eq!(state.selected_row_index(), Some(0));
}

// =============================================================================
// Instance methods
// =============================================================================

#[test]
fn test_instance_handle_event() {
    let state = focused_grid();
    let msg = state.handle_event(&Event::key(KeyCode::Down));
    assert_eq!(msg, Some(DataGridMessage::Down));
}

#[test]
fn test_instance_update() {
    let mut state = focused_grid();
    let output = state.update(DataGridMessage::Down);
    assert_eq!(output, Some(DataGridOutput::SelectionChanged(1)));
}

#[test]
fn test_instance_dispatch_event() {
    let mut state = focused_grid();
    let output = state.dispatch_event(&Event::key(KeyCode::Down));
    assert_eq!(output, Some(DataGridOutput::SelectionChanged(1)));
}

// =============================================================================
// Rendering
// =============================================================================

#[test]
fn test_render_unfocused() {
    let state = DataGridState::new(sample_rows(), sample_columns());
    let (mut terminal, theme) = test_utils::setup_render(60, 15);
    terminal
        .draw(|frame| {
            DataGrid::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_focused() {
    let state = focused_grid();
    let (mut terminal, theme) = test_utils::setup_render(60, 15);
    terminal
        .draw(|frame| {
            DataGrid::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_editing() {
    let mut state = focused_grid();
    DataGrid::update(&mut state, DataGridMessage::Enter);
    let (mut terminal, theme) = test_utils::setup_render(60, 15);
    terminal
        .draw(|frame| {
            DataGrid::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_disabled() {
    let state = DataGridState::new(sample_rows(), sample_columns()).with_disabled(true);
    let (mut terminal, theme) = test_utils::setup_render(60, 15);
    terminal
        .draw(|frame| {
            DataGrid::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_empty() {
    let state = DataGridState::<Person>::new(vec![], sample_columns());
    let (mut terminal, theme) = test_utils::setup_render(60, 15);
    terminal
        .draw(|frame| {
            DataGrid::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

// =============================================================================
// Focusable trait
// =============================================================================

#[test]
fn test_focusable_trait() {
    let mut state = DataGrid::<Person>::init();
    assert!(!DataGrid::is_focused(&state));

    DataGrid::focus(&mut state);
    assert!(DataGrid::is_focused(&state));

    DataGrid::blur(&mut state);
    assert!(!DataGrid::is_focused(&state));
}

// =============================================================================
// PartialEq
// =============================================================================

#[test]
fn test_partial_eq() {
    let state1 = DataGridState::new(sample_rows(), sample_columns());
    let state2 = DataGridState::new(sample_rows(), sample_columns());
    assert_eq!(state1, state2);
}

// =============================================================================
// Edge cases
// =============================================================================

#[test]
fn test_empty_grid_ignores_navigation() {
    let mut state = DataGridState::<Person>::new(vec![], sample_columns());
    DataGrid::set_focused(&mut state, true);

    let output = DataGrid::update(&mut state, DataGridMessage::Down);
    assert_eq!(output, None);

    let output = DataGrid::update(&mut state, DataGridMessage::Enter);
    assert_eq!(output, None);
}

#[test]
fn test_navigation_does_not_change_edit_state() {
    let mut state = focused_grid();
    DataGrid::update(&mut state, DataGridMessage::Down);
    assert!(!state.is_editing());
}
