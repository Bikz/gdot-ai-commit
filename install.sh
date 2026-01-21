#!/bin/sh
set -e

REPO="Bikz/goodcommit"
BIN_NAME="goodcommit"
INSTALL_DIR="$HOME/.local/bin"

for cmd in curl tar mktemp; do
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "error: $cmd is required" >&2
    exit 1
  fi
done

OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS" in
  darwin)
    OS="apple-darwin"
    ;;
  linux)
    OS="unknown-linux-gnu"
    ;;
  *)
    echo "error: unsupported OS: $OS" >&2
    exit 1
    ;;
esac

case "$ARCH" in
  x86_64|amd64)
    ARCH="x86_64"
    ;;
  arm64|aarch64)
    ARCH="aarch64"
    ;;
  *)
    echo "error: unsupported arch: $ARCH" >&2
    exit 1
    ;;
esac

TARGET="${ARCH}-${OS}"
ASSET="${BIN_NAME}-${TARGET}.tar.gz"
URL="https://github.com/${REPO}/releases/latest/download/${ASSET}"

mkdir -p "$INSTALL_DIR"

TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

printf "Downloading %s...\n" "$URL"
if ! curl -fL "$URL" -o "$TMP_DIR/$ASSET"; then
  echo "error: failed to download release asset" >&2
  echo "You can build from source with: cargo build --release" >&2
  exit 1
fi

tar -xzf "$TMP_DIR/$ASSET" -C "$TMP_DIR"
if [ ! -f "$TMP_DIR/$BIN_NAME" ]; then
  echo "error: missing binary in release archive" >&2
  exit 1
fi

cp "$TMP_DIR/$BIN_NAME" "$INSTALL_DIR/$BIN_NAME"
chmod 755 "$INSTALL_DIR/$BIN_NAME"

cat > "$INSTALL_DIR/g" <<'SH'
#!/bin/sh
exec goodcommit "$@"
SH

cat > "$INSTALL_DIR/g." <<'SH'
#!/bin/sh
for arg in "$@"; do
  case "$arg" in
    --stage-all|--no-stage|--interactive) exec goodcommit "$@" ;;
  esac
done

exec goodcommit --stage-all "$@"
SH

chmod +x "$INSTALL_DIR/g" "$INSTALL_DIR/g."

echo "installed $BIN_NAME to $INSTALL_DIR"
echo "aliases installed: g, g."
echo "next: run 'goodcommit setup' to configure your provider"
case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *) echo "note: add $INSTALL_DIR to your PATH to use g or goodcommit directly" ;;
esac
