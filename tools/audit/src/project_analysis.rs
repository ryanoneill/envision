use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

pub fn run(root: &Path) {
    println!("PROJECT ANALYSIS");
    println!("{}", "-".repeat(70));

    print_project_files(root);
    print_feature_flags(root);
    print_dependencies(root);
    print_ci_pipeline(root);
    print_example_coverage(root);
    print_benchmark_listing(root);
}

fn print_project_files(root: &Path) {
    println!("\nProject Files:");

    let files = [
        ("README.md", "Project documentation"),
        ("CHANGELOG.md", "Version history"),
        ("CONTRIBUTING.md", "Contribution guide"),
        ("SECURITY.md", "Security policy"),
        ("MIGRATION.md", "Migration guide"),
        ("LICENSE", "License file"),
        ("Cargo.lock", "Dependency lock file"),
    ];

    for (name, description) in &files {
        let path = root.join(name);
        if path.exists() {
            let size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            let lines = fs::read_to_string(&path)
                .map(|c| c.lines().count())
                .unwrap_or(0);
            println!(
                "  {} {} ({} bytes, {} lines)",
                name, description, size, lines
            );
        } else {
            println!("  {} {} -- MISSING", name, description);
        }
    }
}

fn print_feature_flags(root: &Path) {
    println!("\nFeature Flags:");

    let cargo_toml = fs::read_to_string(root.join("Cargo.toml")).unwrap_or_default();
    let mut in_features = false;
    let mut features: Vec<(String, String)> = Vec::new();

    for line in cargo_toml.lines() {
        let trimmed = line.trim();
        if trimmed == "[features]" {
            in_features = true;
            continue;
        }
        if trimmed.starts_with('[') && in_features {
            break;
        }
        if !in_features {
            continue;
        }
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = trimmed.split_once('=') {
            features.push((key.trim().to_string(), value.trim().to_string()));
        }
    }

    if features.is_empty() {
        println!("  NONE");
    } else {
        // Collect all .rs files in src/ to count cfg gate usage
        let src_dir = root.join("src");
        let all_rs_content = collect_rs_file_contents(&src_dir);

        for (name, deps) in &features {
            // Count lines containing cfg(feature = "NAME") in src/
            let pattern = format!("cfg(feature = \"{}\"", name);
            let gate_count = all_rs_content
                .iter()
                .map(|content| content.lines().filter(|l| l.contains(&pattern)).count())
                .sum::<usize>();

            if gate_count > 0 {
                println!("  {}: {} (gates {} code sites)", name, deps, gate_count);
            } else {
                println!("  {}: {}", name, deps);
            }
        }
        println!("  Total: {} feature flags", features.len());
    }
}

fn print_dependencies(root: &Path) {
    println!("\nDependencies:");

    let cargo_toml = fs::read_to_string(root.join("Cargo.toml")).unwrap_or_default();

    let regular = extract_deps(&cargo_toml, "[dependencies]");
    let dev = extract_deps(&cargo_toml, "[dev-dependencies]");

    println!("  Regular ({}):", regular.len());
    for (name, spec) in &regular {
        let optional = if spec.contains("optional = true") || spec.contains("optional=true") {
            " (optional)"
        } else {
            ""
        };
        println!("    {}{}", name, optional);
    }

    println!("  Dev ({}):", dev.len());
    for (name, _) in &dev {
        println!("    {}", name);
    }
}

fn extract_deps(content: &str, section: &str) -> Vec<(String, String)> {
    let mut in_section = false;
    let mut deps = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == section {
            in_section = true;
            continue;
        }
        if trimmed.starts_with('[') && in_section {
            break;
        }
        if !in_section {
            continue;
        }
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = trimmed.split_once('=') {
            deps.push((key.trim().to_string(), value.trim().to_string()));
        }
    }

    deps
}

fn print_ci_pipeline(root: &Path) {
    println!("\nCI Pipeline:");

    let workflow_dir = root.join(".github/workflows");
    if !workflow_dir.exists() {
        println!("  No .github/workflows/ directory found");
        return;
    }

    let Ok(entries) = fs::read_dir(&workflow_dir) else {
        println!("  Could not read workflow directory");
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext != "yml" && ext != "yaml" {
            continue;
        }

        let filename = path.file_name().unwrap_or_default().to_string_lossy();
        let content = fs::read_to_string(&path).unwrap_or_default();

        // Extract workflow name
        let workflow_name = content
            .lines()
            .find(|l| l.starts_with("name:"))
            .and_then(|l| l.strip_prefix("name:"))
            .map(|n| n.trim().to_string())
            .unwrap_or_else(|| filename.to_string());

        println!("  Workflow: {} ({})", workflow_name, filename);

        // Extract job names
        let mut in_jobs = false;
        let mut jobs = Vec::new();

        for line in content.lines() {
            if line == "jobs:" {
                in_jobs = true;
                continue;
            }
            if !in_jobs {
                continue;
            }
            // Job entries are indented exactly 2 spaces with a trailing colon
            if line.starts_with("  ") && !line.starts_with("    ") {
                let job_id = line.trim().trim_end_matches(':');
                if !job_id.is_empty() {
                    jobs.push(job_id.to_string());
                }
            }
        }

        // Extract job display names
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(name) = trimmed.strip_prefix("name: ") {
                // Skip workflow-level name
                if !line.starts_with("name:") {
                    println!("    - {}", name);
                }
            }
        }

        // Summarize capabilities
        let mut capabilities = Vec::new();
        if content.contains("cargo test") {
            capabilities.push("tests");
        }
        if content.contains("clippy") {
            capabilities.push("clippy");
        }
        if content.contains("cargo fmt") || content.contains("rustfmt") {
            capabilities.push("format");
        }
        if content.contains("cargo doc") {
            capabilities.push("docs");
        }
        if content.contains("tarpaulin") || content.contains("coverage") {
            capabilities.push("coverage");
        }
        if content.contains("cargo bench") {
            capabilities.push("benchmarks");
        }
        if content.contains("deploy-pages") || content.contains("github-pages") {
            capabilities.push("deploy");
        }

        let platforms: Vec<&str> = ["ubuntu", "macos", "windows"]
            .iter()
            .filter(|p| content.contains(**p))
            .copied()
            .collect();

        println!("    Capabilities: {}", capabilities.join(", "));
        println!("    Platforms: {}", platforms.join(", "));

        // Check for MSRV testing
        if content.contains("1.81") {
            println!("    MSRV tested: 1.81");
        }
    }
}

fn print_example_coverage(root: &Path) {
    println!("\nExample Component Coverage:");

    let examples_dir = root.join("examples");
    if !examples_dir.exists() {
        println!("  No examples/ directory");
        return;
    }

    // Collect all component type names by reading each component's mod.rs
    let component_dir = root.join("src/component");
    let mut all_components: BTreeSet<String> = BTreeSet::new();
    // Map from dir_name -> Vec<type_name> (State type, component type, Message type)
    let mut type_names: Vec<(String, Vec<String>)> = Vec::new();

    if let Ok(entries) = fs::read_dir(&component_dir) {
        for entry in entries.flatten() {
            let mod_file = entry.path().join("mod.rs");
            if !entry.path().is_dir() || !mod_file.exists() {
                continue;
            }
            let dir_name = entry.file_name().to_string_lossy().to_string();
            if dir_name == "focus_manager" {
                continue;
            }
            all_components.insert(dir_name.clone());

            // Read mod.rs and extract public type names
            let mod_content = fs::read_to_string(&mod_file).unwrap_or_default();
            let mut names: Vec<String> = Vec::new();

            for line in mod_content.lines() {
                let trimmed = line.trim();
                // Match pub struct TypeName, pub enum TypeName
                if (trimmed.starts_with("pub struct ") || trimmed.starts_with("pub enum "))
                    && !trimmed.contains("pub(crate)")
                    && !trimmed.contains("pub(super)")
                {
                    let after_keyword = if trimmed.starts_with("pub struct ") {
                        trimmed.strip_prefix("pub struct ")
                    } else {
                        trimmed.strip_prefix("pub enum ")
                    };
                    if let Some(rest) = after_keyword {
                        if let Some(name) = rest
                            .split(|c: char| !c.is_alphanumeric() && c != '_')
                            .next()
                        {
                            if !name.is_empty() {
                                names.push(name.to_string());
                            }
                        }
                    }
                }
            }

            type_names.push((dir_name, names));
        }
    }

    // Scan all example files
    let mut example_files: Vec<(String, String)> = Vec::new();
    if let Ok(entries) = fs::read_dir(&examples_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "rs") {
                if let Ok(content) = fs::read_to_string(&path) {
                    let name = path
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    example_files.push((name, content));
                }
            }
        }
    }
    example_files.sort_by(|a, b| a.0.cmp(&b.0));

    // For each example, list which components it uses
    let mut covered: BTreeSet<String> = BTreeSet::new();

    for (example_name, content) in &example_files {
        let mut used: Vec<String> = Vec::new();
        for (dir_name, names) in &type_names {
            for type_name in names {
                if content.contains(type_name.as_str()) {
                    // Use the primary type name (first without State/Message suffix)
                    let display = names
                        .iter()
                        .find(|n| !n.ends_with("State") && !n.ends_with("Message"))
                        .unwrap_or(type_name);
                    if !used.contains(display) {
                        used.push(display.clone());
                    }
                    covered.insert(dir_name.clone());
                    break;
                }
            }
        }
        if used.is_empty() {
            println!("  {}: (no components)", example_name);
        } else {
            println!("  {}: {}", example_name, used.join(", "));
        }
    }

    let total = all_components.len();
    let covered_count = covered.len();
    let uncovered: Vec<_> = all_components.difference(&covered).collect();

    println!(
        "\n  Coverage: {}/{} components ({:.0}%)",
        covered_count,
        total,
        covered_count as f64 / total as f64 * 100.0
    );

    if !uncovered.is_empty() {
        println!("  Not in any example ({}):", uncovered.len());
        for name in &uncovered {
            println!("    {}", name);
        }
    }
}

fn print_benchmark_listing(root: &Path) {
    println!("\nBenchmarks:");

    let bench_dir = root.join("benches");
    if !bench_dir.exists() {
        println!("  No benches/ directory");
        return;
    }

    let Ok(entries) = fs::read_dir(&bench_dir) else {
        println!("  Could not read benches/ directory");
        return;
    };

    let mut bench_files: Vec<(PathBuf, String)> = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "rs") {
            if let Ok(content) = fs::read_to_string(&path) {
                bench_files.push((path, content));
            }
        }
    }
    bench_files.sort_by(|a, b| a.0.cmp(&b.0));

    for (path, content) in &bench_files {
        let name = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let lines = content.lines().count();

        // Extract benchmark function names
        let mut bench_fns: Vec<String> = Vec::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("fn ") && trimmed.contains("Criterion") {
                if let Some(fn_name) = trimmed
                    .strip_prefix("fn ")
                    .and_then(|s| s.split('(').next())
                {
                    bench_fns.push(fn_name.to_string());
                }
            }
        }

        println!("  {} ({} lines):", name, lines);
        for func in &bench_fns {
            // Extract parameterization from the function body
            let params = extract_bench_params(content, func);
            if params.is_empty() {
                println!("    - {}", func);
            } else {
                println!("    - {} [{}]", func, params);
            }
        }
    }
}

/// Extract parameterization info from a benchmark function body.
/// Looks for `for ... in [...]` patterns and `BenchmarkId::new(...)` patterns.
fn extract_bench_params(content: &str, fn_name: &str) -> String {
    // Find the function body
    let fn_sig = format!("fn {}(", fn_name);
    let Some(fn_start) = content.find(&fn_sig) else {
        return String::new();
    };

    // Find the next function or end of file
    let rest = &content[fn_start..];
    let fn_end = rest[1..]
        .find("\nfn ")
        .map(|pos| pos + 1)
        .unwrap_or(rest.len());
    let fn_body = &rest[..fn_end];

    let mut params = Vec::new();

    // Look for `for ... in [values]` patterns (item counts, sizes)
    for line in fn_body.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("for ") || !trimmed.contains(" in [") {
            continue;
        }

        let Some(bracket_start) = trimmed.find('[') else {
            continue;
        };
        let Some(bracket_end) = trimmed.rfind(']') else {
            continue;
        };
        let values = &trimmed[bracket_start + 1..bracket_end];
        if values.is_empty() {
            continue;
        }

        // Check if this is a tuple iteration like `for (width, height) in [(80, 24), ...]`
        let is_dimension_tuple = trimmed.contains("(width") && trimmed.contains("height");

        if is_dimension_tuple {
            // Extract dimension pairs from tuples like "(80, 24), (120, 40)"
            let dims: Vec<String> = values
                .split("), (")
                .map(|d| {
                    d.trim_matches(|c: char| c == '(' || c == ')' || c.is_whitespace())
                        .replace(", ", "x")
                })
                .collect();
            params.push(dims.join(", "));
        } else {
            // Plain value iteration like `for count in [100, 1000]`
            params.push(format!("{} items", values));
        }
    }

    // Look for BenchmarkId::new usage as a fallback indicator
    if params.is_empty() && fn_body.contains("BenchmarkId::new") {
        params.push("parameterized".to_string());
    }

    params.join(" x ")
}

/// Recursively collect all .rs file contents under a directory.
fn collect_rs_file_contents(dir: &Path) -> Vec<String> {
    let mut contents = Vec::new();
    walk_collect_contents(dir, &mut contents);
    contents
}

fn walk_collect_contents(dir: &Path, contents: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_collect_contents(&path, contents);
        } else if path.extension().is_some_and(|e| e == "rs") {
            if let Ok(content) = fs::read_to_string(&path) {
                contents.push(content);
            }
        }
    }
}
