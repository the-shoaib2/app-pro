#!/usr/bin/env bash
# App Pro - Safe Uninstaller Script
set -euo pipefail

# Ensure the script is run with root privileges
if [ "$EUID" -ne 0 ]; then
    echo "ERROR: Please run this script with sudo or as root:"
    echo "  sudo $0"
    exit 1
fi

BIN_PATH="/usr/local/bin/app-pro"
DESKTOP_PATH="/usr/share/applications/app-pro.desktop"

echo "Uninstalling App Pro..."

# Remove binary
if [ -f "$BIN_PATH" ]; then
    echo "Removing binary: $BIN_PATH"
    rm -f "$BIN_PATH"
else
    echo "Binary $BIN_PATH not found."
fi

# Remove desktop launcher
if [ -f "$DESKTOP_PATH" ]; then
    echo "Removing desktop entry: $DESKTOP_PATH"
    rm -f "$DESKTOP_PATH"
else
    echo "Desktop launcher $DESKTOP_PATH not found."
fi

# If standard user ran it under sudo, get their home dir to print cleanup hint
SUDO_USER_HOME=""
if [ -n "${SUDO_USER:-}" ]; then
    SUDO_USER_HOME=$(getent passwd "$SUDO_USER" | cut -d: -f6)
fi

echo "App Pro uninstalled successfully."

if [ -n "$SUDO_USER_HOME" ] && [ -d "$SUDO_USER_HOME/.local/share/app-pro" ]; then
    echo ""
    echo "To delete app database, history and user settings, run:"
    echo "  rm -rf \"$SUDO_USER_HOME/.local/share/app-pro\""
fi
