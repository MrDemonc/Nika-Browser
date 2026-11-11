<p align="center">
  <img src="nika_browser.png" width="128" height="128" alt="Nika Browser Logo" />
</p>

<h1 align="center">🌸 Nika Browser</h1>

<p align="center">
  <strong>A modern, ultra-private, minimalist and fast web browser built in Rust with GTK4, Libadwaita and WebKitGTK 6.0 for Linux.</strong>
</p>

<p align="center">
  <a href="https://github.com/MrDemonc/Nika-Browser/releases"><img src="https://img.shields.io/github/v/release/MrDemonc/Nika-Browser?color=c084fc&label=Release&style=flat-square" alt="Release"></a>
  <img src="https://img.shields.io/badge/Rust-2024%20Edition-orange?style=flat-square&logo=rust" alt="Rust 2024">
  <img src="https://img.shields.io/badge/GTK4%20%2B%20Libadwaita-1.6-blue?style=flat-square&logo=gnome" alt="GTK4 Libadwaita">
  <img src="https://img.shields.io/badge/WebKitGTK-6.0-green?style=flat-square&logo=webkit" alt="WebKitGTK 6">
  <img src="https://img.shields.io/badge/Platform-Linux%20x86__64-purple?style=flat-square&logo=linux" alt="Linux x86_64">
  <img src="https://img.shields.io/badge/License-GNU%20GPL%20v3.0-blue?style=flat-square" alt="License GPL-3.0">
</p>

---

## ⚡ Quick Installation (Linux x86_64)

You can install **Nika Browser** directly with a single command in your terminal using `curl`:

```bash
curl -fsSL https://raw.githubusercontent.com/MrDemonc/Nika-Browser/main/install.sh | bash
```

> **What does this script do?**
> - Downloads the optimized binary of the latest release (`nika-browser`).
> - Installs it to `~/.local/bin/nika-browser`.
> - Installs the official high-resolution transparent icon into `~/.local/share/icons/hicolor/512x512/apps/`.
> - Sets up the desktop application launcher (`.desktop`) so it appears in your application drawer (Rofi, Wofi, GNOME, KDE, Waybar, etc.).

---

## ✨ Key Features

### 🛡️ 1. Privacy and Security by Design
* **Zero Commercial Telemetry**: Free of user profiling, tracking scripts, and data collection.
* **Native WebKit Shield**: Blocks intrusive ads and surveillance scripts (Google Analytics, Facebook Pixel, Hotjar, tracking beacons) compiled directly into the WebKit engine using content filtering rules.
* **Automatic URL Parameter Cleaner**: Strips invasive tracking query parameters (`utm_*`, `fbclid`, `gclid`, `igshid`, `yclid`, etc.) before network requests are dispatched.
* **Automatic Cache Clearance on Exit**: Optional security setting to automatically purge disk cache, WebKit memory cache, and temporary browsing data every time the browser is closed.

### 🔒 2. Local Encrypted Password Vault
* **Military-Grade Local Encryption**: Powered by **AES-256-GCM** with key derivation using **PBKDF2** (600,000 iterations and 16-byte cryptographic salt).
* **Master Password**: Configured during initial onboarding or in Settings. Credentials are saved strictly on your local machine at `~/.config/nika-browser/passwords_vault.json`—no cloud, no remote servers.
* **Lock & Unlock with One Click**: Intuitive UI to search, reveal, copy, add, and delete stored passwords securely.

### 🌐 3. Secure DNS (DNS-over-HTTPS / DoH)
* **Encrypted DNS Queries**: Prevents your Internet Service Provider (ISP) and network eavesdroppers from monitoring the websites you visit.
* **One-Click Privacy Presets**:
  * 🛡️ **AdGuard DNS**: Blocks advertisements and malicious telemetry at the domain resolution level.
  * ⚡ **Cloudflare DNS** (`1.1.1.1`): Blazing speed and strict zero-logging policies.
  * 🔒 **Quad9 DNS** (`9.9.9.9`): Robust malware protection and security.
  * 🌐 **Google Public DNS** (`8.8.8.8`).
  * ⚙️ **Custom DoH Endpoint**: Enter any standard RFC 8484 HTTPS URL.

### 📑 4. Arc-Style Vertical Sidebar Tabs
* **Dual Display Modes**:
  * **Pinned Docked Mode**: Sits cleanly to the left of the web page with a clean divider, adjusting page width without overlaying content.
  * **Auto-Hide Mode (Hover)**: Stays hidden and smoothly slides in when hovering over the left edge of your screen.
* Pill-shaped tabs with active indicator, dynamic site favicons, title truncation, and individual close buttons.
* Dedicated Incognito / Private browsing section.

### 🗂️ 5. Minimalist Site Permissions & Security Center
* Centered popover dialog accessed via the lock icon in the search bar.
* Granular control of web permissions per site:
  * 📷 Camera and Video Capture
  * 🎤 Microphone & Audio (ideal for ChatGPT voice mode and web conferencing)
  * 🔔 Desktop Notifications
  * 📍 Geolocation
  * 💾 Local Storage & IndexedDB
* **One-Click Data Wipe**: Purge cookies and site storage for a specific domain without touching other open tabs or sessions.

### ⏱️ 6. New Tab & Customization
* Integrated clock with toggle between **12-hour (with AM/PM indicator)** and **24-hour** formats.
* Instant autofocus on search bar: open a tab and start typing immediately without reaching for the mouse.
* **Bilingual Support**: Instant switching between English and Spanish.

---

## 🛠️ Version Management & Build Metadata

### 🏷️ Changing the Application Version
Nika Browser provides a dedicated configuration section in compilation code to edit the version centrally:

1. Open [build.rs](build.rs):
   ```rust
   /// ============================================================================
   /// 🏷️ NIKA BROWSER BUILD VERSION CONFIGURATION
   /// Edit the version here to compile new binaries and releases:
   /// ============================================================================
   pub const APP_VERSION: &str = "0.1.0-alpha.1";
   ```
2. Update the `APP_VERSION` string (e.g., `"0.1.0-alpha.2"` or `"0.2.0"`).
3. Update `version` in [Cargo.toml](Cargo.toml):
   ```toml
   [package]
   version = "0.1.0-alpha.1"
   ```

### 📅 Embedded Build Metadata in Binary
During compilation, `build.rs` automatically embeds:
* Application version.
* Exact compilation date, time, and timezone.
* Git commit hash and build target (`x86_64-unknown-linux-gnu`).

To verify build metadata from terminal:
```bash
nika-browser --version
```
Example output:
```text
Nika Browser v0.1.0-alpha.1
Compilado el: 2026-10-31 20:35:18 -0500
Target: x86_64-unknown-linux-gnu (git: ddf4e4b)
Licencia: GNU GPL v3.0
```

---

## 💻 Building from Source

### Prerequisites (Arch Linux / Manjaro)
```bash
sudo pacman -S base-devel rust cargo webkitgtk-6.0 libadwaita gtk4 gstreamer gst-plugins-base gst-plugins-good gst-plugins-bad gst-libav
```

### Prerequisites (Ubuntu 24.04+ / Debian 13+)
```bash
sudo apt update
sudo apt install build-essential cargo libwebkitgtk-6.0-dev libadwaita-1-dev libgtk-4-dev
```

### Prerequisites (Fedora 40+)
```bash
sudo dnf install gcc cargo libwebkitgtk60-devel libadwaita-devel gtk4-devel
```

### Build & Run
```bash
# Clone the repository
git clone https://github.com/MrDemonc/Nika-Browser.git
cd Nika-Browser

# Automated compile & local install (recommended):
./install-local.sh

# Or compile in release mode manually:
cargo build --release
./target/release/nika-browser
```

### Script options:
* `./install-local.sh` : Compila en modo release e instala el binario, icono, recursos y acceso de escritorio.
* `./install-local.sh --run` : Compila, instala e inicia el navegador automáticamente.
* `./install-local.sh --debug` : Compilación rápida en modo debug para desarrollo.

---

## ⌨️ Keyboard Shortcuts

| Shortcut | Description |
| :--- | :--- |
| <kbd>Ctrl</kbd> + <kbd>T</kbd> | Open a new tab |
| <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>N</kbd> | Open private / incognito tab |
| <kbd>Ctrl</kbd> + <kbd>W</kbd> | Close active tab |
| <kbd>Ctrl</kbd> + <kbd>L</kbd> | Focus address bar (Omnibox) |
| <kbd>Ctrl</kbd> + <kbd>B</kbd> | Toggle / dock tabs sidebar |
| <kbd>Ctrl</kbd> + <kbd>R</kbd> / <kbd>F5</kbd> | Reload page |
| <kbd>Ctrl</kbd> + <kbd>J</kbd> | Toggle downloads panel |
| <kbd>Ctrl</kbd> + <kbd>F</kbd> | Open in-page search bar |
| <kbd>Ctrl</kbd> + <kbd>,</kbd> | Open Settings tab (`nika://settings`) |
| <kbd>Alt</kbd> + <kbd>←</kbd> | Navigate back in history |
| <kbd>Alt</kbd> + <kbd>→</kbd> | Navigate forward in history |
| <kbd>Ctrl</kbd> + <kbd>+</kbd> / <kbd>-</kbd> / <kbd>0</kbd> | Zoom in, zoom out, reset zoom |

---

## 📄 License

This project is licensed under the **GNU General Public License v3.0** ([GNU GPL v3.0](LICENSE)).  
Created and maintained by **[MrDemonc](https://github.com/MrDemonc)**.
