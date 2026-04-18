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
    // Instance method checks
    has_instance_handle_event: bool,
    has_instance_dispatch_event: bool,
    has_instance_update: bool,
    // Standard trait derives
    state_derives: Vec<String>,
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
    print_instance_methods(&components);
    print_trait_derives(&components);
    print_module_doc_coverage(&src, root);
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
    let source_content = read_non_test_sources(dir);

    let focusable_re = Regex::new(r"impl\b[^{{]*\bFocusable\b\s+for\b").unwrap();
    let toggleable_re = Regex::new(r"impl\b[^{{]*\bToggleable\b\s+for\b").unwrap();

    let has_focusable = focusable_re.is_match(&all_content);
    let has_toggleable = toggleable_re.is_match(&all_content);
    let has_disabled =
        source_content.contains("fn is_disabled") && source_content.contains("fn set_disabled");
    let has_visible =
        source_content.contains("fn is_visible") && source_content.contains("fn set_visible");

    let builder_methods = extract_method_names(&source_content, "pub fn with_");
    let setter_methods = extract_method_names(&source_content, "pub fn set_");
    let getter_methods = extract_getter_methods(&source_content);
    let public_method_names = extract_all_public_fn_names(&source_content);
    let (pub_fn_count, pub_fn_with_doctest) = count_doctest_coverage(&source_content);

    let test_content = read_test_files(dir);
    let unit_test_count =
        test_content.matches("#[test]").count() + test_content.matches("#[tokio::test]").count();

    let snapshot_content = fs::read_to_string(dir.join("snapshot_tests.rs")).unwrap_or_default();
    let snapshot_test_count = snapshot_content.matches("#[test]").count();

    let has_instance_handle_event = source_content.contains("pub fn handle_event(&self")
        || source_content.contains("pub fn handle_event(&mut self");
    let has_instance_dispatch_event =
        source_content.contains("pub fn dispatch_event(&mut self");
    let has_instance_update = source_content.contains("pub fn update(&mut self");

    // Standard trait derives on State type (check all files, not just mod.rs,
    // since some components define their State struct in a separate state.rs)
    let state_derives = extract_state_derives(&all_content);

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
        has_instance_handle_event,
        has_instance_dispatch_event,
        has_instance_update,
        state_derives,
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
                !info.getter_methods.iter().any(|g| {
                    g == getter_name
                        || g == &format!("is_{}", getter_name)
                        || g == &format!("{}_value", getter_name)
                })
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
        println!(
            "  {:>30}: {:>3}/{:<3} ({:>5.1}%)",
            name, with_dt, total, pct
        );
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

fn print_instance_methods(components: &BTreeMap<String, ComponentInfo>) {
    println!("\nInstance Methods on State Types:");

    // Only check components that have Focusable (interactive components)
    let interactive: Vec<_> = components.iter().filter(|(_, i)| i.has_focusable).collect();

    let checks = [
        ("handle_event", "has_instance_handle_event"),
        ("dispatch_event", "has_instance_dispatch_event"),
        ("update", "has_instance_update"),
    ];

    for (method_name, field_name) in &checks {
        let missing: Vec<_> = interactive
            .iter()
            .filter(|(_, i)| !match *field_name {
                "has_instance_handle_event" => i.has_instance_handle_event,
                "has_instance_dispatch_event" => i.has_instance_dispatch_event,
                "has_instance_update" => i.has_instance_update,
                _ => false,
            })
            .map(|(n, _)| n.as_str())
            .collect();

        if missing.is_empty() {
            println!(
                "  {}: all {} Focusable components have it",
                method_name,
                interactive.len()
            );
        } else {
            println!(
                "  {}: {}/{} Focusable components (missing {}):",
                method_name,
                interactive.len() - missing.len(),
                interactive.len(),
                missing.len()
            );
            for name in &missing {
                println!("    missing: {}", name);
            }
        }
    }
}

fn print_trait_derives(components: &BTreeMap<String, ComponentInfo>) {
    println!("\nStandard Trait Derives on State Types:");

    let traits = ["Debug", "Clone", "Default", "PartialEq"];
    for trait_name in &traits {
        let has_it: Vec<_> = components
            .iter()
            .filter(|(_, i)| i.state_derives.iter().any(|d| d == *trait_name))
            .map(|(n, _)| n.as_str())
            .collect();
        let missing: Vec<_> = components
            .iter()
            .filter(|(_, i)| {
                !i.state_derives.is_empty() && !i.state_derives.iter().any(|d| d == *trait_name)
            })
            .map(|(n, _)| n.as_str())
            .collect();

        println!(
            "  {}: {}/{} components",
            trait_name,
            has_it.len(),
            components.len()
        );
        if !missing.is_empty() && missing.len() <= 10 {
            for name in &missing {
                println!("    missing: {}", name);
            }
        }
    }
}

fn print_module_doc_coverage(src: &Path, root: &Path) {
    println!("\nModule & Type Doc Coverage:");

    let all_files = collect_all_rs_files(src);

    // Check for //! module-level docs in mod.rs files
    let mut total_modules = 0;
    let mut with_module_docs = 0;
    let mut missing_module_docs: Vec<PathBuf> = Vec::new();

    for (path, content) in &all_files {
        if path
            .file_name()
            .is_some_and(|n| n == "mod.rs" || n == "lib.rs")
        {
            total_modules += 1;
            if content.lines().any(|l| l.trim().starts_with("//!")) {
                with_module_docs += 1;
            } else {
                let display = path.strip_prefix(root).unwrap_or(path);
                missing_module_docs.push(display.to_path_buf());
            }
        }
    }

    println!(
        "  Module-level docs (//!): {}/{} ({:.0}%)",
        with_module_docs,
        total_modules,
        if total_modules > 0 {
            with_module_docs as f64 / total_modules as f64 * 100.0
        } else {
            0.0
        }
    );
    if !missing_module_docs.is_empty() && missing_module_docs.len() <= 15 {
        for path in &missing_module_docs {
            println!("    missing: {}", path.display());
        }
    } else if missing_module_docs.len() > 15 {
        for path in missing_module_docs.iter().take(10) {
            println!("    missing: {}", path.display());
        }
        println!("    ... and {} more", missing_module_docs.len() - 10);
    }

    // Check for /// docs on public structs/enums/traits
    let mut total_types = 0;
    let mut with_type_docs = 0;

    for (_path, content) in &all_files {
        let lines: Vec<&str> = content.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if (trimmed.starts_with("pub struct ")
                || trimmed.starts_with("pub enum ")
                || trimmed.starts_with("pub trait "))
                && !trimmed.contains("pub(crate)")
                && !trimmed.contains("pub(super)")
            {
                total_types += 1;
                if has_doc_comment_above(&lines, i) {
                    with_type_docs += 1;
                }
            }
        }
    }

    println!(
        "  Type-level docs (///): {}/{} ({:.0}%)",
        with_type_docs,
        total_types,
        if total_types > 0 {
            with_type_docs as f64 / total_types as f64 * 100.0
        } else {
            0.0
        }
    );
}

/// Checks whether a `pub struct/enum/trait` at `lines[idx]` has a `///` doc
/// comment above it, correctly traversing multi-line attributes like:
///
/// ```text
/// /// Doc comment here.
/// #[derive(Clone, Debug)]
/// #[cfg_attr(
///     feature = "serialization",
///     derive(serde::Serialize, serde::Deserialize)
/// )]
/// pub struct Foo { ... }
/// ```
fn has_doc_comment_above(lines: &[&str], idx: usize) -> bool {
    if idx == 0 {
        return false;
    }

    let mut j = idx;
    // Track whether we are inside a multi-line attribute.
    // When we see a line ending with `)]` or `]` that closes an attribute
    // opened with `#[`, we need to skip upwards through the attribute body.
    let mut attr_depth: i32 = 0;

    while j > 0 {
        j -= 1;
        let prev = lines[j].trim();

        // Doc comment found
        if prev.starts_with("///") {
            return true;
        }

        // Empty lines are fine to skip
        if prev.is_empty() {
            continue;
        }

        // Single-line attribute: #[...] on one line
        if prev.starts_with("#[") && prev.ends_with(']') {
            continue;
        }

        // Start of a multi-line attribute: #[... without closing ]
        if prev.starts_with("#[") && !prev.ends_with(']') {
            // We've reached the opening of a multi-line attribute.
            // If we were tracking depth, decrement. Either way, continue up.
            if attr_depth > 0 {
                attr_depth -= 1;
            }
            continue;
        }

        // End of a multi-line attribute: line ending with )] or ]
        // but not starting with #[ (that would be single-line)
        if (prev.ends_with(")]") || prev.ends_with(']')) && !prev.starts_with("#[") {
            attr_depth += 1;
            continue;
        }

        // Inside a multi-line attribute body (e.g. `feature = "serialization",`
        // or `derive(serde::Serialize, serde::Deserialize)`)
        if attr_depth > 0 {
            continue;
        }

        // Heuristic: if we haven't hit a code boundary, check common
        // attribute continuation patterns (lines that are clearly part of
        // an attribute but don't match the depth tracking above).
        if prev.ends_with(',')
            || prev.ends_with(')')
            || prev.starts_with("derive(")
            || prev.starts_with("feature")
            || prev.starts_with("serde(")
            || prev.starts_with("cfg(")
            || prev.starts_with("cfg_attr(")
        {
            continue;
        }

        // Any other line is a code boundary -- stop
        break;
    }

    false
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
    let pub_re =
        Regex::new(r"(?m)^\s*pub\s+(?:fn|struct|enum|trait|type|const|static|mod|use)\b").unwrap();
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

fn read_non_test_sources(dir: &Path) -> String {
    let mut combined = String::new();
    let test_filenames = ["tests.rs", "snapshot_tests.rs"];

    let Ok(entries) = fs::read_dir(dir) else {
        return combined;
    };
    let mut files: Vec<_> = entries.flatten().collect();
    files.sort_by_key(|e| e.path());

    for entry in files {
        let path = entry.path();
        if !path.is_file() || !path.extension().is_some_and(|e| e == "rs") {
            continue;
        }
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if test_filenames.contains(&name) {
                continue;
            }
        }
        if let Ok(content) = fs::read_to_string(&path) {
            let filtered = non_test_content(&content);
            combined.push_str(&filtered);
            combined.push('\n');
        }
    }
    combined
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

fn extract_state_derives(content: &str) -> Vec<String> {
    let mut derives = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    // Find the first State struct and extract its type name
    let mut state_type_name: Option<String> = None;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        // Look for pub struct ...State
        if trimmed.starts_with("pub struct ") && trimmed.contains("State") {
            // Extract the State type name (e.g. "ButtonState" from "pub struct ButtonState {")
            if let Some(name) = trimmed
                .strip_prefix("pub struct ")
                .and_then(|s| s.split(|c: char| !c.is_alphanumeric() && c != '_').next())
            {
                state_type_name = Some(name.to_string());
            }

            // Collect the attribute block above the struct by joining lines
            // that are part of attributes (handles multi-line #[cfg_attr(...)])
            let mut attr_block = String::new();
            let mut j = i;
            while j > 0 {
                j -= 1;
                let prev = lines[j].trim();
                if prev.starts_with("///") || prev.is_empty() {
                    continue;
                }
                // Part of attribute block: starts with #[ or is continuation
                // of a multi-line attribute (contains ), or derive, or feature)
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
            break; // Only check first State struct
        }
    }

    // Also scan for manual trait impls. Handles both short (`impl Debug for`),
    // fully-qualified (`impl std::fmt::Debug for`, `impl core::fmt::Debug for`),
    // and generic (`impl<T: Clone> PartialEq for FooState<T>`) impls.
    if let Some(ref state_name) = state_type_name {
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
                continue; // Already found via derive
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
