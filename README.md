# csl

A fast, single-binary statusline for [Claude Code](https://docs.anthropic.com/en/docs/claude-code).

![Rust](https://img.shields.io/badge/Rust-2024_edition-orange)
![License](https://img.shields.io/badge/license-MIT-blue)

## What it does

`csl` replaces Claude Code's default statusline with a rich, informative display:

```
claude-sonnet  ▰▰▰▰▰▱▱▱▱▱ 45%  ~/projects (main)  ⏱ 1h23m
hourly  ▱▱▱▱▱▱▱▱▱▱ 0%  resets in 42m
daily  ▰▰▱▱▱▱▱▱▱▱ 15%  resets in 8h
```

**Session line** — model name, context window usage (color-coded progress bar), working directory, git branch, and session duration.

**Rate limit lines** — Anthropic API rate limits for your account, cached and refreshed every 60 seconds.

## Features

- **Color-coded context usage** — green (<50%), yellow (50-70%), orange (70-90%), red (≥90%)
- **Git-aware** — shows branch name with dirty-tree indicator (pure Rust via `gix`, no `git` binary needed)
- **API rate limits** — fetches usage from Anthropic's API with token auto-discovery (env var, keyring, or credentials file)
- **Graceful degradation** — never crashes; falls back to cached or empty data on errors
- **Tiny binary** — release build optimized for size with LTO and stripping

## Installation

### Build from source

```bash
cargo build --release
```

### Install into Claude Code

```bash
./target/release/csl install
```

This copies the binary to `~/.claude/csl` and configures your `~/.claude/settings.json` statusline setting. Any existing statusline config is backed up automatically.

### Uninstall

```bash
~/.claude/csl uninstall
```

Restores your previous statusline configuration and removes the binary.

## How it works

Claude Code pipes JSON to the statusline binary via stdin:

```json
{
  "model": "sonnet 4.6",
  "contextWindow": 200000,
  "tokensUsed": 96000,
  "workingDirectory": "/Users/you/projects",
  "sessionStartTime": "2025-03-12T10:30:00Z"
}
```

`csl` reads this, formats the statusline, and writes it to stdout. Claude Code displays the result.

## Token resolution

Rate limit data requires an Anthropic API token. `csl` checks these sources in order:

1. `CLAUDE_OAUTH_TOKEN` environment variable
2. System keyring (`claude-api` service)
3. `~/.claude/.credentials.json`

## Project structure

```
src/
├── main.rs        # Entry point, CLI dispatch
├── statusline.rs  # Statusline rendering from stdin JSON
├── cli.rs         # Install/uninstall logic
├── git.rs         # Git branch and dirty-tree detection
├── format.rs      # Colors, progress bars, duration, paths
└── usage.rs       # API rate limit fetching and caching
```

## License

MIT
