#!/bin/bash
# Auto-format code after Edit/Write operations
# Runs appropriate formatter based on file extension

set -e

# Read the tool call info from stdin
TOOL_INFO=$(cat)

# Extract file path from the tool call
FILE_PATH=$(echo "$TOOL_INFO" | jq -r '.tool_input.file_path // .tool_input.path // ""')

if [ -z "$FILE_PATH" ] || [ ! -f "$FILE_PATH" ]; then
    exit 0
fi

# Get file extension
EXT="${FILE_PATH##*.}"

# Format based on extension
case "$EXT" in
    rs)
        if command -v rustfmt &> /dev/null; then
            rustfmt --edition 2021 "$FILE_PATH" 2>/dev/null || true
        fi
        ;;
    go)
        if command -v gofmt &> /dev/null; then
            gofmt -w "$FILE_PATH" 2>/dev/null || true
        fi
        ;;
    ts|tsx|js|jsx|json)
        if command -v prettier &> /dev/null; then
            prettier --write "$FILE_PATH" 2>/dev/null || true
        fi
        ;;
    py)
        if command -v ruff &> /dev/null; then
            ruff format "$FILE_PATH" 2>/dev/null || true
        elif command -v black &> /dev/null; then
            black -q "$FILE_PATH" 2>/dev/null || true
        fi
        ;;
    yaml|yml)
        if command -v prettier &> /dev/null; then
            prettier --write "$FILE_PATH" 2>/dev/null || true
        fi
        ;;
    md)
        if command -v prettier &> /dev/null; then
            prettier --write --prose-wrap preserve "$FILE_PATH" 2>/dev/null || true
        fi
        ;;
esac

exit 0
