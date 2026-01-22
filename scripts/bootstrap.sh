#!/usr/bin/env bash
set -euo pipefail

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required tool: $1" >&2
    exit 1
  fi
}

require_cmd git
require_cmd cargo

if command -v rustup >/dev/null 2>&1; then
  rustup component add rustfmt clippy >/dev/null
else
  echo "rustup not found; ensure rustfmt/clippy are installed" >&2
fi

if ! command -v python3 >/dev/null 2>&1; then
  echo "python3 not found; scripts/verify-npm.sh requires python3" >&2
  exit 1
fi

if ! command -v node >/dev/null 2>&1; then
  echo "node not found; required only for npm packaging work" >&2
fi

echo "bootstrap complete"
