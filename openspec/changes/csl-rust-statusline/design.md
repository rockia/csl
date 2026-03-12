## Context

Claude Code supports a custom statusline via a `statusLine` config in `~/.claude/settings.json`. The current community solution (`claude-statusline`) uses bash scripts that shell out to `jq`, `curl`, and `git`, spawning multiple processes per refresh. This project builds a single Rust binary that reads Claude Code's stdin JSON and outputs a formatted statusline with zero runtime dependencies.

## Goals / Non-Goals

**Goals:**
- Single statically-linked binary with no runtime dependencies
- Sub-5ms response on cache hits (no network, no process spawning)
- Graceful degradation — never break the statusline, always exit 0
- Self-installing via `csl install` / `csl uninstall`
- macOS and Linux support

**Non-Goals:**
- Windows support
- Async runtime or background refresh daemon
- Interactive TUI or configuration UI
- Custom themes or user-configurable format strings
- Integration with non-Anthropic APIs

## Decisions

### 1. Blocking HTTP with `ureq` over async (`reqwest`)

Use `ureq` for HTTP requests instead of `reqwest` with tokio.

**Rationale**: The statusline runs once per refresh, makes at most one HTTP call, and exits. An async runtime adds ~1MB to binary size and ~2ms to cold start for zero benefit. `ureq` is blocking, minimal, and perfect for this use case.

**Alternatives**: `reqwest` (too heavy), `attohttpc` (less maintained), raw `std::net` (too low-level).

### 2. Pure-Rust git via `gix` over shelling out

Use the `gix` crate for git operations (branch name, dirty status) instead of spawning `git` CLI.

**Rationale**: Eliminates the `git` CLI dependency and avoids process spawning overhead. `gix` can read HEAD ref and check index status without a full repo scan.

**Risk mitigation**: If `gix` adds >5MB to binary size, fall back to shelling out to `git` as a compile-time feature flag.

**Alternatives**: `git2` (requires libgit2 C library), `git` CLI (process spawn overhead).

### 3. File-based cache at `/tmp/csl/`

Cache API usage responses as JSON at `/tmp/csl/usage-cache.json` with a 60-second TTL.

**Rationale**: Simple, no external dependencies, survives process restarts. `/tmp` is appropriate since cached data is ephemeral and re-fetchable. The 60s TTL balances freshness against API rate limits.

**Alternatives**: In-memory cache (lost on exit), SQLite (overkill), XDG cache dir (more correct but `/tmp` is simpler for ephemeral data).

### 4. OAuth token resolution chain

Resolve the API token in priority order: `CLAUDE_OAUTH_TOKEN` env var → `keyring` crate (OS keychain) → `~/.claude/.credentials.json` file.

**Rationale**: Env var allows CI/scripting override. Keychain is the most secure default. File fallback covers cases where keychain isn't available (headless Linux without secret-tool).

### 5. Inline ANSI escapes over a color crate

Use raw ANSI escape sequences for terminal colors instead of `colored`, `termcolor`, or `owo-colors`.

**Rationale**: We need exactly 4 colors (green, yellow, orange, red) plus reset. A crate adds dependency weight for trivial functionality. A small `format.rs` module with constants is sufficient.

### 6. Module structure

```
src/
├── main.rs          # Entry point — dispatch CLI vs stdin mode
├── cli.rs           # install/uninstall subcommands
├── statusline.rs    # Core formatting pipeline (parse → enrich → render)
├── usage.rs         # Token resolution, API fetch, cache read/write
├── git.rs           # Branch name + dirty status via gix
└── format.rs        # Progress bars, ANSI colors, duration formatting
```

Each module has a clear single responsibility. `statusline.rs` orchestrates the pipeline, calling into `usage.rs`, `git.rs`, and `format.rs`.

## Risks / Trade-offs

- **`gix` binary bloat** → Mitigation: measure after initial build; fall back to `git` CLI behind a feature flag if binary exceeds 10MB
- **Keychain access fails on headless Linux** → Mitigation: `keyring` errors are caught and we fall back to credentials file; never panic
- **API endpoint changes** → Mitigation: usage section is entirely optional; any fetch failure gracefully degrades to showing only the session line
- **`/tmp` cleanup by OS** → Mitigation: cache miss just triggers a re-fetch; no data loss
- **Claude Code stdin format changes** → Mitigation: unknown fields are ignored via `serde(deny_unknown_fields = false)`; missing fields use defaults
