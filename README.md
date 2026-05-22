# App Pro

Unified Linux system utility: application installer, uninstaller, system cleaner, process manager, and app dashboard.

## Features

- **Install** .deb, .AppImage, and .zip applications
- **Uninstall** any application installed via App Pro
- **View** all installed apps with type, version, and install date
- **Manage** running processes with kill/force-kill
- **Clean** system caches: user cache, APT cache, thumbnails, orphan packages
- **System Info**: OS, kernel, CPU, memory, disk usage, cache sizes
- **Modern GTK4 UI** with dark/light theme support
- **SQLite** database tracking all installed apps and cleanup history
- **Auto-creates .desktop entries** for AppImage and zip installations

## Quick Start

### Download (pre-built binary)

```bash
# Download latest release
curl -LO https://github.com/YOUR_USER/app-pro/releases/latest/download/app-pro-linux-x86_64
chmod +x app-pro-linux-x86_64
./app-pro-linux-x86_64
```

### Install to system

```bash
sudo install -m 755 app-pro-linux-x86_64 /usr/local/bin/app-pro
# Now you can run from anywhere:
app-pro
```

### Build from source

```bash
# Install GTK4 development libraries
sudo apt install libgtk-4-dev libgraphene-1.0-dev libvulkan-dev \
  libharfbuzz-dev libfontconfig-dev libfreetype-dev libpixman-1-dev

# Build
cargo build --release

# Run
./target/release/app-pro

# Run with a file pre-selected
./target/release/app-pro /path/to/package.deb
```

### Test

```bash
cargo test
```

## GitHub Actions

Every push to `main` triggers an automatic build. The binary is available as a build artifact.
Pushing a tag `v*` creates a GitHub Release with the binary attached.

## Project Structure

```
src/
├── main.rs
├── core/mod.rs           # System execution, file ops
├── installer/
│   ├── mod.rs, deb.rs, appimage.rs, zip.rs
├── manager/
│   ├── mod.rs, processes.rs
├── cleaner/
│   ├── mod.rs, cache.rs
├── db/mod.rs             # SQLite database
└── ui/
    ├── app.rs, sidebar.rs
    ├── install_page.rs, installed_page.rs
    ├── processes_page.rs, cleaner_page.rs
    ├── info_page.rs, mod.rs, style.css
```

## License

MIT
