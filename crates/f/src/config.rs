use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

const DEFAULT_EDITOR: &str = "vim";
const DEFAULT_ID_CHARS: &str = "dfghklsa";

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Config {
    pub editor: String,
    pub id_chars: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            editor: DEFAULT_EDITOR.to_string(),
            id_chars: DEFAULT_ID_CHARS.to_string(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let config_path = Self::config_path();
        match config_path {
            Some(path) if path.exists() => Self::load_from_file(&path),
            _ => Self::default(),
        }
    }

    pub fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("f.toml"))
    }

    fn load_from_file(path: &PathBuf) -> Self {
        match fs::read_to_string(path) {
            Ok(content) => match toml::from_str(&content) {
                Ok(config) => config,
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to parse config file {}: {}",
                        path.display(),
                        e
                    );
                    Self::default()
                }
            },
            Err(e) => {
                eprintln!(
                    "Warning: Failed to read config file {}: {}",
                    path.display(),
                    e
                );
                Self::default()
            }
        }
    }

    pub fn editor(&self) -> String {
        std::env::var("EDITOR").unwrap_or_else(|_| self.editor.clone())
    }

    pub fn id_chars(&self) -> Vec<char> {
        let chars: Vec<char> = self.id_chars.chars().collect();
        if chars.len() >= 2 {
            chars
        } else {
            DEFAULT_ID_CHARS.chars().collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.editor, "vim");
        assert_eq!(config.id_chars, "dfghklsa");
    }

    #[test]
    fn test_id_chars_valid() {
        let config = Config {
            editor: "vim".to_string(),
            id_chars: "abc".to_string(),
        };
        assert_eq!(config.id_chars(), vec!['a', 'b', 'c']);
    }

    #[test]
    fn test_id_chars_too_short_uses_default() {
        let config = Config {
            editor: "vim".to_string(),
            id_chars: "a".to_string(),
        };
        assert_eq!(
            config.id_chars(),
            DEFAULT_ID_CHARS.chars().collect::<Vec<_>>()
        );
    }
}
