//! Form field types and value representations.
//!
//! Contains [`FormField`], [`FormFieldKind`], and [`FormValue`] used by the
//! [`Form`](super::Form) component.

/// Describes a field to include in a form.
///
/// Each field has an ID for retrieval, a label for display, and a kind
/// that determines the widget type and behavior.
#[derive(Clone, Debug, PartialEq)]
pub struct FormField {
    /// Unique identifier for this field.
    pub(super) id: String,
    /// Display label shown above or beside the field.
    pub(super) label: String,
    /// The type and configuration of this field.
    pub(super) kind: FormFieldKind,
}

/// The type of a form field.
#[derive(Clone, Debug, PartialEq)]
pub enum FormFieldKind {
    /// A text input field.
    Text,
    /// A text input with a placeholder.
    TextWithPlaceholder(String),
    /// A checkbox (boolean toggle).
    Checkbox,
    /// A select dropdown with options.
    Select(Vec<String>),
}

/// A collected value from a form field.
#[derive(Clone, Debug, PartialEq)]
pub enum FormValue {
    /// Text from a text input field.
    Text(String),
    /// Boolean from a checkbox.
    Bool(bool),
    /// Selected option from a select field (value and index).
    Selected(Option<String>),
}

impl FormField {
    /// Creates a text input field.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::FormField;
    ///
    /// let field = FormField::text("email", "Email Address");
    /// assert_eq!(field.id(), "email");
    /// assert_eq!(field.label(), "Email Address");
    /// ```
    pub fn text(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            kind: FormFieldKind::Text,
        }
    }

    /// Creates a text input field with placeholder text.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{FormField, FormFieldKind};
    ///
    /// let field = FormField::text_with_placeholder("email", "Email", "user@example.com");
    /// assert_eq!(field.id(), "email");
    /// assert!(matches!(field.kind(), FormFieldKind::TextWithPlaceholder(_)));
    /// ```
    pub fn text_with_placeholder(
        id: impl Into<String>,
        label: impl Into<String>,
        placeholder: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            kind: FormFieldKind::TextWithPlaceholder(placeholder.into()),
        }
    }

    /// Creates a checkbox field.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::FormField;
    ///
    /// let field = FormField::checkbox("agree", "I agree");
    /// assert_eq!(field.id(), "agree");
    /// ```
    pub fn checkbox(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            kind: FormFieldKind::Checkbox,
        }
    }

    /// Creates a select dropdown field.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{FormField, FormFieldKind};
    ///
    /// let field = FormField::select("color", "Favorite Color", vec!["Red", "Green", "Blue"]);
    /// assert_eq!(field.id(), "color");
    /// assert!(matches!(field.kind(), FormFieldKind::Select(_)));
    /// ```
    pub fn select<S: Into<String>>(
        id: impl Into<String>,
        label: impl Into<String>,
        options: Vec<S>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            kind: FormFieldKind::Select(options.into_iter().map(Into::into).collect()),
        }
    }

    /// Returns the field ID.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::FormField;
    ///
    /// let field = FormField::text("username", "Username");
    /// assert_eq!(field.id(), "username");
    /// ```
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the field label.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::FormField;
    ///
    /// let field = FormField::text("name", "Full Name");
    /// assert_eq!(field.label(), "Full Name");
    /// ```
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns the field kind.
    pub fn kind(&self) -> &FormFieldKind {
        &self.kind
    }
}
