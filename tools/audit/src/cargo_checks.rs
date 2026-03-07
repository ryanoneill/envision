use std::path::Path;
use std::process::Command;

pub fn run(root: &Path) {
    println!("CARGO CHECKS");
    println!("{}", "-".repeat(70));
    println!();

    let checks: &[(&str, &[&str])] = &[
        (
            "cargo test --all-features",
            &["test", "--all-features"],
        ),
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
}

fn run_check(name: &str, args: &[&str], root: &Path) {
    print!("  {} ... ", name);

    let output = Command::new("cargo")
        .args(args)
        .current_dir(root)
        .output();

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
                let start = if lines.len() > 20 { lines.len() - 20 } else { 0 };
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
