use std::fs;
use std::path::{Path, PathBuf};
use crate::Result;

/// Pattern matcher for .gvcignore
#[derive(Debug, Clone)]
pub struct IgnorePattern {
    pattern: String,
    is_negation: bool,
    is_directory_only: bool,
}

impl IgnorePattern {
    /// Parse a pattern from .gvcignore line
    pub fn parse(line: &str) -> Option<Self> {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            return None;
        }

        let (is_negation, pattern) = if let Some(p) = line.strip_prefix('!') {
            (true, p)
        } else {
            (false, line)
        };

        let (pattern, is_directory_only) = if let Some(p) = pattern.strip_suffix('/') {
            (p, true)
        } else {
            (pattern, false)
        };

        Some(IgnorePattern {
            pattern: pattern.to_string(),
            is_negation,
            is_directory_only,
        })
    }

    /// Check if path matches this pattern
    pub fn matches(&self, path: &Path, is_dir: bool) -> bool {
        if self.is_directory_only && !is_dir {
            return false;
        }

        let path_str = path.to_string_lossy();
        self.glob_match(&self.pattern, &path_str)
    }

    /// Simple glob matching (basic implementation)
    /// Supports: *, **, ?
    fn glob_match(&self, pattern: &str, text: &str) -> bool {
        // Handle ** (match any directory depth)
        if pattern.contains("**") {
            return self.glob_match_recursive(pattern, text);
        }

        let pattern_chars: Vec<char> = pattern.chars().collect();
        let text_chars: Vec<char> = text.chars().collect();

        self.glob_match_simple(&pattern_chars, &text_chars, 0, 0)
    }

    /// Recursive matching for ** patterns
    fn glob_match_recursive(&self, pattern: &str, text: &str) -> bool {
        let parts: Vec<&str> = pattern.split("**").collect();

        if parts.len() == 1 {
            return self.glob_match(parts[0], text);
        }

        // Check if text starts with first part
        if !parts[0].is_empty() && !text.starts_with(parts[0]) {
            return false;
        }

        // Check if text ends with last part
        if !parts[parts.len() - 1].is_empty() && !text.ends_with(parts[parts.len() - 1]) {
            return false;
        }

        // ** matches everything in between
        true
    }

    /// Simple glob matching without **
    fn glob_match_simple(
        &self,
        pattern: &[char],
        text: &[char],
        p_idx: usize,
        t_idx: usize,
    ) -> bool {
        // Both exhausted - match
        if p_idx >= pattern.len() && t_idx >= text.len() {
            return true;
        }

        // Pattern exhausted but text remains - no match
        if p_idx >= pattern.len() {
            return false;
        }

        // Text exhausted but pattern has * - continue
        if t_idx >= text.len() {
            return pattern[p_idx] == '*' && self.glob_match_simple(pattern, text, p_idx + 1, t_idx);
        }

        match pattern[p_idx] {
            '*' => {
                // Try matching 0 or more characters
                self.glob_match_simple(pattern, text, p_idx + 1, t_idx)
                    || self.glob_match_simple(pattern, text, p_idx, t_idx + 1)
            }
            '?' => {
                // Match any single character
                self.glob_match_simple(pattern, text, p_idx + 1, t_idx + 1)
            }
            c => {
                // Exact match
                if c == text[t_idx] {
                    self.glob_match_simple(pattern, text, p_idx + 1, t_idx + 1)
                } else {
                    false
                }
            }
        }
    }
}

/// Ignore file manager
pub struct IgnoreFile {
    patterns: Vec<IgnorePattern>,
}

impl IgnoreFile {
    /// Load .gvcignore from repository root
    pub fn load(repo_root: &Path) -> Result<Self> {
        let ignore_path = repo_root.join(".gvcignore");

        if !ignore_path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&ignore_path)?;
        let patterns = content
            .lines()
            .filter_map(IgnorePattern::parse)
            .collect();

        Ok(IgnoreFile { patterns })
    }

    /// Check if path should be ignored
    pub fn is_ignored(&self, path: &Path, is_dir: bool) -> bool {
        let mut ignored = false;

        for pattern in &self.patterns {
            if pattern.matches(path, is_dir) {
                ignored = !pattern.is_negation;
            }
        }

        ignored
    }

    /// Check if path should be ignored (with metadata check)
    pub fn should_ignore(&self, path: &Path) -> bool {
        let is_dir = path.is_dir();
        self.is_ignored(path, is_dir)
    }
}

impl Default for IgnoreFile {
    fn default() -> Self {
        // Default patterns (always ignore .gvc)
        let patterns = vec![IgnorePattern {
            pattern: ".gvc".to_string(),
            is_negation: false,
            is_directory_only: true,
        }];

        IgnoreFile { patterns }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_parse() {
        let pattern = IgnorePattern::parse("*.txt").unwrap();
        assert_eq!(pattern.pattern, "*.txt");
        assert!(!pattern.is_negation);

        let pattern = IgnorePattern::parse("!important.txt").unwrap();
        assert!(pattern.is_negation);

        let pattern = IgnorePattern::parse("build/").unwrap();
        assert!(pattern.is_directory_only);
    }

    #[test]
    fn test_glob_match() {
        let pattern = IgnorePattern::parse("*.txt").unwrap();
        assert!(pattern.matches(Path::new("file.txt"), false));
        assert!(!pattern.matches(Path::new("file.rs"), false));

        let pattern = IgnorePattern::parse("test?.txt").unwrap();
        assert!(pattern.matches(Path::new("test1.txt"), false));
        assert!(!pattern.matches(Path::new("test12.txt"), false));
    }

    #[test]
    fn test_ignore_file() {
        let ignore = IgnoreFile::default();
        assert!(ignore.is_ignored(Path::new(".gvc"), true));
        assert!(!ignore.is_ignored(Path::new("src"), true));
    }

    #[test]
    fn test_negation() {
        let mut patterns = vec![
            IgnorePattern::parse("*.log").unwrap(),
            IgnorePattern::parse("!important.log").unwrap(),
        ];

        let ignore = IgnoreFile { patterns };

        // Regular .log files are ignored
        assert!(ignore.is_ignored(Path::new("debug.log"), false));

        // But important.log is not ignored (negation)
        assert!(!ignore.is_ignored(Path::new("important.log"), false));
    }
}

