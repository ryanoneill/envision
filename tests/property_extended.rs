#![cfg(feature = "full")]
//! Extended property-based tests for envision components using proptest.
//!
//! Covers 10 additional components beyond the original property.rs:
//! Accordion, Breadcrumb, Menu, Table, Tree, TextArea, LineInput,
//! Dropdown, ScrollableText, and LoadingList.
//!
//! Each section defines a strategy generator for the component's Message type,
//! then tests invariants that must hold for arbitrary message sequences.

use envision::{
    Accordion, AccordionMessage, AccordionPanel, AccordionState, Breadcrumb, BreadcrumbMessage,
    BreadcrumbState, Column, Component, Dropdown, DropdownMessage, DropdownState, LineInput,
    LineInputMessage, LineInputState, LoadingList, LoadingListMessage, LoadingListState, Menu,
    MenuItem, MenuMessage, MenuState, ScrollableText, ScrollableTextMessage, ScrollableTextState,
    Table, TableMessage, TableRow, TableState, TextArea, TextAreaMessage, TextAreaState, Tree,
    TreeMessage, TreeNode, TreeState,
};
use proptest::prelude::*;

// ========================================
// Strategy Helpers
// ========================================

/// Generates an AccordionMessage.
fn accordion_message_strategy(max_index: usize) -> impl Strategy<Value = AccordionMessage> {
    prop_oneof![
        Just(AccordionMessage::Down),
        Just(AccordionMessage::Up),
        Just(AccordionMessage::First),
        Just(AccordionMessage::Last),
        Just(AccordionMessage::Toggle),
        Just(AccordionMessage::Expand),
        Just(AccordionMessage::Collapse),
        (0..=max_index).prop_map(AccordionMessage::ToggleIndex),
        Just(AccordionMessage::ExpandAll),
        Just(AccordionMessage::CollapseAll),
    ]
}

/// Generates a BreadcrumbMessage.
fn breadcrumb_message_strategy(max_index: usize) -> impl Strategy<Value = BreadcrumbMessage> {
    prop_oneof![
        Just(BreadcrumbMessage::Left),
        Just(BreadcrumbMessage::Right),
        Just(BreadcrumbMessage::First),
        Just(BreadcrumbMessage::Last),
        Just(BreadcrumbMessage::Select),
        (0..=max_index).prop_map(BreadcrumbMessage::SelectIndex),
    ]
}

/// Generates a MenuMessage.
fn menu_message_strategy(max_index: usize) -> impl Strategy<Value = MenuMessage> {
    prop_oneof![
        Just(MenuMessage::Right),
        Just(MenuMessage::Left),
        Just(MenuMessage::Select),
        (0..=max_index).prop_map(MenuMessage::SelectIndex),
    ]
}

/// Generates a TableMessage (excluding filter/sort for index validity tests).
fn table_message_strategy() -> impl Strategy<Value = TableMessage> {
    prop_oneof![
        Just(TableMessage::Up),
        Just(TableMessage::Down),
        Just(TableMessage::First),
        Just(TableMessage::Last),
        Just(TableMessage::Select),
        (1usize..50).prop_map(TableMessage::PageUp),
        (1usize..50).prop_map(TableMessage::PageDown),
    ]
}

/// Generates a TreeMessage (excluding filter messages).
fn tree_message_strategy() -> impl Strategy<Value = TreeMessage> {
    prop_oneof![
        Just(TreeMessage::Down),
        Just(TreeMessage::Up),
        Just(TreeMessage::Expand),
        Just(TreeMessage::Collapse),
        Just(TreeMessage::Toggle),
        Just(TreeMessage::Select),
        Just(TreeMessage::ExpandAll),
        Just(TreeMessage::CollapseAll),
    ]
}

/// Generates a TextAreaMessage (subset safe for property testing).
fn text_area_message_strategy() -> impl Strategy<Value = TextAreaMessage> {
    prop_oneof![
        any::<char>()
            .prop_filter("printable", |c| !c.is_control())
            .prop_map(TextAreaMessage::Insert),
        Just(TextAreaMessage::NewLine),
        Just(TextAreaMessage::Backspace),
        Just(TextAreaMessage::Delete),
        Just(TextAreaMessage::Left),
        Just(TextAreaMessage::Right),
        Just(TextAreaMessage::Up),
        Just(TextAreaMessage::Down),
        Just(TextAreaMessage::Home),
        Just(TextAreaMessage::End),
        Just(TextAreaMessage::TextStart),
        Just(TextAreaMessage::TextEnd),
        Just(TextAreaMessage::WordLeft),
        Just(TextAreaMessage::WordRight),
        Just(TextAreaMessage::DeleteLine),
        Just(TextAreaMessage::DeleteToEnd),
        Just(TextAreaMessage::DeleteToStart),
        Just(TextAreaMessage::Clear),
        Just(TextAreaMessage::Undo),
        Just(TextAreaMessage::Redo),
    ]
}

/// Generates a LineInputMessage (subset safe for property testing).
fn line_input_message_strategy() -> impl Strategy<Value = LineInputMessage> {
    prop_oneof![
        any::<char>()
            .prop_filter("printable", |c| !c.is_control())
            .prop_map(LineInputMessage::Insert),
        Just(LineInputMessage::Backspace),
        Just(LineInputMessage::Delete),
        Just(LineInputMessage::DeleteWordBack),
        Just(LineInputMessage::DeleteWordForward),
        Just(LineInputMessage::Clear),
        Just(LineInputMessage::Left),
        Just(LineInputMessage::Right),
        Just(LineInputMessage::Home),
        Just(LineInputMessage::End),
        Just(LineInputMessage::WordLeft),
        Just(LineInputMessage::WordRight),
        Just(LineInputMessage::Undo),
        Just(LineInputMessage::Redo),
    ]
}

/// Generates a DropdownMessage (excluding SetFilter for simplicity).
fn dropdown_message_strategy() -> impl Strategy<Value = DropdownMessage> {
    prop_oneof![
        Just(DropdownMessage::Open),
        Just(DropdownMessage::Close),
        Just(DropdownMessage::Toggle),
        any::<char>()
            .prop_filter("printable", |c| !c.is_control())
            .prop_map(DropdownMessage::Insert),
        Just(DropdownMessage::Backspace),
        Just(DropdownMessage::ClearFilter),
        Just(DropdownMessage::Down),
        Just(DropdownMessage::Up),
        Just(DropdownMessage::Confirm),
    ]
}

/// Generates a ScrollableTextMessage.
fn scrollable_text_message_strategy() -> impl Strategy<Value = ScrollableTextMessage> {
    prop_oneof![
        Just(ScrollableTextMessage::ScrollUp),
        Just(ScrollableTextMessage::ScrollDown),
        (1usize..50).prop_map(ScrollableTextMessage::PageUp),
        (1usize..50).prop_map(ScrollableTextMessage::PageDown),
        Just(ScrollableTextMessage::Home),
        Just(ScrollableTextMessage::End),
    ]
}

/// Generates a LoadingListMessage<String> for navigation.
fn loading_list_message_strategy() -> impl Strategy<Value = LoadingListMessage<String>> {
    prop_oneof![
        Just(LoadingListMessage::Up),
        Just(LoadingListMessage::Down),
        Just(LoadingListMessage::First),
        Just(LoadingListMessage::Last),
        Just(LoadingListMessage::Select),
        Just(LoadingListMessage::Tick),
    ]
}

/// Simple TableRow implementation for property tests.
#[derive(Clone, Debug, PartialEq)]
struct TestRow {
    name: String,
}

impl TableRow for TestRow {
    fn cells(&self) -> Vec<String> {
        vec![self.name.clone()]
    }
}

// ========================================
// Accordion Properties
// ========================================

proptest! {
    /// After any sequence of messages, focused_index is always valid.
    #[test]
    fn accordion_index_always_valid(
        panel_count in 1usize..50,
        messages in prop::collection::vec(accordion_message_strategy(49), 1..100),
    ) {
        let panels: Vec<AccordionPanel> = (0..panel_count)
            .map(|i| AccordionPanel::new(format!("Panel {}", i), format!("Content {}", i)))
            .collect();
        let mut state = AccordionState::new(panels);

        for msg in messages {
            Accordion::update(&mut state, msg);
        }

        let index = state.focused_index();
        prop_assert!(index < panel_count, "Index {} out of bounds for len {}", index, panel_count);
    }

    /// ExpandAll makes all panels expanded, CollapseAll makes all collapsed.
    #[test]
    fn accordion_expand_collapse_all(
        panel_count in 2usize..20,
        prefix_messages in prop::collection::vec(accordion_message_strategy(19), 0..50),
    ) {
        let panels: Vec<AccordionPanel> = (0..panel_count)
            .map(|i| AccordionPanel::new(format!("Panel {}", i), format!("Content {}", i)))
            .collect();
        let mut state = AccordionState::new(panels);

        for msg in prefix_messages {
            Accordion::update(&mut state, msg);
        }

        Accordion::update(&mut state, AccordionMessage::ExpandAll);
        prop_assert!(state.is_all_expanded(), "ExpandAll should expand all panels");

        Accordion::update(&mut state, AccordionMessage::CollapseAll);
        prop_assert_eq!(state.expanded_count(), 0, "CollapseAll should collapse all panels");
    }

    /// Empty accordion never panics on any message sequence.
    #[test]
    fn accordion_empty_never_panics(
        messages in prop::collection::vec(accordion_message_strategy(0), 1..50),
    ) {
        let mut state = AccordionState::new(Vec::new());

        for msg in messages {
            Accordion::update(&mut state, msg);
        }

        prop_assert!(state.is_empty());
    }
}

// ========================================
// Breadcrumb Properties
// ========================================

proptest! {
    /// After any sequence of messages, focused_index is always valid.
    #[test]
    fn breadcrumb_index_always_valid(
        segment_count in 1usize..50,
        messages in prop::collection::vec(breadcrumb_message_strategy(49), 1..100),
    ) {
        let labels: Vec<String> = (0..segment_count).map(|i| format!("Seg {}", i)).collect();
        let mut state = BreadcrumbState::from_labels(labels);

        for msg in messages {
            Breadcrumb::update(&mut state, msg);
        }

        let index = state.focused_index();
        prop_assert!(index < segment_count, "Index {} out of bounds for len {}", index, segment_count);
    }

    /// First/Last always reach correct bounds.
    #[test]
    fn breadcrumb_first_last_bounds(
        segment_count in 2usize..20,
        prefix_messages in prop::collection::vec(breadcrumb_message_strategy(19), 0..50),
    ) {
        let labels: Vec<String> = (0..segment_count).map(|i| format!("Seg {}", i)).collect();
        let mut state = BreadcrumbState::from_labels(labels);

        for msg in prefix_messages {
            Breadcrumb::update(&mut state, msg);
        }

        Breadcrumb::update(&mut state, BreadcrumbMessage::First);
        prop_assert_eq!(state.focused_index(), 0);

        Breadcrumb::update(&mut state, BreadcrumbMessage::Last);
        prop_assert_eq!(state.focused_index(), segment_count - 1);
    }

    /// Empty breadcrumb never panics.
    #[test]
    fn breadcrumb_empty_never_panics(
        messages in prop::collection::vec(breadcrumb_message_strategy(0), 1..50),
    ) {
        let mut state = BreadcrumbState::from_labels(Vec::<String>::new());

        for msg in messages {
            Breadcrumb::update(&mut state, msg);
        }

        prop_assert!(state.is_empty());
    }
}

// ========================================
// Menu Properties
// ========================================

proptest! {
    /// After any sequence of messages, selected_index is always valid.
    #[test]
    fn menu_index_always_valid(
        item_count in 1usize..50,
        messages in prop::collection::vec(menu_message_strategy(49), 1..100),
    ) {
        let items: Vec<MenuItem> = (0..item_count)
            .map(|i| MenuItem::new(format!("Item {}", i)))
            .collect();
        let mut state = MenuState::new(items);

        for msg in messages {
            Menu::update(&mut state, msg);
        }

        let index = state.selected_index();
        prop_assert!(index.is_some(), "Non-empty menu should always have selection");
        prop_assert!(index.unwrap() < item_count, "Index {} out of bounds for len {}", index.unwrap(), item_count);
    }

    /// Empty menu never panics and selection is always None.
    #[test]
    fn menu_empty_always_none(
        messages in prop::collection::vec(menu_message_strategy(0), 1..50),
    ) {
        let mut state = MenuState::new(Vec::new());

        for msg in messages {
            Menu::update(&mut state, msg);
        }

        prop_assert_eq!(state.selected_index(), None);
    }
}

// ========================================
// Table Properties
// ========================================

proptest! {
    /// After any sequence of messages, selected index is always valid.
    #[test]
    fn table_index_always_valid(
        row_count in 1usize..100,
        messages in prop::collection::vec(table_message_strategy(), 1..100),
    ) {
        let rows: Vec<TestRow> = (0..row_count)
            .map(|i| TestRow { name: format!("Row {}", i) })
            .collect();
        let columns = vec![Column::fixed("Name", 20)];
        let mut state = TableState::new(rows, columns);

        for msg in messages {
            Table::<TestRow>::update(&mut state, msg);
        }

        let index = state.selected_index();
        prop_assert!(index.is_some(), "Non-empty table should always have selection");
        prop_assert!(index.unwrap() < row_count, "Index {} out of bounds for len {}", index.unwrap(), row_count);
    }

    /// First/Last always reach correct bounds.
    #[test]
    fn table_first_last_bounds(
        row_count in 2usize..50,
        prefix_messages in prop::collection::vec(table_message_strategy(), 0..50),
    ) {
        let rows: Vec<TestRow> = (0..row_count)
            .map(|i| TestRow { name: format!("Row {}", i) })
            .collect();
        let columns = vec![Column::fixed("Name", 20)];
        let mut state = TableState::new(rows, columns);

        for msg in prefix_messages {
            Table::<TestRow>::update(&mut state, msg);
        }

        Table::<TestRow>::update(&mut state, TableMessage::First);
        prop_assert_eq!(state.selected_index(), Some(0));

        Table::<TestRow>::update(&mut state, TableMessage::Last);
        prop_assert_eq!(state.selected_index(), Some(row_count - 1));
    }

    /// Empty table never panics and selection is always None.
    #[test]
    fn table_empty_always_none(
        messages in prop::collection::vec(table_message_strategy(), 1..50),
    ) {
        let mut state: TableState<TestRow> = TableState::new(Vec::new(), vec![Column::fixed("Name", 20)]);

        for msg in messages {
            Table::<TestRow>::update(&mut state, msg);
        }

        prop_assert_eq!(state.selected_index(), None);
    }
}

// ========================================
// Tree Properties
// ========================================

proptest! {
    /// After any sequence of messages, selected_index is always valid.
    #[test]
    fn tree_index_always_valid(
        root_count in 1usize..10,
        children_per_root in 0usize..5,
        messages in prop::collection::vec(tree_message_strategy(), 1..100),
    ) {
        let roots: Vec<TreeNode<String>> = (0..root_count)
            .map(|i| {
                let mut node = TreeNode::new(format!("Root {}", i), format!("data-{}", i));
                for j in 0..children_per_root {
                    node.add_child(TreeNode::new(
                        format!("Child {}-{}", i, j),
                        format!("child-data-{}-{}", i, j),
                    ));
                }
                node
            })
            .collect();
        let mut state = TreeState::new(roots);

        for msg in messages {
            Tree::<String>::update(&mut state, msg);
        }

        let visible_count = state.visible_count();
        if let Some(index) = state.selected_index() {
            prop_assert!(
                index < visible_count,
                "Index {} out of bounds for visible count {}",
                index, visible_count
            );
        }
    }

    /// Expand/collapse all operations leave selection valid.
    #[test]
    fn tree_expand_collapse_all_valid(
        root_count in 1usize..5,
        children_per_root in 1usize..5,
        prefix_messages in prop::collection::vec(tree_message_strategy(), 0..50),
    ) {
        let roots: Vec<TreeNode<String>> = (0..root_count)
            .map(|i| {
                let mut node = TreeNode::new(format!("Root {}", i), format!("data-{}", i));
                for j in 0..children_per_root {
                    node.add_child(TreeNode::new(
                        format!("Child {}-{}", i, j),
                        format!("child-data-{}-{}", i, j),
                    ));
                }
                node
            })
            .collect();
        let mut state = TreeState::new(roots);

        for msg in prefix_messages {
            Tree::<String>::update(&mut state, msg);
        }

        // ExpandAll then check selection valid
        Tree::<String>::update(&mut state, TreeMessage::ExpandAll);
        let visible_after_expand = state.visible_count();
        if let Some(index) = state.selected_index() {
            prop_assert!(index < visible_after_expand);
        }

        // CollapseAll then check selection valid
        Tree::<String>::update(&mut state, TreeMessage::CollapseAll);
        let visible_after_collapse = state.visible_count();
        if let Some(index) = state.selected_index() {
            prop_assert!(index < visible_after_collapse);
        }
    }

    /// Empty tree never panics and selection is always None.
    #[test]
    fn tree_empty_always_none(
        messages in prop::collection::vec(tree_message_strategy(), 1..50),
    ) {
        let mut state: TreeState<String> = TreeState::new(Vec::new());

        for msg in messages {
            Tree::<String>::update(&mut state, msg);
        }

        prop_assert_eq!(state.selected_index(), None);
    }
}

// ========================================
// TextArea Properties
// ========================================

proptest! {
    /// Cursor position is always valid after any operation sequence.
    #[test]
    fn text_area_cursor_always_valid(
        messages in prop::collection::vec(text_area_message_strategy(), 1..200),
    ) {
        let mut state = TextAreaState::new();

        for msg in messages {
            TextArea::update(&mut state, msg);
        }

        let (row, _char_col) = state.cursor_position();
        let line_count = state.line_count();

        prop_assert!(
            row < line_count,
            "Cursor row {} out of bounds for line count {}",
            row, line_count
        );

        // cursor_col (byte offset) must be within line length
        let cursor_col = state.cursor_col();
        let line = state.line(state.cursor_row()).unwrap();
        prop_assert!(
            cursor_col <= line.len(),
            "Cursor col (byte offset) {} exceeds line length {}. Line: {:?}",
            cursor_col, line.len(), line
        );
    }

    /// Clear always resets to empty with cursor at (0, 0).
    #[test]
    fn text_area_clear_resets(
        prefix_messages in prop::collection::vec(text_area_message_strategy(), 0..100),
    ) {
        let mut state = TextAreaState::new();

        for msg in prefix_messages {
            TextArea::update(&mut state, msg);
        }

        TextArea::update(&mut state, TextAreaMessage::Clear);
        prop_assert!(state.is_empty());
        prop_assert_eq!(state.cursor_position(), (0, 0));
    }

    /// Home/End always reach correct positions on current line.
    #[test]
    fn text_area_home_end(
        prefix_messages in prop::collection::vec(text_area_message_strategy(), 0..100),
    ) {
        let mut state = TextAreaState::new();

        for msg in prefix_messages {
            TextArea::update(&mut state, msg);
        }

        let row_before = state.cursor_row();

        TextArea::update(&mut state, TextAreaMessage::Home);
        prop_assert_eq!(state.cursor_col(), 0, "Home should move cursor to column 0");
        prop_assert_eq!(state.cursor_row(), row_before, "Home should not change row");

        TextArea::update(&mut state, TextAreaMessage::End);
        let line = state.line(state.cursor_row()).unwrap();
        prop_assert_eq!(
            state.cursor_col(), line.len(),
            "End should move cursor to end of line"
        );
    }

    /// TextStart/TextEnd always reach correct positions.
    #[test]
    fn text_area_text_start_end(
        prefix_messages in prop::collection::vec(text_area_message_strategy(), 0..100),
    ) {
        let mut state = TextAreaState::new();

        for msg in prefix_messages {
            TextArea::update(&mut state, msg);
        }

        TextArea::update(&mut state, TextAreaMessage::TextStart);
        prop_assert_eq!(state.cursor_row(), 0);
        prop_assert_eq!(state.cursor_col(), 0);

        TextArea::update(&mut state, TextAreaMessage::TextEnd);
        let last_row = state.line_count() - 1;
        let last_line = state.line(last_row).unwrap();
        prop_assert_eq!(state.cursor_row(), last_row);
        prop_assert_eq!(state.cursor_col(), last_line.len());
    }
}

// ========================================
// LineInput Properties
// ========================================

proptest! {
    /// Cursor position (byte offset) is always valid after any operation.
    #[test]
    fn line_input_cursor_always_valid(
        messages in prop::collection::vec(line_input_message_strategy(), 1..200),
    ) {
        let mut state = LineInputState::new();

        for msg in messages {
            LineInput::update(&mut state, msg);
        }

        let cursor = state.cursor_byte_offset();
        let buf_len = state.value().len();
        prop_assert!(
            cursor <= buf_len,
            "Cursor byte offset {} exceeds buffer length {}. Value: {:?}",
            cursor, buf_len, state.value()
        );
        // Verify cursor is on a char boundary
        prop_assert!(
            state.value().is_char_boundary(cursor),
            "Cursor {} is not on a char boundary in {:?}",
            cursor, state.value()
        );
    }

    /// Clear always resets to empty with cursor at 0.
    #[test]
    fn line_input_clear_resets(
        prefix_messages in prop::collection::vec(line_input_message_strategy(), 0..100),
    ) {
        let mut state = LineInputState::new();

        for msg in prefix_messages {
            LineInput::update(&mut state, msg);
        }

        LineInput::update(&mut state, LineInputMessage::Clear);
        prop_assert_eq!(state.value(), "");
        prop_assert_eq!(state.cursor_byte_offset(), 0);
    }

    /// Home moves cursor to 0, End moves cursor to buffer end.
    #[test]
    fn line_input_home_end(
        prefix_messages in prop::collection::vec(line_input_message_strategy(), 0..100),
    ) {
        let mut state = LineInputState::new();

        for msg in prefix_messages {
            LineInput::update(&mut state, msg);
        }

        LineInput::update(&mut state, LineInputMessage::Home);
        prop_assert_eq!(state.cursor_byte_offset(), 0);

        LineInput::update(&mut state, LineInputMessage::End);
        prop_assert_eq!(state.cursor_byte_offset(), state.value().len());
    }

    /// Submit clears the buffer and returns the value.
    #[test]
    fn line_input_submit_clears(
        prefix_messages in prop::collection::vec(line_input_message_strategy(), 0..50),
    ) {
        let mut state = LineInputState::new();

        for msg in prefix_messages {
            LineInput::update(&mut state, msg);
        }

        LineInput::update(&mut state, LineInputMessage::Submit);
        prop_assert_eq!(state.value(), "");
        prop_assert_eq!(state.cursor_byte_offset(), 0);
    }
}

// ========================================
// Dropdown Properties
// ========================================

proptest! {
    /// selected_index is always valid (None or within bounds) after any sequence.
    #[test]
    fn dropdown_selection_always_valid(
        option_count in 1usize..50,
        messages in prop::collection::vec(dropdown_message_strategy(), 1..100),
    ) {
        let options: Vec<String> = (0..option_count).map(|i| format!("Option {}", i)).collect();
        let mut state = DropdownState::new(options);

        for msg in messages {
            Dropdown::update(&mut state, msg);
        }

        if let Some(index) = state.selected_index() {
            prop_assert!(
                index < option_count,
                "Selected index {} out of bounds for {} options",
                index, option_count
            );
        }
    }

    /// Highlighted index is always within filtered bounds when dropdown is open.
    #[test]
    fn dropdown_highlight_always_valid(
        option_count in 1usize..20,
        messages in prop::collection::vec(dropdown_message_strategy(), 1..50),
    ) {
        let options: Vec<String> = (0..option_count).map(|i| format!("Option {}", i)).collect();
        let mut state = DropdownState::new(options);

        for msg in messages {
            Dropdown::update(&mut state, msg);
        }

        // The highlighted_index is internal, but we can verify via filtered_count
        let filtered = state.filtered_count();
        // If open with items, filtered_count should be > 0 or dropdown should close
        if state.is_open() && filtered > 0 {
            // Verify by confirming - this should not panic
            let mut confirm_state = state.clone();
            Dropdown::update(&mut confirm_state, DropdownMessage::Confirm);
            // If confirm produced a selection, it must be valid
            if let Some(idx) = confirm_state.selected_index() {
                prop_assert!(idx < option_count);
            }
        }
    }

    /// Open/Close toggle is consistent.
    #[test]
    fn dropdown_toggle_consistency(
        option_count in 1usize..20,
        prefix_messages in prop::collection::vec(dropdown_message_strategy(), 0..30),
    ) {
        let options: Vec<String> = (0..option_count).map(|i| format!("Option {}", i)).collect();
        let mut state = DropdownState::new(options);

        for msg in prefix_messages {
            Dropdown::update(&mut state, msg);
        }

        // Close then toggle should open
        Dropdown::update(&mut state, DropdownMessage::Close);
        prop_assert!(!state.is_open());

        Dropdown::update(&mut state, DropdownMessage::Toggle);
        prop_assert!(state.is_open());

        // Toggle again should close
        Dropdown::update(&mut state, DropdownMessage::Toggle);
        prop_assert!(!state.is_open());
    }

    /// Empty dropdown never opens.
    #[test]
    fn dropdown_empty_never_opens(
        messages in prop::collection::vec(dropdown_message_strategy(), 1..50),
    ) {
        let mut state = DropdownState::new(Vec::<String>::new());

        for msg in messages {
            Dropdown::update(&mut state, msg);
        }

        // With no options, dropdown should never be open
        // (Insert messages will auto-open, but only if options is non-empty)
        prop_assert!(!state.is_open());
        prop_assert_eq!(state.selected_index(), None);
    }
}

// ========================================
// ScrollableText Properties
// ========================================

proptest! {
    /// scroll_offset is non-negative (usize) and Home always resets to 0.
    #[test]
    fn scrollable_text_home_resets(
        line_count in 1usize..100,
        prefix_messages in prop::collection::vec(scrollable_text_message_strategy(), 0..50),
    ) {
        let content: String = (0..line_count)
            .map(|i| format!("Line {}", i))
            .collect::<Vec<_>>()
            .join("\n");
        let mut state = ScrollableTextState::new().with_content(content);

        for msg in prefix_messages {
            ScrollableText::update(&mut state, msg);
        }

        ScrollableText::update(&mut state, ScrollableTextMessage::Home);
        prop_assert_eq!(state.scroll_offset(), 0);
    }

    /// ScrollUp from 0 stays at 0 (no underflow).
    #[test]
    fn scrollable_text_no_underflow(
        scroll_ups in 1usize..100,
    ) {
        let mut state = ScrollableTextState::new()
            .with_content("Line 1\nLine 2\nLine 3");

        for _ in 0..scroll_ups {
            ScrollableText::update(&mut state, ScrollableTextMessage::ScrollUp);
        }

        prop_assert_eq!(state.scroll_offset(), 0);
    }

    /// PageUp from 0 stays at 0.
    #[test]
    fn scrollable_text_page_up_no_underflow(
        page_size in 1usize..50,
        pages in 1usize..20,
    ) {
        let mut state = ScrollableTextState::new()
            .with_content("Line 1\nLine 2\nLine 3");

        for _ in 0..pages {
            ScrollableText::update(&mut state, ScrollableTextMessage::PageUp(page_size));
        }

        prop_assert_eq!(state.scroll_offset(), 0);
    }

    /// Scroll operations never cause panics regardless of sequence.
    #[test]
    fn scrollable_text_never_panics(
        messages in prop::collection::vec(scrollable_text_message_strategy(), 1..100),
    ) {
        let mut state = ScrollableTextState::new()
            .with_content("Short content");

        for msg in messages {
            ScrollableText::update(&mut state, msg);
        }

        // Just verifying no panics occurred
        let _ = state.scroll_offset();
    }
}

// ========================================
// LoadingList Properties
// ========================================

proptest! {
    /// After any sequence of navigation messages, selected is always valid.
    #[test]
    fn loading_list_index_always_valid(
        item_count in 1usize..100,
        messages in prop::collection::vec(loading_list_message_strategy(), 1..100),
    ) {
        let items: Vec<String> = (0..item_count).map(|i| format!("Item {}", i)).collect();
        let mut state = LoadingListState::with_items(items, |s| s.clone());

        for msg in messages {
            LoadingList::<String>::update(&mut state, msg);
        }

        if let Some(index) = state.selected_index() {
            prop_assert!(
                index < item_count,
                "Index {} out of bounds for len {}",
                index, item_count
            );
        }
    }

    /// First/Last always reach correct bounds.
    #[test]
    fn loading_list_first_last_bounds(
        item_count in 2usize..50,
        prefix_messages in prop::collection::vec(loading_list_message_strategy(), 0..50),
    ) {
        let items: Vec<String> = (0..item_count).map(|i| format!("Item {}", i)).collect();
        let mut state = LoadingListState::with_items(items, |s| s.clone());

        for msg in prefix_messages {
            LoadingList::<String>::update(&mut state, msg);
        }

        LoadingList::<String>::update(&mut state, LoadingListMessage::First);
        prop_assert_eq!(state.selected_index(), Some(0));

        LoadingList::<String>::update(&mut state, LoadingListMessage::Last);
        prop_assert_eq!(state.selected_index(), Some(item_count - 1));
    }

    /// Empty loading list never has selection, regardless of messages.
    #[test]
    fn loading_list_empty_always_none(
        messages in prop::collection::vec(loading_list_message_strategy(), 1..50),
    ) {
        let mut state: LoadingListState<String> = LoadingListState::new();

        for msg in messages {
            LoadingList::<String>::update(&mut state, msg);
        }

        prop_assert_eq!(state.selected_index(), None);
    }
}
