//! 2D viewport for panning and zooming the diagram.
//!
//! The viewport translates between "graph space" (f64 coordinates from
//! the layout engine) and "screen space" (u16 terminal cell coordinates).

use ratatui::layout::Rect;

/// Bounding box in graph coordinates.
///
/// # Examples
///
/// ```
/// use envision::diagram::BoundingBox;
///
/// let bbox = BoundingBox::new(0.0, 0.0, 100.0, 50.0);
/// assert_eq!(bbox.width(), 100.0);
/// assert_eq!(bbox.height(), 50.0);
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct BoundingBox {
    /// Minimum x coordinate.
    pub min_x: f64,
    /// Minimum y coordinate.
    pub min_y: f64,
    /// Maximum x coordinate.
    pub max_x: f64,
    /// Maximum y coordinate.
    pub max_y: f64,
}

impl BoundingBox {
    /// Creates a new bounding box.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::BoundingBox;
    ///
    /// let bbox = BoundingBox::new(-10.0, -5.0, 90.0, 45.0);
    /// assert_eq!(bbox.min_x, -10.0);
    /// assert_eq!(bbox.max_y, 45.0);
    /// ```
    pub fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    /// Returns the width of the bounding box.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::BoundingBox;
    ///
    /// let bbox = BoundingBox::new(10.0, 0.0, 50.0, 30.0);
    /// assert_eq!(bbox.width(), 40.0);
    /// ```
    pub fn width(&self) -> f64 {
        self.max_x - self.min_x
    }

    /// Returns the height of the bounding box.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::BoundingBox;
    ///
    /// let bbox = BoundingBox::new(0.0, 5.0, 100.0, 25.0);
    /// assert_eq!(bbox.height(), 20.0);
    /// ```
    pub fn height(&self) -> f64 {
        self.max_y - self.min_y
    }
}

impl Default for BoundingBox {
    fn default() -> Self {
        Self {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 0.0,
            max_y: 0.0,
        }
    }
}

/// 2D viewport state for panning and zooming the diagram.
///
/// The viewport defines a window into graph space. At `zoom = 1.0`,
/// one graph unit maps to one terminal cell. Panning shifts the window,
/// zooming scales the mapping.
///
/// # Examples
///
/// ```
/// use envision::diagram::{Viewport2D, BoundingBox};
/// use ratatui::layout::Rect;
///
/// let mut vp = Viewport2D::new();
/// vp.set_viewport_size(80, 24);
///
/// // Pan right by 10 units
/// vp.pan(10.0, 0.0);
/// assert_eq!(vp.offset_x(), 10.0);
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct Viewport2D {
    offset_x: f64,
    offset_y: f64,
    zoom: f64,
    content_bbox: BoundingBox,
    viewport_width: f64,
    viewport_height: f64,
}

impl Default for Viewport2D {
    fn default() -> Self {
        Self::new()
    }
}

impl Viewport2D {
    const MIN_ZOOM: f64 = 0.25;
    const MAX_ZOOM: f64 = 4.0;
    const ZOOM_STEP: f64 = 1.25;
    const PAN_STEP: f64 = 5.0;

    /// Creates a new viewport at origin with zoom 1.0.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::Viewport2D;
    ///
    /// let vp = Viewport2D::new();
    /// assert_eq!(vp.offset_x(), 0.0);
    /// assert_eq!(vp.offset_y(), 0.0);
    /// assert_eq!(vp.zoom(), 1.0);
    /// ```
    pub fn new() -> Self {
        Self {
            offset_x: 0.0,
            offset_y: 0.0,
            zoom: 1.0,
            content_bbox: BoundingBox::default(),
            viewport_width: 80.0,
            viewport_height: 24.0,
        }
    }

    /// Returns the horizontal offset in graph units.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::Viewport2D;
    ///
    /// let vp = Viewport2D::new();
    /// assert_eq!(vp.offset_x(), 0.0);
    /// ```
    pub fn offset_x(&self) -> f64 {
        self.offset_x
    }

    /// Returns the vertical offset in graph units.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::Viewport2D;
    ///
    /// let vp = Viewport2D::new();
    /// assert_eq!(vp.offset_y(), 0.0);
    /// ```
    pub fn offset_y(&self) -> f64 {
        self.offset_y
    }

    /// Returns the current zoom level.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::Viewport2D;
    ///
    /// let vp = Viewport2D::new();
    /// assert_eq!(vp.zoom(), 1.0);
    /// ```
    pub fn zoom(&self) -> f64 {
        self.zoom
    }

    /// Pans the viewport by the given delta in graph units.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::Viewport2D;
    ///
    /// let mut vp = Viewport2D::new();
    /// vp.pan(5.0, -3.0);
    /// assert_eq!(vp.offset_x(), 5.0);
    /// assert_eq!(vp.offset_y(), -3.0);
    /// ```
    pub fn pan(&mut self, dx: f64, dy: f64) {
        self.offset_x += dx;
        self.offset_y += dy;
    }

    /// Pans one step in the given direction.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::Viewport2D;
    ///
    /// let mut vp = Viewport2D::new();
    /// vp.pan_step(1.0, 0.0); // pan right
    /// assert!(vp.offset_x() > 0.0);
    /// ```
    pub fn pan_step(&mut self, dx: f64, dy: f64) {
        let step = Self::PAN_STEP / self.zoom;
        self.offset_x += dx * step;
        self.offset_y += dy * step;
    }

    /// Zooms in by one step, clamped to maximum zoom.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::Viewport2D;
    ///
    /// let mut vp = Viewport2D::new();
    /// let before = vp.zoom();
    /// vp.zoom_in();
    /// assert!(vp.zoom() > before);
    /// ```
    pub fn zoom_in(&mut self) {
        self.zoom = (self.zoom * Self::ZOOM_STEP).min(Self::MAX_ZOOM);
    }

    /// Zooms out by one step, clamped to minimum zoom.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::Viewport2D;
    ///
    /// let mut vp = Viewport2D::new();
    /// let before = vp.zoom();
    /// vp.zoom_out();
    /// assert!(vp.zoom() < before);
    /// ```
    pub fn zoom_out(&mut self) {
        self.zoom = (self.zoom / Self::ZOOM_STEP).max(Self::MIN_ZOOM);
    }

    /// Sets the viewport size in terminal cells.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::Viewport2D;
    ///
    /// let mut vp = Viewport2D::new();
    /// vp.set_viewport_size(120, 40);
    /// ```
    pub fn set_viewport_size(&mut self, width: u16, height: u16) {
        self.viewport_width = f64::from(width);
        self.viewport_height = f64::from(height);
    }

    /// Updates the content bounding box (from layout results).
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::{Viewport2D, BoundingBox};
    ///
    /// let mut vp = Viewport2D::new();
    /// vp.set_content_bounds(BoundingBox::new(0.0, 0.0, 200.0, 100.0));
    /// ```
    pub fn set_content_bounds(&mut self, bbox: BoundingBox) {
        self.content_bbox = bbox;
    }

    /// Adjusts offset and zoom to fit the entire graph in the viewport.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::{Viewport2D, BoundingBox};
    ///
    /// let mut vp = Viewport2D::new();
    /// vp.set_viewport_size(80, 24);
    /// vp.set_content_bounds(BoundingBox::new(0.0, 0.0, 160.0, 48.0));
    /// vp.fit_to_content();
    /// // Zoom should be reduced to fit 160-wide content in 80-wide viewport
    /// assert!(vp.zoom() < 1.0);
    /// ```
    pub fn fit_to_content(&mut self) {
        let cw = self.content_bbox.width();
        let ch = self.content_bbox.height();

        if cw <= 0.0 || ch <= 0.0 {
            self.offset_x = 0.0;
            self.offset_y = 0.0;
            self.zoom = 1.0;
            return;
        }

        let padding = 2.0;
        let available_w = (self.viewport_width - padding * 2.0).max(1.0);
        let available_h = (self.viewport_height - padding * 2.0).max(1.0);

        let zoom_x = available_w / cw;
        let zoom_y = available_h / ch;
        self.zoom = zoom_x.min(zoom_y).clamp(Self::MIN_ZOOM, Self::MAX_ZOOM);

        // Center the content
        self.offset_x = self.content_bbox.min_x - padding / self.zoom;
        self.offset_y = self.content_bbox.min_y - padding / self.zoom;
    }

    /// Adjusts the viewport so that the given rectangle is visible.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::Viewport2D;
    ///
    /// let mut vp = Viewport2D::new();
    /// vp.set_viewport_size(80, 24);
    /// vp.ensure_visible(100.0, 50.0, 20.0, 5.0);
    /// // Viewport should have scrolled to make (100, 50) visible
    /// assert!(vp.offset_x() > 0.0);
    /// ```
    pub fn ensure_visible(&mut self, x: f64, y: f64, w: f64, h: f64) {
        let vw = self.viewport_width / self.zoom;
        let vh = self.viewport_height / self.zoom;

        if x < self.offset_x {
            self.offset_x = x - 1.0;
        }
        if x + w > self.offset_x + vw {
            self.offset_x = x + w - vw + 1.0;
        }
        if y < self.offset_y {
            self.offset_y = y - 1.0;
        }
        if y + h > self.offset_y + vh {
            self.offset_y = y + h - vh + 1.0;
        }
    }

    /// Returns true if the given rectangle overlaps the visible viewport.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::Viewport2D;
    ///
    /// let mut vp = Viewport2D::new();
    /// vp.set_viewport_size(80, 24);
    /// assert!(vp.is_visible(10.0, 5.0, 20.0, 3.0));
    /// assert!(!vp.is_visible(200.0, 200.0, 10.0, 3.0));
    /// ```
    pub fn is_visible(&self, x: f64, y: f64, w: f64, h: f64) -> bool {
        let vw = self.viewport_width / self.zoom;
        let vh = self.viewport_height / self.zoom;

        x + w > self.offset_x
            && x < self.offset_x + vw
            && y + h > self.offset_y
            && y < self.offset_y + vh
    }

    /// Converts graph coordinates to screen coordinates.
    ///
    /// Returns `(i32, i32)` because the result may be negative
    /// (off-screen to the left/top).
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::Viewport2D;
    /// use ratatui::layout::Rect;
    ///
    /// let vp = Viewport2D::new();
    /// let area = Rect::new(0, 0, 80, 24);
    /// let (sx, sy) = vp.to_screen(10.0, 5.0, area);
    /// assert_eq!(sx, 10);
    /// assert_eq!(sy, 5);
    /// ```
    pub fn to_screen(&self, gx: f64, gy: f64, area: Rect) -> (i32, i32) {
        let sx = ((gx - self.offset_x) * self.zoom) as i32 + area.x as i32;
        let sy = ((gy - self.offset_y) * self.zoom) as i32 + area.y as i32;
        (sx, sy)
    }

    /// Converts screen coordinates to graph coordinates.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::Viewport2D;
    /// use ratatui::layout::Rect;
    ///
    /// let vp = Viewport2D::new();
    /// let area = Rect::new(0, 0, 80, 24);
    /// let (gx, gy) = vp.to_graph(10, 5, area);
    /// assert!((gx - 10.0).abs() < 0.001);
    /// assert!((gy - 5.0).abs() < 0.001);
    /// ```
    pub fn to_graph(&self, sx: u16, sy: u16, area: Rect) -> (f64, f64) {
        let gx = (f64::from(sx) - f64::from(area.x)) / self.zoom + self.offset_x;
        let gy = (f64::from(sy) - f64::from(area.y)) / self.zoom + self.offset_y;
        (gx, gy)
    }

    /// Returns whether the content exceeds the viewport (scrolling needed).
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::{Viewport2D, BoundingBox};
    ///
    /// let mut vp = Viewport2D::new();
    /// vp.set_viewport_size(80, 24);
    /// vp.set_content_bounds(BoundingBox::new(0.0, 0.0, 200.0, 100.0));
    /// assert!(vp.needs_scroll());
    /// ```
    pub fn needs_scroll(&self) -> bool {
        let cw = self.content_bbox.width() * self.zoom;
        let ch = self.content_bbox.height() * self.zoom;
        cw > self.viewport_width || ch > self.viewport_height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_viewport() {
        let vp = Viewport2D::new();
        assert_eq!(vp.offset_x(), 0.0);
        assert_eq!(vp.offset_y(), 0.0);
        assert_eq!(vp.zoom(), 1.0);
    }

    #[test]
    fn test_pan() {
        let mut vp = Viewport2D::new();
        vp.pan(10.0, 20.0);
        assert_eq!(vp.offset_x(), 10.0);
        assert_eq!(vp.offset_y(), 20.0);
    }

    #[test]
    fn test_zoom_clamp() {
        let mut vp = Viewport2D::new();

        // Zoom in many times, should clamp at MAX_ZOOM
        for _ in 0..20 {
            vp.zoom_in();
        }
        assert!(vp.zoom() <= Viewport2D::MAX_ZOOM);

        // Zoom out many times, should clamp at MIN_ZOOM
        for _ in 0..20 {
            vp.zoom_out();
        }
        assert!(vp.zoom() >= Viewport2D::MIN_ZOOM);
    }

    #[test]
    fn test_fit_to_content() {
        let mut vp = Viewport2D::new();
        vp.set_viewport_size(80, 24);
        vp.set_content_bounds(BoundingBox::new(0.0, 0.0, 160.0, 48.0));
        vp.fit_to_content();

        // Content is 2x wider and 2x taller, so zoom should be ~0.5
        assert!(vp.zoom() < 1.0);
        assert!(vp.zoom() > 0.0);
    }

    #[test]
    fn test_fit_empty_content() {
        let mut vp = Viewport2D::new();
        vp.set_viewport_size(80, 24);
        vp.set_content_bounds(BoundingBox::new(0.0, 0.0, 0.0, 0.0));
        vp.fit_to_content();

        assert_eq!(vp.zoom(), 1.0);
        assert_eq!(vp.offset_x(), 0.0);
    }

    #[test]
    fn test_ensure_visible() {
        let mut vp = Viewport2D::new();
        vp.set_viewport_size(80, 24);

        // Node at (100, 50) is off-screen
        assert!(!vp.is_visible(100.0, 50.0, 10.0, 3.0));

        vp.ensure_visible(100.0, 50.0, 10.0, 3.0);
        assert!(vp.is_visible(100.0, 50.0, 10.0, 3.0));
    }

    #[test]
    fn test_coordinate_roundtrip() {
        let vp = Viewport2D::new();
        let area = Rect::new(5, 3, 80, 24);

        let (sx, sy) = vp.to_screen(20.0, 10.0, area);
        let (gx, gy) = vp.to_graph(sx as u16, sy as u16, area);

        assert!((gx - 20.0).abs() < 0.001);
        assert!((gy - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_needs_scroll() {
        let mut vp = Viewport2D::new();
        vp.set_viewport_size(80, 24);

        // Small content — no scroll
        vp.set_content_bounds(BoundingBox::new(0.0, 0.0, 40.0, 10.0));
        assert!(!vp.needs_scroll());

        // Large content — needs scroll
        vp.set_content_bounds(BoundingBox::new(0.0, 0.0, 200.0, 100.0));
        assert!(vp.needs_scroll());
    }

    #[test]
    fn test_visibility_check() {
        let mut vp = Viewport2D::new();
        vp.set_viewport_size(80, 24);

        // Inside viewport
        assert!(vp.is_visible(10.0, 5.0, 20.0, 3.0));
        // Partially inside
        assert!(vp.is_visible(-5.0, -2.0, 20.0, 5.0));
        // Completely outside
        assert!(!vp.is_visible(200.0, 100.0, 10.0, 3.0));
        // Just at the edge
        assert!(!vp.is_visible(80.0, 0.0, 10.0, 3.0));
    }

    #[test]
    fn test_bounding_box() {
        let bbox = BoundingBox::new(10.0, 20.0, 50.0, 60.0);
        assert_eq!(bbox.width(), 40.0);
        assert_eq!(bbox.height(), 40.0);
    }
}
