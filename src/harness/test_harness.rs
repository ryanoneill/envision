//! Core test harness implementation.

use std::io;

use ratatui::Terminal;

use crate::annotation::{with_annotations, AnnotationRegistry, RegionInfo, WidgetType};
use crate::backend::CaptureBackend;
use crate::input::{EventQueue, Event};

use super::assertions::{Assertion, AssertionError, AssertionResult};
use super::snapshot::Snapshot;

/// Test harness for headless TUI testing.
///
/// The harness provides a unified interface for:
/// - Rendering UI to a capture backend
/// - Simulating user input
/// - Querying rendered content and annotations
/// - Making assertions about UI state
///
/// # Example
///
/// ```rust,no_run
/// use envision::harness::TestHarness;
/// use ratatui::widgets::Paragraph;
///
/// let mut harness = TestHarness::new(80, 24);
///
/// harness.render(|frame| {
///     frame.render_widget(Paragraph::new("Test"), frame.area());
/// });
///
/// assert!(harness.contains("Test"));
/// ```
pub struct TestHarness {
    terminal: Terminal<CaptureBackend>,
    events: EventQueue,
    annotations: AnnotationRegistry,
    frame_count: u64,
}

impl TestHarness {
    /// Creates a new test harness with the given dimensions.
    pub fn new(width: u16, height: u16) -> Self {
        let backend = CaptureBackend::new(width, height);
        let terminal = Terminal::new(backend).expect("Failed to create terminal");

        Self {
            terminal,
            events: EventQueue::new(),
            annotations: AnnotationRegistry::new(),
            frame_count: 0,
        }
    }

    /// Returns the width of the terminal.
    pub fn width(&self) -> u16 {
        self.terminal.backend().width()
    }

    /// Returns the height of the terminal.
    pub fn height(&self) -> u16 {
        self.terminal.backend().height()
    }

    /// Returns the current frame count.
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    // -------------------------------------------------------------------------
    // Rendering
    // -------------------------------------------------------------------------

    /// Renders a frame using the provided closure.
    ///
    /// This collects annotations during rendering and increments the frame count.
    pub fn render<F>(&mut self, f: F) -> io::Result<()>
    where
        F: FnOnce(&mut ratatui::Frame),
    {
        self.annotations = with_annotations(|| {
            self.terminal.draw(f).expect("Failed to draw");
        });
        self.frame_count += 1;
        Ok(())
    }

    /// Returns the current terminal content as plain text.
    pub fn screen(&self) -> String {
        self.terminal.backend().to_string()
    }

    /// Returns the content of a specific row.
    pub fn row(&self, y: u16) -> String {
        self.terminal.backend().row_content(y)
    }

    /// Returns a snapshot of the current frame.
    pub fn snapshot(&self) -> Snapshot {
        Snapshot::new(self.terminal.backend().snapshot(), self.annotations.clone())
    }

    /// Returns a reference to the backend.
    pub fn backend(&self) -> &CaptureBackend {
        self.terminal.backend()
    }

    /// Returns a mutable reference to the backend.
    pub fn backend_mut(&mut self) -> &mut CaptureBackend {
        self.terminal.backend_mut()
    }

    // -------------------------------------------------------------------------
    // Input Simulation
    // -------------------------------------------------------------------------

    /// Returns a reference to the event queue.
    pub fn events(&self) -> &EventQueue {
        &self.events
    }

    /// Returns a mutable reference to the event queue.
    pub fn events_mut(&mut self) -> &mut EventQueue {
        &mut self.events
    }

    /// Queues a single event.
    pub fn push_event(&mut self, event: Event) {
        self.events.push(event);
    }

    /// Pops the next event from the queue.
    pub fn pop_event(&mut self) -> Option<Event> {
        self.events.pop()
    }

    /// Types a string as keyboard input.
    pub fn type_str(&mut self, s: &str) {
        self.events.type_str(s);
    }

    /// Simulates pressing Enter.
    pub fn enter(&mut self) {
        self.events.enter();
    }

    /// Simulates pressing Escape.
    pub fn escape(&mut self) {
        self.events.escape();
    }

    /// Simulates pressing Tab.
    pub fn tab(&mut self) {
        self.events.tab();
    }

    /// Simulates `Ctrl+<key>`.
    pub fn ctrl(&mut self, c: char) {
        self.events.ctrl(c);
    }

    /// Simulates a mouse click at the given position.
    pub fn click(&mut self, x: u16, y: u16) {
        self.events.click(x, y);
    }

    /// Clears all pending events.
    pub fn clear_events(&mut self) {
        self.events.clear();
    }

    // -------------------------------------------------------------------------
    // Annotations
    // -------------------------------------------------------------------------

    /// Returns a reference to the annotation registry.
    pub fn annotations(&self) -> &AnnotationRegistry {
        &self.annotations
    }

    /// Returns the region at the given position.
    pub fn region_at(&self, x: u16, y: u16) -> Option<&RegionInfo> {
        self.annotations.region_at(x, y)
    }

    /// Finds regions by id.
    pub fn find_by_id(&self, id: &str) -> Vec<&RegionInfo> {
        self.annotations.find_by_id(id)
    }

    /// Gets the first region with the given id.
    pub fn get_by_id(&self, id: &str) -> Option<&RegionInfo> {
        self.annotations.get_by_id(id)
    }

    /// Finds regions by widget type.
    pub fn find_by_type(&self, widget_type: &WidgetType) -> Vec<&RegionInfo> {
        self.annotations.find_by_type(widget_type)
    }

    /// Returns the currently focused region.
    pub fn focused(&self) -> Option<&RegionInfo> {
        self.annotations.focused_region()
    }

    /// Returns all interactive regions.
    pub fn interactive(&self) -> Vec<&RegionInfo> {
        self.annotations.interactive_regions()
    }

    /// Clicks on a widget by id.
    ///
    /// Returns true if the widget was found and clicked.
    pub fn click_on(&mut self, id: &str) -> bool {
        if let Some(region) = self.annotations.get_by_id(id) {
            let x = region.area.x + region.area.width / 2;
            let y = region.area.y + region.area.height / 2;
            self.click(x, y);
            true
        } else {
            false
        }
    }

    // -------------------------------------------------------------------------
    // Content Queries
    // -------------------------------------------------------------------------

    /// Returns true if the screen contains the given text.
    pub fn contains(&self, needle: &str) -> bool {
        self.terminal.backend().contains_text(needle)
    }

    /// Finds the first position of text on screen.
    pub fn find_text(&self, needle: &str) -> Option<(u16, u16)> {
        self.terminal
            .backend()
            .find_text(needle)
            .first()
            .map(|p| (p.x, p.y))
    }

    /// Finds all positions of text on screen.
    pub fn find_all_text(&self, needle: &str) -> Vec<(u16, u16)> {
        self.terminal
            .backend()
            .find_text(needle)
            .iter()
            .map(|p| (p.x, p.y))
            .collect()
    }

    /// Returns the content within a rectangular region.
    pub fn region_content(&self, x: u16, y: u16, width: u16, height: u16) -> String {
        let mut lines = Vec::new();
        for row in y..y.saturating_add(height) {
            let row_content = self.row(row);
            let start = x as usize;
            let end = (x + width) as usize;
            if start < row_content.len() {
                let end = end.min(row_content.len());
                lines.push(row_content[start..end].to_string());
            }
        }
        lines.join("\n")
    }

    // -------------------------------------------------------------------------
    // Assertions
    // -------------------------------------------------------------------------

    /// Asserts that the screen contains the given text.
    ///
    /// # Panics
    ///
    /// Panics if the text is not found.
    pub fn assert_contains(&self, needle: &str) {
        if !self.contains(needle) {
            panic!(
                "Expected screen to contain '{}', but it was not found.\n\nScreen:\n{}",
                needle,
                self.screen()
            );
        }
    }

    /// Asserts that the screen does not contain the given text.
    ///
    /// # Panics
    ///
    /// Panics if the text is found.
    pub fn assert_not_contains(&self, needle: &str) {
        if self.contains(needle) {
            panic!(
                "Expected screen to NOT contain '{}', but it was found.\n\nScreen:\n{}",
                needle,
                self.screen()
            );
        }
    }

    /// Asserts that a widget with the given id exists.
    ///
    /// # Panics
    ///
    /// Panics if the widget is not found.
    pub fn assert_widget_exists(&self, id: &str) {
        if self.get_by_id(id).is_none() {
            panic!(
                "Expected widget with id '{}' to exist, but it was not found.\n\nAnnotations:\n{}",
                id,
                self.annotations.format_tree()
            );
        }
    }

    /// Asserts that a widget with the given id does not exist.
    ///
    /// # Panics
    ///
    /// Panics if the widget is found.
    pub fn assert_widget_not_exists(&self, id: &str) {
        if self.get_by_id(id).is_some() {
            panic!(
                "Expected widget with id '{}' to NOT exist, but it was found.\n\nAnnotations:\n{}",
                id,
                self.annotations.format_tree()
            );
        }
    }

    /// Asserts that a widget with the given id is focused.
    ///
    /// # Panics
    ///
    /// Panics if the widget is not focused or doesn't exist.
    pub fn assert_focused(&self, id: &str) {
        match self.get_by_id(id) {
            Some(region) if region.annotation.focused => {}
            Some(_) => panic!(
                "Expected widget '{}' to be focused, but it is not.\n\nAnnotations:\n{}",
                id,
                self.annotations.format_tree()
            ),
            None => panic!(
                "Expected widget '{}' to be focused, but it doesn't exist.\n\nAnnotations:\n{}",
                id,
                self.annotations.format_tree()
            ),
        }
    }

    /// Runs an assertion and returns the result.
    pub fn assert(&self, assertion: Assertion) -> AssertionResult {
        assertion.check(self)
    }

    /// Runs multiple assertions and returns all results.
    pub fn assert_all(&self, assertions: Vec<Assertion>) -> Vec<AssertionResult> {
        assertions.into_iter().map(|a| self.assert(a)).collect()
    }

    /// Runs multiple assertions, returning the first failure if any.
    pub fn assert_all_ok(&self, assertions: Vec<Assertion>) -> Result<(), AssertionError> {
        for assertion in assertions {
            self.assert(assertion)?;
        }
        Ok(())
    }
}

impl Default for TestHarness {
    fn default() -> Self {
        Self::new(80, 24)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::annotation::{Annotate, Annotation};
    use ratatui::widgets::Paragraph;

    #[test]
    fn test_harness_new() {
        let harness = TestHarness::new(80, 24);
        assert_eq!(harness.width(), 80);
        assert_eq!(harness.height(), 24);
        assert_eq!(harness.frame_count(), 0);
    }

    #[test]
    fn test_harness_render() {
        let mut harness = TestHarness::new(80, 24);

        harness
            .render(|frame| {
                frame.render_widget(Paragraph::new("Hello, World!"), frame.area());
            })
            .unwrap();

        assert_eq!(harness.frame_count(), 1);
        assert!(harness.contains("Hello, World!"));
    }

    #[test]
    fn test_harness_annotations() {
        let mut harness = TestHarness::new(80, 24);

        harness
            .render(|frame| {
                let area = frame.area();
                frame.render_widget(
                    Annotate::new(
                        Paragraph::new("Submit"),
                        Annotation::button("submit").with_label("Submit Button"),
                    ),
                    area,
                );
            })
            .unwrap();

        assert!(harness.get_by_id("submit").is_some());
        let buttons = harness.find_by_type(&WidgetType::Button);
        assert_eq!(buttons.len(), 1);
    }

    #[test]
    fn test_harness_input() {
        let mut harness = TestHarness::new(80, 24);

        harness.type_str("hello");
        harness.enter();

        assert_eq!(harness.events().len(), 6); // 5 chars + enter
    }

    #[test]
    fn test_harness_click_on() {
        let mut harness = TestHarness::new(80, 24);

        harness
            .render(|frame| {
                let area = ratatui::layout::Rect::new(10, 5, 20, 3);
                frame.render_widget(
                    Annotate::new(Paragraph::new("Click Me"), Annotation::button("btn")),
                    area,
                );
            })
            .unwrap();

        assert!(harness.click_on("btn"));
        assert_eq!(harness.events().len(), 1);

        assert!(!harness.click_on("nonexistent"));
    }

    #[test]
    fn test_harness_assert_contains() {
        let mut harness = TestHarness::new(80, 24);

        harness
            .render(|frame| {
                frame.render_widget(Paragraph::new("Expected Text"), frame.area());
            })
            .unwrap();

        harness.assert_contains("Expected Text");
        harness.assert_not_contains("Unexpected");
    }

    #[test]
    #[should_panic(expected = "Expected screen to contain")]
    fn test_harness_assert_contains_fails() {
        let harness = TestHarness::new(80, 24);
        harness.assert_contains("Not There");
    }

    #[test]
    fn test_harness_region_content() {
        let mut harness = TestHarness::new(80, 24);

        harness
            .render(|frame| {
                frame.render_widget(
                    Paragraph::new("ABCDEFGHIJ"),
                    ratatui::layout::Rect::new(0, 0, 10, 1),
                );
            })
            .unwrap();

        let content = harness.region_content(2, 0, 4, 1);
        assert_eq!(content, "CDEF");
    }

    #[test]
    fn test_harness_focused() {
        let mut harness = TestHarness::new(80, 24);

        harness
            .render(|frame| {
                let area = frame.area();
                frame.render_widget(
                    Annotate::new(
                        Paragraph::new("Input"),
                        Annotation::input("name").with_focus(true),
                    ),
                    area,
                );
            })
            .unwrap();

        let focused = harness.focused().unwrap();
        assert!(focused.annotation.has_id("name"));
    }

    #[test]
    fn test_harness_default() {
        let harness = TestHarness::default();
        assert_eq!(harness.width(), 80);
        assert_eq!(harness.height(), 24);
    }

    #[test]
    fn test_harness_row() {
        let mut harness = TestHarness::new(20, 5);

        harness
            .render(|frame| {
                frame.render_widget(
                    Paragraph::new("Line One"),
                    ratatui::layout::Rect::new(0, 0, 20, 1),
                );
            })
            .unwrap();

        let row = harness.row(0);
        assert!(row.contains("Line One"));
    }

    #[test]
    fn test_harness_snapshot() {
        let mut harness = TestHarness::new(20, 5);

        harness
            .render(|frame| {
                frame.render_widget(
                    Annotate::new(Paragraph::new("Test"), Annotation::button("btn")),
                    frame.area(),
                );
            })
            .unwrap();

        let snapshot = harness.snapshot();
        // Verify we can get content from the snapshot
        let text = snapshot.to_plain();
        assert!(text.contains("Test"));
    }

    #[test]
    fn test_harness_backend() {
        let harness = TestHarness::new(80, 24);

        let backend = harness.backend();
        assert_eq!(backend.width(), 80);
        assert_eq!(backend.height(), 24);
    }

    #[test]
    fn test_harness_backend_mut() {
        let mut harness = TestHarness::new(80, 24);

        let backend = harness.backend_mut();
        assert_eq!(backend.width(), 80);
    }

    #[test]
    fn test_harness_events_ref() {
        let harness = TestHarness::new(80, 24);
        let events = harness.events();
        assert!(events.is_empty());
    }

    #[test]
    fn test_harness_events_mut() {
        let mut harness = TestHarness::new(80, 24);

        harness.events_mut().push(Event::char('a'));
        assert_eq!(harness.events().len(), 1);
    }

    #[test]
    fn test_harness_push_pop_event() {
        let mut harness = TestHarness::new(80, 24);

        harness.push_event(Event::char('x'));
        harness.push_event(Event::char('y'));

        let event1 = harness.pop_event().unwrap();
        assert!(event1.is_key());

        let event2 = harness.pop_event().unwrap();
        assert!(event2.is_key());

        assert!(harness.pop_event().is_none());
    }

    #[test]
    fn test_harness_escape() {
        let mut harness = TestHarness::new(80, 24);
        harness.escape();
        assert_eq!(harness.events().len(), 1);
    }

    #[test]
    fn test_harness_tab() {
        let mut harness = TestHarness::new(80, 24);
        harness.tab();
        assert_eq!(harness.events().len(), 1);
    }

    #[test]
    fn test_harness_ctrl() {
        let mut harness = TestHarness::new(80, 24);
        harness.ctrl('c');
        assert_eq!(harness.events().len(), 1);
    }

    #[test]
    fn test_harness_clear_events() {
        let mut harness = TestHarness::new(80, 24);

        harness.type_str("abc");
        assert_eq!(harness.events().len(), 3);

        harness.clear_events();
        assert!(harness.events().is_empty());
    }

    #[test]
    fn test_harness_region_at() {
        let mut harness = TestHarness::new(80, 24);

        harness
            .render(|frame| {
                let area = ratatui::layout::Rect::new(10, 10, 20, 5);
                frame.render_widget(
                    Annotate::new(Paragraph::new("Button"), Annotation::button("btn")),
                    area,
                );
            })
            .unwrap();

        // Inside the widget
        let region = harness.region_at(15, 12);
        assert!(region.is_some());

        // Outside the widget
        let region = harness.region_at(0, 0);
        assert!(region.is_none());
    }

    #[test]
    fn test_harness_find_by_id() {
        let mut harness = TestHarness::new(80, 24);

        harness
            .render(|frame| {
                let area1 = ratatui::layout::Rect::new(0, 0, 20, 1);
                let area2 = ratatui::layout::Rect::new(0, 1, 20, 1);

                frame.render_widget(
                    Annotate::new(Paragraph::new("A"), Annotation::button("btn")),
                    area1,
                );
                frame.render_widget(
                    Annotate::new(Paragraph::new("B"), Annotation::button("btn")),
                    area2,
                );
            })
            .unwrap();

        let regions = harness.find_by_id("btn");
        assert_eq!(regions.len(), 2);

        let regions = harness.find_by_id("nonexistent");
        assert!(regions.is_empty());
    }

    #[test]
    fn test_harness_interactive() {
        let mut harness = TestHarness::new(80, 24);

        harness
            .render(|frame| {
                let area1 = ratatui::layout::Rect::new(0, 0, 20, 1);
                let area2 = ratatui::layout::Rect::new(0, 1, 20, 1);

                // Button is interactive
                frame.render_widget(
                    Annotate::new(Paragraph::new("A"), Annotation::button("btn")),
                    area1,
                );
                // Container is not interactive by default
                frame.render_widget(
                    Annotate::new(Paragraph::new("B"), Annotation::container("container")),
                    area2,
                );
            })
            .unwrap();

        let interactive = harness.interactive();
        assert_eq!(interactive.len(), 1);
    }

    #[test]
    fn test_harness_find_text() {
        let mut harness = TestHarness::new(40, 5);

        harness
            .render(|frame| {
                frame.render_widget(
                    Paragraph::new("Hello World"),
                    ratatui::layout::Rect::new(5, 2, 20, 1),
                );
            })
            .unwrap();

        let pos = harness.find_text("Hello");
        assert!(pos.is_some());
        let (x, y) = pos.unwrap();
        assert_eq!(x, 5);
        assert_eq!(y, 2);

        assert!(harness.find_text("Missing").is_none());
    }

    #[test]
    fn test_harness_find_all_text() {
        let mut harness = TestHarness::new(40, 5);

        harness
            .render(|frame| {
                frame.render_widget(
                    Paragraph::new("XX XX"),
                    ratatui::layout::Rect::new(0, 0, 10, 1),
                );
            })
            .unwrap();

        let positions = harness.find_all_text("XX");
        assert_eq!(positions.len(), 2);
    }

    #[test]
    fn test_harness_assert_widget_exists() {
        let mut harness = TestHarness::new(80, 24);

        harness
            .render(|frame| {
                frame.render_widget(
                    Annotate::new(Paragraph::new("Btn"), Annotation::button("btn")),
                    frame.area(),
                );
            })
            .unwrap();

        harness.assert_widget_exists("btn");
    }

    #[test]
    #[should_panic(expected = "Expected widget with id")]
    fn test_harness_assert_widget_exists_fails() {
        let harness = TestHarness::new(80, 24);
        harness.assert_widget_exists("nonexistent");
    }

    #[test]
    fn test_harness_assert_widget_not_exists() {
        let harness = TestHarness::new(80, 24);
        harness.assert_widget_not_exists("nonexistent");
    }

    #[test]
    #[should_panic(expected = "Expected widget with id")]
    fn test_harness_assert_widget_not_exists_fails() {
        let mut harness = TestHarness::new(80, 24);

        harness
            .render(|frame| {
                frame.render_widget(
                    Annotate::new(Paragraph::new("Btn"), Annotation::button("btn")),
                    frame.area(),
                );
            })
            .unwrap();

        harness.assert_widget_not_exists("btn");
    }

    #[test]
    #[should_panic(expected = "Expected screen to NOT contain")]
    fn test_harness_assert_not_contains_fails() {
        let mut harness = TestHarness::new(80, 24);

        harness
            .render(|frame| {
                frame.render_widget(Paragraph::new("Found"), frame.area());
            })
            .unwrap();

        harness.assert_not_contains("Found");
    }

    #[test]
    fn test_harness_assert_focused() {
        let mut harness = TestHarness::new(80, 24);

        harness
            .render(|frame| {
                frame.render_widget(
                    Annotate::new(
                        Paragraph::new("Input"),
                        Annotation::input("name").with_focus(true),
                    ),
                    frame.area(),
                );
            })
            .unwrap();

        harness.assert_focused("name");
    }

    #[test]
    #[should_panic(expected = "Expected widget")]
    fn test_harness_assert_focused_fails_not_focused() {
        let mut harness = TestHarness::new(80, 24);

        harness
            .render(|frame| {
                frame.render_widget(
                    Annotate::new(
                        Paragraph::new("Input"),
                        Annotation::input("name").with_focus(false),
                    ),
                    frame.area(),
                );
            })
            .unwrap();

        harness.assert_focused("name");
    }

    #[test]
    #[should_panic(expected = "doesn't exist")]
    fn test_harness_assert_focused_fails_not_found() {
        let harness = TestHarness::new(80, 24);
        harness.assert_focused("nonexistent");
    }

    #[test]
    fn test_harness_assert_declarative() {
        let mut harness = TestHarness::new(80, 24);

        harness
            .render(|frame| {
                frame.render_widget(Paragraph::new("Hello"), frame.area());
            })
            .unwrap();

        let result = harness.assert(Assertion::contains("Hello"));
        assert!(result.is_ok());

        let result = harness.assert(Assertion::contains("Missing"));
        assert!(result.is_err());
    }

    #[test]
    fn test_harness_assert_all() {
        let mut harness = TestHarness::new(80, 24);

        harness
            .render(|frame| {
                frame.render_widget(Paragraph::new("Hello World"), frame.area());
            })
            .unwrap();

        let results = harness.assert_all(vec![
            Assertion::contains("Hello"),
            Assertion::contains("World"),
            Assertion::contains("Missing"),
        ]);

        assert_eq!(results.len(), 3);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
        assert!(results[2].is_err());
    }

    #[test]
    fn test_harness_assert_all_ok_success() {
        let mut harness = TestHarness::new(80, 24);

        harness
            .render(|frame| {
                frame.render_widget(Paragraph::new("Hello World"), frame.area());
            })
            .unwrap();

        let result = harness.assert_all_ok(vec![
            Assertion::contains("Hello"),
            Assertion::contains("World"),
        ]);

        assert!(result.is_ok());
    }

    #[test]
    fn test_harness_assert_all_ok_failure() {
        let mut harness = TestHarness::new(80, 24);

        harness
            .render(|frame| {
                frame.render_widget(Paragraph::new("Hello"), frame.area());
            })
            .unwrap();

        let result = harness.assert_all_ok(vec![
            Assertion::contains("Hello"),
            Assertion::contains("Missing"),
        ]);

        assert!(result.is_err());
    }

    #[test]
    fn test_harness_region_content_out_of_bounds() {
        let mut harness = TestHarness::new(10, 3);

        harness
            .render(|frame| {
                frame.render_widget(
                    Paragraph::new("Hi"),
                    ratatui::layout::Rect::new(0, 0, 10, 1),
                );
            })
            .unwrap();

        // Should handle out of bounds gracefully
        let content = harness.region_content(100, 0, 5, 1);
        assert!(content.is_empty());
    }

    #[test]
    fn test_harness_click() {
        let mut harness = TestHarness::new(80, 24);

        harness.click(10, 20);
        assert_eq!(harness.events().len(), 1);
    }

    #[test]
    fn test_harness_screen() {
        let mut harness = TestHarness::new(20, 2);

        harness
            .render(|frame| {
                frame.render_widget(
                    Paragraph::new("Test"),
                    ratatui::layout::Rect::new(0, 0, 10, 1),
                );
            })
            .unwrap();

        let screen = harness.screen();
        assert!(screen.contains("Test"));
    }
}
