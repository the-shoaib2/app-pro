#!/usr/bin/env bash
set -euo pipefail

REPO="the-shoaib2/app-pro"

echo "=== App Pro Installer ==="

[ "$EUID" -eq 0 ] || { echo "Run with sudo"; exit 1; }

ARCH=$(uname -m)
case "$ARCH" in
    x86_64)  S="linux-x86_64"  ;;
    aarch64|arm64) S="linux-arm64" ;;
    *) echo "Unsupported: $ARCH"; exit 1 ;;
esac

TMP=$(mktemp -d) && trap 'rm -rf "$TMP"' EXIT

printf "Checking version... "
TAG=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" |
    grep -m1 '"tag_name":' | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/')
[ -n "$TAG" ] || { echo "FAILED"; exit 1; }
echo "$TAG"

URL="https://github.com/${REPO}/releases/download/${TAG}/app-pro-${S}"
echo ""
curl -# -fL -o "$TMP/app-pro" "$URL"
echo ""

install -m 755 "$TMP/app-pro" /usr/local/bin/app-pro
echo "=== Installed ==="
echo "Run: app-pro"
