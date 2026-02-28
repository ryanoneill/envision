//! A component for displaying keyboard shortcuts.
//!
//! `KeyHints` provides a bar displaying keyboard shortcuts with their actions,
//! commonly used at the bottom of TUI applications to show available commands.
//!
//! # Example
//!
//! ```rust
//! use envision::component::{KeyHints, KeyHint, KeyHintsState, KeyHintsLayout, Component};
//!
//! // Create hints using builder pattern
//! let state = KeyHintsState::new()
//!     .hint("Enter", "Select")
//!     .hint("q", "Quit")
//!     .hint("?", "Help");
//!
//! // Or create with a vector
//! let hints = vec![
//!     KeyHint::new("↑/k", "Up"),
//!     KeyHint::new("↓/j", "Down"),
//!     KeyHint::new("Enter", "Select"),
//! ];
//! let state = KeyHintsState::with_hints(hints);
//! ```

use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use super::Component;
use crate::theme::Theme;

/// Layout style for key hints display.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum KeyHintsLayout {
    /// Hints are displayed with even spacing.
    #[default]
    Spaced,
    /// Hints are displayed inline with minimal spacing.
    Inline,
}

/// A single keyboard hint entry.
///
/// Represents a key-action pair that tells the user what a key does.
///
/// # Example
///
/// ```rust
/// use envision::component::KeyHint;
///
/// let hint = KeyHint::new("Ctrl+S", "Save")
///     .with_priority(1)
///     .with_enabled(true);
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyHint {
    /// The key or key combination.
    key: String,
    /// The action description.
    action: String,
    /// Whether this hint is enabled/visible.
    enabled: bool,
    /// Priority for responsive hiding (lower = more important, shown first).
    priority: u8,
}

impl KeyHint {
    /// Creates a new key hint.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::KeyHint;
    ///
    /// let hint = KeyHint::new("Enter", "Select");
    /// assert_eq!(hint.key(), "Enter");
    /// assert_eq!(hint.action(), "Select");
    /// ```
    pub fn new(key: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            action: action.into(),
            enabled: true,
            priority: 100,
        }
    }

    /// Sets the priority for responsive hiding.
    ///
    /// Lower priority values are more important and will be shown first
    /// when space is limited.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::KeyHint;
    ///
    /// let hint = KeyHint::new("q", "Quit").with_priority(1);
    /// assert_eq!(hint.priority(), 1);
    /// ```
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    /// Sets whether the hint is enabled.
    ///
    /// Disabled hints are not displayed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::KeyHint;
    ///
    /// let hint = KeyHint::new("Delete", "Remove").with_enabled(false);
    /// assert!(!hint.is_enabled());
    /// ```
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Returns the key string.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Returns the action description.
    pub fn action(&self) -> &str {
        &self.action
    }

    /// Returns whether the hint is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Returns the priority value.
    pub fn priority(&self) -> u8 {
        self.priority
    }

    /// Sets the key string.
    pub fn set_key(&mut self, key: impl Into<String>) {
        self.key = key.into();
    }

    /// Sets the action description.
    pub fn set_action(&mut self, action: impl Into<String>) {
        self.action = action.into();
    }

    /// Sets whether the hint is enabled.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Sets the priority.
    pub fn set_priority(&mut self, priority: u8) {
        self.priority = priority;
    }
}

/// Messages that can be sent to a KeyHints component.
#[derive(Clone, Debug, PartialEq)]
pub enum KeyHintsMessage {
    /// Set all hints at once.
    SetHints(Vec<KeyHint>),
    /// Add a single hint.
    AddHint(KeyHint),
    /// Remove a hint by key.
    RemoveHint(String),
    /// Enable a hint by key.
    EnableHint(String),
    /// Disable a hint by key.
    DisableHint(String),
    /// Set the layout style.
    SetLayout(KeyHintsLayout),
    /// Clear all hints.
    Clear,
}

/// Output messages from a KeyHints component.
///
/// KeyHints is display-only and doesn't produce output messages.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeyHintsOutput {}

/// State for a KeyHints component.
///
/// Contains all hints and display configuration.
///
/// # Example
///
/// ```rust
/// use envision::component::{KeyHintsState, KeyHintsLayout};
///
/// let state = KeyHintsState::new()
///     .with_layout(KeyHintsLayout::Inline)
///     .hint("Enter", "Select")
///     .hint("Esc", "Cancel");
/// ```
#[derive(Clone, Debug)]
pub struct KeyHintsState {
    /// All key hints.
    hints: Vec<KeyHint>,
    /// Layout style.
    layout: KeyHintsLayout,
    /// Separator between key and action (default: " ").
    key_action_separator: String,
    /// Separator between hints (default: "  ").
    hint_separator: String,
    /// Key style.
    key_style: Style,
    /// Action style.
    action_style: Style,
}

impl Default for KeyHintsState {
    fn default() -> Self {
        Self {
            hints: Vec::new(),
            layout: KeyHintsLayout::default(),
            key_action_separator: " ".to_string(),
            hint_separator: "  ".to_string(),
            key_style: Style::default().fg(Color::Green),
            action_style: Style::default(),
        }
    }
}

impl KeyHintsState {
    /// Creates a new empty KeyHints state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::KeyHintsState;
    ///
    /// let state = KeyHintsState::new();
    /// assert!(state.is_empty());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new state with the given hints.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{KeyHintsState, KeyHint};
    ///
    /// let hints = vec![KeyHint::new("q", "Quit")];
    /// let state = KeyHintsState::with_hints(hints);
    /// assert_eq!(state.len(), 1);
    /// ```
    pub fn with_hints(hints: Vec<KeyHint>) -> Self {
        Self {
            hints,
            ..Self::default()
        }
    }

    /// Sets the layout style.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{KeyHintsState, KeyHintsLayout};
    ///
    /// let state = KeyHintsState::new().with_layout(KeyHintsLayout::Inline);
    /// assert_eq!(state.layout(), KeyHintsLayout::Inline);
    /// ```
    pub fn with_layout(mut self, layout: KeyHintsLayout) -> Self {
        self.layout = layout;
        self
    }

    /// Sets the separator between key and action.
    pub fn with_key_action_separator(mut self, sep: impl Into<String>) -> Self {
        self.key_action_separator = sep.into();
        self
    }

    /// Sets the separator between hints.
    pub fn with_hint_separator(mut self, sep: impl Into<String>) -> Self {
        self.hint_separator = sep.into();
        self
    }

    /// Sets the style for keys.
    pub fn with_key_style(mut self, style: Style) -> Self {
        self.key_style = style;
        self
    }

    /// Sets the style for actions.
    pub fn with_action_style(mut self, style: Style) -> Self {
        self.action_style = style;
        self
    }

    /// Adds a hint using builder pattern.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::KeyHintsState;
    ///
    /// let state = KeyHintsState::new()
    ///     .hint("Enter", "Select")
    ///     .hint("Esc", "Cancel")
    ///     .hint("q", "Quit");
    ///
    /// assert_eq!(state.len(), 3);
    /// ```
    pub fn hint(mut self, key: impl Into<String>, action: impl Into<String>) -> Self {
        self.hints.push(KeyHint::new(key, action));
        self
    }

    /// Adds a hint with priority using builder pattern.
    pub fn hint_with_priority(
        mut self,
        key: impl Into<String>,
        action: impl Into<String>,
        priority: u8,
    ) -> Self {
        self.hints
            .push(KeyHint::new(key, action).with_priority(priority));
        self
    }

    /// Returns all hints.
    pub fn hints(&self) -> &[KeyHint] {
        &self.hints
    }

    /// Returns only enabled hints, sorted by priority.
    pub fn visible_hints(&self) -> Vec<&KeyHint> {
        let mut visible: Vec<_> = self.hints.iter().filter(|h| h.enabled).collect();
        visible.sort_by_key(|h| h.priority);
        visible
    }

    /// Returns the layout style.
    pub fn layout(&self) -> KeyHintsLayout {
        self.layout
    }

    /// Returns the number of hints.
    pub fn len(&self) -> usize {
        self.hints.len()
    }

    /// Returns true if there are no hints.
    pub fn is_empty(&self) -> bool {
        self.hints.is_empty()
    }

    /// Sets the hints.
    pub fn set_hints(&mut self, hints: Vec<KeyHint>) {
        self.hints = hints;
    }

    /// Adds a hint.
    pub fn add_hint(&mut self, hint: KeyHint) {
        self.hints.push(hint);
    }

    /// Removes a hint by key.
    pub fn remove_hint(&mut self, key: &str) {
        self.hints.retain(|h| h.key != key);
    }

    /// Enables a hint by key.
    pub fn enable_hint(&mut self, key: &str) {
        if let Some(hint) = self.hints.iter_mut().find(|h| h.key == key) {
            hint.enabled = true;
        }
    }

    /// Disables a hint by key.
    pub fn disable_hint(&mut self, key: &str) {
        if let Some(hint) = self.hints.iter_mut().find(|h| h.key == key) {
            hint.enabled = false;
        }
    }

    /// Sets the layout.
    pub fn set_layout(&mut self, layout: KeyHintsLayout) {
        self.layout = layout;
    }

    /// Clears all hints.
    pub fn clear(&mut self) {
        self.hints.clear();
    }

    /// Returns the key style.
    pub fn key_style(&self) -> Style {
        self.key_style
    }

    /// Returns the action style.
    pub fn action_style(&self) -> Style {
        self.action_style
    }

    /// Sets the key style.
    pub fn set_key_style(&mut self, style: Style) {
        self.key_style = style;
    }

    /// Sets the action style.
    pub fn set_action_style(&mut self, style: Style) {
        self.action_style = style;
    }
}

/// A component for displaying keyboard shortcuts.
///
/// `KeyHints` displays a row of key-action pairs, typically shown at the
/// bottom of a TUI to inform users of available commands.
///
/// # Visual Format
///
/// ```text
/// Spaced:  Enter Select    Esc Cancel    q Quit
/// Inline:  Enter Select  Esc Cancel  q Quit
/// ```
///
/// # Example
///
/// ```rust
/// use envision::component::{KeyHints, KeyHintsState, KeyHintsMessage, Component};
///
/// let mut state = KeyHintsState::new()
///     .hint("Enter", "Select")
///     .hint("q", "Quit");
///
/// // Disable a hint dynamically
/// KeyHints::update(&mut state, KeyHintsMessage::DisableHint("q".to_string()));
/// ```
pub struct KeyHints;

impl Component for KeyHints {
    type State = KeyHintsState;
    type Message = KeyHintsMessage;
    type Output = KeyHintsOutput;

    fn init() -> Self::State {
        KeyHintsState::default()
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            KeyHintsMessage::SetHints(hints) => {
                state.hints = hints;
            }
            KeyHintsMessage::AddHint(hint) => {
                state.hints.push(hint);
            }
            KeyHintsMessage::RemoveHint(key) => {
                state.hints.retain(|h| h.key != key);
            }
            KeyHintsMessage::EnableHint(key) => {
                if let Some(hint) = state.hints.iter_mut().find(|h| h.key == key) {
                    hint.enabled = true;
                }
            }
            KeyHintsMessage::DisableHint(key) => {
                if let Some(hint) = state.hints.iter_mut().find(|h| h.key == key) {
                    hint.enabled = false;
                }
            }
            KeyHintsMessage::SetLayout(layout) => {
                state.layout = layout;
            }
            KeyHintsMessage::Clear => {
                state.hints.clear();
            }
        }
        None // Display-only, no output
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme) {
        if state.hints.is_empty() || area.width == 0 || area.height == 0 {
            return;
        }

        let visible = state.visible_hints();
        if visible.is_empty() {
            return;
        }

        let mut spans = Vec::new();

        // Use theme for key style (focused/success color for keys)
        let key_style = if state.key_style == Style::default().fg(Color::Green) {
            theme.success_style()
        } else {
            state.key_style
        };

        for (i, hint) in visible.iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw(&state.hint_separator));
            }

            spans.push(Span::styled(&hint.key, key_style));
            spans.push(Span::raw(&state.key_action_separator));
            spans.push(Span::styled(&hint.action, state.action_style));
        }

        let line = Line::from(spans);
        let alignment = match state.layout {
            KeyHintsLayout::Spaced => Alignment::Center,
            KeyHintsLayout::Inline => Alignment::Left,
        };

        let paragraph = Paragraph::new(line).alignment(alignment);
        frame.render_widget(paragraph, area);
    }
}

#[cfg(test)]
mod tests;
