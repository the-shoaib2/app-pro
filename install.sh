#!/usr/bin/env bash
set -euo pipefail

# App Pro - Installer
# Usage: curl -L https://github.com/USER/app-pro/releases/latest/download/app-pro-linux-x86_64 | bash
#   or: ./install.sh /path/to/app-pro

BIN="${1:-./target/release/app-pro}"
DEST="/usr/local/bin/app-pro"

if [ ! -f "$BIN" ]; then
    echo "Usage: $0 <path-to-app-pro-binary>"
    echo "Or download from https://github.com/USER/app-pro/releases"
    exit 1
fi

echo "Installing App Pro to $DEST..."
sudo install -m 755 "$BIN" "$DEST"
echo "Installed! Run with: app-pro"
