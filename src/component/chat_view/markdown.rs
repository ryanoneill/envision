//! Markdown parsing for chat message content.
//!
//! Converts markdown text to [`StyledContent`] using pulldown-cmark.
//! This module is only available when the `markdown` feature is enabled.

use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use crate::component::styled_text::{StyledBlock, StyledContent, StyledInline};

/// Parses markdown text into [`StyledContent`] for rich rendering.
///
/// Supports headings, paragraphs, bold, italic, strikethrough, inline code,
/// code blocks, bullet lists, numbered lists, and horizontal rules.
/// Links and images are rendered as plain text. HTML is ignored.
pub(super) fn parse_markdown(text: &str) -> StyledContent {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(text, options);
    let mut converter = MarkdownConverter::new();
    for event in parser {
        converter.process(event);
    }
    converter.finish()
}

/// Internal state machine for converting pulldown-cmark events to StyledContent.
struct MarkdownConverter {
    blocks: Vec<StyledBlock>,
    /// Stack of active inline styles.
    style_stack: Vec<InlineStyle>,
    /// Inlines being collected for the current block element.
    current_inlines: Vec<StyledInline>,
    /// Stack of active list contexts.
    list_stack: Vec<ListState>,
    /// Active code block state.
    code_block: Option<CodeBlockState>,
    /// Active heading level (when between Start(Heading) and End(Heading)).
    heading_level: Option<u8>,
}

#[derive(Clone, Copy, PartialEq)]
enum InlineStyle {
    Strong,
    Emphasis,
    Strikethrough,
}

struct ListState {
    ordered: bool,
    items: Vec<Vec<StyledInline>>,
}

struct CodeBlockState {
    language: Option<String>,
    content: String,
}

impl MarkdownConverter {
    fn new() -> Self {
        Self {
            blocks: Vec::new(),
            style_stack: Vec::new(),
            current_inlines: Vec::new(),
            list_stack: Vec::new(),
            code_block: None,
            heading_level: None,
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

    fn start_tag(&mut self, tag: Tag) {
        match tag {
            Tag::Heading { level, .. } => {
                self.heading_level = Some(heading_level_to_u8(level));
            }
            Tag::Paragraph => {}
            Tag::List(start) => {
                self.list_stack.push(ListState {
                    ordered: start.is_some(),
                    items: Vec::new(),
                });
            }
            Tag::Item => {
                self.current_inlines.clear();
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
                self.code_block = Some(CodeBlockState {
                    language,
                    content: String::new(),
                });
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
            Tag::BlockQuote(_) | Tag::Link { .. } | Tag::Image { .. } => {}
            _ => {}
        }
    }

    fn end_tag(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Heading(_) => {
                let level = self.heading_level.take().unwrap_or(1);
                let text = self.extract_plain_text();
                self.blocks.push(StyledBlock::Heading { level, text });
                self.current_inlines.clear();
            }
            TagEnd::Paragraph => {
                if !self.list_stack.is_empty() {
                    // Inside a list item — inlines collected at End(Item)
                    return;
                }
                if !self.current_inlines.is_empty() {
                    let inlines = std::mem::take(&mut self.current_inlines);
                    self.blocks.push(StyledBlock::Paragraph(inlines));
                }
            }
            TagEnd::Item => {
                let inlines = std::mem::take(&mut self.current_inlines);
                if let Some(list) = self.list_stack.last_mut() {
                    list.items.push(inlines);
                }
            }
            TagEnd::List(_) => {
                if let Some(list_state) = self.list_stack.pop() {
                    let block = if list_state.ordered {
                        StyledBlock::NumberedList(list_state.items)
                    } else {
                        StyledBlock::BulletList(list_state.items)
                    };
                    self.blocks.push(block);
                }
            }
            TagEnd::CodeBlock => {
                if let Some(cb) = self.code_block.take() {
                    let content = cb.content.trim_end_matches('\n').to_string();
                    self.blocks.push(StyledBlock::CodeBlock {
                        language: cb.language,
                        content,
                    });
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
            TagEnd::BlockQuote(_) | TagEnd::Link | TagEnd::Image => {}
            _ => {}
        }
    }

    fn text(&mut self, text: &str) {
        if let Some(ref mut cb) = self.code_block {
            cb.content.push_str(text);
            return;
        }
        let inline = self.wrap_with_style(text);
        self.current_inlines.push(inline);
    }

    fn inline_code(&mut self, code: &str) {
        self.current_inlines
            .push(StyledInline::Code(code.to_string()));
    }

    fn soft_break(&mut self) {
        if self.code_block.is_none() {
            self.current_inlines
                .push(StyledInline::Plain(" ".to_string()));
        }
    }

    fn hard_break(&mut self) {
        if self.code_block.is_none() && !self.current_inlines.is_empty() {
            let inlines = std::mem::take(&mut self.current_inlines);
            self.blocks.push(StyledBlock::Paragraph(inlines));
        }
    }

    fn rule(&mut self) {
        self.blocks.push(StyledBlock::HorizontalRule);
    }

    fn wrap_with_style(&self, text: &str) -> StyledInline {
        if let Some(style) = self.style_stack.last() {
            match style {
                InlineStyle::Strong => StyledInline::Bold(text.to_string()),
                InlineStyle::Emphasis => StyledInline::Italic(text.to_string()),
                InlineStyle::Strikethrough => StyledInline::Strikethrough(text.to_string()),
            }
        } else {
            StyledInline::Plain(text.to_string())
        }
    }

    fn pop_style(&mut self, expected: InlineStyle) {
        for i in (0..self.style_stack.len()).rev() {
            if self.style_stack[i] == expected {
                self.style_stack.remove(i);
                return;
            }
        }
    }

    /// Extracts plain text from current_inlines (used for headings).
    fn extract_plain_text(&self) -> String {
        let mut text = String::new();
        for inline in &self.current_inlines {
            match inline {
                StyledInline::Plain(t)
                | StyledInline::Bold(t)
                | StyledInline::Italic(t)
                | StyledInline::Underline(t)
                | StyledInline::Strikethrough(t)
                | StyledInline::Code(t) => text.push_str(t),
                StyledInline::Colored { text: t, .. } => text.push_str(t),
            }
        }
        text
    }

    fn finish(mut self) -> StyledContent {
        if !self.current_inlines.is_empty() {
            let inlines = std::mem::take(&mut self.current_inlines);
            self.blocks.push(StyledBlock::Paragraph(inlines));
        }
        StyledContent::from_blocks(self.blocks)
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

    #[test]
    fn plain_text_becomes_paragraph() {
        let content = parse_markdown("Hello, world!");
        let blocks = content.blocks();
        assert_eq!(blocks.len(), 1);
        assert_eq!(
            blocks[0],
            StyledBlock::Paragraph(vec![StyledInline::Plain("Hello, world!".to_string())])
        );
    }

    #[test]
    fn heading_level_1() {
        let content = parse_markdown("# Hello");
        let blocks = content.blocks();
        assert_eq!(blocks.len(), 1);
        assert_eq!(
            blocks[0],
            StyledBlock::Heading {
                level: 1,
                text: "Hello".to_string()
            }
        );
    }

    #[test]
    fn heading_levels_2_and_3() {
        let content = parse_markdown("## Second\n### Third");
        let blocks = content.blocks();
        assert_eq!(blocks.len(), 2);
        assert_eq!(
            blocks[0],
            StyledBlock::Heading {
                level: 2,
                text: "Second".to_string()
            }
        );
        assert_eq!(
            blocks[1],
            StyledBlock::Heading {
                level: 3,
                text: "Third".to_string()
            }
        );
    }

    #[test]
    fn bold_text() {
        let content = parse_markdown("**bold**");
        let blocks = content.blocks();
        assert_eq!(blocks.len(), 1);
        assert_eq!(
            blocks[0],
            StyledBlock::Paragraph(vec![StyledInline::Bold("bold".to_string())])
        );
    }

    #[test]
    fn italic_text() {
        let content = parse_markdown("*italic*");
        let blocks = content.blocks();
        assert_eq!(blocks.len(), 1);
        assert_eq!(
            blocks[0],
            StyledBlock::Paragraph(vec![StyledInline::Italic("italic".to_string())])
        );
    }

    #[test]
    fn mixed_inline_styles() {
        let content = parse_markdown("plain **bold** and *italic*");
        let blocks = content.blocks();
        assert_eq!(blocks.len(), 1);
        match &blocks[0] {
            StyledBlock::Paragraph(inlines) => {
                assert_eq!(inlines[0], StyledInline::Plain("plain ".to_string()));
                assert_eq!(inlines[1], StyledInline::Bold("bold".to_string()));
                assert_eq!(inlines[2], StyledInline::Plain(" and ".to_string()));
                assert_eq!(inlines[3], StyledInline::Italic("italic".to_string()));
            }
            other => panic!("Expected Paragraph, got {:?}", other),
        }
    }

    #[test]
    fn inline_code() {
        let content = parse_markdown("Use `foo()` here");
        let blocks = content.blocks();
        assert_eq!(blocks.len(), 1);
        match &blocks[0] {
            StyledBlock::Paragraph(inlines) => {
                assert_eq!(inlines[0], StyledInline::Plain("Use ".to_string()));
                assert_eq!(inlines[1], StyledInline::Code("foo()".to_string()));
                assert_eq!(inlines[2], StyledInline::Plain(" here".to_string()));
            }
            other => panic!("Expected Paragraph, got {:?}", other),
        }
    }

    #[test]
    fn code_block_with_language() {
        let content = parse_markdown("```rust\nlet x = 42;\n```");
        let blocks = content.blocks();
        assert_eq!(blocks.len(), 1);
        assert_eq!(
            blocks[0],
            StyledBlock::CodeBlock {
                language: Some("rust".to_string()),
                content: "let x = 42;".to_string(),
            }
        );
    }

    #[test]
    fn code_block_without_language() {
        let content = parse_markdown("```\nhello\n```");
        let blocks = content.blocks();
        assert_eq!(blocks.len(), 1);
        assert_eq!(
            blocks[0],
            StyledBlock::CodeBlock {
                language: None,
                content: "hello".to_string(),
            }
        );
    }

    #[test]
    fn bullet_list() {
        let content = parse_markdown("- item one\n- item two\n- item three");
        let blocks = content.blocks();
        assert_eq!(blocks.len(), 1);
        match &blocks[0] {
            StyledBlock::BulletList(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0], vec![StyledInline::Plain("item one".to_string())]);
                assert_eq!(items[1], vec![StyledInline::Plain("item two".to_string())]);
                assert_eq!(
                    items[2],
                    vec![StyledInline::Plain("item three".to_string())]
                );
            }
            other => panic!("Expected BulletList, got {:?}", other),
        }
    }

    #[test]
    fn numbered_list() {
        let content = parse_markdown("1. first\n2. second");
        let blocks = content.blocks();
        assert_eq!(blocks.len(), 1);
        match &blocks[0] {
            StyledBlock::NumberedList(items) => {
                assert_eq!(items.len(), 2);
                assert_eq!(items[0], vec![StyledInline::Plain("first".to_string())]);
                assert_eq!(items[1], vec![StyledInline::Plain("second".to_string())]);
            }
            other => panic!("Expected NumberedList, got {:?}", other),
        }
    }

    #[test]
    fn horizontal_rule() {
        let content = parse_markdown("above\n\n---\n\nbelow");
        let blocks = content.blocks();
        assert_eq!(blocks.len(), 3);
        assert_eq!(blocks[1], StyledBlock::HorizontalRule);
    }

    #[test]
    fn strikethrough_text() {
        let content = parse_markdown("~~deleted~~");
        let blocks = content.blocks();
        assert_eq!(blocks.len(), 1);
        assert_eq!(
            blocks[0],
            StyledBlock::Paragraph(vec![StyledInline::Strikethrough("deleted".to_string())])
        );
    }

    #[test]
    fn multiple_paragraphs() {
        let content = parse_markdown("First paragraph.\n\nSecond paragraph.");
        let blocks = content.blocks();
        assert_eq!(blocks.len(), 2);
        assert_eq!(
            blocks[0],
            StyledBlock::Paragraph(vec![StyledInline::Plain("First paragraph.".to_string())])
        );
        assert_eq!(
            blocks[1],
            StyledBlock::Paragraph(vec![StyledInline::Plain("Second paragraph.".to_string())])
        );
    }

    #[test]
    fn empty_input() {
        let content = parse_markdown("");
        assert!(content.is_empty());
    }

    #[test]
    fn bold_in_list_item() {
        let content = parse_markdown("- **bold item**\n- plain item");
        let blocks = content.blocks();
        assert_eq!(blocks.len(), 1);
        match &blocks[0] {
            StyledBlock::BulletList(items) => {
                assert_eq!(items.len(), 2);
                assert_eq!(items[0], vec![StyledInline::Bold("bold item".to_string())]);
                assert_eq!(
                    items[1],
                    vec![StyledInline::Plain("plain item".to_string())]
                );
            }
            other => panic!("Expected BulletList, got {:?}", other),
        }
    }

    #[test]
    fn code_block_preserves_whitespace() {
        let content = parse_markdown("```\n  indented\n    more\n```");
        let blocks = content.blocks();
        assert_eq!(blocks.len(), 1);
        assert_eq!(
            blocks[0],
            StyledBlock::CodeBlock {
                language: None,
                content: "  indented\n    more".to_string(),
            }
        );
    }

    #[test]
    fn heading_with_inline_code() {
        let content = parse_markdown("# The `parse` function");
        let blocks = content.blocks();
        assert_eq!(blocks.len(), 1);
        assert_eq!(
            blocks[0],
            StyledBlock::Heading {
                level: 1,
                text: "The parse function".to_string()
            }
        );
    }

    #[test]
    fn link_rendered_as_text() {
        let content = parse_markdown("[click here](https://example.com)");
        let blocks = content.blocks();
        assert_eq!(blocks.len(), 1);
        match &blocks[0] {
            StyledBlock::Paragraph(inlines) => {
                assert_eq!(inlines.len(), 1);
                assert_eq!(inlines[0], StyledInline::Plain("click here".to_string()));
            }
            other => panic!("Expected Paragraph, got {:?}", other),
        }
    }

    #[test]
    fn complex_document() {
        let md =
            "# Title\n\nSome **bold** text.\n\n- item 1\n- item 2\n\n```rust\nfn main() {}\n```";
        let content = parse_markdown(md);
        let blocks = content.blocks();
        assert_eq!(blocks.len(), 4);
        assert!(matches!(blocks[0], StyledBlock::Heading { level: 1, .. }));
        assert!(matches!(blocks[1], StyledBlock::Paragraph(_)));
        assert!(matches!(blocks[2], StyledBlock::BulletList(_)));
        assert!(matches!(blocks[3], StyledBlock::CodeBlock { .. }));
    }

    #[test]
    fn whitespace_only_is_empty() {
        let content = parse_markdown("   \n  \n  ");
        assert!(content.is_empty());
    }

    #[test]
    fn nested_list_with_formatting() {
        let content = parse_markdown("- *italic item*\n- `code item`");
        let blocks = content.blocks();
        assert_eq!(blocks.len(), 1);
        match &blocks[0] {
            StyledBlock::BulletList(items) => {
                assert_eq!(items.len(), 2);
                assert_eq!(
                    items[0],
                    vec![StyledInline::Italic("italic item".to_string())]
                );
                assert_eq!(items[1], vec![StyledInline::Code("code item".to_string())]);
            }
            other => panic!("Expected BulletList, got {:?}", other),
        }
    }

    #[test]
    fn multiline_code_block() {
        let content = parse_markdown("```python\ndef hello():\n    print(\"hi\")\n```");
        let blocks = content.blocks();
        assert_eq!(blocks.len(), 1);
        assert_eq!(
            blocks[0],
            StyledBlock::CodeBlock {
                language: Some("python".to_string()),
                content: "def hello():\n    print(\"hi\")".to_string(),
            }
        );
    }

    #[test]
    fn paragraph_then_list() {
        let content = parse_markdown("Here are items:\n\n- one\n- two");
        let blocks = content.blocks();
        assert_eq!(blocks.len(), 2);
        assert!(matches!(blocks[0], StyledBlock::Paragraph(_)));
        assert!(matches!(blocks[1], StyledBlock::BulletList(_)));
    }
}
