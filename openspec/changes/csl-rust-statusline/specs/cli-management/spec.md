## ADDED Requirements

### Requirement: Install binary and configure settings
The `csl install` command SHALL copy the running binary to `~/.claude/csl`, back up any existing `statusLine` config, and set the statusLine command in `~/.claude/settings.json`.

#### Scenario: Fresh install
- **WHEN** `csl install` is run and no previous installation exists
- **THEN** the binary is copied to `~/.claude/csl`, `~/.claude/` is created if needed, and `settings.json` is updated with `{"statusLine": {"type": "command", "command": "$HOME/.claude/csl"}}`

#### Scenario: Existing statusLine config
- **WHEN** `csl install` is run and `settings.json` already has a `statusLine` entry
- **THEN** the existing config is backed up to `~/.claude/statusline-backup.json` before overwriting

#### Scenario: Already installed
- **WHEN** `csl install` is run and the binary is already at `~/.claude/csl` with correct settings
- **THEN** the command is idempotent — it prints a message and exits 0

#### Scenario: Invalid settings.json
- **WHEN** `settings.json` exists but contains invalid JSON
- **THEN** the command prints an error message and exits with code 1 without overwriting the file

### Requirement: Uninstall binary and restore settings
The `csl uninstall` command SHALL remove the installed binary, restore any backed-up statusLine config, and clean up the cache directory.

#### Scenario: Clean uninstall with backup
- **WHEN** `csl uninstall` is run and a backup exists at `~/.claude/statusline-backup.json`
- **THEN** the statusLine config is restored from backup, the backup file is deleted, `~/.claude/csl` is deleted, and `/tmp/csl/` is deleted

#### Scenario: Clean uninstall without backup
- **WHEN** `csl uninstall` is run and no backup exists
- **THEN** the `statusLine` key is removed from `settings.json`, `~/.claude/csl` is deleted, and `/tmp/csl/` is deleted

#### Scenario: Not installed
- **WHEN** `csl uninstall` is run but no installation is found
- **THEN** the command prints a message and exits 0

### Requirement: CLI exit codes
The CLI subcommands SHALL exit with code 0 on success and code 1 on error.

#### Scenario: Successful operation
- **WHEN** install or uninstall completes successfully
- **THEN** the process exits with code 0

#### Scenario: Failed operation
- **WHEN** install or uninstall encounters an error (e.g., permission denied, invalid JSON)
- **THEN** the process prints an error to stderr and exits with code 1
