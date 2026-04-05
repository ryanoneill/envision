use super::*;
use crate::component::test_utils;

fn sample_items() -> Vec<PaletteItem> {
    vec![
        PaletteItem::new("open", "Open File").with_shortcut("Ctrl+O"),
        PaletteItem::new("save", "Save File").with_shortcut("Ctrl+S"),
        PaletteItem::new("close", "Close Tab").with_shortcut("Ctrl+W"),
        PaletteItem::new("settings", "Open Settings"),
        PaletteItem::new("quit", "Quit Application").with_shortcut("Ctrl+Q"),
    ]
}

fn active_state() -> CommandPaletteState {
    let mut state = CommandPaletteState::new(sample_items());
    state.set_visible(true);
    state
}

#[test]
fn test_snapshot_visible_default() {
    let state = active_state();
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            CommandPalette::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_with_query() {
    let mut state = active_state();
    CommandPalette::update(&mut state, CommandPaletteMessage::TypeChar('o'));
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            CommandPalette::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_with_selection() {
    let mut state = active_state();
    CommandPalette::update(&mut state, CommandPaletteMessage::SelectNext);
    CommandPalette::update(&mut state, CommandPaletteMessage::SelectNext);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            CommandPalette::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_no_matches() {
    let mut state = active_state();
    CommandPalette::update(
        &mut state,
        CommandPaletteMessage::SetQuery("xyzxyz".to_string()),
    );
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            CommandPalette::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_hidden() {
    let state = CommandPaletteState::new(sample_items());
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            CommandPalette::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_custom_title_and_placeholder() {
    let mut state = CommandPaletteState::new(sample_items())
        .with_title("Actions")
        .with_placeholder("Search actions...");
    state.set_visible(true);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            CommandPalette::view(
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
fn test_snapshot_empty_items() {
    let mut state = CommandPaletteState::new(vec![]);
    state.set_visible(true);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            CommandPalette::view(
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
fn test_snapshot_with_descriptions() {
    let items = vec![
        PaletteItem::new("open", "Open File")
            .with_description("Open a file from disk")
            .with_shortcut("Ctrl+O"),
        PaletteItem::new("save", "Save File")
            .with_description("Save the current file")
            .with_shortcut("Ctrl+S"),
    ];
    let mut state = CommandPaletteState::new(items);
    state.set_visible(true);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            CommandPalette::view(
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
