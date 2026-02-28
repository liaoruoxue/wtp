# wtp Architecture

This document describes the internal architecture of the wtp project.

## Overview

wtp is structured as a layered application:

```
┌─────────────────────────────────────────┐
│  CLI / TUI Layer (src/cli, src/tui)    │
│  - Argument parsing, user interaction  │
├─────────────────────────────────────────┤
│  Core Layer (src/core)                 │
│  - Business logic, git operations      │
│  - Configuration management            │
├─────────────────────────────────────────┤
│  External Dependencies                 │
│  - git CLI, filesystem                 │
└─────────────────────────────────────────┘
```

## Module Structure

### `src/core/`

The core module contains all business logic and is independent of the UI layer.

#### `config.rs`
- **GlobalConfig**: Manages global configuration loaded from `~/.wtp.toml` or `~/.wtp/config.toml`
- **WorkspaceConfig**: Per-workspace configuration stored in `.wtp/config.toml`
- **HostConfig**: Host alias mappings (e.g., `gh -> ~/codes/github.com`)

Key features:
- Configuration file priority: `~/.wtp.toml` > `~/.wtp/config.toml` > `~/.config/wtp/config.toml`
- Path expansion using `shellexpand` for `~` and environment variables

#### `error.rs`
- **WtpError**: Enum of all possible errors
- **Result**: Type alias for `std::result::Result<T, WtpError>`

Error categories:
- Configuration errors
- Git operation errors
- Workspace/worktree management errors
- IO errors

#### `git.rs`
- **GitClient**: Wrapper around git CLI commands

Key operations:
- `get_repo_root()`: Find git repository root
- `get_current_branch()`: Get current branch name
- `create_worktree_with_branch()`: Create new worktree with new branch
- `add_worktree_for_branch()`: Add worktree for existing branch
- `list_worktrees()`: List all worktrees for a repo
- `is_branch_checked_out()`: Check if branch is in use
- `get_status()`: Get repository status (dirty, ahead/behind)

Design decision: All git operations go through the CLI rather than direct `.git` manipulation for safety and compatibility.

#### `workspace.rs`
- **WorkspaceManager**: Manages workspace lifecycle
- **WorkspaceInfo**: Information about a workspace for display

Key operations:
- `create_workspace()`: Create new workspace directory under `workspace_root`
- `match_host_alias()`: Match absolute path to host alias

Note: There is no "current workspace" concept. All commands that operate on a workspace require explicit `--workspace` argument.

#### `worktree.rs`
- **WorktreeEntry**: Single worktree record
- **WorktreeToml**: The `.wtp/worktree.toml` file structure
- **WorktreeManager**: Manager for worktree operations
- **RepoRef**: Enum for repository references (hosted vs absolute)
- **WorktreeId**: Unique identifier for worktree entries

Worktree directory layout:
```
<workspace>/<repo_slug>/<branch_name>/
```

### `src/cli/`

Each subcommand is implemented in its own module.

#### `mod.rs`
- CLI argument definitions using `clap`
- Command routing and setup

#### Subcommand modules
- `ls.rs`: List workspaces
- `create.rs`: Create new workspace
- `remove.rs`: Remove workspace
- `import.rs`: Import a worktree into a workspace (requires `--workspace`)
- `switch.rs`: Switch current repo to workspace
- `status.rs`: Show workspace status (requires `--workspace`)
- `cd.rs`: Change to workspace directory (requires shell integration)
- `shell_init.rs`: Generate shell integration script

### `src/tui/`

Placeholder for future TUI implementation using `ratatui`.

## Data Flow

### Creating a Workspace

```
User: wtp create my-feature
  │
  ▼
CLI: Parse arguments
  │
  ▼
WorkspaceManager::create_workspace()
  ├─ Check if workspace exists in config
  ├─ Create directory: <workspace_root>/my-feature
  ├─ Create .wtp/worktree.toml
  └─ Register in GlobalConfig
  │
  ▼
Output: Success message with path
```

### Adding a Worktree

```
User: wtp add company/project --host gh
  │
  ▼
CLI: Parse arguments
  │
  ▼
WorkspaceManager::require_current_workspace()
  └─ Find workspace containing current directory
  │
  ▼
WorkspaceManager::resolve_repo_path()
  └─ Resolve "company/project" + host "gh" → absolute path
  │
  ▼
GitClient operations:
  ├─ Verify it's a git repo
  ├─ Check if branch exists
  ├─ Check if branch is already checked out
  ├─ Create worktree with git worktree add
  └─ Get HEAD commit
  │
  ▼
WorktreeManager::add_worktree()
  └─ Record in .wtp/worktree.toml
  │
  ▼
Output: Success message
```

### Switching to a Workspace

```
User: wtp switch my-feature
  │
  ▼
CLI: Parse arguments
  │
  ▼
GitClient::get_repo_root()
  └─ Verify we're in a git repo
  │
  ▼
WorkspaceManager::match_host_alias()
  └─ Try to represent repo path relative to host
  │
  ▼
GitClient operations (same as add):
  └─ Create worktree in target workspace
  │
  ▼
WorktreeManager::add_worktree() (in target workspace)
  └─ Record in target's .wtp/worktree.toml
  │
  ▼
Output: Success message with cd command
```

## Configuration Schema

### Global Config (~/.wtp.toml or ~/.wtp/config.toml)

```toml
workspace_root = "~/.wtp/workspaces"
default_workspace = "main"
default_host = "gh"

[hosts.gh]
root = "~/codes/github.com"

[hosts.gl]
root = "~/codes/gitlab.company.internal"

[workspaces]
main = "~/.wtp/workspaces/main"
feature-x = "~/.wtp/workspaces/feature-x"
```

### Workspace Config (.wtp/config.toml)

```toml
default_host = "gh"

[hosts.gh]
root = "~/codes/github.com"
```

### Worktree Config (.wtp/worktree.toml)

```toml
version = "1"

[[worktrees]]
id = "uuid-here"
branch = "feature-x"
worktree_path = "my-project/feature-x"
created_at = "2024-01-15T10:30:00+00:00"

[worktrees.repo]
hosted = { host = "gh", path = "company/my-project" }

[worktrees.base]
base = "main"
head_commit = "abc123..."
```

## Shell Integration

The `wtp cd` command requires shell integration because a child process cannot change the parent shell's working directory.

### How it Works

1. **Shell Wrapper** (`wtp shell-init`): Generates a shell function that wraps the wtp binary
2. **Directive File**: When `wtp cd` is called, the wrapper creates a temp file and sets `WTP_DIRECTIVE_FILE`
3. **Write Command**: `wtp cd` writes `cd '/path/to/workspace'` to this file
4. **Source**: After wtp exits, the wrapper sources the file, executing the cd in the parent shell

### Flow

```
User runs: wtp cd my-feature

1. Shell wrapper intercepts call
2. Creates temp file: /tmp/wtp.XXXXXX
3. Sets WTP_DIRECTIVE_FILE=/tmp/wtp.XXXXXX
4. Runs: wtp cd my-feature
   └─ wtp writes "cd '/home/user/.wtp/workspaces/my-feature'" to file
5. Wrapper sources the file (executes cd in parent shell)
6. User is now in the workspace directory
```

### Why Not Direct chdir?

```rust
// This only changes wtp's own directory, not the shell's
std::env::set_current_dir("/some/path");
```

Unix process model prevents child processes from modifying parent state. The directive file pattern is a common workaround used by tools like `j` (autojump), `z` (zoxide), and `direnv`.

## Security Fence

wtp implements a **fence** mechanism to prevent accidental file operations outside the `workspace_root`.

### Purpose

- **Protect system files**: Prevent accidental modification of files outside the workspace directory
- **Containment**: Ensure all workspace data stays within the configured `workspace_root`
- **Safety net**: Require explicit user confirmation for out-of-boundary operations

### Implementation

The fence is implemented in `src/core/fence.rs`:

```rust
pub struct Fence {
    boundary: PathBuf,  // The workspace_root
    interactive: bool,  // Whether to prompt for confirmation
}
```

Key methods:
- `is_within_boundary(path)`: Check if a path is within the boundary
- `create_dir_all(path)`: Create directory with fence check
- `write(path, content)`: Write file with fence check
- `remove_dir_all(path)`: Remove directory with fence check

### User Experience

When an operation targets a path outside `workspace_root`:

```
⚠️  Warning: Workspace 'xxx' is outside workspace_root: /Users/you/.wtp/workspaces
Target path: /some/outside/path
Are you sure you want to proceed? [y/N] 
```

The user must explicitly confirm with 'y' to proceed.

### Global Instance

A global fence is initialized at startup with the configured `workspace_root`:

```rust
// In cli/mod.rs
crate::core::fence::init_global_fence(global_config.workspace_root.clone());
```

## Git Worktree Integration

wtp builds on top of git worktree with these design principles:

1. **No direct .git manipulation**: All operations use `git worktree` CLI
2. **Branch constraint awareness**: Enforces git's rule that a branch can only be checked out in one worktree
3. **Original repo preservation**: Worktrees are always created alongside the original repo, never replacing it
4. **Metadata tracking**: wtp maintains its own metadata about worktrees separate from git's internal tracking

## Error Handling Strategy

- All core functions return `Result<T>` (using `thiserror`)
- CLI layer converts errors to user-friendly messages
- Git errors include stderr output for debugging
- IO errors are propagated with context

## Future Extensions

### TUI (src/tui/)

Planned features:
- Interactive workspace browser using `ratatui`
- Fuzzy finder for repositories using `skim`
- Real-time status updates

### Additional Commands

Potential additions:
- `wtp sync`: Synchronize all worktrees in a workspace
- `wtp clean`: Remove stale worktrees
- `wtp prune`: Remove worktrees with merged branches
- `wtp mv`: Move worktree between workspaces

## Testing Strategy

- **Unit tests**: Core logic functions
- **Integration tests**: CLI commands with temporary directories
- **Mock git**: Test without real git repos (future)
