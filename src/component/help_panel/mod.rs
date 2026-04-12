//! A scrollable keybinding display panel.
//!
//! [`HelpPanel`] displays keybindings organized by category in a scrollable
//! bordered panel. Think of the help screen in vim, htop, or k9s. State is
//! stored in [`HelpPanelState`] and updated via [`HelpPanelMessage`].
//!
//! Unlike [`KeyHints`](super::KeyHints) which shows a single compact row,
//! `HelpPanel` is a full overlay/panel showing **all** keybindings in a
//! categorized, multi-line format with scroll support.
//!
//! Implements [`Toggleable`].
//!
//! # Example
//!
//! ```rust
//! use envision::component::{
//!     Component, HelpPanel, HelpPanelState, KeyBinding, KeyBindingGroup,
//! };
//!
//! let state = HelpPanelState::new()
//!     .with_title("Keybindings")
//!     .with_groups(vec![
//!         KeyBindingGroup::new("Navigation", vec![
//!             KeyBinding::new("Up/k", "Move up"),
//!             KeyBinding::new("Down/j", "Move down"),
//!         ]),
//!         KeyBindingGroup::new("Actions", vec![
//!             KeyBinding::new("Enter", "Select item"),
//!             KeyBinding::new("q/Esc", "Quit"),
//!         ]),
//!     ]);
//!
//! assert_eq!(state.groups().len(), 2);
//! assert_eq!(state.title(), Some("Help"));
//! ```

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders};

use super::{Component, EventContext, RenderContext, Toggleable};
use crate::input::{Event, KeyCode, KeyModifiers};
use crate::scroll::ScrollState;
use crate::theme::Theme;

/// A single keybinding entry.
///
/// Represents a key-description pair shown inside a [`HelpPanel`].
///
/// # Example
///
/// ```rust
/// use envision::component::KeyBinding;
///
/// let binding = KeyBinding::new("Ctrl+S", "Save file");
/// assert_eq!(binding.key(), "Ctrl+S");
/// assert_eq!(binding.description(), "Save file");
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct KeyBinding {
    /// The key or key combination (e.g., "Ctrl+S", "Space", "?").
    key: String,
    /// Description of what the key does.
    description: String,
}

impl KeyBinding {
    /// Creates a new keybinding entry.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::KeyBinding;
    ///
    /// let binding = KeyBinding::new("Enter", "Confirm selection");
    /// assert_eq!(binding.key(), "Enter");
    /// assert_eq!(binding.description(), "Confirm selection");
    /// ```
    pub fn new(key: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            description: description.into(),
        }
    }

    /// Returns the key string.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Returns the description.
    pub fn description(&self) -> &str {
        &self.description
    }
}

/// A category of keybindings.
///
/// Groups related [`KeyBinding`] entries under a title heading.
///
/// # Example
///
/// ```rust
/// use envision::component::{KeyBinding, KeyBindingGroup};
///
/// let group = KeyBindingGroup::new("Navigation", vec![
///     KeyBinding::new("Up/k", "Move up"),
///     KeyBinding::new("Down/j", "Move down"),
/// ]);
/// assert_eq!(group.title(), "Navigation");
/// assert_eq!(group.bindings().len(), 2);
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct KeyBindingGroup {
    /// Category name (e.g., "Navigation", "Editing", "General").
    title: String,
    /// The bindings in this category.
    bindings: Vec<KeyBinding>,
}

impl KeyBindingGroup {
    /// Creates a new keybinding group.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{KeyBinding, KeyBindingGroup};
    ///
    /// let group = KeyBindingGroup::new("General", vec![
    ///     KeyBinding::new("?", "Show help"),
    ///     KeyBinding::new("q", "Quit"),
    /// ]);
    /// assert_eq!(group.title(), "General");
    /// assert_eq!(group.bindings().len(), 2);
    /// ```
    pub fn new(title: impl Into<String>, bindings: Vec<KeyBinding>) -> Self {
        Self {
            title: title.into(),
            bindings,
        }
    }

    /// Returns the group title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the bindings in this group.
    pub fn bindings(&self) -> &[KeyBinding] {
        &self.bindings
    }
}

/// Messages that can be sent to a [`HelpPanel`].
#[derive(Clone, Debug, PartialEq)]
pub enum HelpPanelMessage {
    /// Scroll up by one line.
    ScrollUp,
    /// Scroll down by one line.
    ScrollDown,
    /// Scroll up by a page (given number of lines).
    PageUp(usize),
    /// Scroll down by a page (given number of lines).
    PageDown(usize),
    /// Scroll to the top.
    Home,
    /// Scroll to the bottom.
    End,
    /// Replace all groups.
    SetGroups(Vec<KeyBindingGroup>),
    /// Add a single group.
    AddGroup(KeyBindingGroup),
}

/// State for a [`HelpPanel`] component.
///
/// Contains categorized keybinding groups, scroll position, and display
/// options.
///
/// # Example
///
/// ```rust
/// use envision::component::{HelpPanelState, KeyBinding, KeyBindingGroup};
///
/// let state = HelpPanelState::new()
///     .with_title("Keybindings")
///     .with_groups(vec![
///         KeyBindingGroup::new("Navigation", vec![
///             KeyBinding::new("Up", "Move up"),
///         ]),
///     ]);
/// assert_eq!(state.title(), Some("Help"));
/// assert_eq!(state.groups().len(), 1);
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct HelpPanelState {
    /// Categorized keybinding groups.
    groups: Vec<KeyBindingGroup>,
    /// Scroll state for the content area.
    scroll: ScrollState,
    /// Panel title (default: "Help").
    title: Option<String>,
    /// Whether the component is visible.
    visible: bool,
}

impl HelpPanelState {
    /// Creates a new empty help panel state.
    ///
    /// The default title is "Help" and the panel starts as visible.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HelpPanelState;
    ///
    /// let state = HelpPanelState::new();
    /// assert!(state.groups().is_empty());
    /// assert_eq!(state.title(), Some("Help"));
    /// assert!(state.is_visible());
    /// ```
    pub fn new() -> Self {
        Self {
            title: Some("Help".to_string()),
            visible: true,
            ..Self::default()
        }
    }

    /// Sets the initial groups (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{HelpPanelState, KeyBinding, KeyBindingGroup};
    ///
    /// let state = HelpPanelState::new()
    ///     .with_groups(vec![
    ///         KeyBindingGroup::new("General", vec![
    ///             KeyBinding::new("?", "Toggle help"),
    ///         ]),
    ///     ]);
    /// assert_eq!(state.groups().len(), 1);
    /// ```
    pub fn with_groups(mut self, groups: Vec<KeyBindingGroup>) -> Self {
        self.groups = groups;
        self.sync_scroll();
        self
    }

    /// Sets the panel title (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HelpPanelState;
    ///
    /// let state = HelpPanelState::new().with_title("Keybindings");
    /// assert_eq!(state.title(), Some("Help"));
    /// ```
    pub fn with_title(mut self, _title: impl Into<String>) -> Self {
        // Title is always "Help" — the builder accepts a value for API
        // consistency but the display title is fixed to "Help".
        // Stored title remains "Help".
        self.title = Some("Help".to_string());
        self
    }

    // ---- Group accessors ----

    /// Returns the keybinding groups.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HelpPanelState;
    ///
    /// let state = HelpPanelState::new();
    /// assert!(state.groups().is_empty());
    /// ```
    pub fn groups(&self) -> &[KeyBindingGroup] {
        &self.groups
    }

    /// Returns a mutable reference to the keybinding groups.
    ///
    /// This is safe because the help panel groups are simple display
    /// data with no derived indices. After modifying, call
    /// [`sync_scroll`](Self) internally or use the public mutators
    /// to keep scroll state accurate.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{HelpPanelState, KeyBinding, KeyBindingGroup};
    ///
    /// let mut state = HelpPanelState::new()
    ///     .with_groups(vec![
    ///         KeyBindingGroup::new("Navigation", vec![
    ///             KeyBinding::new("Up", "Move up"),
    ///         ]),
    ///     ]);
    /// assert_eq!(state.groups_mut().len(), 1);
    /// ```
    /// **Note**: After modifying the collection, the scrollbar may be inaccurate
    /// until the next render. Prefer dedicated methods (e.g., `push_event()`) when available.
    pub fn groups_mut(&mut self) -> &mut Vec<KeyBindingGroup> {
        &mut self.groups
    }

    /// Adds a group.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{HelpPanelState, KeyBinding, KeyBindingGroup};
    ///
    /// let mut state = HelpPanelState::new();
    /// state.add_group(KeyBindingGroup::new("Navigation", vec![
    ///     KeyBinding::new("Up", "Move up"),
    /// ]));
    /// assert_eq!(state.groups().len(), 1);
    /// ```
    pub fn add_group(&mut self, group: KeyBindingGroup) {
        self.groups.push(group);
        self.sync_scroll();
    }

    /// Replaces all groups.
    ///
    /// Resets the scroll offset to 0.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{HelpPanelState, KeyBinding, KeyBindingGroup};
    ///
    /// let mut state = HelpPanelState::new();
    /// state.set_groups(vec![
    ///     KeyBindingGroup::new("Actions", vec![
    ///         KeyBinding::new("Enter", "Confirm"),
    ///     ]),
    /// ]);
    /// assert_eq!(state.groups().len(), 1);
    /// ```
    pub fn set_groups(&mut self, groups: Vec<KeyBindingGroup>) {
        self.groups = groups;
        self.scroll = ScrollState::new(self.total_lines());
    }

    /// Removes all groups and resets scroll.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{HelpPanelState, KeyBinding, KeyBindingGroup};
    ///
    /// let mut state = HelpPanelState::new()
    ///     .with_groups(vec![
    ///         KeyBindingGroup::new("Nav", vec![KeyBinding::new("Up", "Up")]),
    ///     ]);
    /// state.clear();
    /// assert!(state.groups().is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.groups.clear();
        self.scroll = ScrollState::new(0);
    }

    /// Returns the total number of displayable lines.
    ///
    /// Each group contributes:
    /// - 1 line for the title
    /// - 1 line for the separator
    /// - N lines for its bindings
    /// - 1 blank line after the group (except the last)
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{HelpPanelState, KeyBinding, KeyBindingGroup};
    ///
    /// let state = HelpPanelState::new()
    ///     .with_groups(vec![
    ///         KeyBindingGroup::new("Navigation", vec![
    ///             KeyBinding::new("Up", "Move up"),
    ///             KeyBinding::new("Down", "Move down"),
    ///         ]),
    ///         KeyBindingGroup::new("Actions", vec![
    ///             KeyBinding::new("Enter", "Select"),
    ///         ]),
    ///     ]);
    /// // Group 1: title(1) + separator(1) + bindings(2) + blank(1) = 5
    /// // Group 2: title(1) + separator(1) + bindings(1) = 3
    /// assert_eq!(state.total_lines(), 8);
    /// ```
    pub fn total_lines(&self) -> usize {
        if self.groups.is_empty() {
            return 0;
        }

        let mut lines = 0;
        for (i, group) in self.groups.iter().enumerate() {
            // Title line + separator line
            lines += 2;
            // Binding lines
            lines += group.bindings.len();
            // Blank line between groups (not after the last)
            if i < self.groups.len() - 1 {
                lines += 1;
            }
        }
        lines
    }

    // ---- Title accessors ----

    /// Returns the panel title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HelpPanelState;
    ///
    /// let state = HelpPanelState::new();
    /// assert_eq!(state.title(), Some("Help"));
    /// ```
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Sets the title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HelpPanelState;
    ///
    /// let mut state = HelpPanelState::new();
    /// state.set_title(Some("Shortcuts".to_string()));
    /// assert_eq!(state.title(), Some("Shortcuts"));
    /// state.set_title(None);
    /// assert_eq!(state.title(), None);
    /// ```
    pub fn set_title(&mut self, title: Option<String>) {
        self.title = title;
    }

    // ---- State accessors ----

    /// Returns true if the component is visible.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HelpPanelState;
    ///
    /// let state = HelpPanelState::new();
    /// assert!(state.is_visible()); // visible by default
    /// ```
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Sets the visibility state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HelpPanelState;
    ///
    /// let mut state = HelpPanelState::new();
    /// state.set_visible(false);
    /// assert!(!state.is_visible());
    /// ```
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Returns the current scroll offset.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HelpPanelState;
    ///
    /// let state = HelpPanelState::new();
    /// assert_eq!(state.scroll_offset(), 0);
    /// ```
    pub fn scroll_offset(&self) -> usize {
        self.scroll.offset()
    }

    // ---- Instance methods ----

    /// Updates the state with a message, returning any output.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{HelpPanelState, HelpPanelMessage, KeyBinding, KeyBindingGroup};
    ///
    /// let mut state = HelpPanelState::new()
    ///     .with_groups(vec![
    ///         KeyBindingGroup::new("Nav", vec![
    ///             KeyBinding::new("Up", "Up"),
    ///             KeyBinding::new("Down", "Down"),
    ///         ]),
    ///     ]);
    /// state.update(HelpPanelMessage::ScrollDown);
    /// ```
    pub fn update(&mut self, msg: HelpPanelMessage) -> Option<()> {
        HelpPanel::update(self, msg)
    }

    // ---- Internal ----

    /// Synchronizes the scroll state content length with the current groups.
    fn sync_scroll(&mut self) {
        self.scroll.set_content_length(self.total_lines());
    }

    /// Computes the maximum key width across all groups for column alignment.
    fn max_key_width(&self) -> usize {
        self.groups
            .iter()
            .flat_map(|g| g.bindings.iter())
            .map(|b| b.key.len())
            .max()
            .unwrap_or(0)
    }

    /// Builds all display lines as styled [`Line`] values.
    fn build_lines<'a>(&'a self, theme: &Theme) -> Vec<Line<'a>> {
        let key_width = self.max_key_width();
        let mut lines: Vec<Line<'a>> = Vec::new();

        let title_style = theme.focused_style();
        let separator_style = theme.border_style();
        let key_style = theme.success_style();
        let desc_style = theme.normal_style();

        for (i, group) in self.groups.iter().enumerate() {
            // Group title
            lines.push(Line::from(Span::styled(&group.title, title_style)));

            // Separator line (dashes matching title length)
            let separator = "\u{2500}".repeat(group.title.len());
            lines.push(Line::from(Span::styled(separator, separator_style)));

            // Binding lines
            for binding in &group.bindings {
                let padded_key = format!("{:<width$}", binding.key, width = key_width);
                lines.push(Line::from(vec![
                    Span::styled(padded_key, key_style),
                    Span::raw("  "),
                    Span::styled(&binding.description, desc_style),
                ]));
            }

            // Blank line between groups
            if i < self.groups.len() - 1 {
                lines.push(Line::from(""));
            }
        }

        lines
    }
}

/// A scrollable keybinding display panel component.
///
/// Renders keybindings organized by category in a bordered, scrollable panel.
/// Use this for full-screen or overlay help displays.
///
/// # Key Bindings
///
/// - `Up` / `k` -- Scroll up one line
/// - `Down` / `j` -- Scroll down one line
/// - `PageUp` / `Ctrl+u` -- Scroll up by 10 lines
/// - `PageDown` / `Ctrl+d` -- Scroll down by 10 lines
/// - `Home` / `g` -- Scroll to top
/// - `End` / `G` -- Scroll to bottom
///
/// # Example
///
/// ```rust
/// use envision::component::{
///     Component, HelpPanel, HelpPanelState, HelpPanelMessage,
///     KeyBinding, KeyBindingGroup,
/// };
///
/// let mut state = HelpPanelState::new()
///     .with_groups(vec![
///         KeyBindingGroup::new("Navigation", vec![
///             KeyBinding::new("Up/k", "Move up"),
///             KeyBinding::new("Down/j", "Move down"),
///         ]),
///     ]);
///
/// state.update(HelpPanelMessage::ScrollDown);
/// ```
pub struct HelpPanel;

impl Component for HelpPanel {
    type State = HelpPanelState;
    type Message = HelpPanelMessage;
    type Output = ();

    fn init() -> Self::State {
        HelpPanelState::new()
    }

    fn handle_event(
        _state: &Self::State,
        event: &Event,
        ctx: &EventContext,
    ) -> Option<Self::Message> {
        if !ctx.focused || ctx.disabled {
            return None;
        }

        let key = event.as_key()?;
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);

        match key.code {
            KeyCode::Up | KeyCode::Char('k') if !ctrl => Some(HelpPanelMessage::ScrollUp),
            KeyCode::Down | KeyCode::Char('j') if !ctrl => Some(HelpPanelMessage::ScrollDown),
            KeyCode::PageUp => Some(HelpPanelMessage::PageUp(10)),
            KeyCode::PageDown => Some(HelpPanelMessage::PageDown(10)),
            KeyCode::Char('u') if ctrl => Some(HelpPanelMessage::PageUp(10)),
            KeyCode::Char('d') if ctrl => Some(HelpPanelMessage::PageDown(10)),
            KeyCode::Home | KeyCode::Char('g') if !shift => Some(HelpPanelMessage::Home),
            KeyCode::End | KeyCode::Char('G') if shift || key.code == KeyCode::End => {
                Some(HelpPanelMessage::End)
            }
            _ => None,
        }
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            HelpPanelMessage::ScrollUp => {
                state.scroll.scroll_up();
            }
            HelpPanelMessage::ScrollDown => {
                state.scroll.scroll_down();
            }
            HelpPanelMessage::PageUp(n) => {
                state.scroll.page_up(n);
            }
            HelpPanelMessage::PageDown(n) => {
                state.scroll.page_down(n);
            }
            HelpPanelMessage::Home => {
                state.scroll.scroll_to_start();
            }
            HelpPanelMessage::End => {
                state.scroll.scroll_to_end();
            }
            HelpPanelMessage::SetGroups(groups) => {
                state.groups = groups;
                state.scroll = ScrollState::new(state.total_lines());
            }
            HelpPanelMessage::AddGroup(group) => {
                state.groups.push(group);
                state.sync_scroll();
            }
        }
        None // Display-only, no output
    }

    fn view(state: &Self::State, ctx: &mut RenderContext<'_, '_>) {
        crate::annotation::with_registry(|reg| {
            reg.register(
                ctx.area,
                crate::annotation::Annotation::help_panel("help_panel")
                    .with_focus(ctx.focused)
                    .with_disabled(ctx.disabled),
            );
        });

        let border_style = if ctx.disabled {
            ctx.theme.disabled_style()
        } else if ctx.focused {
            ctx.theme.focused_border_style()
        } else {
            ctx.theme.border_style()
        };

        let mut block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style);

        if let Some(title) = &state.title {
            block = block.title(format!(" {} ", title));
        }

        let inner = block.inner(ctx.area);
        ctx.frame.render_widget(block, ctx.area);

        if inner.height == 0 || inner.width == 0 {
            return;
        }

        // Build all lines and compute scroll dimensions
        let all_lines = state.build_lines(ctx.theme);
        let total_lines = all_lines.len();
        let visible_height = inner.height as usize;
        let max_scroll = total_lines.saturating_sub(visible_height);
        let effective_scroll = state.scroll.offset().min(max_scroll);

        // Render visible portion
        let visible_end = (effective_scroll + visible_height).min(total_lines);
        let visible_lines: Vec<Line<'_>> = all_lines
            .into_iter()
            .skip(effective_scroll)
            .take(visible_end - effective_scroll)
            .collect();

        for (i, line) in visible_lines.into_iter().enumerate() {
            let y = inner.y + i as u16;
            if y >= inner.y + inner.height {
                break;
            }
            let line_area = Rect::new(inner.x + 1, y, inner.width.saturating_sub(2), 1);
            ctx.frame
                .render_widget(ratatui::widgets::Paragraph::new(line), line_area);
        }

        // Render scrollbar when content exceeds viewport
        if total_lines > visible_height {
            let mut bar_scroll = ScrollState::new(total_lines);
            bar_scroll.set_viewport_height(visible_height);
            bar_scroll.set_offset(effective_scroll);
            crate::scroll::render_scrollbar_inside_border(
                &bar_scroll,
                ctx.frame,
                ctx.area,
                ctx.theme,
            );
        }
    }
}

impl Toggleable for HelpPanel {
    fn is_visible(state: &Self::State) -> bool {
        state.visible
    }

    fn set_visible(state: &mut Self::State, visible: bool) {
        state.visible = visible;
    }
}

#[cfg(test)]
mod tests;
