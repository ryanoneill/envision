//! Structured output types for semantic annotation data.
//!
//! These types provide a structured representation of annotation data
//! suitable for AI consumption, automated testing, and programmatic
//! inspection of rendered UI state.

use std::collections::HashMap;

use super::registry::{AnnotationRegistry, RegionInfo, SerializableRect};

/// Combined visual and semantic output from a rendered frame.
///
/// This pairs the visual text representation (what a human would see)
/// with structured annotation data (what a machine can parse).
///
/// # Example
///
/// ```rust
/// use envision::annotation::{AnnotatedOutput, AnnotationRegistry, Annotation, with_annotations};
/// use envision::backend::CaptureBackend;
/// use ratatui::Terminal;
///
/// let backend = CaptureBackend::new(80, 24);
/// let mut terminal = Terminal::new(backend).unwrap();
///
/// let registry = with_annotations(|| {
///     terminal.draw(|frame| {
///         // Render widgets with annotations...
///     }).unwrap();
/// });
///
/// let output = AnnotatedOutput::from_backend_and_registry(terminal.backend(), &registry);
/// assert!(!output.visual.is_empty());
/// ```
#[derive(Clone, Debug)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct AnnotatedOutput {
    /// The visual text representation of the rendered frame.
    pub visual: String,

    /// Structured annotations for each widget in the frame.
    pub annotations: Vec<WidgetAnnotation>,
}

/// Semantic metadata about a single widget in the rendered output.
///
/// This provides machine-readable information about a widget's type,
/// state, position, and any additional metadata attached during rendering.
#[derive(Clone, Debug)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct WidgetAnnotation {
    /// The type of widget (e.g., "Button", "Input", "Container").
    pub widget_type: String,

    /// Optional programmatic identifier for the widget.
    pub id: Option<String>,

    /// Optional human-readable label.
    pub label: Option<String>,

    /// The rectangular area occupied by the widget.
    pub area: AnnotationArea,

    /// Whether this widget currently has focus.
    pub focused: bool,

    /// Whether this widget is currently disabled.
    pub disabled: bool,

    /// Whether this widget is currently selected.
    pub selected: bool,

    /// Whether this widget is expanded (for collapsible widgets).
    pub expanded: Option<bool>,

    /// The current value of the widget (for inputs, etc.).
    pub value: Option<String>,

    /// Index of the parent widget in the annotations list, if nested.
    pub parent_index: Option<usize>,

    /// Depth in the widget tree (0 = root).
    pub depth: usize,

    /// Additional key-value metadata.
    #[cfg_attr(
        feature = "serialization",
        serde(default, skip_serializing_if = "HashMap::is_empty")
    )]
    pub metadata: HashMap<String, String>,
}

/// The rectangular area of a widget annotation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct AnnotationArea {
    /// The x coordinate of the top-left corner.
    pub x: u16,

    /// The y coordinate of the top-left corner.
    pub y: u16,

    /// The width of the area.
    pub width: u16,

    /// The height of the area.
    pub height: u16,
}

impl From<SerializableRect> for AnnotationArea {
    fn from(rect: SerializableRect) -> Self {
        Self {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
        }
    }
}

impl WidgetAnnotation {
    /// Creates a `WidgetAnnotation` from a `RegionInfo`.
    fn from_region(region: &RegionInfo) -> Self {
        Self {
            widget_type: format!("{}", region.annotation.widget_type),
            id: region.annotation.id.clone(),
            label: region.annotation.label.clone(),
            area: region.area.into(),
            focused: region.annotation.focused,
            disabled: region.annotation.disabled,
            selected: region.annotation.selected,
            expanded: region.annotation.expanded,
            value: region.annotation.value.clone(),
            parent_index: region.parent,
            depth: region.depth,
            metadata: region.annotation.metadata.clone(),
        }
    }
}

impl AnnotatedOutput {
    /// Creates an `AnnotatedOutput` from a `CaptureBackend` and an `AnnotationRegistry`.
    ///
    /// The visual output is the plain text representation of the backend's
    /// current buffer. The annotations are converted from the registry's
    /// region data into a flat list of `WidgetAnnotation` values.
    pub fn from_backend_and_registry(
        backend: &crate::backend::CaptureBackend,
        registry: &AnnotationRegistry,
    ) -> Self {
        let visual = backend.to_string();
        let annotations = registry
            .regions()
            .iter()
            .map(WidgetAnnotation::from_region)
            .collect();

        Self {
            visual,
            annotations,
        }
    }

    /// Creates an `AnnotatedOutput` from a visual string and an `AnnotationRegistry`.
    ///
    /// This is useful when you already have the visual output as a string
    /// and want to pair it with annotation data.
    pub fn from_visual_and_registry(visual: String, registry: &AnnotationRegistry) -> Self {
        let annotations = registry
            .regions()
            .iter()
            .map(WidgetAnnotation::from_region)
            .collect();

        Self {
            visual,
            annotations,
        }
    }

    /// Returns the number of annotations.
    pub fn annotation_count(&self) -> usize {
        self.annotations.len()
    }

    /// Returns true if there are no annotations.
    pub fn is_empty(&self) -> bool {
        self.annotations.is_empty()
    }

    /// Finds annotations by widget type name.
    pub fn find_by_type(&self, widget_type: &str) -> Vec<&WidgetAnnotation> {
        self.annotations
            .iter()
            .filter(|a| a.widget_type == widget_type)
            .collect()
    }

    /// Finds an annotation by its id.
    pub fn find_by_id(&self, id: &str) -> Option<&WidgetAnnotation> {
        self.annotations
            .iter()
            .find(|a| a.id.as_deref() == Some(id))
    }

    /// Returns all focused annotations.
    pub fn focused_annotations(&self) -> Vec<&WidgetAnnotation> {
        self.annotations.iter().filter(|a| a.focused).collect()
    }

    /// Serializes the output to a JSON string.
    ///
    /// Returns `None` if serialization fails.
    #[cfg(feature = "serialization")]
    pub fn to_json(&self) -> Option<String> {
        serde_json::to_string(self).ok()
    }

    /// Serializes the output to a pretty-printed JSON string.
    ///
    /// Returns `None` if serialization fails.
    #[cfg(feature = "serialization")]
    pub fn to_json_pretty(&self) -> Option<String> {
        serde_json::to_string_pretty(self).ok()
    }
}

#[cfg(test)]
mod tests;
