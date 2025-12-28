//! Snapshot testing support.

use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::annotation::AnnotationRegistry;
use crate::backend::FrameSnapshot;

/// Format for snapshot output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SnapshotFormat {
    /// Plain text (screen content only)
    #[default]
    Plain,

    /// ANSI-colored text
    Ansi,

    /// JSON with full metadata
    Json,

    /// JSON (pretty-printed)
    JsonPretty,
}

/// A complete snapshot of UI state.
///
/// Includes both the rendered frame and annotation data.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Snapshot {
    /// The captured frame data
    pub frame: FrameSnapshot,

    /// Annotations for this frame
    pub annotations: AnnotationRegistry,
}

impl Snapshot {
    /// Creates a new snapshot.
    pub fn new(frame: FrameSnapshot, annotations: AnnotationRegistry) -> Self {
        Self { frame, annotations }
    }

    /// Returns the plain text representation.
    pub fn to_plain(&self) -> String {
        self.frame.to_plain()
    }

    /// Returns the ANSI-colored representation.
    pub fn to_ansi(&self) -> String {
        self.frame.to_ansi()
    }

    /// Returns the JSON representation.
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }

    /// Returns the pretty-printed JSON representation.
    pub fn to_json_pretty(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }

    /// Formats the snapshot according to the specified format.
    pub fn format(&self, format: SnapshotFormat) -> String {
        match format {
            SnapshotFormat::Plain => self.to_plain(),
            SnapshotFormat::Ansi => self.to_ansi(),
            SnapshotFormat::Json => self.to_json().unwrap_or_default(),
            SnapshotFormat::JsonPretty => self.to_json_pretty().unwrap_or_default(),
        }
    }

    /// Writes the snapshot to a file.
    pub fn write_to_file(&self, path: impl AsRef<Path>, format: SnapshotFormat) -> io::Result<()> {
        let content = self.format(format);
        std::fs::write(path, content)
    }

    /// Loads a snapshot from a JSON file.
    pub fn load_from_file(path: impl AsRef<Path>) -> io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        serde_json::from_str(&content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    /// Compares this snapshot to another.
    pub fn diff(&self, other: &Snapshot) -> SnapshotDiff {
        SnapshotDiff::compute(self, other)
    }

    /// Returns true if this snapshot matches another exactly.
    pub fn matches(&self, other: &Snapshot) -> bool {
        self.to_plain() == other.to_plain()
    }

    /// Returns a formatted tree of annotations.
    pub fn annotation_tree(&self) -> String {
        self.annotations.format_tree()
    }

    /// Returns the number of annotations.
    pub fn annotation_count(&self) -> usize {
        self.annotations.len()
    }
}

/// Difference between two snapshots.
#[derive(Debug, Clone)]
pub struct SnapshotDiff {
    /// Lines that differ
    pub changed_lines: Vec<LineDiff>,

    /// Whether the annotations differ
    pub annotations_differ: bool,

    /// Number of changed lines
    pub changes: usize,
}

/// A single line difference.
#[derive(Debug, Clone)]
pub struct LineDiff {
    /// Line number (0-indexed)
    pub line: usize,

    /// Content in the first snapshot
    pub left: String,

    /// Content in the second snapshot
    pub right: String,
}

impl SnapshotDiff {
    /// Computes the diff between two snapshots.
    pub fn compute(left: &Snapshot, right: &Snapshot) -> Self {
        let left_plain = left.to_plain();
        let right_plain = right.to_plain();

        let left_lines: Vec<&str> = left_plain.lines().collect();
        let right_lines: Vec<&str> = right_plain.lines().collect();

        let max_lines = left_lines.len().max(right_lines.len());
        let mut changed_lines = Vec::new();

        for i in 0..max_lines {
            let l = left_lines.get(i).copied().unwrap_or("");
            let r = right_lines.get(i).copied().unwrap_or("");

            if l != r {
                changed_lines.push(LineDiff {
                    line: i,
                    left: l.to_string(),
                    right: r.to_string(),
                });
            }
        }

        let annotations_differ = left.annotations.format_tree() != right.annotations.format_tree();

        Self {
            changes: changed_lines.len(),
            changed_lines,
            annotations_differ,
        }
    }

    /// Returns true if the snapshots are identical.
    pub fn is_empty(&self) -> bool {
        self.changes == 0 && !self.annotations_differ
    }

    /// Formats the diff for display.
    pub fn format(&self) -> String {
        let mut output = String::new();

        if self.changed_lines.is_empty() && !self.annotations_differ {
            output.push_str("No differences\n");
            return output;
        }

        if !self.changed_lines.is_empty() {
            output.push_str(&format!("Changed lines ({}):\n", self.changes));
            for diff in &self.changed_lines {
                output.push_str(&format!("  Line {}:\n", diff.line + 1));
                output.push_str(&format!("    - {}\n", diff.left));
                output.push_str(&format!("    + {}\n", diff.right));
            }
        }

        if self.annotations_differ {
            output.push_str("Annotations differ\n");
        }

        output
    }
}

/// Asserts that two snapshots match.
///
/// # Panics
///
/// Panics with a diff if the snapshots differ.
#[allow(dead_code)]
pub fn assert_snapshot_eq(left: &Snapshot, right: &Snapshot) {
    let diff = left.diff(right);
    if !diff.is_empty() {
        panic!("Snapshots differ:\n{}", diff.format());
    }
}

/// Asserts that a snapshot matches an expected string.
///
/// # Panics
///
/// Panics if the snapshot's plain text doesn't match.
#[allow(dead_code)]
pub fn assert_snapshot_text(snapshot: &Snapshot, expected: &str) {
    let actual = snapshot.to_plain();
    if actual != expected {
        panic!(
            "Snapshot text differs:\n\nExpected:\n{}\n\nActual:\n{}",
            expected, actual
        );
    }
}

/// Helper for snapshot testing with file storage.
#[derive(Debug)]
#[allow(dead_code)]
pub struct SnapshotTest {
    /// Directory for snapshot files
    pub snapshot_dir: std::path::PathBuf,

    /// Format for snapshot files
    pub format: SnapshotFormat,

    /// Whether to update snapshots
    pub update: bool,
}

#[allow(dead_code)]
impl SnapshotTest {
    /// Creates a new snapshot test helper.
    pub fn new(snapshot_dir: impl AsRef<Path>) -> Self {
        Self {
            snapshot_dir: snapshot_dir.as_ref().to_path_buf(),
            format: SnapshotFormat::Plain,
            update: false,
        }
    }

    /// Sets the snapshot format.
    pub fn with_format(mut self, format: SnapshotFormat) -> Self {
        self.format = format;
        self
    }

    /// Enables update mode (overwrites existing snapshots).
    pub fn with_update(mut self, update: bool) -> Self {
        self.update = update;
        self
    }

    /// Returns the path for a snapshot file.
    pub fn snapshot_path(&self, name: &str) -> std::path::PathBuf {
        let ext = match self.format {
            SnapshotFormat::Plain => "txt",
            SnapshotFormat::Ansi => "ansi",
            SnapshotFormat::Json | SnapshotFormat::JsonPretty => "json",
        };
        self.snapshot_dir.join(format!("{}.{}", name, ext))
    }

    /// Asserts that a snapshot matches the stored version.
    ///
    /// If update mode is enabled, overwrites the stored version.
    pub fn assert(&self, name: &str, snapshot: &Snapshot) -> io::Result<()> {
        let path = self.snapshot_path(name);

        if self.update || !path.exists() {
            std::fs::create_dir_all(&self.snapshot_dir)?;
            snapshot.write_to_file(&path, self.format)?;
            return Ok(());
        }

        let expected = std::fs::read_to_string(&path)?;
        let actual = snapshot.format(self.format);

        if actual != expected {
            // Write actual to a .new file for comparison
            let new_path = path.with_extension(format!(
                "{}.new",
                path.extension().unwrap_or_default().to_string_lossy()
            ));
            std::fs::write(&new_path, &actual)?;

            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "Snapshot '{}' differs. New snapshot written to {:?}",
                    name, new_path
                ),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::harness::TestHarness;
    use ratatui::widgets::Paragraph;

    #[test]
    fn test_snapshot_creation() {
        let mut harness = TestHarness::new(40, 5);
        harness.render(|frame| {
            frame.render_widget(Paragraph::new("Test"), frame.area());
        }).unwrap();

        let snapshot = harness.snapshot();
        assert!(snapshot.to_plain().contains("Test"));
    }

    #[test]
    fn test_snapshot_formats() {
        let mut harness = TestHarness::new(20, 2);
        harness.render(|frame| {
            frame.render_widget(Paragraph::new("Hello"), frame.area());
        }).unwrap();

        let snapshot = harness.snapshot();

        // Plain
        let plain = snapshot.format(SnapshotFormat::Plain);
        assert!(plain.contains("Hello"));

        // JSON
        let json = snapshot.format(SnapshotFormat::Json);
        assert!(json.starts_with("{"));

        // JSON Pretty
        let json_pretty = snapshot.format(SnapshotFormat::JsonPretty);
        assert!(json_pretty.contains("\n"));
    }

    #[test]
    fn test_snapshot_diff() {
        let mut harness1 = TestHarness::new(20, 2);
        harness1.render(|frame| {
            frame.render_widget(Paragraph::new("Hello"), frame.area());
        }).unwrap();

        let mut harness2 = TestHarness::new(20, 2);
        harness2.render(|frame| {
            frame.render_widget(Paragraph::new("World"), frame.area());
        }).unwrap();

        let snap1 = harness1.snapshot();
        let snap2 = harness2.snapshot();

        let diff = snap1.diff(&snap2);
        assert!(!diff.is_empty());
        assert!(!diff.changed_lines.is_empty());
    }

    #[test]
    fn test_snapshot_matches() {
        let mut harness1 = TestHarness::new(20, 2);
        harness1.render(|frame| {
            frame.render_widget(Paragraph::new("Same"), frame.area());
        }).unwrap();

        let mut harness2 = TestHarness::new(20, 2);
        harness2.render(|frame| {
            frame.render_widget(Paragraph::new("Same"), frame.area());
        }).unwrap();

        let snap1 = harness1.snapshot();
        let snap2 = harness2.snapshot();

        assert!(snap1.matches(&snap2));
    }

    #[test]
    fn test_snapshot_diff_format() {
        let mut harness1 = TestHarness::new(20, 2);
        harness1.render(|frame| {
            frame.render_widget(Paragraph::new("A"), frame.area());
        }).unwrap();

        let mut harness2 = TestHarness::new(20, 2);
        harness2.render(|frame| {
            frame.render_widget(Paragraph::new("B"), frame.area());
        }).unwrap();

        let diff = harness1.snapshot().diff(&harness2.snapshot());
        let formatted = diff.format();

        assert!(formatted.contains("Changed lines"));
        assert!(formatted.contains("Line 1"));
    }

    #[test]
    fn test_empty_diff() {
        let mut harness = TestHarness::new(20, 2);
        harness.render(|frame| {
            frame.render_widget(Paragraph::new("Test"), frame.area());
        }).unwrap();

        let snap = harness.snapshot();
        let diff = snap.diff(&snap);

        assert!(diff.is_empty());
        assert_eq!(diff.format(), "No differences\n");
    }

    #[test]
    fn test_snapshot_serialization() {
        let mut harness = TestHarness::new(20, 2);
        harness.render(|frame| {
            frame.render_widget(Paragraph::new("Serialize"), frame.area());
        }).unwrap();

        let snapshot = harness.snapshot();
        let json = snapshot.to_json().unwrap();

        let deserialized: Snapshot = serde_json::from_str(&json).unwrap();
        assert!(deserialized.matches(&snapshot));
    }

    #[test]
    fn test_snapshot_format_default() {
        let format: SnapshotFormat = SnapshotFormat::default();
        assert_eq!(format, SnapshotFormat::Plain);
    }

    #[test]
    fn test_snapshot_format_debug() {
        let format = SnapshotFormat::Json;
        let debug = format!("{:?}", format);
        assert!(debug.contains("Json"));
    }

    #[test]
    fn test_snapshot_format_clone() {
        let format = SnapshotFormat::Ansi;
        let cloned = format;
        assert_eq!(format, cloned);
    }

    #[test]
    fn test_snapshot_to_json() {
        let mut harness = TestHarness::new(20, 2);
        harness
            .render(|frame| {
                frame.render_widget(Paragraph::new("JSON"), frame.area());
            })
            .unwrap();

        let snapshot = harness.snapshot();
        let json = snapshot.to_json();
        assert!(json.is_ok());
        let content = json.unwrap();
        // JSON contains the frame field
        assert!(content.contains("frame"));
        assert!(content.starts_with('{'));
    }

    #[test]
    fn test_snapshot_to_json_pretty() {
        let mut harness = TestHarness::new(20, 2);
        harness
            .render(|frame| {
                frame.render_widget(Paragraph::new("Pretty"), frame.area());
            })
            .unwrap();

        let snapshot = harness.snapshot();
        let json = snapshot.to_json_pretty();
        assert!(json.is_ok());
        let content = json.unwrap();
        // Pretty has newlines and indentation
        assert!(content.contains('\n'));
        assert!(content.contains("frame"));
    }

    #[test]
    fn test_snapshot_to_ansi() {
        let mut harness = TestHarness::new(20, 2);
        harness
            .render(|frame| {
                frame.render_widget(Paragraph::new("ANSI"), frame.area());
            })
            .unwrap();

        let snapshot = harness.snapshot();
        let ansi = snapshot.to_ansi();
        assert!(ansi.contains("ANSI"));
    }

    #[test]
    fn test_snapshot_annotation_tree() {
        use crate::annotation::{Annotate, Annotation};

        let mut harness = TestHarness::new(20, 2);
        harness
            .render(|frame| {
                frame.render_widget(
                    Annotate::new(Paragraph::new("Button"), Annotation::button("btn")),
                    frame.area(),
                );
            })
            .unwrap();

        let snapshot = harness.snapshot();
        let tree = snapshot.annotation_tree();
        assert!(!tree.is_empty());
    }

    #[test]
    fn test_snapshot_annotation_count() {
        use crate::annotation::{Annotate, Annotation};

        let mut harness = TestHarness::new(40, 3);
        harness
            .render(|frame| {
                let area1 = ratatui::layout::Rect::new(0, 0, 20, 1);
                let area2 = ratatui::layout::Rect::new(0, 1, 20, 1);

                frame.render_widget(
                    Annotate::new(Paragraph::new("A"), Annotation::button("a")),
                    area1,
                );
                frame.render_widget(
                    Annotate::new(Paragraph::new("B"), Annotation::button("b")),
                    area2,
                );
            })
            .unwrap();

        let snapshot = harness.snapshot();
        assert_eq!(snapshot.annotation_count(), 2);
    }

    #[test]
    fn test_snapshot_write_and_load() {
        use std::fs;
        use tempfile::TempDir;

        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("test.json");

        let mut harness = TestHarness::new(20, 2);
        harness
            .render(|frame| {
                frame.render_widget(Paragraph::new("File IO"), frame.area());
            })
            .unwrap();

        let snapshot = harness.snapshot();
        snapshot
            .write_to_file(&path, SnapshotFormat::Json)
            .unwrap();

        assert!(path.exists());

        let loaded = Snapshot::load_from_file(&path).unwrap();
        assert!(loaded.matches(&snapshot));

        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_snapshot_load_invalid_file() {
        use tempfile::TempDir;

        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("invalid.json");
        std::fs::write(&path, "not valid json").unwrap();

        let result = Snapshot::load_from_file(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_snapshot_load_nonexistent_file() {
        let result = Snapshot::load_from_file("/nonexistent/path/file.json");
        assert!(result.is_err());
    }

    #[test]
    fn test_snapshot_diff_clone() {
        let mut harness1 = TestHarness::new(20, 2);
        harness1
            .render(|frame| {
                frame.render_widget(Paragraph::new("A"), frame.area());
            })
            .unwrap();

        let mut harness2 = TestHarness::new(20, 2);
        harness2
            .render(|frame| {
                frame.render_widget(Paragraph::new("B"), frame.area());
            })
            .unwrap();

        let diff = harness1.snapshot().diff(&harness2.snapshot());
        let cloned = diff.clone();

        assert_eq!(diff.changes, cloned.changes);
    }

    #[test]
    fn test_snapshot_diff_debug() {
        let mut harness = TestHarness::new(20, 2);
        harness
            .render(|frame| {
                frame.render_widget(Paragraph::new("Test"), frame.area());
            })
            .unwrap();

        let diff = harness.snapshot().diff(&harness.snapshot());
        let debug = format!("{:?}", diff);
        assert!(debug.contains("SnapshotDiff"));
    }

    #[test]
    fn test_line_diff_clone() {
        let diff = LineDiff {
            line: 0,
            left: "hello".to_string(),
            right: "world".to_string(),
        };
        let cloned = diff.clone();
        assert_eq!(diff.line, cloned.line);
        assert_eq!(diff.left, cloned.left);
        assert_eq!(diff.right, cloned.right);
    }

    #[test]
    fn test_line_diff_debug() {
        let diff = LineDiff {
            line: 1,
            left: "a".to_string(),
            right: "b".to_string(),
        };
        let debug = format!("{:?}", diff);
        assert!(debug.contains("LineDiff"));
    }

    #[test]
    fn test_snapshot_diff_annotations_differ() {
        use crate::annotation::{Annotate, Annotation};

        let mut harness1 = TestHarness::new(20, 2);
        harness1
            .render(|frame| {
                frame.render_widget(
                    Annotate::new(Paragraph::new("A"), Annotation::button("a")),
                    frame.area(),
                );
            })
            .unwrap();

        let mut harness2 = TestHarness::new(20, 2);
        harness2
            .render(|frame| {
                frame.render_widget(
                    Annotate::new(Paragraph::new("A"), Annotation::button("b")), // Different ID
                    frame.area(),
                );
            })
            .unwrap();

        let diff = harness1.snapshot().diff(&harness2.snapshot());
        assert!(diff.annotations_differ);
    }

    #[test]
    fn test_snapshot_diff_format_annotations_differ() {
        use crate::annotation::{Annotate, Annotation};

        let mut harness1 = TestHarness::new(20, 2);
        harness1
            .render(|frame| {
                frame.render_widget(
                    Annotate::new(Paragraph::new("A"), Annotation::button("a")),
                    frame.area(),
                );
            })
            .unwrap();

        let mut harness2 = TestHarness::new(20, 2);
        harness2
            .render(|frame| {
                frame.render_widget(
                    Annotate::new(Paragraph::new("A"), Annotation::button("b")),
                    frame.area(),
                );
            })
            .unwrap();

        let diff = harness1.snapshot().diff(&harness2.snapshot());
        let formatted = diff.format();
        assert!(formatted.contains("Annotations differ"));
    }

    #[test]
    fn test_snapshot_test_new() {
        let tester = SnapshotTest::new("/tmp/snapshots");
        assert_eq!(tester.format, SnapshotFormat::Plain);
        assert!(!tester.update);
    }

    #[test]
    fn test_snapshot_test_with_format() {
        let tester = SnapshotTest::new("/tmp/snapshots").with_format(SnapshotFormat::Json);
        assert_eq!(tester.format, SnapshotFormat::Json);
    }

    #[test]
    fn test_snapshot_test_with_update() {
        let tester = SnapshotTest::new("/tmp/snapshots").with_update(true);
        assert!(tester.update);
    }

    #[test]
    fn test_snapshot_test_path() {
        let tester = SnapshotTest::new("/tmp/snapshots");
        let path = tester.snapshot_path("test");
        assert_eq!(path, std::path::PathBuf::from("/tmp/snapshots/test.txt"));

        let tester_json = tester.with_format(SnapshotFormat::Json);
        let path = tester_json.snapshot_path("test");
        assert_eq!(path, std::path::PathBuf::from("/tmp/snapshots/test.json"));

        let tester_ansi = SnapshotTest::new("/tmp/snapshots").with_format(SnapshotFormat::Ansi);
        let path = tester_ansi.snapshot_path("test");
        assert_eq!(path, std::path::PathBuf::from("/tmp/snapshots/test.ansi"));
    }

    #[test]
    fn test_snapshot_test_assert_creates_new() {
        use tempfile::TempDir;

        let tmp = TempDir::new().unwrap();
        let tester = SnapshotTest::new(tmp.path()).with_format(SnapshotFormat::Plain);

        let mut harness = TestHarness::new(20, 2);
        harness
            .render(|frame| {
                frame.render_widget(Paragraph::new("New"), frame.area());
            })
            .unwrap();

        let snapshot = harness.snapshot();
        let result = tester.assert("new_test", &snapshot);
        assert!(result.is_ok());

        // File should exist now
        let path = tester.snapshot_path("new_test");
        assert!(path.exists());
    }

    #[test]
    fn test_snapshot_test_assert_matches() {
        use tempfile::TempDir;

        let tmp = TempDir::new().unwrap();
        let tester = SnapshotTest::new(tmp.path()).with_format(SnapshotFormat::Plain);

        let mut harness = TestHarness::new(20, 2);
        harness
            .render(|frame| {
                frame.render_widget(Paragraph::new("Match"), frame.area());
            })
            .unwrap();

        let snapshot = harness.snapshot();

        // Create initial snapshot
        tester.assert("match_test", &snapshot).unwrap();

        // Assert same snapshot again
        let result = tester.assert("match_test", &snapshot);
        assert!(result.is_ok());
    }

    #[test]
    fn test_snapshot_test_assert_differs() {
        use tempfile::TempDir;

        let tmp = TempDir::new().unwrap();
        let tester = SnapshotTest::new(tmp.path()).with_format(SnapshotFormat::Plain);

        let mut harness1 = TestHarness::new(20, 2);
        harness1
            .render(|frame| {
                frame.render_widget(Paragraph::new("First"), frame.area());
            })
            .unwrap();

        let mut harness2 = TestHarness::new(20, 2);
        harness2
            .render(|frame| {
                frame.render_widget(Paragraph::new("Second"), frame.area());
            })
            .unwrap();

        // Create initial snapshot
        tester.assert("differ_test", &harness1.snapshot()).unwrap();

        // Assert different snapshot - should fail
        let result = tester.assert("differ_test", &harness2.snapshot());
        assert!(result.is_err());

        // Check that .new file was created
        let new_path = tmp.path().join("differ_test.txt.new");
        assert!(new_path.exists());
    }

    #[test]
    fn test_snapshot_test_update_mode() {
        use tempfile::TempDir;

        let tmp = TempDir::new().unwrap();
        let tester = SnapshotTest::new(tmp.path())
            .with_format(SnapshotFormat::Plain)
            .with_update(true);

        let mut harness1 = TestHarness::new(20, 2);
        harness1
            .render(|frame| {
                frame.render_widget(Paragraph::new("Original"), frame.area());
            })
            .unwrap();

        let mut harness2 = TestHarness::new(20, 2);
        harness2
            .render(|frame| {
                frame.render_widget(Paragraph::new("Updated"), frame.area());
            })
            .unwrap();

        // Create initial snapshot
        tester.assert("update_test", &harness1.snapshot()).unwrap();

        // Update with different snapshot - should succeed in update mode
        let result = tester.assert("update_test", &harness2.snapshot());
        assert!(result.is_ok());

        // File should now contain updated content
        let path = tester.snapshot_path("update_test");
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("Updated"));
    }

    #[test]
    fn test_snapshot_debug() {
        let mut harness = TestHarness::new(10, 2);
        harness
            .render(|frame| {
                frame.render_widget(Paragraph::new("D"), frame.area());
            })
            .unwrap();

        let snapshot = harness.snapshot();
        let debug = format!("{:?}", snapshot);
        assert!(debug.contains("Snapshot"));
    }

    #[test]
    fn test_snapshot_clone() {
        let mut harness = TestHarness::new(10, 2);
        harness
            .render(|frame| {
                frame.render_widget(Paragraph::new("Clone"), frame.area());
            })
            .unwrap();

        let snapshot = harness.snapshot();
        let cloned = snapshot.clone();
        assert!(snapshot.matches(&cloned));
    }

    #[test]
    fn test_snapshot_test_debug() {
        let tester = SnapshotTest::new("/tmp");
        let debug = format!("{:?}", tester);
        assert!(debug.contains("SnapshotTest"));
    }

    #[test]
    fn test_snapshot_format_ansi_path() {
        let tester = SnapshotTest::new("/tmp").with_format(SnapshotFormat::Ansi);
        let path = tester.snapshot_path("test");
        assert!(path.to_string_lossy().ends_with(".ansi"));
    }

    #[test]
    fn test_snapshot_format_json_pretty_path() {
        let tester = SnapshotTest::new("/tmp").with_format(SnapshotFormat::JsonPretty);
        let path = tester.snapshot_path("test");
        // JsonPretty uses .json extension too
        assert!(path.to_string_lossy().ends_with(".json"));
    }

    #[test]
    fn test_snapshot_write_ansi() {
        use tempfile::TempDir;

        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("test.ansi");

        let mut harness = TestHarness::new(20, 2);
        harness
            .render(|frame| {
                frame.render_widget(Paragraph::new("ANSI"), frame.area());
            })
            .unwrap();

        let snapshot = harness.snapshot();
        snapshot
            .write_to_file(&path, SnapshotFormat::Ansi)
            .unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("ANSI"));
    }

    #[test]
    fn test_snapshot_write_plain() {
        use tempfile::TempDir;

        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("test.txt");

        let mut harness = TestHarness::new(20, 2);
        harness
            .render(|frame| {
                frame.render_widget(Paragraph::new("Plain"), frame.area());
            })
            .unwrap();

        let snapshot = harness.snapshot();
        snapshot
            .write_to_file(&path, SnapshotFormat::Plain)
            .unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("Plain"));
    }

    #[test]
    fn test_assert_snapshot_eq_matching() {
        let mut harness = TestHarness::new(20, 2);
        harness
            .render(|frame| {
                frame.render_widget(Paragraph::new("Same"), frame.area());
            })
            .unwrap();

        let snapshot1 = harness.snapshot();
        let snapshot2 = harness.snapshot();

        // Should not panic - snapshots are identical
        assert_snapshot_eq(&snapshot1, &snapshot2);
    }

    #[test]
    #[should_panic(expected = "Snapshots differ")]
    fn test_assert_snapshot_eq_different() {
        let mut harness1 = TestHarness::new(20, 2);
        harness1
            .render(|frame| {
                frame.render_widget(Paragraph::new("First"), frame.area());
            })
            .unwrap();

        let mut harness2 = TestHarness::new(20, 2);
        harness2
            .render(|frame| {
                frame.render_widget(Paragraph::new("Second"), frame.area());
            })
            .unwrap();

        // Should panic - snapshots differ
        assert_snapshot_eq(&harness1.snapshot(), &harness2.snapshot());
    }

    #[test]
    fn test_assert_snapshot_text_matching() {
        let mut harness = TestHarness::new(10, 1);
        harness
            .render(|frame| {
                frame.render_widget(Paragraph::new("Hello     "), frame.area());
            })
            .unwrap();

        let snapshot = harness.snapshot();
        let expected = snapshot.to_plain();

        // Should not panic - text matches
        assert_snapshot_text(&snapshot, &expected);
    }

    #[test]
    #[should_panic(expected = "Snapshot text differs")]
    fn test_assert_snapshot_text_different() {
        let mut harness = TestHarness::new(10, 1);
        harness
            .render(|frame| {
                frame.render_widget(Paragraph::new("Actual"), frame.area());
            })
            .unwrap();

        let snapshot = harness.snapshot();

        // Should panic - text doesn't match
        assert_snapshot_text(&snapshot, "Wrong text");
    }
}
