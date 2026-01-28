#!/bin/bash
# Create audit trail for ccswarm sessions
# Logs session activity for compliance and debugging

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

# Create audit log entry
AUDIT_FILE="$AUDIT_DIR/$(date +%Y-%m-%d).jsonl"

# Build audit entry
AUDIT_ENTRY=$(jq -n \
    --arg ts "$TIMESTAMP" \
    --arg sid "$SESSION_ID" \
    --arg aid "$AGENT_ID" \
    --arg role "$AGENT_ROLE" \
    --arg event "session_activity" \
    '{
        timestamp: $ts,
        session_id: $sid,
        agent_id: $aid,
        agent_role: $role,
        event: $event,
        details: {}
    }')

# Append to audit log
echo "$AUDIT_ENTRY" >> "$AUDIT_FILE"

exit 0
