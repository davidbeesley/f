mod config;
mod display;
mod git_status;

use clap::builder::styling::{AnsiColor, Color, Styles};
use clap::{Parser, Subcommand};
use std::os::unix::process::CommandExt;
use std::process::{self, Command};

use config::Config;
use git_status::{
    FileType, GitFile, IdMatch, find_file_by_id, get_all_files, get_first_actionable_file,
};

fn help_styles() -> Styles {
    Styles::styled()
        .header(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Green))),
        )
        .usage(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Green))),
        )
        .literal(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Cyan))),
        )
        .placeholder(anstyle::Style::new().fg_color(Some(Color::Ansi(AnsiColor::Cyan))))
        .error(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Red))),
        )
        .valid(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Green))),
        )
        .invalid(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Yellow))),
        )
}

#[derive(Parser)]
#[command(name = "f")]
#[command(version)]
#[command(about = "A keyboard-driven git file manager", long_about = None)]
#[command(styles = help_styles())]
#[command(
    after_help = "ID-first syntax:\n  f <id> <cmd>   Run command on file (e.g., f df d, f gk a)"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(global = true, short, long, help = "Enable verbose output")]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    #[command(visible_alias = "l", about = "List changed files")]
    List,
    #[command(visible_alias = "d", about = "Show diff for a file")]
    Diff {
        #[arg(help = "File ID (defaults to first unstaged)")]
        id: Option<String>,
    },
    #[command(visible_alias = "sd", about = "Show staged diff for a file")]
    StagedDiff {
        #[arg(help = "File ID (defaults to first staged)")]
        id: Option<String>,
    },
    #[command(visible_alias = "a", about = "Stage a file")]
    Add {
        #[arg(help = "File ID (defaults to first unstaged)")]
        id: Option<String>,
    },
    #[command(visible_aliases = ["e", "v"], about = "Edit a file in $EDITOR")]
    Edit {
        #[arg(help = "File ID (defaults to first unstaged)")]
        id: Option<String>,
    },
    #[command(visible_alias = "c", about = "Commit staged changes")]
    Commit {
        #[arg(help = "Commit message")]
        message: Vec<String>,
    },
    #[command(visible_alias = "p", about = "Push to remote")]
    Push,
    #[command(visible_alias = "i", about = "Interactive file picker")]
    Interactive,
    #[command(visible_alias = "w", about = "Watch file status")]
    Watch {
        #[arg(short, long, default_value = "2", help = "Refresh interval in seconds")]
        interval: u32,
    },
}

fn get_editor(config: &Config) -> String {
    config.editor()
}

enum ResolveResult {
    Found(GitFile),
    Ambiguous(usize),
    NotFound,
    Error(String),
}

fn resolve_file(id: Option<String>, config: &Config) -> ResolveResult {
    let files = match get_all_files(&config.id_chars()) {
        Ok(f) => f,
        Err(e) => return ResolveResult::Error(e.to_string()),
    };
    match id {
        Some(id) => match find_file_by_id(&files, &id) {
            IdMatch::Unique(f) => ResolveResult::Found(f),
            IdMatch::Ambiguous(n) => ResolveResult::Ambiguous(n),
            IdMatch::NotFound => ResolveResult::NotFound,
        },
        None => match get_first_actionable_file(&files) {
            Some(f) => ResolveResult::Found(f),
            None => ResolveResult::NotFound,
        },
    }
}

fn exec_git(args: &[&str]) -> ! {
    let err = Command::new("git").args(args).exec();
    eprintln!("Failed to exec git: {}", err);
    process::exit(1);
}

fn exec_editor(path: &str, config: &Config) -> ! {
    let editor = get_editor(config);
    let err = Command::new(&editor).arg(path).exec();
    eprintln!("Failed to exec {}: {}", editor, err);
    process::exit(1);
}

fn require_file(result: ResolveResult) -> GitFile {
    match result {
        ResolveResult::Found(f) => f,
        ResolveResult::Ambiguous(n) => {
            eprintln!("ID matches {} files - be more specific", n);
            process::exit(1);
        }
        ResolveResult::NotFound => {
            eprintln!("No matching file found");
            process::exit(1);
        }
        ResolveResult::Error(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

fn cmd_list(config: &Config) {
    match get_all_files(&config.id_chars()) {
        Ok(files) => display::list_files(&files),
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

fn cmd_diff(id: Option<String>, config: &Config) -> ! {
    let file = require_file(resolve_file(id, config));
    if file.file_type == FileType::Untracked {
        exec_git(&[
            "diff",
            "--no-index",
            "/dev/null",
            &file.abs_path.to_string_lossy(),
        ])
    } else {
        exec_git(&["diff", &file.abs_path.to_string_lossy()])
    }
}

fn cmd_staged_diff(id: Option<String>, config: &Config) -> ! {
    let file = require_file(resolve_file(id, config));
    exec_git(&["diff", "--staged", &file.abs_path.to_string_lossy()])
}

fn cmd_add(id: Option<String>, config: &Config) -> ! {
    let file = require_file(resolve_file(id, config));
    println!("Adding: {}", file.rel_path);
    exec_git(&["add", &file.abs_path.to_string_lossy()])
}

fn cmd_edit(id: Option<String>, config: &Config) -> ! {
    let file = require_file(resolve_file(id, config));
    exec_editor(&file.abs_path.to_string_lossy(), config)
}

fn cmd_commit(message: Vec<String>) -> ! {
    if message.is_empty() {
        eprintln!("Commit message required");
        process::exit(1);
    }
    let msg = message.join(" ");
    exec_git(&["commit", "-m", &msg])
}

fn cmd_push() -> ! {
    exec_git(&["push"])
}

fn cmd_watch(interval: u32) -> ! {
    let exe = std::env::current_exe().unwrap_or_else(|_| "f".into());
    let interval_arg = format!("-n{}", interval);
    let err = Command::new("watch")
        .args([&interval_arg, "-c", &exe.to_string_lossy()])
        .env("CLICOLOR_FORCE", "1")
        .exec();
    eprintln!("Failed to exec watch: {}", err);
    process::exit(1);
}

fn cmd_interactive(config: &Config) {
    match interactive::run(config) {
        Ok(()) => {}
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

fn is_file_id(s: &str, config: &Config) -> bool {
    let id_chars = config.id_chars();
    !s.is_empty() && s.chars().all(|c| id_chars.contains(&c))
}

fn handle_id_first(id: &str, action: Option<&str>, config: &Config) {
    let files = match get_all_files(&config.id_chars()) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };

    let file = match find_file_by_id(&files, id) {
        IdMatch::Unique(f) => f,
        IdMatch::Ambiguous(n) => {
            eprintln!("ID '{}' matches {} files - be more specific", id, n);
            process::exit(1);
        }
        IdMatch::NotFound => {
            eprintln!("No file matches ID: {}", id);
            process::exit(1);
        }
    };

    match action {
        Some("a" | "add") => {
            println!("Adding: {}", file.rel_path);
            exec_git(&["add", &file.abs_path.to_string_lossy()]);
        }
        Some("d" | "diff") => {
            if file.file_type == FileType::Untracked {
                exec_git(&[
                    "diff",
                    "--no-index",
                    "/dev/null",
                    &file.abs_path.to_string_lossy(),
                ]);
            } else {
                exec_git(&["diff", &file.abs_path.to_string_lossy()]);
            }
        }
        Some("sd" | "staged-diff") => {
            exec_git(&["diff", "--staged", &file.abs_path.to_string_lossy()]);
        }
        Some("e" | "v" | "edit") => {
            exec_editor(&file.abs_path.to_string_lossy(), config);
        }
        Some(other) => {
            eprintln!("Unknown action: {}", other);
            process::exit(1);
        }
        None => {
            eprintln!("Action required (a, d, sd, e)");
            process::exit(1);
        }
    }
}

mod interactive {
    use crate::config::Config;
    use crate::git_status::{FileType, GitFile, get_all_files, get_git_root};
    use anyhow::{Context, Result};
    use colored::Colorize;
    use crossterm::event::{self, Event, KeyCode, KeyModifiers};
    use crossterm::terminal::{self, ClearType};
    use crossterm::{cursor, execute};
    use std::io::{Write, stdout};
    use std::os::unix::process::CommandExt;
    use std::process::Command;

    macro_rules! raw_println {
        () => {
            print!("\r\n");
            let _ = std::io::stdout().flush();
        };
        ($($arg:tt)*) => {{
            print!($($arg)*);
            print!("\r\n");
            let _ = std::io::stdout().flush();
        }};
    }

    fn generate_keys(n: usize, id_chars: &[char]) -> Vec<String> {
        if n == 0 {
            return vec![];
        }
        let mut length = 1;
        while id_chars.len().pow(length as u32) < n {
            length += 1;
        }

        (0..n)
            .map(|i| {
                let mut key = String::new();
                let mut idx = i;
                for _ in 0..length {
                    key.insert(0, id_chars[idx % id_chars.len()]);
                    idx /= id_chars.len();
                }
                key
            })
            .collect()
    }

    fn clear_screen() {
        let mut stdout = stdout();
        let _ = execute!(
            stdout,
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0)
        );
    }

    fn display_files(files: &[GitFile], keys: &[String], prefix: &str) {
        let matching: Vec<_> = keys
            .iter()
            .zip(files.iter())
            .filter(|(k, _)| k.starts_with(prefix))
            .collect();

        raw_println!("{}", "── Select file ──".yellow());
        if !prefix.is_empty() {
            raw_println!("  Prefix: {}", prefix.cyan());
        }

        let mut last_type: Option<FileType> = None;
        for (key, file) in &matching {
            if last_type != Some(file.file_type) {
                if last_type.is_some() {
                    raw_println!();
                }
                let header = match file.file_type {
                    FileType::Unstaged => "Unstaged".yellow(),
                    FileType::Untracked => "Untracked".green(),
                    FileType::Staged => "Staged".cyan(),
                };
                raw_println!("── {} ──", header);
                last_type = Some(file.file_type);
            }

            let typed = &key[..prefix.len()];
            let remaining = &key[prefix.len()..];
            raw_println!(
                "  {}{}  {}",
                typed.cyan().bold(),
                remaining.cyan(),
                file.rel_path
            );
        }
        raw_println!();
        raw_println!("  {}   quit", "q".dimmed());
    }

    fn display_actions(file: &GitFile) {
        raw_println!();
        raw_println!("{} {}", "Selected:".green(), file.rel_path);
        raw_println!("{}", "── Action ──".yellow());
        raw_println!("  {}  add", "a".cyan());
        raw_println!("  {}  diff", "d".cyan());
        raw_println!("  {}  staged diff", "s".cyan());
        raw_println!("  {}  edit", "e".cyan());
        raw_println!("  {}  quit", "q".dimmed());
    }

    pub fn run(config: &Config) -> Result<()> {
        let id_chars = config.id_chars();
        let files = get_all_files(&id_chars)?;
        if files.is_empty() {
            println!("{}", "No changed files".dimmed());
            return Ok(());
        }

        let keys = generate_keys(files.len(), &id_chars);
        let key_len = keys.first().map(|k| k.len()).unwrap_or(0);

        terminal::enable_raw_mode().context("Terminal error")?;

        let result = (|| -> Result<Option<GitFile>> {
            clear_screen();
            display_files(&files, &keys, "");

            let mut prefix = String::new();
            loop {
                if event::poll(std::time::Duration::from_millis(100)).context("Event error")?
                    && let Event::Key(key_event) = event::read().context("Read error")?
                {
                    if key_event.modifiers.contains(KeyModifiers::CONTROL)
                        && key_event.code == KeyCode::Char('c')
                    {
                        return Ok(None);
                    }

                    match key_event.code {
                        KeyCode::Char('q') => return Ok(None),
                        KeyCode::Char(c) if id_chars.contains(&c) => {
                            prefix.push(c);

                            if prefix.len() == key_len {
                                if let Some(idx) = keys.iter().position(|k| k == &prefix) {
                                    return Ok(Some(files[idx].clone()));
                                }
                                prefix.clear();
                            }

                            let matches: Vec<_> =
                                keys.iter().filter(|k| k.starts_with(&prefix)).collect();
                            if matches.is_empty() {
                                prefix.clear();
                            }

                            clear_screen();
                            display_files(&files, &keys, &prefix);
                        }
                        KeyCode::Esc => {
                            prefix.clear();
                            clear_screen();
                            display_files(&files, &keys, "");
                        }
                        _ => {}
                    }
                }
            }
        })();

        terminal::disable_raw_mode().context("Terminal error")?;

        let selected = result?;
        if let Some(file) = selected {
            clear_screen();
            display_actions(&file);

            terminal::enable_raw_mode().context("Terminal error")?;

            let action_result = (|| -> Result<Option<char>> {
                loop {
                    if event::poll(std::time::Duration::from_millis(100)).context("Event error")?
                        && let Event::Key(key_event) = event::read().context("Read error")?
                    {
                        match key_event.code {
                            KeyCode::Char('q') | KeyCode::Esc => return Ok(None),
                            KeyCode::Char(c @ ('a' | 'd' | 's' | 'e')) => return Ok(Some(c)),
                            _ => {}
                        }
                    }
                }
            })();

            terminal::disable_raw_mode().context("Terminal error")?;

            if let Some(action) = action_result? {
                println!();
                let git_root = get_git_root()?;
                std::env::set_current_dir(&git_root).ok();

                match action {
                    'a' => {
                        println!("Adding: {}", file.rel_path);
                        let _ = Command::new("git")
                            .args(["add", &file.abs_path.to_string_lossy()])
                            .exec();
                    }
                    'd' => {
                        let _ = Command::new("git")
                            .args(["diff", &file.abs_path.to_string_lossy()])
                            .exec();
                    }
                    's' => {
                        let _ = Command::new("git")
                            .args(["diff", "--staged", &file.abs_path.to_string_lossy()])
                            .exec();
                    }
                    'e' => {
                        let editor = config.editor();
                        let _ = Command::new(&editor).arg(&file.abs_path).exec();
                    }
                    _ => {}
                }
            }
        } else {
            clear_screen();
        }

        Ok(())
    }
}

fn main() {
    let config = Config::load();
    let args: Vec<String> = std::env::args().collect();

    if args.len() >= 3 && is_file_id(&args[1], &config) {
        let action = args.get(2).map(|s| s.as_str());
        handle_id_first(&args[1], action, &config);
        return;
    }

    let cli = Cli::parse();

    if cli.verbose {
        env_logger::Builder::new()
            .filter_level(log::LevelFilter::Debug)
            .init();
    }

    match cli.command {
        None | Some(Commands::List) => cmd_list(&config),
        Some(Commands::Diff { id }) => cmd_diff(id, &config),
        Some(Commands::StagedDiff { id }) => cmd_staged_diff(id, &config),
        Some(Commands::Add { id }) => cmd_add(id, &config),
        Some(Commands::Edit { id }) => cmd_edit(id, &config),
        Some(Commands::Commit { message }) => cmd_commit(message),
        Some(Commands::Push) => cmd_push(),
        Some(Commands::Watch { interval }) => cmd_watch(interval),
        Some(Commands::Interactive) => cmd_interactive(&config),
    }
}
