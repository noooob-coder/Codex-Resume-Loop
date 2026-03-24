#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INSTALL_DIR="${CRL_INSTALL_DIR:-$HOME/.local/bin}"

mkdir -p "$INSTALL_DIR"
install -m 755 "$SCRIPT_DIR/crl" "$INSTALL_DIR/crl"

cat <<EOF
Installed crl to: $INSTALL_DIR/crl

If '$INSTALL_DIR' is not already in your PATH, add this line to your shell profile:
export PATH="$INSTALL_DIR:\$PATH"
EOF
