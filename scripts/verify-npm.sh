#!/usr/bin/env bash
set -euo pipefail

root_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)

if [ ! -x "$root_dir/npm/bin/goodcommit" ]; then
  echo "missing npm/bin/goodcommit" >&2
  exit 1
fi

if [ ! -x "$root_dir/npm/bin/g" ]; then
  echo "missing npm/bin/g" >&2
  exit 1
fi

npm_version=$(ROOT_DIR="$root_dir" python3 - <<'PY'
import json
import os
from pathlib import Path
root = Path(os.environ["ROOT_DIR"])
package = json.loads((root / "npm/package.json").read_text())
print(package.get("version", ""))
PY
)

if [ -z "$npm_version" ]; then
  echo "unable to read npm/package.json version" >&2
  exit 1
fi

if command -v rg >/dev/null 2>&1; then
  cargo_version=$(rg -m1 '^version = "' "$root_dir/crates/cli/Cargo.toml" | sed -E 's/version = "([^"]+)"/\1/')
else
  cargo_version=$(grep -m1 '^version = "' "$root_dir/crates/cli/Cargo.toml" | sed -E 's/version = "([^"]+)"/\1/')
fi

if [ -z "$cargo_version" ]; then
  echo "unable to read crates/cli/Cargo.toml version" >&2
  exit 1
fi

if [ "$npm_version" != "$cargo_version" ]; then
  echo "version mismatch: npm/package.json ($npm_version) != crates/cli/Cargo.toml ($cargo_version)" >&2
  exit 1
fi

echo "npm wrapper verified"
