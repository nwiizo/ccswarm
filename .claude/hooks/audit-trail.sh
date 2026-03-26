#!/bin/bash
# Audit trail for ccswarm sessions and Agent Teams
# Logs session/subagent lifecycle events for compliance and debugging
# Triggered by: Stop, SubagentStop hooks

set -e

# Configuration
AUDIT_DIR="${CCSWARM_HOME:-$HOME/.ccswarm}/audit"
mkdir -p "$AUDIT_DIR"

# Read hook context from stdin
HOOK_INFO=$(cat)

# Extract relevant information
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
SESSION_ID="${CCSWARM_SESSION_ID:-unknown}"
AGENT_ID="${CCSWARM_AGENT_ID:-main}"
AGENT_ROLE="${CCSWARM_AGENT_ROLE:-orchestrator}"

# Detect event type from hook context
HOOK_EVENT=$(echo "$HOOK_INFO" | jq -r '.hook_event // "stop"' 2>/dev/null || echo "stop")
SUBAGENT_NAME=$(echo "$HOOK_INFO" | jq -r '.agent_name // ""' 2>/dev/null || echo "")
SUBAGENT_TYPE=$(echo "$HOOK_INFO" | jq -r '.subagent_type // ""' 2>/dev/null || echo "")

# Determine event type
if [ -n "$SUBAGENT_NAME" ]; then
    EVENT="subagent_stop"
else
    EVENT="session_stop"
fi

# Create audit log entry
AUDIT_FILE="$AUDIT_DIR/$(date +%Y-%m-%d).jsonl"

# Build audit entry
AUDIT_ENTRY=$(jq -n \
    --arg ts "$TIMESTAMP" \
    --arg sid "$SESSION_ID" \
    --arg aid "$AGENT_ID" \
    --arg role "$AGENT_ROLE" \
    --arg event "$EVENT" \
    --arg subagent "$SUBAGENT_NAME" \
    --arg subagent_type "$SUBAGENT_TYPE" \
    '{
        timestamp: $ts,
        session_id: $sid,
        agent_id: $aid,
        agent_role: $role,
        event: $event,
        subagent: (if $subagent != "" then $subagent else null end),
        subagent_type: (if $subagent_type != "" then $subagent_type else null end)
    }')

# Append to audit log
echo "$AUDIT_ENTRY" >> "$AUDIT_FILE"

exit 0
