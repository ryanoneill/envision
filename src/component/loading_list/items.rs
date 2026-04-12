//! Item types for the LoadingList component.
//!
//! Extracted from the main loading_list module to keep file sizes manageable.

use ratatui::prelude::*;

use crate::theme::Theme;

/// Loading state of an individual item.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum ItemState {
    /// Item is ready (normal state).
    #[default]
    Ready,
    /// Item is currently loading.
    Loading,
    /// Item has an error.
    Error(String),
}

impl ItemState {
    /// Returns true if the item is loading.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ItemState;
    ///
    /// assert!(!ItemState::Ready.is_loading());
    /// assert!(ItemState::Loading.is_loading());
    /// ```
    pub fn is_loading(&self) -> bool {
        matches!(self, Self::Loading)
    }

    /// Returns true if the item has an error.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ItemState;
    ///
    /// assert!(!ItemState::Ready.is_error());
    /// assert!(ItemState::Error("failed".into()).is_error());
    /// ```
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error(_))
    }

    /// Returns true if the item is ready.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ItemState;
    ///
    /// assert!(ItemState::Ready.is_ready());
    /// assert!(!ItemState::Loading.is_ready());
    /// ```
    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready)
    }

    /// Returns the error message if in error state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ItemState;
    ///
    /// let state = ItemState::Error("connection lost".into());
    /// assert_eq!(state.error_message(), Some("connection lost"));
    ///
    /// assert_eq!(ItemState::Ready.error_message(), None);
    /// ```
    pub fn error_message(&self) -> Option<&str> {
        if let Self::Error(msg) = self {
            Some(msg)
        } else {
            None
        }
    }

    /// Returns the symbol for this state.
    pub fn symbol(&self, spinner_frame: usize) -> &'static str {
        match self {
            Self::Ready => " ",
            Self::Loading => {
                // Braille dots animation matching SpinnerStyle::Dots
                const LOADING_FRAMES: [&str; 10] =
                    ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
                LOADING_FRAMES[spinner_frame % LOADING_FRAMES.len()]
            }
            Self::Error(_) => "✗",
        }
    }

    /// Returns the style for this state using the theme.
    pub fn style(&self, theme: &Theme) -> Style {
        match self {
            Self::Ready => theme.normal_style(),
            Self::Loading => theme.warning_style(),
            Self::Error(_) => theme.error_style(),
        }
    }
}

/// A single item in the loading list.
#[derive(Clone, Debug)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct LoadingListItem<T: Clone> {
    /// The underlying data.
    pub(super) data: T,
    /// Display label.
    pub(super) label: String,
    /// Current loading state.
    pub(super) state: ItemState,
}

impl<T: Clone + PartialEq> PartialEq for LoadingListItem<T> {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data && self.label == other.label && self.state == other.state
    }
}

impl<T: Clone> LoadingListItem<T> {
    /// Creates a new item.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LoadingListItem;
    ///
    /// let item = LoadingListItem::new("data", "Label");
    /// assert_eq!(item.label(), "Label");
    /// assert!(item.is_ready());
    /// ```
    pub fn new(data: T, label: impl Into<String>) -> Self {
        Self {
            data,
            label: label.into(),
            state: ItemState::Ready,
        }
    }

    /// Returns the underlying data.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LoadingListItem;
    ///
    /// let item = LoadingListItem::new(42, "Answer");
    /// assert_eq!(*item.data(), 42);
    /// ```
    pub fn data(&self) -> &T {
        &self.data
    }

    /// Returns a mutable reference to the data.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LoadingListItem;
    ///
    /// let mut item = LoadingListItem::new(42u32, "Answer");
    /// *item.data_mut() = 99;
    /// assert_eq!(*item.data(), 99);
    /// ```
    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }

    /// Returns the label.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LoadingListItem;
    ///
    /// let item = LoadingListItem::new("x", "Display Name");
    /// assert_eq!(item.label(), "Display Name");
    /// ```
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Sets the label.
    pub fn set_label(&mut self, label: impl Into<String>) {
        self.label = label.into();
    }

    /// Returns the current state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ItemState, LoadingListItem};
    ///
    /// let item = LoadingListItem::new("x", "Item");
    /// assert_eq!(item.state(), &ItemState::Ready);
    /// ```
    pub fn state(&self) -> &ItemState {
        &self.state
    }

    /// Sets the state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ItemState, LoadingListItem};
    ///
    /// let mut item = LoadingListItem::new("x", "Item");
    /// item.set_state(ItemState::Loading);
    /// assert!(item.is_loading());
    /// ```
    pub fn set_state(&mut self, state: ItemState) {
        self.state = state;
    }

    /// Returns true if the item is loading.
    pub fn is_loading(&self) -> bool {
        self.state.is_loading()
    }

    /// Returns true if the item has an error.
    pub fn is_error(&self) -> bool {
        self.state.is_error()
    }

    /// Returns true if the item is ready.
    pub fn is_ready(&self) -> bool {
        self.state.is_ready()
    }
}
