#!/usr/bin/env bash
# App Pro - Production-Ready Secure Installer
set -euo pipefail

# Configuration
REPO="the-shoaib2/app-pro"
BINARY_NAME="app-pro"
DEST_DIR="/usr/local/bin"
DEST_PATH="${DEST_DIR}/${BINARY_NAME}"
DESKTOP_PATH="/usr/share/applications/app-pro.desktop"

echo "=== App Pro Installer ==="

# Check permissions
if [ "$EUID" -ne 0 ]; then
    echo "This script requires root privileges to install the binary to $DEST_DIR."
    echo "Please re-run with sudo:"
    echo "  curl -fsSL https://raw.githubusercontent.com/${REPO}/main/install.sh | sudo bash"
    exit 1
fi

# Detect architecture
ARCH=$(uname -m)
case "$ARCH" in
    x86_64)
        SUFFIX="linux-x86_64"
        ;;
    aarch64|arm64)
        SUFFIX="linux-arm64"
        ;;
    *)
        echo "ERROR: Unsupported architecture '$ARCH'."
        echo "App Pro currently only supports x86_64 and arm64/aarch64 Linux systems."
        exit 1
        ;;
esac

TEMP_DIR=$(mktemp -d -t app-pro-install.XXXXXX)
# Set up trap to clean up temp files on exit
trap 'rm -rf "$TEMP_DIR"' EXIT

LOCAL_BIN="${1:-}"

if [ -n "$LOCAL_BIN" ]; then
    # Install from a local binary
    if [ ! -f "$LOCAL_BIN" ]; then
        echo "ERROR: Local binary '$LOCAL_BIN' does not exist."
        exit 1
    fi
    echo "Installing local binary from '$LOCAL_BIN'..."
    cp "$LOCAL_BIN" "${TEMP_DIR}/${BINARY_NAME}"
else
    # Install from GitHub releases
    echo "Detecting latest version from GitHub..."
    LATEST_JSON=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest")
    
    # Simple JSON parsing using grep/sed to avoid dependency on jq
    VERSION_TAG=$(echo "$LATEST_JSON" | grep -m1 '"tag_name":' | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/')
    
    if [ -z "$VERSION_TAG" ]; then
        echo "ERROR: Failed to fetch the latest release version tag from GitHub."
        exit 1
    fi
    
    echo "Latest version found: $VERSION_TAG"
    
    # Determine the asset download URL
    DOWNLOAD_URL="https://github.com/the-shoaib2/app-pro/releases/download/${VERSION_TAG}/app-pro-${SUFFIX}"
    
    echo "Downloading binary from: $DOWNLOAD_URL"
    if ! curl -fsSL -o "${TEMP_DIR}/${BINARY_NAME}" "$DOWNLOAD_URL"; then
        echo "ERROR: Failed to download the binary. Please check your internet connection."
        exit 1
    fi
fi

# Make binary executable
chmod +x "${TEMP_DIR}/${BINARY_NAME}"

# Install the binary safely (overwrites previous version if it exists)
echo "Installing binary to $DEST_PATH..."
mkdir -p "$DEST_DIR"
install -m 755 "${TEMP_DIR}/${BINARY_NAME}" "$DEST_PATH"

# Create Desktop Entry for GUI launcher
echo "Installing desktop launcher to $DESKTOP_PATH..."
cat <<EOF > "$DESKTOP_PATH"
[Desktop Entry]
Name=App Pro
Comment=Unified Linux system utility and application installer
Exec=${DEST_PATH}
Icon=system-software-install
Terminal=false
Type=Application
Categories=System;Utility;Settings;
StartupNotify=true
EOF

chmod 644 "$DESKTOP_PATH"

echo "=== Installation Successful ==="
echo "You can launch App Pro from your desktop environment's menu,"
echo "or run it in the terminal with: app-pro"
