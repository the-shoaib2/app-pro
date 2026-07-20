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
HEADERS=(-H "Accept: application/json" -H "User-Agent: App-Pro-Client")
if [ -n "${GITHUB_TOKEN:-}" ]; then
    HEADERS+=(-H "Authorization: Bearer $GITHUB_TOKEN")
fi

TAG=$(curl "${HEADERS[@]}" -fsSL "https://api.github.com/repos/${REPO}/releases/latest" |
    grep -m1 '"tag_name":' | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/' || true)
[ -n "$TAG" ] || { echo "FAILED"; exit 1; }
echo "$TAG"

URL="https://github.com/${REPO}/releases/download/${TAG}/app-pro-${S}"
echo ""
curl -# -fL -o "$TMP/app-pro" "$URL"
echo ""

install -m 755 "$TMP/app-pro" /usr/local/bin/app-pro

# Install desktop entry
mkdir -p /usr/share/applications
cat > /usr/share/applications/app-pro.desktop << 'EOF'
[Desktop Entry]
Name=App Pro
Comment=Linux system utility: install, manage, clean, monitor
Exec=/usr/local/bin/app-pro
Icon=app-pro
Terminal=false
Type=Application
Categories=Utility;System;GTK;
StartupNotify=true
StartupWMClass=app-pro
EOF

# Install SVG icon (fallback for all sizes — scalable)
mkdir -p /usr/share/icons/hicolor/scalable/apps
cat > /usr/share/icons/hicolor/scalable/apps/app-pro.svg << 'ICONEOF'
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" width="512" height="512">
  <defs>
    <linearGradient id="bg" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#2ecc71"/>
      <stop offset="100%" stop-color="#27ae60"/>
    </linearGradient>
  </defs>
  <rect width="512" height="512" rx="96" fill="url(#bg)"/>
  <g transform="translate(96, 120)">
    <rect x="0" y="40" width="320" height="240" rx="24" fill="none" stroke="white" stroke-width="20" opacity=".9"/>
    <rect x="40" y="0" width="240" height="80" rx="16" fill="white" opacity=".2"/>
    <line x1="160" y1="80" x2="160" y2="280" stroke="white" stroke-width="12" opacity=".7"/>
    <line x1="40" y1="160" x2="280" y2="160" stroke="white" stroke-width="12" opacity=".7"/>
  </g>
  <text x="256" y="440" font-family="system-ui,-apple-system,sans-serif" font-size="96" font-weight="800" fill="white" text-anchor="middle" opacity=".95">AP</text>
</svg>
ICONEOF

# Update icon cache so the icon shows up immediately
gtk-update-icon-cache /usr/share/icons/hicolor/ 2>/dev/null || true

echo ""
echo "=== Installed ==="
echo "Run: app-pro"
