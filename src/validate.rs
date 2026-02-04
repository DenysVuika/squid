use log::{debug, warn};
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PathValidationError {
    #[error("Path is not allowed: {0}")]
    PathNotAllowed(String),
    #[error("Path does not exist: {0}")]
    PathDoesNotExist(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Path is ignored by .squidignore: {0}")]
    PathIgnored(String),
}

pub struct PathValidator {
    whitelist: Vec<PathBuf>,
    blacklist: Vec<PathBuf>,
    ignore_patterns: Vec<String>,
}

impl PathValidator {
    /// Create a new PathValidator with default whitelist/blacklist
    pub fn new() -> Self {
        Self::with_ignore_file(None)
    }

    /// Create a new PathValidator with optional custom ignore patterns
    pub fn with_ignore_file(ignore_patterns: Option<Vec<String>>) -> Self {
        // Default whitelist: current directory and subdirectories
        let whitelist = vec![
            PathBuf::from("."),
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        ];

        // Default blacklist: sensitive system paths
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

        // Add home directory sensitive paths if home dir exists
        if let Ok(home) = std::env::var("HOME") {
            let home_path = PathBuf::from(&home);
            blacklist.push(home_path.join(".ssh"));
            blacklist.push(home_path.join(".gnupg"));
            blacklist.push(home_path.join(".aws"));
            blacklist.push(home_path.join(".config/gcloud"));
        }

        // Windows-specific blacklist
        #[cfg(target_os = "windows")]
        {
            blacklist.push(PathBuf::from("C:\\Windows"));
            blacklist.push(PathBuf::from("C:\\Program Files"));
            blacklist.push(PathBuf::from("C:\\Program Files (x86)"));
        }

        let ignore_patterns = ignore_patterns.unwrap_or_default();

        debug!(
            "PathValidator initialized with {} whitelist entries, {} blacklist entries, {} ignore patterns",
            whitelist.len(),
            blacklist.len(),
            ignore_patterns.len()
        );

        Self {
            whitelist,
            blacklist,
            ignore_patterns,
        }
    }

    /// Load ignore patterns from .squidignore file if it exists
    pub fn load_ignore_patterns() -> Vec<String> {
        let ignore_file = PathBuf::from(".squidignore");
        if !ignore_file.exists() {
            debug!("No .squidignore file found");
            return Vec::new();
        }

        match fs::read_to_string(&ignore_file) {
            Ok(content) => {
                let patterns: Vec<String> = content
                    .lines()
                    .map(|line| line.trim())
                    .filter(|line| !line.is_empty() && !line.starts_with('#'))
                    .map(String::from)
                    .collect();

                debug!("Loaded {} patterns from .squidignore", patterns.len());
                patterns
            }
            Err(e) => {
                warn!("Failed to read .squidignore: {}", e);
                Vec::new()
            }
        }
    }

    /// Validates a path against whitelist, blacklist, and ignore rules
    pub fn validate(&self, path: &Path) -> Result<PathBuf, PathValidationError> {
        debug!("Validating path: {}", path.display());

        // Try to canonicalize, but don't fail if path doesn't exist yet (for write operations)
        let canonical_path = if path.exists() {
            fs::canonicalize(path).map_err(|e| {
                PathValidationError::PermissionDenied(format!("{}: {}", path.display(), e))
            })?
        } else {
            // For non-existent paths, resolve relative to current directory
            let absolute = if path.is_absolute() {
                path.to_path_buf()
            } else {
                std::env::current_dir()
                    .unwrap_or_else(|_| PathBuf::from("."))
                    .join(path)
            };

            // Clean the path (remove . and ..)
            Self::normalize_path(&absolute)
        };

        debug!("Canonical path: {}", canonical_path.display());

        // Check blacklist first
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

        // Check whitelist
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

        // Check ignore patterns
        if self.is_ignored(&canonical_path) {
            return Err(PathValidationError::PathIgnored(
                canonical_path.display().to_string(),
            ));
        }

        debug!("Path validation successful: {}", canonical_path.display());
        Ok(canonical_path)
    }

    /// Normalize a path by resolving . and .. components
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

    /// Check if a path matches any ignore pattern
    fn is_ignored(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        for pattern in &self.ignore_patterns {
            if self.matches_pattern(&path_str, pattern) {
                debug!("Path {} matched ignore pattern: {}", path_str, pattern);
                return true;
            }
        }

        false
    }

    /// Check if a path matches a glob-like pattern
    fn matches_pattern(&self, path: &str, pattern: &str) -> bool {
        // If pattern has no path separator, match against filename only
        // This allows ".env" to match "/path/to/.env" like .gitignore does
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

        // For patterns with paths, match against full path
        let regex_pattern = Self::glob_to_regex(pattern);

        if let Ok(regex) = Regex::new(&regex_pattern) {
            regex.is_match(path)
        } else {
            warn!("Invalid pattern: {}", pattern);
            false
        }
    }

    /// Convert a simple glob pattern to regex
    fn glob_to_regex(pattern: &str) -> String {
        let mut regex = String::from("^");
        let mut chars = pattern.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '*' => {
                    if chars.peek() == Some(&'*') {
                        // ** matches any path component
                        chars.next();
                        regex.push_str(".*");
                    } else {
                        // * matches anything except /
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

    /// Add a custom whitelist path
    pub fn add_whitelist(&mut self, path: PathBuf) {
        debug!("Adding to whitelist: {}", path.display());
        self.whitelist.push(path);
    }

    /// Add a custom blacklist path
    pub fn add_blacklist(&mut self, path: PathBuf) {
        debug!("Adding to blacklist: {}", path.display());
        self.blacklist.push(path);
    }
}

impl Default for PathValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_to_regex() {
        assert_eq!(PathValidator::glob_to_regex("*.txt"), "^[^/]*\\.txt$");
        assert_eq!(PathValidator::glob_to_regex("**/*.rs"), "^.*/[^/]*\\.rs$");
        assert_eq!(PathValidator::glob_to_regex("target/"), "^target/$");
    }

    #[test]
    fn test_matches_pattern() {
        let validator = PathValidator::new();

        assert!(validator.matches_pattern("test.txt", "*.txt"));
        assert!(validator.matches_pattern("src/main.rs", "**/*.rs"));
        assert!(!validator.matches_pattern("test.rs", "*.txt"));
    }

    #[test]
    fn test_normalize_path() {
        let path = PathBuf::from("/home/user/../other/./file.txt");
        let normalized = PathValidator::normalize_path(&path);

        // Should remove .. and .
        assert!(normalized.to_string_lossy().contains("other"));
        assert!(!normalized.to_string_lossy().contains(".."));
        assert!(!normalized.to_string_lossy().contains("/./"));
    }

    #[test]
    fn test_ignore_patterns() {
        let patterns = vec![
            "**/*.log".to_string(),
            "**/target/**".to_string(),
            "**/node_modules/**".to_string(),
            ".env".to_string(),
            "*.tmp".to_string(),
        ];
        let validator = PathValidator::with_ignore_file(Some(patterns));

        // These would be ignored (full path patterns)
        assert!(validator.is_ignored(Path::new("/some/path/debug.log")));
        assert!(validator.is_ignored(Path::new("/project/target/debug/binary")));
        assert!(validator.is_ignored(Path::new("/app/node_modules/package/index.js")));

        // These would be ignored (filename-only patterns)
        assert!(validator.is_ignored(Path::new("/project/.env")));
        assert!(validator.is_ignored(Path::new("/some/path/test.tmp")));

        // These would not be ignored
        assert!(!validator.is_ignored(Path::new("/some/path/file.txt")));
    }

    #[test]
    fn test_env_file_blocked() {
        let patterns = vec![".env".to_string()];
        let validator = PathValidator::with_ignore_file(Some(patterns));

        // .env in current directory should be blocked
        let current_dir = std::env::current_dir().unwrap();
        let env_path = current_dir.join(".env");
        assert!(validator.is_ignored(&env_path));

        // .env in any subdirectory should also be blocked
        assert!(validator.is_ignored(Path::new("/project/.env")));
        assert!(validator.is_ignored(Path::new("/home/user/project/.env")));
        assert!(validator.is_ignored(Path::new("./subdir/.env")));
    }
}
