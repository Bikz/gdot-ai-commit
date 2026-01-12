#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FORMULA_SRC="${FORMULA_SRC:-$ROOT_DIR/homebrew/goodcommit.rb}"
TAP_REPO="${TAP_REPO:-Bikz/homebrew-tap}"
TAP_BRANCH="${TAP_BRANCH:-main}"
TAP_REMOTE_URL="${TAP_REMOTE_URL:-https://github.com/${TAP_REPO}.git}"

if ! command -v git >/dev/null 2>&1; then
  echo "error: git is required" >&2
  exit 1
fi

if [ ! -f "$FORMULA_SRC" ]; then
  echo "error: formula not found at $FORMULA_SRC" >&2
  exit 1
fi

TAP_DIR="$(mktemp -d)"
cleanup() {
  rm -rf "$TAP_DIR"
}
trap cleanup EXIT

echo "Cloning tap repo: $TAP_REPO"
git clone "$TAP_REMOTE_URL" "$TAP_DIR"

mkdir -p "$TAP_DIR/Formula"
cp "$FORMULA_SRC" "$TAP_DIR/Formula/goodcommit.rb"

cd "$TAP_DIR"
git add Formula/goodcommit.rb

if git diff --cached --quiet; then
  echo "No changes to publish."
  exit 0
fi

version="$(grep -E '^\s*version "' Formula/goodcommit.rb | head -n1 | sed -E 's/.*version "([^"]+)".*/\1/')"
if [ -z "$version" ]; then
  version="unknown"
fi

git commit -m "chore: update goodcommit ${version}"
git push origin "$TAP_BRANCH"

echo "Published formula to $TAP_REPO on branch $TAP_BRANCH"
