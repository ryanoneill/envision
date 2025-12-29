//! Widget annotation system for semantic UI understanding.
//!
//! Annotations provide semantic metadata about UI regions, enabling:
//!
//! - Querying widgets by type or label ("find the submit button")
//! - Understanding widget hierarchy and focus
//! - Automated testing with semantic queries
//! - Accessibility-like descriptions of the UI
//!
//! # Example
//!
//! ```rust,no_run
//! use envision::annotation::{Annotate, Annotation};
//! use ratatui::widgets::Paragraph;
//! use ratatui::Frame;
//!
//! fn render(frame: &mut Frame) {
//!     let area = frame.area();
//!
//!     // Wrap widget with annotation
//!     let widget = Annotate::new(
//!         Paragraph::new("Submit"),
//!         Annotation::button("submit").with_label("Submit Order"),
//!     );
//!
//!     frame.render_widget(widget, area);
//! }
//! ```

mod registry;
mod types;
mod widget;

pub use registry::{AnnotationRegistry, RegionInfo, SerializableRect};
pub use types::{Annotation, WidgetType};
pub use widget::{with_annotations, with_registry, Annotate, AnnotateContainer};
