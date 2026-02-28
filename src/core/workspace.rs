//! Workspace management

use crate::core::config::{GlobalConfig, WTP_DIR};
use crate::core::error::{Result, WtpError};
use crate::core::fence::Fence;
use crate::core::worktree::{RepoRef, WorktreeManager};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Manages workspaces and their discovery
pub struct WorkspaceManager {
    global_config: GlobalConfig,
}

impl WorkspaceManager {
    pub fn new(global_config: GlobalConfig) -> Self {
        Self { global_config }
    }

    /// Get a reference to the global config
    pub fn global_config(&self) -> &GlobalConfig {
        &self.global_config
    }

    /// Get a mutable reference to the global config
    pub fn global_config_mut(&mut self) -> &mut GlobalConfig {
        &mut self.global_config
    }

    /// List all workspaces
    pub fn list_workspaces(&self) -> Vec<WorkspaceInfo> {
        self.global_config
            .workspaces
            .iter()
            .map(|(name, path)| {
                let exists = path.exists();
                WorkspaceInfo {
                    name: name.clone(),
                    path: path.clone(),
                    exists,
                }
            })
            .collect()
    }

    /// Create a new workspace
    pub fn create_workspace(&mut self, name: &str) -> Result<PathBuf> {
        // Check if workspace already exists in config
        if self.global_config.has_workspace(name) {
            return Err(WtpError::WorkspaceAlreadyExists {
                name: name.to_string(),
                path: self.global_config.get_workspace_path(name).unwrap().clone(),
            });
        }

        let workspace_path = self.global_config.resolve_workspace_path(name);

        // Check if directory already exists
        if workspace_path.exists() {
            return Err(WtpError::WorkspaceAlreadyExists {
                name: name.to_string(),
                path: workspace_path.clone(),
            });
        }

        // Create workspace directory structure with fence protection
        let fence = Fence::from_config(&self.global_config);
        self.initialize_workspace_dir(&workspace_path, &fence)?;

        // Register in global config
        self.global_config.add_workspace(name.to_string(), workspace_path.clone())?;

        Ok(workspace_path)
    }

    /// Initialize workspace directory structure
    fn initialize_workspace_dir(&self, path: &Path, fence: &Fence) -> Result<()> {
        // Create main directory
        fence.create_dir_all(path)?;

        // Create .wtp directory
        let wtp_dir = path.join(WTP_DIR);
        fence.create_dir_all(&wtp_dir)?;

        // Create empty worktree.toml
        let worktree_toml = crate::core::worktree::WorktreeToml::new();
        worktree_toml.save_with_fence(&wtp_dir.join("worktree.toml"), fence)?;

        Ok(())
    }

    /// Remove a workspace from config
    pub fn remove_workspace(&mut self, name: &str, delete_dir: bool) -> Result<Option<PathBuf>> {
        let path = self.global_config.remove_workspace(name)?;

        if let Some(ref p) = path {
            if delete_dir && p.exists() {
                // Check for existing worktrees
                let worktree_toml_path = p.join(WTP_DIR).join("worktree.toml");
                if worktree_toml_path.exists() {
                    let worktrees = WorktreeManager::load(p)?;
                    if !worktrees.list_worktrees().is_empty() {
                        return Err(WtpError::config(format!(
                            "Workspace '{}' still has {} worktrees. \
                            Remove them first with 'wtp rm --delete-dir', \
                            or manually clean up the worktrees.",
                            name,
                            worktrees.list_worktrees().len()
                        )));
                    }
                }

                crate::core::fence::ensure_fence(&self.global_config).remove_dir_all(p)?;
            }
        }

        Ok(path)
    }

    /// Try to match a repository path to a host alias
    pub fn match_host_alias(&self, repo_path: &Path) -> Option<(String, String)> {
        for (alias, host_config) in &self.global_config.hosts {
            if let Ok(rel) = repo_path.strip_prefix(&host_config.root) {
                return Some((alias.clone(), rel.to_string_lossy().to_string()));
            }
        }
        None
    }

    /// Get all host aliases
    pub fn get_hosts(&self) -> &HashMap<String, crate::core::config::HostConfig> {
        &self.global_config.hosts
    }
}

/// Information about a workspace
#[derive(Debug, Clone)]
pub struct WorkspaceInfo {
    pub name: String,
    pub path: PathBuf,
    pub exists: bool,
}
