<p align="center">
  <img src="assets/logo.png" alt="App Pro Logo" width="100" height="100" />
</p>

<h1 align="center">App Pro</h1>

<p align="center">
  <a href="https://github.com/the-shoaib2/app-pro/actions/workflows/release.yml"><img src="https://img.shields.io/github/actions/workflow/status/the-shoaib2/app-pro/release.yml?style=flat-square&logo=github&label=Build" alt="Build Status"></a>
  <a href="https://github.com/the-shoaib2/app-pro/releases"><img src="https://img.shields.io/github/v/release/the-shoaib2/app-pro?style=flat-square&logo=github&color=blue" alt="Latest Release"></a>
  <img src="https://img.shields.io/badge/platform-Linux-lightgrey?style=flat-square&logo=linux&logoColor=white" alt="Platform: Linux">
  <img src="https://img.shields.io/badge/language-Rust-orange?style=flat-square&logo=rust&logoColor=white" alt="Language: Rust">
  <img src="https://img.shields.io/badge/GUI-GTK4-blue?style=flat-square&logo=gnome&logoColor=white" alt="GUI: GTK4">
  <img src="https://img.shields.io/github/license/the-shoaib2/app-pro?style=flat-square&color=green" alt="License: MIT">
</p>

---

App Pro is a premium, lightweight Linux system dashboard built in Rust and powered by GTK4. It provides a sleek, single-header UI to manage, monitor, and clean your system.

## ✨ Features

* 📦 **Application Installer** — Drag & drop to install `.deb`, `.AppImage`, `.zip`, or `.tar.gz`/`.tgz` packages.
* 🗑️ **Uninstaller** — Safely remove and clean up installed applications.
* ⚡ **Process Monitor** — Search by port or name, monitor memory, and terminate processes.
* 🧹 **System Cleaner** — Free disk space by cleaning user cache, thumbnails, APT cache, orphaned packages, and vacuuming the App Pro database.
* 📊 **System Info** — Live CPU/memory stats and background system auto-updates.

---

## 📥 Installation

Install globally using the official installer script:

```bash
curl -fsSL https://raw.githubusercontent.com/the-shoaib2/app-pro/main/install.sh | sudo bash
```

---

<p align="center">
  <img src="assets/screenshot.png" alt="App Pro Interface" width="600" />
</p>

---

## 🗑️ Uninstallation

Remove App Pro and its desktop shortcuts cleanly:

```bash
curl -fsSL https://raw.githubusercontent.com/the-shoaib2/app-pro/main/uninstall.sh | sudo bash
```

To purge local app databases and settings:
```bash
rm -rf ~/.local/share/app-pro
```

---

## ⚖️ License

This project is licensed under the [MIT License](LICENSE).


