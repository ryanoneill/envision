use std::fs;
use std::path::{Path, PathBuf};

struct FileInfo {
    path: PathBuf,
    lines: usize,
}

pub fn run(root: &Path) {
    println!("FILE STATISTICS");
    println!("{}", "-".repeat(70));

    let categories = [
        ("src", 30usize),
        ("tests", 20),
        ("benches", 10),
        ("examples", 10),
    ];

    let mut grand_files = 0;
    let mut grand_lines = 0;
    let mut over_limit = Vec::new();

    for (dir, top_n) in &categories {
        let dir_path = root.join(dir);
        if !dir_path.exists() {
            continue;
        }

        let mut files = collect_rs_files(&dir_path);
        files.sort_by(|a, b| b.lines.cmp(&a.lines));

        let total_lines: usize = files.iter().map(|f| f.lines).sum();
        let file_count = files.len();

        println!(
            "\n{} ({} files, {} lines):",
            dir,
            file_count,
            format_number(total_lines)
        );

        for file in files.iter().take(*top_n) {
            let display = file.path.strip_prefix(root).unwrap_or(&file.path);
            let marker = if file.lines > 1000 { " !!" } else { "" };
            println!("  {:>6}  {}{}", file.lines, display.display(), marker);
        }

        for file in &files {
            if file.lines > 1000 {
                let display = file.path.strip_prefix(root).unwrap_or(&file.path);
                over_limit.push((display.to_path_buf(), file.lines));
            }
        }

        if file_count > *top_n {
            println!("  ... and {} more files", file_count - top_n);
        }

        grand_files += file_count;
        grand_lines += total_lines;
    }

    println!(
        "\nTotal: {} files, {} lines",
        grand_files,
        format_number(grand_lines)
    );

    println!("\nFiles exceeding 1000 lines:");
    if over_limit.is_empty() {
        println!("  NONE");
    } else {
        for (path, lines) in &over_limit {
            println!("  {} ({} lines)", path.display(), lines);
        }
    }
}

fn collect_rs_files(dir: &Path) -> Vec<FileInfo> {
    let mut results = Vec::new();
    walk_dir(dir, &mut results);
    results
}

fn walk_dir(dir: &Path, results: &mut Vec<FileInfo>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = entries.flatten().collect();
    entries.sort_by_key(|e| e.path());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            walk_dir(&path, results);
        } else if path.extension().is_some_and(|e| e == "rs") {
            if let Ok(content) = fs::read_to_string(&path) {
                results.push(FileInfo {
                    lines: content.lines().count(),
                    path,
                });
            }
        }
    }
}

fn format_number(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}
