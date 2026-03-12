# CSL — Claude Statusline in Rust

## Overview

`csl` is a single Rust binary that displays a rich statusline in Claude Code showing model info, context usage, git status, session duration, and API rate limits. It replaces the original [claude-statusline](https://github.com/kamranahmedse/claude-statusline) (bash + Node.js) with a zero-dependency, fast native binary.

## Goals

- **Single binary** — no runtime dependencies on `jq`, `curl`, or `git` CLI
- **Fast** — sub-5ms on cache hits (no network, no process spawning)
- **macOS + Linux** support

## Architecture

### Two Modes

- **Statusline mode** (no args): Reads JSON from stdin, fetches usage data, outputs formatted statusline to stdout. This is what Claude Code calls on every refresh.
- **CLI mode** (subcommands): `csl install` and `csl uninstall` manage `~/.claude/settings.json`.

### Crate Selection

| Concern | Crate | Rationale |
|---|---|---|
| HTTP client | `ureq` | Blocking, minimal, no async runtime — fast cold start |
| JSON | `serde` + `serde_json` | Standard |
| Git | `gix` | Pure Rust — no C deps, no shelling out |
| Credentials | `keyring` | Cross-platform keychain/secret-tool |
| Terminal colors | Inline ANSI | No heavy crate needed |
| CLI args | `clap` (derive) | Lightweight arg parsing |
| Home dir | `dirs` | Cross-platform `~` resolution |

No async runtime. `ureq` keeps the binary small and startup instant.

## Data Flow (Statusline Mode)

1. **Parse stdin JSON** from Claude Code (model name, context window size, token usage, working directory, session start time)
2. **Resolve OAuth token** (first wins):
   1. `CLAUDE_OAUTH_TOKEN` env var
   2. `keyring` crate (macOS Keychain / Linux secret-tool)
   3. `~/.claude/.credentials.json` file
3. **Fetch usage data** with caching:
   - Cache: `/tmp/csl/usage-cache.json`, 60s TTL
   - Fresh cache → use it, skip HTTP
   - Stale cache → attempt refresh from `https://api.anthropic.com/api/oauth/usage`
   - Refresh fails → fall back to stale cache
   - No token → skip usage section entirely
4. **Resolve git info** via `gix`: branch name, dirty/clean status
5. **Format and print** statusline to stdout

## Output Format

**Line 1 — Session info:**
```
sonnet 4.6  ▰▰▰▰▰▱▱▱▱▱ 48%  ~/projects/myapp (main)  ⏱ 2h15m
```
- Model name
- Context usage as block progress bar (`▰▱`)
- Shortened directory + git branch in parens
- Session duration

**Rate limit lines (one per active window):**
```
5h  ▰▰▰▰▰▰▰▱▱▱ 72%  resets in 3h12m
7d  ▰▰▱▱▱▱▱▱▱▱ 18%  resets Mar 15
```
- Only shown when usage data is available
- Extra credits line only if paid extra usage is enabled
- Colors: green (<50%), yellow (50-70%), orange (70-90%), red (>=90%)

**Improvements over original:**
- Block bars (`▰▱`) instead of circles (`●○`)
- More compact layout
- Git branch turns yellow if working tree is dirty

## Install / Uninstall

### `csl install`

1. Ensure `~/.claude/` exists
2. Copy running binary to `~/.claude/csl`
3. Backup existing `statusLine` config to `~/.claude/statusline-backup.json`
4. Set `statusLine` in `~/.claude/settings.json`:
   ```json
   { "statusLine": { "type": "command", "command": "$HOME/.claude/csl" } }
   ```
5. Idempotent — skip if already installed

### `csl uninstall`

1. Restore `statusLine` from backup or remove the key
2. Delete `~/.claude/csl`
3. Delete `/tmp/csl/` cache
4. Print confirmation

## Error Handling

**Philosophy: never break the statusline.** Degrade gracefully.

| Scenario | Behavior |
|---|---|
| Malformed stdin JSON | Print empty string, exit 0 |
| No OAuth token | Show session line, skip rate limits |
| API call fails | Use stale cache, or skip rate limits |
| Corrupted cache | Delete and refetch, or skip |
| Not in git repo | Show directory without branch |
| `gix` fails | Show directory without branch |
| `settings.json` invalid | Error message, don't overwrite |

- HTTP timeout: 3 seconds
- Statusline mode: always exit 0
- CLI mode: exit 0 success, exit 1 error

## Project Structure

```
csl/
├── Cargo.toml
├── src/
│   ├── main.rs          # Entry point — dispatch to CLI or statusline
│   ├── cli.rs           # install/uninstall subcommands
│   ├── statusline.rs    # Core statusline formatting and output
│   ├── usage.rs         # API fetch, caching, token resolution
│   ├── git.rs           # Branch name, dirty status via gix
│   └── format.rs        # Progress bars, colors, duration formatting
```

### Release Profile

```toml
[profile.release]
opt-level = "s"
lto = true
strip = true
```

Target binary size: 5-10 MB. If `gix` bloats beyond acceptable, fall back to shelling out to `git` CLI.
