//! List workspaces command

use clap::Args;
use colored::Colorize;

use crate::core::WorkspaceManager;

#[derive(Args, Debug)]
pub struct LsArgs {
    /// Show detailed information
    #[arg(short, long)]
    long: bool,

    /// Output only workspace names (for shell completion)
    #[arg(short, long)]
    short: bool,
}

pub async fn execute(args: LsArgs, manager: WorkspaceManager) -> anyhow::Result<()> {
    let workspaces = manager.list_workspaces();

    if workspaces.is_empty() {
        if !args.short {
            println!("{}", "No workspaces found.".dimmed());
            println!();
            println!("Create a workspace with:");
            println!("  {}", "wtp create <workspace_name>".cyan());
            println!();
            println!("All workspaces are stored under workspace_root (default: ~/.wtp/workspaces)");
        }
        return Ok(());
    }

    if args.short {
        // Short format - just names, one per line (for shell completion)
        for ws in workspaces {
            println!("{}", ws.name);
        }
    } else if args.long {
        // Detailed listing
        println!("{:<20} {:<40} {}", "NAME", "PATH", "STATUS");
        for ws in workspaces {
            let status = if ws.exists {
                "ok".dimmed().to_string()
            } else {
                "missing".red().to_string()
            };
            println!(
                "{:<20} {:<40} {}",
                ws.name.cyan(),
                ws.path.display().to_string().dimmed(),
                status
            );
        }
    } else {
        // Simple listing
        for ws in workspaces {
            let name = ws.name.cyan().to_string();

            let status = if !ws.exists {
                " [missing]".red().to_string()
            } else {
                String::new()
            };

            println!("{}{}", name, status);
        }
    }

    Ok(())
}
