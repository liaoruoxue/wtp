# Hooks - Safety Mechanisms

This document describes safety hooks implemented in the wtp project.

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
