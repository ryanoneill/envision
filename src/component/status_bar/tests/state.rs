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
