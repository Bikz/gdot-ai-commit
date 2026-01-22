# Architecture

## Overview

Good Commit is a Rust workspace with a CLI (`crates/cli`) backed by a core library (`crates/core`). Packaging lives in `npm/` and `homebrew/`.

## Key modules

- `crates/cli`: CLI entrypoints, setup flow, hooks, UI helpers, and invocation behavior (`g` vs `g.`).
- `crates/core`: config resolution, git/diff handling, prompt construction, pipeline orchestration, and provider integrations.

## Data flow

1. CLI resolves config (defaults + env + repo/global config).
2. Diff context is collected from staged changes.
3. Pipeline either sends the full diff or summarizes file diffs if too large.
4. Provider returns a candidate commit message.
5. Message is sanitized and optionally validated for conventional commit format.

## Providers

- OpenAI: supports Responses and Chat endpoints, with GPT-5 defaults to Responses.
- Ollama: local model support via REST endpoint.

## Observability

Logging uses `tracing`. CLI creates a per-run span with a `run_id` so logs can be correlated. Set `GOODCOMMIT_LOG_JSON=1` for JSON output.
