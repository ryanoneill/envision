use regex::Regex;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

struct ComponentInfo {
    has_focusable: bool,
    has_toggleable: bool,
    has_disabled: bool,
    has_visible: bool,
    builder_methods: Vec<String>,
    setter_methods: Vec<String>,
    getter_methods: Vec<String>,
    pub_fn_count: usize,
    pub_fn_with_doctest: usize,
    unit_test_count: usize,
    snapshot_test_count: usize,
    public_method_names: Vec<String>,
}

pub fn run(root: &Path) {
    println!("CODE ANALYSIS");
    println!("{}", "-".repeat(70));

    let src = root.join("src");

    let components = analyze_components(&src);
    print_component_summary(&components);
    print_trait_summary(&components);
    print_builder_summary(&components);
    print_accessor_symmetry(&components);
    print_doctest_summary(&components);
    print_test_summary(&components);
    print_naming_patterns(&components);
    print_quality_checks(&src, root);
    print_reexports(&src);
}

fn analyze_components(src: &Path) -> BTreeMap<String, ComponentInfo> {
    let component_dir = src.join("component");
    let mut components = BTreeMap::new();

    let Ok(entries) = fs::read_dir(&component_dir) else {
        return components;
    };

    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let mod_file = entry.path().join("mod.rs");
        if !mod_file.exists() {
            continue;
        }

        let info = analyze_single_component(&entry.path());
        components.insert(name, info);
    }

    components
}

fn analyze_single_component(dir: &Path) -> ComponentInfo {
    let all_content = read_all_rs_in_dir(dir);
    let mod_content = fs::read_to_string(dir.join("mod.rs")).unwrap_or_default();

    let focusable_re = Regex::new(r"impl\b[^{{]*\bFocusable\b\s+for\b").unwrap();
    let toggleable_re = Regex::new(r"impl\b[^{{]*\bToggleable\b\s+for\b").unwrap();

    let has_focusable = focusable_re.is_match(&all_content);
    let has_toggleable = toggleable_re.is_match(&all_content);
    let has_disabled =
        mod_content.contains("fn is_disabled") && mod_content.contains("fn set_disabled");
    let has_visible =
        mod_content.contains("fn is_visible") && mod_content.contains("fn set_visible");

    let builder_methods = extract_method_names(&mod_content, "pub fn with_");
    let setter_methods = extract_method_names(&mod_content, "pub fn set_");
    let getter_methods = extract_getter_methods(&mod_content);
    let public_method_names = extract_all_public_fn_names(&mod_content);
    let (pub_fn_count, pub_fn_with_doctest) = count_doctest_coverage(&mod_content);

    let test_content = read_test_files(dir);
    let unit_test_count = test_content.matches("#[test]").count()
        + test_content.matches("#[tokio::test]").count();

    let snapshot_content =
        fs::read_to_string(dir.join("snapshot_tests.rs")).unwrap_or_default();
    let snapshot_test_count = snapshot_content.matches("#[test]").count();

    ComponentInfo {
        has_focusable,
        has_toggleable,
        has_disabled,
        has_visible,
        builder_methods,
        setter_methods,
        getter_methods,
        pub_fn_count,
        pub_fn_with_doctest,
        unit_test_count,
        snapshot_test_count,
        public_method_names,
    }
}

fn print_component_summary(components: &BTreeMap<String, ComponentInfo>) {
    println!("\nComponents found: {}", components.len());
    for name in components.keys() {
        println!("  {}", name);
    }
}

fn print_trait_summary(components: &BTreeMap<String, ComponentInfo>) {
    println!("\nTrait Implementations:");

    let focusable: Vec<_> = components
        .iter()
        .filter(|(_, i)| i.has_focusable)
        .map(|(n, _)| n.as_str())
        .collect();
    println!("  Focusable ({}):", focusable.len());
    for name in &focusable {
        println!("    {}", name);
    }

    let toggleable: Vec<_> = components
        .iter()
        .filter(|(_, i)| i.has_toggleable)
        .map(|(n, _)| n.as_str())
        .collect();
    println!("  Toggleable ({}):", toggleable.len());
    for name in &toggleable {
        println!("    {}", name);
    }

    let without_focusable: Vec<_> = components
        .iter()
        .filter(|(_, i)| !i.has_focusable)
        .map(|(n, _)| n.as_str())
        .collect();
    if !without_focusable.is_empty() {
        println!("  Without Focusable ({}):", without_focusable.len());
        for name in &without_focusable {
            println!("    {}", name);
        }
    }

    let toggleable_without_visible: Vec<_> = components
        .iter()
        .filter(|(_, i)| i.has_toggleable && !i.has_visible)
        .map(|(n, _)| n.as_str())
        .collect();
    if toggleable_without_visible.is_empty() {
        println!("  Toggleable without is_visible: NONE (all consistent)");
    } else {
        println!(
            "  Toggleable WITHOUT is_visible ({}):",
            toggleable_without_visible.len()
        );
        for name in &toggleable_without_visible {
            println!("    {}", name);
        }
    }

    let focusable_without_disabled: Vec<_> = components
        .iter()
        .filter(|(_, i)| i.has_focusable && !i.has_disabled)
        .map(|(n, _)| n.as_str())
        .collect();
    if focusable_without_disabled.is_empty() {
        println!("  Focusable without is_disabled: NONE (all consistent)");
    } else {
        println!(
            "  Focusable WITHOUT is_disabled ({}):",
            focusable_without_disabled.len()
        );
        for name in &focusable_without_disabled {
            println!("    {}", name);
        }
    }
}

fn print_builder_summary(components: &BTreeMap<String, ComponentInfo>) {
    println!("\nBuilder Methods (with_*):");

    let mut by_count: Vec<_> = components
        .iter()
        .map(|(n, i)| (n.as_str(), i.builder_methods.len()))
        .filter(|(_, c)| *c > 0)
        .collect();
    by_count.sort_by(|a, b| b.1.cmp(&a.1));

    for (name, count) in &by_count {
        println!("  {} ({}):", name, count);
        let info = &components[*name];
        for method in &info.builder_methods {
            println!("    with_{}", method);
        }
    }

    let without_builders: Vec<_> = components
        .iter()
        .filter(|(_, i)| i.builder_methods.is_empty())
        .map(|(n, _)| n.as_str())
        .collect();
    if !without_builders.is_empty() {
        println!("  Without any builders ({}):", without_builders.len());
        for name in &without_builders {
            println!("    {}", name);
        }
    }
}

fn print_accessor_symmetry(components: &BTreeMap<String, ComponentInfo>) {
    println!("\nAccessor Symmetry (set_X without matching getter):");

    let mut any_mismatch = false;
    for (name, info) in components {
        let missing: Vec<_> = info
            .setter_methods
            .iter()
            .filter(|setter| {
                let getter_name = setter.strip_prefix("set_").unwrap_or(setter);
                !info
                    .getter_methods
                    .iter()
                    .any(|g| g == getter_name || g == &format!("is_{}", getter_name))
            })
            .collect();
        if !missing.is_empty() {
            any_mismatch = true;
            println!("  {}:", name);
            for m in &missing {
                println!("    set_{} has no matching getter", m);
            }
        }
    }
    if !any_mismatch {
        println!("  All setters have matching getters.");
    }
}

fn print_doctest_summary(components: &BTreeMap<String, ComponentInfo>) {
    println!("\nDoc Test Coverage (pub fn with doc test / total pub fn):");

    let mut total_with = 0usize;
    let mut total_pub = 0usize;
    let mut entries: Vec<_> = components
        .iter()
        .filter(|(_, i)| i.pub_fn_count > 0)
        .map(|(n, i)| {
            let pct = if i.pub_fn_count > 0 {
                i.pub_fn_with_doctest as f64 / i.pub_fn_count as f64 * 100.0
            } else {
                0.0
            };
            (n.as_str(), i.pub_fn_with_doctest, i.pub_fn_count, pct)
        })
        .collect();

    entries.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));

    for (name, with_dt, total, pct) in &entries {
        println!("  {:>30}: {:>3}/{:<3} ({:>5.1}%)", name, with_dt, total, pct);
        total_with += with_dt;
        total_pub += total;
    }

    if total_pub > 0 {
        let pct = total_with as f64 / total_pub as f64 * 100.0;
        println!(
            "  {:>30}: {:>3}/{:<3} ({:>5.1}%)",
            "TOTAL", total_with, total_pub, pct
        );
    }
}

fn print_test_summary(components: &BTreeMap<String, ComponentInfo>) {
    println!("\nTests Per Component:");

    let mut total_unit = 0usize;
    let mut total_snapshot = 0usize;

    let mut entries: Vec<_> = components
        .iter()
        .filter(|(_, i)| i.unit_test_count > 0 || i.snapshot_test_count > 0)
        .map(|(n, i)| (n.as_str(), i.unit_test_count, i.snapshot_test_count))
        .collect();
    entries.sort_by(|a, b| (b.1 + b.2).cmp(&(a.1 + a.2)));

    println!(
        "  {:>30}  {:>5}  {:>8}  {:>5}",
        "Component", "Unit", "Snapshot", "Total"
    );
    println!("  {}", "-".repeat(55));
    for (name, unit, snapshot) in &entries {
        println!(
            "  {:>30}  {:>5}  {:>8}  {:>5}",
            name,
            unit,
            snapshot,
            unit + snapshot
        );
        total_unit += unit;
        total_snapshot += snapshot;
    }
    println!("  {}", "-".repeat(55));
    println!(
        "  {:>30}  {:>5}  {:>8}  {:>5}",
        "TOTAL",
        total_unit,
        total_snapshot,
        total_unit + total_snapshot
    );
}

fn print_naming_patterns(components: &BTreeMap<String, ComponentInfo>) {
    println!("\nNaming Patterns:");

    let patterns = [
        "selected",
        "value",
        "checked",
        "expanded",
        "items",
        "label",
        "title",
        "placeholder",
    ];

    for pattern in &patterns {
        let mut findings: Vec<(&str, &str)> = Vec::new();
        for (name, info) in components {
            for method in &info.public_method_names {
                if method.contains(pattern) {
                    findings.push((name.as_str(), method.as_str()));
                }
            }
        }
        if !findings.is_empty() {
            println!("  \"{}\":", pattern);
            for (comp, method) in &findings {
                println!("    {}: {}()", comp, method);
            }
        }
    }
}

fn print_quality_checks(src: &Path, root: &Path) {
    println!("\nQuality Checks:");

    let all_files = collect_all_rs_files(src);

    // Unsafe blocks
    let mut unsafe_locations: Vec<(PathBuf, usize, String)> = Vec::new();
    for (path, content) in &all_files {
        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if (trimmed.starts_with("unsafe ") || trimmed.contains(" unsafe "))
                && !trimmed.starts_with("//")
                && !trimmed.starts_with("///")
            {
                let display = path.strip_prefix(root).unwrap_or(path);
                unsafe_locations.push((display.to_path_buf(), i + 1, trimmed.to_string()));
            }
        }
    }
    println!("  Unsafe blocks: {}", unsafe_locations.len());
    for (path, line, code) in &unsafe_locations {
        println!("    {}:{} - {}", path.display(), line, code);
    }

    // Clippy suppressions
    let mut clippy_locations: Vec<(PathBuf, usize, String)> = Vec::new();
    for (path, content) in &all_files {
        for (i, line) in content.lines().enumerate() {
            if line.contains("#[allow(clippy::") {
                let display = path.strip_prefix(root).unwrap_or(path);
                clippy_locations.push((display.to_path_buf(), i + 1, line.trim().to_string()));
            }
        }
    }
    println!("  Clippy suppressions: {}", clippy_locations.len());
    for (path, line, code) in &clippy_locations {
        println!("    {}:{} - {}", path.display(), line, code);
    }

    // #![warn(missing_docs)]
    let lib_rs = fs::read_to_string(src.join("lib.rs")).unwrap_or_default();
    let has_warn = lib_rs.contains("#![warn(missing_docs)]");
    println!(
        "  #![warn(missing_docs)]: {}",
        if has_warn { "YES" } else { "NO" }
    );

    // Public items count
    let mut pub_item_count = 0;
    let pub_re = Regex::new(
        r"(?m)^\s*pub\s+(?:fn|struct|enum|trait|type|const|static|mod|use)\b"
    )
    .unwrap();
    let pub_crate_re = Regex::new(r"pub\s*\((?:crate|super|self)\)").unwrap();
    for (_path, content) in &all_files {
        for line in content.lines() {
            if pub_re.is_match(line) && !pub_crate_re.is_match(line) {
                pub_item_count += 1;
            }
        }
    }
    println!("  Total public items (across all src/): {}", pub_item_count);
}

fn print_reexports(src: &Path) {
    println!("\nlib.rs Re-exports:");

    let lib_rs = fs::read_to_string(src.join("lib.rs")).unwrap_or_default();
    let mut total_items = 0;
    let mut reexport_entries: Vec<(String, usize)> = Vec::new();

    // Join multiline pub use statements, then count items
    let mut current_use = String::new();
    let mut in_use = false;

    for line in lib_rs.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("pub use ") {
            in_use = true;
            current_use = trimmed.to_string();
            if trimmed.contains(';') {
                // Single-line use statement
                in_use = false;
                let count = count_use_items(&current_use);
                total_items += count;
                reexport_entries.push((current_use.clone(), count));
                current_use.clear();
            }
        } else if in_use {
            current_use.push(' ');
            current_use.push_str(trimmed);
            if trimmed.contains(';') {
                in_use = false;
                let count = count_use_items(&current_use);
                total_items += count;
                reexport_entries.push((current_use.clone(), count));
                current_use.clear();
            }
        }
    }

    println!("  Total re-exported items: {}", total_items);
    for (line, count) in &reexport_entries {
        println!("  [{:>3}] {}", count, line);
    }
}

fn count_use_items(use_stmt: &str) -> usize {
    if use_stmt.contains('{') {
        if let Some(brace_content) = use_stmt.split('{').nth(1) {
            if let Some(items) = brace_content.split('}').next() {
                return items.split(',').filter(|s| !s.trim().is_empty()).count();
            }
        }
        0
    } else if use_stmt.contains("::*") {
        // Glob re-export like `pub use component::*`
        1
    } else {
        1
    }
}

// --- Helper functions ---

/// Filters out inline test module content from source code.
/// Handles both `#[cfg(test)] mod tests { ... }` (inline test blocks)
/// and skips `#[cfg(test)] mod tests;` (external test file references)
/// which should NOT terminate scanning of the rest of the file.
fn non_test_content(content: &str) -> String {
    non_test_lines(content).collect::<Vec<&str>>().join("\n")
}

fn non_test_lines(content: &str) -> impl Iterator<Item = &str> {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();
    let mut i = 0;
    let mut brace_depth = 0;
    let mut in_test_block = false;

    while i < lines.len() {
        let trimmed = lines[i].trim();

        if in_test_block {
            // Track brace depth to find the end of the test module block
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
            // Look at next non-empty line to determine if this is an inline test block
            let mut next = i + 1;
            while next < lines.len() && lines[next].trim().is_empty() {
                next += 1;
            }
            if next < lines.len() {
                let next_trimmed = lines[next].trim();
                if next_trimmed.starts_with("mod ") && next_trimmed.ends_with(';') {
                    // External test module reference: `#[cfg(test)] mod tests;`
                    // Skip the cfg line and the mod line, but keep scanning
                    result.push(lines[i]); // keep it in output for line number consistency
                    i += 1;
                    continue;
                }
                // Inline test module block: skip everything until closing brace
                in_test_block = true;
                // Count opening braces on the next line
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

    result.into_iter()
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

fn read_test_files(dir: &Path) -> String {
    let mut content = String::new();
    let test_files = ["tests.rs", "snapshot_tests.rs"];
    for name in &test_files {
        if let Ok(c) = fs::read_to_string(dir.join(name)) {
            content.push_str(&c);
            content.push('\n');
        }
    }
    content
}

fn extract_method_names(content: &str, prefix: &str) -> Vec<String> {
    let mut names = Vec::new();
    for line in non_test_lines(content) {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix(prefix) {
            if let Some(name_end) = rest.find(['(', '<', ' ']) {
                let name = &rest[..name_end];
                names.push(name.to_string());
            }
        }
    }
    names
}

fn extract_getter_methods(content: &str) -> Vec<String> {
    let mut names = Vec::new();
    for line in non_test_lines(content) {
        let trimmed = line.trim();
        // Match pub fn name(&self) patterns (getters)
        if (trimmed.starts_with("pub fn ") || trimmed.starts_with("pub const fn "))
            && !trimmed.contains("pub(crate)")
            && !trimmed.contains("pub(super)")
            && trimmed.contains("&self")
            && !trimmed.contains("&mut self")
        {
            if let Some(name) = extract_fn_name(trimmed) {
                // Exclude with_ and set_ (those are builders/setters)
                if !name.starts_with("with_") && !name.starts_with("set_") {
                    names.push(name);
                }
            }
        }
    }
    names
}

fn extract_all_public_fn_names(content: &str) -> Vec<String> {
    let mut names = Vec::new();
    for line in non_test_lines(content) {
        let trimmed = line.trim();
        if is_public_fn(trimmed) {
            if let Some(name) = extract_fn_name(trimmed) {
                names.push(name);
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

fn count_doctest_coverage(content: &str) -> (usize, usize) {
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
            // Attributes between doc comment and fn - skip
            continue;
        } else {
            break;
        }
    }

    doc_lines.iter().any(|line| line.contains("```"))
}

fn collect_all_rs_files(dir: &Path) -> Vec<(PathBuf, String)> {
    let mut files = Vec::new();
    walk_collect(dir, &mut files);
    files
}

fn walk_collect(dir: &Path, files: &mut Vec<(PathBuf, String)>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = entries.flatten().collect();
    entries.sort_by_key(|e| e.path());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            walk_collect(&path, files);
        } else if path.extension().is_some_and(|e| e == "rs") {
            if let Ok(content) = fs::read_to_string(&path) {
                files.push((path, content));
            }
        }
    }
}
