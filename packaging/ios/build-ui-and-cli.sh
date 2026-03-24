#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DIST_DIR="$REPO_ROOT/dist"
TARGET_DIR="$REPO_ROOT/dist-target-ios"
TARGET="aarch64-apple-ios"

if ! command -v xcrun >/dev/null 2>&1; then
  echo "xcrun is required. Run this script on macOS with Xcode installed." >&2
  exit 1
fi

export SDKROOT="${SDKROOT:-$(xcrun --sdk iphoneos --show-sdk-path)}"

cd "$REPO_ROOT"
rustup target add "$TARGET"

cargo build --release --target "$TARGET" --no-default-features --bin crl --target-dir "$TARGET_DIR"
cargo build --release --target "$TARGET" --bin crl-desktop --target-dir "$TARGET_DIR"

mkdir -p "$DIST_DIR"
cp "$TARGET_DIR/$TARGET/release/crl" "$DIST_DIR/crl-ios-cli"
cp "$TARGET_DIR/$TARGET/release/crl-desktop" "$DIST_DIR/crl-ios-ui"

echo "iOS build artifacts copied to $DIST_DIR"
