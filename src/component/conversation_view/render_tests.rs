use super::*;
use crate::component::test_utils;

fn focused_state() -> ConversationViewState {
    ConversationViewState::new()
}

fn state_with_messages() -> ConversationViewState {
    let mut state = focused_state();
    state.push_system("Welcome to the conversation.");
    state.push_user("Hello, can you help me?");
    state.push_assistant("Of course! What do you need?");
    state
}

// =============================================================================
// Rendering
// =============================================================================

#[test]
fn test_render_empty() {
    let state = ConversationViewState::new();
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ConversationView::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
}

#[test]
fn test_render_with_messages() {
    let state = state_with_messages();
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ConversationView::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
}

#[test]
fn test_render_focused() {
    let state = focused_state();
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ConversationView::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
}

#[test]
fn test_render_disabled() {
    let state = ConversationViewState::new();
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ConversationView::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).disabled(true),
            );
        })
        .unwrap();
}

#[test]
fn test_render_with_title() {
    let state = ConversationViewState::new().with_title("Session 1");
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ConversationView::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
}

#[test]
fn test_render_with_timestamps() {
    let mut state = ConversationViewState::new().with_timestamps(true);
    state.push_message(
        ConversationMessage::new(ConversationRole::User, "Hello").with_timestamp("14:30"),
    );
    state.push_message(
        ConversationMessage::new(ConversationRole::Assistant, "Hi!").with_timestamp("14:31"),
    );
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ConversationView::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
}

#[test]
fn test_render_without_role_labels() {
    let mut state = ConversationViewState::new().with_role_labels(false);
    state.push_user("Hello");
    state.push_assistant("Hi!");
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ConversationView::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
}

#[test]
fn test_render_code_block() {
    let mut state = ConversationViewState::new();
    state.push_message(ConversationMessage::with_blocks(
        ConversationRole::Assistant,
        vec![
            MessageBlock::text("Here is the code:"),
            MessageBlock::code("fn main() {\n    println!(\"hello\");\n}", Some("rust")),
        ],
    ));
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ConversationView::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
}

#[test]
fn test_render_tool_use_block() {
    let mut state = ConversationViewState::new();
    state.push_message(ConversationMessage::with_blocks(
        ConversationRole::Assistant,
        vec![
            MessageBlock::text("I'll search for that."),
            MessageBlock::tool_use("web_search").with_input("query: rust TUI frameworks"),
        ],
    ));
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ConversationView::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
}

#[test]
fn test_render_thinking_block() {
    let mut state = ConversationViewState::new();
    state.push_message(ConversationMessage::with_blocks(
        ConversationRole::Assistant,
        vec![
            MessageBlock::thinking("Let me reason through this problem..."),
            MessageBlock::text("The answer is 42."),
        ],
    ));
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ConversationView::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
}

#[test]
fn test_render_error_block() {
    let mut state = ConversationViewState::new();
    state.push_message(ConversationMessage::with_blocks(
        ConversationRole::Tool,
        vec![MessageBlock::error("Connection timeout")],
    ));
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ConversationView::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
}

#[test]
fn test_render_streaming_message() {
    let mut state = ConversationViewState::new();
    state.push_message(
        ConversationMessage::new(ConversationRole::Assistant, "Generating...").with_streaming(true),
    );
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ConversationView::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
}

#[test]
fn test_render_collapsed_thinking() {
    let mut state = ConversationViewState::new();
    state.collapse("thinking");
    state.push_message(ConversationMessage::with_blocks(
        ConversationRole::Assistant,
        vec![
            MessageBlock::thinking("Hidden reasoning"),
            MessageBlock::text("Visible answer"),
        ],
    ));
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ConversationView::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
}

#[test]
fn test_render_collapsed_tool_use() {
    let mut state = ConversationViewState::new();
    state.collapse("tool:search");
    state.push_message(ConversationMessage::with_blocks(
        ConversationRole::Assistant,
        vec![MessageBlock::tool_use("search").with_input("query: test")],
    ));
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ConversationView::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
}

#[test]
fn test_render_small_area() {
    let state = state_with_messages();
    let (mut terminal, theme) = test_utils::setup_render(60, 4);
    terminal
        .draw(|frame| {
            ConversationView::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
}

#[test]
fn test_render_tiny_area_no_panic() {
    let state = state_with_messages();
    let (mut terminal, theme) = test_utils::setup_render(4, 2);
    terminal
        .draw(|frame| {
            ConversationView::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
}

#[test]
fn test_render_mixed_blocks() {
    let mut state = ConversationViewState::new();
    state.push_message(ConversationMessage::with_blocks(
        ConversationRole::Assistant,
        vec![
            MessageBlock::thinking("Analyzing the problem..."),
            MessageBlock::text("I found the answer."),
            MessageBlock::code("x = 42", Some("python")),
            MessageBlock::tool_use("calculator").with_input("42 * 2"),
            MessageBlock::error("Rate limit exceeded"),
        ],
    ));
    let (mut terminal, theme) = test_utils::setup_render(60, 30);
    terminal
        .draw(|frame| {
            ConversationView::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
}

// =============================================================================
// Empty code/tool blocks
// =============================================================================

#[test]
fn test_render_empty_code_block() {
    let mut state = ConversationViewState::new();
    state.push_message(ConversationMessage::with_blocks(
        ConversationRole::Assistant,
        vec![MessageBlock::code("", None)],
    ));
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ConversationView::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
}

#[test]
fn test_render_empty_tool_input() {
    let mut state = ConversationViewState::new();
    state.push_message(ConversationMessage::with_blocks(
        ConversationRole::Assistant,
        vec![MessageBlock::tool_use("noop")],
    ));
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ConversationView::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
}

#[test]
fn test_render_empty_text_block() {
    let mut state = ConversationViewState::new();
    state.push_message(ConversationMessage::with_blocks(
        ConversationRole::User,
        vec![MessageBlock::text("")],
    ));
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ConversationView::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
}

// =============================================================================
// Annotation
// =============================================================================

#[test]
fn test_annotation_emitted() {
    use crate::annotation::with_annotations;
    let state = ConversationViewState::new();
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                ConversationView::view(
                    &state,
                    &mut RenderContext::new(frame, frame.area(), &theme),
                );
            })
            .unwrap();
    });
    assert!(registry.get_by_id("conversation_view").is_some());
}

#[cfg(feature = "markdown")]
#[test]
fn test_role_style_override_in_rendering() {
    use ratatui::style::{Color, Modifier, Style};

    let mut state = ConversationViewState::new().with_markdown(true);
    state.set_role_style(ConversationRole::User, Style::default().fg(Color::Cyan));
    state.push_user("Hello from user");
    state.push_assistant("Hello from assistant");

    let theme = crate::theme::Theme::default();
    let lines = super::render::build_display_lines(state.source_messages(), &state, 80, &theme);

    // Find user body spans — should use overridden Cyan color
    let mut found_user_cyan = false;
    let mut in_user_section = false;
    for line in &lines {
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        if text.contains("User")
            && line
                .spans
                .iter()
                .any(|s| s.style.add_modifier.contains(Modifier::BOLD))
        {
            in_user_section = true;
            continue;
        }
        if text.contains("Assistant")
            && line
                .spans
                .iter()
                .any(|s| s.style.add_modifier.contains(Modifier::BOLD))
        {
            in_user_section = false;
            continue;
        }
        if in_user_section {
            for span in &line.spans {
                if span.content.contains("Hello") && span.style.fg == Some(Color::Cyan) {
                    found_user_cyan = true;
                }
            }
        }
    }
    assert!(
        found_user_cyan,
        "User message body should use the overridden Cyan color"
    );

    // Assistant should still use default Blue (no override set)
    let mut found_assistant_blue = false;
    let mut in_assistant_section = false;
    for line in &lines {
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        if text.contains("Assistant")
            && line
                .spans
                .iter()
                .any(|s| s.style.add_modifier.contains(Modifier::BOLD))
        {
            in_assistant_section = true;
            continue;
        }
        if in_assistant_section {
            for span in &line.spans {
                if span.content.contains("Hello") && span.style.fg == Some(Color::Blue) {
                    found_assistant_blue = true;
                }
            }
        }
    }
    assert!(
        found_assistant_blue,
        "Assistant message body should use default Blue (no override set)"
    );
}

#[cfg(feature = "markdown")]
#[test]
fn test_markdown_role_style_propagation() {
    use ratatui::style::{Color, Modifier, Style};

    let mut state = ConversationViewState::new().with_markdown(true);
    state.push_user("plain text and **bold** and `inline code`");
    state.push_assistant("plain text and **bold** and `inline code`");

    let theme = crate::theme::Theme::default();
    let lines = super::render::build_display_lines(state.source_messages(), &state, 80, &theme);

    // Partition lines into user-section and assistant-section.
    // The header line for each message contains the role label.
    let mut user_lines: Vec<&Line> = Vec::new();
    let mut assistant_lines: Vec<&Line> = Vec::new();
    let mut current_section: Option<&str> = None;

    for line in &lines {
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        if text.contains("User")
            && line
                .spans
                .iter()
                .any(|s| s.style.add_modifier.contains(Modifier::BOLD))
        {
            current_section = Some("user");
            continue;
        }
        if text.contains("Assistant")
            && line
                .spans
                .iter()
                .any(|s| s.style.add_modifier.contains(Modifier::BOLD))
        {
            current_section = Some("assistant");
            continue;
        }
        match current_section {
            Some("user") => user_lines.push(line),
            Some("assistant") => assistant_lines.push(line),
            _ => {}
        }
    }

    assert!(
        !user_lines.is_empty(),
        "should have user message body lines"
    );
    assert!(
        !assistant_lines.is_empty(),
        "should have assistant message body lines"
    );

    // Helper: find a span containing `needle` across a set of lines.
    let find_span = |lines: &[&Line], needle: &str| -> Option<Style> {
        for line in lines {
            for span in &line.spans {
                if span.content.contains(needle) {
                    return Some(span.style);
                }
            }
        }
        None
    };

    // -- User assertions (role color: Green) --
    let user_plain =
        find_span(&user_lines, "plain").expect("user section should contain a span with 'plain'");
    assert_eq!(
        user_plain.fg,
        Some(Color::Green),
        "user plain-text span should have fg=Green (role color), got {:?}",
        user_plain.fg,
    );

    let user_bold =
        find_span(&user_lines, "bold").expect("user section should contain a span with 'bold'");
    assert!(
        user_bold.add_modifier.contains(Modifier::BOLD),
        "user bold span should retain BOLD modifier from markdown",
    );
    assert_eq!(
        user_bold.fg,
        Some(Color::Green),
        "user bold span should have fg=Green (role color fills in unset fg)",
    );

    let user_code = find_span(&user_lines, "inline code")
        .expect("user section should contain a span with 'inline code'");
    assert_ne!(
        user_code.fg,
        Some(Color::Green),
        "user inline-code span should NOT have role color — markdown's code styling wins",
    );
    assert_eq!(
        user_code.fg,
        Some(Color::Yellow),
        "user inline-code span should retain markdown's Yellow code color",
    );

    // -- Assistant assertions (role color: Blue) --
    let asst_plain = find_span(&assistant_lines, "plain")
        .expect("assistant section should contain a span with 'plain'");
    assert_eq!(
        asst_plain.fg,
        Some(Color::Blue),
        "assistant plain-text span should have fg=Blue (role color), got {:?}",
        asst_plain.fg,
    );

    let asst_bold = find_span(&assistant_lines, "bold")
        .expect("assistant section should contain a span with 'bold'");
    assert!(
        asst_bold.add_modifier.contains(Modifier::BOLD),
        "assistant bold span should retain BOLD modifier from markdown",
    );
    assert_eq!(
        asst_bold.fg,
        Some(Color::Blue),
        "assistant bold span should have fg=Blue (role color fills in unset fg)",
    );

    let asst_code = find_span(&assistant_lines, "inline code")
        .expect("assistant section should contain a span with 'inline code'");
    assert_ne!(
        asst_code.fg,
        Some(Color::Blue),
        "assistant inline-code span should NOT have role color — markdown's code styling wins",
    );

    // -- Cross-role differentiation (the original complaint) --
    assert_ne!(
        user_plain.fg, asst_plain.fg,
        "user and assistant plain-text spans must have DIFFERENT fg colors",
    );
}
