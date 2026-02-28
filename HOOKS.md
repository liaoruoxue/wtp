# Hooks - Safety Mechanisms & Lifecycle Events

This document describes hooks implemented in the wtp project.

## Pre-Delete Hook

### Purpose
Before executing any `rm -rf` or file deletion operations, Kimi (the AI assistant) **MUST** use the safe deletion script.

### Usage

**Instead of:**
```bash
rm -rf ~/.wtp/workspaces/my-ws
```

**Use:**
```bash
./scripts/safe-rm.sh ~/.wtp/workspaces/my-ws
```

Or manually confirm:
```bash
echo "Will delete: ~/.wtp/workspaces/my-ws"
ls -la ~/.wtp/workspaces/my-ws
read -p "Confirm? [yes/N] " confirm
[ "$confirm" = "yes" ] && rm -rf ~/.wtp/workspaces/my-ws
```

### Rules for Kimi

1. **Always list first**: Show what will be deleted
2. **Get explicit confirmation**: Wait for user to type "yes"
3. **Prefer safe-rm.sh**: Use the provided script when possible
4. **Never use `rm -rf` directly** without confirmation

### Example Workflow

```bash
# 1. Show what exists
echo "Current .wtp directory:"
ls -la ~/.wtp

# 2. Ask for confirmation
echo "Delete ~/.wtp? [yes/N]"

# 3. Only proceed if user confirms
```

## Workspace Lifecycle Hooks

### On-Create Hook

The `on_create` hook allows you to run a custom script every time a new workspace is created. This is useful for:

- Copying standard configuration files (e.g., spec coding standards)
- Installing workspace-specific tools or plugins
- Initializing project templates
- Setting up development environments
- Creating README or documentation files

### Configuration

Add the hook to your `~/.wtp.toml` or `~/.wtp/config.toml`:

```toml
[hooks]
on_create = "~/.wtp/hooks/on-create.sh"
```

### Environment Variables

The hook script receives the following environment variables:

| Variable | Description | Example |
|----------|-------------|---------|
| `WTP_WORKSPACE_NAME` | Name of the created workspace | `my-feature` |
| `WTP_WORKSPACE_PATH` | Full path to the workspace directory | `/home/user/.wtp/workspaces/my-feature` |

### Example Hook Script

```bash
#!/bin/bash
# ~/.wtp/hooks/on-create.sh

echo "Initializing workspace: $WTP_WORKSPACE_NAME"
cd "$WTP_WORKSPACE_PATH"

# Example: Create a README
cat > README.md << EOF
# $WTP_WORKSPACE_NAME

Created: $(date)
Workspace: $WTP_WORKSPACE_PATH
EOF

# Example: Copy spec coding config
# cp ~/.templates/spec-coding-config.toml "$WTP_WORKSPACE_PATH/.spec.toml"

# Example: Initialize direnv
# echo "layout python" > "$WTP_WORKSPACE_PATH/.envrc"

echo "✅ Workspace initialized!"
```

Make sure the script is executable:

```bash
chmod +x ~/.wtp/hooks/on-create.sh
```

### Skipping the Hook

To create a workspace without running the hook, use the `--no-hook` flag:

```bash
wtp create my-workspace --no-hook
```

### Hook Behavior

- **Hook failures don't block workspace creation**: If the hook script fails (non-zero exit code), a warning is displayed but the workspace is still created.
- **Output is displayed**: stdout from the hook is printed to the terminal.
- **Requires executable permissions**: On Unix systems, the hook script must have execute permissions.

## Code Changes

When modifying code that involves file deletion:

1. Check if the path is within `workspace_root`
2. Use the `Fence` mechanism for additional safety
3. Log all deletion operations
4. Prefer soft-delete (move to trash) over hard-delete when possible

## Recovery

If data is accidentally deleted:

1. Check macOS Trash: `~/.Trash/`
2. Check if Time Machine has backups
3. Contact system administrator

---

**Last Updated**: 2026-02-28  
**Enforced By**: AGENTS.md + HOOKS.md
