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
mod tests;
