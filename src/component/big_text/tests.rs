use super::*;
use crate::component::test_utils;
use crate::input::Event;
use crossterm::event::KeyCode;

// =============================================================================
// Construction
// =============================================================================

#[test]
fn test_new() {
    let state = BigTextState::new("42");
    assert_eq!(state.text(), "42");
    assert_eq!(state.color(), None);
    assert_eq!(state.alignment(), Alignment::Center);
}

#[test]
fn test_default() {
    let state = BigTextState::default();
    assert_eq!(state.text(), "");
    assert_eq!(state.color(), None);
    assert_eq!(state.alignment(), Alignment::Center);
}

#[test]
fn test_init() {
    let state = BigText::init();
    assert_eq!(state.text(), "");
}

#[test]
fn test_with_color() {
    let state = BigTextState::new("99").with_color(Color::Green);
    assert_eq!(state.color(), Some(Color::Green));
}

#[test]
fn test_with_alignment() {
    let state = BigTextState::new("OK").with_alignment(Alignment::Left);
    assert_eq!(state.alignment(), Alignment::Left);
}

#[test]
fn test_chained_builders() {
    let state = BigTextState::new("100")
        .with_color(Color::Cyan)
        .with_alignment(Alignment::Right);
    assert_eq!(state.text(), "100");
    assert_eq!(state.color(), Some(Color::Cyan));
    assert_eq!(state.alignment(), Alignment::Right);
}

// =============================================================================
// Setters
// =============================================================================

#[test]
fn test_set_text() {
    let mut state = BigTextState::new("old");
    state.set_text("new");
    assert_eq!(state.text(), "new");
}

#[test]
fn test_set_color() {
    let mut state = BigTextState::new("0");
    state.set_color(Some(Color::Red));
    assert_eq!(state.color(), Some(Color::Red));
    state.set_color(None);
    assert_eq!(state.color(), None);
}

#[test]
fn test_set_alignment() {
    let mut state = BigTextState::new("0");
    state.set_alignment(Alignment::Right);
    assert_eq!(state.alignment(), Alignment::Right);
}

// =============================================================================
// Update messages
// =============================================================================

#[test]
fn test_update_set_text() {
    let mut state = BigTextState::new("old");
    let output = BigText::update(&mut state, BigTextMessage::SetText("new".to_string()));
    assert_eq!(state.text(), "new");
    assert_eq!(output, None);
}

#[test]
fn test_update_set_color() {
    let mut state = BigTextState::new("0");
    let output = BigText::update(&mut state, BigTextMessage::SetColor(Some(Color::Blue)));
    assert_eq!(state.color(), Some(Color::Blue));
    assert_eq!(output, None);
}

#[test]
fn test_update_set_color_none() {
    let mut state = BigTextState::new("0").with_color(Color::Red);
    let output = BigText::update(&mut state, BigTextMessage::SetColor(None));
    assert_eq!(state.color(), None);
    assert_eq!(output, None);
}

#[test]
fn test_update_set_alignment() {
    let mut state = BigTextState::new("0");
    let output = BigText::update(&mut state, BigTextMessage::SetAlignment(Alignment::Left));
    assert_eq!(state.alignment(), Alignment::Left);
    assert_eq!(output, None);
}

// =============================================================================
// Instance methods
// =============================================================================

#[test]
fn test_instance_update() {
    let mut state = BigTextState::new("old");
    state.update(BigTextMessage::SetText("new".to_string()));
    assert_eq!(state.text(), "new");
}

// =============================================================================
// handle_event (display-only, always None)
// =============================================================================

#[test]
fn test_handle_event_returns_none() {
    let state = BigTextState::new("42");
    assert_eq!(
        BigText::handle_event(&state, &Event::key(KeyCode::Up), &EventContext::default()),
        None
    );
    assert_eq!(
        BigText::handle_event(
            &state,
            &Event::key(KeyCode::Enter),
            &EventContext::default()
        ),
        None
    );
    assert_eq!(
        BigText::handle_event(&state, &Event::char('a'), &EventContext::default()),
        None
    );
}

// =============================================================================
// Font: big_char
// =============================================================================

#[test]
fn test_big_char_digits() {
    for digit in '0'..='9' {
        let rows = big_char(digit);
        assert_eq!(rows.len(), 3, "digit {} should have 3 rows", digit);
        // All rows for a digit should have the same display width
        let width_0 = unicode_width::UnicodeWidthStr::width(rows[0]);
        let width_1 = unicode_width::UnicodeWidthStr::width(rows[1]);
        let width_2 = unicode_width::UnicodeWidthStr::width(rows[2]);
        assert_eq!(
            width_0, width_1,
            "digit {} row 0 and 1 widths differ ({} vs {})",
            digit, width_0, width_1
        );
        assert_eq!(
            width_1, width_2,
            "digit {} row 1 and 2 widths differ ({} vs {})",
            digit, width_1, width_2
        );
    }
}

#[test]
fn test_big_char_punctuation() {
    for ch in ['.', ':', '-', '/', '%', ' '] {
        let rows = big_char(ch);
        assert_eq!(rows.len(), 3, "char '{}' should have 3 rows", ch);
        let width_0 = unicode_width::UnicodeWidthStr::width(rows[0]);
        let width_1 = unicode_width::UnicodeWidthStr::width(rows[1]);
        let width_2 = unicode_width::UnicodeWidthStr::width(rows[2]);
        assert_eq!(
            width_0, width_1,
            "char '{}' row 0 and 1 widths differ ({} vs {})",
            ch, width_0, width_1
        );
        assert_eq!(
            width_1, width_2,
            "char '{}' row 1 and 2 widths differ ({} vs {})",
            ch, width_1, width_2
        );
    }
}

#[test]
fn test_big_char_uppercase_letters() {
    for ch in 'A'..='Z' {
        let rows = big_char(ch);
        assert_eq!(rows.len(), 3, "letter {} should have 3 rows", ch);
        let width_0 = unicode_width::UnicodeWidthStr::width(rows[0]);
        let width_1 = unicode_width::UnicodeWidthStr::width(rows[1]);
        let width_2 = unicode_width::UnicodeWidthStr::width(rows[2]);
        assert_eq!(
            width_0, width_1,
            "letter {} row 0 and 1 widths differ ({} vs {})",
            ch, width_0, width_1
        );
        assert_eq!(
            width_1, width_2,
            "letter {} row 1 and 2 widths differ ({} vs {})",
            ch, width_1, width_2
        );
    }
}

#[test]
fn test_big_char_unsupported() {
    // Unsupported characters should return the placeholder
    let rows = big_char('~');
    assert_eq!(rows.len(), 3);
    // The placeholder should have consistent widths too
    let width_0 = unicode_width::UnicodeWidthStr::width(rows[0]);
    let width_1 = unicode_width::UnicodeWidthStr::width(rows[1]);
    let width_2 = unicode_width::UnicodeWidthStr::width(rows[2]);
    assert_eq!(width_0, width_1);
    assert_eq!(width_1, width_2);
}

#[test]
fn test_big_char_specific_digit_zero() {
    let rows = big_char('0');
    assert_eq!(rows[0], "█▀█");
    assert_eq!(rows[1], "█ █");
    assert_eq!(rows[2], "▀▀▀");
}

#[test]
fn test_big_char_specific_digit_one() {
    let rows = big_char('1');
    assert_eq!(rows[0], "▀█ ");
    assert_eq!(rows[1], " █ ");
    assert_eq!(rows[2], "▀▀▀");
}

#[test]
fn test_big_char_width_digits() {
    for digit in '0'..='9' {
        let width = big_char_width(digit);
        assert_eq!(width, 3, "digit {} should have width 3", digit);
    }
}

#[test]
fn test_big_char_width_period() {
    assert_eq!(big_char_width('.'), 1);
}

#[test]
fn test_big_char_width_colon() {
    assert_eq!(big_char_width(':'), 1);
}

#[test]
fn test_big_char_width_space() {
    assert_eq!(big_char_width(' '), 3);
}

// =============================================================================
// Edge cases
// =============================================================================

#[test]
fn test_empty_text() {
    let state = BigTextState::new("");
    let (mut terminal, theme) = test_utils::setup_render(40, 5);
    terminal
        .draw(|frame| {
            BigText::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_unsupported_characters() {
    let state = BigTextState::new("~@#");
    let (mut terminal, theme) = test_utils::setup_render(40, 5);
    terminal
        .draw(|frame| {
            BigText::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_zero_height_area() {
    let state = BigTextState::new("42");
    let (mut terminal, theme) = test_utils::setup_render(40, 5);
    terminal
        .draw(|frame| {
            let area = Rect::new(0, 0, 40, 0);
            BigText::view(&state, &mut RenderContext::new(frame, area, &theme));
        })
        .unwrap();
    // Should not panic
}

#[test]
fn test_zero_width_area() {
    let state = BigTextState::new("42");
    let (mut terminal, theme) = test_utils::setup_render(40, 5);
    terminal
        .draw(|frame| {
            let area = Rect::new(0, 0, 0, 5);
            BigText::view(&state, &mut RenderContext::new(frame, area, &theme));
        })
        .unwrap();
    // Should not panic
}

// =============================================================================
// Snapshot tests
// =============================================================================

#[test]
fn test_view_digits() {
    let state = BigTextState::new("0123456789");
    let (mut terminal, theme) = test_utils::setup_render(60, 5);
    terminal
        .draw(|frame| {
            BigText::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_clock() {
    let state = BigTextState::new("12:30");
    let (mut terminal, theme) = test_utils::setup_render(40, 5);
    terminal
        .draw(|frame| {
            BigText::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_percentage() {
    let state = BigTextState::new("99.9%");
    let (mut terminal, theme) = test_utils::setup_render(40, 5);
    terminal
        .draw(|frame| {
            BigText::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_with_color() {
    let state = BigTextState::new("42").with_color(Color::Cyan);
    let (mut terminal, theme) = test_utils::setup_render(30, 5);
    terminal
        .draw(|frame| {
            BigText::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_left_aligned() {
    let state = BigTextState::new("42").with_alignment(Alignment::Left);
    let (mut terminal, theme) = test_utils::setup_render(30, 5);
    terminal
        .draw(|frame| {
            BigText::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_right_aligned() {
    let state = BigTextState::new("42").with_alignment(Alignment::Right);
    let (mut terminal, theme) = test_utils::setup_render(30, 5);
    terminal
        .draw(|frame| {
            BigText::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_disabled() {
    let state = BigTextState::new("42");
    let (mut terminal, theme) = test_utils::setup_render(30, 5);
    terminal
        .draw(|frame| {
            BigText::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).disabled(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_letters() {
    let state = BigTextState::new("HELLO");
    let (mut terminal, theme) = test_utils::setup_render(40, 5);
    terminal
        .draw(|frame| {
            BigText::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_lowercase_converted() {
    // Lowercase should render same as uppercase
    let state = BigTextState::new("hello");
    let (mut terminal, theme) = test_utils::setup_render(40, 5);
    terminal
        .draw(|frame| {
            BigText::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_single_char() {
    let state = BigTextState::new("0");
    let (mut terminal, theme) = test_utils::setup_render(20, 5);
    terminal
        .draw(|frame| {
            BigText::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_dash() {
    let state = BigTextState::new("1-0");
    let (mut terminal, theme) = test_utils::setup_render(30, 5);
    terminal
        .draw(|frame| {
            BigText::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_date() {
    let state = BigTextState::new("03/28");
    let (mut terminal, theme) = test_utils::setup_render(40, 5);
    terminal
        .draw(|frame| {
            BigText::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

// =============================================================================
// Annotation tests
// =============================================================================

#[test]
fn test_annotation_emitted() {
    use crate::annotation::{WidgetType, with_annotations};
    let state = BigTextState::new("42");
    let (mut terminal, theme) = test_utils::setup_render(30, 5);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                BigText::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
            })
            .unwrap();
    });
    assert_eq!(registry.len(), 1);
    let regions = registry.find_by_type(&WidgetType::BigText);
    assert_eq!(regions.len(), 1);
    assert_eq!(regions[0].annotation.label, Some("42".to_string()));
    assert!(!regions[0].annotation.disabled);
}

#[test]
fn test_annotation_disabled() {
    use crate::annotation::{WidgetType, with_annotations};
    let state = BigTextState::new("OFF");
    let (mut terminal, theme) = test_utils::setup_render(30, 5);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                BigText::view(
                    &state,
                    &mut RenderContext::new(frame, frame.area(), &theme).disabled(true),
                );
            })
            .unwrap();
    });
    let regions = registry.find_by_type(&WidgetType::BigText);
    assert_eq!(regions.len(), 1);
    assert!(regions[0].annotation.disabled);
}
