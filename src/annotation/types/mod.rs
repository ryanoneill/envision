//! Annotation types for widget metadata.

use std::collections::HashMap;

/// The type of widget being annotated.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "serialization", serde(rename_all = "snake_case"))]
pub enum WidgetType {
    /// A container or panel
    Container,

    /// A dialog or modal
    Dialog,

    /// A button
    Button,

    /// A text input field
    Input,

    /// A text area (multi-line input)
    TextArea,

    /// A checkbox
    Checkbox,

    /// A radio button
    Radio,

    /// A dropdown/select
    Select,

    /// A list widget
    List,

    /// A table widget
    Table,

    /// A tab bar
    TabBar,

    /// A single tab
    Tab,

    /// A menu
    Menu,

    /// A menu item
    MenuItem,

    /// A label/text display
    Label,

    /// A header/title
    Header,

    /// A footer
    Footer,

    /// A sidebar
    Sidebar,

    /// A toolbar
    Toolbar,

    /// A status bar
    StatusBar,

    /// A progress indicator
    Progress,

    /// A scrollable area
    Scroll,

    /// A tree view
    Tree,

    /// A spinner/loading indicator
    Spinner,

    /// A toast notification
    Toast,

    /// A tooltip overlay
    Tooltip,

    /// An accordion panel group
    Accordion,

    /// A breadcrumb navigation trail
    Breadcrumb,

    /// A loading list
    LoadingList,

    /// A key hints display
    KeyHints,

    /// A multi-progress display
    MultiProgress,

    /// A status log
    StatusLog,

    /// A title card display
    TitleCard,

    /// A line input
    LineInput,

    /// A dropdown select
    Dropdown,

    /// A scrollable text display
    ScrollableText,

    /// A form container
    Form,

    /// A split panel container
    SplitPanel,

    /// A searchable list container
    SearchableList,

    /// A radio group
    RadioGroup,

    /// A file browser
    FileBrowser,

    /// A confirm dialog
    ConfirmDialog,

    /// A step indicator
    StepIndicator,

    /// A styled text display
    StyledText,

    /// A pane layout manager
    PaneLayout,

    /// A sparkline data trend display
    Sparkline,

    /// A divider/separator line
    Divider,

    /// A canvas drawing surface
    Canvas,

    /// A paginator navigation indicator
    Paginator,

    /// A help panel display
    HelpPanel,

    /// A toggle switch
    Switch,

    /// A custom widget type
    Custom(String),
}

impl WidgetType {
    /// Returns true if this is an interactive widget (can receive input).
    pub fn is_interactive(&self) -> bool {
        matches!(
            self,
            WidgetType::Button
                | WidgetType::Input
                | WidgetType::TextArea
                | WidgetType::Checkbox
                | WidgetType::Radio
                | WidgetType::Select
                | WidgetType::List
                | WidgetType::Table
                | WidgetType::Tab
                | WidgetType::MenuItem
                | WidgetType::Tree
                | WidgetType::LineInput
                | WidgetType::Dropdown
                | WidgetType::LoadingList
                | WidgetType::Accordion
                | WidgetType::RadioGroup
                | WidgetType::SearchableList
                | WidgetType::FileBrowser
                | WidgetType::StepIndicator
                | WidgetType::Paginator
                | WidgetType::Switch
        )
    }

    /// Returns true if this is a container widget.
    pub fn is_container(&self) -> bool {
        matches!(
            self,
            WidgetType::Container
                | WidgetType::Dialog
                | WidgetType::ConfirmDialog
                | WidgetType::Scroll
                | WidgetType::Sidebar
                | WidgetType::Form
                | WidgetType::SplitPanel
        )
    }
}

impl std::fmt::Display for WidgetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WidgetType::Custom(name) => write!(f, "{}", name),
            other => write!(f, "{:?}", other),
        }
    }
}

/// Metadata annotation for a widget.
///
/// Annotations provide semantic information about widgets that
/// can be used for testing, accessibility, and UI queries.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct Annotation {
    /// The type of widget
    pub widget_type: WidgetType,

    /// Human-readable label
    pub label: Option<String>,

    /// Programmatic identifier
    pub id: Option<String>,

    /// Whether this widget currently has focus
    pub focused: bool,

    /// Whether this widget is disabled
    pub disabled: bool,

    /// Whether this widget is selected (for selectable items)
    pub selected: bool,

    /// Whether this widget is expanded (for collapsible items)
    pub expanded: Option<bool>,

    /// Current value (for inputs, etc.)
    pub value: Option<String>,

    /// Additional metadata
    #[cfg_attr(
        feature = "serialization",
        serde(default, skip_serializing_if = "HashMap::is_empty")
    )]
    pub metadata: HashMap<String, String>,
}

impl Annotation {
    /// Creates a new annotation with the given widget type.
    pub fn new(widget_type: WidgetType) -> Self {
        Self {
            widget_type,
            label: None,
            id: None,
            focused: false,
            disabled: false,
            selected: false,
            expanded: None,
            value: None,
            metadata: HashMap::new(),
        }
    }

    // Convenience constructors for common widget types

    /// Creates a container annotation.
    pub fn container(id: impl Into<String>) -> Self {
        Self::new(WidgetType::Container).with_id(id)
    }

    /// Creates a dialog annotation.
    pub fn dialog(title: impl Into<String>) -> Self {
        Self::new(WidgetType::Dialog).with_label(title)
    }

    /// Creates a button annotation.
    pub fn button(id: impl Into<String>) -> Self {
        Self::new(WidgetType::Button).with_id(id)
    }

    /// Creates an input field annotation.
    pub fn input(id: impl Into<String>) -> Self {
        Self::new(WidgetType::Input).with_id(id)
    }

    /// Creates a text area annotation.
    pub fn text_area(id: impl Into<String>) -> Self {
        Self::new(WidgetType::TextArea).with_id(id)
    }

    /// Creates a checkbox annotation.
    pub fn checkbox(id: impl Into<String>) -> Self {
        Self::new(WidgetType::Checkbox).with_id(id)
    }

    /// Creates a list annotation.
    pub fn list(id: impl Into<String>) -> Self {
        Self::new(WidgetType::List).with_id(id)
    }

    /// Creates a table annotation.
    pub fn table(id: impl Into<String>) -> Self {
        Self::new(WidgetType::Table).with_id(id)
    }

    /// Creates a tab annotation.
    pub fn tab(id: impl Into<String>) -> Self {
        Self::new(WidgetType::Tab).with_id(id)
    }

    /// Creates a menu item annotation.
    pub fn menu_item(id: impl Into<String>) -> Self {
        Self::new(WidgetType::MenuItem).with_id(id)
    }

    /// Creates a label annotation.
    pub fn label(text: impl Into<String>) -> Self {
        Self::new(WidgetType::Label).with_label(text)
    }

    /// Creates a header annotation.
    pub fn header(text: impl Into<String>) -> Self {
        Self::new(WidgetType::Header).with_label(text)
    }

    /// Creates a spinner annotation.
    pub fn spinner(id: impl Into<String>) -> Self {
        Self::new(WidgetType::Spinner).with_id(id)
    }

    /// Creates a toast annotation.
    pub fn toast(id: impl Into<String>) -> Self {
        Self::new(WidgetType::Toast).with_id(id)
    }

    /// Creates a tooltip annotation.
    pub fn tooltip(id: impl Into<String>) -> Self {
        Self::new(WidgetType::Tooltip).with_id(id)
    }

    /// Creates an accordion annotation.
    pub fn accordion(id: impl Into<String>) -> Self {
        Self::new(WidgetType::Accordion).with_id(id)
    }

    /// Creates a breadcrumb annotation.
    pub fn breadcrumb(id: impl Into<String>) -> Self {
        Self::new(WidgetType::Breadcrumb).with_id(id)
    }

    /// Creates a loading list annotation.
    pub fn loading_list(id: impl Into<String>) -> Self {
        Self::new(WidgetType::LoadingList).with_id(id)
    }

    /// Creates a key hints annotation.
    pub fn key_hints(id: impl Into<String>) -> Self {
        Self::new(WidgetType::KeyHints).with_id(id)
    }

    /// Creates a multi-progress annotation.
    pub fn multi_progress(id: impl Into<String>) -> Self {
        Self::new(WidgetType::MultiProgress).with_id(id)
    }

    /// Creates a status log annotation.
    pub fn status_log(id: impl Into<String>) -> Self {
        Self::new(WidgetType::StatusLog).with_id(id)
    }

    /// Creates a title card annotation.
    pub fn title_card(id: impl Into<String>) -> Self {
        Self::new(WidgetType::TitleCard).with_id(id)
    }

    /// Creates a line input annotation.
    pub fn line_input(id: impl Into<String>) -> Self {
        Self::new(WidgetType::LineInput).with_id(id)
    }

    /// Creates a dropdown annotation.
    pub fn dropdown(id: impl Into<String>) -> Self {
        Self::new(WidgetType::Dropdown).with_id(id)
    }

    /// Creates a scrollable text annotation.
    pub fn scrollable_text(id: impl Into<String>) -> Self {
        Self::new(WidgetType::ScrollableText).with_id(id)
    }

    /// Creates a form annotation.
    pub fn form(id: impl Into<String>) -> Self {
        Self::new(WidgetType::Form).with_id(id)
    }

    /// Creates a split panel annotation.
    pub fn split_panel(id: impl Into<String>) -> Self {
        Self::new(WidgetType::SplitPanel).with_id(id)
    }

    /// Creates a searchable list annotation.
    pub fn searchable_list(id: impl Into<String>) -> Self {
        Self::new(WidgetType::SearchableList).with_id(id)
    }

    /// Creates a radio group annotation.
    pub fn radio_group(id: impl Into<String>) -> Self {
        Self::new(WidgetType::RadioGroup).with_id(id)
    }

    /// Creates a file browser annotation.
    pub fn file_browser(id: impl Into<String>) -> Self {
        Self::new(WidgetType::FileBrowser).with_id(id)
    }

    /// Creates a confirm dialog annotation.
    pub fn confirm_dialog(title: impl Into<String>) -> Self {
        Self::new(WidgetType::ConfirmDialog).with_label(title)
    }

    /// Creates a step indicator annotation.
    pub fn step_indicator(id: impl Into<String>) -> Self {
        Self::new(WidgetType::StepIndicator).with_id(id)
    }

    /// Creates a styled text annotation.
    pub fn styled_text(id: impl Into<String>) -> Self {
        Self::new(WidgetType::StyledText).with_id(id)
    }

    /// Creates a pane layout annotation.
    pub fn pane_layout(id: impl Into<String>) -> Self {
        Self::new(WidgetType::PaneLayout).with_id(id)
    }

    /// Creates a sparkline annotation.
    pub fn sparkline(id: impl Into<String>) -> Self {
        Self::new(WidgetType::Sparkline).with_id(id)
    }

    /// Creates a divider annotation.
    pub fn divider(id: impl Into<String>) -> Self {
        Self::new(WidgetType::Divider).with_id(id)
    }

    /// Creates a canvas annotation.
    pub fn canvas(id: impl Into<String>) -> Self {
        Self::new(WidgetType::Canvas).with_id(id)
    }

    /// Creates a paginator annotation.
    pub fn paginator(id: impl Into<String>) -> Self {
        Self::new(WidgetType::Paginator).with_id(id)
    }

    /// Creates a help panel annotation.
    pub fn help_panel(id: impl Into<String>) -> Self {
        Self::new(WidgetType::HelpPanel).with_id(id)
    }

    /// Creates a switch annotation.
    pub fn switch(id: impl Into<String>) -> Self {
        Self::new(WidgetType::Switch).with_id(id)
    }

    /// Creates a custom widget annotation.
    pub fn custom(type_name: impl Into<String>, id: impl Into<String>) -> Self {
        Self::new(WidgetType::Custom(type_name.into())).with_id(id)
    }

    // Builder methods

    /// Sets the label.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets the id.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the focused state.
    pub fn with_focus(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Sets the disabled state.
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets the selected state.
    pub fn with_selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Sets the expanded state.
    pub fn with_expanded(mut self, expanded: bool) -> Self {
        self.expanded = Some(expanded);
        self
    }

    /// Sets the current value.
    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Adds metadata.
    pub fn with_meta(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    // Query methods

    /// Returns true if this annotation matches the given id.
    pub fn has_id(&self, id: &str) -> bool {
        self.id.as_deref() == Some(id)
    }

    /// Returns true if this annotation has the given widget type.
    pub fn is_type(&self, widget_type: &WidgetType) -> bool {
        &self.widget_type == widget_type
    }

    /// Returns true if this is an interactive widget.
    pub fn is_interactive(&self) -> bool {
        self.widget_type.is_interactive() && !self.disabled
    }

    /// Returns a description suitable for display.
    pub fn description(&self) -> String {
        let mut parts = Vec::new();

        parts.push(format!("{:?}", self.widget_type));

        if let Some(label) = &self.label {
            parts.push(format!("\"{}\"", label));
        }

        if let Some(id) = &self.id {
            parts.push(format!("#{}", id));
        }

        if self.focused {
            parts.push("(focused)".to_string());
        }

        if self.disabled {
            parts.push("(disabled)".to_string());
        }

        if self.selected {
            parts.push("(selected)".to_string());
        }

        parts.join(" ")
    }
}

impl Default for Annotation {
    fn default() -> Self {
        Self::new(WidgetType::Container)
    }
}

#[cfg(test)]
mod tests;
