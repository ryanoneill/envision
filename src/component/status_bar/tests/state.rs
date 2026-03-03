use super::*;

// StatusBarState tests

#[test]
fn test_state_new() {
    let state = StatusBarState::new();
    assert!(state.left().is_empty());
    assert!(state.center().is_empty());
    assert!(state.right().is_empty());
    assert_eq!(state.separator(), " | ");
    assert!(state.is_empty());
    assert_eq!(state.len(), 0);
}

#[test]
fn test_state_default() {
    let state = StatusBarState::default();
    assert!(state.is_empty());
}

#[test]
fn test_state_with_separator() {
    let state = StatusBarState::with_separator(" :: ");
    assert_eq!(state.separator(), " :: ");
}

#[test]
fn test_state_push_left() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::new("A"));
    state.push_left(StatusBarItem::new("B"));
    assert_eq!(state.left().len(), 2);
    assert_eq!(state.left()[0].text(), "A");
    assert_eq!(state.left()[1].text(), "B");
}

#[test]
fn test_state_push_center() {
    let mut state = StatusBarState::new();
    state.push_center(StatusBarItem::new("Center"));
    assert_eq!(state.center().len(), 1);
}

#[test]
fn test_state_push_right() {
    let mut state = StatusBarState::new();
    state.push_right(StatusBarItem::new("Right"));
    assert_eq!(state.right().len(), 1);
}

#[test]
fn test_state_clear() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::new("L"));
    state.push_center(StatusBarItem::new("C"));
    state.push_right(StatusBarItem::new("R"));
    assert_eq!(state.len(), 3);

    state.clear();
    assert!(state.is_empty());
    assert_eq!(state.len(), 0);
}

// Disabled tests

#[test]
fn test_with_disabled() {
    let state = StatusBarState::new().with_disabled(true);
    assert!(state.is_disabled());
}

#[test]
fn test_with_disabled_false() {
    let state = StatusBarState::new().with_disabled(false);
    assert!(!state.is_disabled());
}

#[test]
fn test_disabled_default_is_false() {
    let state = StatusBarState::new();
    assert!(!state.is_disabled());
}

#[test]
fn test_set_disabled() {
    let mut state = StatusBarState::new();
    assert!(!state.is_disabled());

    state.set_disabled(true);
    assert!(state.is_disabled());

    state.set_disabled(false);
    assert!(!state.is_disabled());
}

// Section tests

#[test]
fn test_section_enum() {
    assert_ne!(Section::Left, Section::Center);
    assert_ne!(Section::Center, Section::Right);
    assert_ne!(Section::Left, Section::Right);
}

#[test]
fn test_state_section() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::new("L"));
    state.push_center(StatusBarItem::new("C"));
    state.push_right(StatusBarItem::new("R"));

    assert_eq!(state.section(Section::Left).len(), 1);
    assert_eq!(state.section(Section::Center).len(), 1);
    assert_eq!(state.section(Section::Right).len(), 1);
}

#[test]
fn test_state_section_mut() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::new("L"));

    state
        .section_mut(Section::Left)
        .push(StatusBarItem::new("L2"));
    assert_eq!(state.section(Section::Left).len(), 2);
}

#[test]
fn test_state_get_item_mut() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::new("Test"));

    let item = state.get_item_mut(Section::Left, 0);
    assert!(item.is_some());
    item.unwrap().set_text("Updated");
    assert_eq!(state.left()[0].text(), "Updated");
}

#[test]
fn test_state_get_item_mut_invalid_index() {
    let mut state = StatusBarState::new();
    assert!(state.get_item_mut(Section::Left, 0).is_none());
}

// Integration tests

#[test]
fn test_typical_editor_status_bar() {
    let mut state = StatusBarState::new();

    // Left: mode indicator
    state.push_left(StatusBarItem::new("NORMAL").with_style(StatusBarStyle::Info));

    // Center: filename
    state.push_center(StatusBarItem::new("main.rs"));
    state.push_center(StatusBarItem::new("[+]").with_style(StatusBarStyle::Warning));

    // Right: position info
    state.push_right(StatusBarItem::new("UTF-8").with_style(StatusBarStyle::Muted));
    state.push_right(StatusBarItem::new("Ln 42, Col 8"));

    assert_eq!(state.left().len(), 1);
    assert_eq!(state.center().len(), 2);
    assert_eq!(state.right().len(), 2);
    assert_eq!(state.len(), 5);
}

#[test]
fn test_update_mode_indicator() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::new("NORMAL").with_style(StatusBarStyle::Info));

    // Simulate mode change
    StatusBar::update(
        &mut state,
        StatusBarMessage::SetLeftItems(vec![
            StatusBarItem::new("INSERT").with_style(StatusBarStyle::Success)
        ]),
    );

    assert_eq!(state.left().len(), 1);
    assert_eq!(state.left()[0].text(), "INSERT");
    assert_eq!(state.left()[0].style(), StatusBarStyle::Success);
}

// Dynamic content integration tests

#[test]
fn test_media_player_status_bar() {
    let mut state = StatusBarState::new();

    // Left: elapsed time
    state.push_left(StatusBarItem::elapsed_time().with_style(StatusBarStyle::Info));

    // Center: file name
    state.push_center(StatusBarItem::new("song.mp3"));

    // Right: heartbeat for activity
    state.push_right(StatusBarItem::heartbeat());

    assert_eq!(state.len(), 3);
    assert!(state.left()[0].is_dynamic());
}

#[test]
fn test_file_processor_status_bar() {
    let mut state = StatusBarState::new();

    // Left: timer for processing
    state.push_left(StatusBarItem::elapsed_time_long());

    // Center: file count
    state.push_center(StatusBarItem::counter().with_label("Processed"));

    // Right: remaining count
    state.push_right(StatusBarItem::counter().with_label("Remaining"));

    // Simulate processing
    StatusBar::update(
        &mut state,
        StatusBarMessage::StartTimer {
            section: Section::Left,
            index: 0,
        },
    );
    StatusBar::update(
        &mut state,
        StatusBarMessage::SetCounter {
            section: Section::Right,
            index: 0,
            value: 100,
        },
    );

    // Process one file
    StatusBar::update(
        &mut state,
        StatusBarMessage::IncrementCounter {
            section: Section::Center,
            index: 0,
        },
    );
    StatusBar::update(
        &mut state,
        StatusBarMessage::DecrementCounter {
            section: Section::Right,
            index: 0,
        },
    );

    assert_eq!(state.center()[0].text(), "Processed: 1");
    assert_eq!(state.right()[0].text(), "Remaining: 99");
}

// Set section tests

#[test]
fn test_set_left() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::new("Old"));
    assert_eq!(state.left().len(), 1);

    state.set_left(vec![StatusBarItem::new("A"), StatusBarItem::new("B")]);
    assert_eq!(state.left().len(), 2);
    assert_eq!(state.left()[0].text(), "A");
    assert_eq!(state.left()[1].text(), "B");
}

#[test]
fn test_set_center() {
    let mut state = StatusBarState::new();
    state.push_center(StatusBarItem::new("Old"));

    state.set_center(vec![StatusBarItem::new("New")]);
    assert_eq!(state.center().len(), 1);
    assert_eq!(state.center()[0].text(), "New");
}

#[test]
fn test_set_right() {
    let mut state = StatusBarState::new();
    state.push_right(StatusBarItem::new("Old"));

    state.set_right(vec![StatusBarItem::new("X"), StatusBarItem::new("Y")]);
    assert_eq!(state.right().len(), 2);
    assert_eq!(state.right()[0].text(), "X");
}

#[test]
fn test_set_left_to_empty() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::new("A"));
    state.set_left(vec![]);
    assert!(state.left().is_empty());
}

// Separator tests

#[test]
fn test_set_separator() {
    let mut state = StatusBarState::new();
    assert_eq!(state.separator(), " | ");

    state.set_separator(" :: ");
    assert_eq!(state.separator(), " :: ");
}

#[test]
fn test_set_separator_empty_string() {
    let mut state = StatusBarState::new();
    state.set_separator("");
    assert_eq!(state.separator(), "");
}

#[test]
fn test_with_separator_preserves_other_defaults() {
    let state = StatusBarState::with_separator(" - ");
    assert_eq!(state.separator(), " - ");
    assert!(state.is_empty());
    assert!(!state.is_disabled());
    assert_eq!(state.background(), Color::DarkGray);
}

// Background tests

#[test]
fn test_background_default() {
    let state = StatusBarState::new();
    assert_eq!(state.background(), Color::DarkGray);
}

#[test]
fn test_set_background() {
    let mut state = StatusBarState::new();
    state.set_background(Color::Blue);
    assert_eq!(state.background(), Color::Blue);
}

#[test]
fn test_set_background_multiple_times() {
    let mut state = StatusBarState::new();
    state.set_background(Color::Red);
    assert_eq!(state.background(), Color::Red);
    state.set_background(Color::Green);
    assert_eq!(state.background(), Color::Green);
}

// Length and empty tests

#[test]
fn test_len_single_section() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::new("A"));
    assert_eq!(state.len(), 1);
    assert!(!state.is_empty());
}

#[test]
fn test_len_all_sections() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::new("L1"));
    state.push_left(StatusBarItem::new("L2"));
    state.push_center(StatusBarItem::new("C1"));
    state.push_right(StatusBarItem::new("R1"));
    state.push_right(StatusBarItem::new("R2"));
    state.push_right(StatusBarItem::new("R3"));
    assert_eq!(state.len(), 6);
}

#[test]
fn test_is_empty_with_only_center_items() {
    let mut state = StatusBarState::new();
    state.push_center(StatusBarItem::new("C"));
    assert!(!state.is_empty());
}

#[test]
fn test_is_empty_with_only_right_items() {
    let mut state = StatusBarState::new();
    state.push_right(StatusBarItem::new("R"));
    assert!(!state.is_empty());
}

#[test]
fn test_is_empty_after_clear() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::new("L"));
    state.push_center(StatusBarItem::new("C"));
    state.push_right(StatusBarItem::new("R"));
    state.clear();
    assert!(state.is_empty());
    assert_eq!(state.len(), 0);
}

// Section accessor tests with all variants

#[test]
fn test_section_accessor_center() {
    let mut state = StatusBarState::new();
    state.push_center(StatusBarItem::new("C1"));
    state.push_center(StatusBarItem::new("C2"));
    assert_eq!(state.section(Section::Center).len(), 2);
    assert_eq!(state.section(Section::Center)[0].text(), "C1");
}

#[test]
fn test_section_accessor_right() {
    let mut state = StatusBarState::new();
    state.push_right(StatusBarItem::new("R1"));
    assert_eq!(state.section(Section::Right).len(), 1);
    assert_eq!(state.section(Section::Right)[0].text(), "R1");
}

#[test]
fn test_section_mut_center() {
    let mut state = StatusBarState::new();
    state
        .section_mut(Section::Center)
        .push(StatusBarItem::new("C"));
    assert_eq!(state.center().len(), 1);
    assert_eq!(state.center()[0].text(), "C");
}

#[test]
fn test_section_mut_right() {
    let mut state = StatusBarState::new();
    state
        .section_mut(Section::Right)
        .push(StatusBarItem::new("R"));
    assert_eq!(state.right().len(), 1);
    assert_eq!(state.right()[0].text(), "R");
}

// get_item_mut tests with all sections

#[test]
fn test_get_item_mut_center() {
    let mut state = StatusBarState::new();
    state.push_center(StatusBarItem::new("Original"));

    let item = state.get_item_mut(Section::Center, 0).unwrap();
    item.set_text("Modified");
    assert_eq!(state.center()[0].text(), "Modified");
}

#[test]
fn test_get_item_mut_right() {
    let mut state = StatusBarState::new();
    state.push_right(StatusBarItem::new("Original"));

    let item = state.get_item_mut(Section::Right, 0).unwrap();
    item.set_text("Modified");
    assert_eq!(state.right()[0].text(), "Modified");
}

#[test]
fn test_get_item_mut_invalid_index_center() {
    let mut state = StatusBarState::new();
    assert!(state.get_item_mut(Section::Center, 0).is_none());
}

#[test]
fn test_get_item_mut_invalid_index_right() {
    let mut state = StatusBarState::new();
    assert!(state.get_item_mut(Section::Right, 0).is_none());
}

#[test]
fn test_get_item_mut_out_of_bounds() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::new("A"));
    assert!(state.get_item_mut(Section::Left, 1).is_none());
}

// Default vs new

#[test]
fn test_default_vs_new() {
    let default_state = StatusBarState::default();
    let new_state = StatusBarState::new();

    // Both are empty
    assert_eq!(default_state.left().len(), new_state.left().len());
    assert_eq!(default_state.center().len(), new_state.center().len());
    assert_eq!(default_state.right().len(), new_state.right().len());
    assert_eq!(default_state.is_disabled(), new_state.is_disabled());
    assert_eq!(default_state.is_empty(), new_state.is_empty());
    assert_eq!(default_state.len(), new_state.len());

    // Default uses derive(Default), which gives empty separator
    // new() explicitly sets separator to " | "
    assert_eq!(default_state.separator(), "");
    assert_eq!(new_state.separator(), " | ");
}

// Clone and Debug

#[test]
fn test_state_clone() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::new("A"));
    state.push_center(StatusBarItem::new("B"));
    state.set_separator(" - ");
    state.set_background(Color::Blue);
    state.set_disabled(true);

    let cloned = state.clone();
    assert_eq!(cloned.left().len(), 1);
    assert_eq!(cloned.left()[0].text(), "A");
    assert_eq!(cloned.center().len(), 1);
    assert_eq!(cloned.separator(), " - ");
    assert_eq!(cloned.background(), Color::Blue);
    assert!(cloned.is_disabled());
}

#[test]
fn test_state_debug() {
    let state = StatusBarState::new();
    let debug_str = format!("{:?}", state);
    assert!(debug_str.contains("StatusBarState"));
}

// Section enum tests

#[test]
fn test_section_clone() {
    let section = Section::Left;
    let cloned = section;
    assert_eq!(section, cloned);
}

#[test]
fn test_section_debug() {
    assert_eq!(format!("{:?}", Section::Left), "Left");
    assert_eq!(format!("{:?}", Section::Center), "Center");
    assert_eq!(format!("{:?}", Section::Right), "Right");
}

#[test]
fn test_section_copy() {
    let section = Section::Right;
    let copied = section;
    assert_eq!(section, copied);
    // Both still usable after copy
    assert_eq!(section, Section::Right);
    assert_eq!(copied, Section::Right);
}
