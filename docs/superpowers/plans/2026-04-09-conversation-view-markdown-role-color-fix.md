# ConversationView Markdown Role Color Fix — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix `format_text_block` so that markdown-rendered body text carries the role color (green for User, blue for Assistant), instead of rendering in the terminal default (white).

**Architecture:** Patch `style` (the role color) into each markdown-rendered span via `style.patch(span.style)`, preserving markdown-specific styling where set. Promote `build_display_lines` to `pub(super)` for testability. Add a structural test that inspects span styles directly.

**Tech Stack:** Rust (edition 2024), ratatui (terminal rendering), pulldown-cmark (markdown), cargo-nextest.

**Spec:** `docs/superpowers/specs/2026-04-09-conversation-view-markdown-role-color-fix-design.md`

---

## File Structure

- Modify: `src/component/conversation_view/render.rs` — fix `format_text_block` markdown branch, promote `build_display_lines` visibility
- Modify: `src/component/conversation_view/render_tests.rs` — add structural test
- Modify: `CHANGELOG.md` — add entry under `[Unreleased]`

---

## Task 1: Fix markdown branch + structural test (TDD)

**Files:**
- Modify: `src/component/conversation_view/render.rs` (lines 144, 317-337)
- Modify: `src/component/conversation_view/render_tests.rs` (add test at end)

---

- [ ] **Step 1.1: Promote `build_display_lines` to `pub(super)`**

In `src/component/conversation_view/render.rs`, change line 144 from:

```rust
fn build_display_lines<'a>(
```

to:

```rust
pub(super) fn build_display_lines<'a>(
```

This is a visibility-only change. No behavioral effect.

- [ ] **Step 1.2: Write the failing structural test**

Add to `src/component/conversation_view/render_tests.rs`, at the end of the file:

```rust
#[test]
fn test_markdown_role_style_propagation() {
    use ratatui::style::{Color, Modifier, Style};

    let mut state = ConversationViewState::new().with_markdown(true);
    state.push_user("plain text and **bold** and `inline code`");
    state.push_assistant("plain text and **bold** and `inline code`");

    let theme = crate::theme::Theme::default();
    let lines = super::render::build_display_lines(
        state.source_messages(),
        &state,
        80,
        &theme,
    );

    // Partition lines into user-section and assistant-section.
    // The header line for each message contains the role label.
    let mut user_lines: Vec<&Line> = Vec::new();
    let mut assistant_lines: Vec<&Line> = Vec::new();
    let mut current_section: Option<&str> = None;

    for line in &lines {
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        if text.contains("User") && line.spans.iter().any(|s| {
            s.style.add_modifier.contains(Modifier::BOLD)
        }) {
            current_section = Some("user");
            continue;
        }
        if text.contains("Assistant") && line.spans.iter().any(|s| {
            s.style.add_modifier.contains(Modifier::BOLD)
        }) {
            current_section = Some("assistant");
            continue;
        }
        match current_section {
            Some("user") => user_lines.push(line),
            Some("assistant") => assistant_lines.push(line),
            _ => {}
        }
    }

    assert!(!user_lines.is_empty(), "should have user message body lines");
    assert!(!assistant_lines.is_empty(), "should have assistant message body lines");

    // Helper: find a span containing `needle` across a set of lines.
    let find_span = |lines: &[&Line], needle: &str| -> Option<Style> {
        for line in lines {
            for span in &line.spans {
                if span.content.contains(needle) {
                    return Some(span.style);
                }
            }
        }
        None
    };

    // -- User assertions (role color: Green) --
    let user_plain = find_span(&user_lines, "plain")
        .expect("user section should contain a span with 'plain'");
    assert_eq!(
        user_plain.fg, Some(Color::Green),
        "user plain-text span should have fg=Green (role color), got {:?}",
        user_plain.fg,
    );

    let user_bold = find_span(&user_lines, "bold")
        .expect("user section should contain a span with 'bold'");
    assert!(
        user_bold.add_modifier.contains(Modifier::BOLD),
        "user bold span should retain BOLD modifier from markdown",
    );
    assert_eq!(
        user_bold.fg, Some(Color::Green),
        "user bold span should have fg=Green (role color fills in unset fg)",
    );

    let user_code = find_span(&user_lines, "inline code")
        .expect("user section should contain a span with 'inline code'");
    assert_ne!(
        user_code.fg, Some(Color::Green),
        "user inline-code span should NOT have role color — markdown's code styling wins",
    );
    assert_eq!(
        user_code.fg, Some(Color::Yellow),
        "user inline-code span should retain markdown's Yellow code color",
    );

    // -- Assistant assertions (role color: Blue) --
    let asst_plain = find_span(&assistant_lines, "plain")
        .expect("assistant section should contain a span with 'plain'");
    assert_eq!(
        asst_plain.fg, Some(Color::Blue),
        "assistant plain-text span should have fg=Blue (role color), got {:?}",
        asst_plain.fg,
    );

    let asst_bold = find_span(&assistant_lines, "bold")
        .expect("assistant section should contain a span with 'bold'");
    assert!(
        asst_bold.add_modifier.contains(Modifier::BOLD),
        "assistant bold span should retain BOLD modifier from markdown",
    );
    assert_eq!(
        asst_bold.fg, Some(Color::Blue),
        "assistant bold span should have fg=Blue (role color fills in unset fg)",
    );

    let asst_code = find_span(&assistant_lines, "inline code")
        .expect("assistant section should contain a span with 'inline code'");
    assert_ne!(
        asst_code.fg, Some(Color::Blue),
        "assistant inline-code span should NOT have role color — markdown's code styling wins",
    );

    // -- Cross-role differentiation (the original complaint) --
    assert_ne!(
        user_plain.fg, asst_plain.fg,
        "user and assistant plain-text spans must have DIFFERENT fg colors",
    );
}
```

- [ ] **Step 1.3: Run the test and verify it fails (RED)**

```bash
cargo nextest run -p envision conversation_view::render_tests::test_markdown_role_style_propagation
```

Expected: The test should compile but fail on the `user_plain.fg` assertion.
The assertion error should show `fg: None` or similar (the role color is
not reaching the markdown-rendered spans). This confirms the bug exists
and the test catches it.

If the test fails to **compile** (e.g., visibility error on
`super::render::build_display_lines`), verify that Step 1.1 was applied
correctly. If it compiles but the line partitioning fails (empty
`user_lines` or `assistant_lines`), the header-detection logic may need
adjustment — check what the actual header lines look like by printing
`lines` in a debug assertion.

- [ ] **Step 1.4: Apply the fix in `format_text_block`**

In `src/component/conversation_view/render.rs`, replace the markdown
branch of `format_text_block` (the `#[cfg(feature = "markdown")]` block
inside `fn format_text_block`, approximately lines 317-337). The current
code is:

```rust
    #[cfg(feature = "markdown")]
    if markdown_enabled {
        let theme = crate::theme::Theme::default();
        let indent_display_width = UnicodeWidthStr::width(indent);
        let available_width = width.saturating_sub(indent_display_width);
        let md_lines = crate::component::markdown_renderer::render::render_markdown(
            text,
            available_width as u16,
            &theme,
        );
        for md_line in md_lines {
            if indent.is_empty() {
                lines.push(md_line);
            } else {
                let mut spans: Vec<Span> = vec![Span::raw(indent.to_string())];
                spans.extend(md_line.spans);
                lines.push(Line::from(spans));
            }
        }
        return;
    }
```

Replace with:

```rust
    #[cfg(feature = "markdown")]
    if markdown_enabled {
        let theme = crate::theme::Theme::default();
        let indent_display_width = UnicodeWidthStr::width(indent);
        let available_width = width.saturating_sub(indent_display_width);
        let md_lines = crate::component::markdown_renderer::render::render_markdown(
            text,
            available_width as u16,
            &theme,
        );
        for mut md_line in md_lines {
            for span in md_line.spans.iter_mut() {
                span.style = style.patch(span.style);
            }
            if indent.is_empty() {
                lines.push(md_line);
            } else {
                let mut spans: Vec<Span> = vec![Span::styled(indent.to_string(), style)];
                spans.extend(md_line.spans);
                lines.push(Line::from(spans));
            }
        }
        return;
    }
```

Three changes:
1. `for md_line` → `for mut md_line` (need mutability to edit spans).
2. Added: the `for span in md_line.spans.iter_mut() { span.style = style.patch(span.style); }` loop.
3. `Span::raw(indent.to_string())` → `Span::styled(indent.to_string(), style)`.

- [ ] **Step 1.5: Run the test and verify it passes (GREEN)**

```bash
cargo nextest run -p envision conversation_view::render_tests::test_markdown_role_style_propagation
```

Expected: PASS. All assertions should succeed.

If the test fails, the most likely causes are:
- Wrong patch direction (`span.style.patch(style)` instead of `style.patch(span.style)`) — would stomp on markdown styling.
- The markdown renderer splits "plain text" across multiple spans — the `find_span` helper searching for "plain" may find a partial match. Adjust the needle if needed (e.g., search for "plain" alone).
- The bold span's content may be just "bold" without surrounding text, or may be wrapped differently — check what the markdown renderer actually produces.

Debug by printing the lines: `for line in &lines { eprintln!("{:?}", line); }`.

- [ ] **Step 1.6: Run the full conversation_view test suite**

```bash
cargo nextest run -p envision conversation_view
```

Expected: all tests pass, including existing render tests and snapshot tests.
The snapshot tests may or may not change — if they were generated with
`with_markdown(false)` (the default), they should be unaffected because the
non-markdown branch is unchanged. If any snapshot test used
`with_markdown(true)`, the snapshot will change (body text will now carry
role colors). In that case, inspect the new snapshot and accept it if it
now shows colored text where before it showed unstyled text — that's the
fix working.

- [ ] **Step 1.7: Format and lint**

```bash
cargo fmt
cargo clippy -p envision -- -D warnings
```

Expected: no formatting diffs; clippy reports zero warnings.

- [ ] **Step 1.8: Commit (signed)**

```bash
git add src/component/conversation_view/render.rs src/component/conversation_view/render_tests.rs
git commit -S -m "$(cat <<'EOF'
Fix ConversationView markdown rendering to honor role colors

format_text_block was discarding the role_style parameter in the
markdown branch, causing all body text to render in the terminal
default color (white on dark terminals) regardless of role.

Fix: after render_markdown returns, patch role_style into each
span via style.patch(span.style) so that plain-text spans inherit
the role color while markdown-specific styling (bold, inline code)
takes precedence where set. Also style the indent prefix with the
role color instead of using bare Span::raw.

Promotes build_display_lines to pub(super) for testability.

Adds test_markdown_role_style_propagation which structurally
inspects span styles to verify User spans carry Green, Assistant
spans carry Blue, bold spans retain BOLD, and inline-code spans
retain their Yellow code color.

Part of docs/superpowers/specs/2026-04-09-conversation-view-markdown-role-color-fix-design.md

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>
EOF
)"
```

If any snapshot files changed (accepted in Step 1.6), include them in the `git add`:
```bash
git add src/component/conversation_view/render.rs src/component/conversation_view/render_tests.rs src/component/conversation_view/snapshots/
```

---

## Task 2: CHANGELOG + full verification + PR

**Files:**
- Modify: `CHANGELOG.md`

---

- [ ] **Step 2.1: Add a changelog entry**

Open `CHANGELOG.md`. Under `## [Unreleased]`, add a `### Fixed` subsection
(if one doesn't already exist) with this bullet:

```markdown
### Fixed

- `ConversationView` now honors role colors (User=green, Assistant=blue,
  etc.) when markdown rendering is enabled. Previously, the role style was
  discarded in the markdown branch of `format_text_block`, causing all body
  text to render in the terminal default foreground regardless of role.
  Markdown-specific styling (bold, inline code) is preserved where set.
```

If there is already a `### Fixed` subsection under `[Unreleased]`, append
the bullet at the end of the existing list.

- [ ] **Step 2.2: Run the full project test suite**

```bash
cargo nextest run -p envision
cargo test --doc -p envision
```

Expected: all tests pass.

- [ ] **Step 2.3: Final format + clippy + build check**

```bash
cargo fmt
cargo clippy -p envision --all-targets -- -D warnings
cargo build -p envision
```

Expected: no formatting diffs, zero clippy warnings, clean build.

- [ ] **Step 2.4: Commit the changelog**

```bash
git add CHANGELOG.md
git commit -S -m "$(cat <<'EOF'
Document ConversationView markdown role color fix in CHANGELOG

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>
EOF
)"
```

- [ ] **Step 2.5: Merge origin/main and push**

```bash
git fetch origin
git merge origin/main --no-ff -S
```

If no new commits, this is a no-op. Then push:

```bash
git push -u origin conversation-view-markdown-role-style-fix
```

- [ ] **Step 2.6: Open the PR**

```bash
gh pr create --title "Fix ConversationView markdown rendering to honor role colors" --body "$(cat <<'EOF'
## Summary

- `format_text_block` was discarding the `role_style` parameter in the markdown branch, causing all body text to render in terminal default (white) regardless of whose message it is.
- Fix: after `render_markdown` returns, patch `role_style` into each span via `style.patch(span.style)` so plain-text inherits the role color while markdown-specific styling (bold, inline code) takes precedence.
- Also styles the indent prefix with `role_style` instead of bare `Span::raw`.
- Promotes `build_display_lines` to `pub(super)` for testability.

Design spec: `docs/superpowers/specs/2026-04-09-conversation-view-markdown-role-color-fix-design.md`

Addresses customer feedback: "all white" text when markdown is enabled in ConversationView.

## Test plan

- [x] `test_markdown_role_style_propagation` — structural test verifying User spans carry `fg: Green`, Assistant spans carry `fg: Blue`, bold spans retain `BOLD`, and inline-code spans retain `fg: Yellow`
- [x] `cargo nextest run -p envision conversation_view` — all tests pass
- [x] `cargo test --doc -p envision` — doc tests pass
- [x] `cargo clippy -p envision --all-targets -- -D warnings` — no warnings
- [x] `cargo fmt` — no diffs

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

- [ ] **Step 2.7: Check CI**

```bash
gh pr checks $(gh pr view --json number -q .number)
```

Do not merge until all required checks pass.

---

## Definition of done

- [ ] `format_text_block`'s markdown branch patches role_style into spans.
- [ ] Indent prefix is styled with role_style.
- [ ] `build_display_lines` is `pub(super)`.
- [ ] `test_markdown_role_style_propagation` exists and passes.
- [ ] All existing conversation_view tests still pass.
- [ ] `CHANGELOG.md` has an `[Unreleased] / Fixed` entry.
- [ ] PR opened, CI green, ready for review/merge.
- [ ] PR will be squash-merged per project conventions.
