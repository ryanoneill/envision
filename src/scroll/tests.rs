use ratatui::Terminal;

use super::*;
use crate::backend::CaptureBackend;

fn setup_render(width: u16, height: u16) -> Terminal<CaptureBackend> {
    let backend = CaptureBackend::new(width, height);
    Terminal::new(backend).unwrap()
}

// =============================================================================
// Construction and defaults
// =============================================================================

#[test]
fn test_default() {
    let scroll = ScrollState::default();
    assert_eq!(scroll.offset(), 0);
    assert_eq!(scroll.content_length(), 0);
    assert_eq!(scroll.viewport_height(), 0);
}

#[test]
fn test_new() {
    let scroll = ScrollState::new(100);
    assert_eq!(scroll.offset(), 0);
    assert_eq!(scroll.content_length(), 100);
    assert_eq!(scroll.viewport_height(), 0);
}

#[test]
fn test_new_zero_content() {
    let scroll = ScrollState::new(0);
    assert_eq!(scroll.content_length(), 0);
    assert!(!scroll.can_scroll());
}

// =============================================================================
// Setters
// =============================================================================

#[test]
fn test_set_content_length() {
    let mut scroll = ScrollState::new(100);
    scroll.set_viewport_height(10);
    scroll.set_content_length(200);
    assert_eq!(scroll.content_length(), 200);
}

#[test]
fn test_set_content_length_clamps_offset() {
    let mut scroll = ScrollState::new(100);
    scroll.set_viewport_height(10);
    scroll.scroll_to_end();
    assert_eq!(scroll.offset(), 90);

    // Shrink content — offset should clamp
    scroll.set_content_length(50);
    assert_eq!(scroll.offset(), 40); // 50 - 10
}

#[test]
fn test_set_content_length_to_less_than_viewport() {
    let mut scroll = ScrollState::new(100);
    scroll.set_viewport_height(10);
    scroll.set_offset(50);

    scroll.set_content_length(5);
    assert_eq!(scroll.offset(), 0); // content < viewport, offset must be 0
}

#[test]
fn test_set_viewport_height() {
    let mut scroll = ScrollState::new(100);
    scroll.set_viewport_height(20);
    assert_eq!(scroll.viewport_height(), 20);
}

#[test]
fn test_set_viewport_height_clamps_offset() {
    let mut scroll = ScrollState::new(100);
    scroll.set_viewport_height(10);
    scroll.scroll_to_end();
    assert_eq!(scroll.offset(), 90);

    // Increase viewport — offset should clamp
    scroll.set_viewport_height(50);
    assert_eq!(scroll.offset(), 50); // 100 - 50
}

#[test]
fn test_set_offset() {
    let mut scroll = ScrollState::new(100);
    scroll.set_viewport_height(10);
    scroll.set_offset(25);
    assert_eq!(scroll.offset(), 25);
}

#[test]
fn test_set_offset_clamped() {
    let mut scroll = ScrollState::new(100);
    scroll.set_viewport_height(10);
    scroll.set_offset(999);
    assert_eq!(scroll.offset(), 90); // max_offset
}

// =============================================================================
// Scroll operations
// =============================================================================

#[test]
fn test_scroll_up() {
    let mut scroll = ScrollState::new(100);
    scroll.set_viewport_height(10);
    scroll.set_offset(5);

    assert!(scroll.scroll_up());
    assert_eq!(scroll.offset(), 4);
}

#[test]
fn test_scroll_up_at_start() {
    let mut scroll = ScrollState::new(100);
    scroll.set_viewport_height(10);

    assert!(!scroll.scroll_up());
    assert_eq!(scroll.offset(), 0);
}

#[test]
fn test_scroll_down() {
    let mut scroll = ScrollState::new(100);
    scroll.set_viewport_height(10);

    assert!(scroll.scroll_down());
    assert_eq!(scroll.offset(), 1);
}

#[test]
fn test_scroll_down_at_end() {
    let mut scroll = ScrollState::new(100);
    scroll.set_viewport_height(10);
    scroll.scroll_to_end();

    assert!(!scroll.scroll_down());
    assert_eq!(scroll.offset(), 90);
}

#[test]
fn test_scroll_up_by() {
    let mut scroll = ScrollState::new(100);
    scroll.set_viewport_height(10);
    scroll.set_offset(20);

    assert!(scroll.scroll_up_by(5));
    assert_eq!(scroll.offset(), 15);
}

#[test]
fn test_scroll_up_by_saturates() {
    let mut scroll = ScrollState::new(100);
    scroll.set_viewport_height(10);
    scroll.set_offset(3);

    assert!(scroll.scroll_up_by(10));
    assert_eq!(scroll.offset(), 0); // saturating_sub
}

#[test]
fn test_scroll_down_by() {
    let mut scroll = ScrollState::new(100);
    scroll.set_viewport_height(10);

    assert!(scroll.scroll_down_by(5));
    assert_eq!(scroll.offset(), 5);
}

#[test]
fn test_scroll_down_by_clamped() {
    let mut scroll = ScrollState::new(100);
    scroll.set_viewport_height(10);
    scroll.set_offset(85);

    assert!(scroll.scroll_down_by(20));
    assert_eq!(scroll.offset(), 90); // clamped to max_offset
}

#[test]
fn test_page_up() {
    let mut scroll = ScrollState::new(100);
    scroll.set_viewport_height(10);
    scroll.set_offset(25);

    assert!(scroll.page_up(10));
    assert_eq!(scroll.offset(), 15);
}

#[test]
fn test_page_down() {
    let mut scroll = ScrollState::new(100);
    scroll.set_viewport_height(10);

    assert!(scroll.page_down(10));
    assert_eq!(scroll.offset(), 10);
}

#[test]
fn test_scroll_to_start() {
    let mut scroll = ScrollState::new(100);
    scroll.set_viewport_height(10);
    scroll.set_offset(50);

    assert!(scroll.scroll_to_start());
    assert_eq!(scroll.offset(), 0);
}

#[test]
fn test_scroll_to_start_already_there() {
    let mut scroll = ScrollState::new(100);
    scroll.set_viewport_height(10);

    assert!(!scroll.scroll_to_start());
}

#[test]
fn test_scroll_to_end() {
    let mut scroll = ScrollState::new(100);
    scroll.set_viewport_height(10);

    assert!(scroll.scroll_to_end());
    assert_eq!(scroll.offset(), 90);
}

#[test]
fn test_scroll_to_end_already_there() {
    let mut scroll = ScrollState::new(100);
    scroll.set_viewport_height(10);
    scroll.scroll_to_end();

    assert!(!scroll.scroll_to_end());
}

// =============================================================================
// Selection tracking
// =============================================================================

#[test]
fn test_ensure_visible_already_visible() {
    let mut scroll = ScrollState::new(100);
    scroll.set_viewport_height(10);

    // Item 5 is in range 0..10, already visible
    assert!(!scroll.ensure_visible(5));
    assert_eq!(scroll.offset(), 0);
}

#[test]
fn test_ensure_visible_scrolls_down() {
    let mut scroll = ScrollState::new(100);
    scroll.set_viewport_height(10);

    // Item 15 is below viewport 0..10
    assert!(scroll.ensure_visible(15));
    assert_eq!(scroll.offset(), 6); // 15 - 10 + 1
    assert!(scroll.visible_range().contains(&15));
}

#[test]
fn test_ensure_visible_scrolls_up() {
    let mut scroll = ScrollState::new(100);
    scroll.set_viewport_height(10);
    scroll.set_offset(20);

    // Item 5 is above viewport 20..30
    assert!(scroll.ensure_visible(5));
    assert_eq!(scroll.offset(), 5);
    assert!(scroll.visible_range().contains(&5));
}

#[test]
fn test_ensure_visible_at_viewport_boundary_bottom() {
    let mut scroll = ScrollState::new(100);
    scroll.set_viewport_height(10);

    // Item 9 is the last visible item in 0..10 — should not scroll
    assert!(!scroll.ensure_visible(9));
    assert_eq!(scroll.offset(), 0);
}

#[test]
fn test_ensure_visible_just_below_viewport() {
    let mut scroll = ScrollState::new(100);
    scroll.set_viewport_height(10);

    // Item 10 is just below viewport 0..10
    assert!(scroll.ensure_visible(10));
    assert_eq!(scroll.offset(), 1); // 10 - 10 + 1
}

#[test]
fn test_ensure_visible_at_viewport_boundary_top() {
    let mut scroll = ScrollState::new(100);
    scroll.set_viewport_height(10);
    scroll.set_offset(5);

    // Item 5 is the first visible item in 5..15 — should not scroll
    assert!(!scroll.ensure_visible(5));
    assert_eq!(scroll.offset(), 5);
}

#[test]
fn test_ensure_visible_just_above_viewport() {
    let mut scroll = ScrollState::new(100);
    scroll.set_viewport_height(10);
    scroll.set_offset(5);

    // Item 4 is just above viewport 5..15
    assert!(scroll.ensure_visible(4));
    assert_eq!(scroll.offset(), 4);
}

#[test]
fn test_ensure_visible_last_item() {
    let mut scroll = ScrollState::new(100);
    scroll.set_viewport_height(10);

    assert!(scroll.ensure_visible(99));
    assert_eq!(scroll.offset(), 90); // max_offset
    assert!(scroll.visible_range().contains(&99));
}

#[test]
fn test_ensure_visible_first_item() {
    let mut scroll = ScrollState::new(100);
    scroll.set_viewport_height(10);
    scroll.set_offset(50);

    assert!(scroll.ensure_visible(0));
    assert_eq!(scroll.offset(), 0);
}

#[test]
fn test_ensure_visible_zero_viewport() {
    let mut scroll = ScrollState::new(100);
    // viewport_height is 0 — should not change anything
    assert!(!scroll.ensure_visible(50));
    assert_eq!(scroll.offset(), 0);
}

// =============================================================================
// Queries
// =============================================================================

#[test]
fn test_visible_range() {
    let mut scroll = ScrollState::new(50);
    scroll.set_viewport_height(10);

    assert_eq!(scroll.visible_range(), 0..10);

    scroll.set_offset(20);
    assert_eq!(scroll.visible_range(), 20..30);

    scroll.scroll_to_end();
    assert_eq!(scroll.visible_range(), 40..50);
}

#[test]
fn test_visible_range_content_smaller_than_viewport() {
    let mut scroll = ScrollState::new(5);
    scroll.set_viewport_height(10);

    assert_eq!(scroll.visible_range(), 0..5);
}

#[test]
fn test_visible_range_empty_content() {
    let mut scroll = ScrollState::new(0);
    scroll.set_viewport_height(10);

    assert_eq!(scroll.visible_range(), 0..0);
}

#[test]
fn test_max_offset() {
    let mut scroll = ScrollState::new(100);
    scroll.set_viewport_height(10);
    assert_eq!(scroll.max_offset(), 90);
}

#[test]
fn test_max_offset_content_smaller_than_viewport() {
    let mut scroll = ScrollState::new(5);
    scroll.set_viewport_height(10);
    assert_eq!(scroll.max_offset(), 0);
}

#[test]
fn test_can_scroll() {
    let mut scroll = ScrollState::new(100);
    scroll.set_viewport_height(10);
    assert!(scroll.can_scroll());
}

#[test]
fn test_can_scroll_false() {
    let mut scroll = ScrollState::new(5);
    scroll.set_viewport_height(10);
    assert!(!scroll.can_scroll());
}

#[test]
fn test_can_scroll_equal() {
    let mut scroll = ScrollState::new(10);
    scroll.set_viewport_height(10);
    assert!(!scroll.can_scroll());
}

#[test]
fn test_at_start() {
    let scroll = ScrollState::new(100);
    assert!(scroll.at_start());
}

#[test]
fn test_at_start_false() {
    let mut scroll = ScrollState::new(100);
    scroll.set_viewport_height(10);
    scroll.scroll_down();
    assert!(!scroll.at_start());
}

#[test]
fn test_at_end() {
    let mut scroll = ScrollState::new(100);
    scroll.set_viewport_height(10);
    scroll.scroll_to_end();
    assert!(scroll.at_end());
}

#[test]
fn test_at_end_false() {
    let mut scroll = ScrollState::new(100);
    scroll.set_viewport_height(10);
    assert!(!scroll.at_end());
}

#[test]
fn test_at_end_when_content_fits_viewport() {
    let mut scroll = ScrollState::new(5);
    scroll.set_viewport_height(10);
    assert!(scroll.at_end()); // max_offset is 0, offset is 0
}

// =============================================================================
// Scrollbar integration
// =============================================================================

#[test]
fn test_scrollbar_state() {
    let mut scroll = ScrollState::new(100);
    scroll.set_viewport_height(10);
    scroll.set_offset(25);

    let state = scroll.scrollbar_state();
    // ScrollbarState doesn't expose internals directly, but we can verify
    // it was created without panicking
    let _ = state;
}

// =============================================================================
// Edge cases
// =============================================================================

#[test]
fn test_single_item() {
    let mut scroll = ScrollState::new(1);
    scroll.set_viewport_height(10);

    assert!(!scroll.can_scroll());
    assert!(scroll.at_start());
    assert!(scroll.at_end());
    assert_eq!(scroll.visible_range(), 0..1);
    assert!(!scroll.scroll_down());
    assert!(!scroll.scroll_up());
}

#[test]
fn test_viewport_equals_content() {
    let mut scroll = ScrollState::new(10);
    scroll.set_viewport_height(10);

    assert!(!scroll.can_scroll());
    assert_eq!(scroll.max_offset(), 0);
    assert_eq!(scroll.visible_range(), 0..10);
}

#[test]
fn test_viewport_one_more_than_content() {
    let mut scroll = ScrollState::new(9);
    scroll.set_viewport_height(10);

    assert!(!scroll.can_scroll());
    assert_eq!(scroll.visible_range(), 0..9);
}

#[test]
fn test_content_one_more_than_viewport() {
    let mut scroll = ScrollState::new(11);
    scroll.set_viewport_height(10);

    assert!(scroll.can_scroll());
    assert_eq!(scroll.max_offset(), 1);

    scroll.scroll_down();
    assert_eq!(scroll.visible_range(), 1..11);
    assert!(scroll.at_end());
}

#[test]
fn test_large_content() {
    let mut scroll = ScrollState::new(1_000_000);
    scroll.set_viewport_height(50);

    scroll.scroll_to_end();
    assert_eq!(scroll.offset(), 999_950);
    assert_eq!(scroll.visible_range(), 999_950..1_000_000);

    scroll.ensure_visible(500_000);
    assert!(scroll.visible_range().contains(&500_000));
}

#[test]
fn test_scroll_down_by_zero() {
    let mut scroll = ScrollState::new(100);
    scroll.set_viewport_height(10);
    assert!(!scroll.scroll_down_by(0));
    assert_eq!(scroll.offset(), 0);
}

#[test]
fn test_scroll_up_by_zero() {
    let mut scroll = ScrollState::new(100);
    scroll.set_viewport_height(10);
    scroll.set_offset(5);
    assert!(!scroll.scroll_up_by(0));
    assert_eq!(scroll.offset(), 5);
}

#[test]
fn test_page_up_from_near_start() {
    let mut scroll = ScrollState::new(100);
    scroll.set_viewport_height(10);
    scroll.set_offset(3);

    assert!(scroll.page_up(10));
    assert_eq!(scroll.offset(), 0); // saturating sub
}

#[test]
fn test_page_down_near_end() {
    let mut scroll = ScrollState::new(100);
    scroll.set_viewport_height(10);
    scroll.set_offset(85);

    assert!(scroll.page_down(10));
    assert_eq!(scroll.offset(), 90); // clamped
}

#[test]
fn test_clone_and_eq() {
    let mut scroll = ScrollState::new(100);
    scroll.set_viewport_height(10);
    scroll.set_offset(25);

    let cloned = scroll.clone();
    assert_eq!(scroll, cloned);
}

#[test]
fn test_debug_format() {
    let scroll = ScrollState::new(100);
    let debug = format!("{:?}", scroll);
    assert!(debug.contains("ScrollState"));
    assert!(debug.contains("100"));
}

// =============================================================================
// Render helpers (smoke tests)
// =============================================================================

#[test]
fn test_render_scrollbar_no_scroll_needed() {
    let scroll = ScrollState::new(5);
    let theme = Theme::default();
    let mut terminal = setup_render(40, 10);

    terminal
        .draw(|frame| {
            let area = frame.area();
            render_scrollbar(&scroll, frame, area, &theme);
        })
        .unwrap();
    // Should not panic — scrollbar is not rendered when can_scroll is false
}

#[test]
fn test_render_scrollbar_with_content() {
    let mut scroll = ScrollState::new(100);
    scroll.set_viewport_height(10);
    let theme = Theme::default();
    let mut terminal = setup_render(40, 10);

    terminal
        .draw(|frame| {
            let area = frame.area();
            render_scrollbar(&scroll, frame, area, &theme);
        })
        .unwrap();
    // Should render without panicking
}

#[test]
fn test_render_scrollbar_inside_border() {
    let mut scroll = ScrollState::new(100);
    scroll.set_viewport_height(8); // 10 - 2 for borders
    let theme = Theme::default();
    let mut terminal = setup_render(40, 10);

    terminal
        .draw(|frame| {
            let area = frame.area();
            render_scrollbar_inside_border(&scroll, frame, area, &theme);
        })
        .unwrap();
}

#[test]
fn test_render_scrollbar_inside_border_too_small() {
    let scroll = ScrollState::new(100);
    let theme = Theme::default();
    let mut terminal = setup_render(40, 2);

    terminal
        .draw(|frame| {
            let area = frame.area();
            // Area height is 2, which is < 3, so nothing should render
            render_scrollbar_inside_border(&scroll, frame, area, &theme);
        })
        .unwrap();
}
