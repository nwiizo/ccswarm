#!/bin/bash
# Validate agent scope before Edit/Write operations
# This hook checks if the file being edited is within the appropriate agent's scope

set -e

# Read the tool call info from stdin
TOOL_INFO=$(cat)

# Extract file path from the tool call
FILE_PATH=$(echo "$TOOL_INFO" | jq -r '.tool_input.file_path // .tool_input.path // ""')

if [ -z "$FILE_PATH" ]; then
    exit 0  # No file path, allow the operation
fi

# Define agent scope patterns
FRONTEND_PATTERNS=(
    "*.tsx" "*.jsx" "*.css" "*.scss" "*.html"
    "*/components/*" "*/pages/*" "*/styles/*"
)

BACKEND_PATTERNS=(
    "*.rs" "*/api/*" "*/server/*" "*/db/*"
    "*/handlers/*" "*/services/*"
)

DEVOPS_PATTERNS=(
    "Dockerfile*" "docker-compose*" "*.yml" "*.yaml"
    ".github/*" "*/ci/*" "*/deploy/*"
)

QA_PATTERNS=(
    "*_test.rs" "*_test.go" "*.test.ts" "*.spec.ts"
    "*/tests/*" "*/test/*"
)

# Check agent context (if set via environment)
AGENT_ROLE="${CCSWARM_AGENT_ROLE:-}"

if [ -z "$AGENT_ROLE" ]; then
    exit 0  # No agent role set, allow all operations
fi

# Validate scope based on agent role
validate_scope() {
    local file="$1"
    local patterns=("${@:2}")

    for pattern in "${patterns[@]}"; do
        if [[ "$file" == $pattern ]]; then
            return 0
        fi
    done
    return 1
}

case "$AGENT_ROLE" in
    "frontend")
        if ! validate_scope "$FILE_PATH" "${FRONTEND_PATTERNS[@]}"; then
            echo "Warning: Frontend agent editing non-frontend file: $FILE_PATH" >&2
        fi
        ;;
    "backend")
        if ! validate_scope "$FILE_PATH" "${BACKEND_PATTERNS[@]}"; then
            echo "Warning: Backend agent editing non-backend file: $FILE_PATH" >&2
        fi
        ;;
    "devops")
        if ! validate_scope "$FILE_PATH" "${DEVOPS_PATTERNS[@]}"; then
            echo "Warning: DevOps agent editing non-devops file: $FILE_PATH" >&2
        fi
        ;;
    "qa")
        if ! validate_scope "$FILE_PATH" "${QA_PATTERNS[@]}"; then
            echo "Warning: QA agent editing non-test file: $FILE_PATH" >&2
        fi
        ;;
esac

exit 0
