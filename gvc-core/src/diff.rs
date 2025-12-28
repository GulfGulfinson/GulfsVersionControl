use crate::{Blob, Error, Hash, Object, Result};
use std::collections::HashMap;

/// Represents a change in a file
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Change {
    /// Line was added
    Add(String),
    /// Line was deleted
    Delete(String),
    /// Line is unchanged (context)
    Context(String),
}

/// Represents a hunk of changes
#[derive(Debug, Clone)]
pub struct Hunk {
    pub old_start: usize,
    pub old_count: usize,
    pub new_start: usize,
    pub new_count: usize,
    pub changes: Vec<Change>,
}

/// Represents differences between two files
#[derive(Debug, Clone)]
pub struct FileDiff {
    pub path: String,
    pub old_hash: Option<Hash>,
    pub new_hash: Option<Hash>,
    pub hunks: Vec<Hunk>,
}

impl FileDiff {
    pub fn is_new_file(&self) -> bool {
        self.old_hash.is_none()
    }

    pub fn is_deleted_file(&self) -> bool {
        self.new_hash.is_none()
    }

    pub fn is_modified(&self) -> bool {
        self.old_hash.is_some() && self.new_hash.is_some()
    }
}

/// Diff engine using Myers algorithm (simplified)
pub struct DiffEngine;

impl DiffEngine {
    /// Compute diff between two blobs
    pub fn diff_blobs(old_blob: Option<&Blob>, new_blob: Option<&Blob>) -> Result<Vec<Hunk>> {
        match (old_blob, new_blob) {
            (None, None) => Ok(Vec::new()),
            (None, Some(new)) => {
                // New file - all lines added
                let lines = Self::blob_to_lines(new)?;
                Ok(vec![Self::create_add_hunk(lines)])
            }
            (Some(old), None) => {
                // Deleted file - all lines removed
                let lines = Self::blob_to_lines(old)?;
                Ok(vec![Self::create_delete_hunk(lines)])
            }
            (Some(old), Some(new)) => {
                // Modified file - compute diff
                let old_lines = Self::blob_to_lines(old)?;
                let new_lines = Self::blob_to_lines(new)?;
                Self::compute_diff(&old_lines, &new_lines)
            }
        }
    }

    /// Convert blob to lines
    fn blob_to_lines(blob: &Blob) -> Result<Vec<String>> {
        // Try UTF-8 decoding
        match String::from_utf8(blob.data.clone()) {
            Ok(text) => Ok(text.lines().map(String::from).collect()),
            Err(_) => Err(Error::InvalidObjectType(
                "Binary file (not diffable)".to_string(),
            )),
        }
    }

    /// Create hunk with only additions
    fn create_add_hunk(lines: Vec<String>) -> Hunk {
        let changes: Vec<Change> = lines.into_iter().map(Change::Add).collect();
        Hunk {
            old_start: 0,
            old_count: 0,
            new_start: 1,
            new_count: changes.len(),
            changes,
        }
    }

    /// Create hunk with only deletions
    fn create_delete_hunk(lines: Vec<String>) -> Hunk {
        let changes: Vec<Change> = lines.into_iter().map(Change::Delete).collect();
        Hunk {
            old_start: 1,
            old_count: changes.len(),
            new_start: 0,
            new_count: 0,
            changes,
        }
    }

    /// Compute diff using simplified Myers algorithm
    /// This is a basic implementation - production would use more efficient algorithm
    fn compute_diff(old_lines: &[String], new_lines: &[String]) -> Result<Vec<Hunk>> {
        let lcs = Self::longest_common_subsequence(old_lines, new_lines);
        let changes = Self::build_changes(old_lines, new_lines, &lcs);
        let hunks = Self::group_into_hunks(changes, old_lines.len(), new_lines.len());
        Ok(hunks)
    }

    /// Compute longest common subsequence (LCS)
    /// Classic dynamic programming algorithm
    fn longest_common_subsequence(a: &[String], b: &[String]) -> Vec<(usize, usize)> {
        let m = a.len();
        let n = b.len();

        // DP table: lcs[i][j] = length of LCS of a[0..i] and b[0..j]
        let mut lcs = vec![vec![0; n + 1]; m + 1];

        for i in 1..=m {
            for j in 1..=n {
                if a[i - 1] == b[j - 1] {
                    lcs[i][j] = lcs[i - 1][j - 1] + 1;
                } else {
                    lcs[i][j] = lcs[i - 1][j].max(lcs[i][j - 1]);
                }
            }
        }

        // Backtrack to find actual LCS
        let mut result = Vec::new();
        let mut i = m;
        let mut j = n;

        while i > 0 && j > 0 {
            if a[i - 1] == b[j - 1] {
                result.push((i - 1, j - 1));
                i -= 1;
                j -= 1;
            } else if lcs[i - 1][j] > lcs[i][j - 1] {
                i -= 1;
            } else {
                j -= 1;
            }
        }

        result.reverse();
        result
    }

    /// Build changes from LCS
    fn build_changes(
        old_lines: &[String],
        new_lines: &[String],
        lcs: &[(usize, usize)],
    ) -> Vec<Change> {
        let mut changes = Vec::new();
        let mut old_idx = 0;
        let mut new_idx = 0;

        for &(lcs_old, lcs_new) in lcs {
            // Deletions
            while old_idx < lcs_old {
                changes.push(Change::Delete(old_lines[old_idx].clone()));
                old_idx += 1;
            }

            // Additions
            while new_idx < lcs_new {
                changes.push(Change::Add(new_lines[new_idx].clone()));
                new_idx += 1;
            }

            // Context (unchanged line)
            changes.push(Change::Context(old_lines[old_idx].clone()));
            old_idx += 1;
            new_idx += 1;
        }

        // Remaining deletions
        while old_idx < old_lines.len() {
            changes.push(Change::Delete(old_lines[old_idx].clone()));
            old_idx += 1;
        }

        // Remaining additions
        while new_idx < new_lines.len() {
            changes.push(Change::Add(new_lines[new_idx].clone()));
            new_idx += 1;
        }

        changes
    }

    /// Group changes into hunks with context
    fn group_into_hunks(
        changes: Vec<Change>,
        old_total: usize,
        new_total: usize,
    ) -> Vec<Hunk> {
        if changes.is_empty() {
            return Vec::new();
        }

        const CONTEXT_LINES: usize = 3;
        let mut hunks = Vec::new();
        let mut current_hunk: Option<Hunk> = None;
        let mut old_line: usize = 1;
        let mut new_line: usize = 1;
        let mut context_buffer = Vec::new();

        for change in changes {
            match change {
                Change::Context(line) => {
                    if let Some(ref mut hunk) = current_hunk {
                        // Add context to current hunk
                        if context_buffer.len() < CONTEXT_LINES {
                            context_buffer.push(Change::Context(line.clone()));
                        } else {
                            // Too much context - close hunk
                            hunk.changes.extend(context_buffer.drain(..CONTEXT_LINES));
                            hunks.push(current_hunk.take().unwrap());
                            context_buffer.clear();
                            context_buffer.push(Change::Context(line.clone()));
                        }
                    } else {
                        // No hunk yet - keep context for potential next hunk
                        context_buffer.push(Change::Context(line.clone()));
                        if context_buffer.len() > CONTEXT_LINES {
                            context_buffer.remove(0);
                        }
                    }
                    old_line += 1;
                    new_line += 1;
                }
                Change::Add(line) => {
                    if current_hunk.is_none() {
                        // Start new hunk
                        current_hunk = Some(Hunk {
                            old_start: old_line.saturating_sub(context_buffer.len()),
                            old_count: context_buffer.len(),
                            new_start: new_line.saturating_sub(context_buffer.len()),
                            new_count: context_buffer.len(),
                            changes: context_buffer.clone(),
                        });
                        context_buffer.clear();
                    }

                    if let Some(ref mut hunk) = current_hunk {
                        hunk.changes.push(Change::Add(line));
                        hunk.new_count += 1;
                    }
                    new_line += 1;
                }
                Change::Delete(line) => {
                    if current_hunk.is_none() {
                        // Start new hunk
                        current_hunk = Some(Hunk {
                            old_start: old_line.saturating_sub(context_buffer.len()),
                            old_count: context_buffer.len(),
                            new_start: new_line.saturating_sub(context_buffer.len()),
                            new_count: context_buffer.len(),
                            changes: context_buffer.clone(),
                        });
                        context_buffer.clear();
                    }

                    if let Some(ref mut hunk) = current_hunk {
                        hunk.changes.push(Change::Delete(line));
                        hunk.old_count += 1;
                    }
                    old_line += 1;
                }
            }
        }

        // Close final hunk
        if let Some(mut hunk) = current_hunk {
            hunk.changes.extend(context_buffer);
            hunks.push(hunk);
        }

        hunks
    }
}

/// Format hunk as unified diff
pub fn format_hunk(hunk: &Hunk) -> String {
    let mut output = String::new();

    // Hunk header
    output.push_str(&format!(
        "@@ -{},{} +{},{} @@\n",
        hunk.old_start, hunk.old_count, hunk.new_start, hunk.new_count
    ));

    // Changes
    for change in &hunk.changes {
        match change {
            Change::Add(line) => output.push_str(&format!("+{}\n", line)),
            Change::Delete(line) => output.push_str(&format!("-{}\n", line)),
            Change::Context(line) => output.push_str(&format!(" {}\n", line)),
        }
    }

    output
}

/// Format file diff as unified diff
pub fn format_file_diff(diff: &FileDiff) -> String {
    let mut output = String::new();

    // File header
    if diff.is_new_file() {
        output.push_str(&format!("new file: {}\n", diff.path));
    } else if diff.is_deleted_file() {
        output.push_str(&format!("deleted file: {}\n", diff.path));
    } else {
        output.push_str(&format!("diff --gvc a/{} b/{}\n", diff.path, diff.path));
    }

    // Hunks
    for hunk in &diff.hunks {
        output.push_str(&format_hunk(hunk));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_new_file() {
        let new_blob = Blob::new(b"line1\nline2\n".to_vec());
        let hunks = DiffEngine::diff_blobs(None, Some(&new_blob)).unwrap();

        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].changes.len(), 2);
    }

    #[test]
    fn test_diff_deleted_file() {
        let old_blob = Blob::new(b"line1\nline2\n".to_vec());
        let hunks = DiffEngine::diff_blobs(Some(&old_blob), None).unwrap();

        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].changes.len(), 2);
    }

    #[test]
    fn test_diff_modified_file() {
        let old_blob = Blob::new(b"line1\nline2\nline3\n".to_vec());
        let new_blob = Blob::new(b"line1\nmodified\nline3\n".to_vec());

        let hunks = DiffEngine::diff_blobs(Some(&old_blob), Some(&new_blob)).unwrap();
        assert!(!hunks.is_empty());
    }

    #[test]
    fn test_lcs() {
        let a = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let b = vec!["a".to_string(), "x".to_string(), "c".to_string()];

        let lcs = DiffEngine::longest_common_subsequence(&a, &b);
        assert_eq!(lcs, vec![(0, 0), (2, 2)]);
    }
}

