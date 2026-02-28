//! Cd command - change to workspace directory
//!
//! This command writes a cd directive to a file that the parent shell
//! will source after wtp exits. Requires shell wrapper to be configured.

use clap::Args;
use colored::Colorize;

use crate::core::WorkspaceManager;

#[derive(Args, Debug)]
pub struct CdArgs {
    /// Name of the workspace to cd into
    pub workspace: String,
}

pub async fn execute(args: CdArgs, manager: WorkspaceManager) -> anyhow::Result<()> {
    // Get workspace path
    let workspace_path = manager
        .global_config()
        .get_workspace_path(&args.workspace)
        .cloned()
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Workspace '{}' not found. Create it with: wtp create {}",
                args.workspace,
                args.workspace
            )
        })?;

    if !workspace_path.exists() {
        anyhow::bail!(
            "Workspace '{}' directory does not exist at {}",
            args.workspace,
            workspace_path.display()
        );
    }

    // Check if we're running inside the shell wrapper
    let directive_file = std::env::var("WTP_DIRECTIVE_FILE").ok();

    match directive_file {
        Some(file_path) => {
            // Write cd command to the directive file
            let cd_command = format!("cd '{}'", workspace_path.display());
            std::fs::write(&file_path, cd_command)?;
            
            // Success message will be shown after cd completes
            eprintln!(
                "{} Changed to workspace '{}' at {}",
                "✓".green().bold(),
                args.workspace.cyan(),
                workspace_path.display().to_string().dimmed()
            );
        }
        None => {
            // Not running in wrapper mode - print error with instructions
            eprintln!("{}", "Error: wtp cd requires shell integration".red().bold());
            eprintln!();
            eprintln!("To enable 'wtp cd', add the following to your shell config:");
            eprintln!();
            eprintln!("  {}", "eval \"$(wtp shell-init)\"".cyan());
            eprintln!();
            eprintln!("Or manually change to the workspace:");
            eprintln!("  {}", format!("cd {}", workspace_path.display()).cyan());
            
            // Still exit with error since we couldn't actually cd
            anyhow::bail!("Shell integration not configured");
        }
    }

    Ok(())
}
