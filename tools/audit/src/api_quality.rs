use std::fs;
use std::path::{Path, PathBuf};

pub fn run(root: &Path) {
    println!("API QUALITY CHECKS");
    println!("{}", "-".repeat(70));

    let src = root.join("src");

    print_reexport_gaps(&src);
    print_dependency_leakage(&src, root);
    print_test_gating(&src, root);
}

fn print_reexport_gaps(src: &Path) {
    println!("\nRe-export Gaps (types in app/mod.rs but not lib.rs):");

    let app_mod = fs::read_to_string(src.join("app/mod.rs")).unwrap_or_default();
    let lib_rs = fs::read_to_string(src.join("lib.rs")).unwrap_or_default();

    let app_items = extract_pub_use_items(&app_mod);
    let lib_items = extract_pub_use_items(&lib_rs);

    let mut gaps: Vec<&str> = app_items
        .iter()
        .filter(|item| !lib_items.contains(item))
        .map(|s| s.as_str())
        .collect();
    gaps.sort();

    if gaps.is_empty() {
        println!("  No gaps found - all app/mod.rs exports are re-exported from lib.rs");
    } else {
        println!(
            "  REEXPORT_GAPS: {} types in app/mod.rs missing from lib.rs:",
            gaps.len()
        );
        for item in &gaps {
            println!("    {}", item);
        }
    }
}

fn print_dependency_leakage(src: &Path, root: &Path) {
    println!("\nDependency Leakage (external types in public signatures):");

    let all_files = collect_all_rs_files(src);
    let dep_prefixes = ["ratatui::", "crossterm::", "tokio::", "tokio_stream::"];
    let mut leaks: Vec<(PathBuf, usize, String)> = Vec::new();

    for (path, content) in &all_files {
        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            // Skip re-export lines, imports, and comments
            if trimmed.starts_with("pub use ")
                || trimmed.starts_with("use ")
                || trimmed.starts_with("//")
                || trimmed.starts_with("///")
            {
                continue;
            }
            // Look for public function signatures that reference dependency types
            if is_public_fn(trimmed) {
                for prefix in &dep_prefixes {
                    if trimmed.contains(prefix) {
                        let display = path.strip_prefix(root).unwrap_or(path);
                        leaks.push((display.to_path_buf(), i + 1, trimmed.to_string()));
                        break;
                    }
                }
            }
        }
    }

    if leaks.is_empty() {
        println!("  No dependency leakage found in public function signatures");
    } else {
        println!(
            "  DEPENDENCY_LEAKAGE: {} public signatures reference dependency types:",
            leaks.len()
        );
        for (path, line, code) in &leaks {
            println!("    {}:{} - {}", path.display(), line, code);
        }
    }
}

fn print_test_gating(src: &Path, root: &Path) {
    println!("\nTest Gating (#[cfg(test)] on non-module items in harness/):");

    let harness_dir = src.join("harness");
    let all_files = collect_all_rs_files(&harness_dir);
    let mut gated_items: Vec<(PathBuf, usize, String)> = Vec::new();

    for (path, content) in &all_files {
        // Skip test files themselves
        if path.file_name().is_some_and(|n| n == "tests.rs") {
            continue;
        }

        let lines: Vec<&str> = content.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed == "#[cfg(test)]" {
                // Check next non-empty line
                let mut next = i + 1;
                while next < lines.len() && lines[next].trim().is_empty() {
                    next += 1;
                }
                if next < lines.len() {
                    let next_trimmed = lines[next].trim();
                    // Skip mod declarations (those are fine)
                    if next_trimmed.starts_with("mod ") {
                        continue;
                    }
                    let display = path.strip_prefix(root).unwrap_or(path);
                    gated_items.push((display.to_path_buf(), i + 1, next_trimmed.to_string()));
                }
            }
        }
    }

    if gated_items.is_empty() {
        println!("  No non-module items gated by #[cfg(test)] in harness/");
    } else {
        println!(
            "  TEST_GATING: {} non-module items gated by #[cfg(test)] in harness/:",
            gated_items.len()
        );
        for (path, line, code) in &gated_items {
            println!("    {}:{} - {}", path.display(), line, code);
        }
    }
}

fn extract_pub_use_items(content: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut current_use = String::new();
    let mut in_use = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("pub use ") {
            in_use = true;
            current_use = trimmed.to_string();
            if trimmed.contains(';') {
                in_use = false;
                extract_items_from_use(&current_use, &mut items);
                current_use.clear();
            }
        } else if in_use {
            current_use.push(' ');
            current_use.push_str(trimmed);
            if trimmed.contains(';') {
                in_use = false;
                extract_items_from_use(&current_use, &mut items);
                current_use.clear();
            }
        }
    }

    items
}

fn extract_items_from_use(use_stmt: &str, items: &mut Vec<String>) {
    if use_stmt.contains('{') {
        if let Some(brace_content) = use_stmt.split('{').nth(1) {
            if let Some(item_list) = brace_content.split('}').next() {
                for item in item_list.split(',') {
                    let name = item.trim();
                    if !name.is_empty() {
                        items.push(name.to_string());
                    }
                }
            }
        }
    } else if use_stmt.contains("::*") {
        // Glob - skip
    } else {
        // Single item: pub use foo::bar::Baz;
        let stmt = use_stmt.trim_end_matches(';').trim();
        if let Some(last) = stmt.rsplit("::").next() {
            items.push(last.to_string());
        }
    }
}

fn is_public_fn(line: &str) -> bool {
    (line.starts_with("pub fn ")
        || line.starts_with("pub const fn ")
        || line.starts_with("pub async fn "))
        && !line.contains("pub(crate)")
        && !line.contains("pub(super)")
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
