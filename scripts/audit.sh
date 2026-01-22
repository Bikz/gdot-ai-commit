#!/usr/bin/env bash
set -euo pipefail

if ! command -v cargo-audit >/dev/null 2>&1; then
  echo "cargo-audit not installed; run: cargo install cargo-audit --locked" >&2
  exit 1
fi

cargo audit
