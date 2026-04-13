use super::*;
use crate::component::test_utils;

// =============================================================================
// Snapshot tests
// =============================================================================

#[test]
fn test_snapshot_empty() {
    let state = MultiProgressState::new();
    let (mut terminal, theme) = test_utils::setup_render(50, 10);
    terminal
        .draw(|frame| {
            MultiProgress::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_with_items() {
    let mut state = MultiProgressState::new().with_title("Downloads");
    state.add("file1", "Document.pdf");
    state.add("file2", "Image.png");
    state.add("file3", "Archive.zip");
    let (mut terminal, theme) = test_utils::setup_render(50, 10);
    terminal
        .draw(|frame| {
            MultiProgress::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_active_progress() {
    let mut state = MultiProgressState::new().with_title("Progress");
    state.add("task1", "Build");
    state.add("task2", "Test");
    MultiProgress::update(
        &mut state,
        MultiProgressMessage::SetProgress {
            id: "task1".to_string(),
            progress: 0.5,
        },
    );
    MultiProgress::update(
        &mut state,
        MultiProgressMessage::SetProgress {
            id: "task2".to_string(),
            progress: 0.25,
        },
    );
    let (mut terminal, theme) = test_utils::setup_render(50, 10);
    terminal
        .draw(|frame| {
            MultiProgress::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_completed_and_failed() {
    let mut state = MultiProgressState::new().with_title("Tasks");
    state.add("t1", "Compile");
    state.add("t2", "Lint");
    state.add("t3", "Deploy");
    MultiProgress::update(&mut state, MultiProgressMessage::Complete("t1".to_string()));
    MultiProgress::update(
        &mut state,
        MultiProgressMessage::Fail {
            id: "t3".to_string(),
            message: Some("Timeout".to_string()),
        },
    );
    MultiProgress::update(
        &mut state,
        MultiProgressMessage::SetProgress {
            id: "t2".to_string(),
            progress: 0.75,
        },
    );
    let (mut terminal, theme) = test_utils::setup_render(50, 10);
    terminal
        .draw(|frame| {
            MultiProgress::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_without_percentages() {
    let mut state = MultiProgressState::new().with_show_percentages(false);
    state.add("a", "Task A");
    state.add("b", "Task B");
    let (mut terminal, theme) = test_utils::setup_render(50, 10);
    terminal
        .draw(|frame| {
            MultiProgress::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_disabled() {
    let mut state = MultiProgressState::new().with_title("Disabled");
    state.add("x", "Item X");
    let (mut terminal, theme) = test_utils::setup_render(50, 10);
    terminal
        .draw(|frame| {
            MultiProgress::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).disabled(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}
