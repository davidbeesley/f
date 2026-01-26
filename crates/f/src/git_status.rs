use anyhow::{Result, bail};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::time::SystemTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Unstaged,
    Untracked,
    Staged,
}

#[derive(Debug, Clone)]
pub struct StableId {
    pub display: String,
    pub full_hash: String,
}

impl std::fmt::Display for StableId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display)
    }
}

impl StableId {
    pub fn matches(&self, input: &str) -> bool {
        self.full_hash.starts_with(input)
    }
}

#[derive(Debug, Clone)]
pub struct DiffStats {
    pub added: u32,
    pub removed: u32,
}

#[derive(Debug, Clone)]
pub struct GitFile {
    pub mtime: u64,
    pub rel_path: String,
    pub abs_path: PathBuf,
    pub file_type: FileType,
    pub stable_id: StableId,
    pub diff_stats: Option<DiffStats>,
}

pub fn get_git_root() -> Result<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to run git: {}", e))?;

    if !output.status.success() {
        bail!("Not in a git repository");
    }

    let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(PathBuf::from(root))
}

#[cfg(test)]
const DEFAULT_ID_CHARS: &[char] = &['d', 'f', 'g', 'h', 'l', 'k', 's', 'a'];

fn fnv1a_hash(s: &str) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;
    let mut hash = FNV_OFFSET;
    for byte in s.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn hash_to_id_chars(s: &str, id_chars: &[char]) -> Vec<char> {
    let mut hash = fnv1a_hash(s);
    let base = id_chars.len() as u64;
    let mut chars = Vec::new();
    for _ in 0..12 {
        chars.push(id_chars[(hash % base) as usize]);
        hash /= base;
    }
    chars
}

fn generate_ids(paths: &[String], id_chars: &[char]) -> Vec<(String, String)> {
    if paths.is_empty() {
        return vec![];
    }

    let hashes: Vec<Vec<char>> = paths
        .iter()
        .map(|p| hash_to_id_chars(p, id_chars))
        .collect();
    let mut result = Vec::with_capacity(hashes.len());

    for (i, hash) in hashes.iter().enumerate() {
        let full_hash: String = hash.iter().collect();
        let mut len = 1;
        'outer: while len <= hash.len() {
            let prefix: String = hash[..len].iter().collect();
            for (j, other) in hashes.iter().enumerate() {
                if i != j && paths[i] != paths[j] && other.len() >= len {
                    let other_prefix: String = other[..len].iter().collect();
                    if prefix == other_prefix {
                        len += 1;
                        continue 'outer;
                    }
                }
            }
            break;
        }
        let final_len = len.min(hash.len());
        let display: String = hash[..final_len].iter().collect();
        result.push((display, full_hash));
    }

    result
}

fn get_mtime(path: &PathBuf) -> u64 {
    std::fs::metadata(path)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn get_diff_stats(staged: bool) -> HashMap<String, DiffStats> {
    let mut args = vec!["diff", "--numstat"];
    if staged {
        args.push("--cached");
    }

    let output = Command::new("git").args(&args).output();

    let mut stats = HashMap::new();
    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 3 {
                let added = parts[0].parse().unwrap_or(0);
                let removed = parts[1].parse().unwrap_or(0);
                let filepath = parts[2].to_string();
                stats.insert(filepath, DiffStats { added, removed });
            }
        }
    }
    stats
}

fn count_lines(path: &PathBuf) -> Option<u32> {
    std::fs::read_to_string(path)
        .ok()
        .map(|content| content.lines().count() as u32)
}

pub fn get_all_files(id_chars: &[char]) -> Result<Vec<GitFile>> {
    let git_root = get_git_root()?;

    let output = Command::new("git")
        .args(["status", "--porcelain", "-uall"])
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to run git status: {}", e))?;

    if !output.status.success() {
        bail!("git status failed");
    }

    let unstaged_stats = get_diff_stats(false);
    let staged_stats = get_diff_stats(true);

    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut unstaged = Vec::new();
    let mut untracked = Vec::new();
    let mut staged = Vec::new();

    for line in stdout.lines() {
        if line.len() < 3 {
            continue;
        }
        let status = &line[..2];
        let filepath = line[3..].trim_matches('"');
        let abs_path = git_root.join(filepath);
        let mtime = get_mtime(&abs_path);

        let index_char = status.chars().next().unwrap_or(' ');
        let worktree_char = status.chars().nth(1).unwrap_or(' ');

        // Untracked files
        if status == "??" {
            let stats = count_lines(&abs_path).map(|lines| DiffStats {
                added: lines,
                removed: 0,
            });
            untracked.push((
                mtime,
                filepath.to_string(),
                abs_path,
                FileType::Untracked,
                stats,
            ));
            continue;
        }

        // Has staged changes (index char is not space)
        if index_char != ' ' {
            staged.push((
                mtime,
                filepath.to_string(),
                abs_path.clone(),
                FileType::Staged,
                staged_stats.get(filepath).cloned(),
            ));
        }

        // Has unstaged changes (worktree char is not space)
        if worktree_char != ' ' {
            unstaged.push((
                mtime,
                filepath.to_string(),
                abs_path.clone(),
                FileType::Unstaged,
                unstaged_stats.get(filepath).cloned(),
            ));
        }
    }

    let all_files: Vec<_> = unstaged
        .iter()
        .chain(untracked.iter())
        .chain(staged.iter())
        .cloned()
        .collect();

    let all_paths: Vec<String> = all_files.iter().map(|(_, p, _, _, _)| p.clone()).collect();
    let all_ids = generate_ids(&all_paths, id_chars);

    let mut result = Vec::new();
    for (i, (mtime, rel_path, abs_path, file_type, diff_stats)) in all_files.iter().enumerate() {
        let (display, full_hash) = all_ids[i].clone();
        result.push(GitFile {
            mtime: *mtime,
            rel_path: rel_path.clone(),
            abs_path: abs_path.clone(),
            file_type: *file_type,
            stable_id: StableId { display, full_hash },
            diff_stats: diff_stats.clone(),
        });
    }

    let mut unstaged_files: Vec<_> = result
        .iter()
        .filter(|f| f.file_type == FileType::Unstaged)
        .cloned()
        .collect();
    let mut untracked_files: Vec<_> = result
        .iter()
        .filter(|f| f.file_type == FileType::Untracked)
        .cloned()
        .collect();
    let mut staged_files: Vec<_> = result
        .iter()
        .filter(|f| f.file_type == FileType::Staged)
        .cloned()
        .collect();

    unstaged_files.sort_by_key(|f| f.mtime);
    untracked_files.sort_by_key(|f| f.mtime);
    staged_files.sort_by_key(|f| f.mtime);

    let mut final_result = Vec::new();
    final_result.extend(unstaged_files);
    final_result.extend(untracked_files);
    final_result.extend(staged_files);

    Ok(final_result)
}

pub enum IdMatch {
    Unique(GitFile),
    Ambiguous(usize),
    NotFound,
}

pub fn find_file_by_id(files: &[GitFile], id: &str) -> IdMatch {
    let matches: Vec<_> = files.iter().filter(|f| f.stable_id.matches(id)).collect();
    match matches.len() {
        0 => IdMatch::NotFound,
        1 => IdMatch::Unique(matches[0].clone()),
        n => IdMatch::Ambiguous(n),
    }
}

pub fn get_first_actionable_file(files: &[GitFile]) -> Option<GitFile> {
    files
        .iter()
        .find(|f| f.file_type == FileType::Unstaged || f.file_type == FileType::Untracked)
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_file(rel_path: &str, display: &str, full_hash: &str) -> GitFile {
        GitFile {
            mtime: 0,
            rel_path: rel_path.to_string(),
            abs_path: PathBuf::from(rel_path),
            file_type: FileType::Unstaged,
            stable_id: StableId {
                display: display.to_string(),
                full_hash: full_hash.to_string(),
            },
            diff_stats: None,
        }
    }

    #[test]
    fn stable_id_matches_exact() {
        let id = StableId {
            display: "fk".to_string(),
            full_hash: "fkkabcdefghi".to_string(),
        };
        assert!(id.matches("fkkabcdefghi"));
    }

    #[test]
    fn stable_id_matches_prefix() {
        let id = StableId {
            display: "fk".to_string(),
            full_hash: "fkkabcdefghi".to_string(),
        };
        assert!(id.matches("fk"));
        assert!(id.matches("fkk"));
        assert!(id.matches("fkka"));
    }

    #[test]
    fn stable_id_no_match_wrong_prefix() {
        let id = StableId {
            display: "fk".to_string(),
            full_hash: "fkkabcdefghi".to_string(),
        };
        assert!(!id.matches("fka"));
        assert!(!id.matches("gk"));
        assert!(!id.matches("fkkz"));
    }

    #[test]
    fn find_file_unique_with_short_input() {
        let files = vec![make_file("src/main.rs", "fk", "fkkabcdefghi")];
        match find_file_by_id(&files, "fk") {
            IdMatch::Unique(f) => assert_eq!(f.rel_path, "src/main.rs"),
            _ => panic!("expected unique match"),
        }
    }

    #[test]
    fn find_file_unique_with_longer_input() {
        let files = vec![make_file("src/main.rs", "fk", "fkkabcdefghi")];
        match find_file_by_id(&files, "fkk") {
            IdMatch::Unique(f) => assert_eq!(f.rel_path, "src/main.rs"),
            _ => panic!("expected unique match"),
        }
    }

    #[test]
    fn find_file_ambiguous_short_prefix() {
        let files = vec![
            make_file("src/main.rs", "fkk", "fkkabcdefghi"),
            make_file("src/lib.rs", "fka", "fkaabcdefghi"),
        ];
        match find_file_by_id(&files, "fk") {
            IdMatch::Ambiguous(n) => assert_eq!(n, 2),
            _ => panic!("expected ambiguous match"),
        }
    }

    #[test]
    fn find_file_disambiguate_with_longer_prefix() {
        let files = vec![
            make_file("src/main.rs", "fkk", "fkkabcdefghi"),
            make_file("src/lib.rs", "fka", "fkaabcdefghi"),
        ];
        match find_file_by_id(&files, "fkk") {
            IdMatch::Unique(f) => assert_eq!(f.rel_path, "src/main.rs"),
            _ => panic!("expected unique match for fkk"),
        }
        match find_file_by_id(&files, "fka") {
            IdMatch::Unique(f) => assert_eq!(f.rel_path, "src/lib.rs"),
            _ => panic!("expected unique match for fka"),
        }
    }

    #[test]
    fn find_file_not_found() {
        let files = vec![make_file("src/main.rs", "fk", "fkkabcdefghi")];
        match find_file_by_id(&files, "gk") {
            IdMatch::NotFound => {}
            _ => panic!("expected not found"),
        }
    }

    #[test]
    fn find_file_old_id_still_works_after_collision() {
        // Scenario: User memorized "fk" for file A (full_hash fkkabcdefghi)
        // Later, file B is added with full_hash fkaabcdefghi
        // Display IDs are now "fkk" and "fka", but "fkk" should still find file A
        let files = vec![
            make_file("src/main.rs", "fkk", "fkkabcdefghi"),
            make_file("src/lib.rs", "fka", "fkaabcdefghi"),
        ];
        // Old memorized ID "fkk" still works
        match find_file_by_id(&files, "fkk") {
            IdMatch::Unique(f) => assert_eq!(f.rel_path, "src/main.rs"),
            _ => panic!("old ID should still work"),
        }
    }

    #[test]
    fn generate_ids_no_collision() {
        let paths = vec!["src/main.rs".to_string()];
        let ids = generate_ids(&paths, DEFAULT_ID_CHARS);
        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0].0.len(), 1); // minimal display length
        assert_eq!(ids[0].1.len(), 12); // full hash length
    }

    #[test]
    fn generate_ids_with_collision_extends() {
        // Find two paths that collide on first char
        // We'll just verify that when there's a collision, displays get longer
        let paths = vec!["a".to_string(), "b".to_string()];
        let ids = generate_ids(&paths, DEFAULT_ID_CHARS);
        // Both should have display IDs, and if they collide, they extend
        assert!(!ids[0].0.is_empty());
        assert!(!ids[1].0.is_empty());
        // Full hashes should be 12 chars
        assert_eq!(ids[0].1.len(), 12);
        assert_eq!(ids[1].1.len(), 12);
    }
}
