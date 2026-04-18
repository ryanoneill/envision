use ratatui::prelude::*;

use crate::theme::Theme;

/// Status of a progress item.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum ProgressItemStatus {
    /// Item is pending (not yet started).
    #[default]
    Pending,
    /// Item is actively being processed.
    Active,
    /// Item has completed successfully.
    Completed,
    /// Item has failed.
    Failed,
}

impl ProgressItemStatus {
    /// Returns the style for this status using the theme.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ProgressItemStatus;
    /// use envision::Theme;
    ///
    /// let theme = Theme::default();
    /// let pending_style = ProgressItemStatus::Pending.style(&theme);
    /// let active_style = ProgressItemStatus::Active.style(&theme);
    /// // Different statuses produce different styles
    /// assert_ne!(pending_style, active_style);
    /// ```
    pub fn style(&self, theme: &Theme) -> Style {
        match self {
            Self::Pending => theme.disabled_style(),
            Self::Active => theme.info_style(),
            Self::Completed => theme.success_style(),
            Self::Failed => theme.error_style(),
        }
    }

    /// Returns the symbol for this status.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ProgressItemStatus;
    ///
    /// assert_eq!(ProgressItemStatus::Pending.symbol(), "○");
    /// assert_eq!(ProgressItemStatus::Active.symbol(), "●");
    /// assert_eq!(ProgressItemStatus::Completed.symbol(), "✓");
    /// assert_eq!(ProgressItemStatus::Failed.symbol(), "✗");
    /// ```
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Pending => "○",
            Self::Active => "●",
            Self::Completed => "✓",
            Self::Failed => "✗",
        }
    }
}

/// A single progress item.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct ProgressItem {
    /// Unique identifier.
    pub(crate) id: String,
    /// Display label.
    pub(crate) label: String,
    /// Progress from 0.0 to 1.0.
    pub(crate) progress: f32,
    /// Current status.
    pub(crate) status: ProgressItemStatus,
    /// Optional status message.
    pub(crate) message: Option<String>,
}

impl ProgressItem {
    /// Creates a new progress item.
    ///
    /// # Example
    ///
    /// ```
    /// use envision::component::ProgressItem;
    ///
    /// let item = ProgressItem::new("dl-1", "Downloading");
    /// assert_eq!(item.id(), "dl-1");
    /// assert_eq!(item.label(), "Downloading");
    /// assert_eq!(item.progress(), 0.0);
    /// assert_eq!(item.percentage(), 0);
    /// ```
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            progress: 0.0,
            status: ProgressItemStatus::Pending,
            message: None,
        }
    }

    /// Returns the item ID.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ProgressItem;
    ///
    /// let item = ProgressItem::new("task-1", "Download");
    /// assert_eq!(item.id(), "task-1");
    /// ```
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the label.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ProgressItem;
    ///
    /// let item = ProgressItem::new("t1", "Uploading");
    /// assert_eq!(item.label(), "Uploading");
    /// ```
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns the progress (0.0 to 1.0).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ProgressItem;
    ///
    /// let item = ProgressItem::new("dl", "Download");
    /// assert_eq!(item.progress(), 0.0);
    /// ```
    pub fn progress(&self) -> f32 {
        self.progress
    }

    /// Returns the status.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ProgressItem, ProgressItemStatus};
    ///
    /// let item = ProgressItem::new("t1", "Task");
    /// assert_eq!(item.status(), ProgressItemStatus::Pending);
    /// ```
    pub fn status(&self) -> ProgressItemStatus {
        self.status
    }

    /// Returns the optional message.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ProgressItem;
    ///
    /// let item = ProgressItem::new("t1", "Task");
    /// assert_eq!(item.message(), None);
    /// ```
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    /// Returns the progress as a percentage (0-100).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ProgressItem;
    ///
    /// let item = ProgressItem::new("dl", "Download");
    /// assert_eq!(item.percentage(), 0);
    /// ```
    pub fn percentage(&self) -> u16 {
        (self.progress * 100.0).round() as u16
    }
}

/// Messages for the MultiProgress component.
#[derive(Clone, Debug, PartialEq)]
pub enum MultiProgressMessage {
    /// Add a new progress item.
    Add {
        /// Unique identifier.
        id: String,
        /// Display label.
        label: String,
    },
    /// Set progress for an item.
    SetProgress {
        /// Item identifier.
        id: String,
        /// Progress from 0.0 to 1.0.
        progress: f32,
    },
    /// Set status for an item.
    SetStatus {
        /// Item identifier.
        id: String,
        /// New status.
        status: ProgressItemStatus,
    },
    /// Set an optional message for an item.
    SetMessage {
        /// Item identifier.
        id: String,
        /// Optional message.
        message: Option<String>,
    },
    /// Mark an item as completed (sets progress to 1.0 and status to Completed).
    Complete(String),
    /// Mark an item as failed.
    Fail {
        /// Item identifier.
        id: String,
        /// Optional error message.
        message: Option<String>,
    },
    /// Remove an item.
    Remove(String),
    /// Clear all items.
    Clear,
    /// Select the currently focused item.
    Select,
    /// Scroll up.
    ScrollUp,
    /// Scroll down.
    ScrollDown,
    /// Scroll to top.
    ScrollToTop,
    /// Scroll to bottom.
    ScrollToBottom,
}

/// Output messages from MultiProgress.
#[derive(Clone, Debug, PartialEq)]
pub enum MultiProgressOutput {
    /// An item was added.
    Added(String),
    /// An item was completed.
    Completed(String),
    /// An item failed.
    Failed(String),
    /// An item was removed.
    Removed(String),
    /// All items were cleared.
    Cleared,
    /// An item was selected (Enter pressed on item at this index).
    Selected(usize),
}
