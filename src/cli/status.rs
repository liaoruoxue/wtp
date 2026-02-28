//! Status command - Show worktree status
//!
//! This is a local command - it works on the current workspace if you're in one,
//! or requires --workspace to specify which workspace to show.

use clap::Args;
use colored::Colorize;
use std::env;
use std::path::PathBuf;

use crate::core::{GitClient, WorktreeManager, WorkspaceManager};

#[derive(Args, Debug)]
pub struct StatusArgs {
    /// Workspace to show status for (defaults to current workspace if in one)
    #[arg(short, long, value_name = "NAME")]
    workspace: Option<String>,

    /// Show detailed information
    #[arg(short, long)]
    long: bool,

    /// Show dirty status for each worktree (slower)
    #[arg(short, long)]
    dirty: bool,
}

pub async fn execute(args: StatusArgs, manager: WorkspaceManager) -> anyhow::Result<()> {
    let git = GitClient::new();

    // Determine target workspace
    let (workspace_name, workspace_path) = if let Some(name) = args.workspace {
        // Use explicitly specified workspace
        let path = manager
            .global_config()
            .get_workspace_path(&name)
            .cloned()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Workspace '{}' not found. Create it with: wtp create {}",
                    name, name
                )
            })?;
        (name, path)
    } else {
        // Try to detect current workspace from current directory
        detect_current_workspace(&manager)?
    };

    if !workspace_path.exists() {
        anyhow::bail!(
            "Workspace '{}' directory does not exist at {}",
            workspace_name,
            workspace_path.display()
        );
    }

    if !workspace_path.join(".wtp").exists() {
        anyhow::bail!(
            "Workspace '{}' exists in config but the directory is missing or corrupted.",
            workspace_name
        );
    }

    println!(
        "Workspace: {} at {}",
        workspace_name.cyan().bold(),
        workspace_path.display().to_string().dimmed()
    );
    println!();

    // Load worktrees
    let worktree_manager = WorktreeManager::load(&workspace_path)?;
    let worktrees = worktree_manager.list_worktrees();

    if worktrees.is_empty() {
        println!("{}", "No worktrees in this workspace.".dimmed());
        println!();
        println!("Import a worktree with:");
        println!(
            "  {}",
            format!("wtp import <repo_path>",).cyan()
        );
        println!();
        println!("Or switch the current repo to this workspace:");
        println!("  {}", format!("wtp switch {}", workspace_name).cyan());
        return Ok(());
    }

    if args.long {
        print_detailed_status(&git, worktrees, &workspace_path, args.dirty).await?;
    } else {
        print_compact_status(&git, worktrees, &workspace_path, args.dirty).await?;
    }

    Ok(())
}

/// Detect current workspace from current directory
/// Returns (workspace_name, workspace_path) if found
fn detect_current_workspace(
    manager: &WorkspaceManager,
) -> anyhow::Result<(String, PathBuf)> {
    let current_dir = env::current_dir()?;
    let mut check_dir = current_dir.as_path();

    loop {
        // Check if this directory has a .wtp subdirectory
        if check_dir.join(".wtp").is_dir() {
            // Find which workspace this is
            for (name, path) in manager.global_config().workspaces.iter() {
                if path == check_dir {
                    return Ok((name.clone(), path.clone()));
                }
            }
            // Directory has .wtp but not registered - might be an orphan
            // Return with the directory name as workspace name
            let name = check_dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("workspace")
                .to_string();
            return Ok((name, check_dir.to_path_buf()));
        }

        // Move up
        match check_dir.parent() {
            Some(parent) => check_dir = parent,
            None => break,
        }
    }

    // Not in any workspace
    anyhow::bail!(
        "Not in a workspace. Either:\n\
         1. Run this command from within a workspace directory, or\n\
         2. Use --workspace <NAME> to specify the workspace"
    )
}

async fn print_compact_status(
    git: &GitClient,
    worktrees: &[crate::core::WorktreeEntry],
    workspace_path: &std::path::Path,
    check_dirty: bool,
) -> anyhow::Result<()> {
    println!(
        "{:<30} {:<20} {}",
        "REPOSITORY".bold(),
        "BRANCH".bold(),
        "STATUS".bold()
    );

    for wt in worktrees {
        let wt_full_path = workspace_path.join(&wt.worktree_path);

        let status = if check_dirty && wt_full_path.exists() {
            match git.get_status(&wt_full_path) {
                Ok(s) => s.format_compact(),
                Err(_) => "?".to_string(),
            }
        } else if !wt_full_path.exists() {
            "missing".red().to_string()
        } else {
            "-".dimmed().to_string()
        };

        let _repo_name = wt.repo.slug();
        let repo_display = wt.repo.display();

        println!(
            "{:<30} {:<20} {}",
            if repo_display.len() > 30 {
                format!("{}...", &repo_display[..27]).dimmed()
            } else {
                repo_display.dimmed()
            },
            wt.branch.cyan(),
            status
        );
    }

    if !check_dirty {
        println!();
        println!(
            "{}",
            "Use --dirty to check working directory status (slower)".dimmed()
        );
    }

    Ok(())
}

async fn print_detailed_status(
    git: &GitClient,
    worktrees: &[crate::core::WorktreeEntry],
    workspace_path: &std::path::Path,
    check_dirty: bool,
) -> anyhow::Result<()> {
    for (i, wt) in worktrees.iter().enumerate() {
        if i > 0 {
            println!();
        }

        let wt_full_path = workspace_path.join(&wt.worktree_path);

        println!("{}", "─".repeat(60).dimmed());
        println!("{}: {}", "Repository".bold(), wt.repo.display().cyan());
        println!("{}: {}", "Branch".bold(), wt.branch.cyan());
        println!(
            "{}: {}",
            "Worktree".bold(),
            wt.worktree_path.display().to_string().dimmed()
        );

        if let Some(ref base) = wt.base {
            println!("{}: {}", "Base".bold(), base.dimmed());
        }

        if let Some(ref commit) = wt.head_commit {
            println!("{}: {}", "HEAD".bold(), &commit[..8].dimmed());
        }

        // Check if worktree exists
        if wt_full_path.exists() {
            if check_dirty {
                match git.get_status(&wt_full_path) {
                    Ok(status) => {
                        let status_str = if status.dirty {
                            "dirty".yellow().to_string()
                        } else {
                            "clean".green().to_string()
                        };
                        print!("{}: {}", "Status".bold(), status_str);

                        if status.ahead > 0 {
                            print!(" (ahead +{})", status.ahead);
                        }
                        if status.behind > 0 {
                            print!(" (behind -{})", status.behind);
                        }
                        println!();
                    }
                    Err(e) => {
                        println!(
                            "{}: {}",
                            "Status".bold(),
                            format!("error: {}", e).red()
                        );
                    }
                }
            }
        } else {
            println!("{}: {}", "Status".bold(), "MISSING".red().bold());
        }
    }
    println!("{}", "─".repeat(60).dimmed());

    Ok(())
}
