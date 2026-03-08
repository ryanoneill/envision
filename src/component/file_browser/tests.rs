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

fn sample_entries_with_hidden() -> Vec<FileEntry> {
    vec![
        FileEntry::directory("src", "/src"),
        FileEntry::directory(".git", "/.git"),
        FileEntry::file("Cargo.toml", "/Cargo.toml"),
        FileEntry::file(".gitignore", "/.gitignore"),
    ]
}

fn focused_state() -> FileBrowserState {
    let mut state = FileBrowserState::new("/", sample_entries());
    FileBrowser::set_focused(&mut state, true);
    state
}

// =============================================================================
// Construction and defaults
// =============================================================================

#[test]
fn test_new_creates_state_with_entries() {
    let state = FileBrowserState::new("/", sample_entries());
    assert_eq!(state.current_path(), "/");
    assert_eq!(state.entries().len(), 5);
}

#[test]
fn test_new_selects_first_entry() {
    let state = FileBrowserState::new("/", sample_entries());
    assert_eq!(state.selected_index(), Some(0));
}

#[test]
fn test_new_computes_path_segments() {
    let state = FileBrowserState::new("/home/user/project", sample_entries());
    assert_eq!(state.path_segments(), &["/", "home", "user", "project"]);
}

#[test]
fn test_new_root_path_segments() {
    let state = FileBrowserState::new("/", sample_entries());
    assert_eq!(state.path_segments(), &["/"]);
}

#[test]
fn test_new_empty_entries() {
    let state = FileBrowserState::new("/", vec![]);
    assert_eq!(state.entries().len(), 0);
    assert_eq!(state.selected_index(), None);
    assert_eq!(state.selected_entry(), None);
}

#[test]
fn test_default_state() {
    let state = FileBrowserState::default();
    assert_eq!(state.current_path(), "/");
    assert!(state.entries().is_empty());
    assert!(!state.is_focused());
    assert!(!state.is_disabled());
    assert!(!state.show_hidden());
    assert_eq!(state.filter_text(), "");
    assert_eq!(*state.sort_field(), FileSortField::Name);
    assert_eq!(*state.sort_direction(), FileSortDirection::Ascending);
}

// =============================================================================
// Builder methods
// =============================================================================

#[test]
fn test_with_selection_mode() {
    let state = FileBrowserState::new("/", sample_entries())
        .with_selection_mode(SelectionMode::MultipleFiles);
    assert_eq!(*state.selection_mode(), SelectionMode::MultipleFiles);
}

#[test]
fn test_with_sort_field() {
    let state = FileBrowserState::new("/", sample_entries()).with_sort_field(FileSortField::Size);
    assert_eq!(*state.sort_field(), FileSortField::Size);
}

#[test]
fn test_with_sort_direction() {
    let state = FileBrowserState::new("/", sample_entries())
        .with_sort_direction(FileSortDirection::Descending);
    assert_eq!(*state.sort_direction(), FileSortDirection::Descending);
}

#[test]
fn test_with_show_hidden() {
    let state = FileBrowserState::new("/", sample_entries_with_hidden()).with_show_hidden(true);
    assert!(state.show_hidden());
    // All 4 entries visible when hidden shown
    assert_eq!(state.filtered_indices().len(), 4);
}

#[test]
fn test_with_disabled() {
    let state = FileBrowserState::new("/", sample_entries()).with_disabled(true);
    assert!(state.is_disabled());
}

#[test]
fn test_with_directories_first() {
    let state = FileBrowserState::new("/", sample_entries()).with_directories_first(false);
    // When not directories_first, everything sorted alphabetically
    let first = state.selected_entry().unwrap();
    assert_eq!(first.name(), "Cargo.toml");
}

// =============================================================================
// FileEntry
// =============================================================================

#[test]
fn test_file_entry_file() {
    let entry = FileEntry::file("main.rs", "/main.rs");
    assert_eq!(entry.name(), "main.rs");
    assert_eq!(entry.path(), "/main.rs");
    assert!(!entry.is_dir());
    assert_eq!(entry.extension(), Some("rs"));
    assert!(!entry.is_hidden());
}

#[test]
fn test_file_entry_directory() {
    let entry = FileEntry::directory("src", "/src");
    assert_eq!(entry.name(), "src");
    assert_eq!(entry.path(), "/src");
    assert!(entry.is_dir());
    assert_eq!(entry.extension(), None);
}

#[test]
fn test_file_entry_hidden() {
    let entry = FileEntry::file(".gitignore", "/.gitignore");
    assert!(entry.is_hidden());
}

#[test]
fn test_file_entry_with_size() {
    let entry = FileEntry::file("test.txt", "/test.txt").with_size(1024);
    assert_eq!(entry.size(), Some(1024));
}

#[test]
fn test_file_entry_with_modified() {
    let entry = FileEntry::file("test.txt", "/test.txt").with_modified(1700000000);
    assert_eq!(entry.modified(), Some(1700000000));
}

// =============================================================================
// Sorting
// =============================================================================

#[test]
fn test_directories_first_by_default() {
    let state = FileBrowserState::new("/", sample_entries());
    let filtered = state.filtered_entries();
    // Directories should come first
    assert!(filtered[0].is_dir());
    assert!(filtered[1].is_dir());
    assert!(!filtered[2].is_dir());
}

#[test]
fn test_sort_by_name_ascending() {
    let state = FileBrowserState::new("/", sample_entries());
    let filtered = state.filtered_entries();
    // Dirs first (alphabetical): src, tests
    assert_eq!(filtered[0].name(), "src");
    assert_eq!(filtered[1].name(), "tests");
    // Then files alphabetical: Cargo.toml, README.md, main.rs
    // Note: lowercase comparison, so Cargo.toml < README.md < main.rs
    assert_eq!(filtered[2].name(), "Cargo.toml");
}

#[test]
fn test_sort_by_size() {
    let state = FileBrowserState::new("/", sample_entries())
        .with_sort_field(FileSortField::Size)
        .with_directories_first(false);
    let filtered = state.filtered_entries();
    // Size ordering: None < Some(512) < Some(1024) < Some(2048)
    // dirs (no size) first in asc, then by size
    assert_eq!(filtered[0].size(), None);
}

#[test]
fn test_sort_descending() {
    let state = FileBrowserState::new("/", sample_entries())
        .with_sort_direction(FileSortDirection::Descending);
    let filtered = state.filtered_entries();
    // Directories still first (in reverse alpha: tests, src), then files in reverse alpha
    assert!(filtered[0].is_dir());
    assert_eq!(filtered[0].name(), "tests");
    assert!(filtered[1].is_dir());
    assert_eq!(filtered[1].name(), "src");
    assert_eq!(filtered[2].name(), "README.md");
}

// =============================================================================
// Hidden files
// =============================================================================

#[test]
fn test_hidden_files_excluded_by_default() {
    let state = FileBrowserState::new("/", sample_entries_with_hidden());
    assert_eq!(state.filtered_indices().len(), 2); // src and Cargo.toml
}

#[test]
fn test_show_hidden_includes_all() {
    let state = FileBrowserState::new("/", sample_entries_with_hidden()).with_show_hidden(true);
    assert_eq!(state.filtered_indices().len(), 4);
}

#[test]
fn test_toggle_hidden() {
    let mut state = focused_state();
    state.entries = sample_entries_with_hidden();
    state.show_hidden = false;
    state.sort_and_filter();

    let initial_count = state.filtered_indices().len();
    let output = FileBrowser::update(&mut state, FileBrowserMessage::ToggleHidden);
    assert!(matches!(
        output,
        Some(FileBrowserOutput::HiddenToggled(true))
    ));
    assert!(state.filtered_indices().len() > initial_count);
}

// =============================================================================
// Navigation - Up/Down
// =============================================================================

#[test]
fn test_move_down() {
    let mut state = focused_state();
    assert_eq!(state.selected_index(), Some(0));

    let output = FileBrowser::update(&mut state, FileBrowserMessage::Down);
    assert_eq!(state.selected_index(), Some(1));
    assert!(matches!(
        output,
        Some(FileBrowserOutput::SelectionChanged(1))
    ));
}

#[test]
fn test_move_up() {
    let mut state = focused_state();
    FileBrowser::update(&mut state, FileBrowserMessage::Down);
    assert_eq!(state.selected_index(), Some(1));

    let output = FileBrowser::update(&mut state, FileBrowserMessage::Up);
    assert_eq!(state.selected_index(), Some(0));
    assert!(matches!(
        output,
        Some(FileBrowserOutput::SelectionChanged(0))
    ));
}

#[test]
fn test_move_down_wraps() {
    let mut state = focused_state();
    let count = state.filtered_indices().len();
    for _ in 0..count {
        FileBrowser::update(&mut state, FileBrowserMessage::Down);
    }
    // Should wrap back to 0
    assert_eq!(state.selected_index(), Some(0));
}

#[test]
fn test_move_up_wraps() {
    let mut state = focused_state();
    // Starting at 0, going up should wrap to last
    let output = FileBrowser::update(&mut state, FileBrowserMessage::Up);
    let last = state.filtered_indices().len() - 1;
    assert_eq!(state.selected_index(), Some(last));
    assert!(matches!(output, Some(FileBrowserOutput::SelectionChanged(n)) if n == last));
}

#[test]
fn test_first() {
    let mut state = focused_state();
    FileBrowser::update(&mut state, FileBrowserMessage::Down);
    FileBrowser::update(&mut state, FileBrowserMessage::Down);

    let output = FileBrowser::update(&mut state, FileBrowserMessage::First);
    assert_eq!(state.selected_index(), Some(0));
    assert!(matches!(
        output,
        Some(FileBrowserOutput::SelectionChanged(0))
    ));
}

#[test]
fn test_last() {
    let mut state = focused_state();
    let output = FileBrowser::update(&mut state, FileBrowserMessage::Last);
    let last = state.filtered_indices().len() - 1;
    assert_eq!(state.selected_index(), Some(last));
    assert!(matches!(output, Some(FileBrowserOutput::SelectionChanged(n)) if n == last));
}

#[test]
fn test_page_down() {
    let mut state = focused_state();
    let output = FileBrowser::update(&mut state, FileBrowserMessage::PageDown(3));
    assert_eq!(state.selected_index(), Some(3));
    assert!(matches!(
        output,
        Some(FileBrowserOutput::SelectionChanged(3))
    ));
}

#[test]
fn test_page_down_clamps_to_last() {
    let mut state = focused_state();
    let last = state.filtered_indices().len() - 1;
    let output = FileBrowser::update(&mut state, FileBrowserMessage::PageDown(100));
    assert_eq!(state.selected_index(), Some(last));
    assert!(matches!(output, Some(FileBrowserOutput::SelectionChanged(n)) if n == last));
}

#[test]
fn test_page_up() {
    let mut state = focused_state();
    FileBrowser::update(&mut state, FileBrowserMessage::Last);
    let output = FileBrowser::update(&mut state, FileBrowserMessage::PageUp(2));
    let expected = state.filtered_indices().len() - 1 - 2;
    assert_eq!(state.selected_index(), Some(expected));
    assert!(matches!(output, Some(FileBrowserOutput::SelectionChanged(n)) if n == expected));
}

#[test]
fn test_page_up_clamps_to_first() {
    let mut state = focused_state();
    FileBrowser::update(&mut state, FileBrowserMessage::Down);
    let output = FileBrowser::update(&mut state, FileBrowserMessage::PageUp(100));
    assert_eq!(state.selected_index(), Some(0));
    assert!(matches!(
        output,
        Some(FileBrowserOutput::SelectionChanged(0))
    ));
}

// =============================================================================
// Navigation on empty list
// =============================================================================

#[test]
fn test_navigate_empty_list() {
    let mut state = FileBrowserState::new("/", vec![]);
    FileBrowser::set_focused(&mut state, true);

    assert!(FileBrowser::update(&mut state, FileBrowserMessage::Up).is_none());
    assert!(FileBrowser::update(&mut state, FileBrowserMessage::Down).is_none());
    assert!(FileBrowser::update(&mut state, FileBrowserMessage::First).is_none());
    assert!(FileBrowser::update(&mut state, FileBrowserMessage::Last).is_none());
    assert!(FileBrowser::update(&mut state, FileBrowserMessage::PageUp(5)).is_none());
    assert!(FileBrowser::update(&mut state, FileBrowserMessage::PageDown(5)).is_none());
}

// =============================================================================
// Enter directory / file selection
// =============================================================================

#[test]
fn test_enter_directory_without_provider() {
    let mut state = focused_state();
    // First entry is a directory (src)
    let output = FileBrowser::update(&mut state, FileBrowserMessage::Enter);
    assert!(matches!(output, Some(FileBrowserOutput::DirectoryEntered(ref p)) if p == "/src"));
    assert_eq!(state.current_path(), "/src");
    // Without provider, new directory will be empty
    assert!(state.entries().is_empty());
}

#[test]
fn test_enter_file_returns_file_selected() {
    let mut state = focused_state();
    // Navigate past directories to a file
    let dir_count = state
        .filtered_entries()
        .iter()
        .filter(|e| e.is_dir())
        .count();
    for _ in 0..dir_count {
        FileBrowser::update(&mut state, FileBrowserMessage::Down);
    }
    // Now on first file
    let entry = state.selected_entry().unwrap().clone();
    assert!(!entry.is_dir());

    let output = FileBrowser::update(&mut state, FileBrowserMessage::Enter);
    assert!(matches!(output, Some(FileBrowserOutput::FileSelected(_))));
}

#[test]
fn test_enter_on_empty_list() {
    let mut state = FileBrowserState::new("/", vec![]);
    FileBrowser::set_focused(&mut state, true);
    assert!(FileBrowser::update(&mut state, FileBrowserMessage::Enter).is_none());
}

// =============================================================================
// Back navigation
// =============================================================================

#[test]
fn test_back_without_provider_returns_none() {
    let mut state = focused_state();
    // Without a provider, back does nothing
    let output = FileBrowser::update(&mut state, FileBrowserMessage::Back);
    assert!(output.is_none());
}

// =============================================================================
// Filter
// =============================================================================

#[test]
fn test_filter_narrows_entries() {
    let mut state = focused_state();
    FileBrowser::update(&mut state, FileBrowserMessage::FilterChar('m'));
    FileBrowser::update(&mut state, FileBrowserMessage::FilterChar('a'));
    assert_eq!(state.filter_text(), "ma");
    // Should match "main.rs"
    let filtered = state.filtered_entries();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].name(), "main.rs");
}

#[test]
fn test_filter_case_insensitive() {
    let mut state = focused_state();
    FileBrowser::update(&mut state, FileBrowserMessage::FilterChar('r'));
    // Should match "README.md"
    let filtered = state.filtered_entries();
    assert!(filtered.iter().any(|e| e.name() == "README.md"));
}

#[test]
fn test_filter_backspace() {
    let mut state = focused_state();
    FileBrowser::update(&mut state, FileBrowserMessage::FilterChar('x'));
    FileBrowser::update(&mut state, FileBrowserMessage::FilterChar('y'));
    assert_eq!(state.filter_text(), "xy");

    FileBrowser::update(&mut state, FileBrowserMessage::FilterBackspace);
    assert_eq!(state.filter_text(), "x");
}

#[test]
fn test_filter_backspace_on_empty_returns_none() {
    let mut state = focused_state();
    let output = FileBrowser::update(&mut state, FileBrowserMessage::FilterBackspace);
    assert!(output.is_none());
}

#[test]
fn test_filter_clear() {
    let mut state = focused_state();
    FileBrowser::update(&mut state, FileBrowserMessage::FilterChar('s'));
    let output = FileBrowser::update(&mut state, FileBrowserMessage::FilterClear);
    assert!(matches!(output, Some(FileBrowserOutput::FilterChanged(ref s)) if s.is_empty()));
    assert_eq!(state.filter_text(), "");
    assert_eq!(state.filtered_indices().len(), 5);
}

#[test]
fn test_filter_clear_on_empty_returns_none() {
    let mut state = focused_state();
    let output = FileBrowser::update(&mut state, FileBrowserMessage::FilterClear);
    assert!(output.is_none());
}

#[test]
fn test_filter_output_emitted() {
    let mut state = focused_state();
    let output = FileBrowser::update(&mut state, FileBrowserMessage::FilterChar('a'));
    assert!(matches!(output, Some(FileBrowserOutput::FilterChanged(ref s)) if s == "a"));
}

// =============================================================================
// Toggle selection
// =============================================================================

#[test]
fn test_toggle_select_adds_path() {
    let mut state = focused_state();
    let output = FileBrowser::update(&mut state, FileBrowserMessage::ToggleSelect);
    let entry = state.filtered_entries()[0];
    assert!(matches!(
        output,
        Some(FileBrowserOutput::SelectionToggled(_))
    ));
    assert!(state.selected_paths().contains(&entry.path().to_string()));
}

#[test]
fn test_toggle_select_removes_path() {
    let mut state = focused_state();
    FileBrowser::update(&mut state, FileBrowserMessage::ToggleSelect);
    assert_eq!(state.selected_paths().len(), 1);

    FileBrowser::update(&mut state, FileBrowserMessage::ToggleSelect);
    assert!(state.selected_paths().is_empty());
}

// =============================================================================
// Sort operations
// =============================================================================

#[test]
fn test_set_sort() {
    let mut state = focused_state();
    let output = FileBrowser::update(&mut state, FileBrowserMessage::SetSort(FileSortField::Size));
    assert_eq!(*state.sort_field(), FileSortField::Size);
    assert!(matches!(
        output,
        Some(FileBrowserOutput::SortChanged(
            FileSortField::Size,
            FileSortDirection::Ascending
        ))
    ));
}

#[test]
fn test_toggle_sort_direction() {
    let mut state = focused_state();
    let output = FileBrowser::update(&mut state, FileBrowserMessage::ToggleSortDirection);
    assert_eq!(*state.sort_direction(), FileSortDirection::Descending);
    assert!(matches!(
        output,
        Some(FileBrowserOutput::SortChanged(
            FileSortField::Name,
            FileSortDirection::Descending
        ))
    ));
}

// =============================================================================
// Breadcrumb navigation
// =============================================================================

#[test]
fn test_navigate_to_segment() {
    let entries = sample_entries();
    let mut state = FileBrowserState::new("/home/user/project", entries);
    FileBrowser::set_focused(&mut state, true);
    assert_eq!(state.path_segments().len(), 4); // /, home, user, project

    let output = FileBrowser::update(&mut state, FileBrowserMessage::NavigateToSegment(1));
    assert!(matches!(output, Some(FileBrowserOutput::DirectoryEntered(ref p)) if p == "/home"));
    assert_eq!(state.current_path(), "/home");
}

#[test]
fn test_navigate_to_root_segment() {
    let mut state = FileBrowserState::new("/home/user", sample_entries());
    FileBrowser::set_focused(&mut state, true);

    let output = FileBrowser::update(&mut state, FileBrowserMessage::NavigateToSegment(0));
    assert!(matches!(output, Some(FileBrowserOutput::DirectoryEntered(ref p)) if p == "/"));
    assert_eq!(state.current_path(), "/");
}

#[test]
fn test_navigate_to_invalid_segment() {
    let mut state = focused_state();
    let output = FileBrowser::update(&mut state, FileBrowserMessage::NavigateToSegment(100));
    assert!(output.is_none());
}

// =============================================================================
// Cycle focus
// =============================================================================

#[test]
fn test_cycle_focus() {
    let mut state = focused_state();
    // Default: FileList
    FileBrowser::update(&mut state, FileBrowserMessage::CycleFocus);
    // Now: Filter
    FileBrowser::update(&mut state, FileBrowserMessage::CycleFocus);
    // Now: PathBar
    FileBrowser::update(&mut state, FileBrowserMessage::CycleFocus);
    // Now: FileList again
    // Verify by sending a Down message - should work from FileList
    let output = FileBrowser::update(&mut state, FileBrowserMessage::Down);
    assert!(output.is_some());
}

// =============================================================================
// Focus / disabled guards
// =============================================================================

#[test]
fn test_unfocused_ignores_events() {
    let state = FileBrowserState::new("/", sample_entries());
    assert!(!state.is_focused());
    assert!(FileBrowser::handle_event(&state, &Event::key(KeyCode::Down)).is_none());
    assert!(FileBrowser::handle_event(&state, &Event::char('j')).is_none());
}

#[test]
fn test_disabled_ignores_events() {
    let mut state = focused_state();
    state.set_disabled(true);
    assert!(FileBrowser::handle_event(&state, &Event::key(KeyCode::Down)).is_none());
}

// =============================================================================
// Keyboard mapping
// =============================================================================

#[test]
fn test_key_up() {
    let state = focused_state();
    let msg = FileBrowser::handle_event(&state, &Event::key(KeyCode::Up));
    assert_eq!(msg, Some(FileBrowserMessage::Up));
}

#[test]
fn test_key_k_maps_to_up() {
    let state = focused_state();
    let msg = FileBrowser::handle_event(&state, &Event::char('k'));
    assert_eq!(msg, Some(FileBrowserMessage::Up));
}

#[test]
fn test_key_down() {
    let state = focused_state();
    let msg = FileBrowser::handle_event(&state, &Event::key(KeyCode::Down));
    assert_eq!(msg, Some(FileBrowserMessage::Down));
}

#[test]
fn test_key_j_maps_to_down() {
    let state = focused_state();
    let msg = FileBrowser::handle_event(&state, &Event::char('j'));
    assert_eq!(msg, Some(FileBrowserMessage::Down));
}

#[test]
fn test_key_home_maps_to_first() {
    let state = focused_state();
    let msg = FileBrowser::handle_event(&state, &Event::key(KeyCode::Home));
    assert_eq!(msg, Some(FileBrowserMessage::First));
}

#[test]
fn test_key_end_maps_to_last() {
    let state = focused_state();
    let msg = FileBrowser::handle_event(&state, &Event::key(KeyCode::End));
    assert_eq!(msg, Some(FileBrowserMessage::Last));
}

#[test]
fn test_key_enter_maps_to_enter() {
    let state = focused_state();
    let msg = FileBrowser::handle_event(&state, &Event::key(KeyCode::Enter));
    assert_eq!(msg, Some(FileBrowserMessage::Enter));
}

#[test]
fn test_key_backspace_maps_to_back_when_filter_empty() {
    let state = focused_state();
    let msg = FileBrowser::handle_event(&state, &Event::key(KeyCode::Backspace));
    assert_eq!(msg, Some(FileBrowserMessage::Back));
}

#[test]
fn test_key_backspace_maps_to_filter_backspace_when_filter_active() {
    let mut state = focused_state();
    FileBrowser::update(&mut state, FileBrowserMessage::FilterChar('a'));
    let msg = FileBrowser::handle_event(&state, &Event::key(KeyCode::Backspace));
    assert_eq!(msg, Some(FileBrowserMessage::FilterBackspace));
}

#[test]
fn test_key_space_maps_to_toggle_select() {
    let state = focused_state();
    let msg = FileBrowser::handle_event(&state, &Event::char(' '));
    assert_eq!(msg, Some(FileBrowserMessage::ToggleSelect));
}

#[test]
fn test_key_ctrl_h_maps_to_toggle_hidden() {
    let state = focused_state();
    let msg = FileBrowser::handle_event(&state, &Event::ctrl('h'));
    assert_eq!(msg, Some(FileBrowserMessage::ToggleHidden));
}

#[test]
fn test_key_tab_maps_to_cycle_focus() {
    let state = focused_state();
    let msg = FileBrowser::handle_event(&state, &Event::key(KeyCode::Tab));
    assert_eq!(msg, Some(FileBrowserMessage::CycleFocus));
}

#[test]
fn test_key_esc_maps_to_filter_clear() {
    let state = focused_state();
    let msg = FileBrowser::handle_event(&state, &Event::key(KeyCode::Esc));
    assert_eq!(msg, Some(FileBrowserMessage::FilterClear));
}

#[test]
fn test_alphanumeric_char_maps_to_filter_char() {
    let state = focused_state();
    let msg = FileBrowser::handle_event(&state, &Event::char('a'));
    assert_eq!(msg, Some(FileBrowserMessage::FilterChar('a')));
}

#[test]
fn test_dot_maps_to_filter_char() {
    let state = focused_state();
    let msg = FileBrowser::handle_event(&state, &Event::char('.'));
    assert_eq!(msg, Some(FileBrowserMessage::FilterChar('.')));
}

// =============================================================================
// Focus / disabled state
// =============================================================================

#[test]
fn test_focusable_trait() {
    let mut state = FileBrowserState::new("/", sample_entries());
    assert!(!FileBrowser::is_focused(&state));

    FileBrowser::set_focused(&mut state, true);
    assert!(FileBrowser::is_focused(&state));

    FileBrowser::set_focused(&mut state, false);
    assert!(!FileBrowser::is_focused(&state));
}

#[test]
fn test_set_disabled() {
    let mut state = FileBrowserState::new("/", sample_entries());
    assert!(!state.is_disabled());

    state.set_disabled(true);
    assert!(state.is_disabled());
}

// =============================================================================
// Instance method delegation
// =============================================================================

#[test]
fn test_instance_handle_event() {
    let state = focused_state();
    let msg = state.handle_event(&Event::key(KeyCode::Down));
    assert_eq!(msg, Some(FileBrowserMessage::Down));
}

#[test]
fn test_instance_dispatch_event() {
    let mut state = focused_state();
    let output = state.dispatch_event(&Event::key(KeyCode::Down));
    assert!(matches!(
        output,
        Some(FileBrowserOutput::SelectionChanged(1))
    ));
}

#[test]
fn test_instance_update() {
    let mut state = focused_state();
    let output = state.update(FileBrowserMessage::Down);
    assert!(matches!(
        output,
        Some(FileBrowserOutput::SelectionChanged(1))
    ));
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
    assert!(FileBrowser::handle_event(&state, &Event::char('j')).is_none());
    assert!(FileBrowser::handle_event(&state, &Event::key(KeyCode::Enter)).is_none());
    // Tab still works
    let msg = FileBrowser::handle_event(&state, &Event::key(KeyCode::Tab));
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
    let msg = FileBrowser::handle_event(&state, &Event::char('z'));
    assert_eq!(msg, Some(FileBrowserMessage::FilterChar('z')));
}

#[test]
fn test_filter_focus_backspace() {
    let mut state = focused_state();
    FileBrowser::update(&mut state, FileBrowserMessage::CycleFocus);
    let msg = FileBrowser::handle_event(&state, &Event::key(KeyCode::Backspace));
    assert_eq!(msg, Some(FileBrowserMessage::FilterBackspace));
}

#[test]
fn test_filter_focus_esc() {
    let mut state = focused_state();
    FileBrowser::update(&mut state, FileBrowserMessage::CycleFocus);
    let msg = FileBrowser::handle_event(&state, &Event::key(KeyCode::Esc));
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
    FileBrowser::set_focused(&mut state, true);

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
