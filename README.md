<p align="center">
  <img src="assets/logo.png" alt="App Pro Logo" width="100" height="100" />
</p>

<h1 align="center">App Pro</h1>

<p align="center">
  <b>A lightweight Linux application manager & system dashboard built in Rust and GTK4.</b>
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

## 🚀 Features

* 📦 **Install** — Drag & drop or browse `.deb`, `.AppImage`, or `.zip` files.
* 🗑️ **Uninstall** — Easily inspect and cleanly remove installed apps.
* ⚡ **Processes** — Monitor running processes and terminate/force-kill them.
* 🧹 **Cleaner** — Free disk space by clearing user, APT, thumbnail caches & orphans.
* 📊 **System Info** — Live monitors for CPU, memory, disk, and auto-updates.

---

## 📥 Installation & Setup

### Install (Recommended)
Automatically detects architecture (`x86_64` or `arm64`) and configures desktop integration:
```bash
curl -fsSL https://raw.githubusercontent.com/the-shoaib2/app-pro/main/install.sh | sudo bash
```

### Build from Source
```bash
# 1. Install GTK4 dependencies
sudo apt update && sudo apt install -y libgtk-4-dev libgraphene-1.0-dev libvulkan-dev libmount-dev libseccomp-dev

# 2. Build and install
cargo build --release && sudo ./all add in install scritp 
install.sh ./target/release/app-pro
```

---

## 🛠️ Usage

```bash
app-pro                  # Launch GUI dashboard
app-pro <file_path>      # Pre-select a file (.deb, .AppImage, .zip) to install
app-pro update           # Check and perform auto-update via CLI
app-pro -h, --help       # Show help information
app-pro -v, --version    # Show version
```

> **Update Note**: You can also check for updates directly inside the **System Info** tab in the GUI.

---

## 🗑️ Uninstallation

```bash
curl -fsSL https://raw.githubusercontent.com/the-shoaib2/app-pro/main/uninstall.sh | sudo bash
```
*(Or run `sudo ./uninstall.sh` in the source folder).*

---

## ⚖️ License

Distributed under the **MIT License**.


