#!/bin/bash
# Safe removal script - always prompts before deleting
# Usage: ./safe-rm.sh <path1> [path2] ...

set -e

if [ $# -eq 0 ]; then
    echo "Usage: $0 <path1> [path2] ..."
    echo "ERROR: No paths specified" >&2
    exit 1
fi

echo "⚠️  DELETE OPERATION REQUESTED"
echo "=============================="
echo ""
echo "The following items will be REMOVED:"
echo ""

for path in "$@"; do
    if [ -e "$path" ]; then
        echo "  📁 $path"
        # Show size if it's a directory
        if [ -d "$path" ]; then
            size=$(du -sh "$path" 2>/dev/null | cut -f1)
            echo "     Size: $size"
        fi
    else
        echo "  ⚠️  $path (does not exist)"
    fi
done

echo ""
echo "=============================="
echo ""
echo "Are you sure you want to permanently delete these items?"
echo "This action CANNOT be undone!"
echo ""
echo "Type 'yes' to confirm, or anything else to cancel:"
read -r confirmation

if [ "$confirmation" = "yes" ]; then
    echo ""
    echo "Proceeding with deletion..."
    for path in "$@"; do
        if [ -e "$path" ]; then
            echo "  Deleting: $path"
            rm -rf "$path"
        fi
    done
    echo "Done."
else
    echo ""
    echo "❌ Deletion cancelled."
    exit 1
fi
