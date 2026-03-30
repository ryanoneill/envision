//! Rendering functions for the TabBar component.
//!
//! Extracted from the main tab_bar module to keep file sizes manageable.

use ratatui::prelude::*;
use ratatui::widgets::Paragraph;
use unicode_width::UnicodeWidthStr;

use super::*;

/// Renders the tab bar into the given frame area.
pub(super) fn render_tab_bar(state: &TabBarState, frame: &mut Frame, area: Rect, theme: &Theme) {
    if area.height == 0 || area.width == 0 {
        return;
    }

    let available_width = area.width as usize;

    // Build rendered spans for each tab, computing widths.
    struct RenderedTab {
        spans: Vec<Span<'static>>,
        width: usize,
    }

    let rendered: Vec<RenderedTab> = state
        .tabs
        .iter()
        .enumerate()
        .map(|(i, tab)| {
            let is_active = state.active == Some(i);
            let base_style = if state.disabled {
                theme.disabled_style()
            } else if is_active {
                theme.focused_style().add_modifier(Modifier::BOLD)
            } else {
                theme.normal_style()
            };

            let mut parts: Vec<Span<'static>> = Vec::new();
            parts.push(Span::styled(" ", base_style));

            if let Some(icon) = &tab.icon {
                parts.push(Span::styled(format!("{icon} "), base_style));
            }

            // Label (possibly truncated)
            let label_text = if let Some(max) = state.max_tab_width {
                let decoration_width = tab.rendered_width(None) - tab.label.width();
                let max_label = max.saturating_sub(decoration_width);
                truncate_label(&tab.label, max_label)
            } else {
                tab.label.clone()
            };
            parts.push(Span::styled(label_text, base_style));

            if tab.modified {
                let mod_style = if state.disabled {
                    theme.disabled_style()
                } else {
                    theme.warning_style()
                };
                parts.push(Span::styled("*", mod_style));
            }

            if tab.closable {
                let close_style = if state.disabled {
                    theme.disabled_style()
                } else {
                    theme.error_style()
                };
                parts.push(Span::styled(" x", close_style));
            }

            parts.push(Span::styled(" ", base_style));

            let width = tab.rendered_width(state.max_tab_width);
            RenderedTab {
                spans: parts,
                width,
            }
        })
        .collect();

    // Determine which tabs are visible starting from scroll_offset.
    let has_left_overflow = state.scroll_offset > 0;
    let indicator_width: usize = 2; // "< " or " >"

    let usable_left = if has_left_overflow {
        available_width.saturating_sub(indicator_width)
    } else {
        available_width
    };

    // Walk from scroll_offset to find how many tabs fit.
    let mut used = 0usize;
    let mut visible_end = state.scroll_offset;
    for rt in rendered.iter().skip(state.scroll_offset) {
        let needed = used + rt.width;
        if needed > usable_left {
            break;
        }
        used = needed;
        visible_end += 1;
    }

    // Check right overflow; if there are more tabs, reserve indicator space.
    let has_right_overflow = visible_end < rendered.len();
    if has_right_overflow && visible_end > state.scroll_offset {
        // Re-check if the last visible tab still fits with the indicator.
        let total_with_indicator = used + indicator_width;
        if total_with_indicator > available_width {
            // Drop the last visible tab.
            visible_end -= 1;
        }
    }

    // Build the final Line.
    let mut spans: Vec<Span<'static>> = Vec::new();

    if has_left_overflow {
        let indicator_style = if state.disabled {
            theme.disabled_style()
        } else {
            theme.info_style()
        };
        spans.push(Span::styled("< ", indicator_style));
    }

    for rt in rendered
        .iter()
        .skip(state.scroll_offset)
        .take(visible_end.saturating_sub(state.scroll_offset))
    {
        spans.extend(rt.spans.iter().cloned());
    }

    if has_right_overflow {
        let indicator_style = if state.disabled {
            theme.disabled_style()
        } else {
            theme.info_style()
        };
        spans.push(Span::styled(" >", indicator_style));
    }

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line);

    let annotation = crate::annotation::Annotation::new(crate::annotation::WidgetType::TabBar)
        .with_id("tab_bar")
        .with_focus(state.focused)
        .with_disabled(state.disabled)
        .with_selected(state.active.is_some())
        .with_value(state.active.map(|i| i.to_string()).unwrap_or_default());
    let annotated = crate::annotation::Annotate::new(paragraph, annotation);
    frame.render_widget(annotated, area);
}

/// Truncates a label to at most `max_width` display columns, appending
/// an ellipsis if truncation occurs.
pub(super) fn truncate_label(label: &str, max_width: usize) -> String {
    if label.width() <= max_width {
        return label.to_string();
    }
    if max_width == 0 {
        return String::new();
    }
    let mut result = String::new();
    let mut w = 0;
    let target = max_width.saturating_sub(1); // reserve 1 for ellipsis
    for ch in label.chars() {
        let cw = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
        if w + cw > target {
            break;
        }
        result.push(ch);
        w += cw;
    }
    result.push('\u{2026}'); // ellipsis
    result
}
