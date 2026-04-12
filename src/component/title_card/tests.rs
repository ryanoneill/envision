use super::*;
use crate::component::test_utils;

// =============================================================================
// Construction
// =============================================================================

#[test]
fn test_new() {
    let state = TitleCardState::new("My App");
    assert_eq!(state.title(), "My App");
    assert_eq!(state.subtitle(), None);
    assert_eq!(state.prefix(), None);
    assert_eq!(state.suffix(), None);
    assert!(state.is_bordered());
    assert!(!state.is_disabled());
}

#[test]
fn test_default() {
    let state = TitleCardState::default();
    assert_eq!(state.title(), "");
    assert!(state.is_bordered());
}

#[test]
fn test_default_title_style() {
    let state = TitleCardState::new("Test");
    let expected = Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);
    assert_eq!(state.title_style(), expected);
}

#[test]
fn test_default_subtitle_style() {
    let state = TitleCardState::new("Test");
    let expected = Style::default().fg(Color::DarkGray);
    assert_eq!(state.subtitle_style(), expected);
}

#[test]
fn test_init() {
    let state = TitleCard::init();
    assert_eq!(state.title(), "");
}

// =============================================================================
// Builders
// =============================================================================

#[test]
fn test_with_subtitle() {
    let state = TitleCardState::new("App").with_subtitle("v1.0");
    assert_eq!(state.subtitle(), Some("v1.0"));
}

#[test]
fn test_with_prefix() {
    let state = TitleCardState::new("App").with_prefix("🚀 ");
    assert_eq!(state.prefix(), Some("🚀 "));
}

#[test]
fn test_with_suffix() {
    let state = TitleCardState::new("App").with_suffix(" ✨");
    assert_eq!(state.suffix(), Some(" ✨"));
}

#[test]
fn test_with_title_style() {
    let style = Style::default().fg(Color::Red);
    let state = TitleCardState::new("App").with_title_style(style);
    assert_eq!(state.title_style(), style);
}

#[test]
fn test_with_subtitle_style() {
    let style = Style::default().fg(Color::Blue);
    let state = TitleCardState::new("App").with_subtitle_style(style);
    assert_eq!(state.subtitle_style(), style);
}

#[test]
fn test_with_bordered_false() {
    let state = TitleCardState::new("App").with_bordered(false);
    assert!(!state.is_bordered());
}

#[test]
fn test_with_disabled() {
    let state = TitleCardState::new("App").with_disabled(true);
    assert!(state.is_disabled());
}

#[test]
fn test_chained_builders() {
    let state = TitleCardState::new("App")
        .with_subtitle("Sub")
        .with_prefix("! ")
        .with_suffix(" !")
        .with_bordered(false)
        .with_disabled(true);
    assert_eq!(state.title(), "App");
    assert_eq!(state.subtitle(), Some("Sub"));
    assert_eq!(state.prefix(), Some("! "));
    assert_eq!(state.suffix(), Some(" !"));
    assert!(!state.is_bordered());
    assert!(state.is_disabled());
}

// =============================================================================
// Setters
// =============================================================================

#[test]
fn test_set_title() {
    let mut state = TitleCardState::new("Old");
    state.set_title("New");
    assert_eq!(state.title(), "New");
}

#[test]
fn test_set_subtitle() {
    let mut state = TitleCardState::new("App");
    state.set_subtitle(Some("v2.0".to_string()));
    assert_eq!(state.subtitle(), Some("v2.0"));
    state.set_subtitle(None);
    assert_eq!(state.subtitle(), None);
}

#[test]
fn test_set_prefix_and_suffix() {
    let mut state = TitleCardState::new("App");
    state.set_prefix(Some(">> ".to_string()));
    state.set_suffix(Some(" <<".to_string()));
    assert_eq!(state.prefix(), Some(">> "));
    assert_eq!(state.suffix(), Some(" <<"));
}

#[test]
fn test_set_styles() {
    let mut state = TitleCardState::new("App");
    let title_style = Style::default().fg(Color::Green);
    let subtitle_style = Style::default().fg(Color::Yellow);
    state.set_title_style(title_style);
    state.set_subtitle_style(subtitle_style);
    assert_eq!(state.title_style(), title_style);
    assert_eq!(state.subtitle_style(), subtitle_style);
}

#[test]
fn test_set_bordered() {
    let mut state = TitleCardState::new("App");
    state.set_bordered(false);
    assert!(!state.is_bordered());
    state.set_bordered(true);
    assert!(state.is_bordered());
}

#[test]
fn test_set_disabled() {
    let mut state = TitleCardState::new("App");
    state.set_disabled(true);
    assert!(state.is_disabled());
    state.set_disabled(false);
    assert!(!state.is_disabled());
}

// =============================================================================
// Update messages
// =============================================================================

#[test]
fn test_update_set_title() {
    let mut state = TitleCardState::new("Old");
    let output = TitleCard::update(&mut state, TitleCardMessage::SetTitle("New".to_string()));
    assert_eq!(state.title(), "New");
    assert_eq!(output, None);
}

#[test]
fn test_update_set_subtitle() {
    let mut state = TitleCardState::new("App");
    let output = TitleCard::update(
        &mut state,
        TitleCardMessage::SetSubtitle(Some("Sub".to_string())),
    );
    assert_eq!(state.subtitle(), Some("Sub"));
    assert_eq!(output, None);
}

#[test]
fn test_update_set_prefix() {
    let mut state = TitleCardState::new("App");
    let output = TitleCard::update(
        &mut state,
        TitleCardMessage::SetPrefix(Some("! ".to_string())),
    );
    assert_eq!(state.prefix(), Some("! "));
    assert_eq!(output, None);
}

#[test]
fn test_update_set_suffix() {
    let mut state = TitleCardState::new("App");
    let output = TitleCard::update(
        &mut state,
        TitleCardMessage::SetSuffix(Some(" !".to_string())),
    );
    assert_eq!(state.suffix(), Some(" !"));
    assert_eq!(output, None);
}

#[test]
fn test_update_set_title_style() {
    let mut state = TitleCardState::new("App");
    let style = Style::default().fg(Color::Red);
    let output = TitleCard::update(&mut state, TitleCardMessage::SetTitleStyle(style));
    assert_eq!(state.title_style(), style);
    assert_eq!(output, None);
}

#[test]
fn test_update_set_subtitle_style() {
    let mut state = TitleCardState::new("App");
    let style = Style::default().fg(Color::Blue);
    let output = TitleCard::update(&mut state, TitleCardMessage::SetSubtitleStyle(style));
    assert_eq!(state.subtitle_style(), style);
    assert_eq!(output, None);
}

// =============================================================================
// Snapshot tests
// =============================================================================

#[test]
fn test_view_basic() {
    let state = TitleCardState::new("My Application");
    let (mut terminal, theme) = test_utils::setup_render(40, 5);
    terminal
        .draw(|frame| {
            TitleCard::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_with_subtitle() {
    let state = TitleCardState::new("My Application").with_subtitle("Version 1.0");
    let (mut terminal, theme) = test_utils::setup_render(40, 6);
    terminal
        .draw(|frame| {
            TitleCard::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_prefix_suffix() {
    let state = TitleCardState::new("My App")
        .with_prefix("🚀 ")
        .with_suffix(" ✨");
    let (mut terminal, theme) = test_utils::setup_render(40, 5);
    terminal
        .draw(|frame| {
            TitleCard::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_disabled() {
    let state = TitleCardState::new("Disabled Title").with_disabled(true);
    let (mut terminal, theme) = test_utils::setup_render(40, 5);
    terminal
        .draw(|frame| {
            TitleCard::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).disabled(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_no_border() {
    let state = TitleCardState::new("No Border").with_bordered(false);
    let (mut terminal, theme) = test_utils::setup_render(40, 5);
    terminal
        .draw(|frame| {
            TitleCard::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_small_area() {
    let state = TitleCardState::new("Title").with_subtitle("Sub");
    let (mut terminal, theme) = test_utils::setup_render(15, 4);
    terminal
        .draw(|frame| {
            TitleCard::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

// Annotation tests

#[test]
fn test_annotation_emitted() {
    use crate::annotation::{WidgetType, with_annotations};
    let state = TitleCardState::new("Hello");
    let (mut terminal, theme) = test_utils::setup_render(40, 7);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                TitleCard::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
            })
            .unwrap();
    });
    assert_eq!(registry.len(), 1);
    let regions = registry.find_by_type(&WidgetType::TitleCard);
    assert_eq!(regions.len(), 1);
    assert_eq!(regions[0].annotation.label, Some("Hello".to_string()));
    assert!(!regions[0].annotation.disabled);
}
