use super::*;
use crate::component::Component;
use crate::component::test_utils::setup_render;
use crate::input::{Event, KeyCode};

// ========== StyledContent Builder Tests ==========

#[test]
fn test_content_new_is_empty() {
    let content = StyledContent::new();
    assert!(content.is_empty());
    assert_eq!(content.len(), 0);
}

#[test]
fn test_content_heading() {
    let content = StyledContent::new().heading(1, "Title");
    assert_eq!(content.len(), 1);
    assert!(matches!(
        &content.blocks()[0],
        StyledBlock::Heading { level: 1, text } if text == "Title"
    ));
}

#[test]
fn test_content_heading_level_clamped() {
    let content = StyledContent::new().heading(0, "Low").heading(5, "High");
    if let StyledBlock::Heading { level, .. } = &content.blocks()[0] {
        assert_eq!(*level, 1);
    }
    if let StyledBlock::Heading { level, .. } = &content.blocks()[1] {
        assert_eq!(*level, 3);
    }
}

#[test]
fn test_content_text() {
    let content = StyledContent::new().text("Hello");
    assert_eq!(content.len(), 1);
    assert!(matches!(
        &content.blocks()[0],
        StyledBlock::Paragraph(inlines) if inlines.len() == 1
    ));
}

#[test]
fn test_content_paragraph_with_inlines() {
    let inlines = vec![
        StyledInline::Bold("bold".to_string()),
        StyledInline::Plain(" text".to_string()),
    ];
    let content = StyledContent::new().paragraph(inlines);
    assert_eq!(content.len(), 1);
}

#[test]
fn test_content_bullet_list() {
    let items = vec![
        vec![StyledInline::Plain("Item 1".to_string())],
        vec![StyledInline::Plain("Item 2".to_string())],
    ];
    let content = StyledContent::new().bullet_list(items);
    assert_eq!(content.len(), 1);
    assert!(matches!(&content.blocks()[0], StyledBlock::BulletList(items) if items.len() == 2));
}

#[test]
fn test_content_numbered_list() {
    let items = vec![
        vec![StyledInline::Plain("First".to_string())],
        vec![StyledInline::Plain("Second".to_string())],
    ];
    let content = StyledContent::new().numbered_list(items);
    assert_eq!(content.len(), 1);
    assert!(matches!(&content.blocks()[0], StyledBlock::NumberedList(items) if items.len() == 2));
}

#[test]
fn test_content_code_block() {
    let content = StyledContent::new().code_block(Some("rust"), "let x = 1;");
    assert_eq!(content.len(), 1);
    assert!(matches!(
        &content.blocks()[0],
        StyledBlock::CodeBlock { language: Some(lang), content } if lang == "rust" && content == "let x = 1;"
    ));
}

#[test]
fn test_content_code_block_no_language() {
    let content = StyledContent::new().code_block(None::<String>, "code");
    assert!(matches!(
        &content.blocks()[0],
        StyledBlock::CodeBlock { language: None, .. }
    ));
}

#[test]
fn test_content_horizontal_rule() {
    let content = StyledContent::new().horizontal_rule();
    assert_eq!(content.len(), 1);
    assert!(matches!(&content.blocks()[0], StyledBlock::HorizontalRule));
}

#[test]
fn test_content_blank_line() {
    let content = StyledContent::new().blank_line();
    assert_eq!(content.len(), 1);
    assert!(matches!(&content.blocks()[0], StyledBlock::BlankLine));
}

#[test]
fn test_content_raw() {
    use ratatui::text::Line as RatLine;
    let lines = vec![RatLine::from("raw content")];
    let content = StyledContent::new().raw(lines);
    assert_eq!(content.len(), 1);
    assert!(matches!(&content.blocks()[0], StyledBlock::Raw(_)));
}

#[test]
fn test_content_push() {
    let content = StyledContent::new()
        .push(StyledBlock::BlankLine)
        .push(StyledBlock::HorizontalRule);
    assert_eq!(content.len(), 2);
}

#[test]
fn test_content_chained_builder() {
    let content = StyledContent::new()
        .heading(1, "Title")
        .text("Paragraph")
        .blank_line()
        .code_block(None::<String>, "code")
        .horizontal_rule();
    assert_eq!(content.len(), 5);
}

// ========== StyledInline Tests ==========

#[test]
fn test_inline_plain() {
    let inline = StyledInline::Plain("text".to_string());
    assert!(matches!(inline, StyledInline::Plain(s) if s == "text"));
}

#[test]
fn test_inline_bold() {
    let inline = StyledInline::Bold("bold".to_string());
    assert!(matches!(inline, StyledInline::Bold(s) if s == "bold"));
}

#[test]
fn test_inline_italic() {
    let inline = StyledInline::Italic("italic".to_string());
    assert!(matches!(inline, StyledInline::Italic(s) if s == "italic"));
}

#[test]
fn test_inline_underline() {
    let inline = StyledInline::Underline("underline".to_string());
    assert!(matches!(inline, StyledInline::Underline(s) if s == "underline"));
}

#[test]
fn test_inline_strikethrough() {
    let inline = StyledInline::Strikethrough("strike".to_string());
    assert!(matches!(inline, StyledInline::Strikethrough(s) if s == "strike"));
}

#[test]
fn test_inline_colored() {
    use ratatui::style::Color;
    let inline = StyledInline::Colored {
        text: "colored".to_string(),
        fg: Some(Color::Red),
        bg: None,
    };
    assert!(matches!(
        inline,
        StyledInline::Colored { text, fg: Some(Color::Red), bg: None } if text == "colored"
    ));
}

#[test]
fn test_inline_code() {
    let inline = StyledInline::Code("code".to_string());
    assert!(matches!(inline, StyledInline::Code(s) if s == "code"));
}

// ========== State Creation Tests ==========

#[test]
fn test_state_new() {
    let state = StyledTextState::new();
    assert!(state.content().is_empty());
    assert_eq!(state.scroll_offset(), 0);
    assert_eq!(state.title(), None);
    assert!(state.show_border());
}

#[test]
fn test_state_with_content() {
    let content = StyledContent::new().text("Hello");
    let state = StyledTextState::new().with_content(content);
    assert!(!state.content().is_empty());
}

#[test]
fn test_state_with_title() {
    let state = StyledTextState::new().with_title("Preview");
    assert_eq!(state.title(), Some("Preview"));
}

#[test]
fn test_state_with_show_border() {
    let state = StyledTextState::new().with_show_border(false);
    assert!(!state.show_border());
}

#[test]
fn test_state_set_content_resets_scroll() {
    let mut state = StyledTextState::new();
    state.scroll_offset = 5;
    state.set_content(StyledContent::new().text("New"));
    assert_eq!(state.scroll_offset(), 0);
}

// ========== Scroll Update Tests ==========

#[test]
fn test_scroll_down() {
    let mut state = StyledTextState::new();
    let output = StyledText::update(&mut state, StyledTextMessage::ScrollDown);
    assert_eq!(state.scroll_offset(), 1);
    assert_eq!(output, Some(StyledTextOutput::ScrollChanged(1)));
}

#[test]
fn test_scroll_up() {
    let mut state = StyledTextState::new();
    state.scroll_offset = 3;
    let output = StyledText::update(&mut state, StyledTextMessage::ScrollUp);
    assert_eq!(state.scroll_offset(), 2);
    assert_eq!(output, Some(StyledTextOutput::ScrollChanged(2)));
}

#[test]
fn test_scroll_up_at_zero() {
    let mut state = StyledTextState::new();
    let output = StyledText::update(&mut state, StyledTextMessage::ScrollUp);
    assert_eq!(output, None);
}

#[test]
fn test_page_down() {
    let mut state = StyledTextState::new();
    let output = StyledText::update(&mut state, StyledTextMessage::PageDown(10));
    assert_eq!(state.scroll_offset(), 10);
    assert_eq!(output, Some(StyledTextOutput::ScrollChanged(10)));
}

#[test]
fn test_page_up() {
    let mut state = StyledTextState::new();
    state.scroll_offset = 15;
    let output = StyledText::update(&mut state, StyledTextMessage::PageUp(10));
    assert_eq!(state.scroll_offset(), 5);
    assert_eq!(output, Some(StyledTextOutput::ScrollChanged(5)));
}

#[test]
fn test_page_up_at_zero() {
    let mut state = StyledTextState::new();
    let output = StyledText::update(&mut state, StyledTextMessage::PageUp(10));
    assert_eq!(output, None);
}

#[test]
fn test_home() {
    let mut state = StyledTextState::new();
    state.scroll_offset = 10;
    let output = StyledText::update(&mut state, StyledTextMessage::Home);
    assert_eq!(state.scroll_offset(), 0);
    assert_eq!(output, Some(StyledTextOutput::ScrollChanged(0)));
}

#[test]
fn test_home_already_at_top() {
    let mut state = StyledTextState::new();
    let output = StyledText::update(&mut state, StyledTextMessage::Home);
    assert_eq!(output, None);
}

#[test]
fn test_end() {
    let mut state = StyledTextState::new();
    let output = StyledText::update(&mut state, StyledTextMessage::End);
    assert_eq!(state.scroll_offset(), usize::MAX);
    assert!(output.is_some());
}

#[test]
fn test_set_content_resets_scroll() {
    let mut state = StyledTextState::new();
    state.scroll_offset = 10;
    let output = StyledText::update(
        &mut state,
        StyledTextMessage::SetContent(StyledContent::new().text("New content")),
    );
    assert_eq!(state.scroll_offset(), 0);
    assert_eq!(output, None);
}

// ========== handle_event Tests ==========

#[test]
fn test_handle_event_up() {
    let state = StyledTextState::new();
    let msg = StyledText::handle_event(
        &state,
        &Event::key(KeyCode::Up),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(StyledTextMessage::ScrollUp));
}

#[test]
fn test_handle_event_k() {
    let state = StyledTextState::new();
    let msg =
        StyledText::handle_event(&state, &Event::char('k'), &ViewContext::new().focused(true));
    assert_eq!(msg, Some(StyledTextMessage::ScrollUp));
}

#[test]
fn test_handle_event_down() {
    let state = StyledTextState::new();
    let msg = StyledText::handle_event(
        &state,
        &Event::key(KeyCode::Down),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(StyledTextMessage::ScrollDown));
}

#[test]
fn test_handle_event_j() {
    let state = StyledTextState::new();
    let msg =
        StyledText::handle_event(&state, &Event::char('j'), &ViewContext::new().focused(true));
    assert_eq!(msg, Some(StyledTextMessage::ScrollDown));
}

#[test]
fn test_handle_event_page_up() {
    let state = StyledTextState::new();
    let msg = StyledText::handle_event(
        &state,
        &Event::key(KeyCode::PageUp),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(StyledTextMessage::PageUp(10)));
}

#[test]
fn test_handle_event_page_down() {
    let state = StyledTextState::new();
    let msg = StyledText::handle_event(
        &state,
        &Event::key(KeyCode::PageDown),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(StyledTextMessage::PageDown(10)));
}

#[test]
fn test_handle_event_ctrl_u() {
    let state = StyledTextState::new();
    let msg =
        StyledText::handle_event(&state, &Event::ctrl('u'), &ViewContext::new().focused(true));
    assert_eq!(msg, Some(StyledTextMessage::PageUp(10)));
}

#[test]
fn test_handle_event_ctrl_d() {
    let state = StyledTextState::new();
    let msg =
        StyledText::handle_event(&state, &Event::ctrl('d'), &ViewContext::new().focused(true));
    assert_eq!(msg, Some(StyledTextMessage::PageDown(10)));
}

#[test]
fn test_handle_event_home() {
    let state = StyledTextState::new();
    let msg = StyledText::handle_event(
        &state,
        &Event::key(KeyCode::Home),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(StyledTextMessage::Home));
}

#[test]
fn test_handle_event_g() {
    let state = StyledTextState::new();
    let msg =
        StyledText::handle_event(&state, &Event::char('g'), &ViewContext::new().focused(true));
    assert_eq!(msg, Some(StyledTextMessage::Home));
}

#[test]
fn test_handle_event_end() {
    let state = StyledTextState::new();
    let msg = StyledText::handle_event(
        &state,
        &Event::key(KeyCode::End),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(StyledTextMessage::End));
}

#[test]
fn test_handle_event_unfocused_ignored() {
    let state = StyledTextState::new();
    let msg = StyledText::handle_event(&state, &Event::key(KeyCode::Down), &ViewContext::default());
    assert_eq!(msg, None);
}

#[test]
fn test_handle_event_disabled_ignored() {
    let state = StyledTextState::new();
    let msg = StyledText::handle_event(
        &state,
        &Event::key(KeyCode::Down),
        &ViewContext::new().focused(true).disabled(true),
    );
    assert_eq!(msg, None);
}

#[test]
fn test_handle_event_unrecognized() {
    let state = StyledTextState::new();
    let msg =
        StyledText::handle_event(&state, &Event::char('z'), &ViewContext::new().focused(true));
    assert_eq!(msg, None);
}

// ========== dispatch_event and instance method Tests ==========

#[test]
fn test_dispatch_event() {
    let mut state = StyledTextState::new();
    let output = StyledText::dispatch_event(
        &mut state,
        &Event::key(KeyCode::Down),
        &ViewContext::new().focused(true),
    );
    assert_eq!(output, Some(StyledTextOutput::ScrollChanged(1)));
}

#[test]
fn test_instance_update() {
    let mut state = StyledTextState::new();
    let output = state.update(StyledTextMessage::ScrollDown);
    assert_eq!(output, Some(StyledTextOutput::ScrollChanged(1)));
}
// ========== Init Test ==========

#[test]
fn test_init() {
    let state = StyledText::init();
    assert!(state.content().is_empty());
    assert!(state.show_border());
}

// ========== Rendering Snapshot Tests ==========

#[test]
fn test_view_heading_and_text() {
    let (mut terminal, theme) = setup_render(50, 8);
    let content = StyledContent::new()
        .heading(1, "Welcome")
        .text("This is a paragraph.");
    let state = StyledTextState::new().with_content(content);

    terminal
        .draw(|frame| {
            StyledText::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    let display = terminal.backend().to_string();
    insta::assert_snapshot!("view_heading_and_text", display);
}

#[test]
fn test_view_bullet_list() {
    let (mut terminal, theme) = setup_render(50, 8);
    let content = StyledContent::new().heading(2, "Items").bullet_list(vec![
        vec![StyledInline::Plain("First item".to_string())],
        vec![StyledInline::Plain("Second item".to_string())],
        vec![StyledInline::Bold("Bold item".to_string())],
    ]);
    let state = StyledTextState::new().with_content(content);

    terminal
        .draw(|frame| {
            StyledText::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    let display = terminal.backend().to_string();
    insta::assert_snapshot!("view_bullet_list", display);
}

#[test]
fn test_view_numbered_list() {
    let (mut terminal, theme) = setup_render(50, 8);
    let content = StyledContent::new().numbered_list(vec![
        vec![StyledInline::Plain("First".to_string())],
        vec![StyledInline::Plain("Second".to_string())],
        vec![StyledInline::Plain("Third".to_string())],
    ]);
    let state = StyledTextState::new().with_content(content);

    terminal
        .draw(|frame| {
            StyledText::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    let display = terminal.backend().to_string();
    insta::assert_snapshot!("view_numbered_list", display);
}

#[test]
fn test_view_code_block() {
    let (mut terminal, theme) = setup_render(50, 8);
    let content =
        StyledContent::new().code_block(Some("rust"), "let x = 42;\nprintln!(\"{}\", x);");
    let state = StyledTextState::new().with_content(content);

    terminal
        .draw(|frame| {
            StyledText::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    let display = terminal.backend().to_string();
    insta::assert_snapshot!("view_code_block", display);
}

#[test]
fn test_view_horizontal_rule() {
    let (mut terminal, theme) = setup_render(40, 6);
    let content = StyledContent::new()
        .text("Above")
        .horizontal_rule()
        .text("Below");
    let state = StyledTextState::new().with_content(content);

    terminal
        .draw(|frame| {
            StyledText::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    let display = terminal.backend().to_string();
    insta::assert_snapshot!("view_horizontal_rule", display);
}

#[test]
fn test_view_with_title() {
    let (mut terminal, theme) = setup_render(40, 6);
    let content = StyledContent::new().text("Content");
    let state = StyledTextState::new()
        .with_content(content)
        .with_title("My Title");

    terminal
        .draw(|frame| {
            StyledText::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    let display = terminal.backend().to_string();
    insta::assert_snapshot!("view_with_title", display);
}

#[test]
fn test_view_no_border() {
    let (mut terminal, theme) = setup_render(40, 4);
    let content = StyledContent::new().text("No border");
    let state = StyledTextState::new()
        .with_content(content)
        .with_show_border(false);

    terminal
        .draw(|frame| {
            StyledText::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    let display = terminal.backend().to_string();
    insta::assert_snapshot!("view_no_border", display);
}

#[test]
fn test_view_empty_content() {
    let (mut terminal, theme) = setup_render(40, 5);
    let state = StyledTextState::new();

    terminal
        .draw(|frame| {
            StyledText::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    let display = terminal.backend().to_string();
    insta::assert_snapshot!("view_empty", display);
}

#[test]
fn test_view_mixed_content() {
    let (mut terminal, theme) = setup_render(60, 15);
    let content = StyledContent::new()
        .heading(1, "Document Title")
        .blank_line()
        .text("Introduction paragraph.")
        .horizontal_rule()
        .heading(2, "Section")
        .bullet_list(vec![
            vec![StyledInline::Plain("Point A".to_string())],
            vec![StyledInline::Plain("Point B".to_string())],
        ])
        .blank_line()
        .code_block(None::<String>, "example code");
    let state = StyledTextState::new().with_content(content);

    terminal
        .draw(|frame| {
            StyledText::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    let display = terminal.backend().to_string();
    insta::assert_snapshot!("view_mixed_content", display);
}

// ========== Annotation Tests ==========

#[test]
fn test_annotation_emission() {
    use crate::annotation::{WidgetType, with_annotations};
    let state = StyledTextState::new();
    let (mut terminal, theme) = setup_render(40, 5);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                StyledText::view(
                    &state,
                    frame,
                    frame.area(),
                    &theme,
                    &ViewContext::new().focused(true),
                );
            })
            .unwrap();
    });
    assert_eq!(registry.len(), 1);
    let regions = registry.find_by_type(&WidgetType::StyledText);
    assert_eq!(regions.len(), 1);
    assert!(regions[0].annotation.has_id("styled_text"));
    assert!(regions[0].annotation.focused);
    assert!(!regions[0].annotation.disabled);
}
