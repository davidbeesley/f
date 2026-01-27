# f

[![CI](https://github.com/davidbeesley/f/actions/workflows/ci.yml/badge.svg)](https://github.com/davidbeesley/f/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
![Version](https://img.shields.io/badge/version-0.1.1-blue)
![Linux](https://img.shields.io/badge/Linux-supported-green)
![macOS](https://img.shields.io/badge/macOS-supported-green)
![Windows](https://img.shields.io/badge/Windows-unsupported-red)
![Rust](https://img.shields.io/badge/rust-2024-orange)

A keyboard-driven git file manager for the terminal.

![demo](demo/demo-0.1.1.gif)

## The Problem

When working with git, common workflows like staging files, viewing diffs, and editing changed files require typing full file paths repeatedly. Tab completion helps, but with many changed files it's still slow. 
This can be exacerbated by AI driven workflows as the amount of time reviewing grows substantially relative to writing code. 

## The Solution

`f` provides:

- **Stable file IDs** - Each changed file gets a short, memorable ID (like `df`, `gk`, `hls`) that is always safe to use - it will either refer to the same file or error if that file is gone or more precision is needed
- **Quick commands** - Stage, diff, or edit files using their ID: `f df a` (add), `f gk d` (diff)
- **Interactive mode** - Keyboard-driven file picker with vim-like navigation
- **Inline diffs** - Small changes shown directly in the file list

## Installation

```bash
cargo install --git https://github.com/davidbeesley/f
```

## Usage

```
f              List changed files with IDs
f <id> a       Stage file (git add)
f <id> d       Diff file
f <id> sd      Staged diff
f <id> e       Edit file in $EDITOR
f c <msg>      Commit (no quotes needed: f c fix typo)
f p            Push to remote
f i            Interactive file picker
f w [-i N]     Watch mode (default: 2s refresh)
```

### Examples

**List changed files with `f`:**

```
$ f
── Unstaged ──
  df    src/config.rs +2/-1
         -    let timeout = 10;
         +    let timeout = 30;
  gk    src/main.rs +15/-3

── Untracked ──
  hls   notes.txt +5

── Staged ──
  ak    src/lib.rs +8/-2
```

Small changes (≤6 lines) show inline diffs. Larger changes just show the line counts.

**View a diff with `f <id> d`:**

```
$ f df d
diff --git a/src/config.rs b/src/config.rs
index 1234567..abcdefg 100644
--- a/src/config.rs
+++ b/src/config.rs
@@ -10,7 +10,8 @@ impl Config {
     pub fn new() -> Self {
-        let timeout = 10;
+        let timeout = 30;
+        let retries = 3;
         Self { timeout, retries }
     }
 }
```

**Stage and commit:**

```
$ f df a
Adding: src/config.rs

$ f c bump timeout to 30s
[main abc1234] bump timeout to 30s
 1 file changed, 2 insertions(+), 1 deletion(-)
```

### Sort Order

Files are sorted by modification time, with **least recently modified first**. This puts stale changes at the top where you're most likely to want to deal with them, while files you're actively editing stay at the bottom.

### Interactive Mode

Run `f i` to enter interactive mode:

1. Files are listed with key combinations (d, f, g, h, k, l, s, a)
2. Type the key combo to select a file
3. Choose an action: (a)dd, (d)iff, (s)taged diff, (e)dit

## How It Works

File IDs are generated using FNV-1a hashing of the file path, converted to a memorable character set (`d`, `f`, `g`, `h`, `k`, `l`, `s`, `a`). IDs automatically extend if there are collisions, ensuring uniqueness while staying short.

The ID system is designed to be safe for scripting and muscle memory:
- An ID you used before will always match the same file (based on the full hash)
- If the file is gone, you get a clear error instead of accidentally operating on a different file
- If your ID has become ambiguous, you're prompted to be more specific

## Configuration

Config file location:
- **Linux**: `~/.config/f.toml`
- **macOS**: `~/Library/Application Support/f.toml`

```toml
editor = "vim"           # Editor for 'f <id> e' (overridden by $EDITOR)
id_chars = "dfghklsa"    # Characters used for file IDs
```

### Editor

`f` checks `$EDITOR` first, then falls back to the config file, then defaults to `vim`.

### ID Characters

The default character set (`dfghklsa`) uses home-row friendly characters chosen for:
- Easy typing without moving hands from home position
- Memorable combinations (e.g., `df`, `gh`, `ask`)
- No conflicts with common shell characters

You can customize this to any set of at least 2 characters. Shorter character sets produce longer IDs; larger sets produce shorter IDs.

## Better Diffs with Delta

For improved diff display, install [delta](https://github.com/dandavison/delta) and configure git to use it:

```ini
# ~/.gitconfig
[core]
    pager = delta
[interactive]
    diffFilter = delta --color-only
```

## Requirements

- Unix-like OS (Linux, macOS)
- Git
- Rust 2024 edition (for building from source)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

MIT
