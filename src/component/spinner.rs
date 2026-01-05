//! An indeterminate loading indicator component.
//!
//! `Spinner` provides a visual activity indicator that animates through frames
//! to show ongoing activity. This is a **display-only** component that does not
//! receive keyboard focus.
//!
//! # Animation Model
//!
//! The spinner does not animate itself - the parent application sends `Tick`
//! messages to advance the animation. This fits the TEA pattern where external
//! events drive state changes. Typical usage involves a timer subscription
//! sending Tick every 80-100ms.
//!
//! # Example
//!
//! ```rust
//! use envision::component::{Spinner, SpinnerMessage, SpinnerState, SpinnerStyle, Component};
//!
//! // Create a spinner with label
//! let mut state = SpinnerState::with_label("Loading...");
//! assert!(state.is_spinning());
//!
//! // Animate by sending Tick messages (typically from a timer)
//! Spinner::update(&mut state, SpinnerMessage::Tick);
//! let frame1 = state.current_frame();
//!
//! Spinner::update(&mut state, SpinnerMessage::Tick);
//! let frame2 = state.current_frame();
//!
//! assert_ne!(frame1, frame2); // Frame advanced
//!
//! // Stop/start the spinner
//! Spinner::update(&mut state, SpinnerMessage::Stop);
//! assert!(!state.is_spinning());
//! ```

use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use super::Component;
use crate::theme::Theme;

/// Built-in spinner animation styles.
///
/// Each style provides a sequence of characters that cycle to create
/// an animation effect.
///
/// # Example
///
/// ```rust
/// use envision::component::SpinnerStyle;
///
/// let style = SpinnerStyle::Dots;
/// assert_eq!(style.frames().len(), 10);
///
/// let style = SpinnerStyle::Line;
/// assert_eq!(style.frames().len(), 4);
///
/// let custom = SpinnerStyle::Custom(vec!['◯', '◔', '◑', '◕', '●']);
/// assert_eq!(custom.frames().len(), 5);
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum SpinnerStyle {
    /// Braille dots animation (⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏).
    ///
    /// 10 frames, smooth circular motion.
    #[default]
    Dots,
    /// Classic line animation (|/-\\).
    ///
    /// 4 frames, ASCII-compatible.
    Line,
    /// Quarter circle animation (◐◓◑◒).
    ///
    /// 4 frames, rotating circle segments.
    Circle,
    /// Bouncing dot animation (⠁⠂⠄⠂).
    ///
    /// 4 frames, vertical bouncing effect.
    Bounce,
    /// Custom animation frames.
    ///
    /// Provide your own sequence of characters.
    Custom(Vec<char>),
}

impl SpinnerStyle {
    /// Returns the animation frames for this style.
    ///
    /// For `Custom` styles, returns the provided frames.
    /// For empty `Custom` styles, returns a single space character.
    pub fn frames(&self) -> &[char] {
        // Static arrays for built-in styles
        const DOTS: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
        const LINE: &[char] = &['|', '/', '-', '\\'];
        const CIRCLE: &[char] = &['◐', '◓', '◑', '◒'];
        const BOUNCE: &[char] = &['⠁', '⠂', '⠄', '⠂'];
        const EMPTY: &[char] = &[' '];

        match self {
            SpinnerStyle::Dots => DOTS,
            SpinnerStyle::Line => LINE,
            SpinnerStyle::Circle => CIRCLE,
            SpinnerStyle::Bounce => BOUNCE,
            SpinnerStyle::Custom(frames) => {
                if frames.is_empty() {
                    EMPTY
                } else {
                    frames
                }
            }
        }
    }

    /// Returns the number of frames in this style.
    pub fn frame_count(&self) -> usize {
        self.frames().len()
    }
}

/// Messages that can be sent to a Spinner.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpinnerMessage {
    /// Advance to the next animation frame.
    ///
    /// Only advances if the spinner is currently spinning.
    Tick,
    /// Start spinning (if stopped).
    Start,
    /// Stop spinning.
    Stop,
}

/// State for a Spinner component.
#[derive(Clone, Debug)]
pub struct SpinnerState {
    /// The animation style.
    style: SpinnerStyle,
    /// The current frame index.
    frame: usize,
    /// Whether the spinner is currently animating.
    spinning: bool,
    /// Optional label displayed next to the spinner.
    label: Option<String>,
}

impl Default for SpinnerState {
    fn default() -> Self {
        Self {
            style: SpinnerStyle::default(),
            frame: 0,
            spinning: true,
            label: None,
        }
    }
}

impl SpinnerState {
    /// Creates a new spinning spinner with the default Dots style.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SpinnerState;
    ///
    /// let state = SpinnerState::new();
    /// assert!(state.is_spinning());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a spinner with the given style.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{SpinnerState, SpinnerStyle};
    ///
    /// let state = SpinnerState::with_style(SpinnerStyle::Line);
    /// assert_eq!(state.style(), &SpinnerStyle::Line);
    /// ```
    pub fn with_style(style: SpinnerStyle) -> Self {
        Self {
            style,
            ..Self::default()
        }
    }

    /// Creates a spinner with a label.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SpinnerState;
    ///
    /// let state = SpinnerState::with_label("Loading...");
    /// assert_eq!(state.label(), Some("Loading..."));
    /// ```
    pub fn with_label(label: impl Into<String>) -> Self {
        Self {
            label: Some(label.into()),
            ..Self::default()
        }
    }

    /// Returns the current frame character.
    ///
    /// This is the character that should be displayed for the current
    /// animation state.
    pub fn current_frame(&self) -> char {
        let frames = self.style.frames();
        frames[self.frame % frames.len()]
    }

    /// Returns true if the spinner is currently spinning.
    pub fn is_spinning(&self) -> bool {
        self.spinning
    }

    /// Sets whether the spinner is spinning.
    pub fn set_spinning(&mut self, spinning: bool) {
        self.spinning = spinning;
    }

    /// Returns the label if set.
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    /// Sets the label.
    pub fn set_label(&mut self, label: Option<String>) {
        self.label = label;
    }

    /// Returns the spinner style.
    pub fn style(&self) -> &SpinnerStyle {
        &self.style
    }

    /// Sets the spinner style.
    ///
    /// This resets the frame index to 0.
    pub fn set_style(&mut self, style: SpinnerStyle) {
        self.style = style;
        self.frame = 0;
    }

    /// Returns the current frame index.
    pub fn frame_index(&self) -> usize {
        self.frame
    }
}

/// An indeterminate loading indicator component.
///
/// `Spinner` displays an animated indicator to show ongoing activity.
/// Unlike [`ProgressBar`](super::ProgressBar), it does not show specific
/// progress - just that something is happening.
///
/// # Animation
///
/// The spinner advances one frame each time it receives a `Tick` message.
/// Your application should send Tick messages at regular intervals
/// (typically every 80-100ms) to create smooth animation.
///
/// # Styles
///
/// Several built-in styles are available:
/// - `Dots`: Braille dots (⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏) - default
/// - `Line`: Classic ASCII (|/-\\)
/// - `Circle`: Quarter circles (◐◓◑◒)
/// - `Bounce`: Bouncing dot (⠁⠂⠄⠂)
/// - `Custom`: Your own character sequence
///
/// # Example
///
/// ```rust
/// use envision::component::{Spinner, SpinnerMessage, SpinnerState, Component};
///
/// let mut state = SpinnerState::with_label("Processing...");
///
/// // In your app's update function, forward timer ticks:
/// Spinner::update(&mut state, SpinnerMessage::Tick);
///
/// // Stop when done:
/// Spinner::update(&mut state, SpinnerMessage::Stop);
/// ```
pub struct Spinner;

impl Component for Spinner {
    type State = SpinnerState;
    type Message = SpinnerMessage;
    type Output = ();

    fn init() -> Self::State {
        SpinnerState::default()
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            SpinnerMessage::Tick => {
                if state.spinning {
                    let frame_count = state.style.frame_count();
                    state.frame = (state.frame + 1) % frame_count;
                }
            }
            SpinnerMessage::Start => {
                state.spinning = true;
            }
            SpinnerMessage::Stop => {
                state.spinning = false;
            }
        }
        None
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme) {
        let spinner_char = if state.spinning {
            state.current_frame().to_string()
        } else {
            " ".to_string()
        };

        let text = match &state.label {
            Some(label) => format!("{} {}", spinner_char, label),
            None => spinner_char,
        };

        let paragraph = Paragraph::new(text).style(theme.info_style());

        frame.render_widget(paragraph, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let state = SpinnerState::new();
        assert!(state.is_spinning());
        assert_eq!(state.style(), &SpinnerStyle::Dots);
        assert_eq!(state.frame_index(), 0);
        assert!(state.label().is_none());
    }

    #[test]
    fn test_default() {
        let state = SpinnerState::default();
        assert!(state.is_spinning());
        assert_eq!(state.style(), &SpinnerStyle::Dots);
    }

    #[test]
    fn test_with_style() {
        let state = SpinnerState::with_style(SpinnerStyle::Line);
        assert_eq!(state.style(), &SpinnerStyle::Line);
        assert!(state.is_spinning());
    }

    #[test]
    fn test_with_label() {
        let state = SpinnerState::with_label("Loading...");
        assert_eq!(state.label(), Some("Loading..."));
        assert!(state.is_spinning());
    }

    #[test]
    fn test_is_spinning() {
        let state = SpinnerState::new();
        assert!(state.is_spinning());
    }

    #[test]
    fn test_spinning_accessors() {
        let mut state = SpinnerState::new();
        assert!(state.is_spinning());

        state.set_spinning(false);
        assert!(!state.is_spinning());

        state.set_spinning(true);
        assert!(state.is_spinning());
    }

    #[test]
    fn test_label_accessors() {
        let mut state = SpinnerState::new();
        assert!(state.label().is_none());

        state.set_label(Some("Test".to_string()));
        assert_eq!(state.label(), Some("Test"));

        state.set_label(None);
        assert!(state.label().is_none());
    }

    #[test]
    fn test_style_accessors() {
        let mut state = SpinnerState::new();
        assert_eq!(state.style(), &SpinnerStyle::Dots);

        state.set_style(SpinnerStyle::Circle);
        assert_eq!(state.style(), &SpinnerStyle::Circle);
    }

    #[test]
    fn test_current_frame() {
        let state = SpinnerState::new();
        // First frame of Dots is '⠋'
        assert_eq!(state.current_frame(), '⠋');
    }

    #[test]
    fn test_tick_advances_frame() {
        let mut state = SpinnerState::new();
        assert_eq!(state.frame_index(), 0);
        assert_eq!(state.current_frame(), '⠋');

        Spinner::update(&mut state, SpinnerMessage::Tick);
        assert_eq!(state.frame_index(), 1);
        assert_eq!(state.current_frame(), '⠙');

        Spinner::update(&mut state, SpinnerMessage::Tick);
        assert_eq!(state.frame_index(), 2);
        assert_eq!(state.current_frame(), '⠹');
    }

    #[test]
    fn test_tick_wraps_around() {
        let mut state = SpinnerState::with_style(SpinnerStyle::Line);
        // Line has 4 frames: |, /, -, \

        Spinner::update(&mut state, SpinnerMessage::Tick); // 1
        Spinner::update(&mut state, SpinnerMessage::Tick); // 2
        Spinner::update(&mut state, SpinnerMessage::Tick); // 3
        assert_eq!(state.frame_index(), 3);

        Spinner::update(&mut state, SpinnerMessage::Tick); // Wraps to 0
        assert_eq!(state.frame_index(), 0);
        assert_eq!(state.current_frame(), '|');
    }

    #[test]
    fn test_tick_when_stopped() {
        let mut state = SpinnerState::new();
        state.set_spinning(false);

        let initial_frame = state.frame_index();
        Spinner::update(&mut state, SpinnerMessage::Tick);
        assert_eq!(state.frame_index(), initial_frame); // No change
    }

    #[test]
    fn test_start_stop() {
        let mut state = SpinnerState::new();
        assert!(state.is_spinning());

        Spinner::update(&mut state, SpinnerMessage::Stop);
        assert!(!state.is_spinning());

        Spinner::update(&mut state, SpinnerMessage::Start);
        assert!(state.is_spinning());
    }

    #[test]
    fn test_style_frames_dots() {
        let style = SpinnerStyle::Dots;
        assert_eq!(style.frame_count(), 10);
        assert_eq!(
            style.frames(),
            &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏']
        );
    }

    #[test]
    fn test_style_frames_line() {
        let style = SpinnerStyle::Line;
        assert_eq!(style.frame_count(), 4);
        assert_eq!(style.frames(), &['|', '/', '-', '\\']);
    }

    #[test]
    fn test_style_frames_circle() {
        let style = SpinnerStyle::Circle;
        assert_eq!(style.frame_count(), 4);
        assert_eq!(style.frames(), &['◐', '◓', '◑', '◒']);
    }

    #[test]
    fn test_style_frames_bounce() {
        let style = SpinnerStyle::Bounce;
        assert_eq!(style.frame_count(), 4);
        assert_eq!(style.frames(), &['⠁', '⠂', '⠄', '⠂']);
    }

    #[test]
    fn test_custom_style() {
        let custom = SpinnerStyle::Custom(vec!['◯', '◔', '◑', '◕', '●']);
        assert_eq!(custom.frame_count(), 5);
        assert_eq!(custom.frames(), &['◯', '◔', '◑', '◕', '●']);
    }

    #[test]
    fn test_custom_style_empty() {
        let custom = SpinnerStyle::Custom(vec![]);
        assert_eq!(custom.frame_count(), 1);
        assert_eq!(custom.frames(), &[' ']);
    }

    #[test]
    fn test_set_style_resets_frame() {
        let mut state = SpinnerState::new();

        // Advance a few frames
        Spinner::update(&mut state, SpinnerMessage::Tick);
        Spinner::update(&mut state, SpinnerMessage::Tick);
        assert_eq!(state.frame_index(), 2);

        // Change style
        state.set_style(SpinnerStyle::Line);
        assert_eq!(state.frame_index(), 0);
    }

    #[test]
    fn test_clone() {
        let mut state = SpinnerState::with_label("Test");
        Spinner::update(&mut state, SpinnerMessage::Tick);

        let cloned = state.clone();
        assert_eq!(cloned.label(), Some("Test"));
        assert_eq!(cloned.frame_index(), 1);
        assert!(cloned.is_spinning());
    }

    #[test]
    fn test_init() {
        let state = Spinner::init();
        assert!(state.is_spinning());
        assert_eq!(state.style(), &SpinnerStyle::Dots);
        assert!(state.label().is_none());
    }

    #[test]
    fn test_update_returns_none() {
        let mut state = SpinnerState::new();

        assert_eq!(Spinner::update(&mut state, SpinnerMessage::Tick), None);
        assert_eq!(Spinner::update(&mut state, SpinnerMessage::Start), None);
        assert_eq!(Spinner::update(&mut state, SpinnerMessage::Stop), None);
    }

    #[test]
    fn test_view_spinning() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let state = SpinnerState::new();

        let backend = CaptureBackend::new(40, 5);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Spinner::view(&state, frame, frame.area(), &Theme::default());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        // First frame of Dots is '⠋'
        assert!(output.contains('⠋'));
    }

    #[test]
    fn test_view_stopped() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let mut state = SpinnerState::new();
        state.set_spinning(false);

        let backend = CaptureBackend::new(40, 5);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Spinner::view(&state, frame, frame.area(), &Theme::default());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        // Should not contain spinner character when stopped
        assert!(!output.contains('⠋'));
    }

    #[test]
    fn test_view_with_label() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let state = SpinnerState::with_label("Loading");

        let backend = CaptureBackend::new(40, 5);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Spinner::view(&state, frame, frame.area(), &Theme::default());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Loading"));
        assert!(output.contains('⠋'));
    }

    #[test]
    fn test_style_default() {
        let style = SpinnerStyle::default();
        assert_eq!(style, SpinnerStyle::Dots);
    }

    #[test]
    fn test_full_animation_cycle() {
        let mut state = SpinnerState::with_style(SpinnerStyle::Line);

        let mut frames_seen = Vec::new();
        for _ in 0..8 {
            // Two full cycles
            frames_seen.push(state.current_frame());
            Spinner::update(&mut state, SpinnerMessage::Tick);
        }

        // Should cycle through |, /, -, \ twice
        assert_eq!(frames_seen, vec!['|', '/', '-', '\\', '|', '/', '-', '\\']);
    }
}
