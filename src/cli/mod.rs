//! CLI module containing all subcommand implementations

pub mod cd;
pub mod create;
pub mod host;
pub mod import;
pub mod ls;
pub mod remove;
pub mod shell_init;
pub mod status;
pub mod switch;

use clap::builder::{Styles, styling::AnsiColor};
use clap::{Parser, Subcommand};

/// Custom styles for help text
fn help_styles() -> Styles {
    Styles::styled()
        // Headers (Commands, Options, etc.) - yellow
        .header(AnsiColor::Yellow.on_default().bold())
        // Literal command-line syntax (commands, flags like --help) - yellow
        .literal(AnsiColor::Yellow.on_default())
        // Placeholders (value names like <NAME>) - purple/magenta
        .placeholder(AnsiColor::Magenta.on_default())
        // Usage heading - green
        .usage(AnsiColor::Green.on_default().bold())
        // Valid/suggested values - blue
        .valid(AnsiColor::Blue.on_default())
        // Error messages - red
        .error(AnsiColor::Red.on_default().bold())
}

/// WorkTree for Polyrepo - Manage multiple git worktrees across repositories
#[derive(Parser, Debug)]
#[command(name = "wtp")]
#[command(about = "WorkTree for Polyrepo - Manage git worktrees across multiple repositories")]
#[command(version)]
#[command(styles = help_styles())]
#[command(color = clap::ColorChoice::Always)]
#[command(
    help_template = concat!(
        "{about}\n\n",
        "{usage-heading} {usage}\n\n",
        "Options:\n",
        "{options}\n\n",
        "Workspace Management:\n",
        "  \x1b[32mcd\x1b[0m          Change to a workspace directory (requires shell integration)\n",
        "  \x1b[32mcreate\x1b[0m      Create a new workspace\n",
        "  \x1b[32mls\x1b[0m          List all workspaces\n",
        "  \x1b[32mrm\x1b[0m          Remove a workspace\n",
        "  \x1b[32mstatus\x1b[0m      Show status of a workspace\n\n",
        "Repository Operations:\n",
        "  \x1b[32mimport\x1b[0m      Import a repository's worktree into a workspace\n",
        "  \x1b[32mswitch\x1b[0m      Switch current repo to a workspace\n\n",
        "Utilities:\n",
        "  \x1b[32mhost\x1b[0m        Manage host aliases\n",
        "  \x1b[32mshell-init\x1b[0m  Generate shell integration script\n\n",
        "Use `{name} help <command>` for more information on a specific command.\n"
    )
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand, Debug)]
#[command(hide = true)]
pub enum Commands {
    Cd(cd::CdArgs),
    Create(create::CreateArgs),
    Ls(ls::LsArgs),
    Remove(remove::RemoveArgs),
    Status(status::StatusArgs),
    Import(import::ImportArgs),
    Switch(switch::SwitchArgs),
    Host(host::HostArgs),
    ShellInit(shell_init::ShellInitArgs),
}

/// Run the CLI
pub async fn run() -> anyhow::Result<()> {
    use colored::Colorize;

    let cli = Cli::parse();

    // Initialize tracing for verbose mode
    if cli.verbose {
        tracing_subscriber::fmt()
            .with_env_filter("debug")
            .init();
    }

    // Load global config
    let (global_config, warning) = crate::core::GlobalConfig::load()?;

    // Print warning if multiple config files exist
    if let Some(warn) = warning {
        eprintln!("{}", warn.yellow());
    }

    // Initialize security fence
    crate::core::fence::init_global_fence(global_config.workspace_root.clone());

    // Execute command
    match cli.command {
        Commands::Cd(args) => cd::execute(args, crate::core::WorkspaceManager::new(global_config)).await?,
        Commands::Create(args) => create::execute(args, crate::core::WorkspaceManager::new(global_config)).await?,
        Commands::Ls(args) => ls::execute(args, crate::core::WorkspaceManager::new(global_config)).await?,
        Commands::Remove(args) => remove::execute(args, crate::core::WorkspaceManager::new(global_config)).await?,
        Commands::Status(args) => status::execute(args, crate::core::WorkspaceManager::new(global_config)).await?,
        Commands::Import(args) => import::execute(args, crate::core::WorkspaceManager::new(global_config)).await?,
        Commands::Switch(args) => {
            let mut manager = crate::core::WorkspaceManager::new(global_config);
            switch::execute(args, manager).await?
        }
        Commands::Host(args) => host::execute(args, crate::core::WorkspaceManager::new(global_config)).await?,
        Commands::ShellInit(args) => shell_init::execute(args).await?,
    }

    Ok(())
}
