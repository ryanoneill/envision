#![cfg(feature = "full")]
//! Stress tests exercising components with large datasets (10,000+ items).
use envision::ViewContext;

use envision::component::{DataGrid, DataGridMessage, DataGridState};
use envision::{
    Accordion, AccordionMessage, AccordionPanel, AccordionState, CaptureBackend, Column, Component,
    LoadingList, LoadingListMessage, LoadingListState, SelectableList, SelectableListMessage,
    SelectableListState, Table, TableMessage, TableRow, TableState, Theme, Tree, TreeMessage,
    TreeNode, TreeState,
};
use ratatui::prelude::*;
use ratatui::Terminal;

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

#[derive(Clone, PartialEq, Debug)]
struct StressRow {
    id: u32,
    name: String,
    value: String,
}

impl TableRow for StressRow {
    fn cells(&self) -> Vec<String> {
        vec![self.id.to_string(), self.name.clone(), self.value.clone()]
    }
}

fn make_stress_rows(count: u32) -> Vec<StressRow> {
    (0..count)
        .map(|i| StressRow {
            id: i,
            name: format!("Item {}", i),
            value: format!("val-{}", i % 100),
        })
        .collect()
}

fn stress_columns() -> Vec<Column> {
    vec![
        Column::new("ID", Constraint::Length(8)).sortable(),
        Column::new("Name", Constraint::Length(20)).sortable(),
        Column::new("Value", Constraint::Length(12)).sortable(),
    ]
}

/// Renders a view function into a terminal of the given size without panicking.
fn assert_renders_ok<F>(name: &str, width: u16, height: u16, render_fn: F)
where
    F: FnOnce(&mut Frame, Rect, &Theme),
{
    let backend = CaptureBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    let theme = Theme::default();
    terminal
        .draw(|frame| {
            render_fn(frame, frame.area(), &theme);
        })
        .unwrap_or_else(|e| panic!("{} panicked during render: {}", name, e));
}

// ---------------------------------------------------------------------------
// Table stress test: 10,000 rows
// ---------------------------------------------------------------------------

#[test]
fn test_table_stress_10000_rows() {
    let rows = make_stress_rows(10_000);
    let columns = stress_columns();
    let mut state = TableState::new(rows, columns);
    state.set_focused(true);

    assert_eq!(state.selected_index(), Some(0));

    // Navigate down 100 times
    for _ in 0..100 {
        Table::<StressRow>::update(&mut state, TableMessage::Down);
    }
    assert_eq!(state.selected_index(), Some(100));

    // Jump to last
    Table::<StressRow>::update(&mut state, TableMessage::Last);
    assert_eq!(state.selected_index(), Some(9999));
    assert_eq!(state.selected_row().unwrap().name, "Item 9999");

    // Jump to first
    Table::<StressRow>::update(&mut state, TableMessage::First);
    assert_eq!(state.selected_index(), Some(0));
    assert_eq!(state.selected_row().unwrap().name, "Item 0");

    // Sort by column 0 (ascending)
    let output = Table::<StressRow>::update(&mut state, TableMessage::SortBy(0));
    assert!(output.is_some());

    // Render to verify no panics with large dataset
    assert_renders_ok("Table-10k", 80, 24, |frame, area, theme| {
        Table::<StressRow>::view(&state, frame, area, theme, &ViewContext::default());
    });
}

// ---------------------------------------------------------------------------
// Tree stress test: 11,100 nodes (100 roots × 10 children × 10 grandchildren)
// ---------------------------------------------------------------------------

#[test]
fn test_tree_stress_10000_nodes() {
    let roots: Vec<TreeNode<String>> = (0..100)
        .map(|r| {
            let mut root = TreeNode::new(format!("Root {}", r), format!("root-{}", r));
            for c in 0..10 {
                let mut child =
                    TreeNode::new(format!("Child {}-{}", r, c), format!("child-{}-{}", r, c));
                for g in 0..10 {
                    child.add_child(TreeNode::new(
                        format!("Grand {}-{}-{}", r, c, g),
                        format!("grand-{}-{}-{}", r, c, g),
                    ));
                }
                root.add_child(child);
            }
            root
        })
        .collect();

    let mut state = TreeState::new(roots);
    state.set_focused(true);

    assert_eq!(state.selected_index(), Some(0));

    // Expand all nodes
    Tree::<String>::update(&mut state, TreeMessage::ExpandAll);
    let visible = state.visible_count();
    assert_eq!(visible, 11_100); // 100 + 1000 + 10000

    // Navigate down 50 times
    for _ in 0..50 {
        Tree::<String>::update(&mut state, TreeMessage::Down);
    }
    assert_eq!(state.selected_index(), Some(50));

    // Collapse all
    Tree::<String>::update(&mut state, TreeMessage::CollapseAll);
    assert_eq!(state.visible_count(), 100); // only roots visible

    // Navigate back to the top and expand first root
    for _ in 0..100 {
        Tree::<String>::update(&mut state, TreeMessage::Up);
    }
    assert_eq!(state.selected_index(), Some(0));
    Tree::<String>::update(&mut state, TreeMessage::Expand);
    Tree::<String>::update(&mut state, TreeMessage::Down);
    assert_eq!(state.selected_index(), Some(1)); // first child of root 0

    // Render to verify no panics with large tree
    assert_renders_ok("Tree-11k", 100, 40, |frame, area, theme| {
        Tree::<String>::view(&state, frame, area, theme, &ViewContext::default());
    });
}

// ---------------------------------------------------------------------------
// LoadingList stress test: 10,000 items with state changes
// ---------------------------------------------------------------------------

#[test]
fn test_loading_list_stress_10000_items() {
    let items: Vec<String> = (0..10_000).map(|i| format!("Task {}", i)).collect();
    let mut state = LoadingListState::with_items(items, |s| s.clone());
    state.set_focused(true);

    // LoadingList starts with no selection; select the first item
    LoadingList::<String>::update(&mut state, LoadingListMessage::First);
    assert_eq!(state.selected_index(), Some(0));

    // Navigate down 500 times
    for _ in 0..500 {
        LoadingList::<String>::update(&mut state, LoadingListMessage::Down);
    }
    assert_eq!(state.selected_index(), Some(500));

    // Set loading on 100 items
    for i in 0..100 {
        LoadingList::<String>::update(&mut state, LoadingListMessage::SetLoading(i * 100));
    }

    // Set ready on first 50
    for i in 0..50 {
        LoadingList::<String>::update(&mut state, LoadingListMessage::SetReady(i * 100));
    }

    // Set error on next 50
    for i in 50..100 {
        LoadingList::<String>::update(
            &mut state,
            LoadingListMessage::SetError {
                index: i * 100,
                message: format!("Error at {}", i * 100),
            },
        );
    }

    // Navigate to last
    LoadingList::<String>::update(&mut state, LoadingListMessage::Last);
    assert_eq!(state.selected_index(), Some(9999));

    // Navigate to first
    LoadingList::<String>::update(&mut state, LoadingListMessage::First);
    assert_eq!(state.selected_index(), Some(0));

    // Render to verify no panics
    assert_renders_ok("LoadingList-10k", 80, 30, |frame, area, theme| {
        LoadingList::<String>::view(&state, frame, area, theme, &ViewContext::default());
    });
}

// ---------------------------------------------------------------------------
// SelectableList stress test: 50,000 items with large page operations
// ---------------------------------------------------------------------------

#[test]
fn test_selectable_list_stress_50000_items() {
    let items: Vec<String> = (0..50_000).map(|i| format!("Item {}", i)).collect();
    let mut state = SelectableListState::new(items);
    state.set_focused(true);

    assert_eq!(state.selected_index(), Some(0));

    // PageDown by 10,000
    SelectableList::<String>::update(&mut state, SelectableListMessage::PageDown(10_000));
    assert_eq!(state.selected_index(), Some(10_000));

    // PageDown by another 10,000
    SelectableList::<String>::update(&mut state, SelectableListMessage::PageDown(10_000));
    assert_eq!(state.selected_index(), Some(20_000));

    // PageDown by 50,000 (should clamp to last)
    SelectableList::<String>::update(&mut state, SelectableListMessage::PageDown(50_000));
    assert_eq!(state.selected_index(), Some(49_999));

    // PageUp by 50,000 (should clamp to first)
    SelectableList::<String>::update(&mut state, SelectableListMessage::PageUp(50_000));
    assert_eq!(state.selected_index(), Some(0));

    // Last and First
    SelectableList::<String>::update(&mut state, SelectableListMessage::Last);
    assert_eq!(state.selected_index(), Some(49_999));
    assert_eq!(state.selected_item(), Some(&"Item 49999".to_string()));

    SelectableList::<String>::update(&mut state, SelectableListMessage::First);
    assert_eq!(state.selected_index(), Some(0));
    assert_eq!(state.selected_item(), Some(&"Item 0".to_string()));

    // Render to verify no panics with massive list
    assert_renders_ok("SelectableList-50k", 80, 24, |frame, area, theme| {
        SelectableList::<String>::view(&state, frame, area, theme, &ViewContext::default());
    });
}

// ---------------------------------------------------------------------------
// Accordion stress test: 1,000 panels
// ---------------------------------------------------------------------------

#[test]
fn test_accordion_stress_1000_panels() {
    let panels: Vec<AccordionPanel> = (0..1_000)
        .map(|i| AccordionPanel::new(format!("Panel {}", i), format!("Content for panel {}", i)))
        .collect();
    let mut state = AccordionState::new(panels);
    state.set_focused(true);

    assert_eq!(state.selected_index(), Some(0));

    // Navigate down 500 times
    for _ in 0..500 {
        Accordion::update(&mut state, AccordionMessage::Down);
    }
    assert_eq!(state.selected_index(), Some(500));

    // Expand all, then collapse all
    Accordion::update(&mut state, AccordionMessage::ExpandAll);
    assert!(state.is_all_expanded());

    Accordion::update(&mut state, AccordionMessage::CollapseAll);
    assert!(!state.is_any_expanded());

    // Navigate to first and last
    Accordion::update(&mut state, AccordionMessage::First);
    assert_eq!(state.selected_index(), Some(0));

    Accordion::update(&mut state, AccordionMessage::Last);
    assert_eq!(state.selected_index(), Some(999));

    // Render to verify no panics
    assert_renders_ok("Accordion-1k", 80, 40, |frame, area, theme| {
        Accordion::view(&state, frame, area, theme, &ViewContext::default());
    });
}

// ---------------------------------------------------------------------------
// DataGrid stress test: 10,000 rows with navigation and editing
// ---------------------------------------------------------------------------

#[test]
fn test_data_grid_stress_10000_rows() {
    let rows = make_stress_rows(10_000);
    let columns = stress_columns();
    let mut state = DataGridState::new(rows, columns);
    state.set_focused(true);

    assert_eq!(state.selected_index(), Some(0));

    // Navigate down 200 rows
    for _ in 0..200 {
        DataGrid::<StressRow>::update(&mut state, DataGridMessage::Down);
    }
    assert_eq!(state.selected_index(), Some(200));

    // Navigate right to column 2
    DataGrid::<StressRow>::update(&mut state, DataGridMessage::Right);
    DataGrid::<StressRow>::update(&mut state, DataGridMessage::Right);
    assert_eq!(state.selected_column(), 2);

    // Enter edit mode, type a character, confirm edit
    DataGrid::<StressRow>::update(&mut state, DataGridMessage::Enter);
    assert!(state.is_editing());
    DataGrid::<StressRow>::update(&mut state, DataGridMessage::Input('X'));
    DataGrid::<StressRow>::update(&mut state, DataGridMessage::Enter);
    assert!(!state.is_editing());

    // Navigate to last row
    DataGrid::<StressRow>::update(&mut state, DataGridMessage::Last);
    assert_eq!(state.selected_index(), Some(9999));

    // Render to verify no panics
    assert_renders_ok("DataGrid-10k", 120, 40, |frame, area, theme| {
        DataGrid::<StressRow>::view(&state, frame, area, theme, &ViewContext::default());
    });
}

// ---------------------------------------------------------------------------
// Rapid input stress test: 10,000 events on a SelectableList
// ---------------------------------------------------------------------------

#[test]
fn test_rapid_input_10000_events() {
    let items: Vec<String> = (0..100).map(|i| format!("Item {}", i)).collect();
    let mut state = SelectableListState::new(items);
    state.set_focused(true);

    // Send 10,000 alternating Down/Up events via dispatch_event
    let down = envision::Event::key(crossterm::event::KeyCode::Down);
    let up = envision::Event::key(crossterm::event::KeyCode::Up);

    for i in 0..10_000 {
        if i % 2 == 0 {
            state.dispatch_event(&down);
        } else {
            state.dispatch_event(&up);
        }
    }

    // After 10,000 alternating events (starting at 0):
    // even iterations go down, odd go back up → net movement is 0
    // index should be at 0
    assert_eq!(state.selected_index(), Some(0));

    // Now rapid all-down to verify we hit the boundary correctly
    for _ in 0..10_000 {
        state.dispatch_event(&down);
    }
    // Clamped to last item (index 99)
    assert_eq!(state.selected_index(), Some(99));
}
