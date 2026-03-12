## 1. Project Setup

- [x] 1.1 Initialize Cargo project with `cargo init` and configure `Cargo.toml` with dependencies (`ureq`, `serde`, `serde_json`, `gix`, `keyring`, `clap`, `dirs`) and release profile (`opt-level = "s"`, `lto = true`, `strip = true`)
- [x] 1.2 Create module structure: `main.rs`, `cli.rs`, `statusline.rs`, `usage.rs`, `git.rs`, `format.rs`

## 2. Format Utilities (`format.rs`)

- [x] 2.1 Implement progress bar renderer (`▰▱` blocks, 10 segments, percentage)
- [x] 2.2 Implement ANSI color helpers (green, yellow, orange, red, reset) with threshold-based coloring
- [x] 2.3 Implement duration formatter (e.g., `2h15m`, `45m`, `3m`)
- [x] 2.4 Implement home directory path shortener (`/Users/alice/...` → `~/...`)

## 3. Git Info (`git.rs`)

- [x] 3.1 Implement branch name detection via `gix` (named branch or short commit hash for detached HEAD)
- [x] 3.2 Implement dirty/clean working tree detection
- [x] 3.3 Add graceful error handling — return `None` on any `gix` failure

## 4. Usage Fetching (`usage.rs`)

- [x] 4.1 Implement OAuth token resolution chain (env var → keyring → credentials file)
- [x] 4.2 Implement API fetch from `https://api.anthropic.com/api/oauth/usage` with 3s timeout via `ureq`
- [x] 4.3 Implement file-based cache at `/tmp/csl/usage-cache.json` with 60s TTL, stale fallback, and corrupt cache handling
- [x] 4.4 Parse API response into rate limit window structs (window label, usage percentage, reset time)

## 5. Statusline Core (`statusline.rs`)

- [x] 5.1 Define stdin JSON input struct with serde deserialization (model, contextWindow, tokensUsed, workingDirectory, sessionStartTime)
- [x] 5.2 Implement session info line rendering (model, context bar, directory+branch, duration)
- [x] 5.3 Implement rate limit lines rendering (one per active window with bar, percentage, reset info)
- [x] 5.4 Wire up the full pipeline: parse stdin → fetch usage → get git info → format → print

## 6. CLI Management (`cli.rs`)

- [x] 6.1 Implement `csl install`: copy binary to `~/.claude/csl`, backup existing statusLine config, update `settings.json`
- [x] 6.2 Implement `csl uninstall`: restore backup, delete binary, delete cache, handle not-installed case
- [x] 6.3 Add error handling for invalid `settings.json` (exit 1 with message, don't overwrite)

## 7. Entry Point (`main.rs`)

- [x] 7.1 Set up `clap` with subcommands (`install`, `uninstall`) and no-args stdin mode
- [x] 7.2 Dispatch to CLI handler or statusline handler based on args
- [x] 7.3 Ensure statusline mode always exits 0 (catch all panics/errors)

## 8. Testing & Validation

- [x] 8.1 Add unit tests for format utilities (progress bar, duration, path shortening)
- [x] 8.2 Add unit tests for stdin JSON parsing (valid, missing fields, malformed)
- [x] 8.3 Add integration test: pipe sample JSON through binary and verify output format
- [x] 8.4 Build release binary and verify size is within 5-10MB target
