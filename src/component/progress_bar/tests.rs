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
    let output = ProgressBar::update(&mut state, ProgressBarMessage::SetProgress(0.5));
    assert_eq!(output, None);

    // Now complete
    let output = ProgressBar::update(&mut state, ProgressBarMessage::SetProgress(1.0));
    assert_eq!(output, Some(ProgressBarOutput::Completed));

    // Already complete, no output
    let output = ProgressBar::update(&mut state, ProgressBarMessage::SetProgress(1.0));
    assert_eq!(output, None);
}

#[test]
fn test_increment() {
    let mut state = ProgressBarState::new();

    ProgressBar::update(&mut state, ProgressBarMessage::Increment(0.25));
    assert_eq!(state.progress(), 0.25);

    ProgressBar::update(&mut state, ProgressBarMessage::Increment(0.25));
    assert_eq!(state.progress(), 0.5);

    ProgressBar::update(&mut state, ProgressBarMessage::Increment(0.25));
    assert_eq!(state.progress(), 0.75);
}

#[test]
fn test_increment_clamps() {
    let mut state = ProgressBarState::with_progress(0.9);

    let output = ProgressBar::update(&mut state, ProgressBarMessage::Increment(0.5));
    assert_eq!(state.progress(), 1.0);
    assert_eq!(output, Some(ProgressBarOutput::Completed));
}

#[test]
fn test_complete() {
    let mut state = ProgressBarState::with_progress(0.5);

    let output = ProgressBar::update(&mut state, ProgressBarMessage::Complete);
    assert_eq!(state.progress(), 1.0);
    assert!(state.is_complete());
    assert_eq!(output, Some(ProgressBarOutput::Completed));
}

#[test]
fn test_complete_when_already_complete() {
    let mut state = ProgressBarState::with_progress(1.0);

    // Even if already complete, Complete message still emits Completed
    let output = ProgressBar::update(&mut state, ProgressBarMessage::Complete);
    assert_eq!(output, Some(ProgressBarOutput::Completed));
}

#[test]
fn test_reset() {
    let mut state = ProgressBarState::with_progress(0.75);

    let output = ProgressBar::update(&mut state, ProgressBarMessage::Reset);
    assert_eq!(state.progress(), 0.0);
    assert_eq!(output, None);
}

#[test]
fn test_reset_from_complete() {
    let mut state = ProgressBarState::with_progress(1.0);
    assert!(state.is_complete());

    ProgressBar::update(&mut state, ProgressBarMessage::Reset);
    assert_eq!(state.progress(), 0.0);
    assert!(!state.is_complete());
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
fn test_view_zero_progress() {
    let state = ProgressBarState::new();
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 5);

    terminal
        .draw(|frame| {
            ProgressBar::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_half_progress() {
    let state = ProgressBarState::with_progress(0.5);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 5);

    terminal
        .draw(|frame| {
            ProgressBar::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_full_progress() {
    let state = ProgressBarState::with_progress(1.0);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 5);

    terminal
        .draw(|frame| {
            ProgressBar::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_renders() {
    let mut state = ProgressBarState::with_progress(0.5);
    state.set_label(Some("Loading".to_string()));
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 5);

    terminal
        .draw(|frame| {
            ProgressBar::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_without_label() {
    let state = ProgressBarState::with_progress(0.75);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 5);

    terminal
        .draw(|frame| {
            ProgressBar::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_with_disabled() {
    let state = ProgressBarState::new().with_disabled(true);
    assert!(state.is_disabled());
}

#[test]
fn test_with_disabled_false() {
    let state = ProgressBarState::new().with_disabled(false);
    assert!(!state.is_disabled());
}

#[test]
fn test_disabled_default_is_false() {
    let state = ProgressBarState::new();
    assert!(!state.is_disabled());
}

#[test]
fn test_set_disabled() {
    let mut state = ProgressBarState::new();
    assert!(!state.is_disabled());

    state.set_disabled(true);
    assert!(state.is_disabled());

    state.set_disabled(false);
    assert!(!state.is_disabled());
}

#[test]
fn test_full_workflow() {
    let mut state = ProgressBarState::with_label("Downloading");

    // Start
    assert_eq!(state.percentage(), 0);

    // Progress updates
    ProgressBar::update(&mut state, ProgressBarMessage::SetProgress(0.25));
    assert_eq!(state.percentage(), 25);

    ProgressBar::update(&mut state, ProgressBarMessage::Increment(0.25));
    assert_eq!(state.percentage(), 50);

    ProgressBar::update(&mut state, ProgressBarMessage::Increment(0.25));
    assert_eq!(state.percentage(), 75);

    // Complete
    let output = ProgressBar::update(&mut state, ProgressBarMessage::Complete);
    assert_eq!(output, Some(ProgressBarOutput::Completed));
    assert!(state.is_complete());

    // Reset for reuse
    ProgressBar::update(&mut state, ProgressBarMessage::Reset);
    assert_eq!(state.percentage(), 0);
    assert!(!state.is_complete());
}

// ETA and Rate tests

#[test]
fn test_default_show_percentage_is_true() {
    let state = ProgressBarState::new();
    assert!(state.show_percentage());
}

#[test]
fn test_default_show_eta_is_true() {
    let state = ProgressBarState::new();
    assert!(state.show_eta());
}

#[test]
fn test_default_show_rate_is_true() {
    let state = ProgressBarState::new();
    assert!(state.show_rate());
}

#[test]
fn test_default_eta_is_none() {
    let state = ProgressBarState::new();
    assert!(state.eta().is_none());
    assert!(state.eta_millis().is_none());
}

#[test]
fn test_default_rate_text_is_none() {
    let state = ProgressBarState::new();
    assert!(state.rate_text().is_none());
}

#[test]
fn test_with_show_percentage_false() {
    let state = ProgressBarState::new().with_show_percentage(false);
    assert!(!state.show_percentage());
}

#[test]
fn test_with_show_eta_false() {
    let state = ProgressBarState::new().with_show_eta(false);
    assert!(!state.show_eta());
}

#[test]
fn test_with_show_rate_false() {
    let state = ProgressBarState::new().with_show_rate(false);
    assert!(!state.show_rate());
}

#[test]
fn test_set_eta() {
    let mut state = ProgressBarState::new();
    state.set_eta(Some(Duration::from_secs(120)));
    assert_eq!(state.eta(), Some(Duration::from_millis(120_000)));
    assert_eq!(state.eta_millis(), Some(120_000));

    state.set_eta(None);
    assert!(state.eta().is_none());
}

#[test]
fn test_set_rate_text() {
    let mut state = ProgressBarState::new();
    state.set_rate_text(Some("5.2 items/sec".to_string()));
    assert_eq!(state.rate_text(), Some("5.2 items/sec"));

    state.set_rate_text(None);
    assert!(state.rate_text().is_none());
}

#[test]
fn test_set_eta_message() {
    let mut state = ProgressBarState::new();
    let output = ProgressBar::update(
        &mut state,
        ProgressBarMessage::SetEta(Some(Duration::from_secs(60))),
    );
    assert_eq!(output, None);
    assert_eq!(state.eta_millis(), Some(60_000));
}

#[test]
fn test_set_eta_message_none() {
    let mut state = ProgressBarState::new();
    state.set_eta(Some(Duration::from_secs(60)));
    let output = ProgressBar::update(&mut state, ProgressBarMessage::SetEta(None));
    assert_eq!(output, None);
    assert!(state.eta().is_none());
}

#[test]
fn test_set_rate_text_message() {
    let mut state = ProgressBarState::new();
    let output = ProgressBar::update(
        &mut state,
        ProgressBarMessage::SetRateText(Some("10 req/s".to_string())),
    );
    assert_eq!(output, None);
    assert_eq!(state.rate_text(), Some("10 req/s"));
}

#[test]
fn test_set_rate_text_message_none() {
    let mut state = ProgressBarState::new();
    state.set_rate_text(Some("10 req/s".to_string()));
    let output = ProgressBar::update(&mut state, ProgressBarMessage::SetRateText(None));
    assert_eq!(output, None);
    assert!(state.rate_text().is_none());
}

#[test]
fn test_reset_clears_eta_and_rate() {
    let mut state = ProgressBarState::with_progress(0.5);
    state.set_eta(Some(Duration::from_secs(60)));
    state.set_rate_text(Some("5 items/sec".to_string()));

    ProgressBar::update(&mut state, ProgressBarMessage::Reset);
    assert!(state.eta().is_none());
    assert!(state.rate_text().is_none());
}

#[test]
fn test_format_eta_seconds() {
    assert_eq!(format_eta(Duration::from_secs(0)), "0s");
    assert_eq!(format_eta(Duration::from_secs(1)), "1s");
    assert_eq!(format_eta(Duration::from_secs(45)), "45s");
    assert_eq!(format_eta(Duration::from_secs(59)), "59s");
}

#[test]
fn test_format_eta_minutes() {
    assert_eq!(format_eta(Duration::from_secs(60)), "1m 00s");
    assert_eq!(format_eta(Duration::from_secs(61)), "1m 01s");
    assert_eq!(format_eta(Duration::from_secs(202)), "3m 22s");
    assert_eq!(format_eta(Duration::from_secs(3599)), "59m 59s");
}

#[test]
fn test_format_eta_hours() {
    assert_eq!(format_eta(Duration::from_secs(3600)), "1h 00m");
    assert_eq!(format_eta(Duration::from_secs(3720)), "1h 02m");
    assert_eq!(format_eta(Duration::from_secs(7200)), "2h 00m");
    assert_eq!(format_eta(Duration::from_secs(86400)), "24h 00m");
}

#[test]
fn test_label_with_percentage_only() {
    let state = ProgressBarState::with_progress(0.42);
    let label = build_label(&state);
    assert_eq!(label, "42%");
}

#[test]
fn test_label_with_label_and_percentage() {
    let mut state = ProgressBarState::with_label("Loading...");
    state.set_progress(0.42);
    let label = build_label(&state);
    assert_eq!(label, "Loading... 42%");
}

#[test]
fn test_label_with_rate() {
    let mut state = ProgressBarState::with_progress(0.42);
    state.set_rate_text(Some("5.2 items/sec".to_string()));
    let label = build_label(&state);
    assert_eq!(label, "42% [5.2 items/sec]");
}

#[test]
fn test_label_with_eta() {
    let mut state = ProgressBarState::with_progress(0.42);
    state.set_eta(Some(Duration::from_secs(202)));
    let label = build_label(&state);
    assert_eq!(label, "42% ETA: 3m 22s");
}

#[test]
fn test_label_with_all_parts() {
    let mut state = ProgressBarState::with_label("Loading...");
    state.set_progress(0.42);
    state.set_rate_text(Some("5.2 items/sec".to_string()));
    state.set_eta(Some(Duration::from_secs(202)));
    let label = build_label(&state);
    assert_eq!(label, "Loading... 42% [5.2 items/sec] ETA: 3m 22s");
}

#[test]
fn test_label_hide_percentage() {
    let mut state = ProgressBarState::with_progress(0.42).with_show_percentage(false);
    state.set_rate_text(Some("5.2 items/sec".to_string()));
    let label = build_label(&state);
    assert_eq!(label, "[5.2 items/sec]");
}

#[test]
fn test_label_hide_eta() {
    let mut state = ProgressBarState::with_progress(0.42).with_show_eta(false);
    state.set_eta(Some(Duration::from_secs(202)));
    let label = build_label(&state);
    assert_eq!(label, "42%");
}

#[test]
fn test_label_hide_rate() {
    let mut state = ProgressBarState::with_progress(0.42).with_show_rate(false);
    state.set_rate_text(Some("5.2 items/sec".to_string()));
    let label = build_label(&state);
    assert_eq!(label, "42%");
}

#[test]
fn test_label_empty_when_all_hidden() {
    let state = ProgressBarState::with_progress(0.42).with_show_percentage(false);
    let label = build_label(&state);
    assert_eq!(label, "");
}

#[test]
fn test_view_with_eta_and_rate() {
    let mut state = ProgressBarState::with_label("Loading...");
    state.set_progress(0.42);
    state.set_rate_text(Some("5.2 items/sec".to_string()));
    state.set_eta(Some(Duration::from_secs(202)));
    let (mut terminal, theme) = crate::component::test_utils::setup_render(60, 5);

    terminal
        .draw(|frame| {
            ProgressBar::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

// Annotation tests

#[test]
fn test_annotation_emitted() {
    use crate::annotation::{with_annotations, WidgetType};
    let mut state = ProgressBarState::with_label("Downloading");
    state.set_progress(0.5);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 5);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                ProgressBar::view(&state, frame, frame.area(), &theme, &ViewContext::default());
            })
            .unwrap();
    });
    assert_eq!(registry.len(), 1);
    let regions = registry.find_by_type(&WidgetType::Progress);
    assert_eq!(regions.len(), 1);
    assert_eq!(regions[0].annotation.value, Some("50%".to_string()));
}
