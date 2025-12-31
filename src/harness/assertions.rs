//! Declarative assertions for TUI testing.

use std::fmt;

use crate::annotation::WidgetType;

use super::TestHarness;

/// Result type for assertions.
pub type AssertionResult = Result<(), AssertionError>;

/// Error returned when an assertion fails.
#[derive(Debug, Clone)]
pub struct AssertionError {
    /// Description of the assertion
    pub assertion: String,

    /// What was expected
    pub expected: String,

    /// What was actually found
    pub actual: String,

    /// Additional context
    pub context: Option<String>,
}

impl AssertionError {
    /// Creates a new assertion error.
    pub fn new(
        assertion: impl Into<String>,
        expected: impl Into<String>,
        actual: impl Into<String>,
    ) -> Self {
        Self {
            assertion: assertion.into(),
            expected: expected.into(),
            actual: actual.into(),
            context: None,
        }
    }

    /// Adds context to the error.
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }
}

impl fmt::Display for AssertionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Assertion failed: {}", self.assertion)?;
        writeln!(f, "  Expected: {}", self.expected)?;
        writeln!(f, "  Actual:   {}", self.actual)?;
        if let Some(ctx) = &self.context {
            writeln!(f, "\nContext:\n{}", ctx)?;
        }
        Ok(())
    }
}

impl std::error::Error for AssertionError {}

/// A declarative assertion that can be checked against a test harness.
#[derive(Debug, Clone)]
pub enum Assertion {
    /// Assert that the screen contains the given text.
    Contains(String),

    /// Assert that the screen does not contain the given text.
    NotContains(String),

    /// Assert that text appears at a specific position.
    TextAt {
        /// The text to find.
        text: String,
        /// The x coordinate.
        x: u16,
        /// The y coordinate.
        y: u16,
    },

    /// Assert that a widget with the given id exists.
    WidgetExists(String),

    /// Assert that a widget with the given id does not exist.
    WidgetNotExists(String),

    /// Assert that a widget with the given id is focused.
    WidgetFocused(String),

    /// Assert that a widget with the given id is disabled.
    WidgetDisabled(String),

    /// Assert that a widget with the given id has a specific value.
    WidgetValue {
        /// The widget id.
        id: String,
        /// The expected value.
        value: String,
    },

    /// Assert that there are exactly N widgets of a given type.
    WidgetCount {
        /// The type of widget to count.
        widget_type: WidgetType,
        /// The expected count.
        count: usize,
    },

    /// Assert that the screen matches exactly.
    ScreenEquals(String),

    /// Assert that a specific row matches.
    RowEquals {
        /// The row number.
        row: u16,
        /// The expected content.
        content: String,
    },

    /// Assert that a row contains the given text.
    RowContains {
        /// The row number.
        row: u16,
        /// The text to find.
        text: String,
    },

    /// Logical AND of multiple assertions.
    All(Vec<Assertion>),

    /// Logical OR of multiple assertions.
    Any(Vec<Assertion>),

    /// Negation of an assertion.
    Not(Box<Assertion>),
}

impl Assertion {
    /// Creates a Contains assertion.
    pub fn contains(text: impl Into<String>) -> Self {
        Self::Contains(text.into())
    }

    /// Creates a NotContains assertion.
    pub fn not_contains(text: impl Into<String>) -> Self {
        Self::NotContains(text.into())
    }

    /// Creates a TextAt assertion.
    pub fn text_at(text: impl Into<String>, x: u16, y: u16) -> Self {
        Self::TextAt {
            text: text.into(),
            x,
            y,
        }
    }

    /// Creates a WidgetExists assertion.
    pub fn widget_exists(id: impl Into<String>) -> Self {
        Self::WidgetExists(id.into())
    }

    /// Creates a WidgetNotExists assertion.
    pub fn widget_not_exists(id: impl Into<String>) -> Self {
        Self::WidgetNotExists(id.into())
    }

    /// Creates a WidgetFocused assertion.
    pub fn widget_focused(id: impl Into<String>) -> Self {
        Self::WidgetFocused(id.into())
    }

    /// Creates a WidgetDisabled assertion.
    pub fn widget_disabled(id: impl Into<String>) -> Self {
        Self::WidgetDisabled(id.into())
    }

    /// Creates a WidgetValue assertion.
    pub fn widget_value(id: impl Into<String>, value: impl Into<String>) -> Self {
        Self::WidgetValue {
            id: id.into(),
            value: value.into(),
        }
    }

    /// Creates a WidgetCount assertion.
    pub fn widget_count(widget_type: WidgetType, count: usize) -> Self {
        Self::WidgetCount { widget_type, count }
    }

    /// Creates a ScreenEquals assertion.
    pub fn screen_equals(content: impl Into<String>) -> Self {
        Self::ScreenEquals(content.into())
    }

    /// Creates a RowEquals assertion.
    pub fn row_equals(row: u16, content: impl Into<String>) -> Self {
        Self::RowEquals {
            row,
            content: content.into(),
        }
    }

    /// Creates a RowContains assertion.
    pub fn row_contains(row: u16, text: impl Into<String>) -> Self {
        Self::RowContains {
            row,
            text: text.into(),
        }
    }

    /// Creates an All assertion (all must pass).
    pub fn all(assertions: Vec<Assertion>) -> Self {
        Self::All(assertions)
    }

    /// Creates an Any assertion (at least one must pass).
    pub fn any(assertions: Vec<Assertion>) -> Self {
        Self::Any(assertions)
    }

    /// Checks the assertion against the test harness.
    pub fn check(&self, harness: &TestHarness) -> AssertionResult {
        match self {
            Assertion::Contains(text) => {
                if harness.contains(text) {
                    Ok(())
                } else {
                    Err(AssertionError::new(
                        "Contains",
                        format!("screen to contain '{}'", text),
                        "text not found",
                    )
                    .with_context(harness.screen()))
                }
            }

            Assertion::NotContains(text) => {
                if !harness.contains(text) {
                    Ok(())
                } else {
                    Err(AssertionError::new(
                        "NotContains",
                        format!("screen to NOT contain '{}'", text),
                        "text was found",
                    )
                    .with_context(harness.screen()))
                }
            }

            Assertion::TextAt { text, x, y } => {
                let row_content = harness.row(*y);
                let start = *x as usize;
                let end = start + text.len();

                if end <= row_content.len() {
                    let actual = &row_content[start..end];
                    if actual == text {
                        return Ok(());
                    }
                    Err(AssertionError::new(
                        "TextAt",
                        format!("'{}' at ({}, {})", text, x, y),
                        format!("found '{}'", actual),
                    ))
                } else {
                    Err(AssertionError::new(
                        "TextAt",
                        format!("'{}' at ({}, {})", text, x, y),
                        "position out of bounds",
                    ))
                }
            }

            Assertion::WidgetExists(id) => {
                if harness.get_by_id(id).is_some() {
                    Ok(())
                } else {
                    Err(AssertionError::new(
                        "WidgetExists",
                        format!("widget '{}' to exist", id),
                        "widget not found",
                    )
                    .with_context(harness.annotations().format_tree()))
                }
            }

            Assertion::WidgetNotExists(id) => {
                if harness.get_by_id(id).is_none() {
                    Ok(())
                } else {
                    Err(AssertionError::new(
                        "WidgetNotExists",
                        format!("widget '{}' to NOT exist", id),
                        "widget was found",
                    )
                    .with_context(harness.annotations().format_tree()))
                }
            }

            Assertion::WidgetFocused(id) => match harness.get_by_id(id) {
                Some(region) if region.annotation.focused => Ok(()),
                Some(_) => Err(AssertionError::new(
                    "WidgetFocused",
                    format!("widget '{}' to be focused", id),
                    "widget is not focused",
                )),
                None => Err(AssertionError::new(
                    "WidgetFocused",
                    format!("widget '{}' to be focused", id),
                    "widget not found",
                )),
            },

            Assertion::WidgetDisabled(id) => match harness.get_by_id(id) {
                Some(region) if region.annotation.disabled => Ok(()),
                Some(_) => Err(AssertionError::new(
                    "WidgetDisabled",
                    format!("widget '{}' to be disabled", id),
                    "widget is not disabled",
                )),
                None => Err(AssertionError::new(
                    "WidgetDisabled",
                    format!("widget '{}' to be disabled", id),
                    "widget not found",
                )),
            },

            Assertion::WidgetValue { id, value } => match harness.get_by_id(id) {
                Some(region) => {
                    if region.annotation.value.as_deref() == Some(value.as_str()) {
                        Ok(())
                    } else {
                        Err(AssertionError::new(
                            "WidgetValue",
                            format!("widget '{}' to have value '{}'", id, value),
                            format!("value is {:?}", region.annotation.value),
                        ))
                    }
                }
                None => Err(AssertionError::new(
                    "WidgetValue",
                    format!("widget '{}' to have value '{}'", id, value),
                    "widget not found",
                )),
            },

            Assertion::WidgetCount { widget_type, count } => {
                let actual = harness.find_by_type(widget_type).len();
                if actual == *count {
                    Ok(())
                } else {
                    Err(AssertionError::new(
                        "WidgetCount",
                        format!("{} {:?} widgets", count, widget_type),
                        format!("found {}", actual),
                    ))
                }
            }

            Assertion::ScreenEquals(expected) => {
                let actual = harness.screen();
                if actual == *expected {
                    Ok(())
                } else {
                    Err(AssertionError::new(
                        "ScreenEquals",
                        "screen to match exactly",
                        "content differs",
                    )
                    .with_context(format!("Expected:\n{}\n\nActual:\n{}", expected, actual)))
                }
            }

            Assertion::RowEquals { row, content } => {
                let actual = harness.row(*row);
                if actual == *content {
                    Ok(())
                } else {
                    Err(AssertionError::new(
                        "RowEquals",
                        format!("row {} to equal '{}'", row, content),
                        format!("found '{}'", actual),
                    ))
                }
            }

            Assertion::RowContains { row, text } => {
                let actual = harness.row(*row);
                if actual.contains(text) {
                    Ok(())
                } else {
                    Err(AssertionError::new(
                        "RowContains",
                        format!("row {} to contain '{}'", row, text),
                        format!("row content: '{}'", actual),
                    ))
                }
            }

            Assertion::All(assertions) => {
                for assertion in assertions {
                    assertion.check(harness)?;
                }
                Ok(())
            }

            Assertion::Any(assertions) => {
                for assertion in assertions {
                    if assertion.check(harness).is_ok() {
                        return Ok(());
                    }
                }
                Err(AssertionError::new(
                    "Any",
                    "at least one assertion to pass",
                    "all assertions failed",
                ))
            }

            Assertion::Not(inner) => match inner.check(harness) {
                Ok(()) => Err(AssertionError::new(
                    "Not",
                    "assertion to fail",
                    "assertion passed",
                )),
                Err(_) => Ok(()),
            },
        }
    }
}

impl std::ops::Not for Assertion {
    type Output = Self;

    /// Negates the assertion.
    fn not(self) -> Self::Output {
        Self::Not(Box::new(self))
    }
}

#[cfg(test)]
mod tests {
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
}
