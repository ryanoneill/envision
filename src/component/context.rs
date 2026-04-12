//! Render-time and event-time context types for components.
//!
//! This module provides [`RenderContext`] and [`EventContext`], which carry
//! per-render state (frame, area, theme, focus, disabled) into component
//! `view` and `handle_event` methods respectively.

use ratatui::prelude::{Frame, Rect};

use crate::theme::Theme;

/// Context passed to [`Component::handle_event`](crate::component::Component::handle_event).
///
/// Carries focus and disabled state from the parent so the component
/// can decide whether and how to handle events. Use [`RenderContext`]
/// for `view()`.
///
/// # Example
///
/// ```rust
/// use envision::component::EventContext;
///
/// let ctx = EventContext::default();
/// assert!(!ctx.focused);
///
/// let ctx = EventContext::new().focused(true).disabled(false);
/// assert!(ctx.focused);
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EventContext {
    /// Whether this component currently has keyboard focus.
    pub focused: bool,
    /// Whether this component is currently disabled.
    pub disabled: bool,
}

impl EventContext {
    /// Creates a new default EventContext (unfocused, enabled).
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the focused state (builder pattern).
    #[must_use]
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Sets the disabled state (builder pattern).
    #[must_use]
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// Context passed to [`Component::view`](crate::component::Component::view).
///
/// Bundles the frame, area, theme, and focus/disabled state into a
/// single value so component view signatures stay short and adding
/// new render-time fields is non-breaking.
///
/// # Lifetimes
///
/// `RenderContext` carries two lifetime parameters:
/// - `'frame` is the lifetime of the borrow on the [`Frame`] reference.
///   This is the lifetime that shortens during reborrows via [`with_area`](Self::with_area).
/// - `'buf` is the lifetime of the frame's internal buffer (the underlying terminal cells).
///   This stays stable across reborrows.
///
/// Most callers can write `RenderContext<'_, '_>` and let lifetime elision
/// handle both. For example, [`Component::view`](crate::component::Component::view)
/// takes `ctx: &mut RenderContext<'_, '_>`.
///
/// # Example
///
/// ```rust,no_run
/// use envision::component::{Component, RenderContext};
/// use envision::theme::Theme;
/// use envision::backend::CaptureBackend;
/// use ratatui::Terminal;
///
/// let backend = CaptureBackend::new(80, 24);
/// let mut terminal = Terminal::new(backend).unwrap();
/// let theme = Theme::default();
/// terminal.draw(|frame| {
///     let area = frame.area();
///     let mut ctx = RenderContext::new(frame, area, &theme).focused(true);
///     // Pass `&mut ctx` to a component's `view` method.
/// }).unwrap();
/// ```
pub struct RenderContext<'frame, 'buf> {
    /// The ratatui frame to render into.
    pub frame: &'frame mut Frame<'buf>,
    /// The area within the frame to render to.
    pub area: Rect,
    /// The theme to use for styling.
    pub theme: &'frame Theme,
    /// Whether the component currently has keyboard focus.
    pub focused: bool,
    /// Whether the component is currently disabled.
    pub disabled: bool,
}

impl<'frame, 'buf> RenderContext<'frame, 'buf> {
    /// Constructs a new RenderContext with `focused` and `disabled` both `false`.
    pub fn new(frame: &'frame mut Frame<'buf>, area: Rect, theme: &'frame Theme) -> Self {
        Self {
            frame,
            area,
            theme,
            focused: false,
            disabled: false,
        }
    }

    /// Sets the focused state (builder pattern).
    #[must_use]
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Sets the disabled state (builder pattern).
    #[must_use]
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Returns a context with the same frame, theme, and focus state but
    /// a different area.
    ///
    /// The returned context borrows the frame for a shorter lifetime, so
    /// the parent context becomes valid again after the child context
    /// goes out of scope.
    pub fn with_area(&mut self, area: Rect) -> RenderContext<'_, 'buf> {
        RenderContext {
            frame: self.frame,
            area,
            theme: self.theme,
            focused: self.focused,
            disabled: self.disabled,
        }
    }

    /// Convenience: render a widget into this context's area.
    ///
    /// Equivalent to `self.frame.render_widget(widget, self.area)`.
    pub fn render_widget<W: ratatui::widgets::Widget>(&mut self, widget: W) {
        self.frame.render_widget(widget, self.area);
    }

    /// Returns the [`EventContext`] slice of this RenderContext.
    pub fn event_context(&self) -> EventContext {
        EventContext {
            focused: self.focused,
            disabled: self.disabled,
        }
    }
}

impl From<&RenderContext<'_, '_>> for EventContext {
    fn from(ctx: &RenderContext<'_, '_>) -> Self {
        EventContext {
            focused: ctx.focused,
            disabled: ctx.disabled,
        }
    }
}

#[cfg(test)]
mod render_context_tests {
    use super::*;
    use crate::component::test_utils::setup_render;

    #[test]
    fn test_event_context_default() {
        let ctx = EventContext::default();
        assert!(!ctx.focused);
        assert!(!ctx.disabled);
    }

    #[test]
    fn test_event_context_builder() {
        let ctx = EventContext::new().focused(true);
        assert!(ctx.focused);
        assert!(!ctx.disabled);

        let ctx = EventContext::new().focused(true).disabled(true);
        assert!(ctx.focused);
        assert!(ctx.disabled);
    }

    #[test]
    fn test_render_context_construction() {
        let (mut terminal, theme) = setup_render(60, 5);
        terminal
            .draw(|frame| {
                let area = frame.area();
                let ctx = RenderContext::new(frame, area, &theme);
                assert!(!ctx.focused);
                assert!(!ctx.disabled);
                assert_eq!(ctx.area, area);
            })
            .unwrap();
    }

    #[test]
    fn test_render_context_builder() {
        let (mut terminal, theme) = setup_render(60, 5);
        terminal
            .draw(|frame| {
                let area = frame.area();
                let ctx = RenderContext::new(frame, area, &theme)
                    .focused(true)
                    .disabled(true);
                assert!(ctx.focused);
                assert!(ctx.disabled);
            })
            .unwrap();
    }

    #[test]
    fn test_render_context_with_area() {
        use ratatui::widgets::Paragraph;
        let (mut terminal, theme) = setup_render(60, 10);
        terminal
            .draw(|frame| {
                let parent_area = frame.area();
                let mut ctx = RenderContext::new(frame, parent_area, &theme).focused(true);
                let parent_theme_ptr = ctx.theme as *const Theme;
                let child_area = ratatui::layout::Rect::new(5, 2, 20, 3);
                {
                    let mut child_ctx = ctx.with_area(child_area);
                    assert_eq!(child_ctx.area, child_area);
                    assert!(child_ctx.focused);
                    // Verify child shares parent's theme reference (pointer equality)
                    assert_eq!(child_ctx.theme as *const Theme, parent_theme_ptr);
                    child_ctx.render_widget(Paragraph::new("child"));
                }
                // Critical: render through the parent ctx after child scope.
                // This line would NOT compile if the reborrow were broken.
                ctx.render_widget(Paragraph::new("parent"));
                assert_eq!(ctx.area, parent_area);
                assert!(ctx.focused);
            })
            .unwrap();
    }

    #[test]
    fn test_render_context_render_widget() {
        use ratatui::widgets::Paragraph;
        let (mut terminal, theme) = setup_render(60, 5);
        terminal
            .draw(|frame| {
                let area = frame.area();
                let mut ctx = RenderContext::new(frame, area, &theme);
                ctx.render_widget(Paragraph::new("hello"));
            })
            .unwrap();

        let display = terminal.backend().to_string();
        assert!(display.contains("hello"));
    }

    #[test]
    fn test_event_context_from_render_context() {
        let (mut terminal, theme) = setup_render(60, 5);
        terminal
            .draw(|frame| {
                let area = frame.area();
                let ctx = RenderContext::new(frame, area, &theme)
                    .focused(true)
                    .disabled(false);
                let event_ctx: EventContext = (&ctx).into();
                assert!(event_ctx.focused);
                assert!(!event_ctx.disabled);
            })
            .unwrap();
    }

    #[test]
    fn test_render_context_event_context_method() {
        let (mut terminal, theme) = setup_render(60, 5);
        terminal
            .draw(|frame| {
                let area = frame.area();
                let ctx = RenderContext::new(frame, area, &theme)
                    .focused(true)
                    .disabled(true);
                let event_ctx = ctx.event_context();
                assert!(event_ctx.focused);
                assert!(event_ctx.disabled);
            })
            .unwrap();
    }
}
