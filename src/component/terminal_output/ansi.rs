//! ANSI escape sequence parser for terminal output rendering.
//!
//! Supports SGR (Select Graphic Rendition) codes including reset, bold,
//! dim, italic, underline, standard/bright foreground and background
//! colors, and 256-color palette.

use ratatui::style::{Color, Modifier, Style};

/// A segment of text with associated ANSI styling.
///
/// Produced by [`parse_ansi`] when splitting a line containing ANSI
/// escape sequences into styled fragments.
///
/// # Example
///
/// ```rust
/// # #[cfg(feature = "display-components")]
/// # {
/// use envision::component::terminal_output::AnsiSegment;
/// use ratatui::style::{Color, Modifier, Style};
///
/// let seg = AnsiSegment {
///     text: "hello".to_string(),
///     style: Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
/// };
/// assert_eq!(seg.text, "hello");
/// # }
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct AnsiSegment {
    /// The text content of this segment.
    pub text: String,
    /// The style derived from ANSI escape codes.
    pub style: Style,
}

/// Parses a string containing ANSI escape sequences into styled segments.
///
/// Supports the following SGR codes:
/// - `0` — reset all attributes
/// - `1` — bold
/// - `2` — dim/faint
/// - `3` — italic
/// - `4` — underline
/// - `22` — normal intensity (cancel bold/dim)
/// - `23` — not italic
/// - `24` — not underlined
/// - `30`–`37` — standard foreground colors
/// - `38;5;n` — 256-color foreground
/// - `39` — default foreground
/// - `40`–`47` — standard background colors
/// - `48;5;n` — 256-color background
/// - `49` — default background
/// - `90`–`97` — bright foreground colors
/// - `100`–`107` — bright background colors
///
/// Non-SGR escape sequences are silently skipped.
///
/// # Example
///
/// ```rust
/// # #[cfg(feature = "display-components")]
/// # {
/// use envision::component::terminal_output::parse_ansi;
/// use ratatui::style::{Color, Style};
///
/// let segments = parse_ansi("hello \x1b[31mworld\x1b[0m");
/// assert_eq!(segments.len(), 2);
/// assert_eq!(segments[0].text, "hello ");
/// assert_eq!(segments[1].text, "world");
/// assert_eq!(segments[1].style, Style::default().fg(Color::Red));
/// # }
/// ```
pub fn parse_ansi(input: &str) -> Vec<AnsiSegment> {
    let mut segments = Vec::new();
    let mut current_style = Style::default();
    let mut current_text = String::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            // Start of escape sequence
            if chars.peek() == Some(&'[') {
                chars.next(); // consume '['

                // Collect the parameter bytes
                let mut params = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_digit() || c == ';' {
                        params.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }

                // Consume the final byte
                let final_byte = chars.next();

                if final_byte == Some('m') {
                    // SGR sequence — flush current text
                    if !current_text.is_empty() {
                        segments.push(AnsiSegment {
                            text: std::mem::take(&mut current_text),
                            style: current_style,
                        });
                    }
                    current_style = apply_sgr(&params, current_style);
                }
                // Non-SGR sequences are silently ignored
            }
            // Bare ESC without '[' is ignored
        } else {
            current_text.push(ch);
        }
    }

    // Flush remaining text
    if !current_text.is_empty() {
        segments.push(AnsiSegment {
            text: current_text,
            style: current_style,
        });
    }

    segments
}

/// Applies SGR parameter codes to an existing style.
fn apply_sgr(params: &str, mut style: Style) -> Style {
    if params.is_empty() {
        // ESC[m is equivalent to ESC[0m (reset)
        return Style::default();
    }

    let codes: Vec<&str> = params.split(';').collect();
    let mut i = 0;

    while i < codes.len() {
        let code: u16 = match codes[i].parse() {
            Ok(n) => n,
            Err(_) => {
                i += 1;
                continue;
            }
        };

        match code {
            0 => style = Style::default(),
            1 => style = style.add_modifier(Modifier::BOLD),
            2 => style = style.add_modifier(Modifier::DIM),
            3 => style = style.add_modifier(Modifier::ITALIC),
            4 => style = style.add_modifier(Modifier::UNDERLINED),
            22 => {
                style = style
                    .remove_modifier(Modifier::BOLD)
                    .remove_modifier(Modifier::DIM);
            }
            23 => style = style.remove_modifier(Modifier::ITALIC),
            24 => style = style.remove_modifier(Modifier::UNDERLINED),

            // Standard foreground colors
            30 => style = style.fg(Color::Black),
            31 => style = style.fg(Color::Red),
            32 => style = style.fg(Color::Green),
            33 => style = style.fg(Color::Yellow),
            34 => style = style.fg(Color::Blue),
            35 => style = style.fg(Color::Magenta),
            36 => style = style.fg(Color::Cyan),
            37 => style = style.fg(Color::White),

            // 256-color foreground: 38;5;n
            38 if i + 2 < codes.len() && codes[i + 1] == "5" => {
                if let Ok(n) = codes[i + 2].parse::<u8>() {
                    style = style.fg(Color::Indexed(n));
                    i += 2;
                }
            }
            // 24-bit RGB foreground: 38;2;r;g;b
            38 if i + 4 < codes.len() && codes[i + 1] == "2" => {
                if let (Ok(r), Ok(g), Ok(b)) = (
                    codes[i + 2].parse::<u8>(),
                    codes[i + 3].parse::<u8>(),
                    codes[i + 4].parse::<u8>(),
                ) {
                    style = style.fg(Color::Rgb(r, g, b));
                    i += 4;
                }
            }
            38 => {}

            // Default foreground
            39 => style = style.fg(Color::Reset),

            // Standard background colors
            40 => style = style.bg(Color::Black),
            41 => style = style.bg(Color::Red),
            42 => style = style.bg(Color::Green),
            43 => style = style.bg(Color::Yellow),
            44 => style = style.bg(Color::Blue),
            45 => style = style.bg(Color::Magenta),
            46 => style = style.bg(Color::Cyan),
            47 => style = style.bg(Color::White),

            // 256-color background: 48;5;n
            48 if i + 2 < codes.len() && codes[i + 1] == "5" => {
                if let Ok(n) = codes[i + 2].parse::<u8>() {
                    style = style.bg(Color::Indexed(n));
                    i += 2;
                }
            }
            // 24-bit RGB background: 48;2;r;g;b
            48 if i + 4 < codes.len() && codes[i + 1] == "2" => {
                if let (Ok(r), Ok(g), Ok(b)) = (
                    codes[i + 2].parse::<u8>(),
                    codes[i + 3].parse::<u8>(),
                    codes[i + 4].parse::<u8>(),
                ) {
                    style = style.bg(Color::Rgb(r, g, b));
                    i += 4;
                }
            }
            48 => {}

            // Default background
            49 => style = style.bg(Color::Reset),

            // Bright foreground colors
            90 => style = style.fg(Color::DarkGray),
            91 => style = style.fg(Color::LightRed),
            92 => style = style.fg(Color::LightGreen),
            93 => style = style.fg(Color::LightYellow),
            94 => style = style.fg(Color::LightBlue),
            95 => style = style.fg(Color::LightMagenta),
            96 => style = style.fg(Color::LightCyan),
            97 => style = style.fg(Color::Gray),

            // Bright background colors
            100 => style = style.bg(Color::DarkGray),
            101 => style = style.bg(Color::LightRed),
            102 => style = style.bg(Color::LightGreen),
            103 => style = style.bg(Color::LightYellow),
            104 => style = style.bg(Color::LightBlue),
            105 => style = style.bg(Color::LightMagenta),
            106 => style = style.bg(Color::LightCyan),
            107 => style = style.bg(Color::Gray),

            _ => {} // Unknown code — ignore
        }

        i += 1;
    }

    style
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::{Color, Modifier, Style};

    // =========================================================================
    // Plain text (no escapes)
    // =========================================================================

    #[test]
    fn test_plain_text() {
        let segments = parse_ansi("hello world");
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].text, "hello world");
        assert_eq!(segments[0].style, Style::default());
    }

    #[test]
    fn test_empty_string() {
        let segments = parse_ansi("");
        assert!(segments.is_empty());
    }

    // =========================================================================
    // Reset
    // =========================================================================

    #[test]
    fn test_reset_code_0() {
        let segments = parse_ansi("\x1b[31mred\x1b[0mplain");
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].style, Style::default().fg(Color::Red));
        assert_eq!(segments[1].style, Style::default());
        assert_eq!(segments[1].text, "plain");
    }

    #[test]
    fn test_bare_reset() {
        // ESC[m is equivalent to ESC[0m
        let segments = parse_ansi("\x1b[1mbold\x1b[mnormal");
        assert_eq!(segments.len(), 2);
        assert_eq!(
            segments[0].style,
            Style::default().add_modifier(Modifier::BOLD)
        );
        assert_eq!(segments[1].style, Style::default());
    }

    // =========================================================================
    // Text modifiers
    // =========================================================================

    #[test]
    fn test_bold() {
        let segments = parse_ansi("\x1b[1mbold text");
        assert_eq!(segments.len(), 1);
        assert_eq!(
            segments[0].style,
            Style::default().add_modifier(Modifier::BOLD)
        );
    }

    #[test]
    fn test_dim() {
        let segments = parse_ansi("\x1b[2mdim text");
        assert_eq!(segments.len(), 1);
        assert_eq!(
            segments[0].style,
            Style::default().add_modifier(Modifier::DIM)
        );
    }

    #[test]
    fn test_italic() {
        let segments = parse_ansi("\x1b[3mitalic text");
        assert_eq!(segments.len(), 1);
        assert_eq!(
            segments[0].style,
            Style::default().add_modifier(Modifier::ITALIC)
        );
    }

    #[test]
    fn test_underline() {
        let segments = parse_ansi("\x1b[4munderlined");
        assert_eq!(segments.len(), 1);
        assert_eq!(
            segments[0].style,
            Style::default().add_modifier(Modifier::UNDERLINED)
        );
    }

    #[test]
    fn test_cancel_bold_dim() {
        let segments = parse_ansi("\x1b[1;2mboldim\x1b[22mnormal");
        assert_eq!(segments.len(), 2);
        assert_eq!(
            segments[0].style,
            Style::default()
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::DIM)
        );
        // After code 22, bold and dim are removed
        let expected = Style::default()
            .add_modifier(Modifier::BOLD)
            .add_modifier(Modifier::DIM)
            .remove_modifier(Modifier::BOLD)
            .remove_modifier(Modifier::DIM);
        assert_eq!(segments[1].style, expected);
    }

    #[test]
    fn test_cancel_italic() {
        let segments = parse_ansi("\x1b[3mitalic\x1b[23mnormal");
        assert_eq!(segments.len(), 2);
        let expected = Style::default()
            .add_modifier(Modifier::ITALIC)
            .remove_modifier(Modifier::ITALIC);
        assert_eq!(segments[1].style, expected);
    }

    #[test]
    fn test_cancel_underline() {
        let segments = parse_ansi("\x1b[4munder\x1b[24mnormal");
        assert_eq!(segments.len(), 2);
        let expected = Style::default()
            .add_modifier(Modifier::UNDERLINED)
            .remove_modifier(Modifier::UNDERLINED);
        assert_eq!(segments[1].style, expected);
    }

    // =========================================================================
    // Standard foreground colors
    // =========================================================================

    #[test]
    fn test_fg_black() {
        let segments = parse_ansi("\x1b[30mtext");
        assert_eq!(segments[0].style, Style::default().fg(Color::Black));
    }

    #[test]
    fn test_fg_red() {
        let segments = parse_ansi("\x1b[31mtext");
        assert_eq!(segments[0].style, Style::default().fg(Color::Red));
    }

    #[test]
    fn test_fg_green() {
        let segments = parse_ansi("\x1b[32mtext");
        assert_eq!(segments[0].style, Style::default().fg(Color::Green));
    }

    #[test]
    fn test_fg_yellow() {
        let segments = parse_ansi("\x1b[33mtext");
        assert_eq!(segments[0].style, Style::default().fg(Color::Yellow));
    }

    #[test]
    fn test_fg_blue() {
        let segments = parse_ansi("\x1b[34mtext");
        assert_eq!(segments[0].style, Style::default().fg(Color::Blue));
    }

    #[test]
    fn test_fg_magenta() {
        let segments = parse_ansi("\x1b[35mtext");
        assert_eq!(segments[0].style, Style::default().fg(Color::Magenta));
    }

    #[test]
    fn test_fg_cyan() {
        let segments = parse_ansi("\x1b[36mtext");
        assert_eq!(segments[0].style, Style::default().fg(Color::Cyan));
    }

    #[test]
    fn test_fg_white() {
        let segments = parse_ansi("\x1b[37mtext");
        assert_eq!(segments[0].style, Style::default().fg(Color::White));
    }

    #[test]
    fn test_fg_default() {
        let segments = parse_ansi("\x1b[31mred\x1b[39mdefault");
        assert_eq!(segments[1].style, Style::default().fg(Color::Reset));
    }

    // =========================================================================
    // Standard background colors
    // =========================================================================

    #[test]
    fn test_bg_red() {
        let segments = parse_ansi("\x1b[41mtext");
        assert_eq!(segments[0].style, Style::default().bg(Color::Red));
    }

    #[test]
    fn test_bg_green() {
        let segments = parse_ansi("\x1b[42mtext");
        assert_eq!(segments[0].style, Style::default().bg(Color::Green));
    }

    #[test]
    fn test_bg_default() {
        let segments = parse_ansi("\x1b[41mred bg\x1b[49mdefault");
        assert_eq!(segments[1].style, Style::default().bg(Color::Reset));
    }

    // =========================================================================
    // Bright foreground colors
    // =========================================================================

    #[test]
    fn test_bright_fg_red() {
        let segments = parse_ansi("\x1b[91mtext");
        assert_eq!(segments[0].style, Style::default().fg(Color::LightRed));
    }

    #[test]
    fn test_bright_fg_green() {
        let segments = parse_ansi("\x1b[92mtext");
        assert_eq!(segments[0].style, Style::default().fg(Color::LightGreen));
    }

    #[test]
    fn test_bright_fg_yellow() {
        let segments = parse_ansi("\x1b[93mtext");
        assert_eq!(segments[0].style, Style::default().fg(Color::LightYellow));
    }

    #[test]
    fn test_bright_fg_blue() {
        let segments = parse_ansi("\x1b[94mtext");
        assert_eq!(segments[0].style, Style::default().fg(Color::LightBlue));
    }

    #[test]
    fn test_bright_fg_gray() {
        // Code 90 maps to DarkGray
        let segments = parse_ansi("\x1b[90mtext");
        assert_eq!(segments[0].style, Style::default().fg(Color::DarkGray));
    }

    // =========================================================================
    // Bright background colors
    // =========================================================================

    #[test]
    fn test_bright_bg_red() {
        let segments = parse_ansi("\x1b[101mtext");
        assert_eq!(segments[0].style, Style::default().bg(Color::LightRed));
    }

    #[test]
    fn test_bright_bg_green() {
        let segments = parse_ansi("\x1b[102mtext");
        assert_eq!(segments[0].style, Style::default().bg(Color::LightGreen));
    }

    // =========================================================================
    // 256-color palette
    // =========================================================================

    #[test]
    fn test_256_fg_color() {
        let segments = parse_ansi("\x1b[38;5;196mtext");
        assert_eq!(segments[0].style, Style::default().fg(Color::Indexed(196)));
    }

    #[test]
    fn test_256_bg_color() {
        let segments = parse_ansi("\x1b[48;5;22mtext");
        assert_eq!(segments[0].style, Style::default().bg(Color::Indexed(22)));
    }

    #[test]
    fn test_256_fg_zero() {
        let segments = parse_ansi("\x1b[38;5;0mtext");
        assert_eq!(segments[0].style, Style::default().fg(Color::Indexed(0)));
    }

    #[test]
    fn test_256_fg_max() {
        let segments = parse_ansi("\x1b[38;5;255mtext");
        assert_eq!(segments[0].style, Style::default().fg(Color::Indexed(255)));
    }

    // =========================================================================
    // 24-bit RGB colors
    // =========================================================================

    #[test]
    fn test_rgb_fg_color() {
        let segments = parse_ansi("\x1b[38;2;255;128;0mtext");
        assert_eq!(
            segments[0].style,
            Style::default().fg(Color::Rgb(255, 128, 0))
        );
    }

    #[test]
    fn test_rgb_bg_color() {
        let segments = parse_ansi("\x1b[48;2;0;100;200mtext");
        assert_eq!(
            segments[0].style,
            Style::default().bg(Color::Rgb(0, 100, 200))
        );
    }

    #[test]
    fn test_rgb_fg_black() {
        let segments = parse_ansi("\x1b[38;2;0;0;0mtext");
        assert_eq!(segments[0].style, Style::default().fg(Color::Rgb(0, 0, 0)));
    }

    #[test]
    fn test_rgb_fg_white() {
        let segments = parse_ansi("\x1b[38;2;255;255;255mtext");
        assert_eq!(
            segments[0].style,
            Style::default().fg(Color::Rgb(255, 255, 255))
        );
    }

    #[test]
    fn test_rgb_fg_and_bg() {
        let segments = parse_ansi("\x1b[38;2;255;0;0;48;2;0;0;255mtext");
        assert_eq!(
            segments[0].style,
            Style::default()
                .fg(Color::Rgb(255, 0, 0))
                .bg(Color::Rgb(0, 0, 255))
        );
    }

    #[test]
    fn test_rgb_with_bold() {
        let segments = parse_ansi("\x1b[1;38;2;128;64;32mtext");
        assert_eq!(
            segments[0].style,
            Style::default()
                .fg(Color::Rgb(128, 64, 32))
                .add_modifier(Modifier::BOLD)
        );
    }

    // =========================================================================
    // Combined codes
    // =========================================================================

    #[test]
    fn test_bold_red() {
        let segments = parse_ansi("\x1b[1;31mbold red");
        assert_eq!(
            segments[0].style,
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
        );
    }

    #[test]
    fn test_bold_italic_underline() {
        let segments = parse_ansi("\x1b[1;3;4mfancy");
        assert_eq!(
            segments[0].style,
            Style::default()
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::ITALIC)
                .add_modifier(Modifier::UNDERLINED)
        );
    }

    #[test]
    fn test_fg_and_bg() {
        let segments = parse_ansi("\x1b[31;42mred on green");
        assert_eq!(
            segments[0].style,
            Style::default().fg(Color::Red).bg(Color::Green)
        );
    }

    // =========================================================================
    // Multiple segments
    // =========================================================================

    #[test]
    fn test_multiple_color_changes() {
        let segments = parse_ansi("\x1b[31mred\x1b[32mgreen\x1b[34mblue");
        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0].text, "red");
        assert_eq!(segments[0].style, Style::default().fg(Color::Red));
        assert_eq!(segments[1].text, "green");
        assert_eq!(segments[1].style, Style::default().fg(Color::Green));
        assert_eq!(segments[2].text, "blue");
        assert_eq!(segments[2].style, Style::default().fg(Color::Blue));
    }

    #[test]
    fn test_color_then_reset_then_color() {
        let segments = parse_ansi("\x1b[31mred\x1b[0mplain\x1b[34mblue");
        assert_eq!(segments.len(), 3);
        assert_eq!(segments[1].style, Style::default());
        assert_eq!(segments[2].style, Style::default().fg(Color::Blue));
    }

    // =========================================================================
    // Style persistence across segments
    // =========================================================================

    #[test]
    fn test_style_carries_over() {
        // Bold is set, then color is changed — bold should persist
        let segments = parse_ansi("\x1b[1mbold\x1b[31mstill bold red");
        assert_eq!(segments.len(), 2);
        assert_eq!(
            segments[0].style,
            Style::default().add_modifier(Modifier::BOLD)
        );
        assert_eq!(
            segments[1].style,
            Style::default().add_modifier(Modifier::BOLD).fg(Color::Red)
        );
    }

    // =========================================================================
    // Edge cases
    // =========================================================================

    #[test]
    fn test_escape_at_end() {
        let segments = parse_ansi("text\x1b[31m");
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].text, "text");
    }

    #[test]
    fn test_escape_at_start() {
        let segments = parse_ansi("\x1b[31mred");
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].text, "red");
        assert_eq!(segments[0].style, Style::default().fg(Color::Red));
    }

    #[test]
    fn test_consecutive_escapes() {
        let segments = parse_ansi("\x1b[1m\x1b[31mbold red");
        assert_eq!(segments.len(), 1);
        assert_eq!(
            segments[0].style,
            Style::default().add_modifier(Modifier::BOLD).fg(Color::Red)
        );
    }

    #[test]
    fn test_non_sgr_sequence_ignored() {
        // ESC[2J is clear screen — should be ignored
        let segments = parse_ansi("before\x1b[2Jafter");
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].text, "beforeafter");
    }

    #[test]
    fn test_bare_esc_ignored() {
        let segments = parse_ansi("hello\x1bworld");
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].text, "helloworld");
    }

    #[test]
    fn test_unknown_sgr_code_ignored() {
        let segments = parse_ansi("\x1b[999mtext");
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].style, Style::default());
    }

    #[test]
    fn test_incomplete_256_color_ignored() {
        // 38;5 without third param
        let segments = parse_ansi("\x1b[38;5mtext");
        assert_eq!(segments.len(), 1);
        // No color applied since the sequence is incomplete
        assert_eq!(segments[0].style, Style::default());
    }

    #[test]
    fn test_text_between_escapes() {
        let segments = parse_ansi("a\x1b[31mb\x1b[0mc");
        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0].text, "a");
        assert_eq!(segments[0].style, Style::default());
        assert_eq!(segments[1].text, "b");
        assert_eq!(segments[1].style, Style::default().fg(Color::Red));
        assert_eq!(segments[2].text, "c");
        assert_eq!(segments[2].style, Style::default());
    }

    #[test]
    fn test_all_standard_bg_colors() {
        let colors = [
            (40, Color::Black),
            (41, Color::Red),
            (42, Color::Green),
            (43, Color::Yellow),
            (44, Color::Blue),
            (45, Color::Magenta),
            (46, Color::Cyan),
            (47, Color::White),
        ];
        for (code, color) in &colors {
            let input = format!("\x1b[{code}mtext");
            let segments = parse_ansi(&input);
            assert_eq!(segments[0].style, Style::default().bg(*color));
        }
    }

    #[test]
    fn test_bright_fg_magenta() {
        let segments = parse_ansi("\x1b[95mtext");
        assert_eq!(segments[0].style, Style::default().fg(Color::LightMagenta));
    }

    #[test]
    fn test_bright_fg_cyan() {
        let segments = parse_ansi("\x1b[96mtext");
        assert_eq!(segments[0].style, Style::default().fg(Color::LightCyan));
    }

    #[test]
    fn test_bright_fg_white() {
        // Code 97 maps to Gray in ratatui's model
        let segments = parse_ansi("\x1b[97mtext");
        assert_eq!(segments[0].style, Style::default().fg(Color::Gray));
    }

    #[test]
    fn test_all_bright_bg_colors() {
        let colors = [
            (100, Color::DarkGray),
            (101, Color::LightRed),
            (102, Color::LightGreen),
            (103, Color::LightYellow),
            (104, Color::LightBlue),
            (105, Color::LightMagenta),
            (106, Color::LightCyan),
            (107, Color::Gray),
        ];
        for (code, color) in &colors {
            let input = format!("\x1b[{code}mtext");
            let segments = parse_ansi(&input);
            assert_eq!(segments[0].style, Style::default().bg(*color));
        }
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    use ratatui::style::Style;

    /// Generates plain text that contains no ESC (\x1b) characters.
    fn plain_text(max_len: usize) -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop::char::range('\u{20}', '\u{7E}').prop_filter("no ESC", |c| *c != '\x1b'),
            1..=max_len,
        )
        .prop_map(|chars| chars.into_iter().collect())
    }

    /// Generates a valid SGR escape sequence (e.g., `\x1b[1;31m`).
    fn sgr_sequence() -> impl Strategy<Value = String> {
        prop_oneof![
            // Single SGR code
            prop::sample::select(vec![
                0u16, 1, 2, 3, 4, 22, 23, 24, 30, 31, 32, 33, 34, 35, 36, 37, 39, 40, 41, 42, 43,
                44, 45, 46, 47, 49, 90, 91, 92, 93, 94, 95, 96, 97, 100, 101, 102, 103, 104, 105,
                106, 107,
            ])
            .prop_map(|code| format!("\x1b[{code}m")),
            // 256-color foreground: 38;5;n
            (0..=255u8).prop_map(|n| format!("\x1b[38;5;{n}m")),
            // 256-color background: 48;5;n
            (0..=255u8).prop_map(|n| format!("\x1b[48;5;{n}m")),
            // 24-bit RGB foreground: 38;2;r;g;b
            (0..=255u8, 0..=255u8, 0..=255u8)
                .prop_map(|(r, g, b)| format!("\x1b[38;2;{r};{g};{b}m")),
            // 24-bit RGB background: 48;2;r;g;b
            (0..=255u8, 0..=255u8, 0..=255u8)
                .prop_map(|(r, g, b)| format!("\x1b[48;2;{r};{g};{b}m")),
            // Combined: bold + color
            prop::sample::select(vec![31u16, 32, 33, 34, 35, 36, 37])
                .prop_map(|code| format!("\x1b[1;{code}m")),
            // Combined: bold + italic + underline
            Just("\x1b[1;3;4m".to_string()),
            // Reset
            Just("\x1b[0m".to_string()),
            // Bare reset (ESC[m)
            Just("\x1b[m".to_string()),
        ]
    }

    /// Builds an input string by interleaving SGR sequences and plain text.
    fn ansi_input() -> impl Strategy<Value = String> {
        prop::collection::vec(prop_oneof![plain_text(20), sgr_sequence()], 0..=10)
            .prop_map(|parts| parts.into_iter().collect::<String>())
    }

    /// Strips ANSI escape sequences, returning only visible text (test oracle).
    fn strip_ansi(input: &str) -> String {
        let mut result = String::new();
        let mut chars = input.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '\x1b' {
                if chars.peek() == Some(&'[') {
                    chars.next(); // consume '['
                    // Consume parameter bytes and the final byte
                    while let Some(&c) = chars.peek() {
                        if c.is_ascii_digit() || c == ';' {
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    // Consume final byte
                    chars.next();
                }
                // Bare ESC without '[' — just skip the ESC
            } else {
                result.push(ch);
            }
        }
        result
    }

    proptest! {
        /// Concatenating all segment text values reproduces the original
        /// input with escape sequences stripped.
        #[test]
        fn round_trip_text_preservation(input in ansi_input()) {
            let segments = parse_ansi(&input);
            let reconstructed: String = segments.iter().map(|s| s.text.as_str()).collect();
            let expected = strip_ansi(&input);
            prop_assert_eq!(reconstructed, expected);
        }

        /// The parser never panics on arbitrary input bytes.
        #[test]
        fn no_panics_on_arbitrary_input(input in "\\PC{0,200}") {
            let _ = parse_ansi(&input);
        }

        /// After an explicit reset sequence (\x1b[0m), the next segment's
        /// style must be Style::default() (assuming no further SGR codes
        /// intervene before the text).
        #[test]
        fn style_resets_after_reset_code(
            pre_text in plain_text(10),
            post_text in plain_text(10),
            sgr in sgr_sequence(),
        ) {
            // Build: <sgr><pre_text>\x1b[0m<post_text>
            let input = format!("{sgr}{pre_text}\x1b[0m{post_text}");
            let segments = parse_ansi(&input);

            // Find the segment containing post_text — it should be the last one
            let last = segments.last().expect("should have at least one segment");
            prop_assert_eq!(
                &last.text, &post_text,
                "last segment text should be the post-reset text"
            );
            prop_assert_eq!(
                last.style,
                Style::default(),
                "style after reset should be default, got {:?}",
                last.style
            );
        }

        /// Every AnsiSegment produced by the parser has non-empty text.
        /// Escape-only sequences produce no segments.
        #[test]
        fn segment_text_is_never_empty(input in ansi_input()) {
            let segments = parse_ansi(&input);
            for (i, segment) in segments.iter().enumerate() {
                prop_assert!(
                    !segment.text.is_empty(),
                    "segment {} has empty text in parse of {:?}",
                    i,
                    input
                );
            }
        }

        /// An input consisting entirely of escape sequences yields no segments.
        #[test]
        fn escape_only_input_yields_no_segments(
            escapes in prop::collection::vec(sgr_sequence(), 1..=5),
        ) {
            let input: String = escapes.into_iter().collect();
            let segments = parse_ansi(&input);
            prop_assert!(
                segments.is_empty(),
                "expected no segments for escape-only input {:?}, got {}",
                input,
                segments.len()
            );
        }

        /// The bare reset sequence ESC[m behaves identically to ESC[0m.
        #[test]
        fn bare_reset_equals_explicit_reset(
            pre_text in plain_text(10),
            post_text in plain_text(10),
            sgr in sgr_sequence(),
        ) {
            let with_bare = format!("{sgr}{pre_text}\x1b[m{post_text}");
            let with_explicit = format!("{sgr}{pre_text}\x1b[0m{post_text}");
            let segments_bare = parse_ansi(&with_bare);
            let segments_explicit = parse_ansi(&with_explicit);
            prop_assert_eq!(segments_bare, segments_explicit);
        }
    }
}
