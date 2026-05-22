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

## Requirements

- Rust 1.70+ (for building)
- GTK4 development libraries (for building)
- Linux with systemd (for runtime)
- Commands: `dpkg`, `apt-get`, `kill`, `pkill`, `unzip`, `pkexec`

### Install Build Dependencies

```bash
# Debian/Ubuntu/Kali
sudo apt install libgtk-4-dev libpango1.0-dev libcairo2-dev \
  libgdk-pixbuf-2.0-dev libgraphene-1.0-dev libvulkan-dev \
  libwayland-dev libxkbcommon-dev libepoxy-dev libharfbuzz-dev \
  libfontconfig-dev libfreetype-dev libfribidi-dev libthai-dev \
  libx11-dev libxrender-dev libxcb-render0-dev libxcb-shm0-dev \
  libpixman-1-dev libpng-dev libbrotli-dev libbz2-dev \
  libmount-dev libselinux-dev libseccomp-dev liblcms2-dev \
  libglycin-2-dev libdatrie-dev
```

## Build

```bash
# Release build
cargo build --release

# Run
./target/release/app-pro

# Run with a file to pre-select (e.g., from file manager)
./target/release/app-pro /path/to/package.deb
```

## Test

```bash
cargo test
```

## Project Structure

```
src/
├── main.rs              # Entry point, GTK initialization
├── core/mod.rs          # System execution helpers (Command wrappers, file ops)
├── installer/
│   ├── mod.rs           # InstallType enum, InstallResult struct
│   ├── deb.rs           # APT/dpkg .deb install/uninstall
│   ├── appimage.rs      # AppImage copy + .desktop entry
│   └── zip.rs           # Zip extraction + binary discovery
├── manager/
│   ├── mod.rs           # AppManager orchestration
│   └── processes.rs     # /proc process lister + kill
├── cleaner/
│   ├── mod.rs           # Cache/orphan cleanup
│   └── cache.rs         # Cache size analyzer
├── db/mod.rs            # SQLite database (apps, cleanup, settings)
└── ui/
    ├── app.rs           # Main window, CSS provider
    ├── sidebar.rs       # Navigation sidebar
    ├── install_page.rs  # File chooser + install button
    ├── installed_page.rs# Installed apps list + uninstall
    ├── processes_page.rs# Process table + kill buttons
    ├── cleaner_page.rs  # Cache cleaners + Clean All
    ├── info_page.rs     # System information display
    ├── mod.rs           # Module re-exports
    └── style.css        # 276-line theme
```

## License

MIT
