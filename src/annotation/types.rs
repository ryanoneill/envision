//! Annotation types for widget metadata.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The type of widget being annotated.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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
        )
    }

    /// Returns true if this is a container widget.
    pub fn is_container(&self) -> bool {
        matches!(
            self,
            WidgetType::Container | WidgetType::Dialog | WidgetType::Scroll | WidgetType::Sidebar
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
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
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
mod tests {
    use super::*;

    #[test]
    fn test_annotation_builder() {
        let ann = Annotation::button("submit")
            .with_label("Submit Order")
            .with_focus(true);

        assert_eq!(ann.widget_type, WidgetType::Button);
        assert_eq!(ann.id, Some("submit".to_string()));
        assert_eq!(ann.label, Some("Submit Order".to_string()));
        assert!(ann.focused);
    }

    #[test]
    fn test_annotation_has_id() {
        let ann = Annotation::input("username");
        assert!(ann.has_id("username"));
        assert!(!ann.has_id("password"));
    }

    #[test]
    fn test_widget_type_interactive() {
        assert!(WidgetType::Button.is_interactive());
        assert!(WidgetType::Input.is_interactive());
        assert!(!WidgetType::Label.is_interactive());
        assert!(!WidgetType::Container.is_interactive());
    }

    #[test]
    fn test_annotation_description() {
        let ann = Annotation::button("ok").with_label("OK").with_focus(true);

        let desc = ann.description();
        assert!(desc.contains("Button"));
        assert!(desc.contains("OK"));
        assert!(desc.contains("#ok"));
        assert!(desc.contains("(focused)"));
    }

    #[test]
    fn test_annotation_serialization() {
        let ann = Annotation::input("email")
            .with_label("Email Address")
            .with_value("test@example.com");

        let json = serde_json::to_string(&ann).unwrap();
        let deserialized: Annotation = serde_json::from_str(&json).unwrap();

        assert_eq!(ann, deserialized);
    }

    #[test]
    fn test_annotation_metadata() {
        let ann = Annotation::custom("datepicker", "birth_date")
            .with_meta("format", "YYYY-MM-DD")
            .with_meta("min_year", "1900");

        assert_eq!(ann.metadata.get("format"), Some(&"YYYY-MM-DD".to_string()));
        assert_eq!(ann.metadata.get("min_year"), Some(&"1900".to_string()));
    }

    #[test]
    fn test_widget_type_is_container() {
        assert!(WidgetType::Container.is_container());
        assert!(WidgetType::Dialog.is_container());
        assert!(WidgetType::Scroll.is_container());
        assert!(WidgetType::Sidebar.is_container());
        assert!(!WidgetType::Button.is_container());
        assert!(!WidgetType::Input.is_container());
    }

    #[test]
    fn test_widget_type_display() {
        assert_eq!(format!("{}", WidgetType::Button), "Button");
        assert_eq!(format!("{}", WidgetType::Input), "Input");
        assert_eq!(
            format!("{}", WidgetType::Custom("MyWidget".to_string())),
            "MyWidget"
        );
    }

    #[test]
    fn test_annotation_new() {
        let ann = Annotation::new(WidgetType::Progress);
        assert_eq!(ann.widget_type, WidgetType::Progress);
        assert!(ann.label.is_none());
        assert!(ann.id.is_none());
        assert!(!ann.focused);
        assert!(!ann.disabled);
        assert!(!ann.selected);
        assert!(ann.expanded.is_none());
        assert!(ann.value.is_none());
        assert!(ann.metadata.is_empty());
    }

    #[test]
    fn test_annotation_default() {
        let ann = Annotation::default();
        assert_eq!(ann.widget_type, WidgetType::Container);
    }

    #[test]
    fn test_annotation_dialog() {
        let ann = Annotation::dialog("Confirm Delete");
        assert_eq!(ann.widget_type, WidgetType::Dialog);
        assert_eq!(ann.label, Some("Confirm Delete".to_string()));
    }

    #[test]
    fn test_annotation_text_area() {
        let ann = Annotation::text_area("description");
        assert_eq!(ann.widget_type, WidgetType::TextArea);
        assert!(ann.has_id("description"));
    }

    #[test]
    fn test_annotation_checkbox() {
        let ann = Annotation::checkbox("agree_tos");
        assert_eq!(ann.widget_type, WidgetType::Checkbox);
        assert!(ann.has_id("agree_tos"));
    }

    #[test]
    fn test_annotation_list() {
        let ann = Annotation::list("items");
        assert_eq!(ann.widget_type, WidgetType::List);
        assert!(ann.has_id("items"));
    }

    #[test]
    fn test_annotation_table() {
        let ann = Annotation::table("data_grid");
        assert_eq!(ann.widget_type, WidgetType::Table);
        assert!(ann.has_id("data_grid"));
    }

    #[test]
    fn test_annotation_tab() {
        let ann = Annotation::tab("settings_tab");
        assert_eq!(ann.widget_type, WidgetType::Tab);
        assert!(ann.has_id("settings_tab"));
    }

    #[test]
    fn test_annotation_menu_item() {
        let ann = Annotation::menu_item("file_open");
        assert_eq!(ann.widget_type, WidgetType::MenuItem);
        assert!(ann.has_id("file_open"));
    }

    #[test]
    fn test_annotation_label() {
        let ann = Annotation::label("Username:");
        assert_eq!(ann.widget_type, WidgetType::Label);
        assert_eq!(ann.label, Some("Username:".to_string()));
    }

    #[test]
    fn test_annotation_header() {
        let ann = Annotation::header("Welcome");
        assert_eq!(ann.widget_type, WidgetType::Header);
        assert_eq!(ann.label, Some("Welcome".to_string()));
    }

    #[test]
    fn test_annotation_with_disabled() {
        let ann = Annotation::button("submit").with_disabled(true);
        assert!(ann.disabled);
    }

    #[test]
    fn test_annotation_with_selected() {
        let ann = Annotation::checkbox("option").with_selected(true);
        assert!(ann.selected);
    }

    #[test]
    fn test_annotation_with_expanded() {
        let ann = Annotation::container("tree_node").with_expanded(true);
        assert_eq!(ann.expanded, Some(true));

        let collapsed = Annotation::container("tree_node").with_expanded(false);
        assert_eq!(collapsed.expanded, Some(false));
    }

    #[test]
    fn test_annotation_is_type() {
        let ann = Annotation::button("btn");
        assert!(ann.is_type(&WidgetType::Button));
        assert!(!ann.is_type(&WidgetType::Input));
    }

    #[test]
    fn test_annotation_is_interactive() {
        // Button is interactive
        let btn = Annotation::button("btn");
        assert!(btn.is_interactive());

        // Disabled button is not interactive
        let disabled_btn = Annotation::button("btn").with_disabled(true);
        assert!(!disabled_btn.is_interactive());

        // Label is not interactive
        let lbl = Annotation::label("text");
        assert!(!lbl.is_interactive());
    }

    #[test]
    fn test_annotation_description_with_states() {
        let ann = Annotation::button("save")
            .with_disabled(true)
            .with_selected(true);

        let desc = ann.description();
        assert!(desc.contains("(disabled)"));
        assert!(desc.contains("(selected)"));
    }

    #[test]
    fn test_widget_type_interactive_all() {
        // Test all interactive types
        assert!(WidgetType::TextArea.is_interactive());
        assert!(WidgetType::Checkbox.is_interactive());
        assert!(WidgetType::Radio.is_interactive());
        assert!(WidgetType::Select.is_interactive());
        assert!(WidgetType::List.is_interactive());
        assert!(WidgetType::Table.is_interactive());
        assert!(WidgetType::Tab.is_interactive());
        assert!(WidgetType::MenuItem.is_interactive());
        assert!(WidgetType::Tree.is_interactive());

        // Non-interactive types
        assert!(!WidgetType::Header.is_interactive());
        assert!(!WidgetType::Footer.is_interactive());
        assert!(!WidgetType::Sidebar.is_interactive());
        assert!(!WidgetType::Toolbar.is_interactive());
        assert!(!WidgetType::StatusBar.is_interactive());
        assert!(!WidgetType::Progress.is_interactive());
        assert!(!WidgetType::Scroll.is_interactive());
        assert!(!WidgetType::TabBar.is_interactive());
        assert!(!WidgetType::Menu.is_interactive());
        assert!(!WidgetType::Custom("test".to_string()).is_interactive());
    }

    #[test]
    fn test_widget_type_debug() {
        let wt = WidgetType::Button;
        let debug = format!("{:?}", wt);
        assert_eq!(debug, "Button");
    }

    #[test]
    fn test_widget_type_clone() {
        let wt = WidgetType::Custom("MyType".to_string());
        let cloned = wt.clone();
        assert_eq!(wt, cloned);
    }

    #[test]
    fn test_widget_type_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(WidgetType::Button);
        set.insert(WidgetType::Input);
        set.insert(WidgetType::Button); // Duplicate

        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_annotation_clone() {
        let ann = Annotation::input("email")
            .with_label("Email")
            .with_value("test@example.com")
            .with_meta("type", "email");

        let cloned = ann.clone();
        assert_eq!(ann, cloned);
    }

    #[test]
    fn test_annotation_debug() {
        let ann = Annotation::button("btn");
        let debug = format!("{:?}", ann);
        assert!(debug.contains("Annotation"));
        assert!(debug.contains("Button"));
    }

    #[test]
    fn test_annotation_eq() {
        let ann1 = Annotation::button("btn").with_label("Click");
        let ann2 = Annotation::button("btn").with_label("Click");
        let ann3 = Annotation::button("btn").with_label("Different");

        assert_eq!(ann1, ann2);
        assert_ne!(ann1, ann3);
    }
}
