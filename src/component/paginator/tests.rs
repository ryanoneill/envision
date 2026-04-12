use super::*;
use crate::component::test_utils;
use crate::input::{Event, Key};

fn focused_state(total_pages: usize) -> PaginatorState {
    PaginatorState::new(total_pages)
}

// =============================================================================
// Construction
// =============================================================================

#[test]
fn test_new() {
    let state = PaginatorState::new(5);
    assert_eq!(state.current_page(), 0);
    assert_eq!(state.display_page(), 1);
    assert_eq!(state.total_pages(), 5);
    assert_eq!(state.page_size(), 10);
    assert_eq!(state.total_items(), 50);
    assert_eq!(state.style(), &PaginatorStyle::PageOfTotal);
}

#[test]
fn test_default() {
    let state = PaginatorState::default();
    assert_eq!(state.current_page(), 0);
    assert_eq!(state.total_pages(), 1);
    assert_eq!(state.page_size(), 10);
}

#[test]
fn test_init() {
    let state = Paginator::init();
    assert_eq!(state.current_page(), 0);
    assert_eq!(state.total_pages(), 1);
}

#[test]
fn test_from_items() {
    let state = PaginatorState::from_items(247, 25);
    assert_eq!(state.total_pages(), 10);
    assert_eq!(state.total_items(), 247);
    assert_eq!(state.page_size(), 25);
    assert_eq!(state.current_page(), 0);
}

#[test]
fn test_from_items_exact_division() {
    let state = PaginatorState::from_items(100, 25);
    assert_eq!(state.total_pages(), 4);
}

#[test]
fn test_from_items_zero() {
    let state = PaginatorState::from_items(0, 10);
    assert_eq!(state.total_pages(), 1);
    assert_eq!(state.total_items(), 0);
}

#[test]
fn test_from_items_page_size_zero() {
    // page_size of 0 is clamped to 1
    let state = PaginatorState::from_items(10, 0);
    assert_eq!(state.page_size(), 1);
    assert_eq!(state.total_pages(), 10);
}

#[test]
fn test_with_style() {
    let state = PaginatorState::new(5).with_style(PaginatorStyle::Dots);
    assert_eq!(state.style(), &PaginatorStyle::Dots);
}

#[test]
fn test_with_page_size() {
    let state = PaginatorState::from_items(100, 10).with_page_size(25);
    assert_eq!(state.page_size(), 25);
    assert_eq!(state.total_pages(), 4);
}

#[test]
fn test_with_page_size_clamps_current() {
    let state = PaginatorState::from_items(100, 10)
        .with_current_page(9)
        .with_page_size(25);
    assert_eq!(state.total_pages(), 4);
    assert_eq!(state.current_page(), 3); // Clamped from 9
}

#[test]
fn test_with_current_page() {
    let state = PaginatorState::new(5).with_current_page(3);
    assert_eq!(state.current_page(), 3);
    assert_eq!(state.display_page(), 4);
}

#[test]
fn test_with_current_page_clamped() {
    let state = PaginatorState::new(5).with_current_page(100);
    assert_eq!(state.current_page(), 4); // Clamped to last page
}

// =============================================================================
// Page boundary checks
// =============================================================================

#[test]
fn test_is_first_page() {
    let state = PaginatorState::new(5);
    assert!(state.is_first_page());
    assert!(!state.is_last_page());
}

#[test]
fn test_is_last_page() {
    let state = PaginatorState::new(5).with_current_page(4);
    assert!(!state.is_first_page());
    assert!(state.is_last_page());
}

#[test]
fn test_single_page_is_both_first_and_last() {
    let state = PaginatorState::new(1);
    assert!(state.is_first_page());
    assert!(state.is_last_page());
}

// =============================================================================
// Range calculations
// =============================================================================

#[test]
fn test_range_start() {
    let state = PaginatorState::from_items(247, 25).with_current_page(2);
    assert_eq!(state.range_start(), 50);
}

#[test]
fn test_range_end() {
    let state = PaginatorState::from_items(247, 25).with_current_page(2);
    assert_eq!(state.range_end(), 74);
}

#[test]
fn test_range_end_last_page() {
    let state = PaginatorState::from_items(247, 25).with_current_page(9);
    assert_eq!(state.range_end(), 246); // Last item index
}

#[test]
fn test_range_start_first_page() {
    let state = PaginatorState::from_items(100, 10);
    assert_eq!(state.range_start(), 0);
    assert_eq!(state.range_end(), 9);
}

// =============================================================================
// Navigation via update
// =============================================================================

#[test]
fn test_next_page() {
    let mut state = PaginatorState::new(5);
    let output = Paginator::update(&mut state, PaginatorMessage::NextPage);
    assert_eq!(state.current_page(), 1);
    assert_eq!(output, Some(PaginatorOutput::PageChanged(1)));
}

#[test]
fn test_next_page_at_end() {
    let mut state = PaginatorState::new(5).with_current_page(4);
    let output = Paginator::update(&mut state, PaginatorMessage::NextPage);
    assert_eq!(state.current_page(), 4); // No change
    assert_eq!(output, None);
}

#[test]
fn test_prev_page() {
    let mut state = PaginatorState::new(5).with_current_page(3);
    let output = Paginator::update(&mut state, PaginatorMessage::PrevPage);
    assert_eq!(state.current_page(), 2);
    assert_eq!(output, Some(PaginatorOutput::PageChanged(2)));
}

#[test]
fn test_prev_page_at_start() {
    let mut state = PaginatorState::new(5);
    let output = Paginator::update(&mut state, PaginatorMessage::PrevPage);
    assert_eq!(state.current_page(), 0); // No change
    assert_eq!(output, None);
}

#[test]
fn test_first_page() {
    let mut state = PaginatorState::new(5).with_current_page(3);
    let output = Paginator::update(&mut state, PaginatorMessage::FirstPage);
    assert_eq!(state.current_page(), 0);
    assert_eq!(output, Some(PaginatorOutput::PageChanged(0)));
}

#[test]
fn test_first_page_already_first() {
    let mut state = PaginatorState::new(5);
    let output = Paginator::update(&mut state, PaginatorMessage::FirstPage);
    assert_eq!(output, None);
}

#[test]
fn test_last_page() {
    let mut state = PaginatorState::new(5);
    let output = Paginator::update(&mut state, PaginatorMessage::LastPage);
    assert_eq!(state.current_page(), 4);
    assert_eq!(output, Some(PaginatorOutput::PageChanged(4)));
}

#[test]
fn test_last_page_already_last() {
    let mut state = PaginatorState::new(5).with_current_page(4);
    let output = Paginator::update(&mut state, PaginatorMessage::LastPage);
    assert_eq!(output, None);
}

#[test]
fn test_go_to_page() {
    let mut state = PaginatorState::new(10);
    let output = Paginator::update(&mut state, PaginatorMessage::GoToPage(5));
    assert_eq!(state.current_page(), 5);
    assert_eq!(output, Some(PaginatorOutput::PageChanged(5)));
}

#[test]
fn test_go_to_page_clamped() {
    let mut state = PaginatorState::new(5);
    let output = Paginator::update(&mut state, PaginatorMessage::GoToPage(100));
    assert_eq!(state.current_page(), 4);
    assert_eq!(output, Some(PaginatorOutput::PageChanged(4)));
}

#[test]
fn test_go_to_page_same_page() {
    let mut state = PaginatorState::new(5).with_current_page(3);
    let output = Paginator::update(&mut state, PaginatorMessage::GoToPage(3));
    assert_eq!(output, None); // No change
}

#[test]
fn test_set_total_pages_message() {
    let mut state = PaginatorState::new(10).with_current_page(8);
    let output = Paginator::update(&mut state, PaginatorMessage::SetTotalPages(5));
    assert_eq!(state.total_pages(), 5);
    assert_eq!(state.current_page(), 4); // Clamped
    assert_eq!(output, None);
}

#[test]
fn test_set_total_items_message() {
    let mut state = PaginatorState::from_items(100, 10).with_current_page(5);
    let output = Paginator::update(&mut state, PaginatorMessage::SetTotalItems(30));
    assert_eq!(state.total_pages(), 3);
    assert_eq!(state.current_page(), 2); // Clamped
    assert_eq!(output, None);
}

// =============================================================================
// State mutators
// =============================================================================

#[test]
fn test_set_current_page() {
    let mut state = PaginatorState::new(5);
    state.set_current_page(3);
    assert_eq!(state.current_page(), 3);
}

#[test]
fn test_set_current_page_clamped() {
    let mut state = PaginatorState::new(5);
    state.set_current_page(100);
    assert_eq!(state.current_page(), 4);
}

#[test]
fn test_set_total_pages() {
    let mut state = PaginatorState::new(10).with_current_page(8);
    state.set_total_pages(5);
    assert_eq!(state.total_pages(), 5);
    assert_eq!(state.current_page(), 4);
}

#[test]
fn test_set_total_pages_zero_becomes_one() {
    let mut state = PaginatorState::new(5);
    state.set_total_pages(0);
    assert_eq!(state.total_pages(), 1);
}

#[test]
fn test_set_total_items() {
    let mut state = PaginatorState::from_items(100, 10).with_current_page(5);
    state.set_total_items(30);
    assert_eq!(state.total_items(), 30);
    assert_eq!(state.total_pages(), 3);
    assert_eq!(state.current_page(), 2);
}

#[test]
fn test_set_total_items_zero() {
    let mut state = PaginatorState::from_items(100, 10);
    state.set_total_items(0);
    assert_eq!(state.total_pages(), 1);
    assert_eq!(state.current_page(), 0);
}

// =============================================================================
// Event handling
// =============================================================================

#[test]
fn test_handle_event_right_next() {
    let state = focused_state(5);
    let msg = Paginator::handle_event(
        &state,
        &Event::key(Key::Right),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(PaginatorMessage::NextPage));
}

#[test]
fn test_handle_event_l_next() {
    let state = focused_state(5);
    let msg = Paginator::handle_event(
        &state,
        &Event::char('l'),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(PaginatorMessage::NextPage));
}

#[test]
fn test_handle_event_left_prev() {
    let state = focused_state(5);
    let msg = Paginator::handle_event(
        &state,
        &Event::key(Key::Left),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(PaginatorMessage::PrevPage));
}

#[test]
fn test_handle_event_h_prev() {
    let state = focused_state(5);
    let msg = Paginator::handle_event(
        &state,
        &Event::char('h'),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(PaginatorMessage::PrevPage));
}

#[test]
fn test_handle_event_home() {
    let state = focused_state(5);
    let msg = Paginator::handle_event(
        &state,
        &Event::key(Key::Home),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(PaginatorMessage::FirstPage));
}

#[test]
fn test_handle_event_end() {
    let state = focused_state(5);
    let msg = Paginator::handle_event(
        &state,
        &Event::key(Key::End),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(PaginatorMessage::LastPage));
}

#[test]
fn test_handle_event_unrecognized() {
    let state = focused_state(5);
    let msg = Paginator::handle_event(
        &state,
        &Event::char('x'),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, None);
}

#[test]
fn test_handle_event_unfocused_ignores() {
    let state = PaginatorState::new(5);
    let msg = Paginator::handle_event(&state, &Event::key(Key::Right), &EventContext::default());
    assert_eq!(msg, None);
}

#[test]
fn test_handle_event_disabled_ignores() {
    let state = focused_state(5);
    let msg = Paginator::handle_event(
        &state,
        &Event::key(Key::Right),
        &EventContext::new().focused(true).disabled(true),
    );
    assert_eq!(msg, None);
}

// =============================================================================
// Instance methods
// =============================================================================

#[test]
fn test_instance_handle_event() {
    let state = focused_state(5);
    let msg = Paginator::handle_event(
        &state,
        &Event::key(Key::Right),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(PaginatorMessage::NextPage));
}

#[test]
fn test_instance_dispatch_event() {
    let mut state = focused_state(5);
    let output = Paginator::dispatch_event(
        &mut state,
        &Event::key(Key::Right),
        &EventContext::new().focused(true),
    );
    assert_eq!(output, Some(PaginatorOutput::PageChanged(1)));
    assert_eq!(state.current_page(), 1);
}

#[test]
fn test_instance_update() {
    let mut state = PaginatorState::new(5);
    let output = state.update(PaginatorMessage::NextPage);
    assert_eq!(output, Some(PaginatorOutput::PageChanged(1)));
}

// =============================================================================
// Number formatting
// =============================================================================

#[test]
fn test_format_number_small() {
    assert_eq!(format_number(0), "0");
    assert_eq!(format_number(1), "1");
    assert_eq!(format_number(999), "999");
}

#[test]
fn test_format_number_thousands() {
    assert_eq!(format_number(1000), "1,000");
    assert_eq!(format_number(2847), "2,847");
    assert_eq!(format_number(10000), "10,000");
    assert_eq!(format_number(1000000), "1,000,000");
}

// =============================================================================
// calculate_total_pages
// =============================================================================

#[test]
fn test_calculate_total_pages() {
    assert_eq!(calculate_total_pages(0, 10), 1);
    assert_eq!(calculate_total_pages(1, 10), 1);
    assert_eq!(calculate_total_pages(10, 10), 1);
    assert_eq!(calculate_total_pages(11, 10), 2);
    assert_eq!(calculate_total_pages(100, 10), 10);
    assert_eq!(calculate_total_pages(247, 25), 10);
}

// =============================================================================
// PaginatorStyle
// =============================================================================

#[test]
fn test_style_default() {
    let style = PaginatorStyle::default();
    assert_eq!(style, PaginatorStyle::PageOfTotal);
}

#[test]
fn test_style_variants() {
    let _ = PaginatorStyle::PageOfTotal;
    let _ = PaginatorStyle::RangeOfTotal;
    let _ = PaginatorStyle::Dots;
    let _ = PaginatorStyle::Compact;
}

// =============================================================================
// Edge cases
// =============================================================================

#[test]
fn test_new_with_zero_total_pages() {
    // 0 total pages should be clamped to 1
    let state = PaginatorState::new(0);
    assert_eq!(state.total_pages(), 1);
    assert_eq!(state.current_page(), 0);
}

#[test]
fn test_single_page() {
    let mut state = PaginatorState::new(1);
    assert!(state.is_first_page());
    assert!(state.is_last_page());

    let output = Paginator::update(&mut state, PaginatorMessage::NextPage);
    assert_eq!(output, None);

    let output = Paginator::update(&mut state, PaginatorMessage::PrevPage);
    assert_eq!(output, None);
}

#[test]
fn test_default_matches_init() {
    let default_state = PaginatorState::default();
    let init_state = Paginator::init();

    assert_eq!(default_state.current_page(), init_state.current_page());
    assert_eq!(default_state.total_pages(), init_state.total_pages());
    assert_eq!(default_state.page_size(), init_state.page_size());
    assert_eq!(default_state.style(), init_state.style());
}

// =============================================================================
// Snapshot tests - PageOfTotal
// =============================================================================

#[test]
fn test_view_page_of_total_first_page() {
    let state = PaginatorState::new(12);
    let (mut terminal, theme) = test_utils::setup_render(40, 1);
    terminal
        .draw(|frame| {
            Paginator::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_page_of_total_middle_page() {
    let state = PaginatorState::new(12).with_current_page(5);
    let (mut terminal, theme) = test_utils::setup_render(40, 1);
    terminal
        .draw(|frame| {
            Paginator::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_page_of_total_last_page() {
    let state = PaginatorState::new(12).with_current_page(11);
    let (mut terminal, theme) = test_utils::setup_render(40, 1);
    terminal
        .draw(|frame| {
            Paginator::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

// =============================================================================
// Snapshot tests - RangeOfTotal
// =============================================================================

#[test]
fn test_view_range_of_total_first_page() {
    let state = PaginatorState::from_items(247, 25).with_style(PaginatorStyle::RangeOfTotal);
    let (mut terminal, theme) = test_utils::setup_render(40, 1);
    terminal
        .draw(|frame| {
            Paginator::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_range_of_total_middle_page() {
    let state = PaginatorState::from_items(2847, 100)
        .with_style(PaginatorStyle::RangeOfTotal)
        .with_current_page(2);
    let (mut terminal, theme) = test_utils::setup_render(40, 1);
    terminal
        .draw(|frame| {
            Paginator::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_range_of_total_last_page() {
    let state = PaginatorState::from_items(247, 25)
        .with_style(PaginatorStyle::RangeOfTotal)
        .with_current_page(9);
    let (mut terminal, theme) = test_utils::setup_render(40, 1);
    terminal
        .draw(|frame| {
            Paginator::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_range_of_total_zero_items() {
    let state = PaginatorState::from_items(0, 10).with_style(PaginatorStyle::RangeOfTotal);
    let (mut terminal, theme) = test_utils::setup_render(40, 1);
    terminal
        .draw(|frame| {
            Paginator::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

// =============================================================================
// Snapshot tests - Dots
// =============================================================================

#[test]
fn test_view_dots_first_page() {
    let state = PaginatorState::new(5).with_style(PaginatorStyle::Dots);
    let (mut terminal, theme) = test_utils::setup_render(40, 1);
    terminal
        .draw(|frame| {
            Paginator::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_dots_middle_page() {
    let state = PaginatorState::new(5)
        .with_style(PaginatorStyle::Dots)
        .with_current_page(2);
    let (mut terminal, theme) = test_utils::setup_render(40, 1);
    terminal
        .draw(|frame| {
            Paginator::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_dots_last_page() {
    let state = PaginatorState::new(5)
        .with_style(PaginatorStyle::Dots)
        .with_current_page(4);
    let (mut terminal, theme) = test_utils::setup_render(40, 1);
    terminal
        .draw(|frame| {
            Paginator::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_dots_single_page() {
    let state = PaginatorState::new(1).with_style(PaginatorStyle::Dots);
    let (mut terminal, theme) = test_utils::setup_render(40, 1);
    terminal
        .draw(|frame| {
            Paginator::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_dots_many_pages() {
    let state = PaginatorState::new(20)
        .with_style(PaginatorStyle::Dots)
        .with_current_page(10);
    let (mut terminal, theme) = test_utils::setup_render(50, 1);
    terminal
        .draw(|frame| {
            Paginator::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

// =============================================================================
// Snapshot tests - Compact
// =============================================================================

#[test]
fn test_view_compact_first_page() {
    let state = PaginatorState::new(12).with_style(PaginatorStyle::Compact);
    let (mut terminal, theme) = test_utils::setup_render(40, 1);
    terminal
        .draw(|frame| {
            Paginator::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_compact_middle_page() {
    let state = PaginatorState::new(12)
        .with_style(PaginatorStyle::Compact)
        .with_current_page(5);
    let (mut terminal, theme) = test_utils::setup_render(40, 1);
    terminal
        .draw(|frame| {
            Paginator::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_compact_last_page() {
    let state = PaginatorState::new(12)
        .with_style(PaginatorStyle::Compact)
        .with_current_page(11);
    let (mut terminal, theme) = test_utils::setup_render(40, 1);
    terminal
        .draw(|frame| {
            Paginator::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

// =============================================================================
// Snapshot tests - Focused and Disabled
// =============================================================================

#[test]
fn test_view_focused() {
    let state = focused_state(5);
    let (mut terminal, theme) = test_utils::setup_render(40, 1);
    terminal
        .draw(|frame| {
            Paginator::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_disabled() {
    let state = PaginatorState::new(5);
    let (mut terminal, theme) = test_utils::setup_render(40, 1);
    terminal
        .draw(|frame| {
            Paginator::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).disabled(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

// =============================================================================
// Annotation tests
// =============================================================================

#[test]
fn test_annotation_emitted() {
    use crate::annotation::{WidgetType, with_annotations};
    let state = PaginatorState::new(5).with_current_page(2);
    let (mut terminal, theme) = test_utils::setup_render(40, 1);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                Paginator::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
            })
            .unwrap();
    });
    assert_eq!(registry.len(), 1);
    let regions = registry.find_by_type(&WidgetType::Paginator);
    assert_eq!(regions.len(), 1);
    assert!(!regions[0].annotation.focused);
    assert!(!regions[0].annotation.disabled);
    assert_eq!(regions[0].annotation.value, Some("3/5".to_string()));
}

#[test]
fn test_annotation_focused() {
    use crate::annotation::{WidgetType, with_annotations};
    let state = focused_state(5);
    let (mut terminal, theme) = test_utils::setup_render(40, 1);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                Paginator::view(
                    &state,
                    &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
                );
            })
            .unwrap();
    });
    let regions = registry.find_by_type(&WidgetType::Paginator);
    assert_eq!(regions.len(), 1);
    assert!(regions[0].annotation.focused);
}

// =============================================================================
// Dot rendering
// =============================================================================

#[test]
fn test_render_dots_small() {
    let state = PaginatorState::new(3)
        .with_style(PaginatorStyle::Dots)
        .with_current_page(1);
    let result = render_dots(&state);
    assert_eq!(result, "○ ● ○");
}

#[test]
fn test_render_dots_ten() {
    let state = PaginatorState::new(10)
        .with_style(PaginatorStyle::Dots)
        .with_current_page(0);
    let result = render_dots(&state);
    assert_eq!(result, "● ○ ○ ○ ○ ○ ○ ○ ○ ○");
}

#[test]
fn test_render_dots_single() {
    let state = PaginatorState::new(1).with_style(PaginatorStyle::Dots);
    let result = render_dots(&state);
    assert_eq!(result, "●");
}
