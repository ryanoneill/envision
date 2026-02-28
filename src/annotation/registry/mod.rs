//! Registry for storing widget annotations during rendering.

use ratatui::layout::Rect;
use serde::{Deserialize, Serialize};

use super::types::{Annotation, WidgetType};

/// Information about an annotated region.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegionInfo {
    /// The rectangular area of this region
    pub area: SerializableRect,

    /// The annotation for this region
    pub annotation: Annotation,

    /// Parent region index (if nested)
    pub parent: Option<usize>,

    /// Child region indices
    pub children: Vec<usize>,

    /// Depth in the widget tree (0 = root)
    pub depth: usize,
}

/// A serializable version of ratatui's Rect.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SerializableRect {
    /// The x coordinate of the top-left corner.
    pub x: u16,
    /// The y coordinate of the top-left corner.
    pub y: u16,
    /// The width of the rectangle.
    pub width: u16,
    /// The height of the rectangle.
    pub height: u16,
}

impl From<Rect> for SerializableRect {
    fn from(rect: Rect) -> Self {
        Self {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
        }
    }
}

impl From<SerializableRect> for Rect {
    fn from(rect: SerializableRect) -> Self {
        Rect::new(rect.x, rect.y, rect.width, rect.height)
    }
}

impl SerializableRect {
    /// Creates a new rect.
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Returns true if this rect contains the given point.
    pub fn contains(&self, x: u16, y: u16) -> bool {
        x >= self.x
            && x < self.x.saturating_add(self.width)
            && y >= self.y
            && y < self.y.saturating_add(self.height)
    }

    /// Returns true if this rect intersects with another.
    pub fn intersects(&self, other: &Self) -> bool {
        self.x < other.x.saturating_add(other.width)
            && self.x.saturating_add(self.width) > other.x
            && self.y < other.y.saturating_add(other.height)
            && self.y.saturating_add(self.height) > other.y
    }
}

/// Registry that collects widget annotations during rendering.
///
/// The registry maintains a tree structure of annotated regions,
/// enabling queries like "what widget is at position X,Y" or
/// "find all buttons".
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AnnotationRegistry {
    /// All registered regions
    regions: Vec<RegionInfo>,

    /// Stack of currently open regions (for nesting)
    #[serde(skip)]
    open_stack: Vec<usize>,

    /// Current nesting depth
    #[serde(skip)]
    current_depth: usize,
}

impl AnnotationRegistry {
    /// Creates a new empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Clears all registered annotations.
    pub fn clear(&mut self) {
        self.regions.clear();
        self.open_stack.clear();
        self.current_depth = 0;
    }

    /// Registers a new annotated region.
    ///
    /// Returns the index of the registered region.
    pub fn register(&mut self, area: Rect, annotation: Annotation) -> usize {
        let parent = self.open_stack.last().copied();
        let index = self.regions.len();

        self.regions.push(RegionInfo {
            area: area.into(),
            annotation,
            parent,
            children: Vec::new(),
            depth: self.current_depth,
        });

        // Add as child of parent
        if let Some(parent_idx) = parent {
            self.regions[parent_idx].children.push(index);
        }

        index
    }

    /// Opens a region (for nested widgets).
    ///
    /// Subsequent registrations will be children of this region.
    pub fn open(&mut self, area: Rect, annotation: Annotation) -> usize {
        let index = self.register(area, annotation);
        self.open_stack.push(index);
        self.current_depth += 1;
        index
    }

    /// Closes the current region.
    pub fn close(&mut self) {
        self.open_stack.pop();
        self.current_depth = self.current_depth.saturating_sub(1);
    }

    /// Returns the number of registered regions.
    pub fn len(&self) -> usize {
        self.regions.len()
    }

    /// Returns true if no regions are registered.
    pub fn is_empty(&self) -> bool {
        self.regions.is_empty()
    }

    /// Returns all registered regions.
    pub fn regions(&self) -> &[RegionInfo] {
        &self.regions
    }

    /// Returns a region by index.
    pub fn get(&self, index: usize) -> Option<&RegionInfo> {
        self.regions.get(index)
    }

    /// Returns the region at the given position.
    ///
    /// If multiple regions overlap, returns the deepest one (most specific).
    pub fn region_at(&self, x: u16, y: u16) -> Option<&RegionInfo> {
        self.regions
            .iter()
            .filter(|r| r.area.contains(x, y))
            .max_by_key(|r| r.depth)
    }

    /// Returns all regions at the given position.
    pub fn regions_at(&self, x: u16, y: u16) -> Vec<&RegionInfo> {
        self.regions
            .iter()
            .filter(|r| r.area.contains(x, y))
            .collect()
    }

    /// Finds regions by annotation id.
    pub fn find_by_id(&self, id: &str) -> Vec<&RegionInfo> {
        self.regions
            .iter()
            .filter(|r| r.annotation.has_id(id))
            .collect()
    }

    /// Finds the first region with the given id.
    pub fn get_by_id(&self, id: &str) -> Option<&RegionInfo> {
        self.regions.iter().find(|r| r.annotation.has_id(id))
    }

    /// Finds regions by widget type.
    pub fn find_by_type(&self, widget_type: &WidgetType) -> Vec<&RegionInfo> {
        self.regions
            .iter()
            .filter(|r| r.annotation.is_type(widget_type))
            .collect()
    }

    /// Returns all interactive regions.
    pub fn interactive_regions(&self) -> Vec<&RegionInfo> {
        self.regions
            .iter()
            .filter(|r| r.annotation.is_interactive())
            .collect()
    }

    /// Returns the currently focused region, if any.
    pub fn focused_region(&self) -> Option<&RegionInfo> {
        self.regions.iter().find(|r| r.annotation.focused)
    }

    /// Returns root regions (depth 0).
    pub fn root_regions(&self) -> Vec<&RegionInfo> {
        self.regions.iter().filter(|r| r.depth == 0).collect()
    }

    /// Returns children of a region.
    pub fn children_of(&self, index: usize) -> Vec<&RegionInfo> {
        if let Some(region) = self.regions.get(index) {
            region
                .children
                .iter()
                .filter_map(|&i| self.regions.get(i))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Formats the registry as a tree for debugging.
    pub fn format_tree(&self) -> String {
        let mut output = String::new();

        for region in &self.regions {
            if region.parent.is_none() {
                self.format_region(&mut output, region, 0);
            }
        }

        output
    }

    fn format_region(&self, output: &mut String, region: &RegionInfo, indent: usize) {
        let prefix = "  ".repeat(indent);
        output.push_str(&format!(
            "{}[{},{}+{}x{}] {}\n",
            prefix,
            region.area.x,
            region.area.y,
            region.area.width,
            region.area.height,
            region.annotation.description()
        ));

        for &child_idx in &region.children {
            if let Some(child) = self.regions.get(child_idx) {
                self.format_region(output, child, indent + 1);
            }
        }
    }
}

#[cfg(test)]
mod tests;
