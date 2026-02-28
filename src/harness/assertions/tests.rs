use std::ops::Not;

use super::*;
use crate::annotation::{Annotate, Annotation};
use ratatui::widgets::Paragraph;

fn harness_with_content(content: &str) -> TestHarness {
    let mut harness = TestHarness::new(80, 24);
    let content = content.to_string();
    harness
        .render(|frame| {
            frame.render_widget(Paragraph::new(content.clone()), frame.area());
        })
        .unwrap();
    harness
}

#[test]
fn test_assertion_contains() {
    let harness = harness_with_content("Hello World");

    assert!(Assertion::contains("Hello").check(&harness).is_ok());
    assert!(Assertion::contains("Missing").check(&harness).is_err());
}

#[test]
fn test_assertion_not_contains() {
    let harness = harness_with_content("Hello World");

    assert!(Assertion::not_contains("Missing").check(&harness).is_ok());
    assert!(Assertion::not_contains("Hello").check(&harness).is_err());
}

#[test]
fn test_assertion_text_at() {
    let harness = harness_with_content("Hello World");

    assert!(Assertion::text_at("Hello", 0, 0).check(&harness).is_ok());
    assert!(Assertion::text_at("World", 6, 0).check(&harness).is_ok());
    assert!(Assertion::text_at("Wrong", 0, 0).check(&harness).is_err());
}

#[test]
fn test_assertion_widget_exists() {
    let mut harness = TestHarness::new(80, 24);
    harness
        .render(|frame| {
            frame.render_widget(
                Annotate::new(Paragraph::new("Button"), Annotation::button("submit")),
                frame.area(),
            );
        })
        .unwrap();

    assert!(Assertion::widget_exists("submit").check(&harness).is_ok());
    assert!(Assertion::widget_exists("cancel").check(&harness).is_err());
}

#[test]
fn test_assertion_widget_focused() {
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

    assert!(Assertion::widget_focused("name").check(&harness).is_ok());
}

#[test]
fn test_assertion_widget_count() {
    let mut harness = TestHarness::new(80, 24);
    harness
        .render(|frame| {
            let area1 = ratatui::layout::Rect::new(0, 0, 10, 1);
            let area2 = ratatui::layout::Rect::new(0, 1, 10, 1);

            frame.render_widget(
                Annotate::new(Paragraph::new("A"), Annotation::button("a")),
                area1,
            );
            frame.render_widget(
                Annotate::new(Paragraph::new("B"), Annotation::button("b")),
                area2,
            );
        })
        .unwrap();

    assert!(Assertion::widget_count(WidgetType::Button, 2)
        .check(&harness)
        .is_ok());
    assert!(Assertion::widget_count(WidgetType::Button, 1)
        .check(&harness)
        .is_err());
}

#[test]
fn test_assertion_all() {
    let harness = harness_with_content("Hello World");

    let all = Assertion::all(vec![
        Assertion::contains("Hello"),
        Assertion::contains("World"),
    ]);
    assert!(all.check(&harness).is_ok());

    let all_fail = Assertion::all(vec![
        Assertion::contains("Hello"),
        Assertion::contains("Missing"),
    ]);
    assert!(all_fail.check(&harness).is_err());
}

#[test]
fn test_assertion_any() {
    let harness = harness_with_content("Hello World");

    let any = Assertion::any(vec![
        Assertion::contains("Missing"),
        Assertion::contains("Hello"),
    ]);
    assert!(any.check(&harness).is_ok());

    let any_fail = Assertion::any(vec![
        Assertion::contains("Missing1"),
        Assertion::contains("Missing2"),
    ]);
    assert!(any_fail.check(&harness).is_err());
}

#[test]
fn test_assertion_not() {
    let harness = harness_with_content("Hello World");

    assert!(Assertion::contains("Missing").not().check(&harness).is_ok());
    assert!(Assertion::contains("Hello").not().check(&harness).is_err());
}

#[test]
fn test_assertion_row_contains() {
    let harness = harness_with_content("Line One");

    assert!(Assertion::row_contains(0, "Line").check(&harness).is_ok());
    assert!(Assertion::row_contains(0, "Two").check(&harness).is_err());
}

#[test]
fn test_assertion_error_display() {
    let error = AssertionError::new("Test", "expected", "actual").with_context("Some context");

    let display = format!("{}", error);
    assert!(display.contains("Test"));
    assert!(display.contains("expected"));
    assert!(display.contains("actual"));
    assert!(display.contains("Some context"));
}

#[test]
fn test_assertion_error_display_without_context() {
    let error = AssertionError::new("Test", "expected", "actual");

    let display = format!("{}", error);
    assert!(display.contains("Test"));
    assert!(display.contains("expected"));
    assert!(display.contains("actual"));
    assert!(!display.contains("Context:"));
}

#[test]
fn test_assertion_widget_not_exists() {
    let mut harness = TestHarness::new(80, 24);
    harness
        .render(|frame| {
            frame.render_widget(
                Annotate::new(Paragraph::new("Button"), Annotation::button("submit")),
                frame.area(),
            );
        })
        .unwrap();

    // Should pass when widget doesn't exist
    assert!(Assertion::widget_not_exists("cancel")
        .check(&harness)
        .is_ok());
    // Should fail when widget exists
    assert!(Assertion::widget_not_exists("submit")
        .check(&harness)
        .is_err());
}

#[test]
fn test_assertion_widget_disabled() {
    let mut harness = TestHarness::new(80, 24);
    harness
        .render(|frame| {
            let area1 = ratatui::layout::Rect::new(0, 0, 20, 1);
            let area2 = ratatui::layout::Rect::new(0, 1, 20, 1);

            // Disabled button
            frame.render_widget(
                Annotate::new(
                    Paragraph::new("Disabled"),
                    Annotation::button("disabled_btn").with_disabled(true),
                ),
                area1,
            );
            // Enabled button
            frame.render_widget(
                Annotate::new(Paragraph::new("Enabled"), Annotation::button("enabled_btn")),
                area2,
            );
        })
        .unwrap();

    // Should pass when widget is disabled
    assert!(Assertion::widget_disabled("disabled_btn")
        .check(&harness)
        .is_ok());
    // Should fail when widget is not disabled
    assert!(Assertion::widget_disabled("enabled_btn")
        .check(&harness)
        .is_err());
    // Should fail when widget doesn't exist
    assert!(Assertion::widget_disabled("nonexistent")
        .check(&harness)
        .is_err());
}

#[test]
fn test_assertion_widget_value() {
    let mut harness = TestHarness::new(80, 24);
    harness
        .render(|frame| {
            let area1 = ratatui::layout::Rect::new(0, 0, 20, 1);
            let area2 = ratatui::layout::Rect::new(0, 1, 20, 1);

            // Input with value
            frame.render_widget(
                Annotate::new(
                    Paragraph::new("John"),
                    Annotation::input("name").with_value("John"),
                ),
                area1,
            );
            // Input without value
            frame.render_widget(
                Annotate::new(Paragraph::new("Empty"), Annotation::input("empty")),
                area2,
            );
        })
        .unwrap();

    // Should pass when widget has the expected value
    assert!(Assertion::widget_value("name", "John")
        .check(&harness)
        .is_ok());
    // Should fail when widget has different value
    assert!(Assertion::widget_value("name", "Jane")
        .check(&harness)
        .is_err());
    // Should fail when widget has no value
    assert!(Assertion::widget_value("empty", "anything")
        .check(&harness)
        .is_err());
    // Should fail when widget doesn't exist
    assert!(Assertion::widget_value("nonexistent", "test")
        .check(&harness)
        .is_err());
}

#[test]
fn test_assertion_screen_equals() {
    let mut harness = TestHarness::new(10, 2);
    harness
        .render(|frame| {
            let area = ratatui::layout::Rect::new(0, 0, 10, 1);
            frame.render_widget(Paragraph::new("Hello"), area);
        })
        .unwrap();

    let screen = harness.screen();

    // Should pass when screen matches exactly
    assert!(Assertion::screen_equals(&screen).check(&harness).is_ok());
    // Should fail when screen doesn't match
    assert!(Assertion::screen_equals("Different content")
        .check(&harness)
        .is_err());
}

#[test]
fn test_assertion_row_equals() {
    let harness = harness_with_content("Hello World");

    // Should pass when row matches exactly
    let row = harness.row(0);
    assert!(Assertion::row_equals(0, row).check(&harness).is_ok());
    // Should fail when row doesn't match
    assert!(Assertion::row_equals(0, "Different")
        .check(&harness)
        .is_err());
}

#[test]
fn test_assertion_text_at_out_of_bounds() {
    let harness = harness_with_content("Hi");

    // Should fail when position is out of bounds
    assert!(Assertion::text_at("Hello", 100, 0).check(&harness).is_err());

    // Check error message
    let err = Assertion::text_at("Hello", 100, 0)
        .check(&harness)
        .unwrap_err();
    assert!(err.actual.contains("out of bounds"));
}

#[test]
fn test_assertion_widget_focused_not_focused() {
    let mut harness = TestHarness::new(80, 24);
    harness
        .render(|frame| {
            frame.render_widget(
                Annotate::new(
                    Paragraph::new("Input"),
                    Annotation::input("unfocused").with_focus(false),
                ),
                frame.area(),
            );
        })
        .unwrap();

    // Should fail when widget is not focused
    let result = Assertion::widget_focused("unfocused").check(&harness);
    assert!(result.is_err());
    assert!(result.unwrap_err().actual.contains("not focused"));
}

#[test]
fn test_assertion_widget_focused_not_found() {
    let harness = TestHarness::new(80, 24);

    // Should fail when widget doesn't exist
    let result = Assertion::widget_focused("nonexistent").check(&harness);
    assert!(result.is_err());
    assert!(result.unwrap_err().actual.contains("not found"));
}

#[test]
fn test_assertion_contains_error_has_context() {
    let harness = harness_with_content("Hello");

    let err = Assertion::contains("Missing").check(&harness).unwrap_err();
    assert!(err.context.is_some());
}

#[test]
fn test_assertion_not_contains_error_has_context() {
    let harness = harness_with_content("Hello");

    let err = Assertion::not_contains("Hello")
        .check(&harness)
        .unwrap_err();
    assert!(err.context.is_some());
}

#[test]
fn test_assertion_widget_exists_error_has_context() {
    let harness = TestHarness::new(80, 24);

    let err = Assertion::widget_exists("missing")
        .check(&harness)
        .unwrap_err();
    assert!(err.context.is_some());
}

#[test]
fn test_assertion_widget_not_exists_error_has_context() {
    let mut harness = TestHarness::new(80, 24);
    harness
        .render(|frame| {
            frame.render_widget(
                Annotate::new(Paragraph::new("Btn"), Annotation::button("btn")),
                frame.area(),
            );
        })
        .unwrap();

    let err = Assertion::widget_not_exists("btn")
        .check(&harness)
        .unwrap_err();
    assert!(err.context.is_some());
}

#[test]
fn test_assertion_screen_equals_error_has_context() {
    let harness = harness_with_content("Hello");

    let err = Assertion::screen_equals("Different")
        .check(&harness)
        .unwrap_err();
    assert!(err.context.is_some());
    assert!(err.context.unwrap().contains("Expected:"));
}

#[test]
fn test_assertion_error_is_std_error() {
    let error = AssertionError::new("Test", "expected", "actual");
    // Verify it implements std::error::Error
    let _: &dyn std::error::Error = &error;
}

#[test]
fn test_assertion_debug_impl() {
    let assertion = Assertion::contains("test");
    let debug = format!("{:?}", assertion);
    assert!(debug.contains("Contains"));
    assert!(debug.contains("test"));
}

#[test]
fn test_assertion_clone() {
    let assertion = Assertion::contains("test");
    let cloned = assertion.clone();
    let harness = harness_with_content("test");

    // Both should work the same
    assert!(assertion.check(&harness).is_ok());
    assert!(cloned.check(&harness).is_ok());
}

#[test]
fn test_assertion_error_clone() {
    let error = AssertionError::new("Test", "expected", "actual").with_context("ctx");
    let cloned = error.clone();

    assert_eq!(error.assertion, cloned.assertion);
    assert_eq!(error.expected, cloned.expected);
    assert_eq!(error.actual, cloned.actual);
    assert_eq!(error.context, cloned.context);
}
