use std::path::Path;
use std::process::Command;

pub fn run(root: &Path) {
    println!("CARGO CHECKS");
    println!("{}", "-".repeat(70));
    println!();

    let checks: &[(&str, &[&str])] = &[
        ("cargo test --all-features", &["test", "--all-features"]),
        (
            "cargo clippy --all-features",
            &["clippy", "--all-features", "--", "-D", "warnings"],
        ),
        (
            "cargo doc --no-deps --all-features",
            &["doc", "--no-deps", "--all-features"],
        ),
        (
            "cargo build --examples --all-features",
            &["build", "--examples", "--all-features"],
        ),
        (
            "cargo test --doc --all-features",
            &["test", "--doc", "--all-features"],
        ),
    ];

    for (name, args) in checks {
        run_check(name, args, root);
    }

    println!();
    print_test_breakdown(root);
}

fn run_check(name: &str, args: &[&str], root: &Path) {
    print!("  {} ... ", name);

    let output = Command::new("cargo").args(args).current_dir(root).output();

    match output {
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            let stdout = String::from_utf8_lossy(&o.stdout);

            if o.status.success() {
                let combined = format!("{}{}", stdout, stderr);
                if name.contains("test") {
                    let (passed, failed, ignored) = count_tests(&combined);
                    println!(
                        "PASS ({} passed, {} failed, {} ignored)",
                        passed, failed, ignored
                    );
                } else if name.contains("clippy") {
                    let warnings = count_clippy_warnings(&stderr);
                    println!("PASS ({} warnings)", warnings);
                } else {
                    println!("PASS");
                }
            } else {
                println!("FAIL");
                let output_text = if stderr.is_empty() {
                    stdout.to_string()
                } else {
                    stderr.to_string()
                };
                let lines: Vec<&str> = output_text.lines().collect();
                let start = if lines.len() > 20 {
                    lines.len() - 20
                } else {
                    0
                };
                for line in &lines[start..] {
                    println!("    {}", line);
                }
            }
        }
        Err(e) => println!("ERROR: {}", e),
    }
}

fn count_tests(output: &str) -> (usize, usize, usize) {
    let mut passed = 0;
    let mut failed = 0;
    let mut ignored = 0;

    for line in output.lines() {
        if line.contains("test result:") {
            passed += extract_number(line, "passed").unwrap_or(0);
            failed += extract_number(line, "failed").unwrap_or(0);
            ignored += extract_number(line, "ignored").unwrap_or(0);
        }
    }

    (passed, failed, ignored)
}

fn extract_number(line: &str, keyword: &str) -> Option<usize> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    for (i, part) in parts.iter().enumerate() {
        let clean = part.trim_end_matches([';', ',']);
        if clean == keyword && i > 0 {
            return parts[i - 1].parse().ok();
        }
    }
    None
}

fn count_clippy_warnings(output: &str) -> usize {
    output
        .lines()
        .filter(|line| line.starts_with("warning:") && !line.contains("generated"))
        .count()
}

fn print_test_breakdown(root: &Path) {
    println!("Test Breakdown:");

    // Unit tests (cargo test --lib)
    let unit = run_test_count(root, &["test", "--lib", "--all-features"]);

    // Integration tests (cargo test --tests, excluding --lib and --doc)
    let integration = run_test_count(root, &["test", "--tests", "--all-features"]);

    // Doc tests (cargo test --doc)
    let doc = run_test_count(root, &["test", "--doc", "--all-features"]);

    println!(
        "  Unit tests (--lib):   {} passed, {} failed, {} ignored",
        unit.0, unit.1, unit.2
    );
    println!(
        "  Integration (--tests): {} passed, {} failed, {} ignored",
        integration.0, integration.1, integration.2
    );
    println!(
        "  Doc tests (--doc):    {} passed, {} failed, {} ignored",
        doc.0, doc.1, doc.2
    );

    let total_passed = unit.0 + integration.0 + doc.0;
    let total_failed = unit.1 + integration.1 + doc.1;
    let total_ignored = unit.2 + integration.2 + doc.2;
    println!(
        "  Total:                {} passed, {} failed, {} ignored",
        total_passed, total_failed, total_ignored
    );
}

fn run_test_count(root: &Path, args: &[&str]) -> (usize, usize, usize) {
    let output = Command::new("cargo").args(args).current_dir(root).output();

    match output {
        Ok(o) => {
            let combined = format!(
                "{}{}",
                String::from_utf8_lossy(&o.stdout),
                String::from_utf8_lossy(&o.stderr)
            );
            count_tests(&combined)
        }
        Err(_) => (0, 0, 0),
    }
}
