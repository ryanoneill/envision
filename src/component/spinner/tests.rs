use super::*;

#[test]
fn test_new() {
    let state = SpinnerState::new();
    assert!(state.is_spinning());
    assert_eq!(state.style(), &SpinnerStyle::Dots);
    assert_eq!(state.frame_index(), 0);
    assert!(state.label().is_none());
}

#[test]
fn test_default() {
    let state = SpinnerState::default();
    assert!(state.is_spinning());
    assert_eq!(state.style(), &SpinnerStyle::Dots);
}

#[test]
fn test_with_style() {
    let state = SpinnerState::with_style(SpinnerStyle::Line);
    assert_eq!(state.style(), &SpinnerStyle::Line);
    assert!(state.is_spinning());
}

#[test]
fn test_with_label() {
    let state = SpinnerState::with_label("Loading...");
    assert_eq!(state.label(), Some("Loading..."));
    assert!(state.is_spinning());
}

#[test]
fn test_is_spinning() {
    let state = SpinnerState::new();
    assert!(state.is_spinning());
}

#[test]
fn test_spinning_accessors() {
    let mut state = SpinnerState::new();
    assert!(state.is_spinning());

    state.set_spinning(false);
    assert!(!state.is_spinning());

    state.set_spinning(true);
    assert!(state.is_spinning());
}

#[test]
fn test_label_accessors() {
    let mut state = SpinnerState::new();
    assert!(state.label().is_none());

    state.set_label(Some("Test".to_string()));
    assert_eq!(state.label(), Some("Test"));

    state.set_label(None);
    assert!(state.label().is_none());
}

#[test]
fn test_style_accessors() {
    let mut state = SpinnerState::new();
    assert_eq!(state.style(), &SpinnerStyle::Dots);

    state.set_style(SpinnerStyle::Circle);
    assert_eq!(state.style(), &SpinnerStyle::Circle);
}

#[test]
fn test_current_frame() {
    let state = SpinnerState::new();
    // First frame of Dots is '⠋'
    assert_eq!(state.current_frame(), '⠋');
}

#[test]
fn test_tick_advances_frame() {
    let mut state = SpinnerState::new();
    assert_eq!(state.frame_index(), 0);
    assert_eq!(state.current_frame(), '⠋');

    Spinner::update(&mut state, SpinnerMessage::Tick);
    assert_eq!(state.frame_index(), 1);
    assert_eq!(state.current_frame(), '⠙');

    Spinner::update(&mut state, SpinnerMessage::Tick);
    assert_eq!(state.frame_index(), 2);
    assert_eq!(state.current_frame(), '⠹');
}

#[test]
fn test_tick_wraps_around() {
    let mut state = SpinnerState::with_style(SpinnerStyle::Line);
    // Line has 4 frames: |, /, -, \

    Spinner::update(&mut state, SpinnerMessage::Tick); // 1
    Spinner::update(&mut state, SpinnerMessage::Tick); // 2
    Spinner::update(&mut state, SpinnerMessage::Tick); // 3
    assert_eq!(state.frame_index(), 3);

    Spinner::update(&mut state, SpinnerMessage::Tick); // Wraps to 0
    assert_eq!(state.frame_index(), 0);
    assert_eq!(state.current_frame(), '|');
}

#[test]
fn test_tick_when_stopped() {
    let mut state = SpinnerState::new();
    state.set_spinning(false);

    let initial_frame = state.frame_index();
    Spinner::update(&mut state, SpinnerMessage::Tick);
    assert_eq!(state.frame_index(), initial_frame); // No change
}

#[test]
fn test_start_stop() {
    let mut state = SpinnerState::new();
    assert!(state.is_spinning());

    Spinner::update(&mut state, SpinnerMessage::Stop);
    assert!(!state.is_spinning());

    Spinner::update(&mut state, SpinnerMessage::Start);
    assert!(state.is_spinning());
}

#[test]
fn test_style_frames_dots() {
    let style = SpinnerStyle::Dots;
    assert_eq!(style.frame_count(), 10);
    assert_eq!(
        style.frames(),
        &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏']
    );
}

#[test]
fn test_style_frames_line() {
    let style = SpinnerStyle::Line;
    assert_eq!(style.frame_count(), 4);
    assert_eq!(style.frames(), &['|', '/', '-', '\\']);
}

#[test]
fn test_style_frames_circle() {
    let style = SpinnerStyle::Circle;
    assert_eq!(style.frame_count(), 4);
    assert_eq!(style.frames(), &['◐', '◓', '◑', '◒']);
}

#[test]
fn test_style_frames_bounce() {
    let style = SpinnerStyle::Bounce;
    assert_eq!(style.frame_count(), 4);
    assert_eq!(style.frames(), &['⠁', '⠂', '⠄', '⠂']);
}

#[test]
fn test_custom_style() {
    let custom = SpinnerStyle::Custom(vec!['◯', '◔', '◑', '◕', '●']);
    assert_eq!(custom.frame_count(), 5);
    assert_eq!(custom.frames(), &['◯', '◔', '◑', '◕', '●']);
}

#[test]
fn test_custom_style_empty() {
    let custom = SpinnerStyle::Custom(vec![]);
    assert_eq!(custom.frame_count(), 1);
    assert_eq!(custom.frames(), &[' ']);
}

#[test]
fn test_set_style_resets_frame() {
    let mut state = SpinnerState::new();

    // Advance a few frames
    Spinner::update(&mut state, SpinnerMessage::Tick);
    Spinner::update(&mut state, SpinnerMessage::Tick);
    assert_eq!(state.frame_index(), 2);

    // Change style
    state.set_style(SpinnerStyle::Line);
    assert_eq!(state.frame_index(), 0);
}

#[test]
fn test_clone() {
    let mut state = SpinnerState::with_label("Test");
    Spinner::update(&mut state, SpinnerMessage::Tick);

    let cloned = state.clone();
    assert_eq!(cloned.label(), Some("Test"));
    assert_eq!(cloned.frame_index(), 1);
    assert!(cloned.is_spinning());
}

#[test]
fn test_init() {
    let state = Spinner::init();
    assert!(state.is_spinning());
    assert_eq!(state.style(), &SpinnerStyle::Dots);
    assert!(state.label().is_none());
}

#[test]
fn test_update_returns_none() {
    let mut state = SpinnerState::new();

    assert_eq!(Spinner::update(&mut state, SpinnerMessage::Tick), None);
    assert_eq!(Spinner::update(&mut state, SpinnerMessage::Start), None);
    assert_eq!(Spinner::update(&mut state, SpinnerMessage::Stop), None);
}

#[test]
fn test_view_spinning() {
    let state = SpinnerState::new();

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 5);

    terminal
        .draw(|frame| {
            Spinner::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_stopped() {
    let mut state = SpinnerState::new();
    state.set_spinning(false);

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 5);

    terminal
        .draw(|frame| {
            Spinner::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_with_label() {
    let state = SpinnerState::with_label("Loading");

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 5);

    terminal
        .draw(|frame| {
            Spinner::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_style_default() {
    let style = SpinnerStyle::default();
    assert_eq!(style, SpinnerStyle::Dots);
}

#[test]
fn test_full_animation_cycle() {
    let mut state = SpinnerState::with_style(SpinnerStyle::Line);

    let mut frames_seen = Vec::new();
    for _ in 0..8 {
        // Two full cycles
        frames_seen.push(state.current_frame());
        Spinner::update(&mut state, SpinnerMessage::Tick);
    }

    // Should cycle through |, /, -, \ twice
    assert_eq!(frames_seen, vec!['|', '/', '-', '\\', '|', '/', '-', '\\']);
}
