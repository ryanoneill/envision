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
fn test_catppuccin_mocha_theme() {
    let theme = Theme::catppuccin_mocha();
    assert_eq!(theme.background, CATPPUCCIN_BASE);
    assert_eq!(theme.foreground, CATPPUCCIN_TEXT);
    assert_eq!(theme.border, CATPPUCCIN_SURFACE2);
    assert_eq!(theme.focused, CATPPUCCIN_LAVENDER);
    assert_eq!(theme.selected, CATPPUCCIN_MAUVE);
    assert_eq!(theme.disabled, CATPPUCCIN_SURFACE2);
    assert_eq!(theme.placeholder, CATPPUCCIN_OVERLAY0);
    assert_eq!(theme.primary, CATPPUCCIN_BLUE);
    assert_eq!(theme.success, CATPPUCCIN_GREEN);
    assert_eq!(theme.warning, CATPPUCCIN_YELLOW);
    assert_eq!(theme.error, CATPPUCCIN_RED);
    assert_eq!(theme.info, CATPPUCCIN_SAPPHIRE);
    assert_eq!(theme.progress_filled, CATPPUCCIN_LAVENDER);
    assert_eq!(theme.progress_empty, CATPPUCCIN_SURFACE0);
}

#[test]
fn test_catppuccin_mocha_colors() {
    assert_eq!(CATPPUCCIN_BASE, Color::Rgb(30, 30, 46));
    assert_eq!(CATPPUCCIN_LAVENDER, Color::Rgb(180, 190, 254));
    assert_eq!(CATPPUCCIN_GREEN, Color::Rgb(166, 227, 161));
}

#[test]
fn test_all_themes_distinct() {
    let themes = [
        Theme::default(),
        Theme::nord(),
        Theme::dracula(),
        Theme::solarized_dark(),
        Theme::gruvbox_dark(),
        Theme::catppuccin_mocha(),
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

#[test]
fn test_severity_enum_variants() {
    // Pin the four severity variants and their ordering.
    let _good = Severity::Good;
    let _mild = Severity::Mild;
    let _bad = Severity::Bad;
    let _critical = Severity::Critical;
    // Pin Copy/Clone/Eq so consumers can destructure freely.
    let s = Severity::Bad;
    let s2 = s;
    assert_eq!(s, s2);
}

#[test]
fn test_severity_from_thresholds_band_boundaries() {
    let thresholds = [
        (1.0, Severity::Good),
        (3.0, Severity::Mild),
        (10.0, Severity::Bad),
    ];
    // Below first cutoff: Good.
    assert_eq!(Severity::from_thresholds(0.5, &thresholds), Severity::Good);
    assert_eq!(
        Severity::from_thresholds(0.999, &thresholds),
        Severity::Good
    );
    // At cutoff falls through to next band (Mild).
    assert_eq!(Severity::from_thresholds(1.0, &thresholds), Severity::Mild);
    // Inside Mild range.
    assert_eq!(Severity::from_thresholds(2.0, &thresholds), Severity::Mild);
    assert_eq!(
        Severity::from_thresholds(2.999, &thresholds),
        Severity::Mild
    );
    // At Mild cutoff falls through to Bad.
    assert_eq!(Severity::from_thresholds(3.0, &thresholds), Severity::Bad);
    assert_eq!(Severity::from_thresholds(5.0, &thresholds), Severity::Bad);
    // At Bad cutoff falls through to Critical (default).
    assert_eq!(
        Severity::from_thresholds(10.0, &thresholds),
        Severity::Critical
    );
    assert_eq!(
        Severity::from_thresholds(20.0, &thresholds),
        Severity::Critical
    );
}

#[test]
fn test_severity_from_thresholds_empty() {
    // Empty threshold slice: every value is Critical.
    assert_eq!(Severity::from_thresholds(0.0, &[]), Severity::Critical);
    assert_eq!(Severity::from_thresholds(-1.0, &[]), Severity::Critical);
    assert_eq!(Severity::from_thresholds(1e9, &[]), Severity::Critical);
}

#[test]
fn test_severity_from_thresholds_unsorted_first_match_wins() {
    // Documented first-match-wins: iteration is in slice order, not by sorted cutoff.
    // Unsorted thresholds give well-defined but possibly counter-intuitive results.
    let unsorted = [
        (10.0, Severity::Bad),
        (1.0, Severity::Good),
        (3.0, Severity::Mild),
    ];
    // value=2.0: first cutoff is 10.0; 2.0 < 10.0, so returns Bad immediately.
    assert_eq!(Severity::from_thresholds(2.0, &unsorted), Severity::Bad);
    // value=15.0: 15.0 < 10.0? no. 15.0 < 1.0? no. 15.0 < 3.0? no. Critical default.
    assert_eq!(
        Severity::from_thresholds(15.0, &unsorted),
        Severity::Critical
    );
}

#[test]
fn test_named_color_enum_variants() {
    // Pin all 26 NamedColor variants — the spec's complete Catppuccin-derived palette.
    let variants = [
        NamedColor::Rosewater,
        NamedColor::Flamingo,
        NamedColor::Pink,
        NamedColor::Mauve,
        NamedColor::Red,
        NamedColor::Maroon,
        NamedColor::Peach,
        NamedColor::Yellow,
        NamedColor::Green,
        NamedColor::Teal,
        NamedColor::Sky,
        NamedColor::Sapphire,
        NamedColor::Blue,
        NamedColor::Lavender,
        NamedColor::Text,
        NamedColor::Subtext1,
        NamedColor::Subtext0,
        NamedColor::Overlay2,
        NamedColor::Overlay1,
        NamedColor::Overlay0,
        NamedColor::Surface2,
        NamedColor::Surface1,
        NamedColor::Surface0,
        NamedColor::Base,
        NamedColor::Mantle,
        NamedColor::Crust,
    ];
    assert_eq!(variants.len(), 26);
}

#[test]
fn test_palette_struct_construction() {
    // Direct construction: a custom user theme can build a Palette with no envision changes.
    let custom = Palette {
        rosewater: Color::Rgb(255, 0, 0),
        flamingo: Color::Rgb(255, 0, 0),
        pink: Color::Rgb(255, 0, 0),
        mauve: Color::Rgb(255, 0, 0),
        red: Color::Rgb(255, 0, 0),
        maroon: Color::Rgb(255, 0, 0),
        peach: Color::Rgb(255, 0, 0),
        yellow: Color::Rgb(255, 0, 0),
        green: Color::Rgb(255, 0, 0),
        teal: Color::Rgb(255, 0, 0),
        sky: Color::Rgb(255, 0, 0),
        sapphire: Color::Rgb(255, 0, 0),
        blue: Color::Rgb(255, 0, 0),
        lavender: Color::Rgb(255, 0, 0),
        text: Color::Rgb(255, 0, 0),
        subtext1: Color::Rgb(255, 0, 0),
        subtext0: Color::Rgb(255, 0, 0),
        overlay2: Color::Rgb(255, 0, 0),
        overlay1: Color::Rgb(255, 0, 0),
        overlay0: Color::Rgb(255, 0, 0),
        surface2: Color::Rgb(255, 0, 0),
        surface1: Color::Rgb(255, 0, 0),
        surface0: Color::Rgb(255, 0, 0),
        base: Color::Rgb(255, 0, 0),
        mantle: Color::Rgb(255, 0, 0),
        crust: Color::Rgb(255, 0, 0),
    };
    assert_eq!(custom.rosewater, Color::Rgb(255, 0, 0));
    assert_eq!(custom.crust, Color::Rgb(255, 0, 0));
    // Pin Clone + Copy + Debug + PartialEq.
    let cloned = custom;
    assert_eq!(custom, cloned);
}

#[test]
fn test_catppuccin_palette_pinned() {
    // Pin every Catppuccin palette entry to its source constant.
    // Future palette tweaks show up as test diffs.
    let theme = Theme::catppuccin_mocha();
    let p = &theme.palette;
    assert_eq!(p.rosewater, CATPPUCCIN_ROSEWATER);
    assert_eq!(p.flamingo, CATPPUCCIN_FLAMINGO);
    assert_eq!(p.pink, CATPPUCCIN_PINK);
    assert_eq!(p.mauve, CATPPUCCIN_MAUVE);
    assert_eq!(p.red, CATPPUCCIN_RED);
    assert_eq!(p.maroon, CATPPUCCIN_MAROON);
    assert_eq!(p.peach, CATPPUCCIN_PEACH);
    assert_eq!(p.yellow, CATPPUCCIN_YELLOW);
    assert_eq!(p.green, CATPPUCCIN_GREEN);
    assert_eq!(p.teal, CATPPUCCIN_TEAL);
    assert_eq!(p.sky, CATPPUCCIN_SKY);
    assert_eq!(p.sapphire, CATPPUCCIN_SAPPHIRE);
    assert_eq!(p.blue, CATPPUCCIN_BLUE);
    assert_eq!(p.lavender, CATPPUCCIN_LAVENDER);
    assert_eq!(p.text, CATPPUCCIN_TEXT);
    assert_eq!(p.subtext1, CATPPUCCIN_SUBTEXT1);
    assert_eq!(p.subtext0, CATPPUCCIN_SUBTEXT0);
    assert_eq!(p.overlay2, CATPPUCCIN_OVERLAY2);
    assert_eq!(p.overlay1, CATPPUCCIN_OVERLAY1);
    assert_eq!(p.overlay0, CATPPUCCIN_OVERLAY0);
    assert_eq!(p.surface2, CATPPUCCIN_SURFACE2);
    assert_eq!(p.surface1, CATPPUCCIN_SURFACE1);
    assert_eq!(p.surface0, CATPPUCCIN_SURFACE0);
    assert_eq!(p.base, CATPPUCCIN_BASE);
    assert_eq!(p.mantle, CATPPUCCIN_MANTLE);
    assert_eq!(p.crust, CATPPUCCIN_CRUST);
}

#[test]
fn test_nord_palette_pinned() {
    // Pin Nord's nearest-equivalent palette mapping. Documented in Theme::nord().
    let theme = Theme::nord();
    let p = &theme.palette;

    // Nord Aurora warm accents map to closest hue.
    assert_eq!(p.red, NORD11);              // Nord red
    assert_eq!(p.maroon, NORD11);           // No native maroon; reuse red
    assert_eq!(p.peach, NORD12);            // Nord orange
    assert_eq!(p.yellow, NORD13);           // Nord yellow
    assert_eq!(p.green, NORD14);            // Nord green
    assert_eq!(p.teal, NORD7);              // Nord frost teal

    // Nord pinks/rosewaters: no native; map to Snow Storm light tones.
    assert_eq!(p.rosewater, NORD4);
    assert_eq!(p.flamingo, NORD4);

    // Nord purples (only Nord15 exists): pink/mauve/lavender all to Nord15.
    assert_eq!(p.pink, NORD15);
    assert_eq!(p.mauve, NORD15);
    assert_eq!(p.lavender, NORD15);

    // Nord Frost cool blues.
    assert_eq!(p.sky, NORD8);               // light blue
    assert_eq!(p.sapphire, NORD9);          // mid blue
    assert_eq!(p.blue, NORD10);             // deep blue

    // Text/overlay tones from Snow Storm (light → less light).
    assert_eq!(p.text, NORD6);
    assert_eq!(p.subtext1, NORD5);
    assert_eq!(p.subtext0, NORD4);
    assert_eq!(p.overlay2, NORD3);
    assert_eq!(p.overlay1, NORD3);
    assert_eq!(p.overlay0, NORD3);

    // Surface/background tones from Polar Night (light → dark).
    assert_eq!(p.surface2, NORD2);
    assert_eq!(p.surface1, NORD1);
    assert_eq!(p.surface0, NORD1);
    assert_eq!(p.base, NORD0);
    assert_eq!(p.mantle, NORD0);
    assert_eq!(p.crust, NORD0);
}

#[test]
fn test_dracula_palette_pinned() {
    let theme = Theme::dracula();
    let p = &theme.palette;

    // Dracula has named accents: Cyan, Green, Orange, Pink, Purple, Red, Yellow.
    assert_eq!(p.red, DRACULA_RED);
    assert_eq!(p.maroon, DRACULA_RED);          // No maroon; reuse red
    assert_eq!(p.peach, DRACULA_ORANGE);
    assert_eq!(p.yellow, DRACULA_YELLOW);
    assert_eq!(p.green, DRACULA_GREEN);
    assert_eq!(p.teal, DRACULA_CYAN);           // Closest cool teal-ish

    // Pinks/mauves/lavender: native pink and purple available.
    assert_eq!(p.pink, DRACULA_PINK);
    assert_eq!(p.mauve, DRACULA_PURPLE);
    assert_eq!(p.lavender, DRACULA_PURPLE);
    assert_eq!(p.rosewater, DRACULA_PINK);      // Closest pastel pink
    assert_eq!(p.flamingo, DRACULA_PINK);

    // Cool blues: Dracula has only Cyan; map all blues to Cyan.
    assert_eq!(p.sky, DRACULA_CYAN);
    assert_eq!(p.sapphire, DRACULA_CYAN);
    assert_eq!(p.blue, DRACULA_CYAN);

    // Text + overlay tones.
    assert_eq!(p.text, DRACULA_FG);
    assert_eq!(p.subtext1, DRACULA_FG);
    assert_eq!(p.subtext0, DRACULA_COMMENT);
    assert_eq!(p.overlay2, DRACULA_COMMENT);
    assert_eq!(p.overlay1, DRACULA_COMMENT);
    assert_eq!(p.overlay0, DRACULA_COMMENT);

    // Surface / base tones.
    assert_eq!(p.surface2, DRACULA_CURRENT);
    assert_eq!(p.surface1, DRACULA_CURRENT);
    assert_eq!(p.surface0, DRACULA_CURRENT);
    assert_eq!(p.base, DRACULA_BG);
    assert_eq!(p.mantle, DRACULA_BG);
    assert_eq!(p.crust, DRACULA_BG);
}
