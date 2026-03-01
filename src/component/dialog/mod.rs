//! A modal dialog component with configurable buttons.
//!
//! `Dialog` provides a centered overlay dialog with title, message, and
//! configurable buttons. This component implements both `Focusable` and
//! `Toggleable` traits for focus management and visibility control.
//!
//! # Example
//!
//! ```rust
//! use envision::component::{Dialog, DialogMessage, DialogOutput, DialogState, Component, Toggleable};
//!
//! // Create an alert dialog
//! let mut state = DialogState::alert("Error", "File not found.");
//! Dialog::show(&mut state);
//!
//! // Press the OK button
//! let output = Dialog::update(&mut state, DialogMessage::Press);
//! assert_eq!(output, Some(DialogOutput::ButtonPressed("ok".into())));
//! ```

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use super::{Component, Focusable, Toggleable};
use crate::input::{Event, KeyCode};
use crate::theme::Theme;

/// A button configuration for a dialog.
///
/// Each button has a unique identifier that is returned when the button
/// is pressed, and a display label shown to the user.
///
/// # Example
///
/// ```rust
/// use envision::component::DialogButton;
///
/// let button = DialogButton::new("save", "Save Changes");
/// assert_eq!(button.id(), "save");
/// assert_eq!(button.label(), "Save Changes");
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DialogButton {
    /// Unique identifier returned when this button is pressed.
    id: String,
    /// Display label for the button.
    label: String,
}

impl DialogButton {
    /// Creates a new dialog button.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier returned when pressed
    /// * `label` - Display text for the button
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DialogButton;
    ///
    /// let button = DialogButton::new("ok", "OK");
    /// ```
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
        }
    }

    /// Returns the button's unique identifier.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the button's display label.
    pub fn label(&self) -> &str {
        &self.label
    }
}

/// Messages that can be sent to a Dialog component.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DialogMessage {
    /// Move focus to the next button.
    FocusNext,
    /// Move focus to the previous button.
    FocusPrev,
    /// Press the currently focused button.
    Press,
    /// Close the dialog without selecting a button.
    Close,
    /// Show the dialog.
    Open,
}

/// Output messages from a Dialog component.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DialogOutput {
    /// A button was pressed (returns the button id).
    ButtonPressed(String),
    /// The dialog was closed without selection (e.g., Escape).
    Closed,
}

/// State for a Dialog component.
///
/// Contains all the state needed to render and manage a dialog,
/// including title, message, buttons, and focus state.
#[derive(Clone, Debug, Default)]
pub struct DialogState {
    /// Dialog title.
    title: String,
    /// Dialog message/content.
    message: String,
    /// Available buttons.
    buttons: Vec<DialogButton>,
    /// Index of the primary/default button.
    primary_button: usize,
    /// Index of the currently focused button.
    focused_button: usize,
    /// Whether the dialog is visible.
    visible: bool,
    /// Whether the dialog itself is focused (receives input).
    focused: bool,
}

impl DialogState {
    /// Creates a new dialog with the given title, message, and buttons.
    ///
    /// The first button is set as the primary button by default.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{DialogButton, DialogState};
    ///
    /// let buttons = vec![
    ///     DialogButton::new("ok", "OK"),
    ///     DialogButton::new("cancel", "Cancel"),
    /// ];
    /// let state = DialogState::new("Title", "Message", buttons);
    /// assert_eq!(state.title(), "Title");
    /// assert_eq!(state.primary_button(), 0);
    /// ```
    pub fn new(
        title: impl Into<String>,
        message: impl Into<String>,
        buttons: Vec<DialogButton>,
    ) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            buttons,
            primary_button: 0,
            focused_button: 0,
            visible: false,
            focused: false,
        }
    }

    /// Creates a dialog with a primary button specified.
    ///
    /// The primary button index is clamped to the valid range.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{DialogButton, DialogState};
    ///
    /// let buttons = vec![
    ///     DialogButton::new("cancel", "Cancel"),
    ///     DialogButton::new("ok", "OK"),
    /// ];
    /// let state = DialogState::with_primary("Title", "Message", buttons, 1);
    /// assert_eq!(state.primary_button(), 1);
    /// ```
    pub fn with_primary(
        title: impl Into<String>,
        message: impl Into<String>,
        buttons: Vec<DialogButton>,
        primary: usize,
    ) -> Self {
        let primary = if buttons.is_empty() {
            0
        } else {
            primary.min(buttons.len() - 1)
        };
        Self {
            title: title.into(),
            message: message.into(),
            buttons,
            primary_button: primary,
            focused_button: primary,
            visible: false,
            focused: false,
        }
    }

    /// Creates a simple alert dialog with an "OK" button.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DialogState;
    ///
    /// let state = DialogState::alert("Error", "Something went wrong.");
    /// assert_eq!(state.buttons().len(), 1);
    /// assert_eq!(state.buttons()[0].id(), "ok");
    /// ```
    pub fn alert(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(title, message, vec![DialogButton::new("ok", "OK")])
    }

    /// Creates a confirmation dialog with "Cancel" and "OK" buttons.
    ///
    /// The "OK" button is set as the primary button.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DialogState;
    ///
    /// let state = DialogState::confirm("Delete?", "This cannot be undone.");
    /// assert_eq!(state.buttons().len(), 2);
    /// assert_eq!(state.buttons()[0].id(), "cancel");
    /// assert_eq!(state.buttons()[1].id(), "ok");
    /// assert_eq!(state.primary_button(), 1);
    /// ```
    pub fn confirm(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::with_primary(
            title,
            message,
            vec![
                DialogButton::new("cancel", "Cancel"),
                DialogButton::new("ok", "OK"),
            ],
            1,
        )
    }

    /// Returns the dialog title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the dialog message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the dialog buttons.
    pub fn buttons(&self) -> &[DialogButton] {
        &self.buttons
    }

    /// Returns the index of the primary button.
    pub fn primary_button(&self) -> usize {
        self.primary_button
    }

    /// Returns the index of the currently focused button.
    pub fn focused_button(&self) -> usize {
        self.focused_button
    }

    /// Sets the dialog title.
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
    }

    /// Sets the dialog message.
    pub fn set_message(&mut self, message: impl Into<String>) {
        self.message = message.into();
    }

    /// Sets the dialog buttons.
    ///
    /// Resets focus to the first button or primary button index.
    pub fn set_buttons(&mut self, buttons: Vec<DialogButton>) {
        self.buttons = buttons;
        if self.buttons.is_empty() {
            self.primary_button = 0;
            self.focused_button = 0;
        } else {
            self.primary_button = self.primary_button.min(self.buttons.len() - 1);
            self.focused_button = self.primary_button;
        }
    }

    /// Sets the primary button index.
    ///
    /// The index is clamped to the valid range.
    pub fn set_primary_button(&mut self, index: usize) {
        if self.buttons.is_empty() {
            self.primary_button = 0;
        } else {
            self.primary_button = index.min(self.buttons.len() - 1);
        }
    }

    /// Returns true if the dialog is focused.
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Sets the focus state.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Returns true if the dialog is visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Sets the visibility state.
    pub fn set_visible(&mut self, visible: bool) {
        Dialog::set_visible(self, visible);
    }

    /// Maps an input event to a dialog message.
    pub fn handle_event(&self, event: &Event) -> Option<DialogMessage> {
        Dialog::handle_event(self, event)
    }

    /// Dispatches an event, updating state and returning any output.
    pub fn dispatch_event(&mut self, event: &Event) -> Option<DialogOutput> {
        Dialog::dispatch_event(self, event)
    }

    /// Updates the dialog state with a message, returning any output.
    pub fn update(&mut self, msg: DialogMessage) -> Option<DialogOutput> {
        Dialog::update(self, msg)
    }
}

/// A modal dialog component with configurable buttons.
///
/// `Dialog` displays a centered overlay with a title, message, and
/// configurable buttons. It implements:
/// - [`Component`] for update/view logic
/// - [`Focusable`] for keyboard focus
/// - [`Toggleable`] for visibility control
///
/// # Visual Format
///
/// ```text
/// ┌─────── Title ───────┐
/// │                     │
/// │  Message text here  │
/// │                     │
/// │   [Cancel]  [OK]    │
/// └─────────────────────┘
/// ```
///
/// # Messages
///
/// - `FocusNext` - Move focus to the next button
/// - `FocusPrev` - Move focus to the previous button
/// - `Press` - Press the currently focused button
/// - `Close` - Close the dialog without selection
/// - `Open` - Show the dialog
///
/// # Output
///
/// - `ButtonPressed(id)` - A button was pressed
/// - `Closed` - The dialog was closed without selection
///
/// # Example
///
/// ```rust
/// use envision::component::{
///     Dialog, DialogMessage, DialogOutput, DialogState, Component, Toggleable
/// };
///
/// // Create and show a confirmation dialog
/// let mut state = DialogState::confirm("Delete?", "This cannot be undone.");
/// Dialog::show(&mut state);
///
/// // Navigate to Cancel
/// Dialog::update(&mut state, DialogMessage::FocusPrev);
///
/// // Press Cancel
/// let output = Dialog::update(&mut state, DialogMessage::Press);
/// assert_eq!(output, Some(DialogOutput::ButtonPressed("cancel".into())));
/// ```
pub struct Dialog;

impl Component for Dialog {
    type State = DialogState;
    type Message = DialogMessage;
    type Output = DialogOutput;

    fn init() -> Self::State {
        DialogState::default()
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        if !state.visible {
            if matches!(msg, DialogMessage::Open) {
                state.visible = true;
                state.focused_button = state.primary_button;
            }
            return None;
        }

        match msg {
            DialogMessage::FocusNext => {
                if !state.buttons.is_empty() {
                    state.focused_button = (state.focused_button + 1) % state.buttons.len();
                }
                None
            }
            DialogMessage::FocusPrev => {
                if !state.buttons.is_empty() {
                    state.focused_button = state
                        .focused_button
                        .checked_sub(1)
                        .unwrap_or(state.buttons.len() - 1);
                }
                None
            }
            DialogMessage::Press => state.buttons.get(state.focused_button).map(|btn| {
                state.visible = false;
                DialogOutput::ButtonPressed(btn.id.clone())
            }),
            DialogMessage::Close => {
                state.visible = false;
                Some(DialogOutput::Closed)
            }
            DialogMessage::Open => None,
        }
    }

    fn handle_event(state: &Self::State, event: &Event) -> Option<Self::Message> {
        if !state.visible {
            return None;
        }
        if let Some(key) = event.as_key() {
            match key.code {
                KeyCode::Tab => Some(DialogMessage::FocusNext),
                KeyCode::BackTab => Some(DialogMessage::FocusPrev),
                KeyCode::Enter => Some(DialogMessage::Press),
                KeyCode::Esc => Some(DialogMessage::Close),
                _ => None,
            }
        } else {
            None
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme) {
        if !state.visible {
            return;
        }

        // Calculate dialog size
        let dialog_width = (area.width * 60 / 100).clamp(30, 80);
        let message_lines = state.message.lines().count().max(1) as u16;
        let dialog_height = (5 + message_lines).min(area.height);

        // Center the dialog
        let dialog_area = centered_rect(dialog_width, dialog_height, area);

        // Clear the dialog area (overlay effect)
        frame.render_widget(Clear, dialog_area);

        // Render dialog box
        let block = Block::default()
            .title(format!(" {} ", state.title))
            .borders(Borders::ALL)
            .border_style(theme.border_style());

        let inner = block.inner(dialog_area);
        frame.render_widget(block, dialog_area);

        // Layout: message area + button row
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),    // Message
                Constraint::Length(3), // Buttons
            ])
            .split(inner);

        // Render message
        let message = Paragraph::new(state.message.as_str()).wrap(Wrap { trim: true });
        frame.render_widget(message, chunks[0]);

        // Render buttons horizontally centered
        render_buttons(state, frame, chunks[1], theme);
    }
}

impl Focusable for Dialog {
    fn is_focused(state: &Self::State) -> bool {
        state.focused
    }

    fn set_focused(state: &mut Self::State, focused: bool) {
        state.focused = focused;
    }
}

impl Toggleable for Dialog {
    fn is_visible(state: &Self::State) -> bool {
        state.visible
    }

    fn set_visible(state: &mut Self::State, visible: bool) {
        state.visible = visible;
        if visible {
            state.focused_button = state.primary_button;
        }
    }
}

/// Calculates a centered rectangle within the given area.
fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}

/// Renders the dialog buttons horizontally centered.
fn render_buttons(state: &DialogState, frame: &mut Frame, area: Rect, theme: &Theme) {
    if state.buttons.is_empty() {
        return;
    }

    // Calculate button widths (label + padding + borders)
    let button_widths: Vec<u16> = state
        .buttons
        .iter()
        .map(|b| (b.label.len() + 4) as u16)
        .collect();
    let total_width: u16 =
        button_widths.iter().sum::<u16>() + (state.buttons.len().saturating_sub(1) as u16 * 2);

    // Center the buttons
    let start_x = area.x + area.width.saturating_sub(total_width) / 2;
    let mut x = start_x;

    for (i, button) in state.buttons.iter().enumerate() {
        let width = button_widths[i];
        let button_area = Rect::new(x, area.y, width, 3.min(area.height));

        let is_focused = i == state.focused_button && state.focused;
        let is_primary = i == state.primary_button;

        let style = if is_focused {
            theme.focused_bold_style()
        } else if is_primary {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            theme.normal_style()
        };

        let border_style = if is_focused {
            theme.focused_border_style()
        } else {
            theme.border_style()
        };

        let btn = Paragraph::new(button.label.as_str())
            .style(style)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(border_style),
            );

        frame.render_widget(btn, button_area);
        x += width + 2;
    }
}

#[cfg(test)]
mod tests;
