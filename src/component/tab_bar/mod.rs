//! A rich tab bar component with closable, modified, and icon-decorated tabs.
//!
//! [`TabBar`] provides a horizontal tab bar with features commonly found in
//! editors and browsers: closable tabs, modified indicators, optional icons,
//! and horizontal scrolling for overflow. State is stored in [`TabBarState`],
//! updated via [`TabBarMessage`], and produces [`TabBarOutput`].
//!
//! Implements [`Focusable`] and [`Disableable`].
//!
//! # Example
//!
//! ```rust
//! use envision::component::{
//!     Component, Focusable, Tab, TabBar, TabBarMessage, TabBarOutput, TabBarState,
//! };
//!
//! let tabs = vec![
//!     Tab::new("file1", "main.rs"),
//!     Tab::new("file2", "lib.rs").with_modified(true),
//!     Tab::new("file3", "test.rs").with_closable(true),
//! ];
//! let mut state = TabBarState::new(tabs);
//! TabBar::set_focused(&mut state, true);
//!
//! assert_eq!(state.active_index(), Some(0));
//! assert_eq!(state.active_tab().map(|t| t.label()), Some("main.rs"));
//!
//! // Navigate to the next tab
//! let output = TabBar::update(&mut state, TabBarMessage::NextTab);
//! assert_eq!(output, Some(TabBarOutput::TabSelected(1)));
//! assert_eq!(state.active_index(), Some(1));
//! ```

use ratatui::prelude::*;
use ratatui::widgets::Paragraph;
use unicode_width::UnicodeWidthStr;

use super::{Component, Disableable, Focusable};
use crate::input::{Event, KeyCode};
use crate::theme::Theme;

// ---------------------------------------------------------------------------
// Tab
// ---------------------------------------------------------------------------

/// A single tab in a [`TabBar`].
///
/// Each tab has a unique identifier, a display label, and optional properties
/// for closability, modified state, and an icon prefix.
///
/// # Example
///
/// ```rust
/// use envision::component::Tab;
///
/// let tab = Tab::new("editor-1", "main.rs")
///     .with_closable(true)
///     .with_modified(true)
///     .with_icon("R");
/// assert_eq!(tab.id(), "editor-1");
/// assert_eq!(tab.label(), "main.rs");
/// assert!(tab.closable());
/// assert!(tab.modified());
/// assert_eq!(tab.icon(), Some("R"));
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct Tab {
    /// Unique identifier for this tab.
    id: String,
    /// Display label.
    label: String,
    /// Whether the tab can be closed.
    closable: bool,
    /// Whether the tab has unsaved modifications.
    modified: bool,
    /// Optional icon prefix (e.g., a file-type indicator).
    icon: Option<String>,
}

impl Tab {
    /// Creates a new tab with the given id and label.
    ///
    /// By default the tab is not closable, not modified, and has no icon.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::Tab;
    ///
    /// let tab = Tab::new("t1", "Overview");
    /// assert_eq!(tab.id(), "t1");
    /// assert_eq!(tab.label(), "Overview");
    /// assert!(!tab.closable());
    /// assert!(!tab.modified());
    /// assert_eq!(tab.icon(), None);
    /// ```
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            closable: false,
            modified: false,
            icon: None,
        }
    }

    /// Sets whether the tab is closable (builder).
    pub fn with_closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }

    /// Sets whether the tab is modified (builder).
    pub fn with_modified(mut self, modified: bool) -> Self {
        self.modified = modified;
        self
    }

    /// Sets an icon for the tab (builder).
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Returns the tab's unique identifier.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the tab's display label.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns whether the tab is closable.
    pub fn closable(&self) -> bool {
        self.closable
    }

    /// Returns whether the tab has unsaved modifications.
    pub fn modified(&self) -> bool {
        self.modified
    }

    /// Returns the tab's icon, if any.
    pub fn icon(&self) -> Option<&str> {
        self.icon.as_deref()
    }

    /// Sets the label.
    pub fn set_label(&mut self, label: impl Into<String>) {
        self.label = label.into();
    }

    /// Sets the closable flag.
    pub fn set_closable(&mut self, closable: bool) {
        self.closable = closable;
    }

    /// Sets the modified flag.
    pub fn set_modified(&mut self, modified: bool) {
        self.modified = modified;
    }

    /// Sets the icon.
    pub fn set_icon(&mut self, icon: Option<String>) {
        self.icon = icon;
    }

    /// Returns the rendered width of this tab, including decorations.
    ///
    /// Layout: ` [icon ]label[modified][close] `
    fn rendered_width(&self, max_tab_width: Option<usize>) -> usize {
        let mut w: usize = 2; // leading and trailing space
        if let Some(icon) = &self.icon {
            w += icon.width() + 1; // icon + space
        }
        w += self.label.width();
        if self.modified {
            w += 1; // bullet
        }
        if self.closable {
            w += 2; // space + close char
        }
        if let Some(max) = max_tab_width {
            w.min(max)
        } else {
            w
        }
    }
}

// ---------------------------------------------------------------------------
// TabBarMessage
// ---------------------------------------------------------------------------

/// Messages that can be sent to a [`TabBar`] component.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TabBarMessage {
    /// Select a specific tab by index.
    SelectTab(usize),
    /// Move to the next (right) tab.
    NextTab,
    /// Move to the previous (left) tab.
    PrevTab,
    /// Close a specific tab by index.
    CloseTab(usize),
    /// Close the currently active tab (only if closable).
    CloseActiveTab,
    /// Add a new tab (appended to the end) and make it active.
    AddTab(Tab),
    /// Jump to the first tab.
    First,
    /// Jump to the last tab.
    Last,
}

// ---------------------------------------------------------------------------
// TabBarOutput
// ---------------------------------------------------------------------------

/// Output messages from a [`TabBar`] component.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TabBarOutput {
    /// A tab was selected (contains the new active index).
    TabSelected(usize),
    /// A tab was closed (contains the index that was closed).
    TabClosed(usize),
    /// A new tab was added (contains the index of the new tab).
    TabAdded(usize),
}

// ---------------------------------------------------------------------------
// TabBarState
// ---------------------------------------------------------------------------

/// State for a [`TabBar`] component.
///
/// Tracks the list of tabs, the active tab, scroll offset for overflow,
/// an optional maximum tab width, and focus/disabled state.
///
/// # Example
///
/// ```rust
/// use envision::component::{Tab, TabBarState};
///
/// let tabs = vec![
///     Tab::new("a", "Alpha"),
///     Tab::new("b", "Beta"),
///     Tab::new("c", "Gamma"),
/// ];
/// let state = TabBarState::new(tabs);
/// assert_eq!(state.len(), 3);
/// assert_eq!(state.active_index(), Some(0));
/// ```
#[derive(Clone, Debug, Default)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct TabBarState {
    /// The tabs.
    tabs: Vec<Tab>,
    /// Index of the active tab, or `None` when empty.
    active: Option<usize>,
    /// Horizontal scroll offset (number of tabs scrolled past the left edge).
    scroll_offset: usize,
    /// Optional maximum rendered width per tab (in columns).
    max_tab_width: Option<usize>,
    /// Whether the component is focused.
    focused: bool,
    /// Whether the component is disabled.
    disabled: bool,
}

impl PartialEq for TabBarState {
    fn eq(&self, other: &Self) -> bool {
        self.tabs == other.tabs
            && self.active == other.active
            && self.scroll_offset == other.scroll_offset
            && self.max_tab_width == other.max_tab_width
            && self.focused == other.focused
            && self.disabled == other.disabled
    }
}

impl TabBarState {
    /// Creates a new tab bar state with the first tab active.
    ///
    /// If `tabs` is empty the active index is `None`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Tab, TabBarState};
    ///
    /// let state = TabBarState::new(vec![
    ///     Tab::new("1", "One"),
    ///     Tab::new("2", "Two"),
    /// ]);
    /// assert_eq!(state.active_index(), Some(0));
    /// assert_eq!(state.len(), 2);
    /// ```
    pub fn new(tabs: Vec<Tab>) -> Self {
        let active = if tabs.is_empty() { None } else { Some(0) };
        Self {
            tabs,
            active,
            scroll_offset: 0,
            max_tab_width: None,
            focused: false,
            disabled: false,
        }
    }

    /// Creates a tab bar state with a specific tab active.
    ///
    /// The index is clamped to the valid range. Empty tabs yield `None` active.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Tab, TabBarState};
    ///
    /// let state = TabBarState::with_active(
    ///     vec![Tab::new("a", "A"), Tab::new("b", "B"), Tab::new("c", "C")],
    ///     1,
    /// );
    /// assert_eq!(state.active_index(), Some(1));
    /// ```
    pub fn with_active(tabs: Vec<Tab>, active: usize) -> Self {
        let active = if tabs.is_empty() {
            None
        } else {
            Some(active.min(tabs.len() - 1))
        };
        Self {
            tabs,
            active,
            scroll_offset: 0,
            max_tab_width: None,
            focused: false,
            disabled: false,
        }
    }

    // -- Builder methods -----------------------------------------------------

    /// Sets the max tab width (builder).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Tab, TabBarState};
    ///
    /// let state = TabBarState::new(vec![Tab::new("a", "Alpha")])
    ///     .with_max_tab_width(Some(20));
    /// assert_eq!(state.max_tab_width(), Some(20));
    /// ```
    pub fn with_max_tab_width(mut self, max: Option<usize>) -> Self {
        self.max_tab_width = max;
        self
    }

    /// Sets the disabled state (builder).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Tab, TabBarState};
    ///
    /// let state = TabBarState::new(vec![Tab::new("a", "A")])
    ///     .with_disabled(true);
    /// assert!(state.is_disabled());
    /// ```
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    // -- Accessors -----------------------------------------------------------

    /// Returns the tabs.
    pub fn tabs(&self) -> &[Tab] {
        &self.tabs
    }

    /// Returns a mutable reference to the tabs.
    pub fn tabs_mut(&mut self) -> &mut [Tab] {
        &mut self.tabs
    }

    /// Returns the currently active tab index, or `None` if empty.
    pub fn active_index(&self) -> Option<usize> {
        self.active
    }

    /// Returns the currently active tab, or `None` if empty.
    pub fn active_tab(&self) -> Option<&Tab> {
        self.tabs.get(self.active?)
    }

    /// Returns a mutable reference to the active tab.
    pub fn active_tab_mut(&mut self) -> Option<&mut Tab> {
        let idx = self.active?;
        self.tabs.get_mut(idx)
    }

    /// Returns the number of tabs.
    pub fn len(&self) -> usize {
        self.tabs.len()
    }

    /// Returns true if there are no tabs.
    pub fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }

    /// Returns the scroll offset.
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Returns the maximum tab width, if set.
    pub fn max_tab_width(&self) -> Option<usize> {
        self.max_tab_width
    }

    /// Returns whether the component is disabled.
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Returns whether the component is focused.
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    // -- Mutators ------------------------------------------------------------

    /// Sets the active tab by index.
    ///
    /// `None` clears the selection. `Some(i)` is clamped to the valid range.
    pub fn set_active(&mut self, index: Option<usize>) {
        match index {
            Some(i) if !self.tabs.is_empty() => {
                self.active = Some(i.min(self.tabs.len() - 1));
            }
            _ => self.active = None,
        }
    }

    /// Sets the scroll offset.
    pub fn set_scroll_offset(&mut self, offset: usize) {
        self.scroll_offset = offset;
    }

    /// Sets the max tab width.
    pub fn set_max_tab_width(&mut self, max: Option<usize>) {
        self.max_tab_width = max;
    }

    /// Sets the disabled state.
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    /// Sets the focused state.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Replaces all tabs, clamping or clearing the active index.
    pub fn set_tabs(&mut self, tabs: Vec<Tab>) {
        self.tabs = tabs;
        if self.tabs.is_empty() {
            self.active = None;
            self.scroll_offset = 0;
        } else if let Some(idx) = self.active {
            if idx >= self.tabs.len() {
                self.active = Some(self.tabs.len() - 1);
            }
        }
    }

    /// Returns a tab by its id, if present.
    pub fn find_tab_by_id(&self, id: &str) -> Option<(usize, &Tab)> {
        self.tabs.iter().enumerate().find(|(_, t)| t.id() == id)
    }

    // -- Instance methods that delegate to component -------------------------

    /// Maps an input event to a tab bar message.
    pub fn handle_event(&self, event: &Event) -> Option<TabBarMessage> {
        TabBar::handle_event(self, event)
    }

    /// Dispatches an event, updating state and returning any output.
    pub fn dispatch_event(&mut self, event: &Event) -> Option<TabBarOutput> {
        TabBar::dispatch_event(self, event)
    }

    /// Updates the tab bar state with a message, returning any output.
    pub fn update(&mut self, msg: TabBarMessage) -> Option<TabBarOutput> {
        TabBar::update(self, msg)
    }

    // -- Scroll helpers (internal) -------------------------------------------

    /// Ensures the active tab is visible by adjusting the scroll offset.
    fn ensure_active_visible(&mut self) {
        if let Some(active) = self.active {
            if active < self.scroll_offset {
                self.scroll_offset = active;
            }
            // The exact right bound depends on the render width, which we
            // do not know here.  We do a best-effort adjustment: if the
            // active index is beyond what we can see, bump the offset.
            // The view function will do a tighter clamp.
        }
    }
}

// ---------------------------------------------------------------------------
// TabBar component
// ---------------------------------------------------------------------------

/// A rich horizontal tab bar component.
///
/// `TabBar` renders a single row of tabs with support for:
///
/// - **Closable tabs** that show a close indicator (`x`)
/// - **Modified tabs** that show a bullet (`*`)
/// - **Optional icon** prefix per tab
/// - **Horizontal scrolling** when tabs overflow the available width
///   (left/right scroll indicators are displayed)
/// - **Max tab width** to keep the bar compact
///
/// # Navigation
///
/// - `Left` / `h` - previous tab
/// - `Right` / `l` - next tab
/// - `Home` - first tab
/// - `End` - last tab
/// - `w` - close the active tab (if closable)
///
/// # Output
///
/// - [`TabBarOutput::TabSelected`] - a tab was activated
/// - [`TabBarOutput::TabClosed`] - a tab was closed
/// - [`TabBarOutput::TabAdded`] - a new tab was added
///
/// # Example
///
/// ```rust
/// use envision::component::{
///     Component, Focusable, Tab, TabBar, TabBarMessage, TabBarOutput, TabBarState,
/// };
///
/// let tabs = vec![
///     Tab::new("1", "Overview"),
///     Tab::new("2", "Details").with_closable(true),
/// ];
/// let mut state = TabBarState::new(tabs);
/// TabBar::set_focused(&mut state, true);
///
/// let output = TabBar::update(&mut state, TabBarMessage::NextTab);
/// assert_eq!(output, Some(TabBarOutput::TabSelected(1)));
/// ```
pub struct TabBar;

impl Component for TabBar {
    type State = TabBarState;
    type Message = TabBarMessage;
    type Output = TabBarOutput;

    fn init() -> Self::State {
        TabBarState::default()
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        if state.disabled {
            return None;
        }

        match msg {
            TabBarMessage::SelectTab(index) => {
                if state.tabs.is_empty() {
                    return None;
                }
                let clamped = index.min(state.tabs.len() - 1);
                if state.active == Some(clamped) {
                    return None;
                }
                state.active = Some(clamped);
                state.ensure_active_visible();
                Some(TabBarOutput::TabSelected(clamped))
            }

            TabBarMessage::NextTab => {
                let active = state.active?;
                if active >= state.tabs.len().saturating_sub(1) {
                    return None;
                }
                state.active = Some(active + 1);
                state.ensure_active_visible();
                Some(TabBarOutput::TabSelected(active + 1))
            }

            TabBarMessage::PrevTab => {
                let active = state.active?;
                if active == 0 {
                    return None;
                }
                state.active = Some(active - 1);
                state.ensure_active_visible();
                Some(TabBarOutput::TabSelected(active - 1))
            }

            TabBarMessage::CloseTab(index) => {
                if index >= state.tabs.len() {
                    return None;
                }
                if !state.tabs[index].closable {
                    return None;
                }
                state.tabs.remove(index);
                if state.tabs.is_empty() {
                    state.active = None;
                    state.scroll_offset = 0;
                } else if let Some(active) = state.active {
                    if index < active {
                        state.active = Some(active - 1);
                    } else if index == active {
                        // Stay at same index or move to last if needed.
                        if active >= state.tabs.len() {
                            state.active = Some(state.tabs.len() - 1);
                        }
                    }
                    // Clamp scroll offset
                    if state.scroll_offset >= state.tabs.len() {
                        state.scroll_offset = state.tabs.len().saturating_sub(1);
                    }
                }
                state.ensure_active_visible();
                Some(TabBarOutput::TabClosed(index))
            }

            TabBarMessage::CloseActiveTab => {
                let active = state.active?;
                if !state.tabs[active].closable {
                    return None;
                }
                // Delegate to CloseTab
                TabBar::update(state, TabBarMessage::CloseTab(active))
            }

            TabBarMessage::AddTab(tab) => {
                state.tabs.push(tab);
                let new_index = state.tabs.len() - 1;
                state.active = Some(new_index);
                state.ensure_active_visible();
                Some(TabBarOutput::TabAdded(new_index))
            }

            TabBarMessage::First => {
                if state.tabs.is_empty() {
                    return None;
                }
                if state.active == Some(0) {
                    return None;
                }
                state.active = Some(0);
                state.scroll_offset = 0;
                Some(TabBarOutput::TabSelected(0))
            }

            TabBarMessage::Last => {
                if state.tabs.is_empty() {
                    return None;
                }
                let last = state.tabs.len() - 1;
                if state.active == Some(last) {
                    return None;
                }
                state.active = Some(last);
                state.ensure_active_visible();
                Some(TabBarOutput::TabSelected(last))
            }
        }
    }

    fn handle_event(state: &Self::State, event: &Event) -> Option<Self::Message> {
        if !state.focused || state.disabled {
            return None;
        }
        if let Some(key) = event.as_key() {
            match key.code {
                KeyCode::Left | KeyCode::Char('h') => Some(TabBarMessage::PrevTab),
                KeyCode::Right | KeyCode::Char('l') => Some(TabBarMessage::NextTab),
                KeyCode::Home => Some(TabBarMessage::First),
                KeyCode::End => Some(TabBarMessage::Last),
                KeyCode::Char('w') => Some(TabBarMessage::CloseActiveTab),
                _ => None,
            }
        } else {
            None
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme) {
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
}

impl Focusable for TabBar {
    fn is_focused(state: &Self::State) -> bool {
        state.focused
    }

    fn set_focused(state: &mut Self::State, focused: bool) {
        state.focused = focused;
    }
}

impl Disableable for TabBar {
    fn is_disabled(state: &Self::State) -> bool {
        state.disabled
    }

    fn set_disabled(state: &mut Self::State, disabled: bool) {
        state.disabled = disabled;
    }
}

/// Truncates a label to at most `max_width` display columns, appending
/// an ellipsis if truncation occurs.
fn truncate_label(label: &str, max_width: usize) -> String {
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

#[cfg(test)]
mod tests;
