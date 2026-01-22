# Contributing

Thanks for helping improve Good Commit.

## Prerequisites

- Rust stable via `rustup`.
- Git.
- Python 3 (used by repo scripts).
- Node 20 (only needed for npm packaging work).

## One-command setup

```bash
scripts/bootstrap.sh
```

## Common commands

```bash
scripts/lint.sh
scripts/test.sh
scripts/audit.sh
scripts/verify-npm.sh
```

Notes:
- `scripts/lint.sh` runs `cargo fmt` and `cargo clippy`.
- `scripts/test.sh` runs all Rust tests in the workspace.
- `scripts/audit.sh` runs `cargo audit` (install with `cargo install cargo-audit --locked`).
- `scripts/verify-npm.sh` checks npm wrapper files and version alignment.

## Pre-commit hooks (optional but recommended)

Install pre-commit and set up the hooks:

```bash
pre-commit install
```

Hooks run `scripts/lint.sh` to catch formatting/lint issues early.

## Logging

Set `GOODCOMMIT_LOG_JSON=1` to emit structured JSON logs for easier debugging.

## Branch protection and reviews

This repo uses `CODEOWNERS` (`.github/CODEOWNERS`). Main branch should require review before merge.

## Releases

See `RELEASING.md` for the release checklist and tag workflow.
