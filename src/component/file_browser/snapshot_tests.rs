use super::*;
use crate::annotation::{WidgetType, with_annotations};
use crate::component::test_utils;

fn sample_entries() -> Vec<FileEntry> {
    vec![
        FileEntry::directory("src", "/src"),
        FileEntry::directory("tests", "/tests"),
        FileEntry::file("Cargo.toml", "/Cargo.toml").with_size(1024),
        FileEntry::file("README.md", "/README.md").with_size(2048),
        FileEntry::file("main.rs", "/main.rs").with_size(512),
    ]
}

fn focused_state() -> FileBrowserState {
    FileBrowserState::new("/", sample_entries())
}

// =============================================================================
// Rendering (snapshot)
// =============================================================================

#[test]
fn test_render_basic() {
    let state = focused_state();
    let (mut terminal, theme) = test_utils::setup_render(60, 12);
    terminal
        .draw(|frame| {
            FileBrowser::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_render_unfocused() {
    let state = FileBrowserState::new("/", sample_entries());
    let (mut terminal, theme) = test_utils::setup_render(60, 12);
    terminal
        .draw(|frame| {
            FileBrowser::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_render_with_filter() {
    let mut state = focused_state();
    FileBrowser::update(&mut state, FileBrowserMessage::FilterChar('m'));
    let (mut terminal, theme) = test_utils::setup_render(60, 12);
    terminal
        .draw(|frame| {
            FileBrowser::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_render_disabled() {
    let state = FileBrowserState::new("/", sample_entries());
    let (mut terminal, theme) = test_utils::setup_render(60, 12);
    terminal
        .draw(|frame| {
            FileBrowser::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme)
                    .focused(true)
                    .disabled(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_render_with_selection_markers() {
    let mut state = focused_state();
    // Toggle first item
    FileBrowser::update(&mut state, FileBrowserMessage::ToggleSelect);
    let (mut terminal, theme) = test_utils::setup_render(60, 12);
    terminal
        .draw(|frame| {
            FileBrowser::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_render_empty() {
    let state = FileBrowserState::new("/empty", vec![]);
    let (mut terminal, theme) = test_utils::setup_render(60, 8);
    terminal
        .draw(|frame| {
            FileBrowser::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

// =============================================================================
// Annotation
// =============================================================================

#[test]
fn test_annotation_emitted() {
    let state = FileBrowserState::new("/", sample_entries());
    let (mut terminal, theme) = test_utils::setup_render(60, 12);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                FileBrowser::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
            })
            .unwrap();
    });
    assert_eq!(registry.len(), 1);
    let regions = registry.find_by_type(&WidgetType::FileBrowser);
    assert_eq!(regions.len(), 1);
    assert!(regions[0].annotation.has_id("file_browser"));
}

#[test]
fn view_chrome_owned_no_outer_border() {
    let state = focused_state();
    let (mut terminal, theme) = test_utils::setup_render(40, 8);
    terminal
        .draw(|frame| {
            FileBrowser::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).chrome_owned(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}
