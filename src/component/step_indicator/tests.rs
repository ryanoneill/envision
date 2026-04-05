use super::*;
use crate::component::test_utils::setup_render;
use crate::component::{Component, Focusable};
use crate::input::{Event, KeyCode};

// ========== Step Tests ==========

#[test]
fn test_step_new() {
    let step = Step::new("Build");
    assert_eq!(step.label(), "Build");
    assert_eq!(step.status(), &StepStatus::Pending);
    assert_eq!(step.description(), None);
}

#[test]
fn test_step_with_status() {
    let step = Step::new("Test").with_status(StepStatus::Active);
    assert_eq!(step.status(), &StepStatus::Active);
}

#[test]
fn test_step_with_description() {
    let step = Step::new("Deploy").with_description("Push to prod");
    assert_eq!(step.description(), Some("Push to prod"));
}

#[test]
fn test_step_builder_chain() {
    let step = Step::new("Build")
        .with_status(StepStatus::Completed)
        .with_description("Compile sources");
    assert_eq!(step.label(), "Build");
    assert_eq!(step.status(), &StepStatus::Completed);
    assert_eq!(step.description(), Some("Compile sources"));
}

// ========== StepStatus Icon Tests ==========

#[test]
fn test_status_icons() {
    assert_eq!(StepStatus::Pending.icon(), "○");
    assert_eq!(StepStatus::Active.icon(), "●");
    assert_eq!(StepStatus::Completed.icon(), "✓");
    assert_eq!(StepStatus::Failed.icon(), "✗");
    assert_eq!(StepStatus::Skipped.icon(), "⊘");
}

// ========== State Creation Tests ==========

#[test]
fn test_state_new() {
    let steps = vec![Step::new("A"), Step::new("B"), Step::new("C")];
    let state = StepIndicatorState::new(steps);
    assert_eq!(state.steps().len(), 3);
    assert_eq!(state.focused_index(), 0);
    assert!(!state.is_focused());
    assert!(!state.is_disabled());
    assert_eq!(state.orientation(), &StepOrientation::Horizontal);
    assert_eq!(state.connector(), "───");
    assert_eq!(state.title(), None);
    assert!(!state.show_descriptions());
}

#[test]
fn test_state_default() {
    let state = StepIndicatorState::default();
    assert!(state.steps().is_empty());
    assert_eq!(state.focused_index(), 0);
}

#[test]
fn test_state_with_orientation() {
    let state =
        StepIndicatorState::new(vec![Step::new("A")]).with_orientation(StepOrientation::Vertical);
    assert_eq!(state.orientation(), &StepOrientation::Vertical);
}

#[test]
fn test_state_with_title() {
    let state = StepIndicatorState::new(vec![Step::new("A")]).with_title("Pipeline");
    assert_eq!(state.title(), Some("Pipeline"));
}

#[test]
fn test_state_with_connector() {
    let state = StepIndicatorState::new(vec![Step::new("A")]).with_connector("→");
    assert_eq!(state.connector(), "→");
}

#[test]
fn test_state_with_show_descriptions() {
    let state = StepIndicatorState::new(vec![Step::new("A")]).with_show_descriptions(true);
    assert!(state.show_descriptions());
}

#[test]
fn test_state_with_disabled() {
    let state = StepIndicatorState::new(vec![Step::new("A")]).with_disabled(true);
    assert!(state.is_disabled());
}

// ========== Accessor Tests ==========

#[test]
fn test_step_accessor() {
    let state = StepIndicatorState::new(vec![Step::new("A"), Step::new("B")]);
    assert_eq!(state.step(0).unwrap().label(), "A");
    assert_eq!(state.step(1).unwrap().label(), "B");
    assert!(state.step(2).is_none());
}

#[test]
fn test_active_step_index_none() {
    let state = StepIndicatorState::new(vec![Step::new("A"), Step::new("B")]);
    assert_eq!(state.active_step_index(), None);
}

#[test]
fn test_active_step_index_some() {
    let steps = vec![
        Step::new("A").with_status(StepStatus::Completed),
        Step::new("B").with_status(StepStatus::Active),
        Step::new("C"),
    ];
    let state = StepIndicatorState::new(steps);
    assert_eq!(state.active_step_index(), Some(1));
}

#[test]
fn test_is_all_completed_false() {
    let steps = vec![
        Step::new("A").with_status(StepStatus::Completed),
        Step::new("B").with_status(StepStatus::Active),
    ];
    let state = StepIndicatorState::new(steps);
    assert!(!state.is_all_completed());
}

#[test]
fn test_is_all_completed_true() {
    let steps = vec![
        Step::new("A").with_status(StepStatus::Completed),
        Step::new("B").with_status(StepStatus::Completed),
    ];
    let state = StepIndicatorState::new(steps);
    assert!(state.is_all_completed());
}

#[test]
fn test_is_all_completed_with_skipped() {
    let steps = vec![
        Step::new("A").with_status(StepStatus::Completed),
        Step::new("B").with_status(StepStatus::Skipped),
    ];
    let state = StepIndicatorState::new(steps);
    assert!(state.is_all_completed());
}

#[test]
fn test_is_all_completed_empty() {
    let state = StepIndicatorState::new(vec![]);
    assert!(!state.is_all_completed());
}

// ========== SetStatus Tests ==========

#[test]
fn test_set_status() {
    let mut state = StepIndicatorState::new(vec![Step::new("A"), Step::new("B")]);
    let output = StepIndicator::update(
        &mut state,
        StepIndicatorMessage::SetStatus {
            index: 0,
            status: StepStatus::Active,
        },
    );
    assert_eq!(state.steps()[0].status(), &StepStatus::Active);
    assert_eq!(
        output,
        Some(StepIndicatorOutput::StatusChanged {
            index: 0,
            status: StepStatus::Active,
        })
    );
}

#[test]
fn test_set_status_out_of_bounds() {
    let mut state = StepIndicatorState::new(vec![Step::new("A")]);
    let output = StepIndicator::update(
        &mut state,
        StepIndicatorMessage::SetStatus {
            index: 5,
            status: StepStatus::Active,
        },
    );
    assert_eq!(output, None);
}

#[test]
fn test_set_status_triggers_all_completed() {
    let steps = vec![
        Step::new("A").with_status(StepStatus::Completed),
        Step::new("B").with_status(StepStatus::Active),
    ];
    let mut state = StepIndicatorState::new(steps);
    let output = StepIndicator::update(
        &mut state,
        StepIndicatorMessage::SetStatus {
            index: 1,
            status: StepStatus::Completed,
        },
    );
    assert_eq!(output, Some(StepIndicatorOutput::AllCompleted));
}

// ========== ActivateNext Tests ==========

#[test]
fn test_activate_next() {
    let steps = vec![
        Step::new("A").with_status(StepStatus::Completed),
        Step::new("B"),
        Step::new("C"),
    ];
    let mut state = StepIndicatorState::new(steps);
    let output = StepIndicator::update(&mut state, StepIndicatorMessage::ActivateNext);
    assert_eq!(state.steps()[1].status(), &StepStatus::Active);
    assert_eq!(
        output,
        Some(StepIndicatorOutput::StatusChanged {
            index: 1,
            status: StepStatus::Active,
        })
    );
}

#[test]
fn test_activate_next_no_pending() {
    let steps = vec![
        Step::new("A").with_status(StepStatus::Completed),
        Step::new("B").with_status(StepStatus::Completed),
    ];
    let mut state = StepIndicatorState::new(steps);
    let output = StepIndicator::update(&mut state, StepIndicatorMessage::ActivateNext);
    assert_eq!(output, None);
}

// ========== CompleteActive Tests ==========

#[test]
fn test_complete_active() {
    let steps = vec![
        Step::new("A").with_status(StepStatus::Active),
        Step::new("B"),
    ];
    let mut state = StepIndicatorState::new(steps);
    let output = StepIndicator::update(&mut state, StepIndicatorMessage::CompleteActive);
    assert_eq!(state.steps()[0].status(), &StepStatus::Completed);
    assert_eq!(
        output,
        Some(StepIndicatorOutput::StatusChanged {
            index: 0,
            status: StepStatus::Completed,
        })
    );
}

#[test]
fn test_complete_active_no_active() {
    let mut state = StepIndicatorState::new(vec![Step::new("A")]);
    let output = StepIndicator::update(&mut state, StepIndicatorMessage::CompleteActive);
    assert_eq!(output, None);
}

#[test]
fn test_complete_active_triggers_all_completed() {
    let steps = vec![
        Step::new("A").with_status(StepStatus::Completed),
        Step::new("B").with_status(StepStatus::Active),
    ];
    let mut state = StepIndicatorState::new(steps);
    let output = StepIndicator::update(&mut state, StepIndicatorMessage::CompleteActive);
    assert_eq!(output, Some(StepIndicatorOutput::AllCompleted));
}

// ========== FailActive Tests ==========

#[test]
fn test_fail_active() {
    let steps = vec![Step::new("A").with_status(StepStatus::Active)];
    let mut state = StepIndicatorState::new(steps);
    let output = StepIndicator::update(&mut state, StepIndicatorMessage::FailActive);
    assert_eq!(state.steps()[0].status(), &StepStatus::Failed);
    assert_eq!(
        output,
        Some(StepIndicatorOutput::StatusChanged {
            index: 0,
            status: StepStatus::Failed,
        })
    );
}

#[test]
fn test_fail_active_no_active() {
    let mut state = StepIndicatorState::new(vec![Step::new("A")]);
    let output = StepIndicator::update(&mut state, StepIndicatorMessage::FailActive);
    assert_eq!(output, None);
}

// ========== Skip Tests ==========

#[test]
fn test_skip() {
    let mut state = StepIndicatorState::new(vec![Step::new("A"), Step::new("B")]);
    let output = StepIndicator::update(&mut state, StepIndicatorMessage::Skip(0));
    assert_eq!(state.steps()[0].status(), &StepStatus::Skipped);
    assert_eq!(
        output,
        Some(StepIndicatorOutput::StatusChanged {
            index: 0,
            status: StepStatus::Skipped,
        })
    );
}

#[test]
fn test_skip_out_of_bounds() {
    let mut state = StepIndicatorState::new(vec![Step::new("A")]);
    let output = StepIndicator::update(&mut state, StepIndicatorMessage::Skip(5));
    assert_eq!(output, None);
}

#[test]
fn test_skip_triggers_all_completed() {
    let steps = vec![
        Step::new("A").with_status(StepStatus::Completed),
        Step::new("B"),
    ];
    let mut state = StepIndicatorState::new(steps);
    let output = StepIndicator::update(&mut state, StepIndicatorMessage::Skip(1));
    assert_eq!(output, Some(StepIndicatorOutput::AllCompleted));
}

// ========== Reset Tests ==========

#[test]
fn test_reset() {
    let steps = vec![
        Step::new("A").with_status(StepStatus::Completed),
        Step::new("B").with_status(StepStatus::Active),
    ];
    let mut state = StepIndicatorState::new(steps);
    state.focused_index = 1;
    let output = StepIndicator::update(&mut state, StepIndicatorMessage::Reset);
    assert_eq!(output, Some(StepIndicatorOutput::Reset));
    assert_eq!(state.steps()[0].status(), &StepStatus::Pending);
    assert_eq!(state.steps()[1].status(), &StepStatus::Pending);
    assert_eq!(state.focused_index(), 0);
}

// ========== Complete Workflow Chain ==========

#[test]
fn test_workflow_chain() {
    let steps = vec![Step::new("Build"), Step::new("Test"), Step::new("Deploy")];
    let mut state = StepIndicatorState::new(steps);

    // Activate first step
    StepIndicator::update(&mut state, StepIndicatorMessage::ActivateNext);
    assert_eq!(state.steps()[0].status(), &StepStatus::Active);

    // Complete it and activate next
    StepIndicator::update(&mut state, StepIndicatorMessage::CompleteActive);
    assert_eq!(state.steps()[0].status(), &StepStatus::Completed);

    StepIndicator::update(&mut state, StepIndicatorMessage::ActivateNext);
    assert_eq!(state.steps()[1].status(), &StepStatus::Active);

    // Complete it and activate last
    StepIndicator::update(&mut state, StepIndicatorMessage::CompleteActive);
    StepIndicator::update(&mut state, StepIndicatorMessage::ActivateNext);
    assert_eq!(state.steps()[2].status(), &StepStatus::Active);

    // Complete last - should trigger AllCompleted
    let output = StepIndicator::update(&mut state, StepIndicatorMessage::CompleteActive);
    assert_eq!(output, Some(StepIndicatorOutput::AllCompleted));
    assert!(state.is_all_completed());
}

// ========== Focus Navigation Tests ==========

#[test]
fn test_focus_next() {
    let mut state = StepIndicatorState::new(vec![Step::new("A"), Step::new("B"), Step::new("C")]);
    state.set_focused(true);

    let output = StepIndicator::update(&mut state, StepIndicatorMessage::FocusNext);
    assert_eq!(state.focused_index(), 1);
    assert_eq!(output, Some(StepIndicatorOutput::FocusChanged(1)));
}

#[test]
fn test_focus_next_wraps() {
    let mut state = StepIndicatorState::new(vec![Step::new("A"), Step::new("B")]);
    state.set_focused(true);
    state.focused_index = 1;

    let output = StepIndicator::update(&mut state, StepIndicatorMessage::FocusNext);
    assert_eq!(state.focused_index(), 0);
    assert_eq!(output, Some(StepIndicatorOutput::FocusChanged(0)));
}

#[test]
fn test_focus_prev() {
    let mut state = StepIndicatorState::new(vec![Step::new("A"), Step::new("B"), Step::new("C")]);
    state.set_focused(true);
    state.focused_index = 2;

    let output = StepIndicator::update(&mut state, StepIndicatorMessage::FocusPrev);
    assert_eq!(state.focused_index(), 1);
    assert_eq!(output, Some(StepIndicatorOutput::FocusChanged(1)));
}

#[test]
fn test_focus_prev_wraps() {
    let mut state = StepIndicatorState::new(vec![Step::new("A"), Step::new("B")]);
    state.set_focused(true);

    let output = StepIndicator::update(&mut state, StepIndicatorMessage::FocusPrev);
    assert_eq!(state.focused_index(), 1);
    assert_eq!(output, Some(StepIndicatorOutput::FocusChanged(1)));
}

#[test]
fn test_focus_first() {
    let mut state = StepIndicatorState::new(vec![Step::new("A"), Step::new("B")]);
    state.set_focused(true);
    state.focused_index = 1;

    let output = StepIndicator::update(&mut state, StepIndicatorMessage::First);
    assert_eq!(state.focused_index(), 0);
    assert_eq!(output, Some(StepIndicatorOutput::FocusChanged(0)));
}

#[test]
fn test_focus_last() {
    let mut state = StepIndicatorState::new(vec![Step::new("A"), Step::new("B"), Step::new("C")]);
    state.set_focused(true);

    let output = StepIndicator::update(&mut state, StepIndicatorMessage::Last);
    assert_eq!(state.focused_index(), 2);
    assert_eq!(output, Some(StepIndicatorOutput::FocusChanged(2)));
}

#[test]
fn test_select() {
    let mut state = StepIndicatorState::new(vec![Step::new("A"), Step::new("B")]);
    state.set_focused(true);
    state.focused_index = 1;

    let output = StepIndicator::update(&mut state, StepIndicatorMessage::Select);
    assert_eq!(output, Some(StepIndicatorOutput::Selected(1)));
}

// ========== Guard Tests ==========

#[test]
fn test_focus_next_unfocused_guard() {
    let mut state = StepIndicatorState::new(vec![Step::new("A"), Step::new("B")]);
    let output = StepIndicator::update(&mut state, StepIndicatorMessage::FocusNext);
    assert_eq!(output, None);
    assert_eq!(state.focused_index(), 0);
}

#[test]
fn test_focus_next_disabled_guard() {
    let mut state =
        StepIndicatorState::new(vec![Step::new("A"), Step::new("B")]).with_disabled(true);
    state.set_focused(true);
    let output = StepIndicator::update(&mut state, StepIndicatorMessage::FocusNext);
    assert_eq!(output, None);
}

#[test]
fn test_focus_next_empty_guard() {
    let mut state = StepIndicatorState::new(vec![]);
    state.set_focused(true);
    let output = StepIndicator::update(&mut state, StepIndicatorMessage::FocusNext);
    assert_eq!(output, None);
}

#[test]
fn test_select_unfocused_guard() {
    let mut state = StepIndicatorState::new(vec![Step::new("A")]);
    let output = StepIndicator::update(&mut state, StepIndicatorMessage::Select);
    assert_eq!(output, None);
}

#[test]
fn test_select_disabled_guard() {
    let mut state = StepIndicatorState::new(vec![Step::new("A")]).with_disabled(true);
    state.set_focused(true);
    let output = StepIndicator::update(&mut state, StepIndicatorMessage::Select);
    assert_eq!(output, None);
}

// ========== handle_event Tests ==========

#[test]
fn test_handle_event_right_arrow() {
    let mut state = StepIndicatorState::new(vec![Step::new("A"), Step::new("B")]);
    state.set_focused(true);
    let msg = StepIndicator::handle_event(
        &state,
        &Event::key(KeyCode::Right),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(StepIndicatorMessage::FocusNext));
}

#[test]
fn test_handle_event_l_key() {
    let mut state = StepIndicatorState::new(vec![Step::new("A"), Step::new("B")]);
    state.set_focused(true);
    let msg =
        StepIndicator::handle_event(&state, &Event::char('l'), &ViewContext::new().focused(true));
    assert_eq!(msg, Some(StepIndicatorMessage::FocusNext));
}

#[test]
fn test_handle_event_left_arrow() {
    let mut state = StepIndicatorState::new(vec![Step::new("A"), Step::new("B")]);
    state.set_focused(true);
    let msg = StepIndicator::handle_event(
        &state,
        &Event::key(KeyCode::Left),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(StepIndicatorMessage::FocusPrev));
}

#[test]
fn test_handle_event_h_key() {
    let mut state = StepIndicatorState::new(vec![Step::new("A"), Step::new("B")]);
    state.set_focused(true);
    let msg =
        StepIndicator::handle_event(&state, &Event::char('h'), &ViewContext::new().focused(true));
    assert_eq!(msg, Some(StepIndicatorMessage::FocusPrev));
}

#[test]
fn test_handle_event_home() {
    let mut state = StepIndicatorState::new(vec![Step::new("A"), Step::new("B")]);
    state.set_focused(true);
    let msg = StepIndicator::handle_event(
        &state,
        &Event::key(KeyCode::Home),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(StepIndicatorMessage::First));
}

#[test]
fn test_handle_event_end() {
    let mut state = StepIndicatorState::new(vec![Step::new("A"), Step::new("B")]);
    state.set_focused(true);
    let msg = StepIndicator::handle_event(
        &state,
        &Event::key(KeyCode::End),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(StepIndicatorMessage::Last));
}

#[test]
fn test_handle_event_enter() {
    let mut state = StepIndicatorState::new(vec![Step::new("A"), Step::new("B")]);
    state.set_focused(true);
    let msg = StepIndicator::handle_event(
        &state,
        &Event::key(KeyCode::Enter),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(StepIndicatorMessage::Select));
}

#[test]
fn test_handle_event_unfocused_ignored() {
    let state = StepIndicatorState::new(vec![Step::new("A")]);
    let msg =
        StepIndicator::handle_event(&state, &Event::key(KeyCode::Right), &ViewContext::default());
    assert_eq!(msg, None);
}

#[test]
fn test_handle_event_disabled_ignored() {
    let mut state = StepIndicatorState::new(vec![Step::new("A")]).with_disabled(true);
    state.set_focused(true);
    let msg = StepIndicator::handle_event(
        &state,
        &Event::key(KeyCode::Right),
        &ViewContext::new().focused(true).disabled(true),
    );
    assert_eq!(msg, None);
}

#[test]
fn test_handle_event_unrecognized_key() {
    let mut state = StepIndicatorState::new(vec![Step::new("A")]);
    state.set_focused(true);
    let msg =
        StepIndicator::handle_event(&state, &Event::char('z'), &ViewContext::new().focused(true));
    assert_eq!(msg, None);
}

// ========== dispatch_event Tests ==========

#[test]
fn test_dispatch_event() {
    let mut state = StepIndicatorState::new(vec![Step::new("A"), Step::new("B")]);
    state.set_focused(true);
    let output = state.dispatch_event(&Event::key(KeyCode::Right));
    assert_eq!(output, Some(StepIndicatorOutput::FocusChanged(1)));
    assert_eq!(state.focused_index(), 1);
}

#[test]
fn test_instance_update() {
    let mut state = StepIndicatorState::new(vec![Step::new("A"), Step::new("B")]);
    state.set_focused(true);
    let output = state.update(StepIndicatorMessage::FocusNext);
    assert_eq!(output, Some(StepIndicatorOutput::FocusChanged(1)));
}

// ========== Focusable Trait Tests ==========

#[test]
fn test_focusable_is_focused() {
    let state = StepIndicatorState::new(vec![Step::new("A")]);
    assert!(!StepIndicator::is_focused(&state));
}

#[test]
fn test_focusable_set_focused() {
    let mut state = StepIndicatorState::new(vec![Step::new("A")]);
    StepIndicator::set_focused(&mut state, true);
    assert!(StepIndicator::is_focused(&state));
}

#[test]
fn test_focusable_focus_blur() {
    let mut state = StepIndicatorState::new(vec![Step::new("A")]);
    StepIndicator::focus(&mut state);
    assert!(state.is_focused());
    StepIndicator::blur(&mut state);
    assert!(!state.is_focused());
}

// ========== Rendering Snapshot Tests ==========

#[test]
fn test_view_horizontal() {
    let (mut terminal, theme) = setup_render(60, 5);
    let steps = vec![
        Step::new("Build").with_status(StepStatus::Completed),
        Step::new("Test").with_status(StepStatus::Active),
        Step::new("Deploy"),
    ];
    let state = StepIndicatorState::new(steps);

    terminal
        .draw(|frame| {
            StepIndicator::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    let display = terminal.backend().to_string();
    insta::assert_snapshot!("view_horizontal", display);
}

#[test]
fn test_view_vertical() {
    let (mut terminal, theme) = setup_render(30, 10);
    let steps = vec![
        Step::new("Build").with_status(StepStatus::Completed),
        Step::new("Test").with_status(StepStatus::Active),
        Step::new("Deploy"),
    ];
    let state = StepIndicatorState::new(steps).with_orientation(StepOrientation::Vertical);

    terminal
        .draw(|frame| {
            StepIndicator::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    let display = terminal.backend().to_string();
    insta::assert_snapshot!("view_vertical", display);
}

#[test]
fn test_view_with_title() {
    let (mut terminal, theme) = setup_render(60, 5);
    let steps = vec![
        Step::new("Build").with_status(StepStatus::Completed),
        Step::new("Test").with_status(StepStatus::Active),
    ];
    let state = StepIndicatorState::new(steps).with_title("Pipeline");

    terminal
        .draw(|frame| {
            StepIndicator::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    let display = terminal.backend().to_string();
    insta::assert_snapshot!("view_with_title", display);
}

#[test]
fn test_view_focused_step() {
    let (mut terminal, theme) = setup_render(60, 5);
    let steps = vec![Step::new("A"), Step::new("B"), Step::new("C")];
    let mut state = StepIndicatorState::new(steps);
    state.set_focused(true);
    state.focused_index = 1;

    terminal
        .draw(|frame| {
            StepIndicator::view(
                &state,
                frame,
                frame.area(),
                &theme,
                &ViewContext::new().focused(true),
            );
        })
        .unwrap();

    let display = terminal.backend().to_string();
    insta::assert_snapshot!("view_focused_step", display);
}

#[test]
fn test_view_vertical_with_descriptions() {
    let (mut terminal, theme) = setup_render(40, 12);
    let steps = vec![
        Step::new("Build")
            .with_status(StepStatus::Completed)
            .with_description("Compile sources"),
        Step::new("Test")
            .with_status(StepStatus::Active)
            .with_description("Run unit tests"),
        Step::new("Deploy").with_description("Push to production"),
    ];
    let state = StepIndicatorState::new(steps)
        .with_orientation(StepOrientation::Vertical)
        .with_show_descriptions(true);

    terminal
        .draw(|frame| {
            StepIndicator::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    let display = terminal.backend().to_string();
    insta::assert_snapshot!("view_vertical_descriptions", display);
}

#[test]
fn test_view_all_statuses() {
    let (mut terminal, theme) = setup_render(80, 5);
    let steps = vec![
        Step::new("Done").with_status(StepStatus::Completed),
        Step::new("Active").with_status(StepStatus::Active),
        Step::new("Failed").with_status(StepStatus::Failed),
        Step::new("Skipped").with_status(StepStatus::Skipped),
        Step::new("Pending"),
    ];
    let state = StepIndicatorState::new(steps);

    terminal
        .draw(|frame| {
            StepIndicator::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    let display = terminal.backend().to_string();
    insta::assert_snapshot!("view_all_statuses", display);
}

#[test]
fn test_view_empty_steps() {
    let (mut terminal, theme) = setup_render(40, 5);
    let state = StepIndicatorState::new(vec![]);

    terminal
        .draw(|frame| {
            StepIndicator::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    let display = terminal.backend().to_string();
    insta::assert_snapshot!("view_empty", display);
}

// ========== Annotation Tests ==========

#[test]
fn test_annotation_emission() {
    use crate::annotation::{with_annotations, WidgetType};
    let steps = vec![Step::new("A"), Step::new("B")];
    let mut state = StepIndicatorState::new(steps);
    state.set_focused(true);
    let (mut terminal, theme) = setup_render(60, 5);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                StepIndicator::view(
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
    let regions = registry.find_by_type(&WidgetType::StepIndicator);
    assert_eq!(regions.len(), 1);
    assert!(regions[0].annotation.has_id("step_indicator"));
    assert!(regions[0].annotation.focused);
    assert!(!regions[0].annotation.disabled);
}

// ========== Init Tests ==========

#[test]
fn test_init() {
    let state = StepIndicator::init();
    assert!(state.steps().is_empty());
    assert!(!state.is_focused());
}
