use std::sync::Arc;

use super::*;
use crate::input::{Event, KeyCode};

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
// Refresh
// =============================================================================

#[test]
fn test_refresh_without_provider() {
    let mut state = focused_state();
    let output = FileBrowser::update(&mut state, FileBrowserMessage::Refresh);
    // Without provider, entries are cleared
    assert!(output.is_none());
}

// =============================================================================
// format_size helper
// =============================================================================

#[test]
fn test_format_size_bytes() {
    assert_eq!(format_size(100), "100B");
    assert_eq!(format_size(0), "0B");
}

#[test]
fn test_format_size_kilobytes() {
    assert_eq!(format_size(1024), "1.0K");
    assert_eq!(format_size(2048), "2.0K");
}

#[test]
fn test_format_size_megabytes() {
    assert_eq!(format_size(1024 * 1024), "1.0M");
}

#[test]
fn test_format_size_gigabytes() {
    assert_eq!(format_size(1024 * 1024 * 1024), "1.0G");
}

// =============================================================================
// compute_segments helper
// =============================================================================

#[test]
fn test_compute_segments_root() {
    let segments = compute_segments("/");
    assert_eq!(segments, vec!["/"]);
}

#[test]
fn test_compute_segments_nested() {
    let segments = compute_segments("/home/user/docs");
    assert_eq!(segments, vec!["/", "home", "user", "docs"]);
}

#[test]
fn test_compute_segments_empty() {
    let segments = compute_segments("");
    assert_eq!(segments, vec!["/"]);
}

// =============================================================================
// Debug impl
// =============================================================================

#[test]
fn test_debug_impl() {
    let state = FileBrowserState::new("/", sample_entries());
    let debug = format!("{:?}", state);
    assert!(debug.contains("FileBrowserState"));
    assert!(debug.contains("current_path"));
    assert!(debug.contains("path_segments"));
    assert!(debug.contains("entries"));
    assert!(debug.contains("filtered_indices"));
    assert!(debug.contains("selected_index"));
    assert!(debug.contains("selected_paths"));
    assert!(debug.contains("filter_text"));
    assert!(debug.contains("internal_focus"));
    assert!(debug.contains("selection_mode"));
    assert!(debug.contains("sort_field"));
    assert!(debug.contains("sort_direction"));
    assert!(debug.contains("directories_first"));
    assert!(debug.contains("show_hidden"));
    assert!(debug.contains("list_state"));
    assert!(debug.contains("provider"));
}

#[test]
fn test_debug_impl_with_provider() {
    struct TestProvider;
    impl DirectoryProvider for TestProvider {
        fn list_entries(&self, _path: &str) -> Vec<FileEntry> {
            vec![]
        }
        fn parent_path(&self, _path: &str) -> Option<String> {
            None
        }
    }

    let provider = Arc::new(TestProvider);
    let state = FileBrowserState::with_provider("/", provider);
    let debug = format!("{:?}", state);
    assert!(debug.contains("<DirectoryProvider>"));
}

#[test]
fn test_debug_impl_without_provider() {
    let state = FileBrowserState::new("/", vec![]);
    let debug = format!("{:?}", state);
    assert!(debug.contains("provider: None"));
}

// =============================================================================
// PathBar focus keyboard
// =============================================================================

#[test]
fn test_pathbar_focus_only_handles_tab() {
    let mut state = focused_state();
    // Cycle to Filter, then to PathBar
    FileBrowser::update(&mut state, FileBrowserMessage::CycleFocus);
    FileBrowser::update(&mut state, FileBrowserMessage::CycleFocus);
    // In PathBar focus, regular keys shouldn't map
    assert!(
        FileBrowser::handle_event(
            &state,
            &Event::char('j'),
            &EventContext::new().focused(true)
        )
        .is_none()
    );
    assert!(
        FileBrowser::handle_event(
            &state,
            &Event::key(KeyCode::Enter),
            &EventContext::new().focused(true)
        )
        .is_none()
    );
    // Tab still works
    let msg = FileBrowser::handle_event(
        &state,
        &Event::key(KeyCode::Tab),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(FileBrowserMessage::CycleFocus));
}

// =============================================================================
// Filter focus keyboard
// =============================================================================

#[test]
fn test_filter_focus_handles_chars() {
    let mut state = focused_state();
    // Cycle to Filter
    FileBrowser::update(&mut state, FileBrowserMessage::CycleFocus);
    // In Filter focus, chars should map to FilterChar
    let msg = FileBrowser::handle_event(
        &state,
        &Event::char('z'),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(FileBrowserMessage::FilterChar('z')));
}

#[test]
fn test_filter_focus_backspace() {
    let mut state = focused_state();
    FileBrowser::update(&mut state, FileBrowserMessage::CycleFocus);
    let msg = FileBrowser::handle_event(
        &state,
        &Event::key(KeyCode::Backspace),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(FileBrowserMessage::FilterBackspace));
}

#[test]
fn test_filter_focus_esc() {
    let mut state = focused_state();
    FileBrowser::update(&mut state, FileBrowserMessage::CycleFocus);
    let msg = FileBrowser::handle_event(
        &state,
        &Event::key(KeyCode::Esc),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(FileBrowserMessage::FilterClear));
}

// =============================================================================
// DirectoryProvider trait
// =============================================================================

#[test]
fn test_with_provider() {
    struct TestProvider;
    impl DirectoryProvider for TestProvider {
        fn list_entries(&self, path: &str) -> Vec<FileEntry> {
            match path {
                "/" => vec![
                    FileEntry::directory("home", "/home"),
                    FileEntry::file("readme.txt", "/readme.txt"),
                ],
                "/home" => vec![FileEntry::directory("user", "/home/user")],
                _ => vec![],
            }
        }

        fn parent_path(&self, path: &str) -> Option<String> {
            match path {
                "/" => None,
                "/home" => Some("/".to_string()),
                "/home/user" => Some("/home".to_string()),
                _ => None,
            }
        }
    }

    let provider = Arc::new(TestProvider);
    let mut state = FileBrowserState::with_provider("/", provider);

    assert_eq!(state.entries().len(), 2);
    assert_eq!(state.current_path(), "/");

    // Enter /home directory
    let output = FileBrowser::update(&mut state, FileBrowserMessage::Enter);
    assert!(matches!(output, Some(FileBrowserOutput::DirectoryEntered(ref p)) if p == "/home"));
    assert_eq!(state.current_path(), "/home");
    assert_eq!(state.entries().len(), 1);

    // Go back
    let output = FileBrowser::update(&mut state, FileBrowserMessage::Back);
    assert!(matches!(output, Some(FileBrowserOutput::NavigatedBack(ref p)) if p == "/"));
    assert_eq!(state.current_path(), "/");
    assert_eq!(state.entries().len(), 2);
}

#[test]
fn test_selected_alias() {
    let state = FileBrowserState::new("/", sample_entries());
    assert_eq!(state.selected(), state.selected_index());
    assert_eq!(state.selected(), Some(0));
}

#[test]
fn test_selected_alias_none_when_empty() {
    let state = FileBrowserState::new("/empty", vec![]);
    assert_eq!(state.selected(), None);
}

#[test]
fn test_selected_item_alias() {
    let state = FileBrowserState::new("/", sample_entries());
    assert_eq!(state.selected_item(), state.selected_entry());
    let item = state.selected_item().unwrap();
    assert!(item.is_dir());
}

#[test]
fn test_selected_item_none_when_empty() {
    let state = FileBrowserState::new("/empty", vec![]);
    assert!(state.selected_item().is_none());
}

#[test]
fn test_provider_default_separator() {
    struct TestProvider;
    impl DirectoryProvider for TestProvider {
        fn list_entries(&self, _path: &str) -> Vec<FileEntry> {
            vec![]
        }
        fn parent_path(&self, _path: &str) -> Option<String> {
            None
        }
    }

    let provider = TestProvider;
    assert_eq!(provider.separator(), "/");
}
