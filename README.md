# App Pro

Linux system utility: app installer (.deb/.AppImage/.zip), uninstaller, process manager, system cleaner, and info dashboard.

```bash
# Download & run
curl -LO https://github.com/YOUR_USER/app-pro/releases/latest/download/app-pro-linux-x86_64
chmod +x app-pro-linux-x86_64 && ./app-pro-linux-x86_64
sudo install -m 755 app-pro-linux-x86_64 /usr/local/bin/app-pro  # system-wide
```

## Build from source

```bash
sudo apt install libgtk-4-dev libgraphene-1.0-dev libvulkan-dev
cargo build --release && ./target/release/app-pro
cargo test
```

## Structure

```
src/
├── main.rs                # Entry point
├── core/                  # Sys exec, file ops
├── installer/             # .deb/.AppImage/.zip
├── manager/               # App & process management
├── cleaner/               # Cache cleanup
├── db/                    # SQLite database
└── ui/                    # GTK4 interface
```

MIT
