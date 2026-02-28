//! Status bar item types and content.
//!
//! This module contains the types for individual status bar items,
//! their content variants, and visual styles.

use crate::theme::Theme;
use ratatui::prelude::*;

/// Content type for status bar items.
///
/// Items can display static text or dynamic content that updates over time.
#[derive(Clone, Debug, PartialEq)]
pub enum StatusBarItemContent {
    /// Static text content.
    Static(String),
    /// Elapsed time display.
    ///
    /// Shows time elapsed since the timer was started. Format depends on
    /// `long_format`: short format is "MM:SS", long format is "HH:MM:SS".
    ElapsedTime {
        /// Accumulated elapsed time in milliseconds.
        elapsed_ms: u64,
        /// Whether the timer is currently running.
        running: bool,
        /// Whether to use long format (HH:MM:SS vs MM:SS).
        long_format: bool,
    },
    /// Numeric counter display.
    ///
    /// Shows a counter value with an optional label.
    Counter {
        /// Current counter value.
        value: u64,
        /// Optional label (displayed before value).
        label: Option<String>,
    },
    /// Animated heartbeat indicator.
    ///
    /// Shows an animated indicator to show activity.
    Heartbeat {
        /// Whether the heartbeat is active.
        active: bool,
        /// Current animation frame.
        frame: usize,
    },
}

impl StatusBarItemContent {
    /// Creates static text content.
    pub fn static_text(text: impl Into<String>) -> Self {
        Self::Static(text.into())
    }

    /// Creates an elapsed time display.
    pub fn elapsed_time() -> Self {
        Self::ElapsedTime {
            elapsed_ms: 0,
            running: false,
            long_format: false,
        }
    }

    /// Creates a counter display.
    pub fn counter() -> Self {
        Self::Counter {
            value: 0,
            label: None,
        }
    }

    /// Creates a heartbeat indicator.
    pub fn heartbeat() -> Self {
        Self::Heartbeat {
            active: false,
            frame: 0,
        }
    }

    /// Returns the display text for this content.
    pub(super) fn display_text(&self) -> String {
        match self {
            Self::Static(text) => text.clone(),
            Self::ElapsedTime {
                elapsed_ms,
                long_format,
                ..
            } => {
                let total_seconds = elapsed_ms / 1000;
                let hours = total_seconds / 3600;
                let minutes = (total_seconds % 3600) / 60;
                let seconds = total_seconds % 60;

                if *long_format || hours > 0 {
                    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
                } else {
                    format!("{:02}:{:02}", minutes, seconds)
                }
            }
            Self::Counter { value, label } => {
                if let Some(label) = label {
                    format!("{}: {}", label, value)
                } else {
                    value.to_string()
                }
            }
            Self::Heartbeat { active, frame } => {
                const FRAMES: [&str; 4] = ["♡", "♥", "♥", "♡"];
                if *active {
                    FRAMES[*frame % FRAMES.len()].to_string()
                } else {
                    "♡".to_string()
                }
            }
        }
    }

    /// Returns true if this is dynamic content that needs ticking.
    pub(super) fn is_dynamic(&self) -> bool {
        !matches!(self, Self::Static(_))
    }
}

/// Style variants for status bar items.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StatusBarStyle {
    /// Default style (no special coloring).
    #[default]
    Default,
    /// Informational style (typically blue).
    Info,
    /// Success style (typically green).
    Success,
    /// Warning style (typically yellow).
    Warning,
    /// Error style (typically red).
    Error,
    /// Muted/secondary style (typically gray).
    Muted,
}

impl StatusBarStyle {
    /// Returns the ratatui style for this status bar style variant.
    pub(super) fn style(self, theme: &Theme) -> Style {
        match self {
            Self::Default => theme.normal_style(),
            Self::Info => theme.info_style(),
            Self::Success => theme.success_style(),
            Self::Warning => theme.warning_style(),
            Self::Error => theme.error_style(),
            Self::Muted => theme.disabled_style(),
        }
    }
}

/// A single item in the status bar.
#[derive(Clone, Debug, PartialEq)]
pub struct StatusBarItem {
    /// The content of the item.
    pub(super) content: StatusBarItemContent,
    /// The style of the item.
    pub(super) style: StatusBarStyle,
    /// Whether to show a separator after this item.
    separator: bool,
}

impl StatusBarItem {
    /// Creates a new status bar item with static text content.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusBarItem;
    ///
    /// let item = StatusBarItem::new("Ready");
    /// assert_eq!(item.text(), "Ready");
    /// ```
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            content: StatusBarItemContent::Static(text.into()),
            style: StatusBarStyle::Default,
            separator: true,
        }
    }

    /// Creates a new status bar item with an elapsed time display.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusBarItem;
    ///
    /// let item = StatusBarItem::elapsed_time();
    /// assert_eq!(item.text(), "00:00");
    /// ```
    pub fn elapsed_time() -> Self {
        Self {
            content: StatusBarItemContent::elapsed_time(),
            style: StatusBarStyle::Default,
            separator: true,
        }
    }

    /// Creates an elapsed time display with long format (HH:MM:SS).
    pub fn elapsed_time_long() -> Self {
        Self {
            content: StatusBarItemContent::ElapsedTime {
                elapsed_ms: 0,
                running: false,
                long_format: true,
            },
            style: StatusBarStyle::Default,
            separator: true,
        }
    }

    /// Creates a new status bar item with a counter display.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusBarItem;
    ///
    /// let item = StatusBarItem::counter().with_label("Items");
    /// ```
    pub fn counter() -> Self {
        Self {
            content: StatusBarItemContent::counter(),
            style: StatusBarStyle::Default,
            separator: true,
        }
    }

    /// Creates a new status bar item with a heartbeat indicator.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusBarItem;
    ///
    /// let item = StatusBarItem::heartbeat();
    /// ```
    pub fn heartbeat() -> Self {
        Self {
            content: StatusBarItemContent::heartbeat(),
            style: StatusBarStyle::Default,
            separator: true,
        }
    }

    /// Sets the label for counter items.
    ///
    /// This only has an effect on Counter content types.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        if let StatusBarItemContent::Counter {
            value,
            label: ref mut lbl,
        } = self.content
        {
            *lbl = Some(label.into());
            self.content = StatusBarItemContent::Counter {
                value,
                label: lbl.clone(),
            };
        }
        self
    }

    /// Sets long format for elapsed time items.
    ///
    /// This only has an effect on ElapsedTime content types.
    pub fn with_long_format(mut self, long: bool) -> Self {
        if let StatusBarItemContent::ElapsedTime {
            elapsed_ms,
            running,
            ..
        } = self.content
        {
            self.content = StatusBarItemContent::ElapsedTime {
                elapsed_ms,
                running,
                long_format: long,
            };
        }
        self
    }

    /// Sets the style for this item.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{StatusBarItem, StatusBarStyle};
    ///
    /// let item = StatusBarItem::new("Error").with_style(StatusBarStyle::Error);
    /// assert_eq!(item.style(), StatusBarStyle::Error);
    /// ```
    pub fn with_style(mut self, style: StatusBarStyle) -> Self {
        self.style = style;
        self
    }

    /// Sets whether to show a separator after this item.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusBarItem;
    ///
    /// let item = StatusBarItem::new("Last").with_separator(false);
    /// assert!(!item.has_separator());
    /// ```
    pub fn with_separator(mut self, separator: bool) -> Self {
        self.separator = separator;
        self
    }

    /// Returns the display text for this item.
    pub fn text(&self) -> String {
        self.content.display_text()
    }

    /// Returns the content.
    pub fn content(&self) -> &StatusBarItemContent {
        &self.content
    }

    /// Returns a mutable reference to the content.
    pub fn content_mut(&mut self) -> &mut StatusBarItemContent {
        &mut self.content
    }

    /// Sets the text content (converts to static content).
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.content = StatusBarItemContent::Static(text.into());
    }

    /// Returns the style.
    pub fn style(&self) -> StatusBarStyle {
        self.style
    }

    /// Sets the style.
    pub fn set_style(&mut self, style: StatusBarStyle) {
        self.style = style;
    }

    /// Returns whether this item has a separator.
    pub fn has_separator(&self) -> bool {
        self.separator
    }

    /// Sets whether to show a separator.
    pub fn set_separator(&mut self, separator: bool) {
        self.separator = separator;
    }

    /// Returns true if this item has dynamic content.
    pub fn is_dynamic(&self) -> bool {
        self.content.is_dynamic()
    }

    /// Processes a tick for dynamic content.
    ///
    /// Returns true if the content was updated.
    pub(super) fn tick(&mut self, delta_ms: u64) -> bool {
        match &mut self.content {
            StatusBarItemContent::ElapsedTime {
                elapsed_ms,
                running,
                ..
            } => {
                if *running {
                    *elapsed_ms += delta_ms;
                    true
                } else {
                    false
                }
            }
            StatusBarItemContent::Heartbeat { active, frame } => {
                if *active {
                    *frame = (*frame + 1) % 4;
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}
