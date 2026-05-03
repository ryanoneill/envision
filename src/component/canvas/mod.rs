//! A general-purpose drawing surface component.
//!
//! [`Canvas`] wraps ratatui's `Canvas` widget to provide a drawing surface
//! with shape primitives (lines, rectangles, circles, points, labels).
//! This is the foundation for custom visualizations like heatmaps, scatter
//! plots, and flame graphs. State is stored in [`CanvasState`] and updated
//! via [`CanvasMessage`].
//!
//! # Example
//!
//! ```rust
//! use envision::component::{
//!     Canvas, CanvasState, CanvasMessage, CanvasShape, CanvasMarker, Component,
//! };
//! use ratatui::style::Color;
//!
//! let mut state = CanvasState::new()
//!     .with_bounds(0.0, 100.0, 0.0, 100.0)
//!     .with_title("My Canvas");
//!
//! state.add_shape(CanvasShape::Line {
//!     x1: 0.0, y1: 0.0,
//!     x2: 100.0, y2: 100.0,
//!     color: Color::Red,
//! });
//!
//! assert_eq!(state.shapes().len(), 1);
//! assert_eq!(state.x_bounds(), [0.0, 100.0]);
//! ```

use std::marker::PhantomData;

use ratatui::prelude::*;
use ratatui::widgets::canvas::{
    Canvas as RatatuiCanvas, Circle, Line as CanvasLine, Points, Rectangle,
};
use ratatui::widgets::{Block, Borders};

use super::{Component, RenderContext};

/// A drawable shape on the canvas.
///
/// Each variant represents a different kind of shape that can be drawn
/// on the canvas surface. All shapes include a color for rendering.
///
/// # Example
///
/// ```rust
/// use envision::component::CanvasShape;
/// use ratatui::style::Color;
///
/// let line = CanvasShape::Line {
///     x1: 0.0, y1: 0.0,
///     x2: 50.0, y2: 50.0,
///     color: Color::Cyan,
/// };
///
/// let circle = CanvasShape::Circle {
///     x: 50.0, y: 50.0,
///     radius: 20.0,
///     color: Color::Yellow,
/// };
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum CanvasShape {
    /// A line between two points.
    Line {
        /// Start x coordinate.
        x1: f64,
        /// Start y coordinate.
        y1: f64,
        /// End x coordinate.
        x2: f64,
        /// End y coordinate.
        y2: f64,
        /// Line color.
        color: Color,
    },
    /// An axis-aligned rectangle.
    Rectangle {
        /// Left x coordinate.
        x: f64,
        /// Bottom y coordinate.
        y: f64,
        /// Width of the rectangle.
        width: f64,
        /// Height of the rectangle.
        height: f64,
        /// Rectangle color.
        color: Color,
    },
    /// A circle with center and radius.
    Circle {
        /// Center x coordinate.
        x: f64,
        /// Center y coordinate.
        y: f64,
        /// Circle radius.
        radius: f64,
        /// Circle color.
        color: Color,
    },
    /// A set of individual points.
    Points {
        /// The point coordinates.
        coords: Vec<(f64, f64)>,
        /// Point color.
        color: Color,
    },
    /// A text label at a position.
    Label {
        /// Label x coordinate.
        x: f64,
        /// Label y coordinate.
        y: f64,
        /// Label text.
        text: String,
        /// Label color.
        color: Color,
    },
}

/// The marker type used for drawing on the canvas.
///
/// Different markers provide different resolution and visual styles:
/// - `Dot`: Uses Unicode dot character
/// - `Block`: Uses full block character
/// - `HalfBlock`: Uses half block character for higher resolution
/// - `Braille`: Uses Braille patterns for highest resolution (default)
///
/// # Example
///
/// ```rust
/// use envision::component::CanvasMarker;
///
/// let marker = CanvasMarker::default();
/// assert_eq!(marker, CanvasMarker::Braille);
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum CanvasMarker {
    /// Unicode dot character marker.
    Dot,
    /// Full block character marker.
    Block,
    /// Half block character marker for higher resolution.
    HalfBlock,
    /// Braille pattern marker for highest resolution.
    #[default]
    Braille,
}

impl CanvasMarker {
    /// Converts to the ratatui marker type.
    fn to_ratatui(&self) -> ratatui::symbols::Marker {
        match self {
            CanvasMarker::Dot => ratatui::symbols::Marker::Dot,
            CanvasMarker::Block => ratatui::symbols::Marker::Block,
            CanvasMarker::HalfBlock => ratatui::symbols::Marker::HalfBlock,
            CanvasMarker::Braille => ratatui::symbols::Marker::Braille,
        }
    }
}

/// Messages that can be sent to a Canvas.
///
/// # Example
///
/// ```rust
/// use envision::component::{Canvas, CanvasMessage, CanvasShape, CanvasState, Component};
/// use ratatui::style::Color;
///
/// let mut state = CanvasState::new();
///
/// // Add a shape
/// Canvas::update(&mut state, CanvasMessage::AddShape(CanvasShape::Circle {
///     x: 50.0, y: 50.0, radius: 10.0, color: Color::Green,
/// }));
/// assert_eq!(state.shapes().len(), 1);
///
/// // Clear all shapes
/// Canvas::update(&mut state, CanvasMessage::Clear);
/// assert!(state.shapes().is_empty());
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum CanvasMessage {
    /// Add a shape to the canvas.
    AddShape(CanvasShape),
    /// Replace all shapes on the canvas.
    SetShapes(Vec<CanvasShape>),
    /// Remove all shapes from the canvas.
    Clear,
    /// Set the coordinate bounds.
    SetBounds {
        /// X-axis bounds [min, max].
        x: [f64; 2],
        /// Y-axis bounds [min, max].
        y: [f64; 2],
    },
    /// Set the marker type.
    SetMarker(CanvasMarker),
}

/// State for a Canvas component.
///
/// Contains the shapes to draw, coordinate bounds, and display options.
///
/// # Example
///
/// ```rust
/// use envision::component::{CanvasState, CanvasShape, CanvasMarker};
/// use ratatui::style::Color;
///
/// let state = CanvasState::new()
///     .with_bounds(0.0, 200.0, 0.0, 100.0)
///     .with_title("Drawing")
///     .with_marker(CanvasMarker::HalfBlock)
///     .with_shapes(vec![
///         CanvasShape::Circle { x: 100.0, y: 50.0, radius: 25.0, color: Color::Cyan },
///     ]);
///
/// assert_eq!(state.shapes().len(), 1);
/// assert_eq!(state.x_bounds(), [0.0, 200.0]);
/// assert_eq!(state.y_bounds(), [0.0, 100.0]);
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct CanvasState {
    /// The shapes to draw on the canvas.
    shapes: Vec<CanvasShape>,
    /// X-axis range [min, max].
    x_bounds: [f64; 2],
    /// Y-axis range [min, max].
    y_bounds: [f64; 2],
    /// Optional border title.
    title: Option<String>,
    /// The marker type for drawing.
    marker: CanvasMarker,
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            shapes: Vec::new(),
            x_bounds: [0.0, 100.0],
            y_bounds: [0.0, 100.0],
            title: None,
            marker: CanvasMarker::default(),
        }
    }
}

impl CanvasState {
    /// Creates a new empty canvas with default bounds [0, 100] on both axes.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CanvasState;
    ///
    /// let state = CanvasState::new();
    /// assert!(state.shapes().is_empty());
    /// assert_eq!(state.x_bounds(), [0.0, 100.0]);
    /// assert_eq!(state.y_bounds(), [0.0, 100.0]);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the initial shapes (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{CanvasState, CanvasShape};
    /// use ratatui::style::Color;
    ///
    /// let state = CanvasState::new().with_shapes(vec![
    ///     CanvasShape::Line { x1: 0.0, y1: 0.0, x2: 100.0, y2: 100.0, color: Color::Red },
    /// ]);
    /// assert_eq!(state.shapes().len(), 1);
    /// ```
    pub fn with_shapes(mut self, shapes: Vec<CanvasShape>) -> Self {
        self.shapes = shapes;
        self
    }

    /// Sets the x-axis range (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CanvasState;
    ///
    /// let state = CanvasState::new().with_x_bounds(-50.0, 50.0);
    /// assert_eq!(state.x_bounds(), [-50.0, 50.0]);
    /// ```
    pub fn with_x_bounds(mut self, min: f64, max: f64) -> Self {
        self.x_bounds = [min, max];
        self
    }

    /// Sets the y-axis range (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CanvasState;
    ///
    /// let state = CanvasState::new().with_y_bounds(0.0, 200.0);
    /// assert_eq!(state.y_bounds(), [0.0, 200.0]);
    /// ```
    pub fn with_y_bounds(mut self, min: f64, max: f64) -> Self {
        self.y_bounds = [min, max];
        self
    }

    /// Sets both x and y axis ranges (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CanvasState;
    ///
    /// let state = CanvasState::new().with_bounds(0.0, 200.0, 0.0, 100.0);
    /// assert_eq!(state.x_bounds(), [0.0, 200.0]);
    /// assert_eq!(state.y_bounds(), [0.0, 100.0]);
    /// ```
    pub fn with_bounds(mut self, x_min: f64, x_max: f64, y_min: f64, y_max: f64) -> Self {
        self.x_bounds = [x_min, x_max];
        self.y_bounds = [y_min, y_max];
        self
    }

    /// Sets the border title (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CanvasState;
    ///
    /// let state = CanvasState::new().with_title("Drawing Surface");
    /// assert_eq!(state.title(), Some("Drawing Surface"));
    /// ```
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the marker type (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{CanvasState, CanvasMarker};
    ///
    /// let state = CanvasState::new().with_marker(CanvasMarker::Block);
    /// assert_eq!(state.marker(), &CanvasMarker::Block);
    /// ```
    pub fn with_marker(mut self, marker: CanvasMarker) -> Self {
        self.marker = marker;
        self
    }

    // ---- Accessors ----

    /// Returns the shapes on the canvas.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{CanvasState, CanvasShape};
    /// use ratatui::style::Color;
    ///
    /// let state = CanvasState::new().with_shapes(vec![
    ///     CanvasShape::Circle { x: 50.0, y: 50.0, radius: 10.0, color: Color::Red },
    /// ]);
    /// assert_eq!(state.shapes().len(), 1);
    /// ```
    pub fn shapes(&self) -> &[CanvasShape] {
        &self.shapes
    }

    /// Adds a shape to the canvas.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{CanvasState, CanvasShape};
    /// use ratatui::style::Color;
    ///
    /// let mut state = CanvasState::new();
    /// state.add_shape(CanvasShape::Line {
    ///     x1: 0.0, y1: 0.0, x2: 100.0, y2: 100.0, color: Color::White,
    /// });
    /// assert_eq!(state.shapes().len(), 1);
    /// ```
    pub fn add_shape(&mut self, shape: CanvasShape) {
        self.shapes.push(shape);
    }

    /// Removes all shapes from the canvas.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{CanvasState, CanvasShape};
    /// use ratatui::style::Color;
    ///
    /// let mut state = CanvasState::new().with_shapes(vec![
    ///     CanvasShape::Circle { x: 50.0, y: 50.0, radius: 10.0, color: Color::Red },
    /// ]);
    /// state.clear();
    /// assert!(state.shapes().is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.shapes.clear();
    }

    /// Returns the x-axis bounds.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CanvasState;
    ///
    /// let state = CanvasState::new().with_x_bounds(0.0, 50.0);
    /// assert_eq!(state.x_bounds(), [0.0, 50.0]);
    /// ```
    pub fn x_bounds(&self) -> [f64; 2] {
        self.x_bounds
    }

    /// Returns the y-axis bounds.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CanvasState;
    ///
    /// let state = CanvasState::new().with_y_bounds(0.0, 200.0);
    /// assert_eq!(state.y_bounds(), [0.0, 200.0]);
    /// ```
    pub fn y_bounds(&self) -> [f64; 2] {
        self.y_bounds
    }

    /// Sets the x-axis bounds.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CanvasState;
    ///
    /// let mut state = CanvasState::new();
    /// state.set_x_bounds(-100.0, 100.0);
    /// assert_eq!(state.x_bounds(), [-100.0, 100.0]);
    /// ```
    pub fn set_x_bounds(&mut self, min: f64, max: f64) {
        self.x_bounds = [min, max];
    }

    /// Sets the y-axis bounds.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CanvasState;
    ///
    /// let mut state = CanvasState::new();
    /// state.set_y_bounds(-50.0, 50.0);
    /// assert_eq!(state.y_bounds(), [-50.0, 50.0]);
    /// ```
    pub fn set_y_bounds(&mut self, min: f64, max: f64) {
        self.y_bounds = [min, max];
    }

    /// Returns the title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CanvasState;
    ///
    /// let state = CanvasState::new().with_title("Overview");
    /// assert_eq!(state.title(), Some("Overview"));
    /// ```
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Sets the title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CanvasState;
    ///
    /// let mut state = CanvasState::new();
    /// state.set_title(Some("Updated".to_string()));
    /// assert_eq!(state.title(), Some("Updated"));
    /// ```
    pub fn set_title(&mut self, title: Option<String>) {
        self.title = title;
    }

    /// Returns the marker type.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{CanvasMarker, CanvasState};
    ///
    /// let state = CanvasState::new().with_marker(CanvasMarker::Dot);
    /// assert_eq!(state.marker(), &CanvasMarker::Dot);
    /// ```
    pub fn marker(&self) -> &CanvasMarker {
        &self.marker
    }

    /// Sets the marker type.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{CanvasMarker, CanvasState};
    ///
    /// let mut state = CanvasState::new();
    /// state.set_marker(CanvasMarker::Block);
    /// assert_eq!(state.marker(), &CanvasMarker::Block);
    /// ```
    pub fn set_marker(&mut self, marker: CanvasMarker) {
        self.marker = marker;
    }

    // ---- Instance methods ----

    /// Updates the state with a message, returning any output.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{CanvasState, CanvasMessage, CanvasShape};
    /// use ratatui::style::Color;
    ///
    /// let mut state = CanvasState::new();
    /// state.update(CanvasMessage::AddShape(CanvasShape::Circle {
    ///     x: 50.0, y: 50.0, radius: 10.0, color: Color::Cyan,
    /// }));
    /// assert_eq!(state.shapes().len(), 1);
    /// ```
    pub fn update(&mut self, msg: CanvasMessage) -> Option<()> {
        Canvas::update(self, msg)
    }
}

/// A general-purpose drawing surface component.
///
/// `Canvas` provides a drawing surface with shape primitives (lines,
/// rectangles, circles, points, labels). It wraps ratatui's `Canvas`
/// widget and serves as the foundation for custom visualizations.
///
/// The canvas is display-only for now but may support pan/zoom
/// functionality in the future.
///
/// # Example
///
/// ```rust
/// use envision::component::{Canvas, CanvasState, CanvasShape, CanvasMessage, Component};
/// use ratatui::style::Color;
///
/// let mut state = CanvasState::new()
///     .with_title("Visualization")
///     .with_bounds(0.0, 100.0, 0.0, 100.0);
///
/// // Add shapes via messages
/// Canvas::update(&mut state, CanvasMessage::AddShape(CanvasShape::Circle {
///     x: 50.0, y: 50.0, radius: 25.0, color: Color::Cyan,
/// }));
///
/// // Or directly
/// state.add_shape(CanvasShape::Line {
///     x1: 0.0, y1: 0.0, x2: 100.0, y2: 100.0, color: Color::Red,
/// });
///
/// assert_eq!(state.shapes().len(), 2);
/// ```
pub struct Canvas(PhantomData<()>);

impl Component for Canvas {
    type State = CanvasState;
    type Message = CanvasMessage;
    type Output = ();

    fn init() -> Self::State {
        CanvasState::default()
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            CanvasMessage::AddShape(shape) => {
                state.shapes.push(shape);
            }
            CanvasMessage::SetShapes(shapes) => {
                state.shapes = shapes;
            }
            CanvasMessage::Clear => {
                state.shapes.clear();
            }
            CanvasMessage::SetBounds { x, y } => {
                state.x_bounds = x;
                state.y_bounds = y;
            }
            CanvasMessage::SetMarker(marker) => {
                state.marker = marker;
            }
        }
        None
    }

    fn view(state: &Self::State, ctx: &mut RenderContext<'_, '_>) {
        if ctx.area.height < 2 || ctx.area.width < 2 {
            return;
        }

        crate::annotation::with_registry(|reg| {
            reg.register(
                ctx.area,
                crate::annotation::Annotation::canvas("canvas")
                    .with_focus(ctx.focused)
                    .with_disabled(ctx.disabled),
            );
        });

        let needs_border = !ctx.chrome_owned && (state.title.is_some() || ctx.focused);

        let content_style = if ctx.disabled {
            ctx.theme.disabled_style()
        } else {
            ctx.theme.normal_style()
        };

        let canvas_area = if needs_border {
            let border_style = if ctx.disabled {
                ctx.theme.disabled_style()
            } else if ctx.focused {
                ctx.theme.focused_border_style()
            } else {
                ctx.theme.border_style()
            };

            let mut block = Block::default()
                .borders(Borders::ALL)
                .border_style(border_style);

            if let Some(ref title) = state.title {
                block = block.title(title.as_str());
            }

            let inner = block.inner(ctx.area);
            ctx.frame.render_widget(block, ctx.area);
            inner
        } else {
            ctx.area
        };

        if canvas_area.height == 0 || canvas_area.width == 0 {
            return;
        }

        let marker = state.marker.to_ratatui();
        let x_bounds = state.x_bounds;
        let y_bounds = state.y_bounds;
        let shapes = state.shapes.clone();
        let is_disabled = ctx.disabled;
        let disabled_style = ctx.theme.disabled_style();

        let canvas = RatatuiCanvas::default()
            .x_bounds(x_bounds)
            .y_bounds(y_bounds)
            .marker(marker)
            .background_color(content_style.bg.unwrap_or(Color::Reset))
            .paint(move |ctx| {
                for shape in &shapes {
                    match shape {
                        CanvasShape::Line {
                            x1,
                            y1,
                            x2,
                            y2,
                            color,
                        } => {
                            let draw_color = if is_disabled {
                                disabled_style.fg.unwrap_or(Color::DarkGray)
                            } else {
                                *color
                            };
                            ctx.draw(&CanvasLine::new(*x1, *y1, *x2, *y2, draw_color));
                        }
                        CanvasShape::Rectangle {
                            x,
                            y,
                            width,
                            height,
                            color,
                        } => {
                            let draw_color = if is_disabled {
                                disabled_style.fg.unwrap_or(Color::DarkGray)
                            } else {
                                *color
                            };
                            ctx.draw(&Rectangle {
                                x: *x,
                                y: *y,
                                width: *width,
                                height: *height,
                                color: draw_color,
                            });
                        }
                        CanvasShape::Circle {
                            x,
                            y,
                            radius,
                            color,
                        } => {
                            let draw_color = if is_disabled {
                                disabled_style.fg.unwrap_or(Color::DarkGray)
                            } else {
                                *color
                            };
                            ctx.draw(&Circle {
                                x: *x,
                                y: *y,
                                radius: *radius,
                                color: draw_color,
                            });
                        }
                        CanvasShape::Points { coords, color } => {
                            let draw_color = if is_disabled {
                                disabled_style.fg.unwrap_or(Color::DarkGray)
                            } else {
                                *color
                            };
                            ctx.draw(&Points {
                                coords,
                                color: draw_color,
                            });
                        }
                        CanvasShape::Label { x, y, text, color } => {
                            let draw_color = if is_disabled {
                                disabled_style.fg.unwrap_or(Color::DarkGray)
                            } else {
                                *color
                            };
                            ctx.print(
                                *x,
                                *y,
                                Span::styled(text.clone(), Style::default().fg(draw_color)),
                            );
                        }
                    }
                }
            });

        ctx.frame.render_widget(canvas, canvas_area);
    }
}

#[cfg(test)]
mod tests;
