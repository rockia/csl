# claude-status

A rich status line for [Claude Code](https://claude.com/claude-code) written in Rust. Shows model info, context window usage, git branch, session duration, and rate limit consumption at a glance.

```
Claude Sonnet 4 │ ✍️ 23% │ my-project (main*) │ ⏱ 1h 23m │ ◑ default

current ▰▰▱▱▱▱▱▱▱▱  23% ⟳ 3:30pm
weekly  ▰▰▰▰▱▱▱▱▱▱  45% ⟳ mar 19, 12:00am
extra   ▰▱▱▱▱▱▱▱▱▱ $5.00/$50.00
```

## Features

- Model name and context window usage percentage
- Project directory with git branch and dirty indicator
- Session duration
- Effort level
- 5-hour (current) and 7-day (weekly) rate limit bars with color coding
- Extra/paid usage credits display
- Single static binary — no runtime dependencies

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/rockia/claude-status/main/scripts/install.sh | bash
```

## Uninstall

```bash
~/.claude/claude-status uninstall
```

## How It Works

Claude Code pipes a JSON context object to the status line command's stdin. This binary parses that context, fetches rate limit data from the Anthropic OAuth usage API (cached for 60 seconds), and outputs a formatted multi-line status.

Rate limit colors:
- Green: < 50%
- Orange: 50-69%
- Yellow: 70-89%
- Red: >= 90%

## Building from Source

```bash
cargo build --release
./target/release/claude-status install
```

## License

MIT
