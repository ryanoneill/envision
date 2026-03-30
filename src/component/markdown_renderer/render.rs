//! Markdown-to-[`Line`] rendering using pulldown-cmark.
//!
//! This module parses a markdown string and produces a `Vec<Line<'static>>`
//! suitable for rendering in a ratatui `Paragraph`.
//!
//! Supported markdown features:
//! - Headings (bold, with `#` prefix indicators)
//! - Bold, italic, strikethrough inline formatting
//! - Inline code (rendered with reversed style)
//! - Fenced and indented code blocks (indented with a left border character)
//! - Bullet lists (prefixed with `  - `)
//! - Numbered lists (prefixed with `  1. `)
//! - Links rendered as `text (url)`
//! - Horizontal rules (full-width `─` lines)
//! - Blockquotes (prefixed with `▎ `)

use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use crate::theme::Theme;

/// Parses a markdown string and produces styled [`Line`]s for rendering.
///
/// The `width` parameter is used for horizontal rules to fill the available
/// space. Other elements are rendered without width constraints (wrapping is
/// handled by the ratatui `Paragraph` widget).
pub fn render_markdown(source: &str, width: u16, theme: &Theme) -> Vec<Line<'static>> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(source, options);
    let mut renderer = MarkdownLineRenderer::new(width, theme);
    for event in parser {
        renderer.process(event);
    }
    renderer.finish()
}

// ---------------------------------------------------------------------------
// Internal renderer
// ---------------------------------------------------------------------------

/// Inline style markers pushed/popped as pulldown-cmark emits Start/End tags.
#[derive(Clone, Copy, PartialEq, Eq)]
enum InlineStyle {
    Strong,
    Emphasis,
    Strikethrough,
}

/// Tracks the nesting context for list items.
struct ListContext {
    ordered: bool,
    next_number: usize,
}

/// Accumulates rendered lines from a stream of pulldown-cmark events.
struct MarkdownLineRenderer<'t> {
    lines: Vec<Line<'static>>,
    /// Stack of active inline styles.
    style_stack: Vec<InlineStyle>,
    /// Spans being collected for the current line/paragraph.
    current_spans: Vec<Span<'static>>,
    /// Stack of active list contexts (supports nesting).
    list_stack: Vec<ListContext>,
    /// Active code block accumulation.
    code_block: Option<CodeBlockAccumulator>,
    /// Active heading level (between Start(Heading) and End(Heading)).
    heading_level: Option<u8>,
    /// Whether we are inside a blockquote.
    blockquote_depth: usize,
    /// The link URL when inside a Link tag.
    link_url: Option<String>,
    /// Available width for horizontal rules.
    width: u16,
    /// Theme for styling.
    theme: &'t Theme,
}

struct CodeBlockAccumulator {
    #[allow(dead_code)]
    language: Option<String>,
    content: String,
}

impl<'t> MarkdownLineRenderer<'t> {
    fn new(width: u16, theme: &'t Theme) -> Self {
        Self {
            lines: Vec::new(),
            style_stack: Vec::new(),
            current_spans: Vec::new(),
            list_stack: Vec::new(),
            code_block: None,
            heading_level: None,
            blockquote_depth: 0,
            link_url: None,
            width,
            theme,
        }
    }

    fn process(&mut self, event: Event) {
        match event {
            Event::Start(tag) => self.start_tag(tag),
            Event::End(tag) => self.end_tag(tag),
            Event::Text(text) => self.text(&text),
            Event::Code(code) => self.inline_code(&code),
            Event::SoftBreak => self.soft_break(),
            Event::HardBreak => self.hard_break(),
            Event::Rule => self.rule(),
            _ => {}
        }
    }

    // -- Tag handling --------------------------------------------------------

    fn start_tag(&mut self, tag: Tag) {
        match tag {
            Tag::Heading { level, .. } => {
                self.heading_level = Some(heading_level_to_u8(level));
            }
            Tag::Paragraph => {}
            Tag::List(start) => {
                self.list_stack.push(ListContext {
                    ordered: start.is_some(),
                    next_number: start.unwrap_or(0) as usize,
                });
            }
            Tag::Item => {
                self.current_spans.clear();
            }
            Tag::CodeBlock(kind) => {
                let language = match kind {
                    CodeBlockKind::Fenced(lang) => {
                        let lang = lang.trim();
                        if lang.is_empty() {
                            None
                        } else {
                            Some(lang.to_string())
                        }
                    }
                    CodeBlockKind::Indented => None,
                };
                self.code_block = Some(CodeBlockAccumulator {
                    language,
                    content: String::new(),
                });
            }
            Tag::BlockQuote(_) => {
                self.blockquote_depth += 1;
            }
            Tag::Strong => {
                self.style_stack.push(InlineStyle::Strong);
            }
            Tag::Emphasis => {
                self.style_stack.push(InlineStyle::Emphasis);
            }
            Tag::Strikethrough => {
                self.style_stack.push(InlineStyle::Strikethrough);
            }
            Tag::Link { dest_url, .. } => {
                self.link_url = Some(dest_url.to_string());
            }
            _ => {}
        }
    }

    fn end_tag(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Heading(_) => {
                let level = self.heading_level.take().unwrap_or(1);
                let prefix = "#".repeat(level as usize);
                let mut heading_spans =
                    vec![Span::styled(format!("{} ", prefix), self.heading_style())];
                heading_spans.extend(self.apply_heading_style_to_spans());
                self.current_spans.clear();
                self.lines.push(Line::from(heading_spans));
                // Blank line after heading
                self.lines.push(Line::from(""));
            }
            TagEnd::Paragraph => {
                if !self.list_stack.is_empty() {
                    // Inside a list item, inlines are collected at End(Item)
                    return;
                }
                if self.blockquote_depth > 0 {
                    self.flush_blockquote_paragraph();
                } else {
                    self.flush_paragraph();
                }
                // Blank line after paragraph
                self.lines.push(Line::from(""));
            }
            TagEnd::Item => {
                let prefix = if let Some(list) = self.list_stack.last_mut() {
                    if list.ordered {
                        let num = list.next_number;
                        list.next_number += 1;
                        format!("  {}. ", num)
                    } else {
                        "  - ".to_string()
                    }
                } else {
                    "  - ".to_string()
                };

                let prefix_style = self.normal_style();
                let mut item_spans = vec![Span::styled(prefix, prefix_style)];
                item_spans.append(&mut self.current_spans);
                self.lines.push(Line::from(item_spans));
            }
            TagEnd::List(_) => {
                self.list_stack.pop();
                // Blank line after list
                self.lines.push(Line::from(""));
            }
            TagEnd::CodeBlock => {
                if let Some(cb) = self.code_block.take() {
                    let code_style = self.code_block_style();
                    let border_style = Style::default().fg(self.theme.border);
                    for line_text in cb.content.trim_end_matches('\n').split('\n') {
                        let spans = vec![
                            Span::styled("│ ", border_style),
                            Span::styled(format!("  {}", line_text), code_style),
                        ];
                        self.lines.push(Line::from(spans));
                    }
                    // Blank line after code block
                    self.lines.push(Line::from(""));
                }
            }
            TagEnd::BlockQuote(_) => {
                self.blockquote_depth = self.blockquote_depth.saturating_sub(1);
                // Flush any remaining spans from the blockquote paragraph
                if self.blockquote_depth == 0 && !self.current_spans.is_empty() {
                    self.flush_blockquote_paragraph();
                    self.lines.push(Line::from(""));
                }
            }
            TagEnd::Strong => {
                self.pop_style(InlineStyle::Strong);
            }
            TagEnd::Emphasis => {
                self.pop_style(InlineStyle::Emphasis);
            }
            TagEnd::Strikethrough => {
                self.pop_style(InlineStyle::Strikethrough);
            }
            TagEnd::Link => {
                if let Some(url) = self.link_url.take() {
                    let link_style = Style::default().fg(self.theme.info);
                    self.current_spans
                        .push(Span::styled(format!(" ({})", url), link_style));
                }
            }
            _ => {}
        }
    }

    // -- Content handling ----------------------------------------------------

    fn text(&mut self, text: &str) {
        if let Some(ref mut cb) = self.code_block {
            cb.content.push_str(text);
            return;
        }

        if self.blockquote_depth > 0 && self.heading_level.is_none() {
            // Collect text inside blockquotes as spans; they get flushed
            // when the paragraph inside the blockquote ends.
            let style = Style::default()
                .fg(self.theme.disabled)
                .add_modifier(Modifier::ITALIC);
            self.current_spans
                .push(Span::styled(text.to_string(), style));
            return;
        }

        let style = self.current_inline_style();
        self.current_spans
            .push(Span::styled(text.to_string(), style));
    }

    fn inline_code(&mut self, code: &str) {
        if self.code_block.is_some() {
            return;
        }
        let style = Style::default()
            .fg(self.theme.warning)
            .add_modifier(Modifier::BOLD);
        self.current_spans
            .push(Span::styled(format!("`{}`", code), style));
    }

    fn soft_break(&mut self) {
        if self.code_block.is_none() {
            self.current_spans
                .push(Span::styled(" ".to_string(), self.normal_style()));
        }
    }

    fn hard_break(&mut self) {
        if self.code_block.is_none() && !self.current_spans.is_empty() {
            if self.blockquote_depth > 0 {
                self.flush_blockquote_paragraph();
            } else {
                self.flush_paragraph();
            }
        }
    }

    fn rule(&mut self) {
        let rule_char = '─';
        let rule_width = if self.width > 2 {
            (self.width - 2) as usize
        } else {
            1
        };
        let rule_text: String = std::iter::repeat(rule_char).take(rule_width).collect();
        let style = Style::default().fg(self.theme.border);
        self.lines.push(Line::from(Span::styled(rule_text, style)));
        self.lines.push(Line::from(""));
    }

    // -- Style helpers -------------------------------------------------------

    fn current_inline_style(&self) -> Style {
        let base = self.normal_style();
        let mut style = base;
        for s in &self.style_stack {
            match s {
                InlineStyle::Strong => {
                    style = style.add_modifier(Modifier::BOLD);
                }
                InlineStyle::Emphasis => {
                    style = style.add_modifier(Modifier::ITALIC);
                }
                InlineStyle::Strikethrough => {
                    style = style.add_modifier(Modifier::CROSSED_OUT);
                }
            }
        }
        style
    }

    fn heading_style(&self) -> Style {
        Style::default()
            .fg(self.theme.primary)
            .add_modifier(Modifier::BOLD)
    }

    fn code_block_style(&self) -> Style {
        Style::default().fg(self.theme.foreground)
    }

    fn normal_style(&self) -> Style {
        self.theme.normal_style()
    }

    fn pop_style(&mut self, expected: InlineStyle) {
        for i in (0..self.style_stack.len()).rev() {
            if self.style_stack[i] == expected {
                self.style_stack.remove(i);
                return;
            }
        }
    }

    fn apply_heading_style_to_spans(&mut self) -> Vec<Span<'static>> {
        let style = self.heading_style();
        self.current_spans
            .iter()
            .map(|span| Span::styled(span.content.to_string(), style))
            .collect()
    }

    // -- Flush helpers -------------------------------------------------------

    fn flush_paragraph(&mut self) {
        if !self.current_spans.is_empty() {
            let spans = std::mem::take(&mut self.current_spans);
            self.lines.push(Line::from(spans));
        }
    }

    fn flush_blockquote_paragraph(&mut self) {
        if !self.current_spans.is_empty() {
            let border_style = Style::default().fg(self.theme.border);
            let mut spans = vec![Span::styled("▎ ", border_style)];
            spans.append(&mut self.current_spans);
            self.lines.push(Line::from(spans));
        }
    }

    fn finish(mut self) -> Vec<Line<'static>> {
        // Flush any remaining spans
        if !self.current_spans.is_empty() {
            if self.blockquote_depth > 0 {
                self.flush_blockquote_paragraph();
            } else {
                self.flush_paragraph();
            }
        }
        // Remove trailing empty lines
        while self.lines.last().is_some_and(|l| l.width() == 0) {
            self.lines.pop();
        }
        self.lines
    }
}

fn heading_level_to_u8(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_theme() -> Theme {
        Theme::default()
    }

    fn render(source: &str) -> Vec<Line<'static>> {
        render_markdown(source, 40, &default_theme())
    }

    fn plain_text(line: &Line<'_>) -> String {
        line.spans.iter().map(|s| s.content.as_ref()).collect()
    }

    // -- Basic elements ------------------------------------------------------

    #[test]
    fn plain_paragraph() {
        let lines = render("Hello, world!");
        assert_eq!(lines.len(), 1);
        assert_eq!(plain_text(&lines[0]), "Hello, world!");
    }

    #[test]
    fn two_paragraphs() {
        let lines = render("First.\n\nSecond.");
        // "First.", blank, "Second."
        assert_eq!(lines.len(), 3);
        assert_eq!(plain_text(&lines[0]), "First.");
        assert_eq!(plain_text(&lines[2]), "Second.");
    }

    #[test]
    fn heading_level_1() {
        let lines = render("# Title");
        assert!(!lines.is_empty());
        let text = plain_text(&lines[0]);
        assert!(text.contains("# "), "expected heading prefix in: {}", text);
        assert!(text.contains("Title"), "expected title in: {}", text);
    }

    #[test]
    fn heading_level_2() {
        let lines = render("## Subtitle");
        let text = plain_text(&lines[0]);
        assert!(text.contains("## "));
        assert!(text.contains("Subtitle"));
    }

    #[test]
    fn heading_level_3() {
        let lines = render("### Section");
        let text = plain_text(&lines[0]);
        assert!(text.contains("### "));
        assert!(text.contains("Section"));
    }

    #[test]
    fn heading_is_bold() {
        let lines = render("# Bold Heading");
        let first_line = &lines[0];
        for span in &first_line.spans {
            assert!(
                span.style.add_modifier.contains(Modifier::BOLD),
                "heading spans should be bold"
            );
        }
    }

    // -- Inline formatting ---------------------------------------------------

    #[test]
    fn bold_text() {
        let lines = render("Some **bold** text.");
        assert_eq!(lines.len(), 1);
        let has_bold = lines[0]
            .spans
            .iter()
            .any(|s| s.style.add_modifier.contains(Modifier::BOLD));
        assert!(has_bold, "expected bold span");
    }

    #[test]
    fn italic_text() {
        let lines = render("Some *italic* text.");
        assert_eq!(lines.len(), 1);
        let has_italic = lines[0]
            .spans
            .iter()
            .any(|s| s.style.add_modifier.contains(Modifier::ITALIC));
        assert!(has_italic, "expected italic span");
    }

    #[test]
    fn strikethrough_text() {
        let lines = render("Some ~~deleted~~ text.");
        assert_eq!(lines.len(), 1);
        let has_strike = lines[0]
            .spans
            .iter()
            .any(|s| s.style.add_modifier.contains(Modifier::CROSSED_OUT));
        assert!(has_strike, "expected strikethrough span");
    }

    #[test]
    fn inline_code() {
        let lines = render("Use `foo()` here.");
        assert_eq!(lines.len(), 1);
        let code_span = lines[0].spans.iter().find(|s| s.content.contains("foo()"));
        assert!(code_span.is_some(), "expected inline code span");
        let cs = code_span.unwrap();
        assert!(
            cs.content.starts_with('`') && cs.content.ends_with('`'),
            "inline code should have backtick delimiters"
        );
    }

    // -- Code blocks ---------------------------------------------------------

    #[test]
    fn fenced_code_block() {
        let lines = render("```\nhello\nworld\n```");
        assert!(lines.len() >= 2, "expected at least 2 lines for code block");
        let text0 = plain_text(&lines[0]);
        assert!(
            text0.contains("hello"),
            "first code line should contain 'hello'"
        );
        assert!(
            text0.contains("│"),
            "code block lines should have left border"
        );
    }

    #[test]
    fn fenced_code_block_with_language() {
        let lines = render("```rust\nlet x = 42;\n```");
        assert!(!lines.is_empty());
        let text = plain_text(&lines[0]);
        assert!(text.contains("let x = 42;"));
    }

    #[test]
    fn code_block_preserves_indentation() {
        let lines = render("```\n  indented\n    more\n```");
        let texts: Vec<String> = lines.iter().map(|l| plain_text(l)).collect();
        assert!(
            texts.iter().any(|t| t.contains("  indented")),
            "should preserve indentation"
        );
        assert!(
            texts.iter().any(|t| t.contains("    more")),
            "should preserve deeper indentation"
        );
    }

    // -- Lists ---------------------------------------------------------------

    #[test]
    fn bullet_list() {
        let lines = render("- one\n- two\n- three");
        let texts: Vec<String> = lines.iter().map(|l| plain_text(l)).collect();
        assert!(texts.iter().any(|t| t.contains("- ") && t.contains("one")));
        assert!(texts.iter().any(|t| t.contains("- ") && t.contains("two")));
        assert!(texts
            .iter()
            .any(|t| t.contains("- ") && t.contains("three")));
    }

    #[test]
    fn numbered_list() {
        let lines = render("1. first\n2. second\n3. third");
        let texts: Vec<String> = lines.iter().map(|l| plain_text(l)).collect();
        assert!(
            texts
                .iter()
                .any(|t| t.contains("1.") && t.contains("first")),
            "texts: {:?}",
            texts
        );
        assert!(texts
            .iter()
            .any(|t| t.contains("2.") && t.contains("second")));
        assert!(texts
            .iter()
            .any(|t| t.contains("3.") && t.contains("third")));
    }

    #[test]
    fn bullet_list_with_bold() {
        let lines = render("- **bold item**\n- plain item");
        let texts: Vec<String> = lines.iter().map(|l| plain_text(l)).collect();
        assert!(texts.iter().any(|t| t.contains("bold item")));
        assert!(texts.iter().any(|t| t.contains("plain item")));
    }

    // -- Links ---------------------------------------------------------------

    #[test]
    fn link_rendered_with_url() {
        let lines = render("[click here](https://example.com)");
        let text = plain_text(&lines[0]);
        assert!(
            text.contains("click here"),
            "link text should be present: {}",
            text
        );
        assert!(
            text.contains("https://example.com"),
            "url should be shown: {}",
            text
        );
    }

    // -- Horizontal rules ----------------------------------------------------

    #[test]
    fn horizontal_rule() {
        let lines = render("above\n\n---\n\nbelow");
        let has_rule = lines.iter().any(|l| {
            let text = plain_text(l);
            text.chars().all(|c| c == '─') && !text.is_empty()
        });
        assert!(has_rule, "expected horizontal rule line");
    }

    // -- Blockquotes ---------------------------------------------------------

    #[test]
    fn blockquote() {
        let lines = render("> This is a quote");
        assert!(!lines.is_empty());
        let text = plain_text(&lines[0]);
        assert!(
            text.contains("▎"),
            "blockquote should have border prefix: {}",
            text
        );
        assert!(text.contains("This is a quote"));
    }

    // -- Empty input ---------------------------------------------------------

    #[test]
    fn empty_input() {
        let lines = render("");
        assert!(lines.is_empty());
    }

    #[test]
    fn whitespace_only() {
        let lines = render("   \n  \n  ");
        assert!(lines.is_empty());
    }

    // -- Complex documents ---------------------------------------------------

    #[test]
    fn complex_document() {
        let md = "\
# Title

Some **bold** text.

- item 1
- item 2

```rust
fn main() {}
```";
        let lines = render(md);
        // Should have heading, blank, paragraph, blank, 2 list items, blank, code line, blank
        assert!(
            lines.len() >= 6,
            "complex doc should produce at least 6 lines, got {}",
            lines.len()
        );
    }

    #[test]
    fn mixed_inline_styles() {
        let lines = render("Normal **bold** and *italic* and ~~strike~~");
        assert_eq!(lines.len(), 1);
        let has_bold = lines[0]
            .spans
            .iter()
            .any(|s| s.style.add_modifier.contains(Modifier::BOLD));
        let has_italic = lines[0]
            .spans
            .iter()
            .any(|s| s.style.add_modifier.contains(Modifier::ITALIC));
        let has_strike = lines[0]
            .spans
            .iter()
            .any(|s| s.style.add_modifier.contains(Modifier::CROSSED_OUT));
        assert!(has_bold);
        assert!(has_italic);
        assert!(has_strike);
    }

    #[test]
    fn multiple_headings() {
        let lines = render("# H1\n\n## H2\n\n### H3");
        let texts: Vec<String> = lines.iter().map(|l| plain_text(l)).collect();
        assert!(texts.iter().any(|t| t.contains("# ") && t.contains("H1")));
        assert!(texts.iter().any(|t| t.contains("## ") && t.contains("H2")));
        assert!(texts.iter().any(|t| t.contains("### ") && t.contains("H3")));
    }
}
