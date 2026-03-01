use super::*;

// StatusBar component tests

#[test]
fn test_init() {
    let state = StatusBar::init();
    assert!(state.is_empty());
}

#[test]
fn test_set_left_items() {
    let mut state = StatusBarState::new();
    let items = vec![StatusBarItem::new("A"), StatusBarItem::new("B")];

    StatusBar::update(&mut state, StatusBarMessage::SetLeftItems(items));
    assert_eq!(state.left().len(), 2);
}

#[test]
fn test_set_center_items() {
    let mut state = StatusBarState::new();
    let items = vec![StatusBarItem::new("Center")];

    StatusBar::update(&mut state, StatusBarMessage::SetCenterItems(items));
    assert_eq!(state.center().len(), 1);
}

#[test]
fn test_set_right_items() {
    let mut state = StatusBarState::new();
    let items = vec![StatusBarItem::new("Right")];

    StatusBar::update(&mut state, StatusBarMessage::SetRightItems(items));
    assert_eq!(state.right().len(), 1);
}

#[test]
fn test_clear_message() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::new("L"));
    state.push_center(StatusBarItem::new("C"));
    state.push_right(StatusBarItem::new("R"));

    StatusBar::update(&mut state, StatusBarMessage::Clear);
    assert!(state.is_empty());
}

#[test]
fn test_clear_left_message() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::new("L"));
    state.push_center(StatusBarItem::new("C"));

    StatusBar::update(&mut state, StatusBarMessage::ClearLeft);
    assert!(state.left().is_empty());
    assert_eq!(state.center().len(), 1);
}

#[test]
fn test_clear_center_message() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::new("L"));
    state.push_center(StatusBarItem::new("C"));

    StatusBar::update(&mut state, StatusBarMessage::ClearCenter);
    assert_eq!(state.left().len(), 1);
    assert!(state.center().is_empty());
}

#[test]
fn test_clear_right_message() {
    let mut state = StatusBarState::new();
    state.push_right(StatusBarItem::new("R"));
    state.push_center(StatusBarItem::new("C"));

    StatusBar::update(&mut state, StatusBarMessage::ClearRight);
    assert!(state.right().is_empty());
    assert_eq!(state.center().len(), 1);
}

#[test]
fn test_update_returns_none() {
    let mut state = StatusBarState::new();
    let output = StatusBar::update(&mut state, StatusBarMessage::Clear);
    assert!(output.is_none());
}

#[test]
fn test_render_section_empty() {
    let theme = Theme::default();
    let spans = StatusBar::render_section(&[], " | ", &theme);
    assert!(spans.is_empty());
}

#[test]
fn test_render_section_single_item() {
    let theme = Theme::default();
    let items = vec![StatusBarItem::new("Test")];
    let spans = StatusBar::render_section(&items, " | ", &theme);
    assert_eq!(spans.len(), 1);
}

#[test]
fn test_render_section_multiple_items() {
    let theme = Theme::default();
    let items = vec![StatusBarItem::new("A"), StatusBarItem::new("B")];
    let spans = StatusBar::render_section(&items, " | ", &theme);
    // A + separator + B = 3 spans
    assert_eq!(spans.len(), 3);
}

// View tests

#[test]
fn test_view_empty() {
    let state = StatusBarState::new();

    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 1);

    terminal
        .draw(|frame| {
            StatusBar::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_left_only() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::new("LEFT"));

    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 1);

    terminal
        .draw(|frame| {
            StatusBar::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_right_only() {
    let mut state = StatusBarState::new();
    state.push_right(StatusBarItem::new("RIGHT"));

    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 1);

    terminal
        .draw(|frame| {
            StatusBar::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_center_only() {
    let mut state = StatusBarState::new();
    state.push_center(StatusBarItem::new("CENTER"));

    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 1);

    terminal
        .draw(|frame| {
            StatusBar::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_all_sections() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::new("LEFT"));
    state.push_center(StatusBarItem::new("CENTER"));
    state.push_right(StatusBarItem::new("RIGHT"));

    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 1);

    terminal
        .draw(|frame| {
            StatusBar::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_with_separator() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::new("A"));
    state.push_left(StatusBarItem::new("B"));

    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 1);

    terminal
        .draw(|frame| {
            StatusBar::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_custom_separator() {
    let mut state = StatusBarState::with_separator(" :: ");
    state.push_left(StatusBarItem::new("A"));
    state.push_left(StatusBarItem::new("B"));

    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 1);

    terminal
        .draw(|frame| {
            StatusBar::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_no_separator_on_last_item() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::new("A").with_separator(false));
    state.push_left(StatusBarItem::new("B"));

    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 1);

    terminal
        .draw(|frame| {
            StatusBar::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_styled_items() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::new("INFO").with_style(StatusBarStyle::Info));
    state.push_left(StatusBarItem::new("ERROR").with_style(StatusBarStyle::Error));

    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 1);

    terminal
        .draw(|frame| {
            StatusBar::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

// View tests for dynamic content

#[test]
fn test_view_elapsed_time() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::elapsed_time());

    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 1);

    terminal
        .draw(|frame| {
            StatusBar::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_counter_with_label() {
    let mut state = StatusBarState::new();
    state.push_right(StatusBarItem::counter().with_label("Files"));

    StatusBar::update(
        &mut state,
        StatusBarMessage::SetCounter {
            section: Section::Right,
            index: 0,
            value: 15,
        },
    );

    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 1);

    terminal
        .draw(|frame| {
            StatusBar::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_heartbeat() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::heartbeat());

    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 1);

    terminal
        .draw(|frame| {
            StatusBar::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}
