use super::*;
use crate::component::test_utils;

// =============================================================================
// Snapshot tests
// =============================================================================

#[test]
fn test_snapshot_empty() {
    let state = StatusBarState::new();
    let (mut terminal, theme) = test_utils::setup_render(60, 1);
    terminal
        .draw(|frame| {
            StatusBar::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_left_only() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::new("INSERT"));
    state.push_left(StatusBarItem::new("main.rs"));
    let (mut terminal, theme) = test_utils::setup_render(60, 1);
    terminal
        .draw(|frame| {
            StatusBar::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_center_only() {
    let mut state = StatusBarState::new();
    state.push_center(StatusBarItem::new("filename.rs"));
    let (mut terminal, theme) = test_utils::setup_render(60, 1);
    terminal
        .draw(|frame| {
            StatusBar::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_right_only() {
    let mut state = StatusBarState::new();
    state.push_right(StatusBarItem::new("UTF-8"));
    state.push_right(StatusBarItem::new("Ln 42, Col 8"));
    let (mut terminal, theme) = test_utils::setup_render(60, 1);
    terminal
        .draw(|frame| {
            StatusBar::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_all_sections() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::new("NORMAL").with_style(StatusBarStyle::Info));
    state.push_center(StatusBarItem::new("main.rs"));
    state.push_right(StatusBarItem::new("UTF-8"));
    state.push_right(StatusBarItem::new("Ln 42, Col 8"));
    let (mut terminal, theme) = test_utils::setup_render(60, 1);
    terminal
        .draw(|frame| {
            StatusBar::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_styled_items() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::new("ERROR").with_style(StatusBarStyle::Error));
    state.push_left(StatusBarItem::new("WARN").with_style(StatusBarStyle::Warning));
    state.push_right(StatusBarItem::new("OK").with_style(StatusBarStyle::Success));
    let (mut terminal, theme) = test_utils::setup_render(60, 1);
    terminal
        .draw(|frame| {
            StatusBar::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_counter_item() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::new("Mode"));
    state.push_right(StatusBarItem::counter().with_label("Items"));
    StatusBar::update(
        &mut state,
        StatusBarMessage::SetCounter {
            section: Section::Right,
            index: 0,
            value: 42,
        },
    );
    let (mut terminal, theme) = test_utils::setup_render(60, 1);
    terminal
        .draw(|frame| {
            StatusBar::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_custom_separator() {
    let mut state = StatusBarState::with_separator(" :: ");
    state.push_left(StatusBarItem::new("A"));
    state.push_left(StatusBarItem::new("B"));
    state.push_right(StatusBarItem::new("X"));
    state.push_right(StatusBarItem::new("Y"));
    let (mut terminal, theme) = test_utils::setup_render(60, 1);
    terminal
        .draw(|frame| {
            StatusBar::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn snapshot_status_bar_four_stop_severity_ramp() {
    use crate::theme::{Severity, Theme};

    // The Q-gamma payoff: four distinct ANSI fg colors when each Severity
    // band gets its own theme.severity_color. Pre-G5, the consumer-side
    // severity_status_style helper collapsed Bad+Mild -> Warning because
    // StatusBarStyle had no Peach variant. Post-G5, with_color + the
    // theme palette restores the full four-stop ramp.
    let theme = Theme::catppuccin_mocha();
    let items = vec![
        StatusBarItem::new("good").with_color(theme.severity_color(Severity::Good)),
        StatusBarItem::new("mild").with_color(theme.severity_color(Severity::Mild)),
        StatusBarItem::new("bad").with_color(theme.severity_color(Severity::Bad)),
        StatusBarItem::new("crit").with_color(theme.severity_color(Severity::Critical)),
    ];
    let mut state = StatusBarState::new();
    for item in items {
        state.push_left(item);
    }

    // Render with the actual Catppuccin theme so the four colors are
    // distinct RGB values, not basic-Color collapses.
    let (mut terminal, _) = test_utils::setup_render(40, 1);
    terminal
        .draw(|frame| {
            StatusBar::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();

    let plain = terminal.backend().to_string();
    let ansi = terminal.backend().to_ansi();

    // ratatui emits RGB foregrounds as \x1b[38;2;R;G;Bm. Each of the four
    // bands should produce a distinct escape. Catppuccin: Good=Green (166,
    // 227, 161), Mild=Yellow (249, 226, 175), Bad=Peach (250, 179, 135),
    // Critical=Red (243, 139, 168).
    assert!(
        ansi.contains("\x1b[38;2;166;227;161m"),
        "expected Catppuccin Green for Severity::Good, got:\n{ansi}",
    );
    assert!(
        ansi.contains("\x1b[38;2;249;226;175m"),
        "expected Catppuccin Yellow for Severity::Mild, got:\n{ansi}",
    );
    assert!(
        ansi.contains("\x1b[38;2;250;179;135m"),
        "expected Catppuccin Peach for Severity::Bad (the restored band), got:\n{ansi}",
    );
    assert!(
        ansi.contains("\x1b[38;2;243;139;168m"),
        "expected Catppuccin Red for Severity::Critical, got:\n{ansi}",
    );

    insta::assert_snapshot!(plain);
}
