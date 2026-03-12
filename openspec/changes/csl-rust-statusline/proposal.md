## Why

The current [claude-statusline](https://github.com/kamranahmedse/claude-statusline) relies on bash scripts and Node.js, requiring runtime dependencies (`jq`, `curl`, `git`) and spawning multiple processes per refresh. This results in slow updates and a fragile dependency chain. A single Rust binary eliminates all runtime dependencies, achieves sub-5ms cache-hit performance, and provides a more compact, visually improved statusline.

## What Changes

- Replace the bash + Node.js statusline with a single Rust binary (`csl`)
- Two operating modes: **statusline mode** (reads JSON from stdin, outputs formatted line) and **CLI mode** (`csl install` / `csl uninstall`)
- Pure-Rust git integration via `gix` — no shelling out to `git`
- OAuth token resolution chain: env var → keychain → credentials file
- API usage fetching with 60s TTL file cache and graceful fallback
- Block-style progress bars (`▰▱`) with color thresholds (green/yellow/orange/red)
- Self-installing: copies binary to `~/.claude/csl` and configures `settings.json`

## Capabilities

### New Capabilities

- `statusline-output`: Core statusline formatting — parse stdin JSON, render model info, context bar, git status, session duration, and rate limit lines to stdout
- `usage-fetching`: OAuth token resolution, API usage fetching with caching (60s TTL), and graceful degradation when unavailable
- `git-info`: Pure-Rust git branch detection and dirty/clean status via `gix`
- `cli-management`: `csl install` and `csl uninstall` subcommands for self-installation into `~/.claude/`

### Modified Capabilities

_(none — this is a greenfield project)_

## Impact

- **New binary**: `csl` Rust crate with 6 source modules
- **Dependencies**: `ureq`, `serde`/`serde_json`, `gix`, `keyring`, `clap`, `dirs`
- **Filesystem**: writes to `~/.claude/csl` (binary), `~/.claude/settings.json` (config), `/tmp/csl/usage-cache.json` (cache)
- **Network**: calls `https://api.anthropic.com/api/oauth/usage` (with 3s timeout)
- **Replaces**: the existing bash/Node.js claude-statusline setup
