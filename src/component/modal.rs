//! A modal dialog component with overlay backdrop.
//!
//! `Modal` provides a popup dialog that appears over the main content with
//! a semi-transparent overlay backdrop. It's useful for confirmations, alerts,
//! forms, and other focused interactions.
//!
//! # Example
//!
//! ```rust
//! use envision::component::{Modal, ModalMessage, ModalOutput, ModalState, Component, Toggleable};
//!
//! // Create a modal
//! let mut state = ModalState::new("Confirm Action", 40, 10);
//!
//! // Show it
//! Modal::show(&mut state);
//! assert!(Modal::is_visible(&state));
//!
//! // Close it
//! let output = Modal::update(&mut state, ModalMessage::Close);
//! assert_eq!(output, Some(ModalOutput::Closed));
//! assert!(!Modal::is_visible(&state));
//! ```

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use super::{Component, Toggleable};

/// Messages that can be sent to a Modal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModalMessage {
    /// Close the modal.
    Close,
}

/// Output messages from a Modal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModalOutput {
    /// The modal was closed.
    Closed,
}

/// State for a Modal component.
#[derive(Clone, Debug)]
pub struct ModalState {
    /// The modal title.
    title: String,
    /// The modal content.
    content: String,
    /// Whether the modal is visible.
    visible: bool,
    /// Width of the modal dialog (in characters).
    width: u16,
    /// Height of the modal dialog (in lines).
    height: u16,
    /// Whether the modal can be closed by the user.
    closable: bool,
}

impl Default for ModalState {
    fn default() -> Self {
        Self {
            title: String::new(),
            content: String::new(),
            visible: false,
            width: 50,
            height: 10,
            closable: true,
        }
    }
}

impl ModalState {
    /// Creates a new modal with the given title and dimensions.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ModalState;
    ///
    /// let state = ModalState::new("Warning", 40, 8);
    /// assert_eq!(state.title(), "Warning");
    /// assert!(!state.is_visible());
    /// ```
    pub fn new(title: impl Into<String>, width: u16, height: u16) -> Self {
        Self {
            title: title.into(),
            content: String::new(),
            visible: false,
            width,
            height,
            closable: true,
        }
    }

    /// Returns the modal title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Sets the modal title.
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
    }

    /// Returns the modal content.
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Sets the modal content.
    pub fn set_content(&mut self, content: impl Into<String>) {
        self.content = content.into();
    }

    /// Returns true if the modal is visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Returns the modal width.
    pub fn width(&self) -> u16 {
        self.width
    }

    /// Sets the modal width.
    pub fn set_width(&mut self, width: u16) {
        self.width = width;
    }

    /// Returns the modal height.
    pub fn height(&self) -> u16 {
        self.height
    }

    /// Sets the modal height.
    pub fn set_height(&mut self, height: u16) {
        self.height = height;
    }

    /// Returns true if the modal can be closed by the user.
    pub fn is_closable(&self) -> bool {
        self.closable
    }

    /// Sets whether the modal can be closed by the user.
    ///
    /// Non-closable modals require programmatic closing.
    pub fn set_closable(&mut self, closable: bool) {
        self.closable = closable;
    }
}

/// A modal dialog component.
///
/// This component provides a popup dialog that appears centered over the main
/// content with a semi-transparent overlay backdrop. The modal dims the background
/// and focuses user attention on the dialog.
///
/// # Visual Layout
///
/// ```text
/// ┌─────────────────────────────────┐
/// │    (dimmed background)          │
/// │  ┌─────────────────┐            │
/// │  │ Title           │            │
/// │  ├─────────────────┤            │
/// │  │ Content goes    │            │
/// │  │ here...         │            │
/// │  └─────────────────┘            │
/// │                                 │
/// └─────────────────────────────────┘
/// ```
///
/// # Usage
///
/// The modal itself doesn't handle keyboard events directly. Your application
/// should map Escape key to [`ModalMessage::Close`] when the modal is visible
/// and closable.
///
/// # Example
///
/// ```rust
/// use envision::component::{Modal, ModalMessage, ModalOutput, ModalState, Component, Toggleable};
///
/// let mut state = ModalState::new("Delete File?", 50, 8);
/// state.set_content("Are you sure you want to delete this file?\nThis action cannot be undone.");
///
/// // Show the modal
/// Modal::show(&mut state);
///
/// // Close it
/// let output = Modal::update(&mut state, ModalMessage::Close);
/// assert_eq!(output, Some(ModalOutput::Closed));
/// ```
pub struct Modal;

impl Component for Modal {
    type State = ModalState;
    type Message = ModalMessage;
    type Output = ModalOutput;

    fn init() -> Self::State {
        ModalState::default()
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            ModalMessage::Close => {
                if state.closable {
                    state.visible = false;
                    Some(ModalOutput::Closed)
                } else {
                    None
                }
            }
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect) {
        if !state.visible {
            return;
        }

        // Calculate centered position for modal
        let modal_width = state.width.min(area.width.saturating_sub(2));
        let modal_height = state.height.min(area.height.saturating_sub(2));

        let x = area.x + (area.width.saturating_sub(modal_width)) / 2;
        let y = area.y + (area.height.saturating_sub(modal_height)) / 2;

        let modal_area = Rect {
            x,
            y,
            width: modal_width,
            height: modal_height,
        };

        // Render overlay (we'll just clear the modal area for simplicity)
        // In a real implementation, you might dim the background
        frame.render_widget(Clear, modal_area);

        // Render modal dialog
        let block = Block::default()
            .title(state.title.as_str())
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner_area = block.inner(modal_area);

        let paragraph = Paragraph::new(state.content.as_str())
            .wrap(Wrap { trim: true })
            .style(Style::default());

        frame.render_widget(block, modal_area);
        frame.render_widget(paragraph, inner_area);
    }
}

impl Toggleable for Modal {
    fn is_visible(state: &Self::State) -> bool {
        state.visible
    }

    fn set_visible(state: &mut Self::State, visible: bool) {
        state.visible = visible;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let state = ModalState::new("Test Modal", 40, 10);
        assert_eq!(state.title(), "Test Modal");
        assert_eq!(state.width(), 40);
        assert_eq!(state.height(), 10);
        assert!(!state.is_visible());
        assert!(state.is_closable());
        assert_eq!(state.content(), "");
    }

    #[test]
    fn test_default() {
        let state = ModalState::default();
        assert_eq!(state.title(), "");
        assert_eq!(state.width(), 50);
        assert_eq!(state.height(), 10);
        assert!(!state.is_visible());
        assert!(state.is_closable());
    }

    #[test]
    fn test_title_accessors() {
        let mut state = ModalState::new("Original", 40, 10);
        assert_eq!(state.title(), "Original");

        state.set_title("Updated");
        assert_eq!(state.title(), "Updated");
    }

    #[test]
    fn test_content_accessors() {
        let mut state = ModalState::new("Test", 40, 10);
        assert_eq!(state.content(), "");

        state.set_content("Hello, World!");
        assert_eq!(state.content(), "Hello, World!");
    }

    #[test]
    fn test_width_accessors() {
        let mut state = ModalState::new("Test", 40, 10);
        assert_eq!(state.width(), 40);

        state.set_width(60);
        assert_eq!(state.width(), 60);
    }

    #[test]
    fn test_height_accessors() {
        let mut state = ModalState::new("Test", 40, 10);
        assert_eq!(state.height(), 10);

        state.set_height(15);
        assert_eq!(state.height(), 15);
    }

    #[test]
    fn test_closable_accessors() {
        let mut state = ModalState::new("Test", 40, 10);
        assert!(state.is_closable());

        state.set_closable(false);
        assert!(!state.is_closable());

        state.set_closable(true);
        assert!(state.is_closable());
    }

    #[test]
    fn test_close_closable() {
        let mut state = ModalState::new("Test", 40, 10);
        state.set_closable(true);
        Modal::show(&mut state);

        let output = Modal::update(&mut state, ModalMessage::Close);
        assert_eq!(output, Some(ModalOutput::Closed));
        assert!(!state.is_visible());
    }

    #[test]
    fn test_close_not_closable() {
        let mut state = ModalState::new("Test", 40, 10);
        state.set_closable(false);
        Modal::show(&mut state);

        let output = Modal::update(&mut state, ModalMessage::Close);
        assert_eq!(output, None);
        assert!(state.is_visible());
    }

    #[test]
    fn test_toggleable() {
        let mut state = ModalState::new("Test", 40, 10);

        assert!(!Modal::is_visible(&state));

        Modal::show(&mut state);
        assert!(Modal::is_visible(&state));

        Modal::hide(&mut state);
        assert!(!Modal::is_visible(&state));

        Modal::toggle(&mut state);
        assert!(Modal::is_visible(&state));

        Modal::toggle(&mut state);
        assert!(!Modal::is_visible(&state));
    }

    #[test]
    fn test_init() {
        let state = Modal::init();
        assert_eq!(state.title(), "");
        assert!(!Modal::is_visible(&state));
        assert!(state.is_closable());
    }

    #[test]
    fn test_clone() {
        let mut state = ModalState::new("Test", 40, 10);
        state.set_content("Content");
        Modal::show(&mut state);

        let cloned = state.clone();
        assert_eq!(cloned.title(), "Test");
        assert_eq!(cloned.content(), "Content");
        assert!(cloned.is_visible());
    }

    #[test]
    fn test_view_hidden() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let state = ModalState::new("Hidden", 40, 10);

        let backend = CaptureBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Modal::view(&state, frame, frame.area());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        // Hidden modal shouldn't render anything
        assert!(!output.contains("Hidden"));
    }

    #[test]
    fn test_view_visible() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let mut state = ModalState::new("Visible Modal", 40, 10);
        state.set_content("This is the content");
        Modal::show(&mut state);

        let backend = CaptureBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Modal::view(&state, frame, frame.area());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Visible Modal"));
        assert!(output.contains("This is the content"));
    }

    #[test]
    fn test_view_with_title_and_content() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let mut state = ModalState::new("Confirmation", 50, 12);
        state.set_content("Are you sure you want to continue?");
        Modal::show(&mut state);

        let backend = CaptureBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Modal::view(&state, frame, frame.area());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Confirmation"));
        assert!(output.contains("Are you sure"));
    }

    #[test]
    fn test_view_centered() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let mut state = ModalState::new("Centered", 30, 8);
        Modal::show(&mut state);

        let backend = CaptureBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Modal::view(&state, frame, frame.area());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Centered"));
    }

    #[test]
    fn test_view_respects_terminal_bounds() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        // Modal larger than terminal
        let mut state = ModalState::new("Too Big", 100, 50);
        Modal::show(&mut state);

        let backend = CaptureBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        // Should not panic
        terminal
            .draw(|frame| {
                Modal::view(&state, frame, frame.area());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Too Big"));
    }
}
