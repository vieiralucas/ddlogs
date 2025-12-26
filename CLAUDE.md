# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

ddlogs is a CLI tool for tailing and querying Datadog logs, similar to `tail -f` for Datadog. It's a single-file Rust application that wraps the Datadog API client to provide a streamlined log querying experience.

## Architecture

**Single binary structure:** The entire application lives in `src/main.rs` (~400 lines). This is intentional - the tool is simple enough not to require module separation.

**Key components:**
- `Config` struct: Handles credential loading from both `~/.config/ddlogs/config.toml` and environment variables (`DD_API_KEY`, `DD_APP_KEY`, `DD_SITE`). Env vars override config file values.
- `Args` struct: CLI argument parsing via clap's derive macros
- `fetch_logs()`: One-time log query (default: last hour)
- `follow_logs()`: Continuous polling mode that tracks timestamps to fetch only new logs
- `configure()`: Interactive credential setup command

**Datadog API integration:** Uses the `datadog-api-client` crate v0.24. The app configures the client with API/app keys and Datadog site, then uses `LogsAPI::list_logs()` to query logs.

**Output format:** Each log is printed as a single line of JSON to stdout, making it pipeable to tools like `jq`.

**Rate limiting:** Default polling interval is 12 seconds (300 requests/hour) to respect Datadog's API limits (2 requests per 10 seconds).

## Development Commands

```bash
# Build for development
cargo build

# Build release binary
cargo build --release

# Run locally
cargo run -- [args]
cargo run -- --service myapp --follow

# Format code (required for CI)
cargo fmt

# Check formatting without modifying
cargo fmt -- --check

# Run clippy (required for CI, must pass with no warnings)
cargo clippy -- -D warnings

# Run tests
cargo test
```

## Release Process

Releases are automated via GitHub Actions when a git tag matching `v[0-9]+.*` is pushed:

1. Bump version in `Cargo.toml`
2. Update `Cargo.lock` with `cargo update`
3. Commit changes
4. Create and push git tag: `git tag vX.Y.Z && git push --tags`

The release workflow (`.github/workflows/release.yml`) builds binaries for:
- `aarch64-apple-darwin` (macOS ARM64)
- `x86_64-apple-darwin` (macOS Intel)
- `x86_64-unknown-linux-gnu` (Linux x86_64)

Binaries are packaged as `.tar.gz` and uploaded to GitHub Releases.

## Configuration Management

The tool supports two credential sources (env vars take precedence):

1. Config file: `~/.config/ddlogs/config.toml` (created via `ddlogs configure`)
2. Environment variables: `DD_API_KEY`, `DD_APP_KEY`, `DD_SITE`

When modifying config logic, ensure env vars always override config file values (see `Config::load()`).

## Git Commits

When creating commits, do not add Claude as a co-author. Claude is a tool - the user of the tool is always the author. Commit messages should be concise and follow the existing style in the git history.
