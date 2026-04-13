//! A rich tab bar component with closable, modified, and icon-decorated tabs.
//!
//! [`TabBar`] provides a horizontal tab bar with features commonly found in
//! editors and browsers: closable tabs, modified indicators, optional icons,
//! and horizontal scrolling for overflow. State is stored in [`TabBarState`],
//! updated via [`TabBarMessage`], and produces [`TabBarOutput`].
//!
//! See also [`Tabs`](super::Tabs) for a simpler tab component without
//! closable tabs or overflow scrolling.
//!
//! # Example
//!
//! ```rust
//! use envision::component::{
//!     Component, Tab, TabBar, TabBarMessage, TabBarOutput, TabBarState,
//! };
//!
//! let tabs = vec![
//!     Tab::new("file1", "main.rs"),
//!     Tab::new("file2", "lib.rs").with_modified(true),
//!     Tab::new("file3", "test.rs").with_closable(true),
//! ];
//! let mut state = TabBarState::new(tabs);
//!
//! assert_eq!(state.active_index(), Some(0));
//! assert_eq!(state.active_tab().map(|t| t.label()), Some("main.rs"));
//!
//! // Navigate to the next tab
//! let output = TabBar::update(&mut state, TabBarMessage::NextTab);
//! assert_eq!(output, Some(TabBarOutput::TabSelected(1)));
//! assert_eq!(state.active_index(), Some(1));
//! ```

use super::{Component, EventContext, RenderContext};
use crate::input::{Event, Key};
use crate::theme::Theme;

mod render;
#[cfg(test)]
use render::truncate_label;

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
    pub(super) id: String,
    /// Display label.
    pub(super) label: String,
    /// Whether the tab can be closed.
    pub(super) closable: bool,
    /// Whether the tab has unsaved modifications.
    pub(super) modified: bool,
    /// Optional icon prefix (e.g., a file-type indicator).
    pub(super) icon: Option<String>,
}

// Tab methods are in tab.rs
mod tab;

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
}

impl PartialEq for TabBarState {
    fn eq(&self, other: &Self) -> bool {
        self.tabs == other.tabs
            && self.active == other.active
            && self.scroll_offset == other.scroll_offset
            && self.max_tab_width == other.max_tab_width
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

    // -- Accessors -----------------------------------------------------------

    /// Returns the tabs.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Tab, TabBarState};
    ///
    /// let state = TabBarState::new(vec![
    ///     Tab::new("a", "Alpha"),
    ///     Tab::new("b", "Beta"),
    /// ]);
    /// assert_eq!(state.tabs().len(), 2);
    /// assert_eq!(state.tabs()[0].label(), "Alpha");
    /// ```
    pub fn tabs(&self) -> &[Tab] {
        &self.tabs
    }

    /// Returns a mutable reference to the tabs.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Tab, TabBarState};
    ///
    /// let mut state = TabBarState::new(vec![Tab::new("a", "Alpha")]);
    /// state.tabs_mut()[0].set_label("Renamed");
    /// assert_eq!(state.tabs()[0].label(), "Renamed");
    /// ```
    pub fn tabs_mut(&mut self) -> &mut [Tab] {
        &mut self.tabs
    }

    /// Returns the currently active tab index, or `None` if empty.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Tab, TabBarState};
    ///
    /// let state = TabBarState::new(vec![Tab::new("a", "A"), Tab::new("b", "B")]);
    /// assert_eq!(state.active_index(), Some(0));
    ///
    /// let empty = TabBarState::new(vec![]);
    /// assert_eq!(empty.active_index(), None);
    /// ```
    pub fn active_index(&self) -> Option<usize> {
        self.active
    }

    /// Returns the active tab index, or `None` if the tab bar is empty.
    ///
    /// This is the getter counterpart to [`set_active`](Self::set_active).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Tab, TabBarState};
    ///
    /// let mut state = TabBarState::new(vec![
    ///     Tab::new("a", "A"),
    ///     Tab::new("b", "B"),
    /// ]);
    /// assert_eq!(state.active(), Some(0));
    ///
    /// state.set_active(Some(1));
    /// assert_eq!(state.active(), Some(1));
    ///
    /// state.set_active(None);
    /// assert_eq!(state.active(), None);
    /// ```
    pub fn active(&self) -> Option<usize> {
        self.active
    }

    /// Returns the currently active tab, or `None` if empty.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Tab, TabBarState};
    ///
    /// let state = TabBarState::new(vec![Tab::new("a", "Alpha")]);
    /// assert_eq!(state.active_tab().unwrap().label(), "Alpha");
    /// ```
    pub fn active_tab(&self) -> Option<&Tab> {
        self.tabs.get(self.active?)
    }

    /// Returns a mutable reference to the active tab.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Tab, TabBarState};
    ///
    /// let mut state = TabBarState::new(vec![Tab::new("a", "Alpha")]);
    /// state.active_tab_mut().unwrap().set_modified(true);
    /// assert!(state.active_tab().unwrap().modified());
    /// ```
    pub fn active_tab_mut(&mut self) -> Option<&mut Tab> {
        let idx = self.active?;
        self.tabs.get_mut(idx)
    }

    /// Returns the number of tabs.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Tab, TabBarState};
    ///
    /// let state = TabBarState::new(vec![Tab::new("a", "A"), Tab::new("b", "B")]);
    /// assert_eq!(state.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.tabs.len()
    }

    /// Returns true if there are no tabs.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TabBarState;
    ///
    /// let state = TabBarState::new(vec![]);
    /// assert!(state.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }

    /// Returns the scroll offset.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Tab, TabBarState};
    ///
    /// let state = TabBarState::new(vec![Tab::new("a", "A")]);
    /// assert_eq!(state.scroll_offset(), 0);
    /// ```
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Returns the maximum tab width, if set.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Tab, TabBarState};
    ///
    /// let state = TabBarState::new(vec![Tab::new("a", "A")]);
    /// assert_eq!(state.max_tab_width(), None);
    ///
    /// let state = state.with_max_tab_width(Some(20));
    /// assert_eq!(state.max_tab_width(), Some(20));
    /// ```
    pub fn max_tab_width(&self) -> Option<usize> {
        self.max_tab_width
    }

    // -- Mutators ------------------------------------------------------------

    /// Sets the active tab by index.
    ///
    /// `None` clears the selection. `Some(i)` is clamped to the valid range.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Tab, TabBarState};
    ///
    /// let mut state = TabBarState::new(vec![
    ///     Tab::new("a", "A"),
    ///     Tab::new("b", "B"),
    /// ]);
    /// state.set_active(Some(1));
    /// assert_eq!(state.active_index(), Some(1));
    ///
    /// state.set_active(None);
    /// assert_eq!(state.active_index(), None);
    /// ```
    pub fn set_active(&mut self, index: Option<usize>) {
        match index {
            Some(i) if !self.tabs.is_empty() => {
                self.active = Some(i.min(self.tabs.len() - 1));
            }
            _ => self.active = None,
        }
    }

    /// Sets the scroll offset.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Tab, TabBarState};
    ///
    /// let mut state = TabBarState::new(vec![Tab::new("a", "A")]);
    /// state.set_scroll_offset(5);
    /// assert_eq!(state.scroll_offset(), 5);
    /// ```
    pub fn set_scroll_offset(&mut self, offset: usize) {
        self.scroll_offset = offset;
    }

    /// Sets the max tab width.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Tab, TabBarState};
    ///
    /// let mut state = TabBarState::new(vec![Tab::new("a", "A")]);
    /// state.set_max_tab_width(Some(15));
    /// assert_eq!(state.max_tab_width(), Some(15));
    /// ```
    pub fn set_max_tab_width(&mut self, max: Option<usize>) {
        self.max_tab_width = max;
    }

    /// Updates a tab at the given index via a closure.
    ///
    /// No-ops if the index is out of bounds. This is safe because
    /// it does not change the number of tabs or their positions,
    /// so the active index remains valid.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Tab, TabBarState};
    ///
    /// let mut state = TabBarState::new(vec![
    ///     Tab::new("a", "Alpha"),
    ///     Tab::new("b", "Beta"),
    /// ]);
    /// state.update_tab(1, |tab| tab.set_modified(true));
    /// assert!(state.tabs()[1].modified());
    /// ```
    pub fn update_tab(&mut self, index: usize, f: impl FnOnce(&mut Tab)) {
        if let Some(tab) = self.tabs.get_mut(index) {
            f(tab);
        }
    }

    /// Replaces all tabs, clamping or clearing the active index.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Tab, TabBarState};
    ///
    /// let mut state = TabBarState::new(vec![Tab::new("a", "A")]);
    /// state.set_tabs(vec![Tab::new("x", "X"), Tab::new("y", "Y")]);
    /// assert_eq!(state.len(), 2);
    /// assert_eq!(state.tabs()[0].label(), "X");
    /// ```
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Tab, TabBarState};
    ///
    /// let state = TabBarState::new(vec![
    ///     Tab::new("file1", "main.rs"),
    ///     Tab::new("file2", "lib.rs"),
    /// ]);
    /// let (index, tab) = state.find_tab_by_id("file2").unwrap();
    /// assert_eq!(index, 1);
    /// assert_eq!(tab.label(), "lib.rs");
    ///
    /// assert!(state.find_tab_by_id("missing").is_none());
    /// ```
    pub fn find_tab_by_id(&self, id: &str) -> Option<(usize, &Tab)> {
        self.tabs.iter().enumerate().find(|(_, t)| t.id() == id)
    }

    // -- Instance methods that delegate to component -------------------------

    /// Updates the tab bar state with a message, returning any output.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Tab, TabBarState, TabBarMessage, TabBarOutput};
    ///
    /// let mut state = TabBarState::new(vec![
    ///     Tab::new("a", "A"),
    ///     Tab::new("b", "B"),
    /// ]);
    /// let output = state.update(TabBarMessage::NextTab);
    /// assert_eq!(output, Some(TabBarOutput::TabSelected(1)));
    /// assert_eq!(state.active_index(), Some(1));
    /// ```
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
///     Component, Tab, TabBar, TabBarMessage, TabBarOutput, TabBarState,
/// };
///
/// let tabs = vec![
///     Tab::new("1", "Overview"),
///     Tab::new("2", "Details").with_closable(true),
/// ];
/// let mut state = TabBarState::new(tabs);
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

    fn handle_event(
        _state: &Self::State,
        event: &Event,
        ctx: &EventContext,
    ) -> Option<Self::Message> {
        if !ctx.focused || ctx.disabled {
            return None;
        }
        if let Some(key) = event.as_key() {
            match key.code {
                Key::Left | Key::Char('h') => Some(TabBarMessage::PrevTab),
                Key::Right | Key::Char('l') => Some(TabBarMessage::NextTab),
                Key::Home => Some(TabBarMessage::First),
                Key::End => Some(TabBarMessage::Last),
                Key::Char('w') => Some(TabBarMessage::CloseActiveTab),
                _ => None,
            }
        } else {
            None
        }
    }

    fn view(state: &Self::State, ctx: &mut RenderContext<'_, '_>) {
        render::render_tab_bar(
            state,
            ctx.frame,
            ctx.area,
            ctx.theme,
            ctx.focused,
            ctx.disabled,
        );
    }
}

#[cfg(test)]
mod tests;
