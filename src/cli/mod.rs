//! CLI module containing all subcommand implementations

pub mod cd;
pub mod config;
pub mod create;
pub mod eject;
pub mod fuzzy;
pub mod host;
pub mod import;
pub mod ls;
pub mod remove;
pub mod shell_init;
pub mod status;
pub mod switch;
pub mod theme;

use clap::{CommandFactory, Parser, Subcommand};
use colored::Colorize;
use wtp_derive::GroupedSubcommand;

/// Print custom help message with grouped subcommands
fn print_help() {
    let mut cmd = Cli::command().styles(theme::wtp_styles());
    cmd.build();
    Commands::print_help(&cmd);
}

/// Print styled help for any subcommand with consistent coloring
fn print_styled_help(cmd: &clap::Command) {
    use colored::Colorize;

    // About
    if let Some(about) = cmd.get_about() {
        println!("{}", about);
        println!();
    }

    // Usage line
    let name = cmd.get_name();
    let mut usage_parts = Vec::new();

    let has_options = cmd
        .get_arguments()
        .any(|a| a.get_short().is_some() || a.get_long().is_some());
    if has_options {
        usage_parts.push("[OPTIONS]".to_string());
    }

    for arg in cmd.get_arguments().filter(|a| a.is_positional()) {
        let display_name = arg
            .get_value_names()
            .and_then(|v| v.first())
            .map(|n| n.as_str().to_string())
            .unwrap_or_else(|| arg.get_id().as_str().to_uppercase());
        if arg.is_required_set() {
            usage_parts.push(format!("<{}>", display_name));
        } else {
            usage_parts.push(format!("[{}]", display_name));
        }
    }

    if cmd.has_subcommands() {
        usage_parts.push("<COMMAND>".to_string());
    }

    let usage_str = usage_parts.join(" ");
    println!(
        "{}: {} {}",
        "Usage".bold(),
        name.cyan().bold(),
        usage_str.magenta()
    );
    println!();

    // Arguments section (positionals)
    let positionals: Vec<_> = cmd.get_arguments().filter(|a| a.is_positional()).collect();
    if !positionals.is_empty() {
        let mut entries: Vec<(String, String)> = Vec::new();
        for arg in &positionals {
            let display_name = arg
                .get_value_names()
                .and_then(|v| v.first())
                .map(|n| n.as_str().to_string())
                .unwrap_or_else(|| arg.get_id().as_str().to_uppercase());
            let display = if arg.is_required_set() {
                format!("<{}>", display_name)
            } else {
                format!("[{}]", display_name)
            };
            let help = arg
                .get_help()
                .map(|s| s.to_string())
                .unwrap_or_default();
            entries.push((display, help));
        }
        let max_len = entries.iter().map(|(d, _)| d.len()).max().unwrap_or(0);
        println!("{}:", "Arguments".bold());
        for (display, help) in &entries {
            let padding = " ".repeat(max_len - display.len());
            println!("  {}{}  {}", display.magenta(), padding, help);
        }
        println!();
    }

    // Commands section (subcommands)
    if cmd.has_subcommands() {
        let subcmds: Vec<_> = cmd
            .get_subcommands()
            .filter(|s| s.get_name() != "help")
            .collect();
        if !subcmds.is_empty() {
            let max_len = subcmds
                .iter()
                .map(|s| s.get_name().len())
                .max()
                .unwrap_or(0);
            println!("{}:", "Commands".bold());
            for subcmd in &subcmds {
                let sname = subcmd.get_name();
                let about = subcmd
                    .get_about()
                    .map(|s| s.to_string())
                    .unwrap_or_default();
                let padding = " ".repeat(max_len - sname.len());
                println!("  {}{}  {}", sname.cyan().bold(), padding, about);
            }
            println!();
        }
    }

    // Options section
    // Store (flag_part, value_part, help) to color flag and value separately
    let mut options: Vec<(String, String, String)> = Vec::new();
    for arg in cmd.get_arguments() {
        let short = arg.get_short();
        let long = arg.get_long();
        if short.is_none() && long.is_none() {
            continue;
        }
        let flag_part = match (short, long) {
            (Some(s), Some(l)) => format!("-{}, --{}", s, l),
            (Some(s), None) => format!("-{}", s),
            (None, Some(l)) => format!("    --{}", l),
            _ => unreachable!(),
        };
        let value_part = match arg.get_action() {
            clap::ArgAction::Set | clap::ArgAction::Append => {
                let val = arg
                    .get_value_names()
                    .and_then(|v| v.first())
                    .map(|n| n.as_str().to_string())
                    .unwrap_or_else(|| arg.get_id().as_str().to_uppercase());
                format!(" <{}>", val)
            }
            _ => String::new(),
        };
        let help = arg
            .get_help()
            .map(|s| s.to_string())
            .unwrap_or_default();
        options.push((flag_part, value_part, help));
    }
    // Always add -h, --help since we handle it manually
    if !options.iter().any(|(f, _, _)| f.contains("--help")) {
        options.push(("-h, --help".to_string(), String::new(), "Print help".to_string()));
    }

    if !options.is_empty() {
        let max_len = options
            .iter()
            .map(|(f, v, _)| f.len() + v.len())
            .max()
            .unwrap_or(0);
        println!("{}:", "Options".bold());
        for (flag, value, help) in &options {
            let padding = " ".repeat(max_len - flag.len() - value.len());
            if value.is_empty() {
                println!("  {}{}  {}", flag.blue(), padding, help);
            } else {
                println!("  {}{}{}  {}", flag.blue(), value.magenta(), padding, help);
            }
        }
        println!();
    }
}

/// Walk the command tree following subcommand names in args.
/// Returns the deepest matched command and how many levels deep we went.
fn walk_command_tree<'a>(
    root: &'a clap::Command,
    args: &[String],
) -> (&'a clap::Command, usize) {
    let mut current = root;
    let mut depth = 0;
    for arg in args {
        if arg.starts_with('-') {
            continue;
        }
        match current.find_subcommand(arg) {
            Some(subcmd) => {
                current = subcmd;
                depth += 1;
            }
            None => break,
        }
    }
    (current, depth)
}

/// WorkTree for Polyrepo - Manage multiple git worktrees across repositories
#[derive(Parser, Debug)]
#[command(name = "wtp")]
#[command(disable_help_flag = true, disable_help_subcommand = true)]
#[command(version)]
#[command(color = clap::ColorChoice::Always)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(GroupedSubcommand, Subcommand, Debug)]
pub enum Commands {
    /// Change to a workspace directory (requires shell integration)
    #[cmd_group("Workspace Management")]
    Cd(cd::CdArgs),
    /// Create a new workspace
    #[cmd_group("Workspace Management")]
    Create(create::CreateArgs),
    /// List all workspaces
    #[cmd_group("Workspace Management")]
    Ls(ls::LsArgs),
    /// Remove a workspace
    #[cmd_group("Workspace Management")]
    Remove(remove::RemoveArgs),
    /// Show status of a workspace
    #[cmd_group("Workspace Management")]
    Status(status::StatusArgs),
    /// Show current configuration
    #[cmd_group("Utilities")]
    Config(config::ConfigArgs),
    /// Eject a repository's worktree from a workspace
    #[cmd_group("Repository Operations")]
    Eject(eject::EjectArgs),
    /// Import a repository's worktree into a workspace
    #[cmd_group("Repository Operations")]
    Import(import::ImportArgs),
    /// Switch current repo to a workspace
    #[cmd_group("Repository Operations")]
    Switch(switch::SwitchArgs),
    /// Manage host aliases
    #[cmd_group("Utilities")]
    Host(host::HostArgs),
    /// Generate shell integration script
    #[cmd_group("Utilities")]
    ShellInit(shell_init::ShellInitArgs),
}

/// Run the CLI
pub async fn run() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // Build command for help display
    let cmd = Cli::command().styles(theme::wtp_styles());

    // Handle no args or --help/-h as first arg -> main help
    if args.len() == 1
        || (args.len() >= 2 && (args[1] == "-h" || args[1] == "--help"))
    {
        print_help();
        return Ok(());
    }

    // Handle help subcommand: wtp help [cmd...]
    if args[1] == "help" {
        if args.len() >= 3 {
            let mut current_cmd: &clap::Command = &cmd;
            for arg in &args[2..] {
                if arg.starts_with('-') {
                    continue;
                }
                match current_cmd.find_subcommand(arg) {
                    Some(subcmd) => current_cmd = subcmd,
                    None => {
                        eprintln!(
                            "{}: Unknown command '{}'",
                            "Error".red().bold(),
                            arg
                        );
                        std::process::exit(1);
                    }
                }
            }
            print_styled_help(current_cmd);
        } else {
            print_help();
        }
        return Ok(());
    }

    // Handle --help/-h at any position (for subcommands at any depth)
    if let Some(help_pos) = args[1..].iter().position(|a| a == "--help" || a == "-h") {
        let help_pos = help_pos + 1; // adjust for [1..] offset
        let (target_cmd, depth) = walk_command_tree(&cmd, &args[1..help_pos]);
        if depth == 0 {
            print_help();
        } else {
            print_styled_help(target_cmd);
        }
        return Ok(());
    }

    let cli = Cli::parse();

    // Initialize tracing for verbose mode
    if cli.verbose {
        tracing_subscriber::fmt()
            .with_env_filter("debug")
            .init();
    }

    // Load global config
    let (loaded_config, warning) = crate::core::LoadedConfig::load()?;

    // Print warning if multiple config files exist
    if let Some(warn) = warning {
        eprintln!("{}", warn.yellow());
    }

    // Initialize security fence
    crate::core::fence::init_global_fence(loaded_config.config.workspace_root.clone());

    // Execute command
    match cli.command {
        Commands::Cd(args) => cd::execute(args, crate::core::WorkspaceManager::new(loaded_config)).await?,
        Commands::Create(args) => create::execute(args, crate::core::WorkspaceManager::new(loaded_config)).await?,
        Commands::Ls(args) => ls::execute(args, crate::core::WorkspaceManager::new(loaded_config)).await?,
        Commands::Remove(args) => remove::execute(args, crate::core::WorkspaceManager::new(loaded_config)).await?,
        Commands::Status(args) => status::execute(args, crate::core::WorkspaceManager::new(loaded_config)).await?,
        Commands::Eject(args) => eject::execute(args, crate::core::WorkspaceManager::new(loaded_config)).await?,
        Commands::Import(args) => import::execute(args, crate::core::WorkspaceManager::new(loaded_config)).await?,
        Commands::Switch(args) => {
            let manager = crate::core::WorkspaceManager::new(loaded_config);
            switch::execute(args, manager).await?
        }
        Commands::Host(args) => host::execute(args, crate::core::WorkspaceManager::new(loaded_config)).await?,
        Commands::Config(args) => config::execute(args, crate::core::WorkspaceManager::new(loaded_config)).await?,
        Commands::ShellInit(args) => shell_init::execute(args).await?,
    }

    Ok(())
}
