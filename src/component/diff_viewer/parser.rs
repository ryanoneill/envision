//! Unified diff parser and line-based diff computation.
//!
//! Provides two entry points:
//! - [`parse_unified_diff`] for parsing standard unified diff format text.
//! - [`compute_diff`] for computing a diff from two text strings using LCS.

use super::{DiffHunk, DiffLine, DiffLineType};

/// Parses a unified diff string into a sequence of hunks.
///
/// Handles standard unified diff format:
/// - `--- a/file` and `+++ b/file` header lines are skipped.
/// - `@@ -old_start,old_count +new_start,new_count @@` lines start a new hunk.
/// - Lines starting with `+` are additions.
/// - Lines starting with `-` are removals.
/// - Lines starting with ` ` (space) are context.
///
/// # Example
///
/// ```rust
/// use envision::component::diff_viewer::parser::parse_unified_diff;
///
/// let diff_text = "\
/// --- a/file.rs
/// +++ b/file.rs
/// @@ -1,3 +1,3 @@
///  fn main() {
/// -    println!(\"old\");
/// +    println!(\"new\");
///  }
/// ";
///
/// let hunks = parse_unified_diff(diff_text);
/// assert_eq!(hunks.len(), 1);
/// assert_eq!(hunks[0].lines.len(), 5); // header + context + removed + added + context
/// ```
pub fn parse_unified_diff(diff_text: &str) -> Vec<DiffHunk> {
    let mut hunks = Vec::new();
    let mut current_hunk: Option<DiffHunk> = None;
    let mut old_line: usize = 0;
    let mut new_line: usize = 0;

    for line in diff_text.lines() {
        if line.starts_with("--- ") || line.starts_with("+++ ") {
            // File headers - skip
            continue;
        }

        if line.starts_with("@@ ") {
            // Save previous hunk
            if let Some(hunk) = current_hunk.take() {
                hunks.push(hunk);
            }

            // Parse hunk header: @@ -old_start,old_count +new_start,new_count @@
            let (old_start, new_start) = parse_hunk_header(line);
            old_line = old_start;
            new_line = new_start;

            let mut hunk = DiffHunk {
                header: line.to_string(),
                old_start,
                new_start,
                lines: Vec::new(),
            };

            hunk.lines.push(DiffLine {
                line_type: DiffLineType::Header,
                content: line.to_string(),
                old_line_num: None,
                new_line_num: None,
            });

            current_hunk = Some(hunk);
            continue;
        }

        if let Some(ref mut hunk) = current_hunk {
            if let Some(content) = line.strip_prefix('+') {
                hunk.lines.push(DiffLine {
                    line_type: DiffLineType::Added,
                    content: content.to_string(),
                    old_line_num: None,
                    new_line_num: Some(new_line),
                });
                new_line += 1;
            } else if let Some(content) = line.strip_prefix('-') {
                hunk.lines.push(DiffLine {
                    line_type: DiffLineType::Removed,
                    content: content.to_string(),
                    old_line_num: Some(old_line),
                    new_line_num: None,
                });
                old_line += 1;
            } else if let Some(content) = line.strip_prefix(' ') {
                hunk.lines.push(DiffLine {
                    line_type: DiffLineType::Context,
                    content: content.to_string(),
                    old_line_num: Some(old_line),
                    new_line_num: Some(new_line),
                });
                old_line += 1;
                new_line += 1;
            } else if line.is_empty() {
                // Empty context line (no leading space)
                hunk.lines.push(DiffLine {
                    line_type: DiffLineType::Context,
                    content: String::new(),
                    old_line_num: Some(old_line),
                    new_line_num: Some(new_line),
                });
                old_line += 1;
                new_line += 1;
            }
        }
    }

    // Don't forget the last hunk
    if let Some(hunk) = current_hunk {
        hunks.push(hunk);
    }

    hunks
}

/// Parses a hunk header line like `@@ -10,5 +10,7 @@` and returns `(old_start, new_start)`.
fn parse_hunk_header(header: &str) -> (usize, usize) {
    // Format: @@ -old_start[,old_count] +new_start[,new_count] @@[ optional text]
    let mut old_start = 1;
    let mut new_start = 1;

    // Find the range between @@ markers
    if let Some(rest) = header.strip_prefix("@@ ") {
        let parts: Vec<&str> = rest.splitn(3, ' ').collect();
        if parts.len() >= 2 {
            // Parse old range: -start[,count]
            if let Some(old_range) = parts[0].strip_prefix('-') {
                if let Some(start_str) = old_range.split(',').next() {
                    old_start = start_str.parse().unwrap_or(1);
                }
            }
            // Parse new range: +start[,count]
            if let Some(new_range) = parts[1].strip_prefix('+') {
                if let Some(start_str) = new_range.split(',').next() {
                    new_start = start_str.parse().unwrap_or(1);
                }
            }
        }
    }

    (old_start, new_start)
}

/// Computes a line-based diff between two text strings using LCS.
///
/// Returns a sequence of hunks with configurable context lines around changes.
///
/// # Example
///
/// ```rust
/// use envision::component::diff_viewer::parser::compute_diff;
///
/// let old = "fn main() {\n    println!(\"old\");\n}\n";
/// let new = "fn main() {\n    println!(\"new\");\n}\n";
///
/// let hunks = compute_diff(old, new, 3);
/// assert_eq!(hunks.len(), 1);
/// ```
pub fn compute_diff(old_text: &str, new_text: &str, context_lines: usize) -> Vec<DiffHunk> {
    let old_lines: Vec<&str> = old_text.lines().collect();
    let new_lines: Vec<&str> = new_text.lines().collect();

    let lcs = compute_lcs(&old_lines, &new_lines);
    let edit_script = build_edit_script(&old_lines, &new_lines, &lcs);

    group_into_hunks(&edit_script, &old_lines, &new_lines, context_lines)
}

/// An edit operation in the diff.
#[derive(Debug, Clone, PartialEq, Eq)]
enum EditOp {
    /// Line is the same in both files.
    Equal(usize, usize), // (old_idx, new_idx)
    /// Line was removed from old file.
    Remove(usize), // old_idx
    /// Line was added in new file.
    Insert(usize), // new_idx
}

/// Computes the LCS table for two slices of lines.
fn compute_lcs<'a>(old: &[&'a str], new: &[&'a str]) -> Vec<Vec<usize>> {
    let m = old.len();
    let n = new.len();
    let mut table = vec![vec![0usize; n + 1]; m + 1];

    for i in 1..=m {
        for j in 1..=n {
            if old[i - 1] == new[j - 1] {
                table[i][j] = table[i - 1][j - 1] + 1;
            } else {
                table[i][j] = table[i - 1][j].max(table[i][j - 1]);
            }
        }
    }

    table
}

/// Builds an edit script from the LCS table by backtracking.
fn build_edit_script<'a>(old: &[&'a str], new: &[&'a str], lcs: &[Vec<usize>]) -> Vec<EditOp> {
    let mut ops = Vec::new();
    let mut i = old.len();
    let mut j = new.len();

    while i > 0 || j > 0 {
        if i > 0 && j > 0 && old[i - 1] == new[j - 1] {
            ops.push(EditOp::Equal(i - 1, j - 1));
            i -= 1;
            j -= 1;
        } else if j > 0 && (i == 0 || lcs[i][j - 1] >= lcs[i - 1][j]) {
            ops.push(EditOp::Insert(j - 1));
            j -= 1;
        } else {
            ops.push(EditOp::Remove(i - 1));
            i -= 1;
        }
    }

    ops.reverse();
    ops
}

/// Groups edit operations into hunks with the given number of context lines.
fn group_into_hunks(
    ops: &[EditOp],
    old_lines: &[&str],
    new_lines: &[&str],
    context_lines: usize,
) -> Vec<DiffHunk> {
    if ops.is_empty() {
        return Vec::new();
    }

    // Find change regions (indices in ops where changes occur)
    let change_indices: Vec<usize> = ops
        .iter()
        .enumerate()
        .filter(|(_, op)| !matches!(op, EditOp::Equal(_, _)))
        .map(|(i, _)| i)
        .collect();

    if change_indices.is_empty() {
        return Vec::new(); // No changes
    }

    // Group changes that are close together (within 2*context_lines of each other)
    let mut groups: Vec<(usize, usize)> = Vec::new(); // (first_change_idx, last_change_idx)
    let mut group_start = change_indices[0];
    let mut group_end = change_indices[0];

    for &idx in &change_indices[1..] {
        // Count equal ops between group_end and idx
        let gap = count_equal_ops_between(ops, group_end, idx);
        if gap <= 2 * context_lines {
            group_end = idx;
        } else {
            groups.push((group_start, group_end));
            group_start = idx;
            group_end = idx;
        }
    }
    groups.push((group_start, group_end));

    // Build hunks from groups
    let mut hunks = Vec::new();
    for (start, end) in groups {
        let hunk = build_hunk_from_group(ops, old_lines, new_lines, start, end, context_lines);
        hunks.push(hunk);
    }

    hunks
}

/// Counts the number of Equal operations between two indices (exclusive).
fn count_equal_ops_between(ops: &[EditOp], from: usize, to: usize) -> usize {
    ops[from + 1..to]
        .iter()
        .filter(|op| matches!(op, EditOp::Equal(_, _)))
        .count()
}

/// Builds a single hunk from a group of operations with context.
fn build_hunk_from_group(
    ops: &[EditOp],
    old_lines: &[&str],
    new_lines: &[&str],
    first_change: usize,
    last_change: usize,
    context_lines: usize,
) -> DiffHunk {
    // Expand range to include context
    let ctx_start = first_change.saturating_sub(context_lines);
    let ctx_end = (last_change + 1 + context_lines).min(ops.len());

    // Determine old_start and new_start from the first op in range
    let (old_start, new_start) = match &ops[ctx_start] {
        EditOp::Equal(o, n) => (*o + 1, *n + 1), // 1-based
        EditOp::Remove(o) => (*o + 1, find_new_line_at(ops, ctx_start) + 1),
        EditOp::Insert(n) => (find_old_line_at(ops, ctx_start) + 1, *n + 1),
    };

    let mut lines = Vec::new();
    let mut old_count = 0;
    let mut new_count = 0;

    for op in &ops[ctx_start..ctx_end] {
        match op {
            EditOp::Equal(o, n) => {
                lines.push(DiffLine {
                    line_type: DiffLineType::Context,
                    content: old_lines[*o].to_string(),
                    old_line_num: Some(*o + 1),
                    new_line_num: Some(*n + 1),
                });
                old_count += 1;
                new_count += 1;
            }
            EditOp::Remove(o) => {
                lines.push(DiffLine {
                    line_type: DiffLineType::Removed,
                    content: old_lines[*o].to_string(),
                    old_line_num: Some(*o + 1),
                    new_line_num: None,
                });
                old_count += 1;
            }
            EditOp::Insert(n) => {
                lines.push(DiffLine {
                    line_type: DiffLineType::Added,
                    content: new_lines[*n].to_string(),
                    old_line_num: None,
                    new_line_num: Some(*n + 1),
                });
                new_count += 1;
            }
        }
    }

    let header = format!(
        "@@ -{},{} +{},{} @@",
        old_start, old_count, new_start, new_count
    );

    // Prepend the header line
    let mut all_lines = vec![DiffLine {
        line_type: DiffLineType::Header,
        content: header.clone(),
        old_line_num: None,
        new_line_num: None,
    }];
    all_lines.extend(lines);

    DiffHunk {
        header,
        old_start,
        new_start,
        lines: all_lines,
    }
}

/// Finds the nearest old line number at or before the given op index.
fn find_old_line_at(ops: &[EditOp], idx: usize) -> usize {
    for op in ops[..=idx].iter().rev() {
        match op {
            EditOp::Equal(o, _) | EditOp::Remove(o) => return *o,
            _ => {}
        }
    }
    0
}

/// Finds the nearest new line number at or before the given op index.
fn find_new_line_at(ops: &[EditOp], idx: usize) -> usize {
    for op in ops[..=idx].iter().rev() {
        match op {
            EditOp::Equal(_, n) | EditOp::Insert(n) => return *n,
            _ => {}
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty() {
        let hunks = parse_unified_diff("");
        assert!(hunks.is_empty());
    }

    #[test]
    fn test_parse_simple_diff() {
        let diff = "\
--- a/file.rs
+++ b/file.rs
@@ -1,3 +1,3 @@
 fn main() {
-    println!(\"old\");
+    println!(\"new\");
 }
";
        let hunks = parse_unified_diff(diff);
        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].old_start, 1);
        assert_eq!(hunks[0].new_start, 1);
        // header + 2 context + 1 removed + 1 added = 5
        assert_eq!(hunks[0].lines.len(), 5);

        assert_eq!(hunks[0].lines[0].line_type, DiffLineType::Header);
        assert_eq!(hunks[0].lines[1].line_type, DiffLineType::Context);
        assert_eq!(hunks[0].lines[1].content, "fn main() {");
        assert_eq!(hunks[0].lines[2].line_type, DiffLineType::Removed);
        assert_eq!(hunks[0].lines[2].content, "    println!(\"old\");");
        assert_eq!(hunks[0].lines[3].line_type, DiffLineType::Added);
        assert_eq!(hunks[0].lines[3].content, "    println!(\"new\");");
        assert_eq!(hunks[0].lines[4].line_type, DiffLineType::Context);
        assert_eq!(hunks[0].lines[4].content, "}");
    }

    #[test]
    fn test_parse_multiple_hunks() {
        let diff = "\
--- a/file.rs
+++ b/file.rs
@@ -1,3 +1,3 @@
 first
-old1
+new1
 middle
@@ -10,3 +10,3 @@
 second
-old2
+new2
 end
";
        let hunks = parse_unified_diff(diff);
        assert_eq!(hunks.len(), 2);
        assert_eq!(hunks[0].old_start, 1);
        assert_eq!(hunks[1].old_start, 10);
    }

    #[test]
    fn test_parse_additions_only() {
        let diff = "\
--- a/file.rs
+++ b/file.rs
@@ -1,2 +1,4 @@
 line1
+added1
+added2
 line2
";
        let hunks = parse_unified_diff(diff);
        assert_eq!(hunks.len(), 1);

        let added: Vec<_> = hunks[0]
            .lines
            .iter()
            .filter(|l| l.line_type == DiffLineType::Added)
            .collect();
        assert_eq!(added.len(), 2);
    }

    #[test]
    fn test_parse_removals_only() {
        let diff = "\
--- a/file.rs
+++ b/file.rs
@@ -1,4 +1,2 @@
 line1
-removed1
-removed2
 line2
";
        let hunks = parse_unified_diff(diff);
        assert_eq!(hunks.len(), 1);

        let removed: Vec<_> = hunks[0]
            .lines
            .iter()
            .filter(|l| l.line_type == DiffLineType::Removed)
            .collect();
        assert_eq!(removed.len(), 2);
    }

    #[test]
    fn test_parse_line_numbers() {
        let diff = "\
--- a/file.rs
+++ b/file.rs
@@ -5,3 +5,3 @@
 context
-old
+new
 context
";
        let hunks = parse_unified_diff(diff);
        let lines = &hunks[0].lines;

        // First context line: old=5, new=5
        assert_eq!(lines[1].old_line_num, Some(5));
        assert_eq!(lines[1].new_line_num, Some(5));

        // Removed line: old=6, new=None
        assert_eq!(lines[2].old_line_num, Some(6));
        assert_eq!(lines[2].new_line_num, None);

        // Added line: old=None, new=6
        assert_eq!(lines[3].old_line_num, None);
        assert_eq!(lines[3].new_line_num, Some(6));

        // Last context: old=7, new=7
        assert_eq!(lines[4].old_line_num, Some(7));
        assert_eq!(lines[4].new_line_num, Some(7));
    }

    #[test]
    fn test_compute_diff_identical() {
        let text = "line1\nline2\nline3";
        let hunks = compute_diff(text, text, 3);
        assert!(hunks.is_empty());
    }

    #[test]
    fn test_compute_diff_single_change() {
        let old = "fn main() {\n    println!(\"old\");\n}";
        let new = "fn main() {\n    println!(\"new\");\n}";
        let hunks = compute_diff(old, new, 3);
        assert_eq!(hunks.len(), 1);

        let added: Vec<_> = hunks[0]
            .lines
            .iter()
            .filter(|l| l.line_type == DiffLineType::Added)
            .collect();
        let removed: Vec<_> = hunks[0]
            .lines
            .iter()
            .filter(|l| l.line_type == DiffLineType::Removed)
            .collect();
        assert_eq!(added.len(), 1);
        assert_eq!(removed.len(), 1);
        assert!(added[0].content.contains("new"));
        assert!(removed[0].content.contains("old"));
    }

    #[test]
    fn test_compute_diff_all_added() {
        let old = "";
        let new = "line1\nline2\nline3";
        let hunks = compute_diff(old, new, 3);
        assert_eq!(hunks.len(), 1);

        let added: Vec<_> = hunks[0]
            .lines
            .iter()
            .filter(|l| l.line_type == DiffLineType::Added)
            .collect();
        assert_eq!(added.len(), 3);
    }

    #[test]
    fn test_compute_diff_all_removed() {
        let old = "line1\nline2\nline3";
        let new = "";
        let hunks = compute_diff(old, new, 3);
        assert_eq!(hunks.len(), 1);

        let removed: Vec<_> = hunks[0]
            .lines
            .iter()
            .filter(|l| l.line_type == DiffLineType::Removed)
            .collect();
        assert_eq!(removed.len(), 3);
    }

    #[test]
    fn test_compute_diff_context_lines() {
        let old = "1\n2\n3\n4\n5\n6\n7\n8\n9\n10";
        let new = "1\n2\n3\n4\nFIVE\n6\n7\n8\n9\n10";
        let hunks = compute_diff(old, new, 2);
        assert_eq!(hunks.len(), 1);

        let ctx: Vec<_> = hunks[0]
            .lines
            .iter()
            .filter(|l| l.line_type == DiffLineType::Context)
            .collect();
        // 2 context before + 2 context after = 4
        assert_eq!(ctx.len(), 4);
    }

    #[test]
    fn test_hunk_header_parsing() {
        assert_eq!(parse_hunk_header("@@ -1,3 +1,3 @@"), (1, 1));
        assert_eq!(parse_hunk_header("@@ -10,5 +12,7 @@"), (10, 12));
        assert_eq!(parse_hunk_header("@@ -1,3 +1,3 @@ fn main()"), (1, 1));
    }

    #[test]
    fn test_parse_hunk_header_single_line() {
        // Single line hunks: @@ -1 +1 @@
        assert_eq!(parse_hunk_header("@@ -1 +1 @@"), (1, 1));
    }
}
