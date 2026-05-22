# App Pro

Linux system utility — install apps (.deb, .AppImage, .zip), uninstall, manage processes, clean caches, view system info.

## Install

```bash
curl -LO https://github.com/the-shoaib2/app-pro/releases/latest/download/app-pro-linux-x86_64
chmod +x app-pro-linux-x86_64
sudo install -m 755 app-pro-linux-x86_64 /usr/local/bin/app-pro
app-pro
```

Or build from source:

```bash
sudo apt install libgtk-4-dev libgraphene-1.0-dev libvulkan-dev
cargo build --release
./target/release/app-pro
```

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
