use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

mod cargo_checks;
mod code_analysis;
mod file_stats;
mod project_analysis;

fn main() {
    let args: Vec<String> = env::args().collect();
    let (subcommand, project_root) = parse_args(&args);

    if !project_root.join("Cargo.toml").exists() {
        eprintln!("Error: No Cargo.toml found in {}", project_root.display());
        process::exit(1);
    }

    let version = extract_version(&project_root);
    let git_hash = extract_git_hash(&project_root);

    println!("======================================================================");
    println!("ENVISION AUDIT TOOL");
    println!("Version: {}  Commit: {}", version, git_hash);
    println!("======================================================================");
    println!();

    match subcommand.as_deref() {
        Some("stats") => file_stats::run(&project_root),
        Some("code") => code_analysis::run(&project_root),
        Some("cargo") => cargo_checks::run(&project_root),
        Some("project") => project_analysis::run(&project_root),
        Some("all") | None => {
            file_stats::run(&project_root);
            println!();
            code_analysis::run(&project_root);
            println!();
            project_analysis::run(&project_root);
            println!();
            cargo_checks::run(&project_root);
        }
        Some(cmd) => {
            eprintln!("Unknown command: {cmd}");
            print_usage();
            process::exit(1);
        }
    }
}

fn parse_args(args: &[String]) -> (Option<String>, PathBuf) {
    let mut subcommand = None;
    let mut path = None;
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "--path" => {
                i += 1;
                if i < args.len() {
                    path = Some(PathBuf::from(&args[i]));
                } else {
                    eprintln!("Error: --path requires a value");
                    process::exit(1);
                }
            }
            "--help" | "-h" => {
                print_usage();
                process::exit(0);
            }
            arg if !arg.starts_with('-') && subcommand.is_none() => {
                subcommand = Some(arg.to_string());
            }
            other => {
                eprintln!("Unknown option: {other}");
                print_usage();
                process::exit(1);
            }
        }
        i += 1;
    }

    let root = path.unwrap_or_else(|| {
        env::current_dir().unwrap_or_else(|e| {
            eprintln!("Error: {e}");
            process::exit(1);
        })
    });

    (subcommand, root)
}

fn extract_version(root: &Path) -> String {
    let content = fs::read_to_string(root.join("Cargo.toml")).unwrap_or_default();
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("version = \"") {
            if let Some(version) = rest.strip_suffix('"') {
                return version.to_string();
            }
        }
    }
    "unknown".to_string()
}

fn extract_git_hash(root: &Path) -> String {
    process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(root)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn print_usage() {
    eprintln!("Usage: envision-audit [COMMAND] [OPTIONS]");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  stats    File and line count statistics");
    eprintln!("  code     Source code analysis");
    eprintln!("  project  Project-level analysis (files, deps, CI, examples)");
    eprintln!("  cargo    Run cargo checks (test, clippy, doc)");
    eprintln!("  all      Run all analyses (default)");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --path <dir>  Project root directory (default: current dir)");
}
