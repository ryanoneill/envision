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
    /// One line of styled inline elements (renamed from `Paragraph` —
    /// the variant produces a single line, not a wrapped block).
    Line(Vec<StyledInline>),
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
///
/// `#[non_exhaustive]` so envision can add inline variants later without
/// breaking downstream `match` arms in consumer crates.
#[non_exhaustive]
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
    /// Styled run combining color, modifiers, and optional background.
    ///
    /// The composable form. Use [`StyledInline::styled`] or one of the
    /// leaf-helper constructors (`bold`, `italic`, `underlined`,
    /// `strikethrough`, `colored`) to construct.
    Styled {
        /// The text content.
        text: String,
        /// Style dimensions applied on top of the surrounding base style.
        style: InlineStyle,
    },
}

/// Style dimensions for a styled inline run.
///
/// All dimensions are optional and compose freely. Use [`InlineStyle::new`]
/// + builder methods (`fg`, `bg`, `bold`, `italic`, `underlined`,
/// `strikethrough`) to construct; struct-literal construction is
/// intentionally not supported (`#[non_exhaustive]`) so future modifier
/// additions land additively without breaking consumers.
///
/// All builder methods are `const fn` — `InlineStyle` chains can be used
/// in `const` contexts (e.g., module-level static styles).
///
/// # Example
///
/// ```rust
/// use envision::component::styled_text::InlineStyle;
/// use ratatui::style::Color;
///
/// let style = InlineStyle::new().fg(Color::Red).bold();
/// assert_eq!(style.fg, Some(Color::Red));
/// assert!(style.bold);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct InlineStyle {
    /// Foreground color override.
    pub fg: Option<Color>,
    /// Background color override.
    pub bg: Option<Color>,
    /// Render text in bold.
    pub bold: bool,
    /// Render text in italic.
    pub italic: bool,
    /// Render text underlined (past tense — matches `ratatui::style::Modifier::UNDERLINED`).
    pub underlined: bool,
    /// Render text with strikethrough.
    ///
    /// Note: ratatui's modifier name for this is `Modifier::CROSSED_OUT`,
    /// not `STRIKETHROUGH`. The render path maps `strikethrough: true` to
    /// `add_modifier(Modifier::CROSSED_OUT)`.
    pub strikethrough: bool,
}

impl InlineStyle {
    /// Creates an empty style (no modifiers, no colors).
    ///
    /// Equivalent to [`InlineStyle::default`].
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::styled_text::InlineStyle;
    ///
    /// let s = InlineStyle::new();
    /// assert_eq!(s, InlineStyle::default());
    /// ```
    pub const fn new() -> Self {
        Self {
            fg: None,
            bg: None,
            bold: false,
            italic: false,
            underlined: false,
            strikethrough: false,
        }
    }

    /// Builder: set foreground color.
    pub const fn fg(mut self, c: Color) -> Self {
        self.fg = Some(c);
        self
    }

    /// Builder: set background color.
    pub const fn bg(mut self, c: Color) -> Self {
        self.bg = Some(c);
        self
    }

    /// Builder: enable bold.
    pub const fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    /// Builder: enable italic.
    pub const fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    /// Builder: enable underlined (past tense — matches ratatui's `Modifier::UNDERLINED`).
    pub const fn underlined(mut self) -> Self {
        self.underlined = true;
        self
    }

    /// Builder: enable strikethrough (maps to `Modifier::CROSSED_OUT` in ratatui).
    pub const fn strikethrough(mut self) -> Self {
        self.strikethrough = true;
        self
    }
}

impl StyledInline {
    /// Wrap text with an explicit [`InlineStyle`]. The general-purpose
    /// constructor for any combination of dimensions.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::styled_text::{InlineStyle, StyledInline};
    /// use ratatui::style::Color;
    ///
    /// let inline = StyledInline::styled(
    ///     "840.16 ms",
    ///     InlineStyle::new().fg(Color::Red).bold(),
    /// );
    /// // Renders as red AND bold.
    /// # let _ = inline;
    /// ```
    pub fn styled(text: impl Into<String>, style: InlineStyle) -> Self {
        Self::Styled {
            text: text.into(),
            style,
        }
    }

    /// Single-dimension helper: bold text.
    ///
    /// Equivalent to `StyledInline::styled(text, InlineStyle::new().bold())`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::styled_text::StyledInline;
    /// let inline = StyledInline::bold("emphasis");
    /// # let _ = inline;
    /// ```
    pub fn bold(text: impl Into<String>) -> Self {
        Self::styled(text, InlineStyle::new().bold())
    }

    /// Single-dimension helper: italic text.
    ///
    /// Equivalent to `StyledInline::styled(text, InlineStyle::new().italic())`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::styled_text::StyledInline;
    /// let inline = StyledInline::italic("aside");
    /// # let _ = inline;
    /// ```
    pub fn italic(text: impl Into<String>) -> Self {
        Self::styled(text, InlineStyle::new().italic())
    }

    /// Single-dimension helper: underlined text.
    ///
    /// Equivalent to `StyledInline::styled(text, InlineStyle::new().underlined())`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::styled_text::StyledInline;
    /// let inline = StyledInline::underlined("link");
    /// # let _ = inline;
    /// ```
    pub fn underlined(text: impl Into<String>) -> Self {
        Self::styled(text, InlineStyle::new().underlined())
    }

    /// Single-dimension helper: strikethrough text.
    ///
    /// Equivalent to `StyledInline::styled(text, InlineStyle::new().strikethrough())`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::styled_text::StyledInline;
    /// let inline = StyledInline::strikethrough("deleted");
    /// # let _ = inline;
    /// ```
    pub fn strikethrough(text: impl Into<String>) -> Self {
        Self::styled(text, InlineStyle::new().strikethrough())
    }

    /// Single-dimension helper: text with foreground color.
    ///
    /// "Colored" idiomatically means foreground in TUI contexts (matches
    /// `Span::styled(text, Style::default().fg(...))` ergonomics). For
    /// bg-only or fg+bg cases, use [`StyledInline::styled`] with
    /// `InlineStyle::new().bg(...)` or `.fg(...).bg(...)`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::styled_text::StyledInline;
    /// use ratatui::style::Color;
    ///
    /// let inline = StyledInline::colored("warning", Color::Yellow);
    /// # let _ = inline;
    /// ```
    pub fn colored(text: impl Into<String>, fg: Color) -> Self {
        Self::styled(text, InlineStyle::new().fg(fg))
    }
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::styled_text::StyledContent;
    ///
    /// let content = StyledContent::new();
    /// assert!(content.is_empty());
    /// assert_eq!(content.len(), 0);
    /// ```
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
    ///     StyledBlock::Line(vec![StyledInline::Plain("Hello".to_string())]),
    /// ];
    /// let content = StyledContent::from_blocks(blocks);
    /// assert_eq!(content.len(), 2);
    /// ```
    pub fn from_blocks(blocks: Vec<StyledBlock>) -> Self {
        Self { blocks }
    }

    /// Adds a heading block.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::styled_text::StyledContent;
    ///
    /// let content = StyledContent::new()
    ///     .heading(1, "Main Title")
    ///     .heading(2, "Subtitle");
    /// assert_eq!(content.len(), 2);
    /// ```
    pub fn heading(mut self, level: u8, text: impl Into<String>) -> Self {
        self.blocks.push(StyledBlock::Heading {
            level: level.clamp(1, 3),
            text: text.into(),
        });
        self
    }

    /// Append a single styled line composed of inline elements.
    ///
    /// (Renamed from `paragraph(...)` — but the method produces one line,
    /// not a block-level paragraph. The `paragraph` name is reserved for
    /// future real block-level wrapped text.)
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::styled_text::{StyledContent, StyledInline};
    ///
    /// let content = StyledContent::new()
    ///     .line(vec![
    ///         StyledInline::Plain("Hello, ".to_string()),
    ///         StyledInline::bold("world".to_string()),
    ///     ]);
    /// assert_eq!(content.len(), 1);
    /// ```
    pub fn line(mut self, inlines: Vec<StyledInline>) -> Self {
        self.blocks.push(StyledBlock::Line(inlines));
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
        self.line(vec![StyledInline::Plain(text.into())])
    }

    /// Adds a bulleted list.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::styled_text::{StyledContent, StyledInline};
    ///
    /// let content = StyledContent::new()
    ///     .bullet_list(vec![
    ///         vec![StyledInline::Plain("First item".to_string())],
    ///         vec![StyledInline::Plain("Second item".to_string())],
    ///     ]);
    /// assert_eq!(content.len(), 1);
    /// ```
    pub fn bullet_list(mut self, items: Vec<Vec<StyledInline>>) -> Self {
        self.blocks.push(StyledBlock::BulletList(items));
        self
    }

    /// Adds a numbered list.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::styled_text::{StyledContent, StyledInline};
    ///
    /// let content = StyledContent::new()
    ///     .numbered_list(vec![
    ///         vec![StyledInline::Plain("Step one".to_string())],
    ///         vec![StyledInline::Plain("Step two".to_string())],
    ///     ]);
    /// assert_eq!(content.len(), 1);
    /// ```
    pub fn numbered_list(mut self, items: Vec<Vec<StyledInline>>) -> Self {
        self.blocks.push(StyledBlock::NumberedList(items));
        self
    }

    /// Adds a code block with optional language annotation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::styled_text::StyledContent;
    ///
    /// let content = StyledContent::new()
    ///     .code_block(Some("rust"), "let x = 42;");
    /// assert_eq!(content.len(), 1);
    /// ```
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::styled_text::StyledContent;
    ///
    /// let content = StyledContent::new()
    ///     .text("Above")
    ///     .horizontal_rule()
    ///     .text("Below");
    /// assert_eq!(content.len(), 3);
    /// ```
    pub fn horizontal_rule(mut self) -> Self {
        self.blocks.push(StyledBlock::HorizontalRule);
        self
    }

    /// Adds a blank line.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::styled_text::StyledContent;
    ///
    /// let content = StyledContent::new()
    ///     .text("Paragraph 1")
    ///     .blank_line()
    ///     .text("Paragraph 2");
    /// assert_eq!(content.len(), 3);
    /// ```
    pub fn blank_line(mut self) -> Self {
        self.blocks.push(StyledBlock::BlankLine);
        self
    }

    /// Adds raw pre-rendered lines.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::styled_text::StyledContent;
    /// use ratatui::text::Line;
    ///
    /// let content = StyledContent::new()
    ///     .raw(vec![Line::from("Custom rendered line")]);
    /// assert_eq!(content.len(), 1);
    /// ```
    pub fn raw(mut self, lines: Vec<RatLine<'static>>) -> Self {
        self.blocks.push(StyledBlock::Raw(lines));
        self
    }

    /// Pushes any block element.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::styled_text::{StyledContent, StyledBlock};
    ///
    /// let content = StyledContent::new()
    ///     .push(StyledBlock::HorizontalRule)
    ///     .push(StyledBlock::BlankLine);
    /// assert_eq!(content.len(), 2);
    /// ```
    pub fn push(mut self, block: StyledBlock) -> Self {
        self.blocks.push(block);
        self
    }

    /// Returns true if there are no blocks.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::styled_text::StyledContent;
    ///
    /// assert!(StyledContent::new().is_empty());
    /// assert!(!StyledContent::new().text("hello").is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }

    /// Returns the number of blocks.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::styled_text::StyledContent;
    ///
    /// let content = StyledContent::new()
    ///     .heading(1, "Title")
    ///     .text("Body");
    /// assert_eq!(content.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    /// Returns a reference to the blocks.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::styled_text::{StyledContent, StyledBlock};
    ///
    /// let content = StyledContent::new()
    ///     .heading(1, "Title")
    ///     .blank_line();
    /// assert_eq!(content.blocks().len(), 2);
    /// assert!(matches!(content.blocks()[1], StyledBlock::BlankLine));
    /// ```
    pub fn blocks(&self) -> &[StyledBlock] {
        &self.blocks
    }

    /// Renders this content into ratatui `Line` objects for display.
    pub(crate) fn render_lines(&self, width: u16, theme: &Theme) -> Vec<RatLine<'static>> {
        self.render_lines_styled(width, theme, theme.normal_style())
    }

    /// Renders this content using a caller-provided base style for inline text.
    ///
    /// `base_style` replaces `theme.normal_style()` for paragraphs, list item
    /// text, bold, italic, underline, strikethrough, and colored inlines.
    /// Headings, code blocks, horizontal rules, and inline code retain their
    /// semantic theme styles.
    pub(crate) fn render_lines_styled(
        &self,
        width: u16,
        theme: &Theme,
        base_style: Style,
    ) -> Vec<RatLine<'static>> {
        let mut lines = Vec::new();
        for block in &self.blocks {
            render_block(block, width, theme, base_style, &mut lines);
        }
        lines
    }
}

fn render_block(
    block: &StyledBlock,
    width: u16,
    theme: &Theme,
    base_style: Style,
    lines: &mut Vec<RatLine<'static>>,
) {
    match block {
        StyledBlock::Heading { level, text } => {
            let style = match level {
                1 => theme
                    .focused_style()
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                2 => theme.info_style().add_modifier(Modifier::BOLD),
                _ => base_style.add_modifier(Modifier::BOLD | Modifier::ITALIC),
            };
            lines.push(RatLine::from(RatSpan::styled(text.clone(), style)));
        }
        StyledBlock::Line(inlines) => {
            render_line(inlines, theme, base_style, lines);
        }
        StyledBlock::BulletList(items) => {
            for item in items {
                let mut spans = vec![RatSpan::styled("  • ", base_style)];
                for inline in item {
                    spans.push(render_inline(inline, theme, base_style));
                }
                lines.push(RatLine::from(spans));
            }
        }
        StyledBlock::NumberedList(items) => {
            for (i, item) in items.iter().enumerate() {
                let prefix = format!("  {}. ", i + 1);
                let mut spans = vec![RatSpan::styled(prefix, base_style)];
                for inline in item {
                    spans.push(render_inline(inline, theme, base_style));
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
                    base_style,
                )));
            }
            // Handle empty code blocks
            if content.is_empty() {
                lines.push(RatLine::from(RatSpan::styled(
                    "    ".to_string(),
                    base_style,
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

fn render_line(
    inlines: &[StyledInline],
    theme: &Theme,
    base_style: Style,
    lines: &mut Vec<RatLine<'static>>,
) {
    let spans: Vec<RatSpan<'static>> = inlines
        .iter()
        .map(|i| render_inline(i, theme, base_style))
        .collect();
    lines.push(RatLine::from(spans));
}

fn render_inline(inline: &StyledInline, theme: &Theme, base_style: Style) -> RatSpan<'static> {
    match inline {
        // Code and Colored use theme-specific styles; everything else uses base_style
        StyledInline::Code(text) => RatSpan::styled(
            text.clone(),
            theme.info_style().add_modifier(Modifier::BOLD),
        ),
        StyledInline::Colored { text, fg, bg } => {
            let mut style = base_style;
            if let Some(fg) = fg {
                style = style.fg(*fg);
            }
            if let Some(bg) = bg {
                style = style.bg(*bg);
            }
            RatSpan::styled(text.clone(), style)
        }
        other => render_inline_styled(other, base_style),
    }
}

/// Renders an inline element using a given base style (no theme needed).
fn render_inline_styled(inline: &StyledInline, base_style: Style) -> RatSpan<'static> {
    match inline {
        StyledInline::Plain(text) => RatSpan::styled(text.clone(), base_style),
        StyledInline::Bold(text) => {
            RatSpan::styled(text.clone(), base_style.add_modifier(Modifier::BOLD))
        }
        StyledInline::Italic(text) => {
            RatSpan::styled(text.clone(), base_style.add_modifier(Modifier::ITALIC))
        }
        StyledInline::Underline(text) => {
            RatSpan::styled(text.clone(), base_style.add_modifier(Modifier::UNDERLINED))
        }
        StyledInline::Strikethrough(text) => {
            RatSpan::styled(text.clone(), base_style.add_modifier(Modifier::CROSSED_OUT))
        }
        StyledInline::Colored { text, fg, bg } => {
            let mut style = base_style;
            if let Some(fg) = fg {
                style = style.fg(*fg);
            }
            if let Some(bg) = bg {
                style = style.bg(*bg);
            }
            RatSpan::styled(text.clone(), style)
        }
        StyledInline::Code(text) => {
            RatSpan::styled(text.clone(), base_style.add_modifier(Modifier::BOLD))
        }
        StyledInline::Styled { text, style } => {
            let mut s = base_style;
            if let Some(fg) = style.fg {
                s = s.fg(fg);
            }
            if let Some(bg) = style.bg {
                s = s.bg(bg);
            }
            if style.bold {
                s = s.add_modifier(Modifier::BOLD);
            }
            if style.italic {
                s = s.add_modifier(Modifier::ITALIC);
            }
            if style.underlined {
                s = s.add_modifier(Modifier::UNDERLINED);
            }
            if style.strikethrough {
                // ratatui names this modifier CROSSED_OUT, not STRIKETHROUGH.
                s = s.add_modifier(Modifier::CROSSED_OUT);
            }
            RatSpan::styled(text.clone(), s)
        }
    }
}

#[cfg(test)]
mod const_builder_test {
    use super::InlineStyle;
    use ratatui::style::Color;

    // Compile-time verification: all 7 builder methods are const fn.
    // If any method drops const, this const declaration fails to compile.
    const _STYLE: InlineStyle = InlineStyle::new()
        .fg(Color::Red)
        .bg(Color::Black)
        .bold()
        .italic()
        .underlined()
        .strikethrough();
}
