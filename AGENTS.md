# wtp (WorkTree for Polyrepo) - Agent Context

## Project Overview
A CLI tool for managing git worktrees across multiple repositories in a polyrepo workflow.

## Current Architecture

### Command Structure (Flat)
Commands are grouped visually in help output but kept flat (not nested):

**Workspace Management:**
- `wtp cd <workspace>` - Change to workspace dir (requires shell integration)
- `wtp create <name>` - Create new workspace
- `wtp ls` - List workspaces (supports `--short`, `--long`)
- `wtp rm <name>` - Remove workspace (supports `--delete-dir`, `--force`)
- `wtp status` - Show workspace status (supports `--long`, `--dirty`)

**Repository Operations:**
- `wtp import <path>` - Import repo worktree into current workspace
- `wtp switch <workspace>` - Switch current repo to workspace (supports `--create`, `--branch`, `--base`)

**Utilities:**
- `wtp host <subcommand>` - Manage host aliases (`add`, `ls`, `rm`, `set-default`)
- `wtp shell-init` - Generate shell integration script

### Worktree Layout
```
<workspace_root>/<workspace_name>/<repo_slug>/
```
- Flat structure (no branch subdirectories)
- One repo per workspace limit enforced

### Key Features
1. **Security Fence** - Prevents file operations outside `workspace_root`
2. **Host Aliases** - Map short names to code roots (e.g., `gh` → `~/codes/github.com`)
3. **Shell Integration** - `wtp cd` via `WTP_DIRECTIVE_FILE` mechanism
4. **Colored Output** - Green command names, yellow flags, red errors

### Config Locations (Priority Order)
1. `~/.wtp.toml`
2. `~/.wtp/config.toml`
3. `~/.config/wtp/config.toml`

### Config Format
```toml
workspace_root = "~/.wtp/workspaces"
default_workspace = "main"

[hosts.gh]
root = "~/codes/github.com"

[hosts.gl]
root = "~/codes/gitlab.company.internal"

default_host = "gh"
```

## Technical Stack
- Rust 2024 edition
- clap 4.5 with derive features
- colored, anstyle/anstream for colors
- shellexpand for path expansion

## Important Notes
- `add` command was renamed to `import`
- Nested subcommand structure was reverted to flat commands
- TUI mode was removed
- Error messages are displayed in red
- All commands use green color for command names in help output
