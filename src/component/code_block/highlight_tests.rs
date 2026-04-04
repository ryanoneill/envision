use super::*;

// =============================================================================
// Language::name
// =============================================================================

#[test]
fn test_language_name() {
    assert_eq!(Language::Rust.name(), "Rust");
    assert_eq!(Language::Python.name(), "Python");
    assert_eq!(Language::JavaScript.name(), "JavaScript");
    assert_eq!(Language::TypeScript.name(), "TypeScript");
    assert_eq!(Language::Go.name(), "Go");
    assert_eq!(Language::Shell.name(), "Shell");
    assert_eq!(Language::Json.name(), "JSON");
    assert_eq!(Language::Yaml.name(), "YAML");
    assert_eq!(Language::Toml.name(), "TOML");
    assert_eq!(Language::Sql.name(), "SQL");
    assert_eq!(Language::Plain.name(), "Plain");
}

#[test]
fn test_language_display() {
    assert_eq!(format!("{}", Language::Rust), "Rust");
    assert_eq!(format!("{}", Language::Python), "Python");
}

#[test]
fn test_language_default() {
    assert_eq!(Language::default(), Language::Plain);
}

#[test]
fn test_language_clone_eq() {
    let lang = Language::Rust;
    let cloned = lang.clone();
    assert_eq!(lang, cloned);
}

#[test]
fn test_language_debug() {
    let debug = format!("{:?}", Language::Rust);
    assert!(debug.contains("Rust"));
}

// =============================================================================
// Language keyword lists
// =============================================================================

#[test]
fn test_rust_keywords_not_empty() {
    assert!(!Language::Rust.keywords().is_empty());
}

#[test]
fn test_python_keywords_not_empty() {
    assert!(!Language::Python.keywords().is_empty());
}

#[test]
fn test_javascript_keywords_not_empty() {
    assert!(!Language::JavaScript.keywords().is_empty());
}

#[test]
fn test_typescript_keywords_not_empty() {
    assert!(!Language::TypeScript.keywords().is_empty());
}

#[test]
fn test_go_keywords_not_empty() {
    assert!(!Language::Go.keywords().is_empty());
}

#[test]
fn test_shell_keywords_not_empty() {
    assert!(!Language::Shell.keywords().is_empty());
}

#[test]
fn test_sql_keywords_not_empty() {
    assert!(!Language::Sql.keywords().is_empty());
}

#[test]
fn test_json_keywords_empty() {
    assert!(Language::Json.keywords().is_empty());
}

#[test]
fn test_yaml_keywords_empty() {
    assert!(Language::Yaml.keywords().is_empty());
}

#[test]
fn test_toml_keywords_empty() {
    assert!(Language::Toml.keywords().is_empty());
}

#[test]
fn test_plain_keywords_empty() {
    assert!(Language::Plain.keywords().is_empty());
}

// =============================================================================
// Type keywords
// =============================================================================

#[test]
fn test_rust_type_keywords() {
    let types = Language::Rust.type_keywords();
    assert!(types.contains(&"String"));
    assert!(types.contains(&"Vec"));
    assert!(types.contains(&"Option"));
}

#[test]
fn test_python_type_keywords() {
    let types = Language::Python.type_keywords();
    assert!(types.contains(&"True"));
    assert!(types.contains(&"False"));
    assert!(types.contains(&"None"));
}

#[test]
fn test_go_type_keywords() {
    let types = Language::Go.type_keywords();
    assert!(types.contains(&"nil"));
    assert!(types.contains(&"int"));
}

// =============================================================================
// Line comment prefixes
// =============================================================================

#[test]
fn test_rust_comment_prefix() {
    assert_eq!(Language::Rust.line_comment_prefix(), Some("//"));
}

#[test]
fn test_python_comment_prefix() {
    assert_eq!(Language::Python.line_comment_prefix(), Some("#"));
}

#[test]
fn test_sql_comment_prefix() {
    assert_eq!(Language::Sql.line_comment_prefix(), Some("--"));
}

#[test]
fn test_json_no_comment_prefix() {
    assert_eq!(Language::Json.line_comment_prefix(), None);
}

#[test]
fn test_plain_no_comment_prefix() {
    assert_eq!(Language::Plain.line_comment_prefix(), None);
}

// =============================================================================
// Tokenizer
// =============================================================================

#[test]
fn test_tokenize_empty() {
    let tokens = tokenize("");
    assert!(tokens.is_empty());
}

#[test]
fn test_tokenize_whitespace() {
    let tokens = tokenize("   ");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::Whitespace);
    assert_eq!(tokens[0].text, "   ");
}

#[test]
fn test_tokenize_word() {
    let tokens = tokenize("hello");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::Word);
    assert_eq!(tokens[0].text, "hello");
}

#[test]
fn test_tokenize_number() {
    let tokens = tokenize("42");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::Number);
    assert_eq!(tokens[0].text, "42");
}

#[test]
fn test_tokenize_hex_number() {
    let tokens = tokenize("0xFF");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::Number);
    assert_eq!(tokens[0].text, "0xFF");
}

#[test]
fn test_tokenize_string_double() {
    let tokens = tokenize("\"hello\"");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::String);
    assert_eq!(tokens[0].text, "\"hello\"");
}

#[test]
fn test_tokenize_string_single() {
    let tokens = tokenize("'hello'");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::String);
    assert_eq!(tokens[0].text, "'hello'");
}

#[test]
fn test_tokenize_escaped_string() {
    let tokens = tokenize("\"he\\\"llo\"");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::String);
}

#[test]
fn test_tokenize_punctuation() {
    let tokens = tokenize("(");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::Punctuation);
}

#[test]
fn test_tokenize_rust_let() {
    let tokens = tokenize("let x = 42;");
    assert!(tokens.len() >= 5); // let, space, x, space, =, space, 42, ;
    assert_eq!(tokens[0].kind, TokenKind::Word);
    assert_eq!(tokens[0].text, "let");
}

#[test]
fn test_tokenize_rust_macro() {
    let tokens = tokenize("println!");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::Word);
    assert_eq!(tokens[0].text, "println!");
}

#[test]
fn test_tokenize_float() {
    let tokens = tokenize("3.14");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::Number);
    assert_eq!(tokens[0].text, "3.14");
}

#[test]
fn test_tokenize_underscored_number() {
    let tokens = tokenize("1_000_000");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::Number);
}

// =============================================================================
// highlight_line
// =============================================================================

#[test]
fn test_highlight_plain_line() {
    let spans = highlight_line("hello world", &Language::Plain);
    assert_eq!(spans.len(), 1);
}

#[test]
fn test_highlight_empty_line() {
    let spans = highlight_line("", &Language::Rust);
    assert!(spans.is_empty() || spans.iter().all(|s| s.content.is_empty()));
}

#[test]
fn test_highlight_rust_keyword() {
    let spans = highlight_line("fn main()", &Language::Rust);
    // The first span should be "fn" with keyword style
    assert!(!spans.is_empty());
    let fn_span = &spans[0];
    assert_eq!(fn_span.content.as_ref(), "fn");
    assert_eq!(fn_span.style, keyword_style());
}

#[test]
fn test_highlight_rust_type() {
    let spans = highlight_line("let x: String = String::new();", &Language::Rust);
    // Should contain a span for "String" with type style
    let type_spans: Vec<_> = spans
        .iter()
        .filter(|s| s.content.as_ref() == "String")
        .collect();
    assert!(!type_spans.is_empty());
    assert_eq!(type_spans[0].style, type_style());
}

#[test]
fn test_highlight_rust_string_literal() {
    let spans = highlight_line("let s = \"hello\";", &Language::Rust);
    let string_spans: Vec<_> = spans
        .iter()
        .filter(|s| s.content.contains("hello"))
        .collect();
    assert!(!string_spans.is_empty());
    assert_eq!(string_spans[0].style, string_style());
}

#[test]
fn test_highlight_rust_number() {
    let spans = highlight_line("let x = 42;", &Language::Rust);
    let num_spans: Vec<_> = spans
        .iter()
        .filter(|s| s.content.as_ref() == "42")
        .collect();
    assert!(!num_spans.is_empty());
    assert_eq!(num_spans[0].style, number_style());
}

#[test]
fn test_highlight_rust_comment() {
    let spans = highlight_line("// this is a comment", &Language::Rust);
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].style, comment_style());
}

#[test]
fn test_highlight_python_keyword() {
    let spans = highlight_line("def foo():", &Language::Python);
    let def_span = &spans[0];
    assert_eq!(def_span.content.as_ref(), "def");
    assert_eq!(def_span.style, keyword_style());
}

#[test]
fn test_highlight_python_comment() {
    let spans = highlight_line("# comment", &Language::Python);
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].style, comment_style());
}

#[test]
fn test_highlight_python_type() {
    let spans = highlight_line("x = True", &Language::Python);
    let true_spans: Vec<_> = spans
        .iter()
        .filter(|s| s.content.as_ref() == "True")
        .collect();
    assert!(!true_spans.is_empty());
    assert_eq!(true_spans[0].style, type_style());
}

#[test]
fn test_highlight_javascript_keyword() {
    let spans = highlight_line("const x = 1;", &Language::JavaScript);
    let const_span = &spans[0];
    assert_eq!(const_span.content.as_ref(), "const");
    assert_eq!(const_span.style, keyword_style());
}

#[test]
fn test_highlight_typescript_extra_keyword() {
    let spans = highlight_line("interface Foo {}", &Language::TypeScript);
    let iface_span = &spans[0];
    assert_eq!(iface_span.content.as_ref(), "interface");
    assert_eq!(iface_span.style, keyword_style());
}

#[test]
fn test_highlight_go_keyword() {
    let spans = highlight_line("func main() {", &Language::Go);
    let func_span = &spans[0];
    assert_eq!(func_span.content.as_ref(), "func");
    assert_eq!(func_span.style, keyword_style());
}

#[test]
fn test_highlight_shell_keyword() {
    let spans = highlight_line("if [ -f file ]; then", &Language::Shell);
    let if_span = &spans[0];
    assert_eq!(if_span.content.as_ref(), "if");
    assert_eq!(if_span.style, keyword_style());
}

#[test]
fn test_highlight_shell_comment() {
    let spans = highlight_line("# shell comment", &Language::Shell);
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].style, comment_style());
}

#[test]
fn test_highlight_sql_keyword() {
    let spans = highlight_line("SELECT * FROM users;", &Language::Sql);
    let select_span = &spans[0];
    assert_eq!(select_span.content.as_ref(), "SELECT");
    assert_eq!(select_span.style, keyword_style());
}

#[test]
fn test_highlight_sql_case_insensitive() {
    let spans = highlight_line("select * from users;", &Language::Sql);
    let select_span = &spans[0];
    assert_eq!(select_span.content.as_ref(), "select");
    assert_eq!(select_span.style, keyword_style());
}

#[test]
fn test_highlight_sql_comment() {
    let spans = highlight_line("-- SQL comment", &Language::Sql);
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].style, comment_style());
}

#[test]
fn test_highlight_json_string() {
    let spans = highlight_line("\"key\": \"value\"", &Language::Json);
    let string_spans: Vec<_> = spans.iter().filter(|s| s.style == string_style()).collect();
    assert!(string_spans.len() >= 2);
}

#[test]
fn test_highlight_yaml_comment() {
    let spans = highlight_line("# yaml comment", &Language::Yaml);
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].style, comment_style());
}

#[test]
fn test_highlight_toml_comment() {
    let spans = highlight_line("# toml comment", &Language::Toml);
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].style, comment_style());
}

#[test]
fn test_highlight_indented_comment() {
    let spans = highlight_line("    // indented comment", &Language::Rust);
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].style, comment_style());
}

#[test]
fn test_highlight_multiple_keywords() {
    let spans = highlight_line("pub fn foo() {", &Language::Rust);
    let kw_spans: Vec<_> = spans
        .iter()
        .filter(|s| s.style == keyword_style())
        .collect();
    assert_eq!(kw_spans.len(), 2); // pub, fn
}

#[test]
fn test_highlight_mixed_content() {
    let spans = highlight_line("let x = \"hello\";", &Language::Rust);
    // Should have keyword, whitespace, word, whitespace, punct, whitespace, string, punct
    assert!(spans.len() >= 5);
}

#[test]
fn test_highlight_preserves_full_line() {
    let line = "fn main() { let x = 42; }";
    let spans = highlight_line(line, &Language::Rust);
    let reconstructed: String = spans.iter().map(|s| s.content.as_ref()).collect();
    assert_eq!(reconstructed, line);
}

#[test]
fn test_highlight_preserves_whitespace() {
    let line = "    fn foo() {";
    let spans = highlight_line(line, &Language::Rust);
    let reconstructed: String = spans.iter().map(|s| s.content.as_ref()).collect();
    assert_eq!(reconstructed, line);
}

// =============================================================================
// HCL highlighting
// =============================================================================

#[test]
fn test_hcl_language_name() {
    assert_eq!(Language::Hcl.name(), "HCL");
}

#[test]
fn test_hcl_keywords_highlighted() {
    let spans = highlight_line("resource \"aws_instance\" \"web\" {", &Language::Hcl);
    assert!(!spans.is_empty());
    // "resource" should be a keyword
    let first_word = spans.iter().find(|s| s.content.as_ref() == "resource");
    assert!(first_word.is_some(), "resource should appear as a span");
    assert_eq!(
        first_word.unwrap().style.fg,
        Some(Color::Magenta),
        "resource should be keyword-colored"
    );
}

#[test]
fn test_hcl_type_keywords() {
    let spans = highlight_line("  type = string", &Language::Hcl);
    let type_span = spans.iter().find(|s| s.content.as_ref() == "string");
    assert!(type_span.is_some());
    assert_eq!(type_span.unwrap().style.fg, Some(Color::Cyan));
}

#[test]
fn test_hcl_comments() {
    let spans = highlight_line("# This is a comment", &Language::Hcl);
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].style.fg, Some(Color::DarkGray));
}

#[test]
fn test_hcl_preserves_content() {
    let line = "  ami           = \"ami-0c55b159cbfafe1f0\"";
    let spans = highlight_line(line, &Language::Hcl);
    let reconstructed: String = spans.iter().map(|s| s.content.as_ref()).collect();
    assert_eq!(reconstructed, line);
}
