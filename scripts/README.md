# Scripts Directory

This directory contains utility scripts for ccswarm development and maintenance.

## Git Operations

These scripts are for local development use only and should not be committed to the repository:

- `git_operations.sh` - General git operations and cleanup
- `execute_git_ops.sh` - Execute git operations with safety checks
- `merge_to_master.sh` - Merge main branch to master
- `sync_and_push.sh` - Sync with remote and push changes
- `commit_and_push.sh` - Create commits and push to remote
- `remove_shells.sh` - Remove temporary shell scripts
- `organize_scripts.sh` - Organize scripts into this directory

## Usage

```bash
cd scripts/
chmod +x <script_name>.sh
./<script_name>.sh
```

## Note

These scripts are excluded from version control via `.gitignore`.
They are generated as needed for local development workflows.