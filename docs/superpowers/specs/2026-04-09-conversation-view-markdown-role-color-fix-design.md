# ConversationView markdown role color fix — design

**Status:** approved
**Date:** 2026-04-09
**Source:** customer feedback (customer Claude reported "all white" text
when markdown is enabled in ConversationView)
**Scope:** single PR / bug fix

## Problem

`format_text_block` in `src/component/conversation_view/render.rs:304-344`
takes a `style: Style` parameter that carries the role color (e.g.,
`fg: Green` for User, `fg: Blue` for Assistant). This style is used
correctly in the **non-markdown branch** (lines 339-343), but
**completely discarded** in the **markdown branch** (lines 317-337).

In the markdown branch:

1. `render_markdown(text, width, &theme)` is called and produces styled
   `Line<'static>` values using the markdown theme's own styling — but with
   no knowledge of the message role. Plain-text spans come back with no
   foreground color set (they inherit the terminal default, which is white
   on a dark terminal).

2. The indent prefix is a bare `Span::raw(indent.to_string())`, which also
   has no foreground color — unlike the non-markdown branch where the indent
   implicitly inherits from the styled paragraph.

**Result:** With markdown enabled, every line of body text renders in the
terminal default foreground (white on a dark terminal) regardless of whose
message it is. Users cannot visually distinguish User messages from
Assistant messages.

**Impact:** Any consumer using `ConversationViewState::with_markdown(true)`
(the expected production configuration for AI chat UIs) sees undifferentiated
body text. The role color only appears on the header line (the role label),
not the message body.

## Goal

Make the markdown rendering path honor the role color for plain-text spans,
so that User and Assistant messages are visually distinguishable by color
when markdown is enabled — matching the behavior of the non-markdown path.

Non-goals:
- Role style override API (`HashMap<ConversationRole, Style>` on state).
  That's a separate follow-up PR.
- Changing the markdown renderer's API. The fix is local to
  `format_text_block`.
- Changing the `ConversationRole::color()` defaults in `types.rs`.

## Fix

### Patch role_style into markdown spans

After `render_markdown` returns its `Vec<Line<'static>>`, iterate over
each line's spans and apply `style.patch(span.style)`:

```rust
for span in md_line.spans.iter_mut() {
    span.style = style.patch(span.style);
}
```

**Patch direction:** `style.patch(span.style)` means "start with the role
style as the base, then overlay the span's existing fields on top". This
ensures:

- **Plain-text spans** (no fg set by markdown renderer): pick up the role
  color. User text is green, Assistant text is blue.
- **Bold spans** (markdown renderer sets `BOLD` but no fg): pick up the
  role color AND keep `BOLD`.
- **Inline-code spans** (markdown renderer sets `fg: Yellow, BOLD`): keep
  `fg: Yellow` because the span's explicit fg overrides the role's fg.
  The role color does NOT stomp on markdown-specific styling.
- **Italic, strikethrough, etc.:** same principle — span's set fields win.

### Style the indent prefix

Change `Span::raw(indent.to_string())` to
`Span::styled(indent.to_string(), style)` so the leading whitespace also
carries the role color. This matches the non-markdown branch where the
indent is part of the styled paragraph text.

### What does NOT change

- `render_markdown`'s API or implementation — unchanged. The fix is
  entirely in `format_text_block`.
- The non-markdown branch of `format_text_block` — already honors `style`.
- The `ConversationRole::color()` defaults in `types.rs` — unchanged.
- Code block / tool use / thinking / error block rendering — these use
  their own block-type styles (yellow for tool, magenta for thinking, red
  for error) and are unaffected. The bug is specifically in the text block
  markdown path.
- The `format_message` function — unchanged. `role_style` is already
  computed correctly there and passed to `format_block` → `format_text_block`.
  The problem was only that `format_text_block` threw it away.

## Visibility change

`build_display_lines` in `render.rs:144` is currently `fn` (private to the
`render` module). It needs to be promoted to `pub(super)` so the structural
test in `render_tests.rs` can call it. This is a visibility-only change
with no behavioral effect — `build_display_lines` is already called through
`pub(super)` entry points (`render_messages_from`, `total_display_lines`).

## Testing

A **structural render test** (not a snapshot) that inspects `Line`/`Span`
style values directly. This proves the `Style::patch` semantics work and
documents the expected behavior as a regression guard.

**Test:** `test_markdown_role_style_propagation` in
`src/component/conversation_view/render_tests.rs`:

1. Create a `ConversationViewState` with `with_markdown(true)`.
2. Push a User message with text: `plain text and **bold** and \`inline code\``
3. Push an Assistant message with the same text.
4. Call `render::build_display_lines(messages, &state, 80, &Theme::default())`
   to get `Vec<Line>`.
5. Partition the returned lines into User-message lines and
   Assistant-message lines (using the header span containing the role
   label as the delimiter).
6. For User message lines, find spans by content and assert:
   - A span containing "plain" → `fg == Some(Color::Green)`, NOT `None`.
   - A span containing "bold" with `BOLD` modifier → has `BOLD` AND
     `fg == Some(Color::Green)` (role color fills in the unset fg, BOLD
     from markdown survives).
   - A span containing "inline code" → retains `fg == Some(Color::Yellow)`
     (the markdown renderer's inline-code color, NOT the role's Green —
     proves the patch didn't stomp on explicit markdown styling).
7. For Assistant message lines, same assertions but
   `fg == Some(Color::Blue)`.
8. Assert User plain-text fg != Assistant plain-text fg (the original
   complaint: "all the same color").

**Span matching strategy:** Search by content substring, not by index.
The markdown renderer may split text into spans differently across
versions. Searching for substrings makes the test resilient to layout
changes.

The markdown feature is in the default feature set, so the test
exercises the markdown branch automatically during normal test runs.

## Risk and rollback

Risk is low. The change is ~10 lines in a single function (`format_text_block`),
affecting only the markdown branch. The non-markdown path is unchanged.
The patch semantics are well-defined (ratatui's `Style::patch` is
well-tested upstream).

Rollback: revert the single PR.
