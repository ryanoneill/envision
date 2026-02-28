use super::*;
use crate::annotation::{Annotate, Annotation};
use ratatui::widgets::Paragraph;

#[test]
fn test_harness_new() {
    let harness = TestHarness::new(80, 24);
    assert_eq!(harness.width(), 80);
    assert_eq!(harness.height(), 24);
    assert_eq!(harness.frame_count(), 0);
}

#[test]
fn test_harness_render() {
    let mut harness = TestHarness::new(80, 24);

    harness
        .render(|frame| {
            frame.render_widget(Paragraph::new("Hello, World!"), frame.area());
        })
        .unwrap();

    assert_eq!(harness.frame_count(), 1);
    assert!(harness.contains("Hello, World!"));
}

#[test]
fn test_harness_annotations() {
    let mut harness = TestHarness::new(80, 24);

    harness
        .render(|frame| {
            let area = frame.area();
            frame.render_widget(
                Annotate::new(
                    Paragraph::new("Submit"),
                    Annotation::button("submit").with_label("Submit Button"),
                ),
                area,
            );
        })
        .unwrap();

    assert!(harness.get_by_id("submit").is_some());
    let buttons = harness.find_by_type(&WidgetType::Button);
    assert_eq!(buttons.len(), 1);
}

#[test]
fn test_harness_input() {
    let mut harness = TestHarness::new(80, 24);

    harness.type_str("hello");
    harness.enter();

    assert_eq!(harness.events().len(), 6); // 5 chars + enter
}

#[test]
fn test_harness_click_on() {
    let mut harness = TestHarness::new(80, 24);

    harness
        .render(|frame| {
            let area = ratatui::layout::Rect::new(10, 5, 20, 3);
            frame.render_widget(
                Annotate::new(Paragraph::new("Click Me"), Annotation::button("btn")),
                area,
            );
        })
        .unwrap();

    assert!(harness.click_on("btn"));
    assert_eq!(harness.events().len(), 1);

    assert!(!harness.click_on("nonexistent"));
}

#[test]
fn test_harness_assert_contains() {
    let mut harness = TestHarness::new(80, 24);

    harness
        .render(|frame| {
            frame.render_widget(Paragraph::new("Expected Text"), frame.area());
        })
        .unwrap();

    harness.assert_contains("Expected Text");
    harness.assert_not_contains("Unexpected");
}

#[test]
#[should_panic(expected = "Expected screen to contain")]
fn test_harness_assert_contains_fails() {
    let harness = TestHarness::new(80, 24);
    harness.assert_contains("Not There");
}

#[test]
fn test_harness_region_content() {
    let mut harness = TestHarness::new(80, 24);

    harness
        .render(|frame| {
            frame.render_widget(
                Paragraph::new("ABCDEFGHIJ"),
                ratatui::layout::Rect::new(0, 0, 10, 1),
            );
        })
        .unwrap();

    let content = harness.region_content(2, 0, 4, 1);
    assert_eq!(content, "CDEF");
}

#[test]
fn test_harness_focused() {
    let mut harness = TestHarness::new(80, 24);

    harness
        .render(|frame| {
            let area = frame.area();
            frame.render_widget(
                Annotate::new(
                    Paragraph::new("Input"),
                    Annotation::input("name").with_focus(true),
                ),
                area,
            );
        })
        .unwrap();

    let focused = harness.focused().unwrap();
    assert!(focused.annotation.has_id("name"));
}

#[test]
fn test_harness_default() {
    let harness = TestHarness::default();
    assert_eq!(harness.width(), 80);
    assert_eq!(harness.height(), 24);
}

#[test]
fn test_harness_row() {
    let mut harness = TestHarness::new(20, 5);

    harness
        .render(|frame| {
            frame.render_widget(
                Paragraph::new("Line One"),
                ratatui::layout::Rect::new(0, 0, 20, 1),
            );
        })
        .unwrap();

    let row = harness.row(0);
    assert!(row.contains("Line One"));
}

#[test]
fn test_harness_snapshot() {
    let mut harness = TestHarness::new(20, 5);

    harness
        .render(|frame| {
            frame.render_widget(
                Annotate::new(Paragraph::new("Test"), Annotation::button("btn")),
                frame.area(),
            );
        })
        .unwrap();

    let snapshot = harness.snapshot();
    // Verify we can get content from the snapshot
    let text = snapshot.to_plain();
    assert!(text.contains("Test"));
}

#[test]
fn test_harness_cell_at() {
    let mut harness = TestHarness::new(40, 5);

    harness
        .render(|frame| {
            frame.render_widget(
                Paragraph::new("Hello"),
                ratatui::layout::Rect::new(0, 0, 10, 1),
            );
        })
        .unwrap();

    // Cell at (0,0) should have the 'H' from "Hello"
    let cell = harness.cell_at(0, 0).unwrap();
    assert_eq!(cell.symbol(), "H");

    // Out of bounds should return None
    assert!(harness.cell_at(100, 100).is_none());
}

#[test]
fn test_harness_backend() {
    let harness = TestHarness::new(80, 24);

    let backend = harness.backend();
    assert_eq!(backend.width(), 80);
    assert_eq!(backend.height(), 24);
}

#[test]
fn test_harness_backend_mut() {
    let mut harness = TestHarness::new(80, 24);

    let backend = harness.backend_mut();
    assert_eq!(backend.width(), 80);
}

#[test]
fn test_harness_events_ref() {
    let harness = TestHarness::new(80, 24);
    let events = harness.events();
    assert!(events.is_empty());
}

#[test]
fn test_harness_events_mut() {
    let mut harness = TestHarness::new(80, 24);

    harness.events_mut().push(Event::char('a'));
    assert_eq!(harness.events().len(), 1);
}

#[test]
fn test_harness_push_pop_event() {
    let mut harness = TestHarness::new(80, 24);

    harness.push_event(Event::char('x'));
    harness.push_event(Event::char('y'));

    let event1 = harness.pop_event().unwrap();
    assert!(event1.is_key());

    let event2 = harness.pop_event().unwrap();
    assert!(event2.is_key());

    assert!(harness.pop_event().is_none());
}

#[test]
fn test_harness_escape() {
    let mut harness = TestHarness::new(80, 24);
    harness.escape();
    assert_eq!(harness.events().len(), 1);
}

#[test]
fn test_harness_tab() {
    let mut harness = TestHarness::new(80, 24);
    harness.tab();
    assert_eq!(harness.events().len(), 1);
}

#[test]
fn test_harness_ctrl() {
    let mut harness = TestHarness::new(80, 24);
    harness.ctrl('c');
    assert_eq!(harness.events().len(), 1);
}

#[test]
fn test_harness_clear_events() {
    let mut harness = TestHarness::new(80, 24);

    harness.type_str("abc");
    assert_eq!(harness.events().len(), 3);

    harness.clear_events();
    assert!(harness.events().is_empty());
}

#[test]
fn test_harness_region_at() {
    let mut harness = TestHarness::new(80, 24);

    harness
        .render(|frame| {
            let area = ratatui::layout::Rect::new(10, 10, 20, 5);
            frame.render_widget(
                Annotate::new(Paragraph::new("Button"), Annotation::button("btn")),
                area,
            );
        })
        .unwrap();

    // Inside the widget
    let region = harness.region_at(15, 12);
    assert!(region.is_some());

    // Outside the widget
    let region = harness.region_at(0, 0);
    assert!(region.is_none());
}

#[test]
fn test_harness_find_by_id() {
    let mut harness = TestHarness::new(80, 24);

    harness
        .render(|frame| {
            let area1 = ratatui::layout::Rect::new(0, 0, 20, 1);
            let area2 = ratatui::layout::Rect::new(0, 1, 20, 1);

            frame.render_widget(
                Annotate::new(Paragraph::new("A"), Annotation::button("btn")),
                area1,
            );
            frame.render_widget(
                Annotate::new(Paragraph::new("B"), Annotation::button("btn")),
                area2,
            );
        })
        .unwrap();

    let regions = harness.find_by_id("btn");
    assert_eq!(regions.len(), 2);

    let regions = harness.find_by_id("nonexistent");
    assert!(regions.is_empty());
}

#[test]
fn test_harness_interactive() {
    let mut harness = TestHarness::new(80, 24);

    harness
        .render(|frame| {
            let area1 = ratatui::layout::Rect::new(0, 0, 20, 1);
            let area2 = ratatui::layout::Rect::new(0, 1, 20, 1);

            // Button is interactive
            frame.render_widget(
                Annotate::new(Paragraph::new("A"), Annotation::button("btn")),
                area1,
            );
            // Container is not interactive by default
            frame.render_widget(
                Annotate::new(Paragraph::new("B"), Annotation::container("container")),
                area2,
            );
        })
        .unwrap();

    let interactive = harness.interactive();
    assert_eq!(interactive.len(), 1);
}

#[test]
fn test_harness_find_text() {
    let mut harness = TestHarness::new(40, 5);

    harness
        .render(|frame| {
            frame.render_widget(
                Paragraph::new("Hello World"),
                ratatui::layout::Rect::new(5, 2, 20, 1),
            );
        })
        .unwrap();

    let pos = harness.find_text("Hello");
    assert!(pos.is_some());
    let (x, y) = pos.unwrap();
    assert_eq!(x, 5);
    assert_eq!(y, 2);

    assert!(harness.find_text("Missing").is_none());
}

#[test]
fn test_harness_find_all_text() {
    let mut harness = TestHarness::new(40, 5);

    harness
        .render(|frame| {
            frame.render_widget(
                Paragraph::new("XX XX"),
                ratatui::layout::Rect::new(0, 0, 10, 1),
            );
        })
        .unwrap();

    let positions = harness.find_all_text("XX");
    assert_eq!(positions.len(), 2);
}

#[test]
fn test_harness_assert_widget_exists() {
    let mut harness = TestHarness::new(80, 24);

    harness
        .render(|frame| {
            frame.render_widget(
                Annotate::new(Paragraph::new("Btn"), Annotation::button("btn")),
                frame.area(),
            );
        })
        .unwrap();

    harness.assert_widget_exists("btn");
}

#[test]
#[should_panic(expected = "Expected widget with id")]
fn test_harness_assert_widget_exists_fails() {
    let harness = TestHarness::new(80, 24);
    harness.assert_widget_exists("nonexistent");
}

#[test]
fn test_harness_assert_widget_not_exists() {
    let harness = TestHarness::new(80, 24);
    harness.assert_widget_not_exists("nonexistent");
}

#[test]
#[should_panic(expected = "Expected widget with id")]
fn test_harness_assert_widget_not_exists_fails() {
    let mut harness = TestHarness::new(80, 24);

    harness
        .render(|frame| {
            frame.render_widget(
                Annotate::new(Paragraph::new("Btn"), Annotation::button("btn")),
                frame.area(),
            );
        })
        .unwrap();

    harness.assert_widget_not_exists("btn");
}

#[test]
#[should_panic(expected = "Expected screen to NOT contain")]
fn test_harness_assert_not_contains_fails() {
    let mut harness = TestHarness::new(80, 24);

    harness
        .render(|frame| {
            frame.render_widget(Paragraph::new("Found"), frame.area());
        })
        .unwrap();

    harness.assert_not_contains("Found");
}

#[test]
fn test_harness_assert_focused() {
    let mut harness = TestHarness::new(80, 24);

    harness
        .render(|frame| {
            frame.render_widget(
                Annotate::new(
                    Paragraph::new("Input"),
                    Annotation::input("name").with_focus(true),
                ),
                frame.area(),
            );
        })
        .unwrap();

    harness.assert_focused("name");
}

#[test]
#[should_panic(expected = "Expected widget")]
fn test_harness_assert_focused_fails_not_focused() {
    let mut harness = TestHarness::new(80, 24);

    harness
        .render(|frame| {
            frame.render_widget(
                Annotate::new(
                    Paragraph::new("Input"),
                    Annotation::input("name").with_focus(false),
                ),
                frame.area(),
            );
        })
        .unwrap();

    harness.assert_focused("name");
}

#[test]
#[should_panic(expected = "doesn't exist")]
fn test_harness_assert_focused_fails_not_found() {
    let harness = TestHarness::new(80, 24);
    harness.assert_focused("nonexistent");
}

#[test]
fn test_harness_assert_declarative() {
    let mut harness = TestHarness::new(80, 24);

    harness
        .render(|frame| {
            frame.render_widget(Paragraph::new("Hello"), frame.area());
        })
        .unwrap();

    let result = harness.assert(Assertion::contains("Hello"));
    assert!(result.is_ok());

    let result = harness.assert(Assertion::contains("Missing"));
    assert!(result.is_err());
}

#[test]
fn test_harness_assert_all() {
    let mut harness = TestHarness::new(80, 24);

    harness
        .render(|frame| {
            frame.render_widget(Paragraph::new("Hello World"), frame.area());
        })
        .unwrap();

    let results = harness.assert_all(vec![
        Assertion::contains("Hello"),
        Assertion::contains("World"),
        Assertion::contains("Missing"),
    ]);

    assert_eq!(results.len(), 3);
    assert!(results[0].is_ok());
    assert!(results[1].is_ok());
    assert!(results[2].is_err());
}

#[test]
fn test_harness_assert_all_ok_success() {
    let mut harness = TestHarness::new(80, 24);

    harness
        .render(|frame| {
            frame.render_widget(Paragraph::new("Hello World"), frame.area());
        })
        .unwrap();

    let result = harness.assert_all_ok(vec![
        Assertion::contains("Hello"),
        Assertion::contains("World"),
    ]);

    assert!(result.is_ok());
}

#[test]
fn test_harness_assert_all_ok_failure() {
    let mut harness = TestHarness::new(80, 24);

    harness
        .render(|frame| {
            frame.render_widget(Paragraph::new("Hello"), frame.area());
        })
        .unwrap();

    let result = harness.assert_all_ok(vec![
        Assertion::contains("Hello"),
        Assertion::contains("Missing"),
    ]);

    assert!(result.is_err());
}

#[test]
fn test_harness_region_content_out_of_bounds() {
    let mut harness = TestHarness::new(10, 3);

    harness
        .render(|frame| {
            frame.render_widget(
                Paragraph::new("Hi"),
                ratatui::layout::Rect::new(0, 0, 10, 1),
            );
        })
        .unwrap();

    // Should handle out of bounds gracefully
    let content = harness.region_content(100, 0, 5, 1);
    assert!(content.is_empty());
}

#[test]
fn test_harness_click() {
    let mut harness = TestHarness::new(80, 24);

    harness.click(10, 20);
    assert_eq!(harness.events().len(), 1);
}

#[test]
fn test_harness_screen() {
    let mut harness = TestHarness::new(20, 2);

    harness
        .render(|frame| {
            frame.render_widget(
                Paragraph::new("Test"),
                ratatui::layout::Rect::new(0, 0, 10, 1),
            );
        })
        .unwrap();

    let screen = harness.screen();
    assert!(screen.contains("Test"));
}
