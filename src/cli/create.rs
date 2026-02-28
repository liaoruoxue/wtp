//! Create workspace command

use clap::Args;
use colored::Colorize;

use crate::core::WorkspaceManager;

#[derive(Args, Debug)]
pub struct CreateArgs {
    /// Name of the workspace to create
    pub name: String,
}

pub async fn execute(args: CreateArgs, mut manager: WorkspaceManager) -> anyhow::Result<()> {
    let workspace_path = manager.create_workspace(&args.name)?;

    println!(
        "{} Created workspace '{}' at {}",
        "✓".green().bold(),
        args.name.cyan(),
        workspace_path.display().to_string().dimmed()
    );
    println!();
    println!("To use this workspace, run:");
    println!("  {}", format!("cd {}", workspace_path.display()).cyan());

    Ok(())
}
