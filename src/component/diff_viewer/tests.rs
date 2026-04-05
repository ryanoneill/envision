use super::*;
use crate::component::test_utils;
use crate::input::Event;

fn focused_state() -> DiffViewerState {
    let mut state = DiffViewerState::new();
    DiffViewer::set_focused(&mut state, true);
    state
}

fn sample_diff_text() -> &'static str {
    "\
--- a/file.rs
+++ b/file.rs
@@ -1,5 +1,6 @@
 fn main() {
-    println!(\"hello\");
-    return;
+    println!(\"world\");
+    eprintln!(\"debug\");
+    return Ok(());
 }
"
}

fn sample_state() -> DiffViewerState {
    let mut state = DiffViewerState::from_diff(sample_diff_text());
    state.set_focused(true);
    state
}

fn two_hunk_state() -> DiffViewerState {
    let diff = "\
--- a/file.rs
+++ b/file.rs
@@ -1,3 +1,3 @@
 first
-old1
+new1
 middle
@@ -10,3 +10,3 @@
 second
-old2
+new2
 end
";
    let mut state = DiffViewerState::from_diff(diff);
    state.set_focused(true);
    state
}

// =============================================================================
// Construction
// =============================================================================

#[test]
fn test_new() {
    let state = DiffViewerState::new();
    assert!(state.hunks().is_empty());
    assert_eq!(state.current_hunk(), 0);
    assert_eq!(state.hunk_count(), 0);
    assert_eq!(state.total_lines(), 0);
    assert!(!state.is_focused());
    assert!(!state.is_disabled());
    assert_eq!(*state.mode(), DiffMode::Unified);
}

#[test]
fn test_default() {
    let state = DiffViewerState::default();
    assert!(state.hunks().is_empty());
    assert_eq!(state.current_hunk(), 0);
}

#[test]
fn test_from_diff() {
    let state = DiffViewerState::from_diff(sample_diff_text());
    assert_eq!(state.hunk_count(), 1);
    assert!(state.total_lines() > 0);
}

#[test]
fn test_from_texts() {
    let old = "fn main() {\n    println!(\"old\");\n}";
    let new = "fn main() {\n    println!(\"new\");\n}";
    let state = DiffViewerState::from_texts(old, new);
    assert_eq!(state.hunk_count(), 1);
    assert!(state.added_count() >= 1);
    assert!(state.removed_count() >= 1);
}

#[test]
fn test_from_texts_identical() {
    let text = "fn main() {\n    println!(\"same\");\n}";
    let state = DiffViewerState::from_texts(text, text);
    assert_eq!(state.hunk_count(), 0);
    assert_eq!(state.added_count(), 0);
    assert_eq!(state.removed_count(), 0);
}

// =============================================================================
// Builder methods
// =============================================================================

#[test]
fn test_with_mode() {
    let state = DiffViewerState::new().with_mode(DiffMode::SideBySide);
    assert_eq!(*state.mode(), DiffMode::SideBySide);
}

#[test]
fn test_with_title() {
    let state = DiffViewerState::new().with_title("My Diff");
    assert_eq!(state.title.as_deref(), Some("My Diff"));
}

#[test]
fn test_with_labels() {
    let state = DiffViewerState::new()
        .with_old_label("original.rs")
        .with_new_label("modified.rs");
    assert_eq!(state.old_label.as_deref(), Some("original.rs"));
    assert_eq!(state.new_label.as_deref(), Some("modified.rs"));
}

#[test]
fn test_with_context_lines() {
    let state = DiffViewerState::new().with_context_lines(5);
    assert_eq!(state.context_lines, 5);
}

#[test]
fn test_with_show_line_numbers() {
    let state = DiffViewerState::new().with_show_line_numbers(false);
    assert!(!state.show_line_numbers);
}

#[test]
fn test_with_disabled() {
    let state = DiffViewerState::new().with_disabled(true);
    assert!(state.is_disabled());
}

// =============================================================================
// Counts
// =============================================================================

#[test]
fn test_added_count() {
    let state = sample_state();
    assert_eq!(state.added_count(), 3);
}

#[test]
fn test_removed_count() {
    let state = sample_state();
    assert_eq!(state.removed_count(), 2);
}

#[test]
fn test_changed_count() {
    let state = sample_state();
    assert_eq!(state.changed_count(), 5);
}

#[test]
fn test_counts_empty() {
    let state = DiffViewerState::new();
    assert_eq!(state.added_count(), 0);
    assert_eq!(state.removed_count(), 0);
    assert_eq!(state.changed_count(), 0);
}

// =============================================================================
// Hunk navigation
// =============================================================================

#[test]
fn test_next_hunk() {
    let mut state = two_hunk_state();
    assert_eq!(state.current_hunk(), 0);

    let output = DiffViewer::update(&mut state, DiffViewerMessage::NextHunk);
    assert_eq!(state.current_hunk(), 1);
    assert_eq!(output, Some(DiffViewerOutput::HunkChanged(1)));
}

#[test]
fn test_next_hunk_wraps() {
    let mut state = two_hunk_state();
    DiffViewer::update(&mut state, DiffViewerMessage::NextHunk); // -> 1
    let output = DiffViewer::update(&mut state, DiffViewerMessage::NextHunk); // -> 0 (wrap)
    assert_eq!(state.current_hunk(), 0);
    assert_eq!(output, Some(DiffViewerOutput::HunkChanged(0)));
}

#[test]
fn test_prev_hunk() {
    let mut state = two_hunk_state();
    DiffViewer::update(&mut state, DiffViewerMessage::NextHunk); // -> 1
    let output = DiffViewer::update(&mut state, DiffViewerMessage::PrevHunk); // -> 0
    assert_eq!(state.current_hunk(), 0);
    assert_eq!(output, Some(DiffViewerOutput::HunkChanged(0)));
}

#[test]
fn test_prev_hunk_wraps() {
    let mut state = two_hunk_state();
    assert_eq!(state.current_hunk(), 0);
    let output = DiffViewer::update(&mut state, DiffViewerMessage::PrevHunk); // -> 1 (wrap)
    assert_eq!(state.current_hunk(), 1);
    assert_eq!(output, Some(DiffViewerOutput::HunkChanged(1)));
}

#[test]
fn test_next_hunk_empty() {
    let mut state = focused_state();
    let output = DiffViewer::update(&mut state, DiffViewerMessage::NextHunk);
    assert_eq!(output, None);
}

#[test]
fn test_prev_hunk_empty() {
    let mut state = focused_state();
    let output = DiffViewer::update(&mut state, DiffViewerMessage::PrevHunk);
    assert_eq!(output, None);
}

#[test]
fn test_single_hunk_next_wraps_to_same() {
    let mut state = sample_state(); // 1 hunk
    let output = DiffViewer::update(&mut state, DiffViewerMessage::NextHunk);
    // Wraps from 0 back to 0 -> no change
    assert_eq!(output, None);
    assert_eq!(state.current_hunk(), 0);
}

// =============================================================================
// Scroll operations
// =============================================================================

#[test]
fn test_scroll_down() {
    let mut state = sample_state();
    DiffViewer::update(&mut state, DiffViewerMessage::ScrollDown);
    assert_eq!(state.scroll_offset(), 1);
}

#[test]
fn test_scroll_up() {
    let mut state = sample_state();
    DiffViewer::update(&mut state, DiffViewerMessage::ScrollDown);
    DiffViewer::update(&mut state, DiffViewerMessage::ScrollDown);
    DiffViewer::update(&mut state, DiffViewerMessage::ScrollUp);
    assert_eq!(state.scroll_offset(), 1);
}

#[test]
fn test_scroll_up_at_top() {
    let mut state = sample_state();
    DiffViewer::update(&mut state, DiffViewerMessage::ScrollUp);
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_page_down() {
    let mut state = sample_state();
    DiffViewer::update(&mut state, DiffViewerMessage::PageDown(5));
    assert!(state.scroll_offset() > 0);
}

#[test]
fn test_page_up() {
    let mut state = sample_state();
    DiffViewer::update(&mut state, DiffViewerMessage::PageDown(5));
    let after_down = state.scroll_offset();
    DiffViewer::update(&mut state, DiffViewerMessage::PageUp(5));
    assert!(state.scroll_offset() < after_down);
}

#[test]
fn test_home() {
    let mut state = sample_state();
    DiffViewer::update(&mut state, DiffViewerMessage::ScrollDown);
    DiffViewer::update(&mut state, DiffViewerMessage::ScrollDown);
    DiffViewer::update(&mut state, DiffViewerMessage::Home);
    assert_eq!(state.scroll_offset(), 0);
    assert_eq!(state.current_hunk(), 0);
}

#[test]
fn test_end() {
    let mut state = two_hunk_state();
    DiffViewer::update(&mut state, DiffViewerMessage::End);
    assert_eq!(state.current_hunk(), state.hunk_count() - 1);
}

// =============================================================================
// Mode toggle
// =============================================================================

#[test]
fn test_toggle_mode() {
    let mut state = sample_state();
    assert_eq!(*state.mode(), DiffMode::Unified);

    let output = DiffViewer::update(&mut state, DiffViewerMessage::ToggleMode);
    assert_eq!(*state.mode(), DiffMode::SideBySide);
    assert_eq!(
        output,
        Some(DiffViewerOutput::ModeChanged(DiffMode::SideBySide))
    );

    let output = DiffViewer::update(&mut state, DiffViewerMessage::ToggleMode);
    assert_eq!(*state.mode(), DiffMode::Unified);
    assert_eq!(
        output,
        Some(DiffViewerOutput::ModeChanged(DiffMode::Unified))
    );
}

#[test]
fn test_set_mode() {
    let mut state = sample_state();
    let output = DiffViewer::update(&mut state, DiffViewerMessage::SetMode(DiffMode::SideBySide));
    assert_eq!(*state.mode(), DiffMode::SideBySide);
    assert_eq!(
        output,
        Some(DiffViewerOutput::ModeChanged(DiffMode::SideBySide))
    );
}

#[test]
fn test_set_mode_same() {
    let mut state = sample_state();
    let output = DiffViewer::update(&mut state, DiffViewerMessage::SetMode(DiffMode::Unified));
    assert_eq!(output, None);
}

// =============================================================================
// Messages: SetDiff, SetTexts, SetHunks, Clear
// =============================================================================

#[test]
fn test_set_diff_message() {
    let mut state = focused_state();
    DiffViewer::update(
        &mut state,
        DiffViewerMessage::SetDiff(sample_diff_text().to_string()),
    );
    assert_eq!(state.hunk_count(), 1);
    assert_eq!(state.current_hunk(), 0);
}

#[test]
fn test_set_texts_message() {
    let mut state = focused_state();
    DiffViewer::update(
        &mut state,
        DiffViewerMessage::SetTexts {
            old: "old line".to_string(),
            new: "new line".to_string(),
        },
    );
    assert_eq!(state.hunk_count(), 1);
}

#[test]
fn test_set_hunks_message() {
    let hunks = vec![DiffHunk {
        header: "@@ -1,1 +1,1 @@".to_string(),
        old_start: 1,
        new_start: 1,
        lines: vec![
            DiffLine {
                line_type: DiffLineType::Header,
                content: "@@ -1,1 +1,1 @@".to_string(),
                old_line_num: None,
                new_line_num: None,
            },
            DiffLine {
                line_type: DiffLineType::Removed,
                content: "old".to_string(),
                old_line_num: Some(1),
                new_line_num: None,
            },
            DiffLine {
                line_type: DiffLineType::Added,
                content: "new".to_string(),
                old_line_num: None,
                new_line_num: Some(1),
            },
        ],
    }];

    let mut state = focused_state();
    DiffViewer::update(&mut state, DiffViewerMessage::SetHunks(hunks));
    assert_eq!(state.hunk_count(), 1);
    assert_eq!(state.added_count(), 1);
    assert_eq!(state.removed_count(), 1);
}

#[test]
fn test_clear_message() {
    let mut state = sample_state();
    assert!(state.hunk_count() > 0);
    DiffViewer::update(&mut state, DiffViewerMessage::Clear);
    assert_eq!(state.hunk_count(), 0);
    assert_eq!(state.current_hunk(), 0);
    assert_eq!(state.total_lines(), 0);
}

// =============================================================================
// Disabled and unfocused guards
// =============================================================================

#[test]
fn test_disabled_ignores_events() {
    let mut state = focused_state();
    state.set_disabled(true);
    let msg = DiffViewer::handle_event(
        &state,
        &Event::key(KeyCode::Up),
        &ViewContext::new().focused(true).disabled(true),
    );
    assert_eq!(msg, None);
}

#[test]
fn test_unfocused_ignores_events() {
    let state = DiffViewerState::new();
    let msg = DiffViewer::handle_event(&state, &Event::key(KeyCode::Up), &ViewContext::default());
    assert_eq!(msg, None);
}

// =============================================================================
// Event mapping
// =============================================================================

#[test]
fn test_handle_event_up() {
    let state = focused_state();
    assert_eq!(
        DiffViewer::handle_event(
            &state,
            &Event::key(KeyCode::Up),
            &ViewContext::new().focused(true)
        ),
        Some(DiffViewerMessage::ScrollUp)
    );
}

#[test]
fn test_handle_event_down() {
    let state = focused_state();
    assert_eq!(
        DiffViewer::handle_event(
            &state,
            &Event::key(KeyCode::Down),
            &ViewContext::new().focused(true)
        ),
        Some(DiffViewerMessage::ScrollDown)
    );
}

#[test]
fn test_handle_event_k_j() {
    let state = focused_state();
    assert_eq!(
        DiffViewer::handle_event(&state, &Event::char('k'), &ViewContext::new().focused(true)),
        Some(DiffViewerMessage::ScrollUp)
    );
    assert_eq!(
        DiffViewer::handle_event(&state, &Event::char('j'), &ViewContext::new().focused(true)),
        Some(DiffViewerMessage::ScrollDown)
    );
}

#[test]
fn test_handle_event_n_for_next_hunk() {
    let state = focused_state();
    assert_eq!(
        DiffViewer::handle_event(&state, &Event::char('n'), &ViewContext::new().focused(true)),
        Some(DiffViewerMessage::NextHunk)
    );
}

#[test]
fn test_handle_event_shift_n_for_prev_hunk() {
    let state = focused_state();
    assert_eq!(
        DiffViewer::handle_event(
            &state,
            &Event::key_with(KeyCode::Char('N'), KeyModifiers::SHIFT),
            &ViewContext::new().focused(true),
        ),
        Some(DiffViewerMessage::PrevHunk)
    );
}

#[test]
fn test_handle_event_p_for_prev_hunk() {
    let state = focused_state();
    assert_eq!(
        DiffViewer::handle_event(&state, &Event::char('p'), &ViewContext::new().focused(true)),
        Some(DiffViewerMessage::PrevHunk)
    );
}

#[test]
fn test_handle_event_page_up_down() {
    let state = focused_state();
    assert_eq!(
        DiffViewer::handle_event(
            &state,
            &Event::key(KeyCode::PageUp),
            &ViewContext::new().focused(true)
        ),
        Some(DiffViewerMessage::PageUp(10))
    );
    assert_eq!(
        DiffViewer::handle_event(
            &state,
            &Event::key(KeyCode::PageDown),
            &ViewContext::new().focused(true)
        ),
        Some(DiffViewerMessage::PageDown(10))
    );
}

#[test]
fn test_handle_event_ctrl_u_d() {
    let state = focused_state();
    assert_eq!(
        DiffViewer::handle_event(&state, &Event::ctrl('u'), &ViewContext::new().focused(true)),
        Some(DiffViewerMessage::PageUp(10))
    );
    assert_eq!(
        DiffViewer::handle_event(&state, &Event::ctrl('d'), &ViewContext::new().focused(true)),
        Some(DiffViewerMessage::PageDown(10))
    );
}

#[test]
fn test_handle_event_home_end() {
    let state = focused_state();
    assert_eq!(
        DiffViewer::handle_event(
            &state,
            &Event::key(KeyCode::Home),
            &ViewContext::new().focused(true)
        ),
        Some(DiffViewerMessage::Home)
    );
    assert_eq!(
        DiffViewer::handle_event(
            &state,
            &Event::key(KeyCode::End),
            &ViewContext::new().focused(true)
        ),
        Some(DiffViewerMessage::End)
    );
}

#[test]
#[allow(non_snake_case)]
fn test_handle_event_g_and_G() {
    let state = focused_state();
    assert_eq!(
        DiffViewer::handle_event(&state, &Event::char('g'), &ViewContext::new().focused(true)),
        Some(DiffViewerMessage::Home)
    );
    assert_eq!(
        DiffViewer::handle_event(
            &state,
            &Event::key_with(KeyCode::Char('G'), KeyModifiers::SHIFT),
            &ViewContext::new().focused(true),
        ),
        Some(DiffViewerMessage::End)
    );
}

#[test]
fn test_handle_event_m_toggle_mode() {
    let state = focused_state();
    assert_eq!(
        DiffViewer::handle_event(&state, &Event::char('m'), &ViewContext::new().focused(true)),
        Some(DiffViewerMessage::ToggleMode)
    );
}

#[test]
fn test_handle_event_unrecognized() {
    let state = focused_state();
    assert_eq!(
        DiffViewer::handle_event(&state, &Event::char('x'), &ViewContext::new().focused(true)),
        None
    );
}

// =============================================================================
// Instance methods
// =============================================================================

#[test]
fn test_instance_handle_event() {
    let state = focused_state();
    let msg = state.handle_event(&Event::key(KeyCode::Up));
    assert_eq!(msg, Some(DiffViewerMessage::ScrollUp));
}

#[test]
fn test_instance_dispatch_event() {
    let mut state = sample_state();
    let _output = state.dispatch_event(&Event::key(KeyCode::Down));
    assert_eq!(state.scroll_offset(), 1);
}

#[test]
fn test_instance_update() {
    let mut state = sample_state();
    state.update(DiffViewerMessage::ScrollDown);
    assert_eq!(state.scroll_offset(), 1);
}

// =============================================================================
// Focusable / Disableable traits
// =============================================================================

#[test]
fn test_focusable_trait() {
    let mut state = DiffViewer::init();
    assert!(!DiffViewer::is_focused(&state));

    DiffViewer::focus(&mut state);
    assert!(DiffViewer::is_focused(&state));

    DiffViewer::blur(&mut state);
    assert!(!DiffViewer::is_focused(&state));
}

#[test]
fn test_disableable_trait() {
    let mut state = DiffViewer::init();
    assert!(!DiffViewer::is_disabled(&state));

    DiffViewer::disable(&mut state);
    assert!(DiffViewer::is_disabled(&state));

    DiffViewer::enable(&mut state);
    assert!(!DiffViewer::is_disabled(&state));
}

// =============================================================================
// Side-by-side pairs
// =============================================================================

#[test]
fn test_side_by_side_pairs() {
    let state = sample_state();
    let pairs = state.collect_side_by_side_pairs();
    assert!(!pairs.is_empty());

    // First pair should be the header on both sides
    assert!(pairs[0].0.is_some());
    assert!(pairs[0].1.is_some());
    assert_eq!(pairs[0].0.as_ref().unwrap().line_type, DiffLineType::Header);
}

#[test]
fn test_side_by_side_context_on_both_sides() {
    let state = sample_state();
    let pairs = state.collect_side_by_side_pairs();

    // Find context lines - they should appear on both sides
    let context_pairs: Vec<_> = pairs
        .iter()
        .filter(|(l, r)| {
            l.as_ref()
                .map(|ll| ll.line_type == DiffLineType::Context)
                .unwrap_or(false)
                && r.as_ref()
                    .map(|rr| rr.line_type == DiffLineType::Context)
                    .unwrap_or(false)
        })
        .collect();
    assert!(!context_pairs.is_empty());
}

// =============================================================================
// Edge cases
// =============================================================================

#[test]
fn test_empty_diff() {
    let state = DiffViewerState::from_diff("");
    assert_eq!(state.hunk_count(), 0);
    assert_eq!(state.total_lines(), 0);
    assert_eq!(state.added_count(), 0);
    assert_eq!(state.removed_count(), 0);
}

#[test]
fn test_all_additions() {
    let diff = "\
--- a/file.rs
+++ b/file.rs
@@ -0,0 +1,3 @@
+line1
+line2
+line3
";
    let state = DiffViewerState::from_diff(diff);
    assert_eq!(state.added_count(), 3);
    assert_eq!(state.removed_count(), 0);
}

#[test]
fn test_all_removals() {
    let diff = "\
--- a/file.rs
+++ b/file.rs
@@ -1,3 +0,0 @@
-line1
-line2
-line3
";
    let state = DiffViewerState::from_diff(diff);
    assert_eq!(state.added_count(), 0);
    assert_eq!(state.removed_count(), 3);
}

#[test]
fn test_no_changes() {
    let text = "line1\nline2\nline3";
    let state = DiffViewerState::from_texts(text, text);
    assert_eq!(state.hunk_count(), 0);
    assert_eq!(state.changed_count(), 0);
}

// =============================================================================
// Snapshot / View tests
// =============================================================================

#[test]
fn test_view_unified_empty() {
    let state = DiffViewerState::new();
    let (mut terminal, theme) = test_utils::setup_render(50, 10);
    terminal
        .draw(|frame| {
            DiffViewer::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_unified_with_diff() {
    let state = DiffViewerState::from_diff(sample_diff_text()).with_title("Changes");
    let (mut terminal, theme) = test_utils::setup_render(60, 12);
    terminal
        .draw(|frame| {
            DiffViewer::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_unified_focused() {
    let mut state = DiffViewerState::from_diff(sample_diff_text()).with_title("Changes");
    state.set_focused(true);
    let (mut terminal, theme) = test_utils::setup_render(60, 12);
    terminal
        .draw(|frame| {
            DiffViewer::view(
                &state,
                frame,
                frame.area(),
                &theme,
                &ViewContext::new().focused(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_unified_disabled() {
    let state = DiffViewerState::from_diff(sample_diff_text())
        .with_title("Changes")
        .with_disabled(true);
    let (mut terminal, theme) = test_utils::setup_render(60, 12);
    terminal
        .draw(|frame| {
            DiffViewer::view(
                &state,
                frame,
                frame.area(),
                &theme,
                &ViewContext::new().disabled(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_side_by_side() {
    let state = DiffViewerState::from_diff(sample_diff_text())
        .with_mode(DiffMode::SideBySide)
        .with_old_label("a/file.rs")
        .with_new_label("b/file.rs");
    let (mut terminal, theme) = test_utils::setup_render(80, 12);
    terminal
        .draw(|frame| {
            DiffViewer::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_with_line_numbers() {
    let state = DiffViewerState::from_diff(sample_diff_text()).with_show_line_numbers(true);
    let (mut terminal, theme) = test_utils::setup_render(60, 12);
    terminal
        .draw(|frame| {
            DiffViewer::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_without_line_numbers() {
    let state = DiffViewerState::from_diff(sample_diff_text()).with_show_line_numbers(false);
    let (mut terminal, theme) = test_utils::setup_render(60, 12);
    terminal
        .draw(|frame| {
            DiffViewer::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

// =============================================================================
// Annotation test
// =============================================================================

#[test]
fn test_annotation_emitted() {
    use crate::annotation::{with_annotations, WidgetType};

    let state = DiffViewerState::from_diff(sample_diff_text());
    let (mut terminal, theme) = test_utils::setup_render(60, 12);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                DiffViewer::view(&state, frame, frame.area(), &theme, &ViewContext::default());
            })
            .unwrap();
    });
    assert_eq!(registry.len(), 1);
    let regions = registry.find_by_type(&WidgetType::DiffViewer);
    assert_eq!(regions.len(), 1);
    assert!(!regions[0].annotation.focused);
    assert!(!regions[0].annotation.disabled);
}
