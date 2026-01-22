#!/usr/bin/env bash
set -euo pipefail

cargo fmt -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
