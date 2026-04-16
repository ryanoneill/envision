//! An indeterminate loading indicator component.
//!
//! [`Spinner`] provides a visual activity indicator that animates through frames
//! to show ongoing activity. This is a **display-only** component that does not
//! receive keyboard focus. State is stored in [`SpinnerState`] and updated via
//! [`SpinnerMessage`].
//!
//! See also [`ProgressBar`](super::ProgressBar) for determinate progress,
//! and [`MultiProgress`](super::MultiProgress) for tracking multiple tasks.
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

use ratatui::widgets::Paragraph;

use super::{Component, RenderContext};

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
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SpinnerStyle;
    ///
    /// assert_eq!(SpinnerStyle::Line.frames(), &['|', '/', '-', '\\']);
    /// ```
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SpinnerStyle;
    ///
    /// assert_eq!(SpinnerStyle::Line.frame_count(), 4);
    /// assert_eq!(SpinnerStyle::Dots.frame_count(), 10);
    /// ```
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
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct SpinnerState {
    /// The animation style.
    style: SpinnerStyle,
    /// The current frame index.
    frame: usize,
    /// Whether the spinner is currently animating.
    spinning: bool,
    /// Optional label displayed next to the spinner.
    label: Option<String>,
    /// Whether the component is disabled.
    disabled: bool,
}

impl Default for SpinnerState {
    fn default() -> Self {
        Self {
            style: SpinnerStyle::default(),
            frame: 0,
            spinning: true,
            label: None,
            disabled: false,
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Spinner, SpinnerMessage, SpinnerState, Component};
    ///
    /// let mut state = SpinnerState::new();
    /// let first = state.current_frame();
    /// Spinner::update(&mut state, SpinnerMessage::Tick);
    /// let second = state.current_frame();
    /// assert_ne!(first, second);
    /// ```
    pub fn current_frame(&self) -> char {
        let frames = self.style.frames();
        frames[self.frame % frames.len()]
    }

    /// Returns true if the spinner is currently spinning.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Spinner, SpinnerMessage, SpinnerState, Component};
    ///
    /// let mut state = SpinnerState::new();
    /// assert!(state.is_spinning());
    /// Spinner::update(&mut state, SpinnerMessage::Stop);
    /// assert!(!state.is_spinning());
    /// ```
    pub fn is_spinning(&self) -> bool {
        self.spinning
    }

    /// Sets whether the spinner is spinning.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SpinnerState;
    ///
    /// let mut state = SpinnerState::new();
    /// state.set_spinning(false);
    /// assert!(!state.is_spinning());
    /// ```
    pub fn set_spinning(&mut self, spinning: bool) {
        self.spinning = spinning;
    }

    /// Returns the label if set.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SpinnerState;
    ///
    /// let state = SpinnerState::with_label("Uploading...");
    /// assert_eq!(state.label(), Some("Uploading..."));
    /// ```
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    /// Sets the label.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SpinnerState;
    ///
    /// let mut state = SpinnerState::new();
    /// state.set_label(Some("Processing...".to_string()));
    /// assert_eq!(state.label(), Some("Processing..."));
    /// ```
    pub fn set_label(&mut self, label: Option<String>) {
        self.label = label;
    }

    /// Returns the spinner style.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{SpinnerState, SpinnerStyle};
    ///
    /// let state = SpinnerState::with_style(SpinnerStyle::Circle);
    /// assert_eq!(state.style(), &SpinnerStyle::Circle);
    /// ```
    pub fn style(&self) -> &SpinnerStyle {
        &self.style
    }

    /// Sets the spinner style.
    ///
    /// This resets the frame index to 0.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{SpinnerState, SpinnerStyle};
    ///
    /// let mut state = SpinnerState::new();
    /// state.set_style(SpinnerStyle::Bounce);
    /// assert_eq!(state.style(), &SpinnerStyle::Bounce);
    /// assert_eq!(state.frame_index(), 0);
    /// ```
    pub fn set_style(&mut self, style: SpinnerStyle) {
        self.style = style;
        self.frame = 0;
    }

    /// Returns the current frame index.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Spinner, SpinnerMessage, SpinnerState, Component};
    ///
    /// let mut state = SpinnerState::new();
    /// assert_eq!(state.frame_index(), 0);
    /// Spinner::update(&mut state, SpinnerMessage::Tick);
    /// assert_eq!(state.frame_index(), 1);
    /// ```
    pub fn frame_index(&self) -> usize {
        self.frame
    }

    /// Returns true if the spinner is disabled.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SpinnerState;
    ///
    /// let state = SpinnerState::new();
    /// assert!(!state.is_disabled());
    /// ```
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Sets the disabled state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SpinnerState;
    ///
    /// let mut state = SpinnerState::new();
    /// state.set_disabled(true);
    /// assert!(state.is_disabled());
    /// ```
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    /// Sets the disabled state using builder pattern.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SpinnerState;
    ///
    /// let state = SpinnerState::new()
    ///     .with_disabled(true);
    /// assert!(state.is_disabled());
    /// ```
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
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

    fn view(state: &Self::State, ctx: &mut RenderContext<'_, '_>) {
        let spinner_char = if state.spinning {
            state.current_frame().to_string()
        } else {
            " ".to_string()
        };

        let text = match &state.label {
            Some(label) => format!("{} {}", spinner_char, label),
            None => spinner_char,
        };

        let paragraph = Paragraph::new(text).style(ctx.theme.info_style());

        let annotation = crate::annotation::Annotation::spinner("spinner")
            .with_label(state.label.as_deref().unwrap_or(""));
        let annotated = crate::annotation::Annotate::new(paragraph, annotation);
        ctx.frame.render_widget(annotated, ctx.area);
    }
}

#[cfg(test)]
mod tests;
