//! Git command wrapper
//!
//! All git operations are performed through the git CLI to avoid
//! direct manipulation of .git internals.

use crate::core::error::{Result, WtpError};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Git client for executing git commands
#[derive(Debug, Clone)]
pub struct GitClient;

impl GitClient {
    pub fn new() -> Self {
        Self
    }

    /// Check if git is available
    pub fn check_git(&self) -> Result<()> {
        match Command::new("git").arg("--version").output() {
            Ok(output) if output.status.success() => Ok(()),
            _ => Err(WtpError::git("Git is not installed or not in PATH")),
        }
    }

    /// Get the root directory of a git repository
    pub fn get_repo_root(&self, cwd: Option<&Path>) -> Result<PathBuf> {
        let mut cmd = Command::new("git");
        cmd.arg("rev-parse").arg("--show-toplevel");
        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }

        let output = cmd.output()?;
        if !output.status.success() {
            return Err(WtpError::NotInGitRepo);
        }

        let path = String::from_utf8_lossy(&output.stdout);
        Ok(PathBuf::from(path.trim()))
    }

    /// Check if a directory is inside a git repository
    pub fn is_in_git_repo(&self, path: &Path) -> bool {
        self.get_repo_root(Some(path)).is_ok()
    }

    /// Get the current branch name
    pub fn get_current_branch(&self, repo_path: &Path) -> Result<String> {
        let output = Command::new("git")
            .current_dir(repo_path)
            .arg("rev-parse")
            .arg("--abbrev-ref")
            .arg("HEAD")
            .output()?;

        if !output.status.success() {
            return Err(WtpError::git(format!(
                "Failed to get current branch: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Get the current HEAD commit hash (short)
    pub fn get_head_commit(&self, repo_path: &Path) -> Result<String> {
        let output = Command::new("git")
            .current_dir(repo_path)
            .arg("rev-parse")
            .arg("--short")
            .arg("HEAD")
            .output()?;

        if !output.status.success() {
            return Err(WtpError::git(format!(
                "Failed to get HEAD commit: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Get the full HEAD commit hash
    pub fn get_head_commit_full(&self, repo_path: &Path) -> Result<String> {
        let output = Command::new("git")
            .current_dir(repo_path)
            .arg("rev-parse")
            .arg("HEAD")
            .output()?;

        if !output.status.success() {
            return Err(WtpError::git(format!(
                "Failed to get HEAD commit: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Check if a branch exists in the repository
    pub fn branch_exists(&self, repo_path: &Path, branch: &str) -> Result<bool> {
        let output = Command::new("git")
            .current_dir(repo_path)
            .arg("show-ref")
            .arg("--verify")
            .arg(format!("refs/heads/{}", branch))
            .output()?;

        Ok(output.status.success())
    }

    /// Check if a branch is already checked out in a worktree
    /// Returns the worktree path if already checked out
    pub fn is_branch_checked_out(&self, repo_path: &Path, branch: &str) -> Result<Option<PathBuf>> {
        let worktrees = self.list_worktrees(repo_path)?;

        for worktree in worktrees {
            // Get the branch for this worktree
            let wt_branch = self.get_worktree_branch(&worktree)?;
            if wt_branch == branch {
                return Ok(Some(worktree));
            }
        }

        Ok(None)
    }

    /// Get the branch checked out in a worktree
    fn get_worktree_branch(&self, worktree_path: &Path) -> Result<String> {
        let output = Command::new("git")
            .current_dir(worktree_path)
            .arg("rev-parse")
            .arg("--abbrev-ref")
            .arg("HEAD")
            .output()?;

        if !output.status.success() {
            // Worktree might be in detached HEAD state
            return Ok("(detached)".to_string());
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// List all worktrees for a repository
    pub fn list_worktrees(&self, repo_path: &Path) -> Result<Vec<PathBuf>> {
        let output = Command::new("git")
            .current_dir(repo_path)
            .arg("worktree")
            .arg("list")
            .arg("--porcelain")
            .output()?;

        if !output.status.success() {
            return Err(WtpError::git(format!(
                "Failed to list worktrees: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let mut worktrees = Vec::new();
        let output_str = String::from_utf8_lossy(&output.stdout);

        for line in output_str.lines() {
            if line.starts_with("worktree ") {
                let path = line.strip_prefix("worktree ").unwrap();
                worktrees.push(PathBuf::from(path));
            }
        }

        Ok(worktrees)
    }

    /// Create a new worktree with a new branch
    pub fn create_worktree_with_branch(
        &self,
        repo_path: &Path,
        worktree_path: &Path,
        branch: &str,
        base: &str,
    ) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = worktree_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let output = Command::new("git")
            .current_dir(repo_path)
            .arg("worktree")
            .arg("add")
            .arg("-b")
            .arg(branch)
            .arg(worktree_path)
            .arg(base)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Check for common errors
            if stderr.contains("already checked out") {
                return Err(WtpError::BranchAlreadyCheckedOut {
                    branch: branch.to_string(),
                    worktree_path: worktree_path.to_path_buf(),
                });
            }
            if stderr.contains("already exists") {
                return Err(WtpError::WorktreeAlreadyExists {
                    path: worktree_path.to_path_buf(),
                });
            }
            return Err(WtpError::git(format!(
                "Failed to create worktree: {}",
                stderr
            )));
        }

        Ok(())
    }

    /// Add a worktree for an existing branch
    pub fn add_worktree_for_branch(
        &self,
        repo_path: &Path,
        worktree_path: &Path,
        branch: &str,
    ) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = worktree_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let output = Command::new("git")
            .current_dir(repo_path)
            .arg("worktree")
            .arg("add")
            .arg(worktree_path)
            .arg(branch)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("already checked out") {
                return Err(WtpError::BranchAlreadyCheckedOut {
                    branch: branch.to_string(),
                    worktree_path: worktree_path.to_path_buf(),
                });
            }
            if stderr.contains("already exists") {
                return Err(WtpError::WorktreeAlreadyExists {
                    path: worktree_path.to_path_buf(),
                });
            }
            return Err(WtpError::git(format!(
                "Failed to add worktree: {}",
                stderr
            )));
        }

        Ok(())
    }

    /// Remove a worktree
    pub fn remove_worktree(&self, repo_path: &Path, worktree_path: &Path, force: bool) -> Result<()> {
        let mut cmd = Command::new("git");
        cmd.current_dir(repo_path)
            .arg("worktree")
            .arg("remove");

        if force {
            cmd.arg("--force");
        }

        cmd.arg(worktree_path);

        let output = cmd.output()?;

        if !output.status.success() {
            return Err(WtpError::git(format!(
                "Failed to remove worktree: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    /// Check if working directory has uncommitted changes
    pub fn is_dirty(&self, repo_path: &Path) -> Result<bool> {
        let output = Command::new("git")
            .current_dir(repo_path)
            .arg("status")
            .arg("--porcelain")
            .output()?;

        if !output.status.success() {
            return Err(WtpError::git(format!(
                "Failed to check status: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(!output.stdout.is_empty())
    }

    /// Get detailed status of a repository
    pub fn get_status(&self, repo_path: &Path) -> Result<GitStatus> {
        let output = Command::new("git")
            .current_dir(repo_path)
            .arg("status")
            .arg("--porcelain")
            .arg("--branch")
            .output()?;

        if !output.status.success() {
            return Err(WtpError::git(format!(
                "Failed to get status: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut ahead = 0;
        let mut behind = 0;
        let mut dirty = false;

        for line in output_str.lines() {
            if line.starts_with("## ") {
                // Parse branch info
                if let Some(ab) = line.find("[ahead ") {
                    let start = ab + 7;
                    if let Some(end) = line[start..].find(']') {
                        let ahead_str = &line[start..start + end];
                        ahead = ahead_str
                            .split(',')
                            .next()
                            .and_then(|s| s.trim().parse().ok())
                            .unwrap_or(0);
                    }
                }
                if let Some(bb) = line.find("behind ") {
                    let start = bb + 7;
                    if let Some(end) = line[start..].find(']') {
                        behind = line[start..start + end].trim().parse().unwrap_or(0);
                    }
                }
            } else if !line.is_empty() {
                dirty = true;
            }
        }

        Ok(GitStatus {
            dirty,
            ahead,
            behind,
        })
    }

    /// Check if a worktree path already exists
    pub fn worktree_exists(&self, repo_path: &Path, worktree_path: &Path) -> Result<bool> {
        let worktrees = self.list_worktrees(repo_path)?;
        Ok(worktrees.iter().any(|p| p == worktree_path))
    }
}

impl Default for GitClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Git status information
#[derive(Debug, Clone, Default)]
pub struct GitStatus {
    /// Has uncommitted changes
    pub dirty: bool,
    /// Commits ahead of remote
    pub ahead: u32,
    /// Commits behind remote
    pub behind: u32,
}

impl GitStatus {
    /// Format status as a compact string
    pub fn format_compact(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        if self.dirty {
            parts.push("*".to_string());
        }
        if self.ahead > 0 {
            parts.push(format!("+{}", self.ahead));
        }
        if self.behind > 0 {
            parts.push(format!("-{}", self.behind));
        }
        if parts.is_empty() {
            "clean".to_string()
        } else {
            parts.join(" ")
        }
    }
}
