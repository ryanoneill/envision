use super::*;

#[derive(Clone, Debug, PartialEq)]
struct TestRow {
    name: String,
    category: String,
}

impl TableRow for TestRow {
    fn cells(&self) -> Vec<String> {
        vec![self.name.clone(), self.category.clone()]
    }
}

fn test_columns() -> Vec<Column> {
    vec![
        Column::new("Name", Constraint::Length(15)).sortable(),
        Column::new("Category", Constraint::Length(15)),
    ]
}

fn test_rows() -> Vec<TestRow> {
    vec![
        TestRow { name: "Apple".into(), category: "Fruit".into() },
        TestRow { name: "Banana".into(), category: "Fruit".into() },
        TestRow { name: "Carrot".into(), category: "Vegetable".into() },
        TestRow { name: "Apricot".into(), category: "Fruit".into() },
    ]
}

#[test]
fn test_filter_text_default() {
    let state = TableState::new(test_rows(), test_columns());
    assert_eq!(state.filter_text(), "");
    assert_eq!(state.visible_count(), 4);
}

#[test]
fn test_set_filter_text() {
    let mut state = TableState::new(test_rows(), test_columns());
    state.set_filter_text("ap");
    assert_eq!(state.filter_text(), "ap");
    assert_eq!(state.visible_count(), 2); // Apple, Apricot
}

#[test]
fn test_filter_matches_any_cell() {
    let mut state = TableState::new(test_rows(), test_columns());
    state.set_filter_text("vegetable");
    assert_eq!(state.visible_count(), 1); // Carrot (category = Vegetable)
    assert_eq!(state.selected_row().unwrap().name, "Carrot");
}

#[test]
fn test_filter_case_insensitive() {
    let mut state = TableState::new(test_rows(), test_columns());
    state.set_filter_text("APPLE");
    assert_eq!(state.visible_count(), 1);
    assert_eq!(state.selected_row().unwrap().name, "Apple");
}

#[test]
fn test_filter_no_matches() {
    let mut state = TableState::new(test_rows(), test_columns());
    state.set_filter_text("xyz");
    assert_eq!(state.visible_count(), 0);
    assert_eq!(state.selected_row(), None);
}

#[test]
fn test_clear_filter() {
    let mut state = TableState::new(test_rows(), test_columns());
    state.set_filter_text("ap");
    assert_eq!(state.visible_count(), 2);

    state.clear_filter();
    assert_eq!(state.filter_text(), "");
    assert_eq!(state.visible_count(), 4);
}

#[test]
fn test_filter_preserves_selection() {
    let mut state = TableState::new(test_rows(), test_columns());
    // Select Apricot (display index 3)
    state.set_selected(Some(3));
    assert_eq!(state.selected_row().unwrap().name, "Apricot");

    // Filter to "ap" — Apple(0), Apricot(3)
    state.set_filter_text("ap");
    assert_eq!(state.visible_count(), 2);
    assert_eq!(state.selected_row().unwrap().name, "Apricot");
}

#[test]
fn test_filter_resets_selection_when_row_hidden() {
    let mut state = TableState::new(test_rows(), test_columns());
    // Select Carrot (display index 2)
    state.set_selected(Some(2));
    assert_eq!(state.selected_row().unwrap().name, "Carrot");

    // Filter to "fruit" — Carrot is "Vegetable", gets filtered out
    state.set_filter_text("fruit");
    assert_eq!(state.visible_count(), 3); // Apple, Banana, Apricot
    // Selection moves to first visible
    assert_eq!(state.selected_row().unwrap().name, "Apple");
}

#[test]
fn test_filter_navigation() {
    let mut state = TableState::new(test_rows(), test_columns());
    state.focused = true;
    state.set_filter_text("ap");
    // Filtered: Apple(0), Apricot(3)
    assert_eq!(state.visible_count(), 2);
    assert_eq!(state.selected_row().unwrap().name, "Apple");

    let output = Table::<TestRow>::update(&mut state, TableMessage::Down);
    assert_eq!(state.selected_row().unwrap().name, "Apricot");
    assert_eq!(output, Some(TableOutput::SelectionChanged(1)));

    // At end, stay
    let output = Table::<TestRow>::update(&mut state, TableMessage::Down);
    assert_eq!(output, None);
}

#[test]
fn test_filter_select_returns_original_row() {
    let mut state = TableState::new(test_rows(), test_columns());
    state.set_filter_text("ap");
    Table::<TestRow>::update(&mut state, TableMessage::Down);
    let output = Table::<TestRow>::update(&mut state, TableMessage::Select);
    assert_eq!(
        output,
        Some(TableOutput::Selected(TestRow {
            name: "Apricot".into(),
            category: "Fruit".into(),
        }))
    );
}

#[test]
fn test_filter_with_sort() {
    let mut state = TableState::new(test_rows(), test_columns());
    // Sort by name ascending
    Table::<TestRow>::update(&mut state, TableMessage::SortBy(0));

    // Filter to "ap" — should show Apple and Apricot, sorted
    state.set_filter_text("ap");
    assert_eq!(state.visible_count(), 2);
    assert_eq!(state.selected_row().unwrap().name, "Apple");

    Table::<TestRow>::update(&mut state, TableMessage::Down);
    assert_eq!(state.selected_row().unwrap().name, "Apricot");
}

#[test]
fn test_filter_message_set_filter() {
    let mut state = TableState::new(test_rows(), test_columns());
    let output = Table::<TestRow>::update(&mut state, TableMessage::SetFilter("ap".into()));
    assert_eq!(state.filter_text(), "ap");
    assert_eq!(state.visible_count(), 2);
    assert_eq!(output, Some(TableOutput::FilterChanged("ap".into())));
}

#[test]
fn test_filter_message_clear_filter() {
    let mut state = TableState::new(test_rows(), test_columns());
    state.set_filter_text("ap");

    let output = Table::<TestRow>::update(&mut state, TableMessage::ClearFilter);
    assert_eq!(state.filter_text(), "");
    assert_eq!(state.visible_count(), 4);
    assert_eq!(output, Some(TableOutput::FilterChanged(String::new())));
}

#[test]
fn test_set_rows_clears_filter() {
    let mut state = TableState::new(test_rows(), test_columns());
    state.set_filter_text("ap");
    assert_eq!(state.visible_count(), 2);

    state.set_rows(vec![
        TestRow { name: "X".into(), category: "Y".into() },
    ]);
    assert_eq!(state.filter_text(), "");
    assert_eq!(state.visible_count(), 1);
}

#[test]
fn test_filter_empty_string_shows_all() {
    let mut state = TableState::new(test_rows(), test_columns());
    state.set_filter_text("");
    assert_eq!(state.visible_count(), 4);
}
