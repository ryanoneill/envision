//! A confirmation dialog component with preset button configurations.
//!
//! [`ConfirmDialog`] provides a centered modal overlay with a title, message,
//! and button configurations such as Ok, Ok/Cancel, Yes/No, and Yes/No/Cancel.
//! It supports keyboard shortcuts for quick responses. State is stored in
//! [`ConfirmDialogState`], updated via [`ConfirmDialogMessage`], and produces
//! [`ConfirmDialogOutput`] containing a [`ConfirmDialogResult`].
//! Button layouts are configured with [`ButtonConfig`].
//!
//! Implements [`Toggleable`].
//!
//! See also [`Dialog`](super::Dialog) for a general-purpose modal dialog.
//!
//! # Example
//!
//! ```rust
//! use envision::component::{
//!     ConfirmDialog, ConfirmDialogMessage, ConfirmDialogOutput,
//!     ConfirmDialogResult, ConfirmDialogState, Component, Toggleable,
//! };
//!
//! // Create a Yes/No dialog
//! let mut state = ConfirmDialogState::yes_no("Delete?", "This cannot be undone.");
//! ConfirmDialog::show(&mut state);
//!
//! // Press Yes
//! let output = ConfirmDialog::update(&mut state, ConfirmDialogMessage::SelectResult(ConfirmDialogResult::Yes));
//! assert_eq!(output, Some(ConfirmDialogOutput::Confirmed(ConfirmDialogResult::Yes)));
//! ```

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use super::{Component, EventContext, RenderContext, Toggleable};
use crate::input::{Event, Key};
use crate::theme::Theme;

/// Preset button configurations for the confirm dialog.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum ButtonConfig {
    /// Single "OK" button.
    Ok,
    /// "OK" and "Cancel" buttons.
    OkCancel,
    /// "Yes" and "No" buttons.
    YesNo,
    /// "Yes", "No", and "Cancel" buttons.
    YesNoCancel,
}

impl ButtonConfig {
    fn labels(&self) -> Vec<(&'static str, ConfirmDialogResult)> {
        match self {
            ButtonConfig::Ok => vec![("OK", ConfirmDialogResult::Ok)],
            ButtonConfig::OkCancel => vec![
                ("OK", ConfirmDialogResult::Ok),
                ("Cancel", ConfirmDialogResult::Cancel),
            ],
            ButtonConfig::YesNo => vec![
                ("Yes", ConfirmDialogResult::Yes),
                ("No", ConfirmDialogResult::No),
            ],
            ButtonConfig::YesNoCancel => vec![
                ("Yes", ConfirmDialogResult::Yes),
                ("No", ConfirmDialogResult::No),
                ("Cancel", ConfirmDialogResult::Cancel),
            ],
        }
    }

    fn button_count(&self) -> usize {
        self.labels().len()
    }

    fn has_yes_no(&self) -> bool {
        matches!(self, ButtonConfig::YesNo | ButtonConfig::YesNoCancel)
    }
}

/// The result of a confirm dialog interaction.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum ConfirmDialogResult {
    /// The user confirmed with "OK".
    Ok,
    /// The user cancelled.
    Cancel,
    /// The user confirmed with "Yes".
    Yes,
    /// The user declined with "No".
    No,
}

/// Messages that can be sent to a ConfirmDialog.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConfirmDialogMessage {
    /// Move focus to the next button.
    FocusNext,
    /// Move focus to the previous button.
    FocusPrev,
    /// Press the currently focused button.
    Press,
    /// Close the dialog without selecting.
    Close,
    /// Open the dialog.
    Open,
    /// Select a specific result directly.
    SelectResult(ConfirmDialogResult),
}

/// Output messages from a ConfirmDialog.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConfirmDialogOutput {
    /// A result was selected.
    Confirmed(ConfirmDialogResult),
    /// The dialog was closed without selection.
    Closed,
}

/// State for a ConfirmDialog component.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct ConfirmDialogState {
    title: String,
    message: String,
    button_config: ButtonConfig,
    focused_button: usize,
    visible: bool,
    destructive_button: Option<usize>,
}

impl Default for ConfirmDialogState {
    fn default() -> Self {
        Self {
            title: String::new(),
            message: String::new(),
            button_config: ButtonConfig::Ok,
            focused_button: 0,
            visible: false,
            destructive_button: None,
        }
    }
}

impl ConfirmDialogState {
    /// Creates a new confirm dialog with default Ok button config.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ConfirmDialogState;
    ///
    /// let state = ConfirmDialogState::new("Title", "Message");
    /// assert_eq!(state.title(), "Title");
    /// assert_eq!(state.message(), "Message");
    /// ```
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            ..Self::default()
        }
    }

    /// Creates an Ok-only dialog.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ConfirmDialogState, confirm_dialog::ButtonConfig};
    ///
    /// let state = ConfirmDialogState::ok("Info", "Operation complete.");
    /// assert_eq!(state.button_config(), &ButtonConfig::Ok);
    /// ```
    pub fn ok(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            button_config: ButtonConfig::Ok,
            ..Self::default()
        }
    }

    /// Creates an Ok/Cancel dialog.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ConfirmDialogState, confirm_dialog::ButtonConfig};
    ///
    /// let state = ConfirmDialogState::ok_cancel("Confirm", "Are you sure?");
    /// assert_eq!(state.button_config(), &ButtonConfig::OkCancel);
    /// ```
    pub fn ok_cancel(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            button_config: ButtonConfig::OkCancel,
            ..Self::default()
        }
    }

    /// Creates a Yes/No dialog.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ConfirmDialogState, confirm_dialog::ButtonConfig};
    ///
    /// let state = ConfirmDialogState::yes_no("Delete?", "Are you sure?");
    /// assert_eq!(state.button_config(), &ButtonConfig::YesNo);
    /// ```
    pub fn yes_no(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            button_config: ButtonConfig::YesNo,
            ..Self::default()
        }
    }

    /// Creates a Yes/No/Cancel dialog.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ConfirmDialogState, confirm_dialog::ButtonConfig};
    ///
    /// let state = ConfirmDialogState::yes_no_cancel("Save?", "Save changes?");
    /// assert_eq!(state.button_config(), &ButtonConfig::YesNoCancel);
    /// ```
    pub fn yes_no_cancel(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            button_config: ButtonConfig::YesNoCancel,
            ..Self::default()
        }
    }

    /// Creates a destructive confirmation dialog.
    ///
    /// The `destructive_index` indicates which button is destructive
    /// and will be styled with error colors.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ConfirmDialogState, confirm_dialog::ButtonConfig};
    ///
    /// let state = ConfirmDialogState::destructive(
    ///     "Delete?", "This cannot be undone.",
    ///     ButtonConfig::YesNo, 0,
    /// );
    /// assert_eq!(state.destructive_button(), Some(0));
    /// ```
    pub fn destructive(
        title: impl Into<String>,
        message: impl Into<String>,
        config: ButtonConfig,
        destructive_index: usize,
    ) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            button_config: config,
            destructive_button: Some(destructive_index),
            ..Self::default()
        }
    }

    /// Sets the button configuration (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ConfirmDialogState, confirm_dialog::ButtonConfig};
    ///
    /// let state = ConfirmDialogState::new("Title", "Message")
    ///     .with_button_config(ButtonConfig::YesNo);
    /// assert_eq!(state.button_config(), &ButtonConfig::YesNo);
    /// ```
    pub fn with_button_config(mut self, config: ButtonConfig) -> Self {
        self.button_config = config;
        self
    }

    /// Sets the destructive button index (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ConfirmDialogState;
    ///
    /// let state = ConfirmDialogState::new("Delete", "Sure?")
    ///     .with_destructive_button(Some(0));
    /// assert_eq!(state.destructive_button(), Some(0));
    /// ```
    pub fn with_destructive_button(mut self, index: Option<usize>) -> Self {
        self.destructive_button = index;
        self
    }

    /// Returns the dialog title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ConfirmDialogState;
    ///
    /// let state = ConfirmDialogState::new("Confirm", "Are you sure?");
    /// assert_eq!(state.title(), "Confirm");
    /// ```
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the dialog message.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ConfirmDialogState;
    ///
    /// let state = ConfirmDialogState::new("Confirm", "Are you sure?");
    /// assert_eq!(state.message(), "Are you sure?");
    /// ```
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the button configuration.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ConfirmDialogState, confirm_dialog::ButtonConfig};
    ///
    /// let state = ConfirmDialogState::yes_no("Delete?", "Are you sure?");
    /// assert_eq!(state.button_config(), &ButtonConfig::YesNo);
    /// ```
    pub fn button_config(&self) -> &ButtonConfig {
        &self.button_config
    }

    /// Returns the focused button index.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ConfirmDialogState;
    ///
    /// let state = ConfirmDialogState::new("Title", "Message");
    /// assert_eq!(state.focused_button(), 0);
    /// ```
    pub fn focused_button(&self) -> usize {
        self.focused_button
    }

    /// Returns the destructive button index, if any.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ConfirmDialogState;
    ///
    /// let state = ConfirmDialogState::new("Title", "Message");
    /// assert_eq!(state.destructive_button(), None);
    /// ```
    pub fn destructive_button(&self) -> Option<usize> {
        self.destructive_button
    }

    /// Returns true if the dialog is visible.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ConfirmDialogState;
    ///
    /// let state = ConfirmDialogState::new("Title", "Message");
    /// assert!(!state.is_visible());
    /// ```
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Sets the visibility state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ConfirmDialogState;
    ///
    /// let mut state = ConfirmDialogState::new("Title", "Message");
    /// state.set_visible(true);
    /// assert!(state.is_visible());
    /// ```
    pub fn set_visible(&mut self, visible: bool) {
        ConfirmDialog::set_visible(self, visible);
    }

    /// Sets the visibility (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ConfirmDialogState;
    ///
    /// let state = ConfirmDialogState::new("Title", "Message")
    ///     .with_visible(true);
    /// assert!(state.is_visible());
    /// ```
    pub fn with_visible(mut self, visible: bool) -> Self {
        ConfirmDialog::set_visible(&mut self, visible);
        self
    }

    /// Maps an input event to a dialog message.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ConfirmDialogState, ConfirmDialogMessage};
    /// use envision::input::{Event, Key};
    ///
    /// let mut state = ConfirmDialogState::yes_no("Delete?", "Sure?");
    /// state.set_visible(true);
    /// let event = Event::key(Key::Esc);
    /// assert_eq!(state.handle_event(&event), Some(ConfirmDialogMessage::Close));
    /// ```
    pub fn handle_event(&self, event: &Event) -> Option<ConfirmDialogMessage> {
        ConfirmDialog::handle_event(self, event, &EventContext::default())
    }

    /// Dispatches an event, updating state and returning any output.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ConfirmDialogState, ConfirmDialogOutput};
    /// use envision::input::{Event, Key};
    ///
    /// let mut state = ConfirmDialogState::new("Info", "Done.");
    /// state.set_visible(true);
    /// let event = Event::key(Key::Esc);
    /// let output = state.dispatch_event(&event);
    /// assert_eq!(output, Some(ConfirmDialogOutput::Closed));
    /// assert!(!state.is_visible());
    /// ```
    pub fn dispatch_event(&mut self, event: &Event) -> Option<ConfirmDialogOutput> {
        ConfirmDialog::dispatch_event(self, event, &EventContext::default())
    }

    /// Updates the dialog state with a message, returning any output.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{
    ///     ConfirmDialogState, ConfirmDialogMessage, ConfirmDialogOutput, ConfirmDialogResult,
    /// };
    ///
    /// let mut state = ConfirmDialogState::yes_no("Delete?", "Sure?");
    /// state.set_visible(true);
    /// let output = state.update(ConfirmDialogMessage::SelectResult(ConfirmDialogResult::Yes));
    /// assert_eq!(output, Some(ConfirmDialogOutput::Confirmed(ConfirmDialogResult::Yes)));
    /// ```
    pub fn update(&mut self, msg: ConfirmDialogMessage) -> Option<ConfirmDialogOutput> {
        ConfirmDialog::update(self, msg)
    }
}

/// A confirmation dialog with preset button configurations.
///
/// `ConfirmDialog` displays a centered overlay with a title, message,
/// and configurable buttons. It implements:
/// - [`Component`] for update/view logic
/// - [`Toggleable`] for visibility control
///
/// # Visual Format
///
/// ```text
/// ┌────── Delete? ──────┐
/// │                      │
/// │  This cannot be      │
/// │  undone.             │
/// │                      │
/// │    [Yes]   [No]      │
/// └──────────────────────┘
/// ```
///
/// # Keyboard Shortcuts
///
/// - `Tab` / `Shift+Tab` - Navigate between buttons
/// - `Enter` - Press focused button
/// - `Esc` - Close without selection
/// - `y` / `Y` - Select Yes (when Yes/No config)
/// - `n` / `N` - Select No (when Yes/No config)
///
/// # Example
///
/// ```rust
/// use envision::component::{
///     ConfirmDialog, ConfirmDialogMessage, ConfirmDialogOutput,
///     ConfirmDialogResult, ConfirmDialogState, Component, Toggleable,
/// };
///
/// let mut state = ConfirmDialogState::yes_no("Save?", "Save before closing?");
/// ConfirmDialog::show(&mut state);
///
/// // User presses 'y'
/// let output = ConfirmDialog::update(
///     &mut state,
///     ConfirmDialogMessage::SelectResult(ConfirmDialogResult::Yes),
/// );
/// assert_eq!(output, Some(ConfirmDialogOutput::Confirmed(ConfirmDialogResult::Yes)));
/// ```
pub struct ConfirmDialog;

impl Component for ConfirmDialog {
    type State = ConfirmDialogState;
    type Message = ConfirmDialogMessage;
    type Output = ConfirmDialogOutput;

    fn init() -> Self::State {
        ConfirmDialogState::default()
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        if !state.visible {
            if matches!(msg, ConfirmDialogMessage::Open) {
                state.visible = true;
                state.focused_button = 0;
            }
            return None;
        }

        let button_count = state.button_config.button_count();

        match msg {
            ConfirmDialogMessage::FocusNext => {
                if button_count > 0 {
                    state.focused_button = (state.focused_button + 1) % button_count;
                }
                None
            }
            ConfirmDialogMessage::FocusPrev => {
                if button_count > 0 {
                    state.focused_button = state
                        .focused_button
                        .checked_sub(1)
                        .unwrap_or(button_count - 1);
                }
                None
            }
            ConfirmDialogMessage::Press => {
                let labels = state.button_config.labels();
                labels.get(state.focused_button).map(|(_, result)| {
                    state.visible = false;
                    ConfirmDialogOutput::Confirmed(result.clone())
                })
            }
            ConfirmDialogMessage::Close => {
                state.visible = false;
                Some(ConfirmDialogOutput::Closed)
            }
            ConfirmDialogMessage::Open => None,
            ConfirmDialogMessage::SelectResult(result) => {
                state.visible = false;
                Some(ConfirmDialogOutput::Confirmed(result))
            }
        }
    }

    fn handle_event(
        state: &Self::State,
        event: &Event,
        _ctx: &EventContext,
    ) -> Option<Self::Message> {
        if !state.visible {
            return None;
        }
        if let Some(key) = event.as_key() {
            match key.code {
                Key::Tab if key.modifiers.shift() => Some(ConfirmDialogMessage::FocusPrev),
                Key::Tab => Some(ConfirmDialogMessage::FocusNext),
                Key::Enter => Some(ConfirmDialogMessage::Press),
                Key::Esc => Some(ConfirmDialogMessage::Close),
                Key::Char('y') if state.button_config.has_yes_no() => {
                    Some(ConfirmDialogMessage::SelectResult(ConfirmDialogResult::Yes))
                }
                Key::Char('n') if state.button_config.has_yes_no() => {
                    Some(ConfirmDialogMessage::SelectResult(ConfirmDialogResult::No))
                }
                _ => None,
            }
        } else {
            None
        }
    }

    fn view(state: &Self::State, ctx: &mut RenderContext<'_, '_>) {
        if !state.visible {
            return;
        }

        crate::annotation::with_registry(|reg| {
            reg.register(
                ctx.area,
                crate::annotation::Annotation::new(crate::annotation::WidgetType::ConfirmDialog)
                    .with_id("confirm_dialog")
                    .with_label(state.title.as_str())
                    .with_focus(ctx.focused)
                    .with_disabled(ctx.disabled),
            );
        });

        // Calculate dialog size
        let dialog_width = (ctx.area.width * 60 / 100).clamp(30, 80);
        let message_lines = state.message.lines().count().max(1) as u16;
        let dialog_height = (5 + message_lines).min(ctx.area.height);

        let dialog_area = crate::util::centered_rect(dialog_width, dialog_height, ctx.area);

        // Clear the dialog ctx.area (overlay effect)
        ctx.frame.render_widget(Clear, dialog_area);

        // Render dialog box
        let block = Block::default()
            .title(format!(" {} ", state.title))
            .borders(Borders::ALL)
            .border_style(ctx.theme.border_style());

        let inner = block.inner(dialog_area);
        ctx.frame.render_widget(block, dialog_area);

        // Layout: message ctx.area + button row
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),    // Message
                Constraint::Length(3), // Buttons
            ])
            .split(inner);

        // Render message
        let message = Paragraph::new(state.message.as_str()).wrap(Wrap { trim: true });
        ctx.frame.render_widget(message, chunks[0]);

        // Render buttons
        render_confirm_buttons(
            state,
            ctx.frame,
            chunks[1],
            ctx.theme,
            ctx.focused,
            ctx.disabled,
        );
    }
}

impl Toggleable for ConfirmDialog {
    fn is_visible(state: &Self::State) -> bool {
        state.visible
    }

    fn set_visible(state: &mut Self::State, visible: bool) {
        state.visible = visible;
        if visible {
            state.focused_button = 0;
        }
    }
}

/// Renders the confirm dialog buttons horizontally centered.
fn render_confirm_buttons(
    state: &ConfirmDialogState,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    focused: bool,
    disabled: bool,
) {
    let labels = state.button_config.labels();
    if labels.is_empty() {
        return;
    }

    let button_widths: Vec<u16> = labels.iter().map(|(l, _)| (l.len() + 4) as u16).collect();
    let total_width: u16 =
        button_widths.iter().sum::<u16>() + (labels.len().saturating_sub(1) as u16 * 2);

    let start_x = area.x + area.width.saturating_sub(total_width) / 2;
    let mut x = start_x;

    for (i, (label, _)) in labels.iter().enumerate() {
        let width = button_widths[i];
        let button_area = Rect::new(x, area.y, width, 3.min(area.height));

        let is_focused = i == state.focused_button && focused;
        let is_destructive = state.destructive_button == Some(i);

        let style = if disabled {
            theme.disabled_style()
        } else if is_destructive && is_focused {
            theme.error_style().add_modifier(Modifier::BOLD)
        } else if is_destructive {
            theme.error_style()
        } else if is_focused {
            theme.focused_bold_style()
        } else {
            theme.normal_style()
        };

        let border_style = if is_focused && !disabled {
            theme.focused_border_style()
        } else {
            theme.border_style()
        };

        let btn = Paragraph::new(*label)
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
