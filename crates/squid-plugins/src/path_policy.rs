use log::{debug, warn};
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors returned by path policy checks.
#[derive(Error, Debug)]
pub enum PathValidationError {
    #[error("Path is not allowed: {0}")]
    PathNotAllowed(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Path is ignored by ignore file: {0}")]
    PathIgnored(String),
}

/// Simple path policy with whitelist, blacklist, and ignore-pattern checks.
pub struct PathValidator {
    whitelist: Vec<PathBuf>,
    blacklist: Vec<PathBuf>,
    ignore_patterns: Vec<String>,
}

impl PathValidator {
    /// Creates a validator with caller-provided ignore patterns.
    pub fn with_ignore_patterns(ignore_patterns: Vec<String>) -> Self {
        let whitelist = vec![
            PathBuf::from("."),
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        ];

        let mut blacklist = vec![
            PathBuf::from("/etc"),
            PathBuf::from("/bin"),
            PathBuf::from("/sbin"),
            PathBuf::from("/usr/bin"),
            PathBuf::from("/usr/sbin"),
            PathBuf::from("/root"),
            PathBuf::from("/var"),
            PathBuf::from("/sys"),
            PathBuf::from("/proc"),
        ];

        if let Ok(home) = std::env::var("HOME") {
            let home_path = PathBuf::from(&home);
            blacklist.push(home_path.join(".ssh"));
            blacklist.push(home_path.join(".gnupg"));
            blacklist.push(home_path.join(".aws"));
            blacklist.push(home_path.join(".config/gcloud"));
        }

        #[cfg(target_os = "windows")]
        {
            blacklist.push(PathBuf::from("C:\\Windows"));
            blacklist.push(PathBuf::from("C:\\Program Files"));
            blacklist.push(PathBuf::from("C:\\Program Files (x86)"));
        }

        Self {
            whitelist,
            blacklist,
            ignore_patterns,
        }
    }

    /// Loads ignore patterns from a file (for example `.squidignore`).
    pub fn load_ignore_patterns(ignore_file_name: &str) -> Vec<String> {
        let ignore_file = PathBuf::from(ignore_file_name);
        if !ignore_file.exists() {
            debug!("No ignore file found: {}", ignore_file_name);
            return Vec::new();
        }

        match fs::read_to_string(&ignore_file) {
            Ok(content) => content
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty() && !line.starts_with('#'))
                .map(String::from)
                .collect(),
            Err(e) => {
                warn!("Failed to read ignore file {}: {}", ignore_file_name, e);
                Vec::new()
            }
        }
    }

    /// Validates a path against blacklist, whitelist, and ignore patterns.
    pub fn validate(&self, path: &Path) -> Result<PathBuf, PathValidationError> {
        let canonical_path = if path.exists() {
            fs::canonicalize(path).map_err(|e| {
                PathValidationError::PermissionDenied(format!("{}: {}", path.display(), e))
            })?
        } else {
            let absolute = if path.is_absolute() {
                path.to_path_buf()
            } else {
                std::env::current_dir()
                    .unwrap_or_else(|_| PathBuf::from("."))
                    .join(path)
            };
            Self::normalize_path(&absolute)
        };

        for blocked in &self.blacklist {
            let blocked_canonical = if blocked.exists() {
                fs::canonicalize(blocked).unwrap_or_else(|_| blocked.clone())
            } else {
                blocked.clone()
            };

            if canonical_path.starts_with(&blocked_canonical) {
                return Err(PathValidationError::PathNotAllowed(format!(
                    "Path is in blacklisted directory: {}",
                    canonical_path.display()
                )));
            }
        }

        let mut is_whitelisted = false;
        for allowed in &self.whitelist {
            let allowed_canonical = if allowed.exists() {
                fs::canonicalize(allowed).unwrap_or_else(|_| allowed.clone())
            } else {
                allowed.clone()
            };

            if canonical_path.starts_with(&allowed_canonical) {
                is_whitelisted = true;
                break;
            }
        }

        if !is_whitelisted {
            return Err(PathValidationError::PathNotAllowed(format!(
                "Path is not in whitelisted directory: {}",
                canonical_path.display()
            )));
        }

        if self.is_ignored(&canonical_path) {
            return Err(PathValidationError::PathIgnored(
                canonical_path.display().to_string(),
            ));
        }

        Ok(canonical_path)
    }

    fn normalize_path(path: &Path) -> PathBuf {
        let mut components = Vec::new();
        for component in path.components() {
            match component {
                std::path::Component::ParentDir => {
                    components.pop();
                }
                std::path::Component::CurDir => {}
                _ => components.push(component),
            }
        }
        components.iter().collect()
    }

    fn is_ignored(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        for pattern in &self.ignore_patterns {
            if self.matches_pattern(&path_str, pattern) {
                return true;
            }
        }

        false
    }

    fn matches_pattern(&self, path: &str, pattern: &str) -> bool {
        if !pattern.contains('/') && !pattern.starts_with("**") {
            if let Some(filename) = std::path::Path::new(path).file_name() {
                let filename_str = filename.to_string_lossy();
                let regex_pattern = Self::glob_to_regex(pattern);
                if let Ok(regex) = Regex::new(&regex_pattern) {
                    return regex.is_match(&filename_str);
                }
            }
            return false;
        }

        let regex_pattern = Self::glob_to_regex(pattern);
        Regex::new(&regex_pattern)
            .map(|regex| regex.is_match(path))
            .unwrap_or(false)
    }

    fn glob_to_regex(pattern: &str) -> String {
        let mut regex = String::from("^");
        let mut chars = pattern.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '*' => {
                    if chars.peek() == Some(&'*') {
                        chars.next();
                        regex.push_str(".*");
                    } else {
                        regex.push_str("[^/]*");
                    }
                }
                '?' => regex.push_str("[^/]"),
                '.' => regex.push_str("\\."),
                '/' => regex.push('/'),
                '+' | '^' | '$' | '(' | ')' | '[' | ']' | '{' | '}' | '|' | '\\' => {
                    regex.push('\\');
                    regex.push(ch);
                }
                _ => regex.push(ch),
            }
        }

        regex.push('$');
        regex
    }
}
