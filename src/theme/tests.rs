use super::*;

#[test]
fn test_default_theme() {
    let theme = Theme::default();
    assert_eq!(theme.focused, Color::Yellow);
    assert_eq!(theme.disabled, Color::DarkGray);
    assert_eq!(theme.success, Color::Green);
    assert_eq!(theme.warning, Color::Yellow);
    assert_eq!(theme.error, Color::Red);
    assert_eq!(theme.info, Color::Cyan);
}

#[test]
fn test_nord_theme() {
    let theme = Theme::nord();
    assert_eq!(theme.focused, NORD8);
    assert_eq!(theme.selected, NORD9);
    assert_eq!(theme.disabled, NORD3);
    assert_eq!(theme.success, NORD14);
    assert_eq!(theme.warning, NORD13);
    assert_eq!(theme.error, NORD11);
    assert_eq!(theme.info, NORD8);
    assert_eq!(theme.background, NORD0);
    assert_eq!(theme.foreground, NORD6);
}

#[test]
fn test_nord_colors() {
    assert_eq!(NORD0, Color::Rgb(46, 52, 64));
    assert_eq!(NORD8, Color::Rgb(136, 192, 208));
    assert_eq!(NORD14, Color::Rgb(163, 190, 140));
}

#[test]
fn test_focused_style() {
    let theme = Theme::default();
    let style = theme.focused_style();
    assert_eq!(style.fg, Some(Color::Yellow));
}

#[test]
fn test_focused_bold_style() {
    let theme = Theme::default();
    let style = theme.focused_bold_style();
    assert_eq!(style.fg, Some(Color::Yellow));
    assert!(style.add_modifier.contains(Modifier::BOLD));
}

#[test]
fn test_selected_style_focused() {
    let theme = Theme::default();
    let style = theme.selected_style(true);
    assert_eq!(style.fg, Some(Color::Yellow));
    assert!(style.add_modifier.contains(Modifier::BOLD));
}

#[test]
fn test_selected_style_unfocused() {
    let theme = Theme::default();
    let style = theme.selected_style(false);
    assert_eq!(style.fg, None);
    assert!(style.add_modifier.contains(Modifier::BOLD));
}

#[test]
fn test_disabled_style() {
    let theme = Theme::default();
    let style = theme.disabled_style();
    assert_eq!(style.fg, Some(Color::DarkGray));
}

#[test]
fn test_placeholder_style() {
    let theme = Theme::default();
    let style = theme.placeholder_style();
    assert_eq!(style.fg, Some(Color::DarkGray));
}

#[test]
fn test_semantic_styles() {
    let theme = Theme::default();

    let success = theme.success_style();
    assert_eq!(success.fg, Some(Color::Green));

    let warning = theme.warning_style();
    assert_eq!(warning.fg, Some(Color::Yellow));

    let error = theme.error_style();
    assert_eq!(error.fg, Some(Color::Red));

    let info = theme.info_style();
    assert_eq!(info.fg, Some(Color::Cyan));
}

#[test]
fn test_progress_filled_style() {
    let theme = Theme::default();
    let style = theme.progress_filled_style();
    assert_eq!(style.fg, Some(Color::Cyan));
    assert_eq!(style.bg, Some(Color::Black));
}

#[test]
fn test_theme_clone() {
    let theme = Theme::nord();
    let cloned = theme.clone();
    assert_eq!(theme, cloned);
}

#[test]
fn test_custom_theme() {
    let custom = Theme {
        focused: Color::Magenta,
        selected: Color::LightBlue,
        ..Theme::default()
    };
    assert_eq!(custom.focused, Color::Magenta);
    assert_eq!(custom.selected, Color::LightBlue);
    assert_eq!(custom.disabled, Color::DarkGray); // From default
}

#[test]
fn test_normal_style() {
    let theme = Theme::default();
    let style = theme.normal_style();
    // Default theme uses Color::Reset for fg/bg
    assert_eq!(style.fg, Some(Color::Reset));
    assert_eq!(style.bg, Some(Color::Reset));
}

#[test]
fn test_normal_style_nord() {
    let theme = Theme::nord();
    let style = theme.normal_style();
    assert_eq!(style.fg, Some(NORD6));
    assert_eq!(style.bg, Some(NORD0));
}

#[test]
fn test_focused_border_style_differs_from_focused_style() {
    let theme = Theme::nord();
    let border_style = theme.focused_border_style();
    let focused_style = theme.focused_style();
    // Both use focused color for fg
    assert_eq!(border_style.fg, Some(NORD8));
    assert_eq!(focused_style.fg, Some(NORD8));
    // Border style includes background, focused style does not
    assert_eq!(border_style.bg, Some(NORD0));
    assert_eq!(focused_style.bg, None);
}

#[test]
fn test_primary_style() {
    let theme = Theme::default();
    let style = theme.primary_style();
    assert_eq!(style.fg, Some(Color::Cyan));
}

#[test]
fn test_primary_style_nord() {
    let theme = Theme::nord();
    let style = theme.primary_style();
    assert_eq!(style.fg, Some(NORD10));
}

#[test]
fn test_border_style() {
    let theme = Theme::nord();
    let style = theme.border_style();
    assert_eq!(style.fg, Some(NORD3));
}

#[test]
fn test_selected_highlight_style_focused() {
    let theme = Theme::nord();
    let style = theme.selected_highlight_style(true);
    assert_eq!(style.bg, Some(NORD9));
    assert_eq!(style.fg, Some(NORD6));
}

#[test]
fn test_selected_highlight_style_unfocused() {
    let theme = Theme::nord();
    let style = theme.selected_highlight_style(false);
    assert_eq!(style.bg, Some(NORD3));
    assert_eq!(style.fg, Some(NORD6));
}

#[test]
fn test_dracula_theme() {
    let theme = Theme::dracula();
    assert_eq!(theme.background, DRACULA_BG);
    assert_eq!(theme.foreground, DRACULA_FG);
    assert_eq!(theme.border, DRACULA_COMMENT);
    assert_eq!(theme.focused, DRACULA_PURPLE);
    assert_eq!(theme.selected, DRACULA_PINK);
    assert_eq!(theme.disabled, DRACULA_COMMENT);
    assert_eq!(theme.placeholder, DRACULA_COMMENT);
    assert_eq!(theme.primary, DRACULA_CYAN);
    assert_eq!(theme.success, DRACULA_GREEN);
    assert_eq!(theme.warning, DRACULA_YELLOW);
    assert_eq!(theme.error, DRACULA_RED);
    assert_eq!(theme.info, DRACULA_CYAN);
    assert_eq!(theme.progress_filled, DRACULA_PURPLE);
    assert_eq!(theme.progress_empty, DRACULA_CURRENT);
}

#[test]
fn test_dracula_colors() {
    assert_eq!(DRACULA_BG, Color::Rgb(40, 42, 54));
    assert_eq!(DRACULA_PURPLE, Color::Rgb(189, 147, 249));
    assert_eq!(DRACULA_GREEN, Color::Rgb(80, 250, 123));
}

#[test]
fn test_solarized_dark_theme() {
    let theme = Theme::solarized_dark();
    assert_eq!(theme.background, SOLARIZED_BASE03);
    assert_eq!(theme.foreground, SOLARIZED_BASE0);
    assert_eq!(theme.border, SOLARIZED_BASE01);
    assert_eq!(theme.focused, SOLARIZED_BLUE);
    assert_eq!(theme.selected, SOLARIZED_CYAN);
    assert_eq!(theme.disabled, SOLARIZED_BASE01);
    assert_eq!(theme.placeholder, SOLARIZED_BASE01);
    assert_eq!(theme.primary, SOLARIZED_BLUE);
    assert_eq!(theme.success, SOLARIZED_GREEN);
    assert_eq!(theme.warning, SOLARIZED_YELLOW);
    assert_eq!(theme.error, SOLARIZED_RED);
    assert_eq!(theme.info, SOLARIZED_CYAN);
    assert_eq!(theme.progress_filled, SOLARIZED_BLUE);
    assert_eq!(theme.progress_empty, SOLARIZED_BASE02);
}

#[test]
fn test_solarized_dark_colors() {
    assert_eq!(SOLARIZED_BASE03, Color::Rgb(0, 43, 54));
    assert_eq!(SOLARIZED_BLUE, Color::Rgb(38, 139, 210));
    assert_eq!(SOLARIZED_GREEN, Color::Rgb(133, 153, 0));
}

#[test]
fn test_gruvbox_dark_theme() {
    let theme = Theme::gruvbox_dark();
    assert_eq!(theme.background, GRUVBOX_BG);
    assert_eq!(theme.foreground, GRUVBOX_FG);
    assert_eq!(theme.border, GRUVBOX_GRAY);
    assert_eq!(theme.focused, GRUVBOX_YELLOW);
    assert_eq!(theme.selected, GRUVBOX_BLUE);
    assert_eq!(theme.disabled, GRUVBOX_GRAY);
    assert_eq!(theme.placeholder, GRUVBOX_GRAY);
    assert_eq!(theme.primary, GRUVBOX_AQUA);
    assert_eq!(theme.success, GRUVBOX_GREEN);
    assert_eq!(theme.warning, GRUVBOX_ORANGE);
    assert_eq!(theme.error, GRUVBOX_RED);
    assert_eq!(theme.info, GRUVBOX_BLUE);
    assert_eq!(theme.progress_filled, GRUVBOX_YELLOW);
    assert_eq!(theme.progress_empty, GRUVBOX_BG1);
}

#[test]
fn test_gruvbox_dark_colors() {
    assert_eq!(GRUVBOX_BG, Color::Rgb(40, 40, 40));
    assert_eq!(GRUVBOX_YELLOW, Color::Rgb(250, 189, 47));
    assert_eq!(GRUVBOX_GREEN, Color::Rgb(184, 187, 38));
}

#[test]
fn test_all_themes_distinct() {
    let themes = [
        Theme::default(),
        Theme::nord(),
        Theme::dracula(),
        Theme::solarized_dark(),
        Theme::gruvbox_dark(),
    ];
    for i in 0..themes.len() {
        for j in (i + 1)..themes.len() {
            assert_ne!(
                themes[i], themes[j],
                "themes at indices {} and {} should differ",
                i, j
            );
        }
    }
}
