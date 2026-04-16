use std::fs;
use std::path::Path;

struct Metric {
    name: &'static str,
    value: String,
    target: String,
    passed: bool,
}

pub fn run(root: &Path) {
    println!("SCORECARD");
    println!("{}", "=".repeat(70));

    let src = root.join("src");
    let mut metrics = Vec::new();

    // 1. Files over 1000 lines
    let over_limit = count_files_over_limit(&src, 1000);
    metrics.push(Metric {
        name: "Files over 1000 lines",
        value: format!("{}", over_limit),
        target: "0".to_string(),
        passed: over_limit == 0,
    });

    // 2. Accessor symmetry gaps
    let accessor_gaps = count_accessor_gaps(&src);
    metrics.push(Metric {
        name: "Accessor symmetry gaps",
        value: format!("{}", accessor_gaps),
        target: "0".to_string(),
        passed: accessor_gaps == 0,
    });

    // 3. Doc test coverage
    let (with_doctest, total_pub) = count_doctest_coverage(&src);
    let pct = if total_pub > 0 {
        with_doctest as f64 / total_pub as f64 * 100.0
    } else {
        0.0
    };
    metrics.push(Metric {
        name: "Doc test coverage",
        value: format!("{:.1}% ({}/{})", pct, with_doctest, total_pub),
        target: "100%".to_string(),
        passed: with_doctest == total_pub,
    });

    // 4. Standard trait derives
    let (debug, clone, default, partial_eq, total_components) = count_trait_derives(&src);
    metrics.push(Metric {
        name: "  Debug on State types",
        value: format!("{}/{}", debug, total_components),
        target: format!("{}/{}", total_components, total_components),
        passed: debug == total_components,
    });
    metrics.push(Metric {
        name: "  Clone on State types",
        value: format!("{}/{}", clone, total_components),
        target: format!("{}/{}", total_components, total_components),
        passed: clone == total_components,
    });
    metrics.push(Metric {
        name: "  Default on State types",
        value: format!("{}/{}", default, total_components),
        target: format!("{}/{}", total_components, total_components),
        passed: default == total_components,
    });
    metrics.push(Metric {
        name: "  PartialEq on State types",
        value: format!("{}/{}", partial_eq, total_components),
        target: format!("{}/{}", total_components, total_components),
        passed: partial_eq == total_components,
    });

    // 5. Unsafe blocks
    let unsafe_count = count_unsafe_blocks(&src);
    metrics.push(Metric {
        name: "Unsafe blocks",
        value: format!("{}", unsafe_count),
        target: "0".to_string(),
        passed: unsafe_count == 0,
    });

    // 6. Clippy suppressions
    let clippy_count = count_clippy_suppressions(&src);
    metrics.push(Metric {
        name: "Clippy suppressions",
        value: format!("{}", clippy_count),
        target: "0".to_string(),
        passed: clippy_count == 0,
    });

    // Print scorecard
    println!();
    let mut pass_count = 0;
    let mut fail_count = 0;

    for metric in &metrics {
        let status = if metric.passed {
            pass_count += 1;
            "PASS"
        } else {
            fail_count += 1;
            "FAIL"
        };
        println!(
            "  {:<30} {:>20}  (target: {:<10}) {}",
            metric.name, metric.value, metric.target, status
        );
    }

    println!();
    println!(
        "  Result: {}/{} checks passing",
        pass_count,
        pass_count + fail_count
    );
    if fail_count == 0 {
        println!("  ALL CHECKS PASSING");
    } else {
        println!("  {} checks failing", fail_count);
    }
}

fn count_files_over_limit(src: &Path, limit: usize) -> usize {
    let mut count = 0;
    walk_rs_files(src, &mut |_path, content| {
        if content.lines().count() > limit {
            count += 1;
        }
    });
    count
}

fn count_accessor_gaps(src: &Path) -> usize {
    let component_dir = src.join("component");
    let mut total_gaps = 0;

    let Ok(entries) = fs::read_dir(&component_dir) else {
        return 0;
    };

    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        let mod_file = entry.path().join("mod.rs");
        if !mod_file.exists() {
            continue;
        }
        let content = fs::read_to_string(&mod_file).unwrap_or_default();
        let filtered = non_test_content(&content);
        let setters = extract_method_names(&filtered, "pub fn set_");
        let getters = extract_getter_methods(&filtered);

        for setter in &setters {
            let getter_name = setter.strip_prefix("set_").unwrap_or(setter);
            let has_getter = getters
                .iter()
                .any(|g| g == getter_name || g == &format!("is_{}", getter_name));
            if !has_getter {
                total_gaps += 1;
            }
        }
    }

    total_gaps
}

fn count_doctest_coverage(src: &Path) -> (usize, usize) {
    let component_dir = src.join("component");
    let mut total_with = 0;
    let mut total_pub = 0;

    let Ok(entries) = fs::read_dir(&component_dir) else {
        return (0, 0);
    };

    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        let mod_file = entry.path().join("mod.rs");
        if !mod_file.exists() {
            continue;
        }
        let content = fs::read_to_string(&mod_file).unwrap_or_default();
        let (pub_fns, with_doctest) = count_doctest_in_content(&content);
        total_pub += pub_fns;
        total_with += with_doctest;
    }

    (total_with, total_pub)
}

fn count_trait_derives(src: &Path) -> (usize, usize, usize, usize, usize) {
    let component_dir = src.join("component");
    let mut debug = 0;
    let mut clone = 0;
    let mut default = 0;
    let mut partial_eq = 0;
    let mut total = 0;

    let Ok(entries) = fs::read_dir(&component_dir) else {
        return (0, 0, 0, 0, 0);
    };

    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        let mod_file = entry.path().join("mod.rs");
        if !mod_file.exists() {
            continue;
        }

        let all_content = read_all_rs_in_dir(&entry.path());
        let derives = extract_state_derives(&all_content);
        if derives.is_empty() {
            // No State type found — skip (e.g., context module)
            continue;
        }

        total += 1;
        if derives.contains(&"Debug".to_string()) {
            debug += 1;
        }
        if derives.contains(&"Clone".to_string()) {
            clone += 1;
        }
        if derives.contains(&"Default".to_string()) {
            default += 1;
        }
        if derives.contains(&"PartialEq".to_string()) {
            partial_eq += 1;
        }
    }

    (debug, clone, default, partial_eq, total)
}

fn count_unsafe_blocks(src: &Path) -> usize {
    let mut count = 0;
    walk_rs_files(src, &mut |_path, content| {
        for line in content.lines() {
            let trimmed = line.trim();
            if (trimmed.starts_with("unsafe ") || trimmed.contains(" unsafe "))
                && !trimmed.starts_with("//")
                && !trimmed.starts_with("///")
            {
                count += 1;
            }
        }
    });
    count
}

fn count_clippy_suppressions(src: &Path) -> usize {
    let mut count = 0;
    walk_rs_files(src, &mut |_path, content| {
        for line in content.lines() {
            if line.contains("#[allow(clippy::") {
                count += 1;
            }
        }
    });
    count
}

// --- Helper functions ---

fn walk_rs_files(dir: &Path, callback: &mut dyn FnMut(&Path, &str)) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_rs_files(&path, callback);
        } else if path.extension().is_some_and(|e| e == "rs") {
            if let Ok(content) = fs::read_to_string(&path) {
                callback(&path, &content);
            }
        }
    }
}

fn read_all_rs_in_dir(dir: &Path) -> String {
    let mut content = String::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return content;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|e| e == "rs") {
            if let Ok(c) = fs::read_to_string(&path) {
                content.push_str(&c);
                content.push('\n');
            }
        }
    }
    content
}

fn non_test_content(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();
    let mut i = 0;
    let mut brace_depth = 0;
    let mut in_test_block = false;

    while i < lines.len() {
        let trimmed = lines[i].trim();

        if in_test_block {
            for ch in trimmed.chars() {
                match ch {
                    '{' => brace_depth += 1,
                    '}' => {
                        brace_depth -= 1;
                        if brace_depth == 0 {
                            in_test_block = false;
                        }
                    }
                    _ => {}
                }
            }
            i += 1;
            continue;
        }

        if trimmed == "#[cfg(test)]" {
            let mut next = i + 1;
            while next < lines.len() && lines[next].trim().is_empty() {
                next += 1;
            }
            if next < lines.len() {
                let next_trimmed = lines[next].trim();
                if next_trimmed.starts_with("mod ") && next_trimmed.ends_with(';') {
                    result.push(lines[i]);
                    i += 1;
                    continue;
                }
                in_test_block = true;
                for ch in next_trimmed.chars() {
                    match ch {
                        '{' => brace_depth += 1,
                        '}' => brace_depth -= 1,
                        _ => {}
                    }
                }
                if brace_depth == 0 {
                    in_test_block = false;
                }
                i += 1;
                continue;
            }
        }

        result.push(lines[i]);
        i += 1;
    }

    result.join("\n")
}

fn extract_method_names(content: &str, prefix: &str) -> Vec<String> {
    let mut names = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix(prefix) {
            if let Some(name_end) = rest.find(['(', '<', ' ']) {
                names.push(format!(
                    "{}{}",
                    prefix.strip_prefix("pub fn ").unwrap_or(prefix),
                    &rest[..name_end]
                ));
            }
        }
    }
    names
}

fn extract_getter_methods(content: &str) -> Vec<String> {
    let mut names = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if (trimmed.starts_with("pub fn ") || trimmed.starts_with("pub const fn "))
            && !trimmed.contains("pub(crate)")
            && !trimmed.contains("pub(super)")
            && trimmed.contains("&self")
            && !trimmed.contains("&mut self")
        {
            if let Some(name) = extract_fn_name(trimmed) {
                if !name.starts_with("with_") && !name.starts_with("set_") {
                    names.push(name);
                }
            }
        }
    }
    names
}

fn extract_fn_name(line: &str) -> Option<String> {
    let fn_pos = line.find("fn ")?;
    let after_fn = &line[fn_pos + 3..];
    let name_end = after_fn.find(|c: char| !c.is_alphanumeric() && c != '_')?;
    Some(after_fn[..name_end].to_string())
}

fn is_public_fn(line: &str) -> bool {
    (line.starts_with("pub fn ")
        || line.starts_with("pub const fn ")
        || line.starts_with("pub async fn "))
        && !line.contains("pub(crate)")
        && !line.contains("pub(super)")
}

fn count_doctest_in_content(content: &str) -> (usize, usize) {
    let filtered = non_test_content(content);
    let lines: Vec<&str> = filtered.lines().collect();
    let mut pub_fn_count = 0;
    let mut with_doctest = 0;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if is_public_fn(trimmed) {
            pub_fn_count += 1;
            if has_doc_test_above(&lines, i) {
                with_doctest += 1;
            }
        }
    }

    (pub_fn_count, with_doctest)
}

fn has_doc_test_above(lines: &[&str], fn_line: usize) -> bool {
    let mut doc_lines = Vec::new();
    let mut j = fn_line;

    while j > 0 {
        j -= 1;
        let trimmed = lines[j].trim();
        if trimmed.starts_with("///") {
            doc_lines.push(trimmed);
        } else if trimmed.starts_with("#[") {
            continue;
        } else {
            break;
        }
    }

    doc_lines.iter().any(|line| line.contains("```"))
}

fn extract_state_derives(content: &str) -> Vec<String> {
    let mut derives = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    let mut state_type_name: Option<String> = None;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("pub struct ") && trimmed.contains("State") {
            if let Some(name) = trimmed
                .strip_prefix("pub struct ")
                .and_then(|s| s.split(|c: char| !c.is_alphanumeric() && c != '_').next())
            {
                state_type_name = Some(name.to_string());
            }

            let mut attr_block = String::new();
            let mut j = i;
            while j > 0 {
                j -= 1;
                let prev = lines[j].trim();
                if prev.starts_with("///") || prev.is_empty() {
                    continue;
                }
                if prev.starts_with("#[")
                    || prev.starts_with("//!")
                    || prev.ends_with(')')
                    || prev.ends_with(")]")
                    || prev.ends_with(',')
                    || prev.starts_with("derive(")
                    || prev.starts_with("feature")
                {
                    attr_block.push(' ');
                    attr_block.push_str(prev);
                } else {
                    break;
                }
            }

            for trait_name in ["Debug", "Clone", "Default", "PartialEq"] {
                if attr_block.contains(trait_name) {
                    derives.push(trait_name.to_string());
                }
            }
            break;
        }
    }

    if let Some(ref state_name) = state_type_name {
        // Trait paths to recognize. The leading `impl` token may be followed by
        // a generic parameter list (e.g. `impl<T: TableRow + PartialEq>`), so we
        // scan impl lines for the `{TraitPath} for {StateName}` signature rather
        // than anchoring on `impl {TraitPath} for`.
        let trait_patterns: &[(&str, &[&str])] = &[
            (
                "Debug",
                &["Debug for", "std::fmt::Debug for", "core::fmt::Debug for"],
            ),
            ("Clone", &["Clone for", "std::clone::Clone for"]),
            ("Default", &["Default for", "std::default::Default for"]),
            (
                "PartialEq",
                &[
                    "PartialEq for",
                    "std::cmp::PartialEq for",
                    "core::cmp::PartialEq for",
                ],
            ),
        ];

        for (trait_name, patterns) in trait_patterns {
            if derives.iter().any(|d| d == *trait_name) {
                continue;
            }
            if has_manual_trait_impl(content, patterns, state_name) {
                derives.push(trait_name.to_string());
            }
        }
    }

    derives
}

/// Returns true if `content` contains a manual trait impl for `state_name`
/// matching any of the given trait path patterns (e.g. `"PartialEq for"`).
///
/// Handles both non-generic (`impl PartialEq for FooState`) and generic
/// (`impl<T: Clone> PartialEq for FooState<T>`) impls by scanning lines that
/// begin with `impl` and checking for the `{trait} for {state_name}` signature
/// anywhere on that line. A word-boundary check prevents `FooState` from
/// matching `FooStateExt`.
fn has_manual_trait_impl(content: &str, trait_patterns: &[&str], state_name: &str) -> bool {
    for line in content.lines() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("impl") {
            continue;
        }
        // After the `impl` keyword we expect either whitespace (`impl Foo`) or
        // a generic parameter list (`impl<T> Foo`). Anything else (e.g. an
        // identifier like `implementation`) is not an impl block.
        let after_impl = &trimmed[4..];
        if !after_impl.starts_with(' ') && !after_impl.starts_with('<') {
            continue;
        }
        for pattern in trait_patterns {
            let needle = format!("{pattern} {state_name}");
            if let Some(idx) = trimmed.find(&needle) {
                let end = idx + needle.len();
                let next_char = trimmed[end..].chars().next();
                let is_boundary = match next_char {
                    None => true,
                    Some(c) => !c.is_alphanumeric() && c != '_',
                };
                if is_boundary {
                    return true;
                }
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_derive_list_from_attribute() {
        let content = "#[derive(Clone, Debug, PartialEq)]\npub struct FooState { x: u8 }";
        let derives = extract_state_derives(content);
        assert!(derives.contains(&"Clone".to_string()));
        assert!(derives.contains(&"Debug".to_string()));
        assert!(derives.contains(&"PartialEq".to_string()));
        assert!(!derives.contains(&"Default".to_string()));
    }

    #[test]
    fn detects_manual_non_generic_impl() {
        let content =
            "pub struct FooState { x: u8 }\nimpl Default for FooState { fn default() -> Self { unimplemented!() } }";
        let derives = extract_state_derives(content);
        assert!(derives.contains(&"Default".to_string()));
    }

    #[test]
    fn detects_manual_generic_impl() {
        // Regression test: audit previously missed generic manual impls.
        let content = "pub struct FooState<T: Clone> { x: T }\nimpl<T: Clone + PartialEq> PartialEq for FooState<T> { fn eq(&self, other: &Self) -> bool { true } }";
        let derives = extract_state_derives(content);
        assert!(derives.contains(&"PartialEq".to_string()));
    }

    #[test]
    fn detects_manual_generic_default_impl() {
        let content = "pub struct FooState<T: Clone> { x: T }\nimpl<T: Clone> Default for FooState<T> { fn default() -> Self { unimplemented!() } }";
        let derives = extract_state_derives(content);
        assert!(derives.contains(&"Default".to_string()));
    }

    #[test]
    fn detects_fully_qualified_path() {
        let content =
            "pub struct FooState;\nimpl std::cmp::PartialEq for FooState { fn eq(&self, _other: &Self) -> bool { true } }";
        let derives = extract_state_derives(content);
        assert!(derives.contains(&"PartialEq".to_string()));
    }

    #[test]
    fn does_not_match_state_with_extended_name() {
        // `FooState` should not match `FooStateExt` due to word boundary.
        let content = "pub struct FooState;\nimpl PartialEq for FooStateExt { fn eq(&self, _other: &Self) -> bool { true } }";
        let derives = extract_state_derives(content);
        assert!(!derives.contains(&"PartialEq".to_string()));
    }

    #[test]
    fn ignores_impl_like_words_in_comments() {
        // Lines like `// implementation notes` should not trigger detection.
        let content =
            "pub struct FooState;\n// implementation of PartialEq for FooState would be...";
        let derives = extract_state_derives(content);
        assert!(!derives.contains(&"PartialEq".to_string()));
    }
}
