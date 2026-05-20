use super::*;
use crate::component::Component;
use crate::component::test_utils::setup_render;
use crate::input::{Event, Key};

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
        StyledBlock::Line(inlines) if inlines.len() == 1
    ));
}

#[test]
fn test_content_line_with_inlines() {
    let inlines = vec![
        StyledInline::bold("bold".to_string()),
        StyledInline::Plain(" text".to_string()),
    ];
    let content = StyledContent::new().line(inlines);
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
    let inline = StyledInline::bold("bold".to_string());
    assert!(matches!(
        inline,
        StyledInline::Styled { ref text, style: InlineStyle { bold: true, .. } } if text == "bold"
    ));
}

#[test]
fn test_inline_italic() {
    let inline = StyledInline::italic("italic".to_string());
    assert!(matches!(
        inline,
        StyledInline::Styled { ref text, style: InlineStyle { italic: true, .. } } if text == "italic"
    ));
}

#[test]
fn test_inline_underline() {
    let inline = StyledInline::underlined("underline".to_string());
    assert!(matches!(
        inline,
        StyledInline::Styled { ref text, style: InlineStyle { underlined: true, .. } } if text == "underline"
    ));
}

#[test]
fn test_inline_strikethrough() {
    let inline = StyledInline::strikethrough("strike".to_string());
    assert!(matches!(
        inline,
        StyledInline::Styled { ref text, style: InlineStyle { strikethrough: true, .. } } if text == "strike"
    ));
}

#[test]
fn test_inline_colored() {
    use ratatui::style::Color;
    let inline = StyledInline::colored("colored".to_string(), Color::Red);
    assert!(matches!(
        inline,
        StyledInline::Styled { ref text, style: InlineStyle { fg: Some(Color::Red), bg: None, .. } } if text == "colored"
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
        &Event::key(Key::Up),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(StyledTextMessage::ScrollUp));
}

#[test]
fn test_handle_event_k() {
    let state = StyledTextState::new();
    let msg = StyledText::handle_event(
        &state,
        &Event::char('k'),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(StyledTextMessage::ScrollUp));
}

#[test]
fn test_handle_event_down() {
    let state = StyledTextState::new();
    let msg = StyledText::handle_event(
        &state,
        &Event::key(Key::Down),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(StyledTextMessage::ScrollDown));
}

#[test]
fn test_handle_event_j() {
    let state = StyledTextState::new();
    let msg = StyledText::handle_event(
        &state,
        &Event::char('j'),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(StyledTextMessage::ScrollDown));
}

#[test]
fn test_handle_event_page_up() {
    let state = StyledTextState::new();
    let msg = StyledText::handle_event(
        &state,
        &Event::key(Key::PageUp),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(StyledTextMessage::PageUp(10)));
}

#[test]
fn test_handle_event_page_down() {
    let state = StyledTextState::new();
    let msg = StyledText::handle_event(
        &state,
        &Event::key(Key::PageDown),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(StyledTextMessage::PageDown(10)));
}

#[test]
fn test_handle_event_ctrl_u() {
    let state = StyledTextState::new();
    let msg = StyledText::handle_event(
        &state,
        &Event::ctrl('u'),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(StyledTextMessage::PageUp(10)));
}

#[test]
fn test_handle_event_ctrl_d() {
    let state = StyledTextState::new();
    let msg = StyledText::handle_event(
        &state,
        &Event::ctrl('d'),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(StyledTextMessage::PageDown(10)));
}

#[test]
fn test_handle_event_home() {
    let state = StyledTextState::new();
    let msg = StyledText::handle_event(
        &state,
        &Event::key(Key::Home),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(StyledTextMessage::Home));
}

#[test]
fn test_handle_event_g() {
    let state = StyledTextState::new();
    let msg = StyledText::handle_event(
        &state,
        &Event::char('g'),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(StyledTextMessage::Home));
}

#[test]
fn test_handle_event_end() {
    let state = StyledTextState::new();
    let msg = StyledText::handle_event(
        &state,
        &Event::key(Key::End),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(StyledTextMessage::End));
}

#[test]
fn test_handle_event_unfocused_ignored() {
    let state = StyledTextState::new();
    let msg = StyledText::handle_event(&state, &Event::key(Key::Down), &EventContext::default());
    assert_eq!(msg, None);
}

#[test]
fn test_handle_event_disabled_ignored() {
    let state = StyledTextState::new();
    let msg = StyledText::handle_event(
        &state,
        &Event::key(Key::Down),
        &EventContext::new().focused(true).disabled(true),
    );
    assert_eq!(msg, None);
}

#[test]
fn test_handle_event_unrecognized() {
    let state = StyledTextState::new();
    let msg = StyledText::handle_event(
        &state,
        &Event::char('z'),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, None);
}

// ========== dispatch_event and instance method Tests ==========

#[test]
fn test_dispatch_event() {
    let mut state = StyledTextState::new();
    let output = StyledText::dispatch_event(
        &mut state,
        &Event::key(Key::Down),
        &EventContext::new().focused(true),
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
            StyledText::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
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
        vec![StyledInline::bold("Bold item".to_string())],
    ]);
    let state = StyledTextState::new().with_content(content);

    terminal
        .draw(|frame| {
            StyledText::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
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
            StyledText::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
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
            StyledText::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
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
            StyledText::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
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
            StyledText::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
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
            StyledText::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
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
            StyledText::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
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
            StyledText::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
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
                    &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
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

#[test]
fn test_view_skips_border_when_chrome_owned_even_if_show_border_true() {
    let (mut terminal, theme) = setup_render(40, 5);
    let content = StyledContent::new().text("Hello, embedded.");
    let state = StyledTextState::new()
        .with_content(content)
        .with_show_border(true);

    terminal
        .draw(|frame| {
            StyledText::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).chrome_owned(true),
            );
        })
        .unwrap();

    let display = terminal.backend().to_string();
    insta::assert_snapshot!("view_chrome_owned_no_border", display);
}

// ========== InlineStyle + StyledInline Composable API Tests ==========

#[test]
fn test_inline_style_new_is_default() {
    use crate::component::styled_text::InlineStyle;

    // Pins the contract that ::new() and ::default() produce equivalent
    // empty styles. Consumer code can use either interchangeably.
    assert_eq!(InlineStyle::new(), InlineStyle::default());
}

#[test]
fn test_inline_style_builder_chain() {
    use crate::component::styled_text::InlineStyle;
    use ratatui::style::Color;

    // Each builder method sets exactly the field it names; other fields
    // stay at their default. Pin via field-level assertions.
    let style = InlineStyle::new().bold().fg(Color::Red).underlined();

    assert!(style.bold);
    assert_eq!(style.fg, Some(Color::Red));
    assert!(style.underlined);

    // Untouched fields remain default.
    assert!(!style.italic);
    assert!(!style.strikethrough);
    assert_eq!(style.bg, None);
}

#[test]
fn test_styled_inline_styled_pairs_text_and_style() {
    use crate::component::styled_text::{InlineStyle, StyledInline};
    use ratatui::style::Color;

    let style = InlineStyle::new().fg(Color::Magenta).bold();
    let inline = StyledInline::styled("hello", style);

    // The general-purpose constructor pairs text with style verbatim.
    match inline {
        StyledInline::Styled { text, style: s } => {
            assert_eq!(text, "hello");
            assert_eq!(s, style);
        }
        _ => panic!("expected Styled variant, got: {inline:?}"),
    }
}

#[test]
fn test_styled_inline_leaf_helpers_match_builder() {
    use crate::component::styled_text::{InlineStyle, StyledInline};
    use ratatui::style::Color;

    // Each leaf helper must produce the same StyledInline value as
    // StyledInline::styled(t, InlineStyle::new().<dim>()). Pins the
    // helper-vs-builder contract so refactors don't drift.
    assert_eq!(
        StyledInline::bold("t"),
        StyledInline::styled("t", InlineStyle::new().bold()),
    );
    assert_eq!(
        StyledInline::italic("t"),
        StyledInline::styled("t", InlineStyle::new().italic()),
    );
    assert_eq!(
        StyledInline::underlined("t"),
        StyledInline::styled("t", InlineStyle::new().underlined()),
    );
    assert_eq!(
        StyledInline::strikethrough("t"),
        StyledInline::styled("t", InlineStyle::new().strikethrough()),
    );
    assert_eq!(
        StyledInline::colored("t", Color::Red),
        StyledInline::styled("t", InlineStyle::new().fg(Color::Red)),
    );
}

// ========== StyledInline Render-Path Tests ==========

#[test]
fn snapshot_styled_inline_bold_and_colored_combined() {
    use crate::component::styled_text::{InlineStyle, StyledInline};
    use crate::render::styled_line;
    use ratatui::style::Color;

    // THE LOAD-BEARING G6 PAYOFF PIN.
    //
    // The bold+colored combo specifically because it's the user-visible
    // payoff for G6: leadline's build_summary_inlines (app.rs:412-455)
    // emits 5 value segments (iconnx/ort/ratio/delta/iters) that need
    // bold + severity-color in a SINGLE inline run. Pre-G6, Bold(t)
    // had no color field and Colored {..} had no bold field, so the
    // bold half was dropped. Post-G6, this test asserts the combo
    // lands — both \x1b[31m (red) AND \x1b[1m (BOLD) appear on the
    // same Span in the rendered ANSI output.
    //
    // If either escape goes missing, build_summary_inlines reads flat
    // again — the user loses the magnitude-jump weight contrast that
    // makes the per-op summary banner readable at a glance.
    let inlines = vec![StyledInline::styled(
        "840.16 ms",
        InlineStyle::new().fg(Color::Red).bold(),
    )];

    let (mut terminal, theme) = setup_render(20, 1);
    terminal
        .draw(|frame| {
            styled_line(frame, frame.area(), &inlines, &theme);
        })
        .unwrap();

    let plain = terminal.backend().to_string();
    let ansi = terminal.backend().to_ansi();

    assert!(
        ansi.contains("\x1b[31m"),
        "expected red (31m) ANSI fg for fg(Red), got:\n{ansi}",
    );
    assert!(
        ansi.contains("\x1b[1m"),
        "expected BOLD (1m) ANSI modifier for bold(), got:\n{ansi}",
    );

    insta::assert_snapshot!(plain);
}

#[test]
fn snapshot_styled_inline_full_dimension_combo() {
    use crate::component::styled_text::{InlineStyle, StyledInline};
    use crate::render::styled_line;
    use ratatui::style::Color;

    // Render every dimension at once. Pins the full composability
    // surface: 6 dimensions in a single inline (bold + italic +
    // underlined + strikethrough + fg + bg).
    let inlines = vec![StyledInline::styled(
        "ALL",
        InlineStyle::new()
            .bold()
            .italic()
            .underlined()
            .strikethrough()
            .fg(Color::Red)
            .bg(Color::Black),
    )];

    let (mut terminal, theme) = setup_render(20, 1);
    terminal
        .draw(|frame| {
            styled_line(frame, frame.area(), &inlines, &theme);
        })
        .unwrap();

    let ansi = terminal.backend().to_ansi();

    // All 6 SGR codes appear. CaptureBackend combines modifier codes into a
    // single CSI sequence (e.g. \x1b[1;3;4;9m) while emitting color codes
    // separately. The helper below checks for a code appearing either
    // standalone (\x1b[Xm) or combined (\x1b[...;X;...m / \x1b[X;...m /
    // \x1b[...;Xm) so assertions are robust against either format.
    fn has_sgr(ansi: &str, code: &str) -> bool {
        // Matches: \x1b[ ... code ... m where code is preceded by [ or ; and
        // followed by ; or m.
        let pat_standalone = format!("\x1b[{}m", code);
        let pat_prefix = format!("\x1b[{};", code);
        let pat_mid = format!(";{};", code);
        let pat_suffix = format!(";{}m", code);
        ansi.contains(&pat_standalone)
            || ansi.contains(&pat_prefix)
            || ansi.contains(&pat_mid)
            || ansi.contains(&pat_suffix)
    }

    // - 1   bold
    // - 3   italic
    // - 4   underlined
    // - 9   strikethrough (ratatui Modifier::CROSSED_OUT)
    // - 31  red foreground
    // - 40  black background
    assert!(has_sgr(&ansi, "1"), "expected bold (1), got:\n{ansi}");
    assert!(has_sgr(&ansi, "3"), "expected italic (3), got:\n{ansi}");
    assert!(has_sgr(&ansi, "4"), "expected underlined (4), got:\n{ansi}");
    assert!(
        has_sgr(&ansi, "9"),
        "expected strikethrough/crossed_out (9), got:\n{ansi}"
    );
    assert!(has_sgr(&ansi, "31"), "expected red fg (31), got:\n{ansi}");
    assert!(has_sgr(&ansi, "40"), "expected black bg (40), got:\n{ansi}");

    let plain = terminal.backend().to_string();
    insta::assert_snapshot!(plain);
}

#[test]
fn test_inline_style_default_no_modifiers_applied() {
    use crate::component::styled_text::{InlineStyle, StyledInline};
    use crate::render::styled_line;

    // Rendering StyledInline::styled(t, InlineStyle::default()) should
    // produce no ANSI modifiers — equivalent to Plain(t) at the
    // rendering level.
    let styled_default = vec![StyledInline::styled("text", InlineStyle::default())];

    let (mut term_styled, theme) = setup_render(20, 1);
    term_styled
        .draw(|frame| {
            styled_line(frame, frame.area(), &styled_default, &theme);
        })
        .unwrap();
    let styled_ansi = term_styled.backend().to_ansi();

    // No bold/italic/underlined/strikethrough escapes — empty InlineStyle
    // adds no modifiers.
    for escape in ["\x1b[1m", "\x1b[3m", "\x1b[4m", "\x1b[9m"] {
        assert!(
            !styled_ansi.contains(escape),
            "Styled(default) should not emit modifier {escape}, got:\n{styled_ansi}",
        );
    }
}

#[test]
fn snapshot_styled_inline_plain_and_code_unchanged_postmigration() {
    use crate::component::styled_text::StyledInline;
    use crate::render::styled_line;

    // Plain and Code are the two variants that survive G6 unchanged.
    // Their rendering must be byte-identical post-migration — pin via
    // snapshot. If either snapshot drifts, the G6 enum reshape
    // inadvertently altered the surviving variants.
    let inlines = vec![
        StyledInline::Plain("plain text".into()),
        StyledInline::Code("code text".into()),
    ];

    let (mut terminal, theme) = setup_render(40, 1);
    terminal
        .draw(|frame| {
            styled_line(frame, frame.area(), &inlines, &theme);
        })
        .unwrap();

    let plain = terminal.backend().to_string();
    insta::assert_snapshot!(plain);
}
