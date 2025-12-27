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
        text: String,
        x: u16,
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
        id: String,
        value: String,
    },

    /// Assert that there are exactly N widgets of a given type.
    WidgetCount {
        widget_type: WidgetType,
        count: usize,
    },

    /// Assert that the screen matches exactly.
    ScreenEquals(String),

    /// Assert that a specific row matches.
    RowEquals {
        row: u16,
        content: String,
    },

    /// Assert that a row contains the given text.
    RowContains {
        row: u16,
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
        Self::TextAt { text: text.into(), x, y }
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
        Self::WidgetValue { id: id.into(), value: value.into() }
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
        Self::RowEquals { row, content: content.into() }
    }

    /// Creates a RowContains assertion.
    pub fn row_contains(row: u16, text: impl Into<String>) -> Self {
        Self::RowContains { row, text: text.into() }
    }

    /// Creates an All assertion (all must pass).
    pub fn all(assertions: Vec<Assertion>) -> Self {
        Self::All(assertions)
    }

    /// Creates an Any assertion (at least one must pass).
    pub fn any(assertions: Vec<Assertion>) -> Self {
        Self::Any(assertions)
    }

    /// Negates the assertion.
    pub fn not(self) -> Self {
        Self::Not(Box::new(self))
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
                    ).with_context(harness.screen()))
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
                    ).with_context(harness.screen()))
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
                    ).with_context(harness.annotations().format_tree()))
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
                    ).with_context(harness.annotations().format_tree()))
                }
            }

            Assertion::WidgetFocused(id) => {
                match harness.get_by_id(id) {
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
                }
            }

            Assertion::WidgetDisabled(id) => {
                match harness.get_by_id(id) {
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
                }
            }

            Assertion::WidgetValue { id, value } => {
                match harness.get_by_id(id) {
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
                }
            }

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
                    ).with_context(format!(
                        "Expected:\n{}\n\nActual:\n{}",
                        expected, actual
                    )))
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

            Assertion::Not(inner) => {
                match inner.check(harness) {
                    Ok(()) => Err(AssertionError::new(
                        "Not",
                        "assertion to fail",
                        "assertion passed",
                    )),
                    Err(_) => Ok(()),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::annotation::{Annotate, Annotation};
    use ratatui::widgets::Paragraph;

    fn harness_with_content(content: &str) -> TestHarness {
        let mut harness = TestHarness::new(80, 24);
        let content = content.to_string();
        harness.render(|frame| {
            frame.render_widget(
                Paragraph::new(content.clone()),
                frame.area(),
            );
        }).unwrap();
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
        harness.render(|frame| {
            frame.render_widget(
                Annotate::new(
                    Paragraph::new("Button"),
                    Annotation::button("submit"),
                ),
                frame.area(),
            );
        }).unwrap();

        assert!(Assertion::widget_exists("submit").check(&harness).is_ok());
        assert!(Assertion::widget_exists("cancel").check(&harness).is_err());
    }

    #[test]
    fn test_assertion_widget_focused() {
        let mut harness = TestHarness::new(80, 24);
        harness.render(|frame| {
            frame.render_widget(
                Annotate::new(
                    Paragraph::new("Input"),
                    Annotation::input("name").with_focus(true),
                ),
                frame.area(),
            );
        }).unwrap();

        assert!(Assertion::widget_focused("name").check(&harness).is_ok());
    }

    #[test]
    fn test_assertion_widget_count() {
        let mut harness = TestHarness::new(80, 24);
        harness.render(|frame| {
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
        }).unwrap();

        assert!(Assertion::widget_count(WidgetType::Button, 2).check(&harness).is_ok());
        assert!(Assertion::widget_count(WidgetType::Button, 1).check(&harness).is_err());
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
        let error = AssertionError::new("Test", "expected", "actual")
            .with_context("Some context");

        let display = format!("{}", error);
        assert!(display.contains("Test"));
        assert!(display.contains("expected"));
        assert!(display.contains("actual"));
        assert!(display.contains("Some context"));
    }
}
