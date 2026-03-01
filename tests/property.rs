//! Property-based tests for envision components using proptest.
//!
//! These tests verify invariants that must hold for arbitrary sequences
//! of operations, catching edge cases that hand-written tests miss.

use envision::{
    Component, FocusManager, InputField, InputFieldMessage, InputFieldState, RadioGroup,
    RadioGroupMessage, RadioGroupState, SelectableList, SelectableListMessage, SelectableListState,
    Tabs, TabsMessage, TabsState,
};
use proptest::prelude::*;

// ========================================
// Strategy Helpers
// ========================================

/// Generates a SelectableListMessage (excluding PageUp/PageDown which need a size param).
fn selectable_list_message_strategy() -> impl Strategy<Value = SelectableListMessage> {
    prop_oneof![
        Just(SelectableListMessage::Up),
        Just(SelectableListMessage::Down),
        Just(SelectableListMessage::First),
        Just(SelectableListMessage::Last),
        Just(SelectableListMessage::Select),
        (1usize..50).prop_map(SelectableListMessage::PageUp),
        (1usize..50).prop_map(SelectableListMessage::PageDown),
    ]
}

/// Generates an InputFieldMessage.
fn input_field_message_strategy() -> impl Strategy<Value = InputFieldMessage> {
    prop_oneof![
        any::<char>()
            .prop_filter("printable", |c| !c.is_control())
            .prop_map(InputFieldMessage::Insert),
        Just(InputFieldMessage::Backspace),
        Just(InputFieldMessage::Delete),
        Just(InputFieldMessage::Left),
        Just(InputFieldMessage::Right),
        Just(InputFieldMessage::Home),
        Just(InputFieldMessage::End),
        Just(InputFieldMessage::WordLeft),
        Just(InputFieldMessage::WordRight),
        Just(InputFieldMessage::DeleteWordBack),
        Just(InputFieldMessage::DeleteWordForward),
        Just(InputFieldMessage::Clear),
    ]
}

/// Generates a RadioGroupMessage.
fn radio_group_message_strategy() -> impl Strategy<Value = RadioGroupMessage> {
    prop_oneof![
        Just(RadioGroupMessage::Up),
        Just(RadioGroupMessage::Down),
        Just(RadioGroupMessage::Confirm),
    ]
}

/// Generates a TabsMessage.
fn tabs_message_strategy(max_index: usize) -> impl Strategy<Value = TabsMessage> {
    prop_oneof![
        Just(TabsMessage::Left),
        Just(TabsMessage::Right),
        Just(TabsMessage::First),
        Just(TabsMessage::Last),
        Just(TabsMessage::Confirm),
        (0..=max_index).prop_map(TabsMessage::Select),
    ]
}

/// Generates a FocusManager operation.
#[derive(Clone, Debug)]
enum FocusOp {
    Next,
    Prev,
    First,
    Last,
    Blur,
}

fn focus_op_strategy() -> impl Strategy<Value = FocusOp> {
    prop_oneof![
        Just(FocusOp::Next),
        Just(FocusOp::Prev),
        Just(FocusOp::First),
        Just(FocusOp::Last),
        Just(FocusOp::Blur),
    ]
}

// ========================================
// SelectableList Properties
// ========================================

proptest! {
    /// After any sequence of messages, selected_index is always valid
    /// (either None for empty lists or Some(i) where i < len).
    #[test]
    fn selectable_list_index_always_valid(
        item_count in 1usize..200,
        messages in prop::collection::vec(selectable_list_message_strategy(), 1..100),
    ) {
        let items: Vec<String> = (0..item_count).map(|i| format!("Item {}", i)).collect();
        let mut state = SelectableListState::new(items);

        for msg in messages {
            SelectableList::<String>::update(&mut state, msg);
        }

        let index = state.selected_index();
        prop_assert!(index.is_some(), "Non-empty list should always have selection");
        prop_assert!(index.unwrap() < item_count, "Index {} out of bounds for len {}", index.unwrap(), item_count);
    }

    /// First always selects index 0, Last always selects the final index.
    #[test]
    fn selectable_list_first_last_bounds(
        item_count in 1usize..200,
        prefix_messages in prop::collection::vec(selectable_list_message_strategy(), 0..50),
    ) {
        let items: Vec<String> = (0..item_count).map(|i| format!("Item {}", i)).collect();
        let mut state = SelectableListState::new(items);

        // Apply random prefix to get into an arbitrary state
        for msg in prefix_messages {
            SelectableList::<String>::update(&mut state, msg);
        }

        // First always goes to 0
        SelectableList::<String>::update(&mut state, SelectableListMessage::First);
        prop_assert_eq!(state.selected_index(), Some(0));

        // Last always goes to len - 1
        SelectableList::<String>::update(&mut state, SelectableListMessage::Last);
        prop_assert_eq!(state.selected_index(), Some(item_count - 1));
    }

    /// Empty list never has a selection, regardless of messages.
    #[test]
    fn selectable_list_empty_always_none(
        messages in prop::collection::vec(selectable_list_message_strategy(), 1..50),
    ) {
        let mut state: SelectableListState<String> = SelectableListState::new(Vec::new());

        for msg in messages {
            SelectableList::<String>::update(&mut state, msg);
        }

        prop_assert_eq!(state.selected_index(), None);
    }
}

// ========================================
// InputField Properties
// ========================================

proptest! {
    /// Cursor position is always within [0, char_count] after any operation sequence.
    /// Note: cursor_position() is character-based, not byte-based.
    #[test]
    fn input_field_cursor_always_valid(
        messages in prop::collection::vec(input_field_message_strategy(), 1..200),
    ) {
        let mut state = InputFieldState::new();

        for msg in messages {
            InputField::update(&mut state, msg);
        }

        let cursor = state.cursor_position();
        let char_count = state.value().chars().count();
        prop_assert!(
            cursor <= char_count,
            "Cursor {} exceeds char count {}. Value: {:?}",
            cursor, char_count, state.value()
        );
    }

    /// Clear always resets to empty value and cursor at 0.
    #[test]
    fn input_field_clear_resets(
        prefix_messages in prop::collection::vec(input_field_message_strategy(), 0..100),
    ) {
        let mut state = InputFieldState::new();

        for msg in prefix_messages {
            InputField::update(&mut state, msg);
        }

        InputField::update(&mut state, InputFieldMessage::Clear);
        prop_assert_eq!(state.value(), "");
        prop_assert_eq!(state.cursor_position(), 0);
    }

    /// Home always moves cursor to 0, End always moves cursor to char count.
    /// Note: cursor_position() is character-based, not byte-based.
    #[test]
    fn input_field_home_end(
        prefix_messages in prop::collection::vec(input_field_message_strategy(), 0..100),
    ) {
        let mut state = InputFieldState::new();

        for msg in prefix_messages {
            InputField::update(&mut state, msg);
        }

        InputField::update(&mut state, InputFieldMessage::Home);
        prop_assert_eq!(state.cursor_position(), 0);

        InputField::update(&mut state, InputFieldMessage::End);
        let char_count = state.value().chars().count();
        prop_assert_eq!(state.cursor_position(), char_count);
    }

    /// Insert followed by Backspace is a no-op on the value when cursor is at the end.
    #[test]
    fn input_field_insert_backspace_roundtrip(
        prefix_messages in prop::collection::vec(input_field_message_strategy(), 0..50),
        ch in any::<char>().prop_filter("printable ascii", |c| c.is_ascii_alphanumeric()),
    ) {
        let mut state = InputFieldState::new();

        for msg in prefix_messages {
            InputField::update(&mut state, msg);
        }

        // Move to end first
        InputField::update(&mut state, InputFieldMessage::End);
        let value_before = state.value().to_string();

        // Insert then backspace should restore value
        InputField::update(&mut state, InputFieldMessage::Insert(ch));
        InputField::update(&mut state, InputFieldMessage::Backspace);
        prop_assert_eq!(state.value(), &value_before);
    }
}

// ========================================
// RadioGroup Properties
// ========================================

proptest! {
    /// After any sequence of messages, selected_index is always valid.
    #[test]
    fn radio_group_index_always_valid(
        item_count in 2usize..50,
        messages in prop::collection::vec(radio_group_message_strategy(), 1..100),
    ) {
        let items: Vec<String> = (0..item_count).map(|i| format!("Option {}", i)).collect();
        let mut state = RadioGroupState::new(items);

        for msg in messages {
            RadioGroup::<String>::update(&mut state, msg);
        }

        let index = state.selected_index();
        prop_assert!(index.is_some());
        prop_assert!(index.unwrap() < item_count);
    }
}

// ========================================
// Tabs Properties
// ========================================

proptest! {
    /// After any sequence of messages, selected_index is always valid.
    #[test]
    fn tabs_index_always_valid(
        tab_count in 2usize..20,
        messages in prop::collection::vec(tabs_message_strategy(19), 1..100),
    ) {
        let tabs: Vec<String> = (0..tab_count).map(|i| format!("Tab {}", i)).collect();
        let mut state = TabsState::new(tabs);

        for msg in messages {
            Tabs::<String>::update(&mut state, msg);
        }

        let index = state.selected_index();
        prop_assert!(index.is_some());
        prop_assert!(index.unwrap() < tab_count);
    }

    /// First/Last always reach the correct bounds.
    #[test]
    fn tabs_first_last_bounds(
        tab_count in 2usize..20,
        prefix_messages in prop::collection::vec(tabs_message_strategy(19), 0..50),
    ) {
        let tabs: Vec<String> = (0..tab_count).map(|i| format!("Tab {}", i)).collect();
        let mut state = TabsState::new(tabs);

        for msg in prefix_messages {
            Tabs::<String>::update(&mut state, msg);
        }

        Tabs::<String>::update(&mut state, TabsMessage::First);
        prop_assert_eq!(state.selected_index(), Some(0));

        Tabs::<String>::update(&mut state, TabsMessage::Last);
        prop_assert_eq!(state.selected_index(), Some(tab_count - 1));
    }
}

// ========================================
// FocusManager Properties
// ========================================

proptest! {
    /// After any sequence of operations, focused() is always either None or a valid ID.
    #[test]
    fn focus_manager_always_valid(
        field_count in 2usize..20,
        ops in prop::collection::vec(focus_op_strategy(), 1..100),
    ) {
        let fields: Vec<usize> = (0..field_count).collect();
        let mut fm = FocusManager::with_initial_focus(fields.clone());

        for op in ops {
            match op {
                FocusOp::Next => { fm.focus_next(); }
                FocusOp::Prev => { fm.focus_prev(); }
                FocusOp::First => { fm.focus_first(); }
                FocusOp::Last => { fm.focus_last(); }
                FocusOp::Blur => { fm.blur(); }
            }
        }

        if let Some(focused) = fm.focused() {
            prop_assert!(fields.contains(focused), "Focused ID {} not in fields", focused);
        }
    }

    /// focus_next n times from first always reaches specific positions (modular cycling).
    #[test]
    fn focus_manager_next_cycles(
        field_count in 2usize..20,
        steps in 1usize..100,
    ) {
        let fields: Vec<usize> = (0..field_count).collect();
        let mut fm = FocusManager::with_initial_focus(fields);

        for _ in 0..steps {
            fm.focus_next();
        }

        let expected = steps % field_count;
        prop_assert_eq!(fm.focused(), Some(&expected));
    }

    /// Blur followed by focus_next always produces the first element.
    #[test]
    fn focus_manager_blur_then_next(
        field_count in 2usize..20,
        prefix_ops in prop::collection::vec(focus_op_strategy(), 0..50),
    ) {
        let fields: Vec<usize> = (0..field_count).collect();
        let mut fm = FocusManager::with_initial_focus(fields);

        for op in prefix_ops {
            match op {
                FocusOp::Next => { fm.focus_next(); }
                FocusOp::Prev => { fm.focus_prev(); }
                FocusOp::First => { fm.focus_first(); }
                FocusOp::Last => { fm.focus_last(); }
                FocusOp::Blur => { fm.blur(); }
            }
        }

        fm.blur();
        prop_assert_eq!(fm.focused(), None);

        fm.focus_next();
        prop_assert_eq!(fm.focused(), Some(&0));
    }
}
