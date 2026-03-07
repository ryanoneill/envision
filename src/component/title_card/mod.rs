//! A stylish display-only component for application titles.
//!
//! `TitleCard` provides a centered title display with optional emoji
//! prefix/suffix, subtitle, configurable styles, and borders.
//!
//! # Example
//!
//! ```rust
//! use envision::component::{Component, TitleCard, TitleCardState, TitleCardMessage};
//! use ratatui::style::{Color, Modifier, Style};
//!
//! let state = TitleCardState::new("My App")
//!     .with_subtitle("A TUI Application")
//!     .with_prefix("🚀 ")
//!     .with_suffix(" ✨");
//!
//! assert_eq!(state.title(), "My App");
//! assert_eq!(state.subtitle(), Some("A TUI Application"));
//! assert_eq!(state.prefix(), Some("🚀 "));
//! assert_eq!(state.suffix(), Some(" ✨"));
//! ```

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use super::Component;
use crate::theme::Theme;

/// Messages that can be sent to a TitleCard.
#[derive(Clone, Debug, PartialEq)]
pub enum TitleCardMessage {
    /// Set the title text.
    SetTitle(String),
    /// Set the subtitle text.
    SetSubtitle(Option<String>),
    /// Set the prefix decoration.
    SetPrefix(Option<String>),
    /// Set the suffix decoration.
    SetSuffix(Option<String>),
    /// Set the title style.
    SetTitleStyle(Style),
    /// Set the subtitle style.
    SetSubtitleStyle(Style),
}

/// State for a TitleCard component.
///
/// Contains the title text, optional decorations, and styling configuration.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct TitleCardState {
    /// The title text.
    title: String,
    /// Optional subtitle displayed below the title.
    subtitle: Option<String>,
    /// Optional prefix displayed before the title (e.g., emoji).
    prefix: Option<String>,
    /// Optional suffix displayed after the title (e.g., emoji).
    suffix: Option<String>,
    /// Style for the title text.
    title_style: Style,
    /// Style for the subtitle text.
    subtitle_style: Style,
    /// Whether to show a border.
    bordered: bool,
    /// Whether the component is disabled.
    disabled: bool,
}

impl Default for TitleCardState {
    fn default() -> Self {
        Self {
            title: String::new(),
            subtitle: None,
            prefix: None,
            suffix: None,
            title_style: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            subtitle_style: Style::default().fg(Color::DarkGray),
            bordered: true,
            disabled: false,
        }
    }
}

impl TitleCardState {
    /// Creates a new title card with the given title.
    ///
    /// The default title style is Cyan + Bold, and the default subtitle
    /// style is DarkGray.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TitleCardState;
    ///
    /// let state = TitleCardState::new("My App");
    /// assert_eq!(state.title(), "My App");
    /// assert!(state.is_bordered());
    /// ```
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            ..Self::default()
        }
    }

    // ---- Builders ----

    /// Sets the subtitle (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TitleCardState;
    ///
    /// let state = TitleCardState::new("App")
    ///     .with_subtitle("Version 1.0");
    /// assert_eq!(state.subtitle(), Some("Version 1.0"));
    /// ```
    pub fn with_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Sets the prefix decoration (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TitleCardState;
    ///
    /// let state = TitleCardState::new("App")
    ///     .with_prefix("📚 ");
    /// assert_eq!(state.prefix(), Some("📚 "));
    /// ```
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// Sets the suffix decoration (builder pattern).
    pub fn with_suffix(mut self, suffix: impl Into<String>) -> Self {
        self.suffix = Some(suffix.into());
        self
    }

    /// Sets the title style (builder pattern).
    pub fn with_title_style(mut self, style: Style) -> Self {
        self.title_style = style;
        self
    }

    /// Sets the subtitle style (builder pattern).
    pub fn with_subtitle_style(mut self, style: Style) -> Self {
        self.subtitle_style = style;
        self
    }

    /// Sets whether to show a border (builder pattern).
    pub fn with_bordered(mut self, bordered: bool) -> Self {
        self.bordered = bordered;
        self
    }

    /// Sets the disabled state (builder pattern).
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    // ---- Getters ----

    /// Returns the title text.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the subtitle text.
    pub fn subtitle(&self) -> Option<&str> {
        self.subtitle.as_deref()
    }

    /// Returns the prefix decoration.
    pub fn prefix(&self) -> Option<&str> {
        self.prefix.as_deref()
    }

    /// Returns the suffix decoration.
    pub fn suffix(&self) -> Option<&str> {
        self.suffix.as_deref()
    }

    /// Returns the title style.
    pub fn title_style(&self) -> Style {
        self.title_style
    }

    /// Returns the subtitle style.
    pub fn subtitle_style(&self) -> Style {
        self.subtitle_style
    }

    /// Returns whether the border is shown.
    pub fn is_bordered(&self) -> bool {
        self.bordered
    }

    /// Returns whether the component is disabled.
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    // ---- Setters ----

    /// Sets the title text.
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
    }

    /// Sets the subtitle text.
    pub fn set_subtitle(&mut self, subtitle: Option<String>) {
        self.subtitle = subtitle;
    }

    /// Sets the prefix decoration.
    pub fn set_prefix(&mut self, prefix: Option<String>) {
        self.prefix = prefix;
    }

    /// Sets the suffix decoration.
    pub fn set_suffix(&mut self, suffix: Option<String>) {
        self.suffix = suffix;
    }

    /// Sets the title style.
    pub fn set_title_style(&mut self, style: Style) {
        self.title_style = style;
    }

    /// Sets the subtitle style.
    pub fn set_subtitle_style(&mut self, style: Style) {
        self.subtitle_style = style;
    }

    /// Sets whether to show a border.
    pub fn set_bordered(&mut self, bordered: bool) {
        self.bordered = bordered;
    }

    /// Sets the disabled state.
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }
}

/// A stylish display-only component for application titles.
///
/// Renders a centered title with optional decorations, subtitle, and borders.
/// This is a display-only component and does not implement [`Focusable`](super::Focusable).
///
/// # Example
///
/// ```rust
/// use envision::component::{Component, TitleCard, TitleCardState};
///
/// let state = TitleCardState::new("My Application")
///     .with_subtitle("v1.0.0")
///     .with_prefix("🎯 ");
///
/// assert_eq!(state.title(), "My Application");
/// assert_eq!(state.subtitle(), Some("v1.0.0"));
/// ```
pub struct TitleCard;

impl Component for TitleCard {
    type State = TitleCardState;
    type Message = TitleCardMessage;
    type Output = ();

    fn init() -> Self::State {
        TitleCardState::default()
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            TitleCardMessage::SetTitle(title) => state.title = title,
            TitleCardMessage::SetSubtitle(subtitle) => state.subtitle = subtitle,
            TitleCardMessage::SetPrefix(prefix) => state.prefix = prefix,
            TitleCardMessage::SetSuffix(suffix) => state.suffix = suffix,
            TitleCardMessage::SetTitleStyle(style) => state.title_style = style,
            TitleCardMessage::SetSubtitleStyle(style) => state.subtitle_style = style,
        }
        None
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme) {
        crate::annotation::with_registry(|reg| {
            reg.register(
                area,
                crate::annotation::Annotation::title_card("title_card")
                    .with_label(state.title.as_str())
                    .with_disabled(state.disabled),
            );
        });

        let render_area = if state.bordered {
            let border_style = if state.disabled {
                theme.disabled_style()
            } else {
                theme.border_style()
            };

            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(border_style);

            let inner = block.inner(area);
            frame.render_widget(block, area);
            inner
        } else {
            area
        };

        if render_area.height == 0 || render_area.width == 0 {
            return;
        }

        // Build title line with optional prefix and suffix
        let title_style = if state.disabled {
            theme.disabled_style()
        } else {
            state.title_style
        };

        let subtitle_style = if state.disabled {
            theme.disabled_style()
        } else {
            state.subtitle_style
        };

        let mut title_spans = Vec::new();
        if let Some(prefix) = &state.prefix {
            title_spans.push(Span::styled(prefix.as_str(), title_style));
        }
        title_spans.push(Span::styled(state.title.as_str(), title_style));
        if let Some(suffix) = &state.suffix {
            title_spans.push(Span::styled(suffix.as_str(), title_style));
        }

        let title_line = Line::from(title_spans);

        // Calculate vertical centering
        let content_height = if state.subtitle.is_some() { 2 } else { 1 };
        let vertical_offset = render_area.height.saturating_sub(content_height) / 2;

        // Render title
        let title_area = Rect::new(
            render_area.x,
            render_area.y + vertical_offset,
            render_area.width,
            1.min(render_area.height.saturating_sub(vertical_offset)),
        );

        if title_area.height > 0 {
            let title_paragraph = Paragraph::new(title_line).alignment(Alignment::Center);
            frame.render_widget(title_paragraph, title_area);
        }

        // Render subtitle if present
        if let Some(subtitle) = &state.subtitle {
            let subtitle_y = render_area.y + vertical_offset + 1;
            if subtitle_y < render_area.y + render_area.height {
                let subtitle_area = Rect::new(render_area.x, subtitle_y, render_area.width, 1);

                let subtitle_paragraph =
                    Paragraph::new(Span::styled(subtitle.as_str(), subtitle_style))
                        .alignment(Alignment::Center);
                frame.render_widget(subtitle_paragraph, subtitle_area);
            }
        }
    }
}

#[cfg(test)]
mod tests;
