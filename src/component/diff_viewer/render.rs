//! Rendering logic for the DiffViewer component.
//!
//! Provides unified and side-by-side rendering modes for diff display.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use super::{DiffLineType, DiffMode, DiffViewerState};
use crate::scroll::ScrollState;
use crate::theme::Theme;

/// Renders the DiffViewer in the given area.
pub(super) fn render(state: &DiffViewerState, frame: &mut Frame, area: Rect, theme: &Theme) {
    crate::annotation::with_registry(|reg| {
        reg.register(
            area,
            crate::annotation::Annotation::diff_viewer("diff_viewer")
                .with_focus(state.focused)
                .with_disabled(state.disabled),
        );
    });

    let border_style = if state.disabled {
        theme.disabled_style()
    } else if state.focused {
        theme.focused_border_style()
    } else {
        theme.border_style()
    };

    let title = build_title(state);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(title);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    match state.mode {
        DiffMode::Unified => render_unified(state, frame, inner, theme),
        DiffMode::SideBySide => render_side_by_side(state, frame, inner, theme),
    }
}

/// Builds the title string for the diff viewer border.
fn build_title(state: &DiffViewerState) -> String {
    let added = state.added_count();
    let removed = state.removed_count();

    if let Some(ref title) = state.title {
        if added > 0 || removed > 0 {
            format!(" {} (+{}, -{}) ", title, added, removed)
        } else {
            format!(" {} ", title)
        }
    } else if added > 0 || removed > 0 {
        format!(" Diff (+{}, -{}) ", added, removed)
    } else {
        " Diff ".to_string()
    }
}

/// Returns the style for a hunk header line (blue + bold).
fn header_style(theme: &Theme) -> Style {
    theme.info_style().add_modifier(Modifier::BOLD)
}

/// Returns the style for an added line.
fn added_style(state: &DiffViewerState, theme: &Theme) -> Style {
    if state.disabled {
        theme.disabled_style()
    } else {
        Style::default().fg(Color::Green).bg(Color::Rgb(0, 40, 0))
    }
}

/// Returns the style for a removed line.
fn removed_style(state: &DiffViewerState, theme: &Theme) -> Style {
    if state.disabled {
        theme.disabled_style()
    } else {
        Style::default().fg(Color::Red).bg(Color::Rgb(40, 0, 0))
    }
}

/// Returns the style for a context line.
fn context_style(state: &DiffViewerState, theme: &Theme) -> Style {
    if state.disabled {
        theme.disabled_style()
    } else {
        theme.normal_style()
    }
}

/// Renders the diff in unified mode.
fn render_unified(state: &DiffViewerState, frame: &mut Frame, area: Rect, theme: &Theme) {
    let all_lines = state.collect_display_lines();
    let total = all_lines.len();
    let visible = area.height as usize;

    let scroll_offset = state.scroll.offset().min(total.saturating_sub(visible));
    let end = (scroll_offset + visible).min(total);

    for (row_idx, line_idx) in (scroll_offset..end).enumerate() {
        let y = area.y + row_idx as u16;
        if y >= area.y + area.height {
            break;
        }

        let display_line = &all_lines[line_idx];
        let line_area = Rect::new(area.x, y, area.width, 1);

        let (text, style) = match display_line.line_type {
            DiffLineType::Header => (display_line.content.clone(), header_style(theme)),
            DiffLineType::Added => {
                let prefix = build_unified_prefix(display_line, state.show_line_numbers, '+');
                let text = format!("{}{}", prefix, display_line.content);
                (text, added_style(state, theme))
            }
            DiffLineType::Removed => {
                let prefix = build_unified_prefix(display_line, state.show_line_numbers, '-');
                let text = format!("{}{}", prefix, display_line.content);
                (text, removed_style(state, theme))
            }
            DiffLineType::Context => {
                let prefix = build_unified_prefix(display_line, state.show_line_numbers, ' ');
                let text = format!("{}{}", prefix, display_line.content);
                (text, context_style(state, theme))
            }
        };

        let paragraph = Paragraph::new(text).style(style);
        frame.render_widget(paragraph, line_area);
    }

    // Render scrollbar
    if total > visible {
        let mut bar_scroll = ScrollState::new(total);
        bar_scroll.set_viewport_height(visible);
        bar_scroll.set_offset(scroll_offset);
        render_scrollbar_in_area(&bar_scroll, frame, area, theme);
    }
}

/// Builds the prefix string for a unified-mode line (line numbers + sigil).
fn build_unified_prefix(line: &super::DiffLine, show_line_numbers: bool, sigil: char) -> String {
    if show_line_numbers {
        let old_num = line
            .old_line_num
            .map(|n| format!("{:>4}", n))
            .unwrap_or_else(|| "    ".to_string());
        let new_num = line
            .new_line_num
            .map(|n| format!("{:>4}", n))
            .unwrap_or_else(|| "    ".to_string());
        format!("{} {} {}", old_num, new_num, sigil)
    } else {
        format!("{}", sigil)
    }
}

/// Renders the diff in side-by-side mode.
fn render_side_by_side(state: &DiffViewerState, frame: &mut Frame, area: Rect, theme: &Theme) {
    let pairs = state.collect_side_by_side_pairs();
    let total = pairs.len();
    let visible = area.height as usize;

    let scroll_offset = state.scroll.offset().min(total.saturating_sub(visible));
    let end = (scroll_offset + visible).min(total);

    // Split area in half
    let half_width = area.width / 2;
    let right_x = area.x + half_width;
    let right_width = area.width.saturating_sub(half_width);

    // Render header labels on top row if we have labels
    let content_start_row = if state.old_label.is_some() || state.new_label.is_some() {
        if area.height > 1 {
            let left_label = state.old_label.as_deref().unwrap_or("Old");
            let right_label = state.new_label.as_deref().unwrap_or("New");

            let hdr_style = header_style(theme);

            let left_header = Paragraph::new(format!(" {}", left_label)).style(hdr_style);
            let right_header = Paragraph::new(format!(" {}", right_label)).style(hdr_style);

            frame.render_widget(left_header, Rect::new(area.x, area.y, half_width, 1));
            frame.render_widget(right_header, Rect::new(right_x, area.y, right_width, 1));
            1
        } else {
            0
        }
    } else {
        0
    };

    for (row_idx, pair_idx) in (scroll_offset..end).enumerate() {
        let y = area.y + content_start_row as u16 + row_idx as u16;
        if y >= area.y + area.height {
            break;
        }

        let (ref left_line, ref right_line) = pairs[pair_idx];

        let left_rect = Rect::new(area.x, y, half_width, 1);
        let right_rect = Rect::new(right_x, y, right_width, 1);

        // Left side (prefer old line numbers)
        render_side_line(left_line, frame, left_rect, state, theme, false);
        // Right side (prefer new line numbers)
        render_side_line(right_line, frame, right_rect, state, theme, true);
    }

    // Render scrollbar
    if total > visible {
        let mut bar_scroll = ScrollState::new(total);
        bar_scroll.set_viewport_height(visible);
        bar_scroll.set_offset(scroll_offset);
        render_scrollbar_in_area(&bar_scroll, frame, area, theme);
    }
}

/// Renders a single line on one side of the side-by-side view.
///
/// When `prefer_new` is true, the new-file line number is shown (right side).
/// When false, the old-file line number is shown (left side).
fn render_side_line(
    line: &Option<super::DiffLine>,
    frame: &mut Frame,
    line_area: Rect,
    state: &DiffViewerState,
    theme: &Theme,
    prefer_new: bool,
) {
    if let Some(ref diff_line) = line {
        let style = match diff_line.line_type {
            DiffLineType::Header => header_style(theme),
            DiffLineType::Added => added_style(state, theme),
            DiffLineType::Removed => removed_style(state, theme),
            DiffLineType::Context => context_style(state, theme),
        };

        let line_num = if prefer_new {
            diff_line.new_line_num.or(diff_line.old_line_num)
        } else {
            diff_line.old_line_num.or(diff_line.new_line_num)
        };

        let text = if state.show_line_numbers {
            if let Some(num) = line_num {
                format!("{:>4} {}", num, diff_line.content)
            } else {
                format!("     {}", diff_line.content)
            }
        } else {
            diff_line.content.clone()
        };

        let paragraph = Paragraph::new(text).style(style);
        frame.render_widget(paragraph, line_area);
    } else {
        // Empty line (padding for alignment)
        let style = context_style(state, theme);
        let paragraph = Paragraph::new("").style(style);
        frame.render_widget(paragraph, line_area);
    }
}

/// Renders a vertical scrollbar within the given content area.
fn render_scrollbar_in_area(scroll: &ScrollState, frame: &mut Frame, area: Rect, theme: &Theme) {
    use ratatui::widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState};

    if !scroll.can_scroll() {
        return;
    }

    let mut scrollbar_state = ScrollbarState::default()
        .content_length(
            scroll
                .content_length()
                .saturating_sub(scroll.viewport_height()),
        )
        .viewport_content_length(scroll.viewport_height())
        .position(scroll.offset());

    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .thumb_style(theme.normal_style())
        .track_style(theme.disabled_style());

    frame.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
}
