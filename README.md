<p align="center">
  <img src="assets/logo.png" alt="App Pro Logo" width="128" height="128" />
</p>

<h1 align="center">App Pro</h1>

<p align="center">
  <b>A unified, lightweight, and modern Linux application manager and system dashboard.</b>
</p>

<p align="center">
  <a href="https://github.com/the-shoaib2/app-pro/actions/workflows/release.yml"><img src="https://img.shields.io/github/actions/workflow/status/the-shoaib2/app-pro/release.yml?style=flat-square&logo=github&label=Build" alt="Build Status"></a>
  <a href="https://github.com/the-shoaib2/app-pro/releases"><img src="https://img.shields.io/github/v/release/the-shoaib2/app-pro?style=flat-square&logo=github&color=blue" alt="Latest Release"></a>
  <img src="https://img.shields.io/badge/platform-Linux-lightgrey?style=flat-square&logo=linux&logoColor=white" alt="Platform: Linux">
  <img src="https://img.shields.io/badge/language-Rust-orange?style=flat-square&logo=rust&logoColor=white" alt="Language: Rust">
  <img src="https://img.shields.io/badge/GUI-GTK4-blue?style=flat-square&logo=gnome&logoColor=white" alt="GUI: GTK4">
  <img src="https://img.shields.io/github/license/the-shoaib2/app-pro?style=flat-square&color=green" alt="License: MIT">
</p>

---

## 🌟 Overview

**App Pro** is a production-grade utility for Debian-based Linux systems (such as Ubuntu, Debian, and Kali). Built in **Rust** using the native **GTK4** toolkit, it delivers a high-performance GUI and CLI interface without the overhead of heavy runtimes (no Tokio or reqwest, leveraging system `curl` and native standard library channels for responsive background operations).

---

## 🚀 Key Features

* 📦 **App Installer** — Drag & drop or browse to install `.deb` packages, standalone `.AppImage` files, and extractable `.zip` archives.
* 🗑️ **Clean Uninstaller** — Easily inspect and remove installed apps managed by App Pro from your system.
* ⚙️ **Process Manager** — Monitor active processes, search by name, and terminate (`kill`) or force-stop (`kill -9`) processes.
* 🧹 **Cache Cleaner** — Reclaim disk space by clearing user cache, temporary files, APT cache, thumbnails, and orphan dependencies.
* 🖥️ **System Dashboard** — View real-time info on OS release, kernel version, CPU core usage, RAM consumption, and disk status.
* 🔄 **Smart Auto-Updater** — Automatically fetches, downloads, and swaps the running binary safely from GitHub Releases (supporting both `x86_64` and `arm64`/`aarch64` architectures) via terminal or GUI.

---

## 📥 Installation

### Option 1: One-Line Installer (Recommended)
Installs the latest binary release automatically, matching your system's architecture (`x86_64` or `arm64`) and setting up the desktop launcher:

```bash
curl -fsSL https://raw.githubusercontent.com/the-shoaib2/app-pro/main/install.sh | sudo bash
```

### Option 2: Build From Source
To compile App Pro manually from source, follow these steps:

1. **Install GTK4 & System Libraries**:
   ```bash
   sudo apt update
   sudo apt install -y libgtk-4-dev libgraphene-1.0-dev libvulkan-dev \
                       libmount-dev libseccomp-dev
   ```

2. **Clone and Build**:
   ```bash
   git clone https://github.com/the-shoaib2/app-pro.git
   cd app-pro
   cargo build --release
   ```

3. **Install locally**:
   ```bash
   sudo ./install.sh ./target/release/app-pro
   ```

---

## 🔄 Updating App Pro

App Pro checks the GitHub Releases API for new updates. If a newer version is available, it downloads the binary and performs an atomic swap.

* **Via terminal CLI**:
  ```bash
  app-pro update
  ```
* **Via GUI Dashboard**:
  Go to the **System Info** tab and click **Check for Updates**.

---

## 🛠️ Usage & CLI Reference

Launch the GUI dashboard or pre-select packages for installation from the command line:

```bash
# Launch the main GUI Dashboard
app-pro

# Launch directly into the Installer tab with a pre-selected package
app-pro ~/Downloads/package.deb
app-pro ~/Downloads/application.AppImage
app-pro ~/Downloads/archive.zip

# CLI Commands
app-pro update             # Check and install updates
app-pro -v, --version      # Display version information
app-pro -h, --help         # Show help information
```

### Dashboard Tabs Overview

| Tab | Functionality |
| :--- | :--- |
| 📥 **Install** | Drag and drop files or browse locally to install package types (`.deb`, `.AppImage`, `.zip`). |
| 📋 **Installed** | View metadata, track installed packages, and uninstall them completely. |
| ⚡ **Processes** | List running processes, filter/search by name, and terminate selected processes. |
| 🧹 **Cleaner** | Analyze and delete user cache, APT logs, thumbnail cache, and orphaned dependencies. |
| 📊 **System Info** | Graphical hardware monitors, system information, and the auto-update manager. |

---

## 🗑️ Uninstallation

To cleanly remove App Pro along with its desktop shortcut and metadata database:

```bash
curl -fsSL https://raw.githubusercontent.com/the-shoaib2/app-pro/main/uninstall.sh | sudo bash
```

*Or, from a local source folder:*
```bash
sudo ./uninstall.sh
```

---

## ⚖️ License

Distributed under the **MIT License**.

