//! Keyword-based syntax highlighting for common programming languages.
//!
//! Provides a [`Language`] enum for language selection and a
//! [`highlight_line`] function that produces styled [`Span`]s for a
//! single source line. The highlighter is intentionally simple: it
//! tokenises by whitespace and punctuation boundaries and applies
//! keyword/literal/comment colouring without building an AST.

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;

/// Supported languages for syntax highlighting.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum Language {
    /// Rust
    Rust,
    /// Python
    Python,
    /// JavaScript
    #[cfg_attr(feature = "serialization", serde(rename = "javascript"))]
    JavaScript,
    /// TypeScript
    #[cfg_attr(feature = "serialization", serde(rename = "typescript"))]
    TypeScript,
    /// Go
    Go,
    /// Shell / Bash
    Shell,
    /// JSON
    Json,
    /// YAML
    Yaml,
    /// TOML
    Toml,
    /// SQL
    Sql,
    /// HCL (HashiCorp Configuration Language — Terraform, etc.)
    Hcl,
    /// Plain text (no highlighting)
    #[default]
    Plain,
}

impl Language {
    /// Returns the display name for the language.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::code_block::highlight::Language;
    ///
    /// assert_eq!(Language::Rust.name(), "Rust");
    /// assert_eq!(Language::Plain.name(), "Plain");
    /// ```
    pub fn name(&self) -> &'static str {
        match self {
            Language::Rust => "Rust",
            Language::Python => "Python",
            Language::JavaScript => "JavaScript",
            Language::TypeScript => "TypeScript",
            Language::Go => "Go",
            Language::Shell => "Shell",
            Language::Json => "JSON",
            Language::Yaml => "YAML",
            Language::Toml => "TOML",
            Language::Sql => "SQL",
            Language::Hcl => "HCL",
            Language::Plain => "Plain",
        }
    }

    /// Returns the keyword list for the language.
    fn keywords(&self) -> &'static [&'static str] {
        match self {
            Language::Rust => &[
                "fn", "let", "mut", "const", "static", "struct", "enum", "impl", "trait", "type",
                "pub", "use", "mod", "crate", "self", "super", "as", "in", "for", "while", "loop",
                "if", "else", "match", "return", "break", "continue", "where", "async", "await",
                "move", "ref", "dyn", "unsafe", "extern",
            ],
            Language::Python => &[
                "def", "class", "import", "from", "return", "if", "elif", "else", "for", "while",
                "try", "except", "finally", "with", "as", "yield", "lambda", "pass", "raise",
                "break", "continue", "and", "or", "not", "in", "is", "global", "nonlocal",
                "assert", "async", "await",
            ],
            Language::JavaScript => &[
                "function",
                "const",
                "let",
                "var",
                "return",
                "if",
                "else",
                "for",
                "while",
                "do",
                "switch",
                "case",
                "break",
                "continue",
                "class",
                "extends",
                "new",
                "this",
                "import",
                "export",
                "default",
                "from",
                "try",
                "catch",
                "finally",
                "throw",
                "async",
                "await",
                "yield",
                "typeof",
                "instanceof",
                "in",
                "of",
                "delete",
                "void",
            ],
            Language::TypeScript => &[
                "function",
                "const",
                "let",
                "var",
                "return",
                "if",
                "else",
                "for",
                "while",
                "do",
                "switch",
                "case",
                "break",
                "continue",
                "class",
                "extends",
                "new",
                "this",
                "import",
                "export",
                "default",
                "from",
                "try",
                "catch",
                "finally",
                "throw",
                "async",
                "await",
                "yield",
                "typeof",
                "instanceof",
                "in",
                "of",
                "delete",
                "void",
                "interface",
                "type",
                "enum",
                "namespace",
                "declare",
                "implements",
                "abstract",
                "readonly",
                "as",
                "keyof",
                "infer",
            ],
            Language::Go => &[
                "func",
                "package",
                "import",
                "return",
                "if",
                "else",
                "for",
                "range",
                "switch",
                "case",
                "default",
                "break",
                "continue",
                "go",
                "defer",
                "select",
                "chan",
                "map",
                "struct",
                "interface",
                "type",
                "const",
                "var",
                "fallthrough",
                "goto",
            ],
            Language::Shell => &[
                "if", "then", "else", "elif", "fi", "for", "while", "do", "done", "case", "esac",
                "function", "return", "exit", "echo", "export", "local", "source", "alias",
                "unset", "readonly", "shift", "set", "in",
            ],
            Language::Json => &[],
            Language::Yaml => &[],
            Language::Toml => &[],
            Language::Sql => &[
                "SELECT",
                "FROM",
                "WHERE",
                "INSERT",
                "INTO",
                "VALUES",
                "UPDATE",
                "SET",
                "DELETE",
                "CREATE",
                "TABLE",
                "DROP",
                "ALTER",
                "INDEX",
                "JOIN",
                "INNER",
                "LEFT",
                "RIGHT",
                "OUTER",
                "ON",
                "AND",
                "OR",
                "NOT",
                "NULL",
                "IS",
                "IN",
                "LIKE",
                "BETWEEN",
                "ORDER",
                "BY",
                "GROUP",
                "HAVING",
                "LIMIT",
                "OFFSET",
                "AS",
                "DISTINCT",
                "UNION",
                "ALL",
                "EXISTS",
                "CASE",
                "WHEN",
                "THEN",
                "ELSE",
                "END",
                "ASC",
                "DESC",
                "PRIMARY",
                "KEY",
                "FOREIGN",
                "REFERENCES",
                "CONSTRAINT",
                "DEFAULT",
                "BEGIN",
                "COMMIT",
                "ROLLBACK",
                "GRANT",
                "REVOKE",
            ],
            Language::Hcl => &[
                "resource",
                "data",
                "variable",
                "output",
                "locals",
                "module",
                "provider",
                "terraform",
                "backend",
                "required_providers",
                "required_version",
                "for_each",
                "count",
                "depends_on",
                "lifecycle",
                "provisioner",
                "connection",
                "dynamic",
                "content",
                "for",
                "in",
                "if",
                "else",
                "endif",
            ],
            Language::Plain => &[],
        }
    }

    /// Returns the type/built-in keyword list for the language.
    fn type_keywords(&self) -> &'static [&'static str] {
        match self {
            Language::Rust => &[
                "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128",
                "usize", "f32", "f64", "bool", "char", "str", "String", "Vec", "Option", "Result",
                "Box", "Rc", "Arc", "Self", "Some", "None", "Ok", "Err",
            ],
            Language::Python => &[
                "True", "False", "None", "int", "float", "str", "bool", "list", "dict", "tuple",
                "set", "bytes", "type", "object", "range", "print", "len", "super", "self",
            ],
            Language::JavaScript => &[
                "true",
                "false",
                "null",
                "undefined",
                "NaN",
                "Infinity",
                "console",
                "Array",
                "Object",
                "String",
                "Number",
                "Boolean",
                "Promise",
                "Map",
                "Set",
                "Date",
                "RegExp",
                "Error",
                "Math",
                "JSON",
            ],
            Language::TypeScript => &[
                "true",
                "false",
                "null",
                "undefined",
                "NaN",
                "Infinity",
                "console",
                "Array",
                "Object",
                "String",
                "Number",
                "Boolean",
                "Promise",
                "Map",
                "Set",
                "Date",
                "RegExp",
                "Error",
                "Math",
                "JSON",
                "any",
                "number",
                "string",
                "boolean",
                "never",
                "void",
                "unknown",
                "symbol",
                "bigint",
                "object",
            ],
            Language::Go => &[
                "true",
                "false",
                "nil",
                "int",
                "int8",
                "int16",
                "int32",
                "int64",
                "uint",
                "uint8",
                "uint16",
                "uint32",
                "uint64",
                "float32",
                "float64",
                "complex64",
                "complex128",
                "string",
                "bool",
                "byte",
                "rune",
                "error",
                "any",
                "make",
                "len",
                "cap",
                "append",
                "copy",
                "close",
                "delete",
                "new",
                "panic",
                "recover",
                "print",
                "println",
                "iota",
            ],
            Language::Shell => &[
                "true", "false", "cd", "ls", "cp", "mv", "rm", "mkdir", "cat", "grep", "sed",
                "awk", "find", "sort", "head", "tail", "wc", "cut", "tr", "xargs", "tee", "chmod",
                "chown",
            ],
            Language::Sql => &[
                "INT",
                "INTEGER",
                "BIGINT",
                "SMALLINT",
                "FLOAT",
                "DOUBLE",
                "DECIMAL",
                "NUMERIC",
                "VARCHAR",
                "CHAR",
                "TEXT",
                "BLOB",
                "DATE",
                "TIMESTAMP",
                "BOOLEAN",
                "SERIAL",
                "COUNT",
                "SUM",
                "AVG",
                "MIN",
                "MAX",
            ],
            Language::Hcl => &[
                "string",
                "number",
                "bool",
                "list",
                "map",
                "set",
                "object",
                "tuple",
                "any",
                "true",
                "false",
                "null",
                "each",
                "self",
                "var",
                "local",
                "path",
                "tostring",
                "tonumber",
                "tobool",
                "tolist",
                "toset",
                "tomap",
                "length",
                "lookup",
                "merge",
                "concat",
                "join",
                "split",
                "replace",
                "format",
                "formatlist",
                "file",
                "templatefile",
                "jsonencode",
                "jsondecode",
                "yamlencode",
                "yamldecode",
                "base64encode",
                "base64decode",
                "cidrsubnet",
                "cidrhost",
            ],
            _ => &[],
        }
    }

    /// Returns the single-line comment prefix for the language, if any.
    fn line_comment_prefix(&self) -> Option<&'static str> {
        match self {
            Language::Rust | Language::Go | Language::JavaScript | Language::TypeScript => {
                Some("//")
            }
            Language::Python
            | Language::Shell
            | Language::Yaml
            | Language::Toml
            | Language::Hcl => Some("#"),
            Language::Sql => Some("--"),
            Language::Json | Language::Plain => None,
        }
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

// =============================================================================
// Syntax Highlighting Colors
// =============================================================================

/// Style for language keywords (e.g., `fn`, `let`, `if`).
fn keyword_style() -> Style {
    Style::default()
        .fg(Color::Magenta)
        .add_modifier(Modifier::BOLD)
}

/// Style for type names and built-in constants.
fn type_style() -> Style {
    Style::default().fg(Color::Cyan)
}

/// Style for string literals.
fn string_style() -> Style {
    Style::default().fg(Color::Green)
}

/// Style for numeric literals.
fn number_style() -> Style {
    Style::default().fg(Color::Yellow)
}

/// Style for comments.
fn comment_style() -> Style {
    Style::default().fg(Color::DarkGray)
}

/// Style for punctuation and operators.
fn punctuation_style() -> Style {
    Style::default().fg(Color::White)
}

/// Style for regular code text.
fn default_style() -> Style {
    Style::default().fg(Color::White)
}

// =============================================================================
// Tokenizer
// =============================================================================

/// A token extracted from a source line.
#[derive(Debug, PartialEq)]
enum TokenKind {
    Word,
    Whitespace,
    String,
    Number,
    Punctuation,
}

struct Token {
    kind: TokenKind,
    text: String,
}

/// Tokenizes a line of source code into basic tokens.
///
/// This is a simple character-by-character scanner that handles strings
/// (double and single quoted), numbers, words, whitespace, and
/// punctuation.
fn tokenize(line: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let ch = chars[i];

        // Whitespace
        if ch.is_whitespace() {
            let start = i;
            while i < len && chars[i].is_whitespace() {
                i += 1;
            }
            tokens.push(Token {
                kind: TokenKind::Whitespace,
                text: chars[start..i].iter().collect(),
            });
            continue;
        }

        // String literals (double or single quoted)
        if ch == '"' || ch == '\'' {
            let quote = ch;
            let start = i;
            i += 1;
            while i < len && chars[i] != quote {
                if chars[i] == '\\' {
                    i += 1; // skip escaped character
                }
                i += 1;
            }
            if i < len {
                i += 1; // closing quote
            }
            tokens.push(Token {
                kind: TokenKind::String,
                text: chars[start..i].iter().collect(),
            });
            continue;
        }

        // Numbers (including hex, binary, float with dot)
        if ch.is_ascii_digit() || (ch == '.' && i + 1 < len && chars[i + 1].is_ascii_digit()) {
            let start = i;
            // Handle 0x, 0b, 0o prefixes
            if ch == '0' && i + 1 < len {
                let next = chars[i + 1];
                if next == 'x' || next == 'X' || next == 'b' || next == 'B' || next == 'o' {
                    i += 2;
                    while i < len && (chars[i].is_ascii_hexdigit() || chars[i] == '_') {
                        i += 1;
                    }
                    tokens.push(Token {
                        kind: TokenKind::Number,
                        text: chars[start..i].iter().collect(),
                    });
                    continue;
                }
            }
            while i < len && (chars[i].is_ascii_digit() || chars[i] == '.' || chars[i] == '_') {
                i += 1;
            }
            // Handle suffixes like i32, u64, f64 etc.
            if i < len && (chars[i] == 'e' || chars[i] == 'E') {
                i += 1;
                if i < len && (chars[i] == '+' || chars[i] == '-') {
                    i += 1;
                }
                while i < len && chars[i].is_ascii_digit() {
                    i += 1;
                }
            }
            tokens.push(Token {
                kind: TokenKind::Number,
                text: chars[start..i].iter().collect(),
            });
            continue;
        }

        // Words (identifiers, keywords)
        if ch.is_alphanumeric() || ch == '_' {
            let start = i;
            while i < len && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            // Also include ! for Rust macros like println!
            if i < len && chars[i] == '!' {
                i += 1;
            }
            tokens.push(Token {
                kind: TokenKind::Word,
                text: chars[start..i].iter().collect(),
            });
            continue;
        }

        // Punctuation and operators
        let start = i;
        i += 1;
        tokens.push(Token {
            kind: TokenKind::Punctuation,
            text: chars[start..i].iter().collect(),
        });
    }

    tokens
}

/// Highlights a single line of source code, returning styled spans.
///
/// Applies keyword-based syntax colouring for the given [`Language`].
/// For [`Language::Plain`], the line is returned as a single
/// unstyled span.
///
/// # Example
///
/// ```rust
/// use envision::component::code_block::highlight::{highlight_line, Language};
///
/// let spans = highlight_line("let x = 42;", &Language::Rust);
/// assert!(!spans.is_empty());
/// ```
pub fn highlight_line<'a>(line: &'a str, language: &Language) -> Vec<Span<'a>> {
    if *language == Language::Plain {
        return vec![Span::styled(line.to_string(), default_style())];
    }

    // Check for line comments first
    if let Some(prefix) = language.line_comment_prefix() {
        let trimmed = line.trim_start();
        if trimmed.starts_with(prefix) {
            return vec![Span::styled(line.to_string(), comment_style())];
        }
    }

    let tokens = tokenize(line);
    let keywords = language.keywords();
    let type_kws = language.type_keywords();

    let mut spans = Vec::with_capacity(tokens.len());

    for token in tokens {
        let style = match token.kind {
            TokenKind::String => string_style(),
            TokenKind::Number => number_style(),
            TokenKind::Whitespace => default_style(),
            TokenKind::Punctuation => punctuation_style(),
            TokenKind::Word => {
                let word_without_bang = token.text.trim_end_matches('!');
                if is_keyword(word_without_bang, keywords, language) {
                    keyword_style()
                } else if type_kws.contains(&word_without_bang) {
                    type_style()
                } else {
                    default_style()
                }
            }
        };
        spans.push(Span::styled(token.text, style));
    }

    spans
}

/// Checks if a word is a keyword, considering case sensitivity per language.
fn is_keyword(word: &str, keywords: &[&str], language: &Language) -> bool {
    match language {
        Language::Sql => {
            // SQL keywords are case-insensitive
            let upper = word.to_uppercase();
            keywords.iter().any(|kw| kw.to_uppercase() == upper)
        }
        _ => keywords.contains(&word),
    }
}

#[cfg(test)]
#[path = "highlight_tests.rs"]
mod tests;
