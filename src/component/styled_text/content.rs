//! Content types for rich text display.
//!
//! Provides [`StyledContent`], [`StyledBlock`], and [`StyledInline`] types
//! for building structured rich text with semantic blocks and inline styling.

use ratatui::prelude::*;
use ratatui::text::{Line as RatLine, Span as RatSpan};

use crate::theme::Theme;

/// A block-level element in styled text content.
#[derive(Clone, Debug, PartialEq)]
pub enum StyledBlock {
    /// A heading with a level (1-3).
    Heading {
        /// Heading level: 1 for top-level, 2 for secondary, 3 for tertiary.
        level: u8,
        /// The heading text.
        text: String,
    },
    /// A paragraph composed of inline elements.
    Paragraph(Vec<StyledInline>),
    /// A bulleted list where each item is a list of inline elements.
    BulletList(Vec<Vec<StyledInline>>),
    /// A numbered list where each item is a list of inline elements.
    NumberedList(Vec<Vec<StyledInline>>),
    /// A code block with optional language annotation.
    CodeBlock {
        /// Optional language for syntax highlighting hints.
        language: Option<String>,
        /// The code content.
        content: String,
    },
    /// A horizontal rule divider.
    HorizontalRule,
    /// A blank line.
    BlankLine,
    /// Raw pre-rendered lines (escape hatch for custom content).
    Raw(Vec<RatLine<'static>>),
}

/// An inline styling element within a paragraph or list item.
#[derive(Clone, Debug, PartialEq)]
pub enum StyledInline {
    /// Plain unstyled text.
    Plain(String),
    /// Bold text.
    Bold(String),
    /// Italic text.
    Italic(String),
    /// Underlined text.
    Underline(String),
    /// Strikethrough text.
    Strikethrough(String),
    /// Text with explicit foreground and/or background colors.
    Colored {
        /// The text content.
        text: String,
        /// Optional foreground color.
        fg: Option<Color>,
        /// Optional background color.
        bg: Option<Color>,
    },
    /// Inline code (displayed with distinct styling).
    Code(String),
}

/// A builder for constructing rich text content.
///
/// `StyledContent` holds a sequence of [`StyledBlock`] elements that are
/// rendered by the [`StyledText`](super::StyledText) component.
///
/// # Example
///
/// ```rust
/// use envision::component::styled_text::StyledContent;
///
/// let content = StyledContent::new()
///     .heading(1, "Welcome")
///     .text("This is a simple paragraph.")
///     .blank_line()
///     .code_block(None::<String>, "let x = 42;");
///
/// assert_eq!(content.len(), 4);
/// assert!(!content.is_empty());
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StyledContent {
    blocks: Vec<StyledBlock>,
}

impl StyledContent {
    /// Creates an empty styled content builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates styled content from a pre-built vector of blocks.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::styled_text::{StyledContent, StyledBlock, StyledInline};
    ///
    /// let blocks = vec![
    ///     StyledBlock::Heading { level: 1, text: "Title".to_string() },
    ///     StyledBlock::Paragraph(vec![StyledInline::Plain("Hello".to_string())]),
    /// ];
    /// let content = StyledContent::from_blocks(blocks);
    /// assert_eq!(content.len(), 2);
    /// ```
    pub fn from_blocks(blocks: Vec<StyledBlock>) -> Self {
        Self { blocks }
    }

    /// Adds a heading block.
    pub fn heading(mut self, level: u8, text: impl Into<String>) -> Self {
        self.blocks.push(StyledBlock::Heading {
            level: level.clamp(1, 3),
            text: text.into(),
        });
        self
    }

    /// Adds a paragraph composed of inline elements.
    pub fn paragraph(mut self, inlines: Vec<StyledInline>) -> Self {
        self.blocks.push(StyledBlock::Paragraph(inlines));
        self
    }

    /// Adds a paragraph with plain text (convenience method).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::styled_text::StyledContent;
    ///
    /// let content = StyledContent::new()
    ///     .text("Hello, world!");
    /// assert_eq!(content.len(), 1);
    /// ```
    pub fn text(self, text: impl Into<String>) -> Self {
        self.paragraph(vec![StyledInline::Plain(text.into())])
    }

    /// Adds a bulleted list.
    pub fn bullet_list(mut self, items: Vec<Vec<StyledInline>>) -> Self {
        self.blocks.push(StyledBlock::BulletList(items));
        self
    }

    /// Adds a numbered list.
    pub fn numbered_list(mut self, items: Vec<Vec<StyledInline>>) -> Self {
        self.blocks.push(StyledBlock::NumberedList(items));
        self
    }

    /// Adds a code block with optional language annotation.
    pub fn code_block(
        mut self,
        language: Option<impl Into<String>>,
        content: impl Into<String>,
    ) -> Self {
        self.blocks.push(StyledBlock::CodeBlock {
            language: language.map(|l| l.into()),
            content: content.into(),
        });
        self
    }

    /// Adds a horizontal rule.
    pub fn horizontal_rule(mut self) -> Self {
        self.blocks.push(StyledBlock::HorizontalRule);
        self
    }

    /// Adds a blank line.
    pub fn blank_line(mut self) -> Self {
        self.blocks.push(StyledBlock::BlankLine);
        self
    }

    /// Adds raw pre-rendered lines.
    pub fn raw(mut self, lines: Vec<RatLine<'static>>) -> Self {
        self.blocks.push(StyledBlock::Raw(lines));
        self
    }

    /// Pushes any block element.
    pub fn push(mut self, block: StyledBlock) -> Self {
        self.blocks.push(block);
        self
    }

    /// Returns true if there are no blocks.
    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }

    /// Returns the number of blocks.
    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    /// Returns a reference to the blocks.
    pub fn blocks(&self) -> &[StyledBlock] {
        &self.blocks
    }

    /// Renders this content into ratatui `Line` objects for display.
    pub(crate) fn render_lines(&self, width: u16, theme: &Theme) -> Vec<RatLine<'static>> {
        let mut lines = Vec::new();
        for block in &self.blocks {
            render_block(block, width, theme, &mut lines);
        }
        lines
    }
}

fn render_block(block: &StyledBlock, width: u16, theme: &Theme, lines: &mut Vec<RatLine<'static>>) {
    match block {
        StyledBlock::Heading { level, text } => {
            let style = match level {
                1 => theme
                    .focused_style()
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                2 => theme.info_style().add_modifier(Modifier::BOLD),
                _ => theme
                    .normal_style()
                    .add_modifier(Modifier::BOLD | Modifier::ITALIC),
            };
            lines.push(RatLine::from(RatSpan::styled(text.clone(), style)));
        }
        StyledBlock::Paragraph(inlines) => {
            render_paragraph(inlines, width, theme, lines);
        }
        StyledBlock::BulletList(items) => {
            for item in items {
                let mut spans = vec![RatSpan::styled("  * ", theme.normal_style())];
                for inline in item {
                    spans.push(render_inline(inline, theme));
                }
                lines.push(RatLine::from(spans));
            }
        }
        StyledBlock::NumberedList(items) => {
            for (i, item) in items.iter().enumerate() {
                let prefix = format!("  {}. ", i + 1);
                let mut spans = vec![RatSpan::styled(prefix, theme.normal_style())];
                for inline in item {
                    spans.push(render_inline(inline, theme));
                }
                lines.push(RatLine::from(spans));
            }
        }
        StyledBlock::CodeBlock { language, content } => {
            if let Some(lang) = language {
                lines.push(RatLine::from(RatSpan::styled(
                    format!("  [{}]", lang),
                    theme.disabled_style().add_modifier(Modifier::ITALIC),
                )));
            }
            for line in content.lines() {
                lines.push(RatLine::from(RatSpan::styled(
                    format!("    {}", line),
                    theme.normal_style(),
                )));
            }
            // Handle empty code blocks
            if content.is_empty() {
                lines.push(RatLine::from(RatSpan::styled(
                    "    ".to_string(),
                    theme.normal_style(),
                )));
            }
        }
        StyledBlock::HorizontalRule => {
            let rule = "─".repeat(width as usize);
            lines.push(RatLine::from(RatSpan::styled(rule, theme.border_style())));
        }
        StyledBlock::BlankLine => {
            lines.push(RatLine::from(""));
        }
        StyledBlock::Raw(raw_lines) => {
            lines.extend(raw_lines.iter().cloned());
        }
    }
}

fn render_paragraph(
    inlines: &[StyledInline],
    _width: u16,
    theme: &Theme,
    lines: &mut Vec<RatLine<'static>>,
) {
    let spans: Vec<RatSpan<'static>> = inlines.iter().map(|i| render_inline(i, theme)).collect();
    lines.push(RatLine::from(spans));
}

fn render_inline(inline: &StyledInline, theme: &Theme) -> RatSpan<'static> {
    match inline {
        StyledInline::Plain(text) => RatSpan::styled(text.clone(), theme.normal_style()),
        StyledInline::Bold(text) => RatSpan::styled(
            text.clone(),
            theme.normal_style().add_modifier(Modifier::BOLD),
        ),
        StyledInline::Italic(text) => RatSpan::styled(
            text.clone(),
            theme.normal_style().add_modifier(Modifier::ITALIC),
        ),
        StyledInline::Underline(text) => RatSpan::styled(
            text.clone(),
            theme.normal_style().add_modifier(Modifier::UNDERLINED),
        ),
        StyledInline::Strikethrough(text) => RatSpan::styled(
            text.clone(),
            theme.normal_style().add_modifier(Modifier::CROSSED_OUT),
        ),
        StyledInline::Colored { text, fg, bg } => {
            let mut style = theme.normal_style();
            if let Some(fg) = fg {
                style = style.fg(*fg);
            }
            if let Some(bg) = bg {
                style = style.bg(*bg);
            }
            RatSpan::styled(text.clone(), style)
        }
        StyledInline::Code(text) => RatSpan::styled(
            text.clone(),
            theme.info_style().add_modifier(Modifier::BOLD),
        ),
    }
}
