use super::*;

// -------------------------------------------------------------------------
// EnhancedCell tests
// -------------------------------------------------------------------------

#[test]
fn test_enhanced_cell_default() {
    let cell = EnhancedCell::new();
    assert_eq!(cell.symbol(), " ");
    assert_eq!(cell.fg, SerializableColor::Reset);
    assert_eq!(cell.bg, SerializableColor::Reset);
    assert!(cell.modifiers.is_empty());
    assert!(cell.is_empty());
}

#[test]
fn test_enhanced_cell_with_symbol() {
    let cell = EnhancedCell::with_symbol("A");
    assert_eq!(cell.symbol(), "A");
    assert!(!cell.is_empty());
}

#[test]
fn test_enhanced_cell_set_symbol() {
    let mut cell = EnhancedCell::new();
    cell.set_symbol("Hello");
    assert_eq!(cell.symbol(), "Hello");
}

#[test]
fn test_enhanced_cell_set_char() {
    let mut cell = EnhancedCell::new();
    cell.set_char('X');
    assert_eq!(cell.symbol(), "X");
}

#[test]
fn test_enhanced_cell_symbol_width() {
    let cell = EnhancedCell::with_symbol("A");
    assert_eq!(cell.symbol_width(), 1);

    // Wide character (e.g., CJK)
    let cell = EnhancedCell::with_symbol("日");
    assert_eq!(cell.symbol_width(), 2);
}

#[test]
fn test_enhanced_cell_set_style() {
    let mut cell = EnhancedCell::new();
    let style = Style::new()
        .fg(Color::Red)
        .bg(Color::Blue)
        .add_modifier(Modifier::BOLD);

    cell.set_style(style);

    assert_eq!(cell.fg, SerializableColor::Red);
    assert_eq!(cell.bg, SerializableColor::Blue);
    assert!(cell.modifiers.bold);
}

#[test]
fn test_enhanced_cell_set_style_with_underline_color() {
    let mut cell = EnhancedCell::new();
    let style = Style::new().underline_color(Color::Green);

    cell.set_style(style);

    assert_eq!(cell.underline_color, Some(SerializableColor::Green));
}

#[test]
fn test_enhanced_cell_set_style_sub_modifier() {
    let mut cell = EnhancedCell::new();
    cell.modifiers.bold = true;
    cell.modifiers.italic = true;

    let style = Style::new().remove_modifier(Modifier::BOLD);
    cell.set_style(style);

    assert!(!cell.modifiers.bold);
    assert!(cell.modifiers.italic);
}

#[test]
fn test_enhanced_cell_style() {
    let mut cell = EnhancedCell::new();
    cell.fg = SerializableColor::Red;
    cell.bg = SerializableColor::Blue;
    cell.modifiers.bold = true;

    let style = cell.style();
    assert_eq!(style.fg, Some(Color::Red));
    assert_eq!(style.bg, Some(Color::Blue));
    assert!(style.add_modifier.contains(Modifier::BOLD));
}

#[test]
fn test_enhanced_cell_reset() {
    let mut cell = EnhancedCell::new();
    cell.set_char('X');
    cell.fg = SerializableColor::Red;
    cell.modifiers.bold = true;
    cell.skip = true;

    cell.reset();

    assert_eq!(cell.symbol(), " ");
    assert_eq!(cell.fg, SerializableColor::Reset);
    assert!(cell.modifiers.is_empty());
    assert!(!cell.skip);
}

#[test]
fn test_enhanced_cell_is_empty() {
    let mut cell = EnhancedCell::new();
    assert!(cell.is_empty());

    cell.fg = SerializableColor::Red;
    assert!(!cell.is_empty());

    cell.fg = SerializableColor::Reset;
    cell.bg = SerializableColor::Blue;
    assert!(!cell.is_empty());

    cell.bg = SerializableColor::Reset;
    cell.modifiers.bold = true;
    assert!(!cell.is_empty());
}

#[test]
fn test_enhanced_cell_from_ratatui_cell() {
    let mut ratatui_cell = ratatui::buffer::Cell::default();
    ratatui_cell.set_char('Z');
    ratatui_cell.set_style(
        Style::new()
            .fg(Color::Yellow)
            .add_modifier(Modifier::ITALIC),
    );

    let cell = EnhancedCell::from_ratatui_cell(&ratatui_cell, 42);

    assert_eq!(cell.symbol(), "Z");
    assert_eq!(cell.fg, SerializableColor::Yellow);
    assert!(cell.modifiers.italic);
    assert_eq!(cell.last_modified_frame, 42);
}

#[test]
fn test_enhanced_cell_default_trait() {
    let cell: EnhancedCell = Default::default();
    assert_eq!(cell.symbol(), " ");
}

// -------------------------------------------------------------------------
// SerializableColor tests - all color conversions
// -------------------------------------------------------------------------

#[test]
fn test_serializable_color_roundtrip() {
    let colors = vec![
        Color::Red,
        Color::Rgb(100, 150, 200),
        Color::Indexed(42),
        Color::Reset,
    ];

    for color in colors {
        let serializable = SerializableColor::from(color);
        let back: Color = serializable.into();
        assert_eq!(color, back);
    }
}

#[test]
fn test_serializable_color_all_basic_colors() {
    let pairs = [
        (Color::Reset, SerializableColor::Reset),
        (Color::Black, SerializableColor::Black),
        (Color::Red, SerializableColor::Red),
        (Color::Green, SerializableColor::Green),
        (Color::Yellow, SerializableColor::Yellow),
        (Color::Blue, SerializableColor::Blue),
        (Color::Magenta, SerializableColor::Magenta),
        (Color::Cyan, SerializableColor::Cyan),
        (Color::Gray, SerializableColor::Gray),
        (Color::DarkGray, SerializableColor::DarkGray),
        (Color::LightRed, SerializableColor::LightRed),
        (Color::LightGreen, SerializableColor::LightGreen),
        (Color::LightYellow, SerializableColor::LightYellow),
        (Color::LightBlue, SerializableColor::LightBlue),
        (Color::LightMagenta, SerializableColor::LightMagenta),
        (Color::LightCyan, SerializableColor::LightCyan),
        (Color::White, SerializableColor::White),
    ];

    for (ratatui, serializable) in pairs {
        assert_eq!(SerializableColor::from(ratatui), serializable);
        assert_eq!(Color::from(serializable), ratatui);
    }
}

#[test]
fn test_serializable_color_rgb() {
    let color = Color::Rgb(10, 20, 30);
    let serializable = SerializableColor::from(color);
    assert_eq!(
        serializable,
        SerializableColor::Rgb {
            r: 10,
            g: 20,
            b: 30
        }
    );
    assert_eq!(Color::from(serializable), color);
}

#[test]
fn test_serializable_color_indexed() {
    let color = Color::Indexed(200);
    let serializable = SerializableColor::from(color);
    assert_eq!(serializable, SerializableColor::Indexed(200));
    assert_eq!(Color::from(serializable), color);
}

// -------------------------------------------------------------------------
// SerializableColor ANSI code tests - foreground
// -------------------------------------------------------------------------

#[test]
fn test_ansi_fg_codes() {
    assert_eq!(SerializableColor::Red.to_ansi_fg(), "\x1b[31m");
    assert_eq!(
        SerializableColor::Rgb {
            r: 255,
            g: 128,
            b: 0
        }
        .to_ansi_fg(),
        "\x1b[38;2;255;128;0m"
    );
    assert_eq!(SerializableColor::Indexed(42).to_ansi_fg(), "\x1b[38;5;42m");
}

#[test]
fn test_ansi_fg_all_basic_colors() {
    assert_eq!(SerializableColor::Reset.to_ansi_fg(), "\x1b[39m");
    assert_eq!(SerializableColor::Black.to_ansi_fg(), "\x1b[30m");
    assert_eq!(SerializableColor::Red.to_ansi_fg(), "\x1b[31m");
    assert_eq!(SerializableColor::Green.to_ansi_fg(), "\x1b[32m");
    assert_eq!(SerializableColor::Yellow.to_ansi_fg(), "\x1b[33m");
    assert_eq!(SerializableColor::Blue.to_ansi_fg(), "\x1b[34m");
    assert_eq!(SerializableColor::Magenta.to_ansi_fg(), "\x1b[35m");
    assert_eq!(SerializableColor::Cyan.to_ansi_fg(), "\x1b[36m");
    assert_eq!(SerializableColor::Gray.to_ansi_fg(), "\x1b[37m");
    assert_eq!(SerializableColor::DarkGray.to_ansi_fg(), "\x1b[90m");
    assert_eq!(SerializableColor::LightRed.to_ansi_fg(), "\x1b[91m");
    assert_eq!(SerializableColor::LightGreen.to_ansi_fg(), "\x1b[92m");
    assert_eq!(SerializableColor::LightYellow.to_ansi_fg(), "\x1b[93m");
    assert_eq!(SerializableColor::LightBlue.to_ansi_fg(), "\x1b[94m");
    assert_eq!(SerializableColor::LightMagenta.to_ansi_fg(), "\x1b[95m");
    assert_eq!(SerializableColor::LightCyan.to_ansi_fg(), "\x1b[96m");
    assert_eq!(SerializableColor::White.to_ansi_fg(), "\x1b[97m");
}

// -------------------------------------------------------------------------
// SerializableColor ANSI code tests - background
// -------------------------------------------------------------------------

#[test]
fn test_ansi_bg_all_basic_colors() {
    assert_eq!(SerializableColor::Reset.to_ansi_bg(), "\x1b[49m");
    assert_eq!(SerializableColor::Black.to_ansi_bg(), "\x1b[40m");
    assert_eq!(SerializableColor::Red.to_ansi_bg(), "\x1b[41m");
    assert_eq!(SerializableColor::Green.to_ansi_bg(), "\x1b[42m");
    assert_eq!(SerializableColor::Yellow.to_ansi_bg(), "\x1b[43m");
    assert_eq!(SerializableColor::Blue.to_ansi_bg(), "\x1b[44m");
    assert_eq!(SerializableColor::Magenta.to_ansi_bg(), "\x1b[45m");
    assert_eq!(SerializableColor::Cyan.to_ansi_bg(), "\x1b[46m");
    assert_eq!(SerializableColor::Gray.to_ansi_bg(), "\x1b[47m");
    assert_eq!(SerializableColor::DarkGray.to_ansi_bg(), "\x1b[100m");
    assert_eq!(SerializableColor::LightRed.to_ansi_bg(), "\x1b[101m");
    assert_eq!(SerializableColor::LightGreen.to_ansi_bg(), "\x1b[102m");
    assert_eq!(SerializableColor::LightYellow.to_ansi_bg(), "\x1b[103m");
    assert_eq!(SerializableColor::LightBlue.to_ansi_bg(), "\x1b[104m");
    assert_eq!(SerializableColor::LightMagenta.to_ansi_bg(), "\x1b[105m");
    assert_eq!(SerializableColor::LightCyan.to_ansi_bg(), "\x1b[106m");
    assert_eq!(SerializableColor::White.to_ansi_bg(), "\x1b[107m");
}

#[test]
fn test_ansi_bg_rgb() {
    assert_eq!(
        SerializableColor::Rgb {
            r: 10,
            g: 20,
            b: 30
        }
        .to_ansi_bg(),
        "\x1b[48;2;10;20;30m"
    );
}

#[test]
fn test_ansi_bg_indexed() {
    assert_eq!(
        SerializableColor::Indexed(123).to_ansi_bg(),
        "\x1b[48;5;123m"
    );
}

// -------------------------------------------------------------------------
// SerializableModifier tests
// -------------------------------------------------------------------------

#[test]
fn test_serializable_modifier_roundtrip() {
    let modifier = Modifier::BOLD | Modifier::ITALIC | Modifier::UNDERLINED;
    let serializable = SerializableModifier::from(modifier);
    assert!(serializable.bold);
    assert!(serializable.italic);
    assert!(serializable.underlined);
    assert!(!serializable.dim);

    let back: Modifier = serializable.into();
    assert_eq!(modifier, back);
}

#[test]
fn test_serializable_modifier_all_flags() {
    let all = Modifier::BOLD
        | Modifier::DIM
        | Modifier::ITALIC
        | Modifier::UNDERLINED
        | Modifier::SLOW_BLINK
        | Modifier::RAPID_BLINK
        | Modifier::REVERSED
        | Modifier::HIDDEN
        | Modifier::CROSSED_OUT;

    let serializable = SerializableModifier::from(all);
    assert!(serializable.bold);
    assert!(serializable.dim);
    assert!(serializable.italic);
    assert!(serializable.underlined);
    assert!(serializable.slow_blink);
    assert!(serializable.rapid_blink);
    assert!(serializable.reversed);
    assert!(serializable.hidden);
    assert!(serializable.crossed_out);

    let back: Modifier = serializable.into();
    assert_eq!(all, back);
}

#[test]
fn test_serializable_modifier_union() {
    let a = SerializableModifier {
        bold: true,
        ..Default::default()
    };
    let b = SerializableModifier {
        italic: true,
        ..Default::default()
    };
    let union = a.union(b);

    assert!(union.bold);
    assert!(union.italic);
    assert!(!union.dim);
}

#[test]
fn test_serializable_modifier_difference() {
    let a = SerializableModifier {
        bold: true,
        italic: true,
        ..Default::default()
    };
    let b = SerializableModifier {
        bold: true,
        ..Default::default()
    };
    let diff = a.difference(b);

    assert!(!diff.bold);
    assert!(diff.italic);
}

#[test]
fn test_modifier_ansi_codes() {
    let mut modifier = SerializableModifier::empty();
    modifier.bold = true;
    modifier.italic = true;
    assert_eq!(modifier.to_ansi(), "\x1b[1;3m");
}

#[test]
fn test_modifier_ansi_empty() {
    let modifier = SerializableModifier::empty();
    assert_eq!(modifier.to_ansi(), "");
}

#[test]
fn test_modifier_ansi_all() {
    let modifier = SerializableModifier {
        bold: true,
        dim: true,
        italic: true,
        underlined: true,
        slow_blink: true,
        rapid_blink: true,
        reversed: true,
        hidden: true,
        crossed_out: true,
    };
    assert_eq!(modifier.to_ansi(), "\x1b[1;2;3;4;5;6;7;8;9m");
}

#[test]
fn test_modifier_is_empty() {
    let empty = SerializableModifier::empty();
    assert!(empty.is_empty());

    let mut not_empty = SerializableModifier::empty();
    not_empty.bold = true;
    assert!(!not_empty.is_empty());
}

// -------------------------------------------------------------------------
// Serialization tests
// -------------------------------------------------------------------------

#[test]
fn test_cell_serialization() {
    let cell = EnhancedCell {
        symbol: CompactString::from("X"),
        fg: SerializableColor::Red,
        bg: SerializableColor::Blue,
        modifiers: SerializableModifier {
            bold: true,
            ..Default::default()
        },
        underline_color: None,
        last_modified_frame: 5,
        skip: false,
    };

    let json = serde_json::to_string(&cell).unwrap();
    let deserialized: EnhancedCell = serde_json::from_str(&json).unwrap();
    assert_eq!(cell, deserialized);
}

#[test]
fn test_cell_serialization_with_underline_color() {
    let cell = EnhancedCell {
        symbol: CompactString::from("Y"),
        fg: SerializableColor::Reset,
        bg: SerializableColor::Reset,
        modifiers: SerializableModifier::empty(),
        underline_color: Some(SerializableColor::Green),
        last_modified_frame: 0,
        skip: true,
    };

    let json = serde_json::to_string(&cell).unwrap();
    let deserialized: EnhancedCell = serde_json::from_str(&json).unwrap();
    assert_eq!(cell, deserialized);
}
