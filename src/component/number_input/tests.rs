use super::*;
use crate::input::{Event, KeyCode};

// ========================================
// Construction Tests
// ========================================

#[test]
fn test_new() {
    let state = NumberInputState::new(42.0);
    assert_eq!(state.value(), 42.0);
    assert_eq!(state.step(), 1.0);
    assert_eq!(state.precision(), 0);
    assert_eq!(state.min(), None);
    assert_eq!(state.max(), None);
    assert_eq!(state.label(), None);
    assert_eq!(state.placeholder(), None);
    assert!(!state.is_editing());
}

#[test]
fn test_integer() {
    let state = NumberInputState::integer(42);
    assert_eq!(state.value(), 42.0);
    assert_eq!(state.step(), 1.0);
    assert_eq!(state.precision(), 0);
    assert_eq!(state.format_value(), "42");
}

#[test]
fn test_integer_negative() {
    let state = NumberInputState::integer(-10);
    assert_eq!(state.value(), -10.0);
    assert_eq!(state.format_value(), "-10");
}

#[test]
fn test_default() {
    let state = NumberInputState::default();
    assert_eq!(state.value(), 0.0);
    assert_eq!(state.step(), 1.0);
    assert_eq!(state.precision(), 0);
    assert_eq!(state.min(), None);
    assert_eq!(state.max(), None);
}

#[test]
fn test_with_min() {
    let state = NumberInputState::new(5.0).with_min(0.0);
    assert_eq!(state.min(), Some(0.0));
    assert_eq!(state.value(), 5.0);
}

#[test]
fn test_with_min_clamps_value() {
    let state = NumberInputState::new(-5.0).with_min(0.0);
    assert_eq!(state.value(), 0.0);
}

#[test]
fn test_with_max() {
    let state = NumberInputState::new(5.0).with_max(10.0);
    assert_eq!(state.max(), Some(10.0));
    assert_eq!(state.value(), 5.0);
}

#[test]
fn test_with_max_clamps_value() {
    let state = NumberInputState::new(15.0).with_max(10.0);
    assert_eq!(state.value(), 10.0);
}

#[test]
fn test_with_range() {
    let state = NumberInputState::new(5.0).with_range(0.0, 10.0);
    assert_eq!(state.min(), Some(0.0));
    assert_eq!(state.max(), Some(10.0));
    assert_eq!(state.value(), 5.0);
}

#[test]
fn test_with_range_clamps_above() {
    let state = NumberInputState::new(20.0).with_range(0.0, 10.0);
    assert_eq!(state.value(), 10.0);
}

#[test]
fn test_with_range_clamps_below() {
    let state = NumberInputState::new(-5.0).with_range(0.0, 10.0);
    assert_eq!(state.value(), 0.0);
}

#[test]
fn test_with_step() {
    let state = NumberInputState::new(0.0).with_step(0.5);
    assert_eq!(state.step(), 0.5);
}

#[test]
fn test_with_precision() {
    let state = NumberInputState::new(3.75).with_precision(2);
    assert_eq!(state.precision(), 2);
    assert_eq!(state.format_value(), "3.75");
}

#[test]
fn test_with_label() {
    let state = NumberInputState::new(0.0).with_label("Quantity");
    assert_eq!(state.label(), Some("Quantity"));
}

#[test]
fn test_with_placeholder() {
    let state = NumberInputState::new(0.0).with_placeholder("Enter value...");
    assert_eq!(state.placeholder(), Some("Enter value..."));
}

#[test]
fn test_builder_chaining() {
    let state = NumberInputState::new(5.0)
        .with_min(0.0)
        .with_max(100.0)
        .with_step(5.0)
        .with_precision(1)
        .with_label("Volume")
        .with_placeholder("0-100");
    assert_eq!(state.value(), 5.0);
    assert_eq!(state.min(), Some(0.0));
    assert_eq!(state.max(), Some(100.0));
    assert_eq!(state.step(), 5.0);
    assert_eq!(state.precision(), 1);
    assert_eq!(state.label(), Some("Volume"));
    assert_eq!(state.placeholder(), Some("0-100"));
}

// ========================================
// Value Operation Tests
// ========================================

#[test]
fn test_increment() {
    let mut state = NumberInputState::new(10.0);
    let output = NumberInput::update(&mut state, NumberInputMessage::Increment);
    assert_eq!(output, Some(NumberInputOutput::ValueChanged(11.0)));
    assert_eq!(state.value(), 11.0);
}

#[test]
fn test_decrement() {
    let mut state = NumberInputState::new(10.0);
    let output = NumberInput::update(&mut state, NumberInputMessage::Decrement);
    assert_eq!(output, Some(NumberInputOutput::ValueChanged(9.0)));
    assert_eq!(state.value(), 9.0);
}

#[test]
fn test_increment_with_custom_step() {
    let mut state = NumberInputState::new(10.0).with_step(5.0);
    let output = NumberInput::update(&mut state, NumberInputMessage::Increment);
    assert_eq!(output, Some(NumberInputOutput::ValueChanged(15.0)));
    assert_eq!(state.value(), 15.0);
}

#[test]
fn test_decrement_with_custom_step() {
    let mut state = NumberInputState::new(10.0).with_step(5.0);
    let output = NumberInput::update(&mut state, NumberInputMessage::Decrement);
    assert_eq!(output, Some(NumberInputOutput::ValueChanged(5.0)));
    assert_eq!(state.value(), 5.0);
}

#[test]
fn test_increment_clamped_at_max() {
    let mut state = NumberInputState::new(100.0).with_max(100.0);
    let output = NumberInput::update(&mut state, NumberInputMessage::Increment);
    assert_eq!(output, None);
    assert_eq!(state.value(), 100.0);
}

#[test]
fn test_decrement_clamped_at_min() {
    let mut state = NumberInputState::new(0.0).with_min(0.0);
    let output = NumberInput::update(&mut state, NumberInputMessage::Decrement);
    assert_eq!(output, None);
    assert_eq!(state.value(), 0.0);
}

#[test]
fn test_increment_near_max_clamps() {
    let mut state = NumberInputState::new(99.5).with_max(100.0);
    let output = NumberInput::update(&mut state, NumberInputMessage::Increment);
    assert_eq!(output, Some(NumberInputOutput::ValueChanged(100.0)));
    assert_eq!(state.value(), 100.0);
}

#[test]
fn test_decrement_near_min_clamps() {
    let mut state = NumberInputState::new(0.5).with_min(0.0);
    let output = NumberInput::update(&mut state, NumberInputMessage::Decrement);
    assert_eq!(output, Some(NumberInputOutput::ValueChanged(0.0)));
    assert_eq!(state.value(), 0.0);
}

#[test]
fn test_set_value() {
    let mut state = NumberInputState::new(0.0);
    let output = NumberInput::update(&mut state, NumberInputMessage::SetValue(42.0));
    assert_eq!(output, Some(NumberInputOutput::ValueChanged(42.0)));
    assert_eq!(state.value(), 42.0);
}

#[test]
fn test_set_value_clamped_above() {
    let mut state = NumberInputState::new(0.0).with_max(100.0);
    let output = NumberInput::update(&mut state, NumberInputMessage::SetValue(200.0));
    assert_eq!(output, Some(NumberInputOutput::ValueChanged(100.0)));
    assert_eq!(state.value(), 100.0);
}

#[test]
fn test_set_value_clamped_below() {
    let mut state = NumberInputState::new(50.0).with_min(0.0);
    let output = NumberInput::update(&mut state, NumberInputMessage::SetValue(-50.0));
    assert_eq!(output, Some(NumberInputOutput::ValueChanged(0.0)));
    assert_eq!(state.value(), 0.0);
}

#[test]
fn test_set_value_same_value() {
    let mut state = NumberInputState::new(42.0);
    let output = NumberInput::update(&mut state, NumberInputMessage::SetValue(42.0));
    assert_eq!(output, None);
}

#[test]
fn test_set_value_method() {
    let mut state = NumberInputState::new(0.0).with_range(0.0, 100.0);
    state.set_value(50.0);
    assert_eq!(state.value(), 50.0);
}

#[test]
fn test_set_value_method_clamped() {
    let mut state = NumberInputState::new(0.0).with_range(0.0, 100.0);
    state.set_value(150.0);
    assert_eq!(state.value(), 100.0);

    state.set_value(-10.0);
    assert_eq!(state.value(), 0.0);
}

// ========================================
// Edit Mode Tests
// ========================================

#[test]
fn test_start_edit() {
    let mut state = NumberInputState::new(42.0);
    let output = NumberInput::update(&mut state, NumberInputMessage::StartEdit);
    assert_eq!(output, Some(NumberInputOutput::EditStarted));
    assert!(state.is_editing());
    assert_eq!(state.edit_buffer(), "42");
}

#[test]
fn test_start_edit_with_precision() {
    let mut state = NumberInputState::new(3.75).with_precision(2);
    let output = NumberInput::update(&mut state, NumberInputMessage::StartEdit);
    assert_eq!(output, Some(NumberInputOutput::EditStarted));
    assert_eq!(state.edit_buffer(), "3.75");
}

#[test]
fn test_edit_char() {
    let mut state = NumberInputState::new(0.0);
    NumberInput::update(&mut state, NumberInputMessage::StartEdit);
    // Clear the buffer to simulate typing fresh
    state.edit_buffer.clear();
    NumberInput::update(&mut state, NumberInputMessage::EditChar('5'));
    assert_eq!(state.edit_buffer(), "5");
    NumberInput::update(&mut state, NumberInputMessage::EditChar('0'));
    assert_eq!(state.edit_buffer(), "50");
}

#[test]
fn test_edit_char_decimal() {
    let mut state = NumberInputState::new(0.0);
    NumberInput::update(&mut state, NumberInputMessage::StartEdit);
    state.edit_buffer.clear();
    NumberInput::update(&mut state, NumberInputMessage::EditChar('3'));
    NumberInput::update(&mut state, NumberInputMessage::EditChar('.'));
    NumberInput::update(&mut state, NumberInputMessage::EditChar('5'));
    assert_eq!(state.edit_buffer(), "3.5");
}

#[test]
fn test_edit_char_negative() {
    let mut state = NumberInputState::new(0.0);
    NumberInput::update(&mut state, NumberInputMessage::StartEdit);
    state.edit_buffer.clear();
    NumberInput::update(&mut state, NumberInputMessage::EditChar('-'));
    NumberInput::update(&mut state, NumberInputMessage::EditChar('5'));
    assert_eq!(state.edit_buffer(), "-5");
}

#[test]
fn test_edit_char_rejects_duplicate_decimal() {
    let mut state = NumberInputState::new(0.0);
    NumberInput::update(&mut state, NumberInputMessage::StartEdit);
    state.edit_buffer.clear();
    NumberInput::update(&mut state, NumberInputMessage::EditChar('3'));
    NumberInput::update(&mut state, NumberInputMessage::EditChar('.'));
    NumberInput::update(&mut state, NumberInputMessage::EditChar('.'));
    assert_eq!(state.edit_buffer(), "3.");
}

#[test]
fn test_edit_char_rejects_non_leading_minus() {
    let mut state = NumberInputState::new(0.0);
    NumberInput::update(&mut state, NumberInputMessage::StartEdit);
    state.edit_buffer.clear();
    NumberInput::update(&mut state, NumberInputMessage::EditChar('5'));
    NumberInput::update(&mut state, NumberInputMessage::EditChar('-'));
    assert_eq!(state.edit_buffer(), "5");
}

#[test]
fn test_edit_char_rejects_letters() {
    let mut state = NumberInputState::new(0.0);
    NumberInput::update(&mut state, NumberInputMessage::StartEdit);
    state.edit_buffer.clear();
    NumberInput::update(&mut state, NumberInputMessage::EditChar('a'));
    NumberInput::update(&mut state, NumberInputMessage::EditChar('b'));
    assert_eq!(state.edit_buffer(), "");
}

#[test]
fn test_edit_backspace() {
    let mut state = NumberInputState::new(42.0);
    NumberInput::update(&mut state, NumberInputMessage::StartEdit);
    assert_eq!(state.edit_buffer(), "42");
    NumberInput::update(&mut state, NumberInputMessage::EditBackspace);
    assert_eq!(state.edit_buffer(), "4");
    NumberInput::update(&mut state, NumberInputMessage::EditBackspace);
    assert_eq!(state.edit_buffer(), "");
}

#[test]
fn test_edit_backspace_empty() {
    let mut state = NumberInputState::new(0.0);
    NumberInput::update(&mut state, NumberInputMessage::StartEdit);
    state.edit_buffer.clear();
    NumberInput::update(&mut state, NumberInputMessage::EditBackspace);
    assert_eq!(state.edit_buffer(), "");
}

#[test]
fn test_confirm_edit_valid() {
    let mut state = NumberInputState::new(0.0);
    NumberInput::update(&mut state, NumberInputMessage::StartEdit);
    state.edit_buffer = "42".to_string();
    let output = NumberInput::update(&mut state, NumberInputMessage::ConfirmEdit);
    assert_eq!(output, Some(NumberInputOutput::EditConfirmed(42.0)));
    assert!(!state.is_editing());
    assert_eq!(state.value(), 42.0);
    assert_eq!(state.edit_buffer(), "");
}

#[test]
fn test_confirm_edit_valid_float() {
    let mut state = NumberInputState::new(0.0).with_precision(2);
    NumberInput::update(&mut state, NumberInputMessage::StartEdit);
    state.edit_buffer = "3.75".to_string();
    let output = NumberInput::update(&mut state, NumberInputMessage::ConfirmEdit);
    assert_eq!(output, Some(NumberInputOutput::EditConfirmed(3.75)));
    assert_eq!(state.value(), 3.75);
}

#[test]
fn test_confirm_edit_clamped() {
    let mut state = NumberInputState::new(0.0).with_range(0.0, 100.0);
    NumberInput::update(&mut state, NumberInputMessage::StartEdit);
    state.edit_buffer = "200".to_string();
    let output = NumberInput::update(&mut state, NumberInputMessage::ConfirmEdit);
    assert_eq!(output, Some(NumberInputOutput::EditConfirmed(100.0)));
    assert_eq!(state.value(), 100.0);
}

#[test]
fn test_confirm_edit_invalid_reverts() {
    let mut state = NumberInputState::new(42.0);
    NumberInput::update(&mut state, NumberInputMessage::StartEdit);
    state.edit_buffer = "not_a_number".to_string();
    let output = NumberInput::update(&mut state, NumberInputMessage::ConfirmEdit);
    assert_eq!(output, Some(NumberInputOutput::EditCancelled));
    assert!(!state.is_editing());
    assert_eq!(state.value(), 42.0); // Value unchanged
}

#[test]
fn test_confirm_edit_empty_buffer() {
    let mut state = NumberInputState::new(42.0);
    NumberInput::update(&mut state, NumberInputMessage::StartEdit);
    state.edit_buffer.clear();
    let output = NumberInput::update(&mut state, NumberInputMessage::ConfirmEdit);
    // Empty string fails to parse, so it cancels
    assert_eq!(output, Some(NumberInputOutput::EditCancelled));
    assert_eq!(state.value(), 42.0);
}

#[test]
fn test_cancel_edit() {
    let mut state = NumberInputState::new(42.0);
    NumberInput::update(&mut state, NumberInputMessage::StartEdit);
    state.edit_buffer = "99".to_string();
    let output = NumberInput::update(&mut state, NumberInputMessage::CancelEdit);
    assert_eq!(output, Some(NumberInputOutput::EditCancelled));
    assert!(!state.is_editing());
    assert_eq!(state.value(), 42.0); // Value unchanged
    assert_eq!(state.edit_buffer(), "");
}

#[test]
fn test_confirm_edit_same_value() {
    let mut state = NumberInputState::new(42.0);
    NumberInput::update(&mut state, NumberInputMessage::StartEdit);
    // Buffer is "42", confirming same value
    let output = NumberInput::update(&mut state, NumberInputMessage::ConfirmEdit);
    assert_eq!(output, Some(NumberInputOutput::EditConfirmed(42.0)));
    assert!(!state.is_editing());
}

// ========================================
// Format Tests
// ========================================

#[test]
fn test_format_value_integer() {
    let state = NumberInputState::new(42.0);
    assert_eq!(state.format_value(), "42");
}

#[test]
fn test_format_value_precision_zero() {
    let state = NumberInputState::new(42.7).with_precision(0);
    assert_eq!(state.format_value(), "43");
}

#[test]
fn test_format_value_precision_two() {
    let state = NumberInputState::new(3.75).with_precision(2);
    assert_eq!(state.format_value(), "3.75");
}

#[test]
fn test_format_value_precision_three() {
    let state = NumberInputState::new(1.0).with_precision(3);
    assert_eq!(state.format_value(), "1.000");
}

#[test]
fn test_format_value_negative() {
    let state = NumberInputState::new(-42.0);
    assert_eq!(state.format_value(), "-42");
}

#[test]
fn test_format_value_zero() {
    let state = NumberInputState::new(0.0);
    assert_eq!(state.format_value(), "0");
}

// ========================================
// Event Handling Tests - Normal Mode
// ========================================

#[test]
fn test_handle_event_up_increments() {
    let state = NumberInputState::new(0.0);
    let msg = NumberInput::handle_event(
        &state,
        &Event::key(KeyCode::Up),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(NumberInputMessage::Increment));
}

#[test]
fn test_handle_event_down_decrements() {
    let state = NumberInputState::new(0.0);
    let msg = NumberInput::handle_event(
        &state,
        &Event::key(KeyCode::Down),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(NumberInputMessage::Decrement));
}

#[test]
fn test_handle_event_k_increments() {
    let state = NumberInputState::new(0.0);
    let msg =
        NumberInput::handle_event(&state, &Event::char('k'), &ViewContext::new().focused(true));
    assert_eq!(msg, Some(NumberInputMessage::Increment));
}

#[test]
fn test_handle_event_j_decrements() {
    let state = NumberInputState::new(0.0);
    let msg =
        NumberInput::handle_event(&state, &Event::char('j'), &ViewContext::new().focused(true));
    assert_eq!(msg, Some(NumberInputMessage::Decrement));
}

#[test]
fn test_handle_event_enter_starts_edit() {
    let state = NumberInputState::new(0.0);
    let msg = NumberInput::handle_event(
        &state,
        &Event::key(KeyCode::Enter),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(NumberInputMessage::StartEdit));
}

#[test]
fn test_handle_event_unrelated_key() {
    let state = NumberInputState::new(0.0);
    let msg =
        NumberInput::handle_event(&state, &Event::char('q'), &ViewContext::new().focused(true));
    assert_eq!(msg, None);
}

// ========================================
// Event Handling Tests - Edit Mode
// ========================================

#[test]
fn test_handle_event_edit_mode_char() {
    let mut state = NumberInputState::new(0.0);
    state.editing = true;
    state.edit_buffer.clear();
    let msg =
        NumberInput::handle_event(&state, &Event::char('5'), &ViewContext::new().focused(true));
    assert_eq!(msg, Some(NumberInputMessage::EditChar('5')));
}

#[test]
fn test_handle_event_edit_mode_decimal() {
    let mut state = NumberInputState::new(0.0);
    state.editing = true;
    state.edit_buffer = "3".to_string();
    let msg =
        NumberInput::handle_event(&state, &Event::char('.'), &ViewContext::new().focused(true));
    assert_eq!(msg, Some(NumberInputMessage::EditChar('.')));
}

#[test]
fn test_handle_event_edit_mode_minus() {
    let mut state = NumberInputState::new(0.0);
    state.editing = true;
    state.edit_buffer.clear();
    let msg =
        NumberInput::handle_event(&state, &Event::char('-'), &ViewContext::new().focused(true));
    assert_eq!(msg, Some(NumberInputMessage::EditChar('-')));
}

#[test]
fn test_handle_event_edit_mode_enter_confirms() {
    let mut state = NumberInputState::new(0.0);
    state.editing = true;
    let msg = NumberInput::handle_event(
        &state,
        &Event::key(KeyCode::Enter),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(NumberInputMessage::ConfirmEdit));
}

#[test]
fn test_handle_event_edit_mode_escape_cancels() {
    let mut state = NumberInputState::new(0.0);
    state.editing = true;
    let msg = NumberInput::handle_event(
        &state,
        &Event::key(KeyCode::Esc),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(NumberInputMessage::CancelEdit));
}

#[test]
fn test_handle_event_edit_mode_backspace() {
    let mut state = NumberInputState::new(0.0);
    state.editing = true;
    let msg = NumberInput::handle_event(
        &state,
        &Event::key(KeyCode::Backspace),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(NumberInputMessage::EditBackspace));
}

#[test]
fn test_handle_event_edit_mode_rejects_letter() {
    let mut state = NumberInputState::new(0.0);
    state.editing = true;
    state.edit_buffer.clear();
    let msg =
        NumberInput::handle_event(&state, &Event::char('a'), &ViewContext::new().focused(true));
    assert_eq!(msg, None);
}

#[test]
fn test_handle_event_edit_mode_rejects_duplicate_decimal() {
    let mut state = NumberInputState::new(0.0);
    state.editing = true;
    state.edit_buffer = "3.1".to_string();
    let msg =
        NumberInput::handle_event(&state, &Event::char('.'), &ViewContext::new().focused(true));
    assert_eq!(msg, None);
}

#[test]
fn test_handle_event_edit_mode_rejects_non_leading_minus() {
    let mut state = NumberInputState::new(0.0);
    state.editing = true;
    state.edit_buffer = "5".to_string();
    let msg =
        NumberInput::handle_event(&state, &Event::char('-'), &ViewContext::new().focused(true));
    assert_eq!(msg, None);
}

// ========================================
// Event Guard Tests
// ========================================

#[test]
fn test_handle_event_unfocused() {
    let state = NumberInputState::new(0.0);
    let msg = NumberInput::handle_event(&state, &Event::key(KeyCode::Up), &ViewContext::default());
    assert_eq!(msg, None);
}

#[test]
fn test_handle_event_disabled() {
    let state = NumberInputState::new(0.0);
    let msg = NumberInput::handle_event(
        &state,
        &Event::key(KeyCode::Up),
        &ViewContext::new().focused(true).disabled(true),
    );
    assert_eq!(msg, None);
}

#[test]
fn test_handle_event_disabled_edit_mode() {
    let mut state = NumberInputState::new(0.0);
    state.editing = true;
    let msg = NumberInput::handle_event(
        &state,
        &Event::char('5'),
        &ViewContext::new().focused(true).disabled(true),
    );
    assert_eq!(msg, None);
}

// ========================================
// Dispatch Event Tests
// ========================================

#[test]
fn test_dispatch_event_increment() {
    let mut state = NumberInputState::new(0.0);
    let output = NumberInput::dispatch_event(
        &mut state,
        &Event::key(KeyCode::Up),
        &ViewContext::new().focused(true),
    );
    assert_eq!(output, Some(NumberInputOutput::ValueChanged(1.0)));
    assert_eq!(state.value(), 1.0);
}

#[test]
fn test_dispatch_event_unfocused() {
    let mut state = NumberInputState::new(0.0);
    let output = NumberInput::dispatch_event(
        &mut state,
        &Event::key(KeyCode::Up),
        &ViewContext::default(),
    );
    assert_eq!(output, None);
    assert_eq!(state.value(), 0.0);
}

#[test]
fn test_dispatch_event_enter_then_type() {
    let mut state = NumberInputState::new(0.0);

    // Enter edit mode
    let output = NumberInput::dispatch_event(
        &mut state,
        &Event::key(KeyCode::Enter),
        &ViewContext::new().focused(true),
    );
    assert_eq!(output, Some(NumberInputOutput::EditStarted));
    assert!(state.is_editing());

    // Type a character
    let output = NumberInput::dispatch_event(
        &mut state,
        &Event::char('5'),
        &ViewContext::new().focused(true),
    );
    assert_eq!(output, None);
    assert_eq!(state.edit_buffer(), "05");
}

// ========================================
// Instance Method Tests
// ========================================

#[test]
fn test_instance_handle_event() {
    let state = NumberInputState::new(0.0);
    let msg = NumberInput::handle_event(
        &state,
        &Event::key(KeyCode::Up),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(NumberInputMessage::Increment));
}

#[test]
fn test_instance_dispatch_event() {
    let mut state = NumberInputState::new(0.0);
    let output = NumberInput::dispatch_event(
        &mut state,
        &Event::key(KeyCode::Up),
        &ViewContext::new().focused(true),
    );
    assert_eq!(output, Some(NumberInputOutput::ValueChanged(1.0)));
}

#[test]
fn test_instance_update() {
    let mut state = NumberInputState::new(0.0);
    let output = state.update(NumberInputMessage::Increment);
    assert_eq!(output, Some(NumberInputOutput::ValueChanged(1.0)));
    assert_eq!(state.value(), 1.0);
}

// ========================================
// Init Test
// ========================================

#[test]
fn test_init() {
    let state = NumberInput::init();
    assert_eq!(state.value(), 0.0);
    assert_eq!(state.step(), 1.0);
    assert_eq!(state.precision(), 0);
}

// ========================================
// Edge Case Tests
// ========================================

#[test]
fn test_min_equals_max() {
    let state = NumberInputState::new(10.0).with_range(10.0, 10.0);
    assert_eq!(state.value(), 10.0);
}

#[test]
fn test_min_equals_max_increment() {
    let mut state = NumberInputState::new(10.0).with_range(10.0, 10.0);
    let output = NumberInput::update(&mut state, NumberInputMessage::Increment);
    assert_eq!(output, None);
    assert_eq!(state.value(), 10.0);
}

#[test]
fn test_min_equals_max_decrement() {
    let mut state = NumberInputState::new(10.0).with_range(10.0, 10.0);
    let output = NumberInput::update(&mut state, NumberInputMessage::Decrement);
    assert_eq!(output, None);
    assert_eq!(state.value(), 10.0);
}

#[test]
fn test_step_larger_than_range() {
    let mut state = NumberInputState::new(0.0)
        .with_range(0.0, 5.0)
        .with_step(10.0);
    let output = NumberInput::update(&mut state, NumberInputMessage::Increment);
    assert_eq!(output, Some(NumberInputOutput::ValueChanged(5.0)));
    assert_eq!(state.value(), 5.0);
}

#[test]
fn test_negative_values() {
    let mut state = NumberInputState::new(-5.0).with_range(-10.0, 10.0);
    let output = NumberInput::update(&mut state, NumberInputMessage::Decrement);
    assert_eq!(output, Some(NumberInputOutput::ValueChanged(-6.0)));
    assert_eq!(state.value(), -6.0);
}

#[test]
fn test_no_bounds_increment() {
    let mut state = NumberInputState::new(1_000_000.0);
    let output = NumberInput::update(&mut state, NumberInputMessage::Increment);
    assert_eq!(output, Some(NumberInputOutput::ValueChanged(1_000_001.0)));
}

#[test]
fn test_no_bounds_decrement() {
    let mut state = NumberInputState::new(-1_000_000.0);
    let output = NumberInput::update(&mut state, NumberInputMessage::Decrement);
    assert_eq!(output, Some(NumberInputOutput::ValueChanged(-1_000_001.0)));
}

#[test]
fn test_only_min_set() {
    let mut state = NumberInputState::new(5.0).with_min(0.0);
    // Can go up without limit
    for _ in 0..100 {
        NumberInput::update(&mut state, NumberInputMessage::Increment);
    }
    assert_eq!(state.value(), 105.0);
}

#[test]
fn test_only_max_set() {
    let mut state = NumberInputState::new(5.0).with_max(100.0);
    // Can go down without limit
    for _ in 0..200 {
        NumberInput::update(&mut state, NumberInputMessage::Decrement);
    }
    assert_eq!(state.value(), -195.0);
}

// ========================================
// Valid Numeric Char Tests
// ========================================

#[test]
fn test_is_valid_numeric_char_digit() {
    for d in '0'..='9' {
        assert!(is_valid_numeric_char(d, ""));
    }
}

#[test]
fn test_is_valid_numeric_char_decimal() {
    assert!(is_valid_numeric_char('.', "3"));
    assert!(!is_valid_numeric_char('.', "3.1"));
}

#[test]
fn test_is_valid_numeric_char_minus() {
    assert!(is_valid_numeric_char('-', ""));
    assert!(!is_valid_numeric_char('-', "3"));
}

#[test]
fn test_is_valid_numeric_char_letters() {
    assert!(!is_valid_numeric_char('a', ""));
    assert!(!is_valid_numeric_char('z', ""));
    assert!(!is_valid_numeric_char(' ', ""));
}
