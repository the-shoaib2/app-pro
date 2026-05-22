# App Pro

Linux system utility — install apps (.deb, .AppImage, .zip), uninstall, manage processes, clean caches, view system info.

## Install

### One-line Installer Command
Install the latest release automatically (detects system architecture: x86_64 or arm64):
```bash
curl -fsSL https://raw.githubusercontent.com/the-shoaib2/app-pro/main/install.sh | sudo bash
```

### Or build from source:
```bash
# 1. Install GTK4 dependencies
sudo apt install libgtk-4-dev libgraphene-1.0-dev libvulkan-dev

# 2. Build the project
cargo build --release

# 3. Install locally
sudo ./install.sh ./target/release/app-pro
```

## Uninstallation
To completely remove App Pro from your system:
```bash
curl -fsSL https://raw.githubusercontent.com/the-shoaib2/app-pro/main/uninstall.sh | sudo bash
```
Or if you have the source tree:
```bash
sudo ./uninstall.sh
```

## Auto-Update System
App Pro supports safe and secure auto-updates using the GitHub Releases API.
* **CLI update**: Run `app-pro update` in the terminal to inspect release notes and download/install the latest version.
* **GUI update**: Click the **Check for Updates** button in the **System Info** tab to check and install updates right from the dashboard.

## Usage

| Tab | What it does |
|-----|-------------|
| **Install** | Drag & drop or browse for .deb/.AppImage/.zip files to install |
| **Installed** | View and uninstall apps managed by App Pro |
| **Processes** | Running process list — kill or force-kill any process |
| **Cleaner** | Clear user cache, APT cache, thumbnails, orphan packages |
| **Info** | OS, kernel, CPU, memory, disk usage at a glance |

Pass a file to pre-select it:
```bash
app-pro ~/Downloads/some-app.deb
```

## License

MIT
