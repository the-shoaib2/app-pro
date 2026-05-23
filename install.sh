#!/usr/bin/env bash
set -euo pipefail

REPO="the-shoaib2/app-pro"
BINARY_NAME="app-pro"
DEST_DIR="/usr/local/bin"
DEST_PATH="${DEST_DIR}/${BINARY_NAME}"

echo "=== App Pro Installer ==="

if [ "$EUID" -ne 0 ]; then
    echo "This script requires root privileges."
    echo "  curl -fsSL https://raw.githubusercontent.com/${REPO}/main/install.sh | sudo bash"
    exit 1
fi

ARCH=$(uname -m)
case "$ARCH" in
    x86_64)  SUFFIX="linux-x86_64"  ;;
    aarch64|arm64) SUFFIX="linux-arm64" ;;
    *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

TEMP_DIR=$(mktemp -d -t app-pro.XXXXXX)
trap 'rm -rf "$TEMP_DIR"' EXIT

LOCAL_BIN="${1:-}"

if [ -n "$LOCAL_BIN" ]; then
    if [ ! -f "$LOCAL_BIN" ]; then
        echo "ERROR: '$LOCAL_BIN' not found."
        exit 1
    fi
    echo "Installing local binary..."
    cp "$LOCAL_BIN" "${TEMP_DIR}/${BINARY_NAME}"
else
    echo -n "Checking latest version... "
    LATEST_JSON=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest")
    VERSION_TAG=$(echo "$LATEST_JSON" | grep -m1 '"tag_name":' | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/')
    if [ -z "$VERSION_TAG" ]; then
        echo "FAILED"
        exit 1
    fi
    echo "$VERSION_TAG"

    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION_TAG}/app-pro-${SUFFIX}"
    echo ""
    echo "Downloading..."
    if ! curl -# -o "${TEMP_DIR}/${BINARY_NAME}" "$DOWNLOAD_URL" 2>&1; then
        echo ""
        echo "Download failed."
        exit 1
    fi
    echo ""
fi

chmod +x "${TEMP_DIR}/${BINARY_NAME}"

echo -n "Installing to $DEST_PATH... "
mkdir -p "$DEST_DIR"
install -m 755 "${TEMP_DIR}/${BINARY_NAME}" "$DEST_PATH"
echo "done"

echo ""
echo "=== Installed ==="
echo "Run: app-pro"
