//! Widget wrapper for adding annotations.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;

use super::annotation::Annotation;

/// A wrapper that adds annotation metadata to a widget.
///
/// When rendered, the annotation is registered with the current
/// annotation context (if one is active).
///
/// # Example
///
/// ```rust
/// use envision::annotation::{Annotate, Annotation};
/// use ratatui::widgets::Paragraph;
///
/// // Wrap a widget with annotation
/// let annotated = Annotate::new(
///     Paragraph::new("Click me"),
///     Annotation::button("my-button").with_label("My Button"),
/// );
/// ```
pub struct Annotate<W> {
    widget: W,
    annotation: Annotation,
}

impl<W> Annotate<W> {
    /// Creates a new annotated widget.
    pub fn new(widget: W, annotation: Annotation) -> Self {
        Self { widget, annotation }
    }

    /// Returns a reference to the annotation.
    pub fn annotation(&self) -> &Annotation {
        &self.annotation
    }

    /// Returns a mutable reference to the annotation.
    pub fn annotation_mut(&mut self) -> &mut Annotation {
        &mut self.annotation
    }

    /// Returns a reference to the inner widget.
    pub fn inner(&self) -> &W {
        &self.widget
    }

    /// Returns a mutable reference to the inner widget.
    pub fn inner_mut(&mut self) -> &mut W {
        &mut self.widget
    }

    /// Unwraps and returns the inner widget.
    pub fn into_inner(self) -> W {
        self.widget
    }

    /// Sets the focused state.
    pub fn focused(mut self, focused: bool) -> Self {
        self.annotation.focused = focused;
        self
    }

    /// Sets the disabled state.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.annotation.disabled = disabled;
        self
    }

    /// Sets the selected state.
    pub fn selected(mut self, selected: bool) -> Self {
        self.annotation.selected = selected;
        self
    }

    /// Sets the current value.
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.annotation.value = Some(value.into());
        self
    }
}

impl<W: Widget> Widget for Annotate<W> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Register annotation with context if available
        ANNOTATION_CONTEXT.with(|ctx| {
            if let Some(registry) = ctx.borrow_mut().as_mut() {
                registry.register(area, self.annotation.clone());
            }
        });

        // Render the inner widget
        self.widget.render(area, buf);
    }
}

/// A container annotation that can hold child widgets.
///
/// This opens an annotation scope for nested widgets.
pub struct AnnotateContainer<W> {
    widget: W,
    annotation: Annotation,
}

impl<W> AnnotateContainer<W> {
    /// Creates a new annotated container.
    pub fn new(widget: W, annotation: Annotation) -> Self {
        Self { widget, annotation }
    }
}

impl<W: Widget> Widget for AnnotateContainer<W> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Open annotation scope
        ANNOTATION_CONTEXT.with(|ctx| {
            if let Some(registry) = ctx.borrow_mut().as_mut() {
                registry.open(area, self.annotation.clone());
            }
        });

        // Render the inner widget
        self.widget.render(area, buf);

        // Close annotation scope
        ANNOTATION_CONTEXT.with(|ctx| {
            if let Some(registry) = ctx.borrow_mut().as_mut() {
                registry.close();
            }
        });
    }
}

// Thread-local annotation context for collecting annotations during rendering
use std::cell::RefCell;
use super::registry::AnnotationRegistry;

thread_local! {
    static ANNOTATION_CONTEXT: RefCell<Option<AnnotationRegistry>> = const { RefCell::new(None) };
}

/// Sets up an annotation context for collecting annotations during rendering.
///
/// Returns the collected annotations when done.
///
/// # Example
///
/// ```rust,no_run
/// use envision::annotation::{with_annotations, Annotate, Annotation};
/// use ratatui::widgets::Paragraph;
/// use ratatui::Terminal;
/// use envision::backend::CaptureBackend;
///
/// let backend = CaptureBackend::new(80, 24);
/// let mut terminal = Terminal::new(backend).unwrap();
///
/// let registry = with_annotations(|| {
///     terminal.draw(|frame| {
///         let widget = Annotate::new(
///             Paragraph::new("Hello"),
///             Annotation::label("greeting"),
///         );
///         frame.render_widget(widget, frame.area());
///     }).unwrap();
/// });
///
/// assert_eq!(registry.len(), 1);
/// ```
pub fn with_annotations<F, R>(f: F) -> AnnotationRegistry
where
    F: FnOnce() -> R,
{
    // Set up the context
    ANNOTATION_CONTEXT.with(|ctx| {
        *ctx.borrow_mut() = Some(AnnotationRegistry::new());
    });

    // Run the function
    f();

    // Extract and return the registry
    ANNOTATION_CONTEXT.with(|ctx| {
        ctx.borrow_mut().take().unwrap_or_default()
    })
}

/// Runs a function with access to the current annotation registry.
///
/// This allows registering annotations manually or querying
/// during rendering.
pub fn with_registry<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut AnnotationRegistry) -> R,
{
    ANNOTATION_CONTEXT.with(|ctx| {
        ctx.borrow_mut().as_mut().map(f)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::widgets::Paragraph;

    #[test]
    fn test_annotate_widget() {
        let widget = Annotate::new(
            Paragraph::new("Test"),
            Annotation::button("test-btn"),
        );

        assert!(widget.annotation().has_id("test-btn"));
    }

    #[test]
    fn test_annotate_builder_methods() {
        let widget = Annotate::new(
            Paragraph::new("Input"),
            Annotation::input("name"),
        )
        .focused(true)
        .value("John");

        assert!(widget.annotation().focused);
        assert_eq!(widget.annotation().value, Some("John".to_string()));
    }

    #[test]
    fn test_with_annotations() {
        let registry = with_annotations(|| {
            // Simulate rendering by directly using the context
            with_registry(|reg| {
                reg.register(Rect::new(0, 0, 10, 1), Annotation::button("a"));
                reg.register(Rect::new(0, 2, 10, 1), Annotation::button("b"));
            });
        });

        assert_eq!(registry.len(), 2);
    }

    #[test]
    fn test_nested_annotations() {
        let registry = with_annotations(|| {
            with_registry(|reg| {
                reg.open(Rect::new(0, 0, 80, 24), Annotation::container("main"));
                reg.register(Rect::new(5, 5, 10, 1), Annotation::button("inner"));
                reg.close();
            });
        });

        assert_eq!(registry.len(), 2);

        let inner = registry.get_by_id("inner").unwrap();
        assert_eq!(inner.parent, Some(0));
    }
}
