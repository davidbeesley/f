use crate::git_status::{FileType, GitFile};
use colored::Colorize;
use std::process::Command;

fn get_inline_diff(file: &GitFile) -> Vec<String> {
    let output = if file.file_type == FileType::Untracked {
        Command::new("git")
            .args([
                "diff",
                "--no-index",
                "--color=always",
                "/dev/null",
                file.abs_path.to_string_lossy().as_ref(),
            ])
            .output()
    } else {
        Command::new("git")
            .args([
                "diff",
                "--color=always",
                "--",
                file.abs_path.to_string_lossy().as_ref(),
            ])
            .output()
    };

    let Ok(output) = output else {
        return vec![];
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut lines = Vec::new();

    for line in stdout.lines() {
        let plain: String = line
            .chars()
            .filter(|c| !matches!(c, '\x1b'))
            .collect::<String>()
            .replace("[0m", "")
            .replace("[31m", "")
            .replace("[32m", "")
            .replace("[1m", "")
            .replace("[m", "");

        if (plain.starts_with('+') || plain.starts_with('-'))
            && !plain.starts_with("+++")
            && !plain.starts_with("---")
        {
            lines.push(line.to_string());
        }
    }

    lines
}

pub fn list_files(files: &[GitFile]) {
    if files.is_empty() {
        println!("{}", "No changed files".dimmed());
        return;
    }

    let mut last_type: Option<FileType> = None;

    for file in files {
        if last_type != Some(file.file_type) {
            if last_type.is_some() {
                println!();
            }
            let header = match file.file_type {
                FileType::Unstaged => format!("── {} ──", "Unstaged").yellow(),
                FileType::Untracked => format!("── {} ──", "Untracked").green(),
                FileType::Staged => format!("── {} ──", "Staged").cyan(),
            };
            println!("{}", header);
            last_type = Some(file.file_type);
        }

        let id_str = format!("{:<5}", file.stable_id);
        let stats_str = match &file.diff_stats {
            Some(stats) if stats.added > 0 || stats.removed > 0 => {
                format!(
                    " {}{}",
                    format!("+{}", stats.added).green(),
                    format!("/-{}", stats.removed).red()
                )
            }
            Some(stats) if stats.added > 0 => {
                format!(" {}", format!("{} lines", stats.added).green())
            }
            _ => String::new(),
        };

        println!("  {} {}{}", id_str.cyan(), file.rel_path, stats_str);

        if file.file_type == FileType::Unstaged || file.file_type == FileType::Untracked {
            let total_changes = file
                .diff_stats
                .as_ref()
                .map(|s| s.added + s.removed)
                .unwrap_or(0);

            if total_changes > 0 && total_changes <= 6 {
                let diff_lines = get_inline_diff(file);
                for line in diff_lines {
                    println!("         {}", line);
                }
            }
        }
    }
}
