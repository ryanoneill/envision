use super::highlight::Language;
use super::*;
use crate::component::test_utils;

fn focused_state() -> CodeBlockState {
    CodeBlockState::new()
}

fn code_state() -> CodeBlockState {
    let mut state = focused_state();
    state.set_code(
        "fn main() {\n\
         \x20   println!(\"hello\");\n\
         \x20   let x = 42;\n\
         \x20   let y = x + 1;\n\
         \x20   println!(\"{}\", y);\n\
         \x20   if x > 0 {\n\
         \x20       return;\n\
         \x20   }\n\
         }",
    );
    state.set_language(Language::Rust);
    state
}

// =============================================================================
// Construction
// =============================================================================

#[test]
fn test_new() {
    let state = CodeBlockState::new();
    assert!(state.code().is_empty());
    assert_eq!(state.scroll_offset(), 0);
    assert_eq!(state.title(), None);
    assert_eq!(state.language(), &Language::Plain);
    assert!(!state.show_line_numbers());
    assert!(state.highlighted_lines().is_empty());
}

#[test]
fn test_default() {
    let state = CodeBlockState::default();
    assert!(state.code().is_empty());
    assert_eq!(state.scroll_offset(), 0);
    assert_eq!(state.language(), &Language::Plain);
}

#[test]
fn test_with_code() {
    let state = CodeBlockState::new().with_code("fn main() {}");
    assert_eq!(state.code(), "fn main() {}");
}

#[test]
fn test_with_language() {
    let state = CodeBlockState::new().with_language(Language::Rust);
    assert_eq!(state.language(), &Language::Rust);
}

#[test]
fn test_with_title() {
    let state = CodeBlockState::new().with_title("main.rs");
    assert_eq!(state.title(), Some("main.rs"));
}

#[test]
fn test_with_line_numbers() {
    let state = CodeBlockState::new().with_line_numbers(true);
    assert!(state.show_line_numbers());
}

#[test]
fn test_with_highlight_lines() {
    let state = CodeBlockState::new().with_highlight_lines(vec![1, 3, 5]);
    assert!(state.is_line_highlighted(1));
    assert!(!state.is_line_highlighted(2));
    assert!(state.is_line_highlighted(3));
    assert!(state.is_line_highlighted(5));
}

#[test]
fn test_builder_chaining() {
    let state = CodeBlockState::new()
        .with_code("let x = 1;")
        .with_language(Language::Rust)
        .with_title("test.rs")
        .with_line_numbers(true)
        .with_highlight_lines(vec![1]);

    assert_eq!(state.code(), "let x = 1;");
    assert_eq!(state.language(), &Language::Rust);
    assert_eq!(state.title(), Some("test.rs"));
    assert!(state.show_line_numbers());
    assert!(state.is_line_highlighted(1));
}

// =============================================================================
// Code management
// =============================================================================

#[test]
fn test_set_code() {
    let mut state = CodeBlockState::new();
    state.set_scroll_offset(5);
    state.set_code("new code");
    assert_eq!(state.code(), "new code");
    assert_eq!(state.scroll_offset(), 0); // Reset on set_code
}

#[test]
fn test_line_count() {
    let state = CodeBlockState::new().with_code("a\nb\nc");
    assert_eq!(state.line_count(), 3);
}

#[test]
fn test_line_count_empty() {
    let state = CodeBlockState::new();
    assert_eq!(state.line_count(), 1);
}

#[test]
fn test_line_count_single_line() {
    let state = CodeBlockState::new().with_code("hello");
    assert_eq!(state.line_count(), 1);
}

// =============================================================================
// Language management
// =============================================================================

#[test]
fn test_set_language() {
    let mut state = CodeBlockState::new();
    state.set_language(Language::Python);
    assert_eq!(state.language(), &Language::Python);
}

// =============================================================================
// Title management
// =============================================================================

#[test]
fn test_set_title() {
    let mut state = CodeBlockState::new();
    state.set_title(Some("Title".to_string()));
    assert_eq!(state.title(), Some("Title"));
    state.set_title(None);
    assert_eq!(state.title(), None);
}

// =============================================================================
// Line number management
// =============================================================================

#[test]
fn test_set_show_line_numbers() {
    let mut state = CodeBlockState::new();
    assert!(!state.show_line_numbers());
    state.set_show_line_numbers(true);
    assert!(state.show_line_numbers());
    state.set_show_line_numbers(false);
    assert!(!state.show_line_numbers());
}

// =============================================================================
// Highlight line management
// =============================================================================

#[test]
fn test_add_highlight_line() {
    let mut state = CodeBlockState::new();
    state.add_highlight_line(3);
    assert!(state.is_line_highlighted(3));
    assert!(!state.is_line_highlighted(4));
}

#[test]
fn test_remove_highlight_line() {
    let mut state = CodeBlockState::new().with_highlight_lines(vec![1, 2, 3]);
    state.remove_highlight_line(2);
    assert!(state.is_line_highlighted(1));
    assert!(!state.is_line_highlighted(2));
    assert!(state.is_line_highlighted(3));
}

#[test]
fn test_clear_highlights() {
    let mut state = CodeBlockState::new().with_highlight_lines(vec![1, 2, 3]);
    state.clear_highlights();
    assert!(state.highlighted_lines().is_empty());
}

#[test]
fn test_highlighted_lines_accessor() {
    let state = CodeBlockState::new().with_highlight_lines(vec![2, 4]);
    let lines = state.highlighted_lines();
    assert!(lines.contains(&2));
    assert!(lines.contains(&4));
    assert_eq!(lines.len(), 2);
}

// =============================================================================
// Scroll operations
// =============================================================================

#[test]
fn test_scroll_down() {
    let mut state = code_state();
    CodeBlock::update(&mut state, CodeBlockMessage::ScrollDown);
    assert_eq!(state.scroll_offset(), 1);
}

#[test]
fn test_scroll_up() {
    let mut state = code_state();
    state.set_scroll_offset(5);
    CodeBlock::update(&mut state, CodeBlockMessage::ScrollUp);
    assert_eq!(state.scroll_offset(), 4);
}

#[test]
fn test_scroll_up_at_top() {
    let mut state = code_state();
    CodeBlock::update(&mut state, CodeBlockMessage::ScrollUp);
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_page_up() {
    let mut state = code_state();
    state.set_scroll_offset(8);
    CodeBlock::update(&mut state, CodeBlockMessage::PageUp(5));
    assert_eq!(state.scroll_offset(), 3);
}

#[test]
fn test_page_up_at_top() {
    let mut state = code_state();
    CodeBlock::update(&mut state, CodeBlockMessage::PageUp(10));
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_page_down() {
    let mut state = code_state();
    CodeBlock::update(&mut state, CodeBlockMessage::PageDown(5));
    assert_eq!(state.scroll_offset(), 5);
}

#[test]
fn test_home() {
    let mut state = code_state();
    state.set_scroll_offset(5);
    CodeBlock::update(&mut state, CodeBlockMessage::Home);
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_end() {
    let mut state = code_state();
    CodeBlock::update(&mut state, CodeBlockMessage::End);
    // Should scroll to end (9 lines, so max offset = 9 with viewport_height = 0)
    assert!(state.scroll_offset() > 0);
}

#[test]
fn test_set_code_message() {
    let mut state = code_state();
    state.set_scroll_offset(5);
    CodeBlock::update(
        &mut state,
        CodeBlockMessage::SetCode("new code".to_string()),
    );
    assert_eq!(state.code(), "new code");
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_set_language_message() {
    let mut state = code_state();
    CodeBlock::update(&mut state, CodeBlockMessage::SetLanguage(Language::Python));
    assert_eq!(state.language(), &Language::Python);
}

#[test]
fn test_toggle_line_numbers_message() {
    let mut state = code_state();
    assert!(!state.show_line_numbers());
    CodeBlock::update(&mut state, CodeBlockMessage::ToggleLineNumbers);
    assert!(state.show_line_numbers());
    CodeBlock::update(&mut state, CodeBlockMessage::ToggleLineNumbers);
    assert!(!state.show_line_numbers());
}

#[test]
fn test_highlight_line_message() {
    let mut state = code_state();
    CodeBlock::update(&mut state, CodeBlockMessage::HighlightLine(3));
    assert!(state.is_line_highlighted(3));
}

#[test]
fn test_unhighlight_line_message() {
    let mut state = CodeBlockState::new().with_highlight_lines(vec![3]);
    CodeBlock::update(&mut state, CodeBlockMessage::UnhighlightLine(3));
    assert!(!state.is_line_highlighted(3));
}

#[test]
fn test_clear_highlights_message() {
    let mut state = CodeBlockState::new().with_highlight_lines(vec![1, 2, 3]);
    CodeBlock::update(&mut state, CodeBlockMessage::ClearHighlights);
    assert!(state.highlighted_lines().is_empty());
}

#[test]
fn test_update_returns_none() {
    // CodeBlock's Output type is (), so update always returns None
    let mut state = code_state();
    assert_eq!(
        CodeBlock::update(&mut state, CodeBlockMessage::ScrollDown),
        None
    );
}

// =============================================================================
// Disabled and unfocused guards
// =============================================================================

#[test]
fn test_disabled_ignores_events() {
    let state = focused_state();
    let msg = CodeBlock::handle_event(
        &state,
        &Event::key(KeyCode::Up),
        &EventContext::new().focused(true).disabled(true),
    );
    assert_eq!(msg, None);
}

#[test]
fn test_unfocused_ignores_events() {
    let state = CodeBlockState::new();
    let msg = CodeBlock::handle_event(&state, &Event::key(KeyCode::Up), &EventContext::default());
    assert_eq!(msg, None);
}

// =============================================================================
// Event mapping
// =============================================================================

#[test]
fn test_handle_event_up() {
    let state = focused_state();
    assert_eq!(
        CodeBlock::handle_event(
            &state,
            &Event::key(KeyCode::Up),
            &EventContext::new().focused(true)
        ),
        Some(CodeBlockMessage::ScrollUp)
    );
}

#[test]
fn test_handle_event_down() {
    let state = focused_state();
    assert_eq!(
        CodeBlock::handle_event(
            &state,
            &Event::key(KeyCode::Down),
            &EventContext::new().focused(true)
        ),
        Some(CodeBlockMessage::ScrollDown)
    );
}

#[test]
fn test_handle_event_k_j() {
    let state = focused_state();
    assert_eq!(
        CodeBlock::handle_event(
            &state,
            &Event::char('k'),
            &EventContext::new().focused(true)
        ),
        Some(CodeBlockMessage::ScrollUp)
    );
    assert_eq!(
        CodeBlock::handle_event(
            &state,
            &Event::char('j'),
            &EventContext::new().focused(true)
        ),
        Some(CodeBlockMessage::ScrollDown)
    );
}

#[test]
fn test_handle_event_page_up_down() {
    let state = focused_state();
    assert_eq!(
        CodeBlock::handle_event(
            &state,
            &Event::key(KeyCode::PageUp),
            &EventContext::new().focused(true)
        ),
        Some(CodeBlockMessage::PageUp(10))
    );
    assert_eq!(
        CodeBlock::handle_event(
            &state,
            &Event::key(KeyCode::PageDown),
            &EventContext::new().focused(true)
        ),
        Some(CodeBlockMessage::PageDown(10))
    );
}

#[test]
fn test_handle_event_ctrl_u_d() {
    let state = focused_state();
    assert_eq!(
        CodeBlock::handle_event(
            &state,
            &Event::ctrl('u'),
            &EventContext::new().focused(true)
        ),
        Some(CodeBlockMessage::PageUp(10))
    );
    assert_eq!(
        CodeBlock::handle_event(
            &state,
            &Event::ctrl('d'),
            &EventContext::new().focused(true)
        ),
        Some(CodeBlockMessage::PageDown(10))
    );
}

#[test]
fn test_handle_event_home_end() {
    let state = focused_state();
    assert_eq!(
        CodeBlock::handle_event(
            &state,
            &Event::key(KeyCode::Home),
            &EventContext::new().focused(true)
        ),
        Some(CodeBlockMessage::Home)
    );
    assert_eq!(
        CodeBlock::handle_event(
            &state,
            &Event::key(KeyCode::End),
            &EventContext::new().focused(true)
        ),
        Some(CodeBlockMessage::End)
    );
}

#[test]
#[allow(non_snake_case)]
fn test_handle_event_g_and_G() {
    let state = focused_state();
    assert_eq!(
        CodeBlock::handle_event(
            &state,
            &Event::char('g'),
            &EventContext::new().focused(true)
        ),
        Some(CodeBlockMessage::Home)
    );
    assert_eq!(
        CodeBlock::handle_event(
            &state,
            &Event::key_with(KeyCode::Char('G'), KeyModifiers::SHIFT),
            &EventContext::new().focused(true),
        ),
        Some(CodeBlockMessage::End)
    );
}

#[test]
fn test_handle_event_l_scroll_right() {
    let state = focused_state();
    assert_eq!(
        CodeBlock::handle_event(
            &state,
            &Event::char('l'),
            &EventContext::new().focused(true)
        ),
        Some(CodeBlockMessage::ScrollRight)
    );
}

#[test]
fn test_handle_event_n_toggle_line_numbers() {
    let state = focused_state();
    assert_eq!(
        CodeBlock::handle_event(
            &state,
            &Event::char('n'),
            &EventContext::new().focused(true)
        ),
        Some(CodeBlockMessage::ToggleLineNumbers)
    );
}

#[test]
fn test_handle_event_unrecognized() {
    let state = focused_state();
    assert_eq!(
        CodeBlock::handle_event(
            &state,
            &Event::char('x'),
            &EventContext::new().focused(true)
        ),
        None
    );
}

// =============================================================================
// Instance methods
// =============================================================================

#[test]
fn test_instance_update() {
    let mut state = code_state();
    state.update(CodeBlockMessage::ScrollDown);
    assert_eq!(state.scroll_offset(), 1);
}

// =============================================================================
// PartialEq
// =============================================================================

#[test]
fn test_partial_eq_same() {
    let a = CodeBlockState::new().with_code("fn main() {}");
    let b = CodeBlockState::new().with_code("fn main() {}");
    assert_eq!(a, b);
}

#[test]
fn test_partial_eq_different_code() {
    let a = CodeBlockState::new().with_code("fn main() {}");
    let b = CodeBlockState::new().with_code("fn foo() {}");
    assert_ne!(a, b);
}

#[test]
fn test_partial_eq_different_language() {
    let a = CodeBlockState::new().with_language(Language::Rust);
    let b = CodeBlockState::new().with_language(Language::Python);
    assert_ne!(a, b);
}

// =============================================================================
// Snapshot tests
// =============================================================================

#[test]
fn test_view_empty() {
    let state = CodeBlockState::new();
    let (mut terminal, theme) = test_utils::setup_render(50, 10);
    terminal
        .draw(|frame| {
            CodeBlock::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_with_rust_code() {
    let state = CodeBlockState::new()
        .with_code("fn main() {\n    println!(\"Hello!\");\n}")
        .with_language(Language::Rust)
        .with_title("main.rs");
    let (mut terminal, theme) = test_utils::setup_render(50, 10);
    terminal
        .draw(|frame| {
            CodeBlock::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_with_line_numbers() {
    let state = CodeBlockState::new()
        .with_code("fn main() {\n    println!(\"Hello!\");\n}")
        .with_language(Language::Rust)
        .with_line_numbers(true);
    let (mut terminal, theme) = test_utils::setup_render(50, 10);
    terminal
        .draw(|frame| {
            CodeBlock::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_focused() {
    let state = CodeBlockState::new()
        .with_code("let x = 1;")
        .with_language(Language::Rust);
    let (mut terminal, theme) = test_utils::setup_render(50, 10);
    terminal
        .draw(|frame| {
            CodeBlock::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_disabled() {
    let state = CodeBlockState::new()
        .with_code("let x = 1;")
        .with_language(Language::Rust);
    let (mut terminal, theme) = test_utils::setup_render(50, 10);
    terminal
        .draw(|frame| {
            CodeBlock::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).disabled(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_scrolled() {
    let code = (1..=20)
        .map(|i| format!("line {}", i))
        .collect::<Vec<_>>()
        .join("\n");
    let mut state = CodeBlockState::new().with_code(&code);
    state.set_scroll_offset(5);
    let (mut terminal, theme) = test_utils::setup_render(50, 8);
    terminal
        .draw(|frame| {
            CodeBlock::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_with_highlight_lines() {
    let state = CodeBlockState::new()
        .with_code("line 1\nline 2\nline 3\nline 4\nline 5")
        .with_line_numbers(true)
        .with_highlight_lines(vec![2, 4]);
    let (mut terminal, theme) = test_utils::setup_render(50, 10);
    terminal
        .draw(|frame| {
            CodeBlock::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_python_code() {
    let state = CodeBlockState::new()
        .with_code("def greet(name):\n    # Print greeting\n    print(f\"Hello, {name}!\")")
        .with_language(Language::Python)
        .with_title("greet.py")
        .with_line_numbers(true);
    let (mut terminal, theme) = test_utils::setup_render(60, 8);
    terminal
        .draw(|frame| {
            CodeBlock::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
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
    let state = CodeBlockState::new().with_code("fn main() {}");
    let (mut terminal, theme) = test_utils::setup_render(40, 5);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                CodeBlock::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
            })
            .unwrap();
    });
    assert_eq!(registry.len(), 1);
    let regions = registry.find_by_type(&WidgetType::Custom("CodeBlock".to_string()));
    assert_eq!(regions.len(), 1);
    assert!(!regions[0].annotation.focused);
    assert!(!regions[0].annotation.disabled);
}

#[test]
fn test_annotation_focused() {
    use crate::annotation::{WidgetType, with_annotations};
    let state = CodeBlockState::new().with_code("fn main() {}");
    let (mut terminal, theme) = test_utils::setup_render(40, 5);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                CodeBlock::view(
                    &state,
                    &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
                );
            })
            .unwrap();
    });
    let regions = registry.find_by_type(&WidgetType::Custom("CodeBlock".to_string()));
    assert_eq!(regions.len(), 1);
    assert!(regions[0].annotation.focused);
}

// =============================================================================
// Edge cases
// =============================================================================

#[test]
fn test_zero_size_render() {
    let state = CodeBlockState::new().with_code("fn main() {}");
    let (mut terminal, theme) = test_utils::setup_render(3, 3);
    terminal
        .draw(|frame| {
            CodeBlock::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    // Should not panic
}

#[test]
fn test_single_line_render() {
    let state = CodeBlockState::new().with_code("hello");
    let (mut terminal, theme) = test_utils::setup_render(30, 5);
    terminal
        .draw(|frame| {
            CodeBlock::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_scroll_beyond_content() {
    let mut state = CodeBlockState::new().with_code("a\nb\nc");
    state.set_scroll_offset(100); // Way beyond content
    let (mut terminal, theme) = test_utils::setup_render(30, 5);
    terminal
        .draw(|frame| {
            CodeBlock::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    // Should not panic; scroll is clamped during render
}

// =============================================================================
// Horizontal scroll
// =============================================================================

#[test]
fn test_horizontal_scroll_right() {
    let mut state = CodeBlockState::new().with_code("Hello, World!");
    assert_eq!(state.horizontal_offset(), 0);

    CodeBlock::update(&mut state, CodeBlockMessage::ScrollRight);
    assert_eq!(state.horizontal_offset(), 1);

    CodeBlock::update(&mut state, CodeBlockMessage::ScrollRight);
    assert_eq!(state.horizontal_offset(), 2);
}

#[test]
fn test_horizontal_scroll_left() {
    let mut state = CodeBlockState::new().with_code("Hello, World!");
    state.set_horizontal_offset(5);

    CodeBlock::update(&mut state, CodeBlockMessage::ScrollLeft);
    assert_eq!(state.horizontal_offset(), 4);
}

#[test]
fn test_horizontal_scroll_left_at_zero() {
    let mut state = CodeBlockState::new().with_code("Hello");

    CodeBlock::update(&mut state, CodeBlockMessage::ScrollLeft);
    assert_eq!(state.horizontal_offset(), 0); // Cannot go below 0
}

#[test]
fn test_horizontal_scroll_clamped_to_max_width() {
    let mut state = CodeBlockState::new().with_code("abc"); // 3 chars

    for _ in 0..10 {
        CodeBlock::update(&mut state, CodeBlockMessage::ScrollRight);
    }
    // Should not exceed line length
    assert!(state.horizontal_offset() <= 3);
}

#[test]
fn test_home_resets_horizontal_scroll() {
    let mut state = CodeBlockState::new().with_code("Hello, World!");
    state.set_horizontal_offset(5);
    state.set_scroll_offset(3);

    CodeBlock::update(&mut state, CodeBlockMessage::Home);
    assert_eq!(state.horizontal_offset(), 0);
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_set_code_resets_horizontal_scroll() {
    let mut state = CodeBlockState::new().with_code("Hello");
    state.set_horizontal_offset(3);

    CodeBlock::update(
        &mut state,
        CodeBlockMessage::SetCode("New code".to_string()),
    );
    assert_eq!(state.horizontal_offset(), 0);
}

#[test]
fn test_horizontal_scroll_key_bindings() {
    let state = CodeBlockState::new().with_code("Long line of code here");
    // Left arrow
    let msg = CodeBlock::handle_event(
        &state,
        &Event::key(KeyCode::Left),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(CodeBlockMessage::ScrollLeft));

    // Right arrow
    let msg = CodeBlock::handle_event(
        &state,
        &Event::key(KeyCode::Right),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(CodeBlockMessage::ScrollRight));

    // h key
    let msg = CodeBlock::handle_event(
        &state,
        &Event::char('h'),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(CodeBlockMessage::ScrollLeft));

    // l key (now horizontal scroll, not toggle line numbers)
    let msg = CodeBlock::handle_event(
        &state,
        &Event::char('l'),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(CodeBlockMessage::ScrollRight));

    // n key (toggle line numbers)
    let msg = CodeBlock::handle_event(
        &state,
        &Event::char('n'),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(CodeBlockMessage::ToggleLineNumbers));
}

#[test]
fn test_horizontal_scroll_renders_shifted_content() {
    let code = "resource \"aws_instance\" \"web\" {\n  ami = \"ami-0c55b159\"\n}";
    let mut state = CodeBlockState::new()
        .with_code(code)
        .with_language(Language::Hcl);
    state.set_horizontal_offset(10);

    let (mut terminal, theme) = crate::component::test_utils::setup_render(33, 7);
    terminal
        .draw(|frame| {
            CodeBlock::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
            );
        })
        .unwrap();

    let output = terminal.backend().to_string();
    // After scrolling 10 chars right, "resource \"" is gone,
    // the visible part should start with "aws_instance"
    assert!(
        output.contains("aws_instance"),
        "Should see aws_instance after scrolling right 10 chars, got:\n{}",
        output
    );
    // "resource" should NOT be visible
    assert!(
        !output.contains("resource"),
        "resource should be scrolled off-screen, got:\n{}",
        output
    );
}
