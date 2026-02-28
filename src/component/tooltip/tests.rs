use super::*;

// ========================================
// TooltipPosition Tests
// ========================================

#[test]
fn test_position_default() {
    let position = TooltipPosition::default();
    assert_eq!(position, TooltipPosition::Below);
}

#[test]
fn test_position_clone() {
    let position = TooltipPosition::Above;
    let cloned = position;
    assert_eq!(cloned, TooltipPosition::Above);
}

#[test]
fn test_position_eq() {
    assert_eq!(TooltipPosition::Above, TooltipPosition::Above);
    assert_ne!(TooltipPosition::Above, TooltipPosition::Below);
    assert_eq!(TooltipPosition::Left, TooltipPosition::Left);
    assert_ne!(TooltipPosition::Left, TooltipPosition::Right);
}

// ========================================
// State Creation Tests
// ========================================

#[test]
fn test_new() {
    let state = TooltipState::new("Test content");
    assert_eq!(state.content(), "Test content");
    assert_eq!(state.title(), None);
    assert_eq!(state.position(), TooltipPosition::Below);
    assert!(!state.is_visible());
    assert_eq!(state.duration_ms(), None);
}

#[test]
fn test_with_title() {
    let state = TooltipState::new("Content").with_title("My Title");
    assert_eq!(state.title(), Some("My Title"));
}

#[test]
fn test_with_position() {
    let state = TooltipState::new("Content").with_position(TooltipPosition::Above);
    assert_eq!(state.position(), TooltipPosition::Above);
}

#[test]
fn test_with_duration() {
    let state = TooltipState::new("Content").with_duration(5000);
    assert_eq!(state.duration_ms(), Some(5000));
}

#[test]
fn test_with_fg_color() {
    let state = TooltipState::new("Content").with_fg_color(Color::Yellow);
    assert_eq!(state.fg_color(), Color::Yellow);
}

#[test]
fn test_with_bg_color() {
    let state = TooltipState::new("Content").with_bg_color(Color::DarkGray);
    assert_eq!(state.bg_color(), Color::DarkGray);
}

#[test]
fn test_with_border_color() {
    let state = TooltipState::new("Content").with_border_color(Color::Cyan);
    assert_eq!(state.border_color(), Color::Cyan);
}

#[test]
fn test_default() {
    let state = TooltipState::default();
    assert_eq!(state.content(), "");
    assert_eq!(state.title(), None);
    assert_eq!(state.position(), TooltipPosition::Below);
    assert!(!state.is_visible());
    assert_eq!(state.duration_ms(), None);
    assert_eq!(state.remaining_ms(), None);
    assert_eq!(state.fg_color(), Color::White);
    assert_eq!(state.bg_color(), Color::Black);
    assert_eq!(state.border_color(), Color::Gray);
}

#[test]
fn test_builder_chain() {
    let state = TooltipState::new("Content")
        .with_title("Title")
        .with_position(TooltipPosition::Left)
        .with_duration(3000)
        .with_fg_color(Color::Red)
        .with_bg_color(Color::Blue)
        .with_border_color(Color::Green);

    assert_eq!(state.content(), "Content");
    assert_eq!(state.title(), Some("Title"));
    assert_eq!(state.position(), TooltipPosition::Left);
    assert_eq!(state.duration_ms(), Some(3000));
    assert_eq!(state.fg_color(), Color::Red);
    assert_eq!(state.bg_color(), Color::Blue);
    assert_eq!(state.border_color(), Color::Green);
}

// ========================================
// Accessor Tests
// ========================================

#[test]
fn test_content() {
    let state = TooltipState::new("My content");
    assert_eq!(state.content(), "My content");
}

#[test]
fn test_title() {
    let state = TooltipState::new("Content").with_title("Header");
    assert_eq!(state.title(), Some("Header"));
}

#[test]
fn test_position() {
    let state = TooltipState::new("Content").with_position(TooltipPosition::Right);
    assert_eq!(state.position(), TooltipPosition::Right);
}

#[test]
fn test_is_visible() {
    let state = TooltipState::new("Content");
    assert!(!state.is_visible());
}

#[test]
fn test_duration_ms() {
    let state = TooltipState::new("Content").with_duration(2000);
    assert_eq!(state.duration_ms(), Some(2000));
}

#[test]
fn test_remaining_ms() {
    let state = TooltipState::new("Content");
    assert_eq!(state.remaining_ms(), None);
}

#[test]
fn test_fg_color() {
    let state = TooltipState::new("Content").with_fg_color(Color::Magenta);
    assert_eq!(state.fg_color(), Color::Magenta);
}

#[test]
fn test_bg_color() {
    let state = TooltipState::new("Content").with_bg_color(Color::LightBlue);
    assert_eq!(state.bg_color(), Color::LightBlue);
}

#[test]
fn test_border_color() {
    let state = TooltipState::new("Content").with_border_color(Color::LightGreen);
    assert_eq!(state.border_color(), Color::LightGreen);
}

// ========================================
// Mutator Tests
// ========================================

#[test]
fn test_set_content() {
    let mut state = TooltipState::new("Old");
    state.set_content("New");
    assert_eq!(state.content(), "New");
}

#[test]
fn test_set_title() {
    let mut state = TooltipState::new("Content");
    state.set_title(Some("New Title".to_string()));
    assert_eq!(state.title(), Some("New Title"));

    state.set_title(None);
    assert_eq!(state.title(), None);
}

#[test]
fn test_set_position() {
    let mut state = TooltipState::new("Content");
    state.set_position(TooltipPosition::Above);
    assert_eq!(state.position(), TooltipPosition::Above);
}

#[test]
fn test_set_duration() {
    let mut state = TooltipState::new("Content");
    state.set_duration(Some(4000));
    assert_eq!(state.duration_ms(), Some(4000));

    state.set_duration(None);
    assert_eq!(state.duration_ms(), None);
}

#[test]
fn test_set_fg_color() {
    let mut state = TooltipState::new("Content");
    state.set_fg_color(Color::Rgb(100, 150, 200));
    assert_eq!(state.fg_color(), Color::Rgb(100, 150, 200));
}

#[test]
fn test_set_bg_color() {
    let mut state = TooltipState::new("Content");
    state.set_bg_color(Color::Indexed(42));
    assert_eq!(state.bg_color(), Color::Indexed(42));
}

#[test]
fn test_set_border_color() {
    let mut state = TooltipState::new("Content");
    state.set_border_color(Color::LightRed);
    assert_eq!(state.border_color(), Color::LightRed);
}

// ========================================
// Show/Hide Tests
// ========================================

#[test]
fn test_show() {
    let mut state = TooltipState::new("Content");
    Tooltip::update(&mut state, TooltipMessage::Show);
    assert!(state.is_visible());
}

#[test]
fn test_show_returns_shown() {
    let mut state = TooltipState::new("Content");
    let output = Tooltip::update(&mut state, TooltipMessage::Show);
    assert_eq!(output, Some(TooltipOutput::Shown));
}

#[test]
fn test_show_already_visible() {
    let mut state = TooltipState::new("Content");
    Tooltip::update(&mut state, TooltipMessage::Show);

    let output = Tooltip::update(&mut state, TooltipMessage::Show);
    assert_eq!(output, None);
}

#[test]
fn test_hide() {
    let mut state = TooltipState::new("Content");
    Tooltip::update(&mut state, TooltipMessage::Show);
    Tooltip::update(&mut state, TooltipMessage::Hide);
    assert!(!state.is_visible());
}

#[test]
fn test_hide_returns_hidden() {
    let mut state = TooltipState::new("Content");
    Tooltip::update(&mut state, TooltipMessage::Show);

    let output = Tooltip::update(&mut state, TooltipMessage::Hide);
    assert_eq!(output, Some(TooltipOutput::Hidden));
}

#[test]
fn test_hide_already_hidden() {
    let mut state = TooltipState::new("Content");
    let output = Tooltip::update(&mut state, TooltipMessage::Hide);
    assert_eq!(output, None);
}

#[test]
fn test_toggle_show() {
    let mut state = TooltipState::new("Content");
    let output = Tooltip::update(&mut state, TooltipMessage::Toggle);
    assert!(state.is_visible());
    assert_eq!(output, Some(TooltipOutput::Shown));
}

#[test]
fn test_toggle_hide() {
    let mut state = TooltipState::new("Content");
    Tooltip::update(&mut state, TooltipMessage::Show);

    let output = Tooltip::update(&mut state, TooltipMessage::Toggle);
    assert!(!state.is_visible());
    assert_eq!(output, Some(TooltipOutput::Hidden));
}

// ========================================
// Auto-hide Tests
// ========================================

#[test]
fn test_show_sets_remaining() {
    let mut state = TooltipState::new("Content").with_duration(3000);
    Tooltip::update(&mut state, TooltipMessage::Show);
    assert_eq!(state.remaining_ms(), Some(3000));
}

#[test]
fn test_tick_decrements() {
    let mut state = TooltipState::new("Content").with_duration(3000);
    Tooltip::update(&mut state, TooltipMessage::Show);

    Tooltip::update(&mut state, TooltipMessage::Tick(1000));
    assert_eq!(state.remaining_ms(), Some(2000));
}

#[test]
fn test_tick_expires() {
    let mut state = TooltipState::new("Content").with_duration(1000);
    Tooltip::update(&mut state, TooltipMessage::Show);

    Tooltip::update(&mut state, TooltipMessage::Tick(1000));
    assert!(!state.is_visible());
}

#[test]
fn test_tick_returns_expired() {
    let mut state = TooltipState::new("Content").with_duration(1000);
    Tooltip::update(&mut state, TooltipMessage::Show);

    let output = Tooltip::update(&mut state, TooltipMessage::Tick(1000));
    assert_eq!(output, Some(TooltipOutput::Expired));
}

#[test]
fn test_tick_no_duration() {
    let mut state = TooltipState::new("Content");
    Tooltip::update(&mut state, TooltipMessage::Show);

    let output = Tooltip::update(&mut state, TooltipMessage::Tick(10000));
    assert_eq!(output, None);
    assert!(state.is_visible());
}

#[test]
fn test_tick_not_visible() {
    let mut state = TooltipState::new("Content").with_duration(1000);
    // Don't show - state is not visible

    let output = Tooltip::update(&mut state, TooltipMessage::Tick(100));
    assert_eq!(output, None);
}

#[test]
fn test_hide_clears_remaining() {
    let mut state = TooltipState::new("Content").with_duration(3000);
    Tooltip::update(&mut state, TooltipMessage::Show);
    assert_eq!(state.remaining_ms(), Some(3000));

    Tooltip::update(&mut state, TooltipMessage::Hide);
    assert_eq!(state.remaining_ms(), None);
}

// ========================================
// SetContent/SetPosition Message Tests
// ========================================

#[test]
fn test_set_content_message() {
    let mut state = TooltipState::new("Old");
    Tooltip::update(&mut state, TooltipMessage::SetContent("New".into()));
    assert_eq!(state.content(), "New");
}

#[test]
fn test_set_position_message() {
    let mut state = TooltipState::new("Content");
    Tooltip::update(
        &mut state,
        TooltipMessage::SetPosition(TooltipPosition::Left),
    );
    assert_eq!(state.position(), TooltipPosition::Left);
}

// ========================================
// Toggleable Trait Tests
// ========================================

#[test]
fn test_toggleable_is_visible() {
    let state = TooltipState::new("Content");
    assert!(!Tooltip::is_visible(&state));
}

#[test]
fn test_toggleable_set_visible() {
    let mut state = TooltipState::new("Content").with_duration(3000);
    Tooltip::set_visible(&mut state, true);
    assert!(Tooltip::is_visible(&state));
    assert_eq!(state.remaining_ms(), Some(3000));

    Tooltip::set_visible(&mut state, false);
    assert!(!Tooltip::is_visible(&state));
    assert_eq!(state.remaining_ms(), None);
}

#[test]
fn test_toggleable_show() {
    let mut state = TooltipState::new("Content");
    Tooltip::show(&mut state);
    assert!(Tooltip::is_visible(&state));
}

#[test]
fn test_toggleable_hide() {
    let mut state = TooltipState::new("Content");
    Tooltip::show(&mut state);
    Tooltip::hide(&mut state);
    assert!(!Tooltip::is_visible(&state));
}

// ========================================
// Position Calculation Tests
// ========================================

#[test]
fn test_position_below() {
    let state = TooltipState::new("Test").with_position(TooltipPosition::Below);
    let target = Rect::new(10, 5, 20, 3);
    let bounds = Rect::new(0, 0, 80, 24);

    let area = calculate_tooltip_area(&state, target, bounds);
    assert_eq!(area.y, target.bottom());
}

#[test]
fn test_position_above() {
    let state = TooltipState::new("Test").with_position(TooltipPosition::Above);
    let target = Rect::new(10, 10, 20, 3);
    let bounds = Rect::new(0, 0, 80, 24);

    let area = calculate_tooltip_area(&state, target, bounds);
    assert!(area.bottom() <= target.y);
}

#[test]
fn test_position_left() {
    let state = TooltipState::new("Test").with_position(TooltipPosition::Left);
    let target = Rect::new(20, 5, 20, 3);
    let bounds = Rect::new(0, 0, 80, 24);

    let area = calculate_tooltip_area(&state, target, bounds);
    assert!(area.right() <= target.x);
}

#[test]
fn test_position_right() {
    let state = TooltipState::new("Test").with_position(TooltipPosition::Right);
    let target = Rect::new(10, 5, 20, 3);
    let bounds = Rect::new(0, 0, 80, 24);

    let area = calculate_tooltip_area(&state, target, bounds);
    assert_eq!(area.x, target.right());
}

#[test]
fn test_position_below_fallback() {
    let state = TooltipState::new("Test").with_position(TooltipPosition::Below);
    // Target at the bottom - no room below
    let target = Rect::new(10, 20, 20, 3);
    let bounds = Rect::new(0, 0, 80, 24);

    let area = calculate_tooltip_area(&state, target, bounds);
    // Should fall back to above
    assert!(area.bottom() <= target.y);
}

#[test]
fn test_position_above_fallback() {
    let state = TooltipState::new("Test").with_position(TooltipPosition::Above);
    // Target at the top - no room above
    let target = Rect::new(10, 0, 20, 3);
    let bounds = Rect::new(0, 0, 80, 24);

    let area = calculate_tooltip_area(&state, target, bounds);
    // Should fall back to below
    assert!(area.y >= target.bottom());
}

#[test]
fn test_position_left_fallback() {
    let state = TooltipState::new("Test").with_position(TooltipPosition::Left);
    // Target at the left edge - no room left
    let target = Rect::new(0, 5, 20, 3);
    let bounds = Rect::new(0, 0, 80, 24);

    let area = calculate_tooltip_area(&state, target, bounds);
    // Should fall back to right
    assert!(area.x >= target.right());
}

#[test]
fn test_position_right_fallback() {
    let state = TooltipState::new("Test").with_position(TooltipPosition::Right);
    // Target at the right edge - no room right
    let target = Rect::new(70, 5, 10, 3);
    let bounds = Rect::new(0, 0, 80, 24);

    let area = calculate_tooltip_area(&state, target, bounds);
    // Should fall back to left
    assert!(area.right() <= target.x);
}

// ========================================
// View Tests
// ========================================

#[test]
fn test_view_hidden() {
    let state = TooltipState::new("Content");

    let (mut terminal, _theme) = crate::component::test_utils::setup_render(80, 24);

    terminal
        .draw(|frame| {
            let target = Rect::new(10, 5, 20, 3);
            Tooltip::view_at(&state, frame, target, frame.area());
        })
        .unwrap();

    let output = terminal.backend().to_string();
    assert!(!output.contains("Content"));
}

#[test]
fn test_view_empty_content() {
    let mut state = TooltipState::new("");
    Tooltip::show(&mut state);

    let (mut terminal, _theme) = crate::component::test_utils::setup_render(80, 24);

    terminal
        .draw(|frame| {
            let target = Rect::new(10, 5, 20, 3);
            Tooltip::view_at(&state, frame, target, frame.area());
        })
        .unwrap();

    // Should render nothing for empty content
    let output = terminal.backend().to_string();
    assert!(output.trim().is_empty());
}

#[test]
fn test_view_visible() {
    let mut state = TooltipState::new("Helpful tooltip");
    Tooltip::show(&mut state);

    let (mut terminal, _theme) = crate::component::test_utils::setup_render(80, 24);

    terminal
        .draw(|frame| {
            let target = Rect::new(10, 5, 20, 3);
            Tooltip::view_at(&state, frame, target, frame.area());
        })
        .unwrap();

    let output = terminal.backend().to_string();
    assert!(output.contains("Helpful tooltip"));
}

#[test]
fn test_view_with_title() {
    let mut state = TooltipState::new("Content").with_title("Info");
    Tooltip::show(&mut state);

    let (mut terminal, _theme) = crate::component::test_utils::setup_render(80, 24);

    terminal
        .draw(|frame| {
            let target = Rect::new(10, 5, 20, 3);
            Tooltip::view_at(&state, frame, target, frame.area());
        })
        .unwrap();

    let output = terminal.backend().to_string();
    assert!(output.contains("Info"));
    assert!(output.contains("Content"));
}

#[test]
fn test_view_multiline() {
    let mut state = TooltipState::new("Line 1\nLine 2\nLine 3");
    Tooltip::show(&mut state);

    let (mut terminal, _theme) = crate::component::test_utils::setup_render(80, 24);

    terminal
        .draw(|frame| {
            let target = Rect::new(10, 5, 20, 3);
            Tooltip::view_at(&state, frame, target, frame.area());
        })
        .unwrap();

    let output = terminal.backend().to_string();
    assert!(output.contains("Line 1"));
    assert!(output.contains("Line 2"));
    assert!(output.contains("Line 3"));
}

// ========================================
// Integration Tests
// ========================================

#[test]
fn test_clone() {
    let state = TooltipState::new("Content")
        .with_title("Title")
        .with_position(TooltipPosition::Above)
        .with_duration(5000)
        .with_fg_color(Color::Yellow);

    let cloned = state.clone();
    assert_eq!(cloned.content(), "Content");
    assert_eq!(cloned.title(), Some("Title"));
    assert_eq!(cloned.position(), TooltipPosition::Above);
    assert_eq!(cloned.duration_ms(), Some(5000));
    assert_eq!(cloned.fg_color(), Color::Yellow);
}

#[test]
fn test_init() {
    let state = Tooltip::init();
    assert_eq!(state.content(), "");
    assert!(!state.is_visible());
}

#[test]
fn test_full_workflow() {
    let mut state = TooltipState::new("Click to submit").with_duration(3000);

    // Show
    let output = Tooltip::update(&mut state, TooltipMessage::Show);
    assert_eq!(output, Some(TooltipOutput::Shown));
    assert!(state.is_visible());
    assert_eq!(state.remaining_ms(), Some(3000));

    // Tick partial
    Tooltip::update(&mut state, TooltipMessage::Tick(1000));
    assert_eq!(state.remaining_ms(), Some(2000));
    assert!(state.is_visible());

    // Tick more
    Tooltip::update(&mut state, TooltipMessage::Tick(1000));
    assert_eq!(state.remaining_ms(), Some(1000));

    // Tick to expire
    let output = Tooltip::update(&mut state, TooltipMessage::Tick(1000));
    assert_eq!(output, Some(TooltipOutput::Expired));
    assert!(!state.is_visible());
    assert_eq!(state.remaining_ms(), None);
}
