use super::*;

#[test]
fn test_new() {
    let state = ProgressBarState::new();
    assert_eq!(state.progress(), 0.0);
    assert_eq!(state.percentage(), 0);
    assert!(!state.is_complete());
    assert!(state.label().is_none());
}

#[test]
fn test_default() {
    let state = ProgressBarState::default();
    assert_eq!(state.progress(), 0.0);
    assert_eq!(state.percentage(), 0);
}

#[test]
fn test_with_progress() {
    let state = ProgressBarState::with_progress(0.5);
    assert_eq!(state.progress(), 0.5);
    assert_eq!(state.percentage(), 50);
}

#[test]
fn test_with_progress_clamps() {
    let state = ProgressBarState::with_progress(1.5);
    assert_eq!(state.progress(), 1.0);

    let state = ProgressBarState::with_progress(-0.5);
    assert_eq!(state.progress(), 0.0);
}

#[test]
fn test_with_label() {
    let state = ProgressBarState::with_label("Loading...");
    assert_eq!(state.label(), Some("Loading..."));
    assert_eq!(state.progress(), 0.0);
}

#[test]
fn test_progress_accessors() {
    let mut state = ProgressBarState::new();

    state.set_progress(0.75);
    assert_eq!(state.progress(), 0.75);
    assert_eq!(state.percentage(), 75);
}

#[test]
fn test_label_accessors() {
    let mut state = ProgressBarState::new();
    assert!(state.label().is_none());

    state.set_label(Some("Test".to_string()));
    assert_eq!(state.label(), Some("Test"));

    state.set_label(None);
    assert!(state.label().is_none());
}

#[test]
fn test_is_complete() {
    let mut state = ProgressBarState::new();
    assert!(!state.is_complete());

    state.set_progress(0.99);
    assert!(!state.is_complete());

    state.set_progress(1.0);
    assert!(state.is_complete());

    state.set_progress(1.5); // Clamped to 1.0
    assert!(state.is_complete());
}

#[test]
fn test_set_progress_clamps() {
    let mut state = ProgressBarState::new();

    state.set_progress(-0.5);
    assert_eq!(state.progress(), 0.0);

    state.set_progress(1.5);
    assert_eq!(state.progress(), 1.0);

    state.set_progress(0.5);
    assert_eq!(state.progress(), 0.5);
}

#[test]
fn test_set_progress_emits_completed() {
    let mut state = ProgressBarState::new();

    // Not complete yet
    let output = ProgressBar::update(&mut state, ProgressMessage::SetProgress(0.5));
    assert_eq!(output, None);

    // Now complete
    let output = ProgressBar::update(&mut state, ProgressMessage::SetProgress(1.0));
    assert_eq!(output, Some(ProgressOutput::Completed));

    // Already complete, no output
    let output = ProgressBar::update(&mut state, ProgressMessage::SetProgress(1.0));
    assert_eq!(output, None);
}

#[test]
fn test_increment() {
    let mut state = ProgressBarState::new();

    ProgressBar::update(&mut state, ProgressMessage::Increment(0.25));
    assert_eq!(state.progress(), 0.25);

    ProgressBar::update(&mut state, ProgressMessage::Increment(0.25));
    assert_eq!(state.progress(), 0.5);

    ProgressBar::update(&mut state, ProgressMessage::Increment(0.25));
    assert_eq!(state.progress(), 0.75);
}

#[test]
fn test_increment_clamps() {
    let mut state = ProgressBarState::with_progress(0.9);

    let output = ProgressBar::update(&mut state, ProgressMessage::Increment(0.5));
    assert_eq!(state.progress(), 1.0);
    assert_eq!(output, Some(ProgressOutput::Completed));
}

#[test]
fn test_complete() {
    let mut state = ProgressBarState::with_progress(0.5);

    let output = ProgressBar::update(&mut state, ProgressMessage::Complete);
    assert_eq!(state.progress(), 1.0);
    assert!(state.is_complete());
    assert_eq!(output, Some(ProgressOutput::Completed));
}

#[test]
fn test_complete_when_already_complete() {
    let mut state = ProgressBarState::with_progress(1.0);

    // Even if already complete, Complete message still emits Completed
    let output = ProgressBar::update(&mut state, ProgressMessage::Complete);
    assert_eq!(output, Some(ProgressOutput::Completed));
}

#[test]
fn test_reset() {
    let mut state = ProgressBarState::with_progress(0.75);

    let output = ProgressBar::update(&mut state, ProgressMessage::Reset);
    assert_eq!(state.progress(), 0.0);
    assert_eq!(output, None);
}

#[test]
fn test_reset_from_complete() {
    let mut state = ProgressBarState::with_progress(1.0);
    assert!(state.is_complete());

    ProgressBar::update(&mut state, ProgressMessage::Reset);
    assert_eq!(state.progress(), 0.0);
    assert!(!state.is_complete());
}

#[test]
fn test_clone() {
    let mut state = ProgressBarState::with_progress(0.5);
    state.set_label(Some("Test".to_string()));

    let cloned = state.clone();
    assert_eq!(cloned.progress(), 0.5);
    assert_eq!(cloned.label(), Some("Test"));
}

#[test]
fn test_init() {
    let state = ProgressBar::init();
    assert_eq!(state.progress(), 0.0);
    assert!(state.label().is_none());
}

#[test]
fn test_percentage_rounding() {
    let mut state = ProgressBarState::new();

    state.set_progress(0.334);
    assert_eq!(state.percentage(), 33);

    state.set_progress(0.335);
    assert_eq!(state.percentage(), 34);

    state.set_progress(0.999);
    assert_eq!(state.percentage(), 100);
}

#[test]
fn test_view_renders() {
    let mut state = ProgressBarState::with_progress(0.5);
    state.set_label(Some("Loading".to_string()));
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 5);

    terminal
        .draw(|frame| {
            ProgressBar::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    let output = terminal.backend().to_string();
    assert!(output.contains("50%"));
    assert!(output.contains("Loading"));
}

#[test]
fn test_view_without_label() {
    let state = ProgressBarState::with_progress(0.75);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 5);

    terminal
        .draw(|frame| {
            ProgressBar::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    let output = terminal.backend().to_string();
    assert!(output.contains("75%"));
}

#[test]
fn test_full_workflow() {
    let mut state = ProgressBarState::with_label("Downloading");

    // Start
    assert_eq!(state.percentage(), 0);

    // Progress updates
    ProgressBar::update(&mut state, ProgressMessage::SetProgress(0.25));
    assert_eq!(state.percentage(), 25);

    ProgressBar::update(&mut state, ProgressMessage::Increment(0.25));
    assert_eq!(state.percentage(), 50);

    ProgressBar::update(&mut state, ProgressMessage::Increment(0.25));
    assert_eq!(state.percentage(), 75);

    // Complete
    let output = ProgressBar::update(&mut state, ProgressMessage::Complete);
    assert_eq!(output, Some(ProgressOutput::Completed));
    assert!(state.is_complete());

    // Reset for reuse
    ProgressBar::update(&mut state, ProgressMessage::Reset);
    assert_eq!(state.percentage(), 0);
    assert!(!state.is_complete());
}
