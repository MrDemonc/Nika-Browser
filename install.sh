#!/usr/bin/env bash
# ==============================================================================
# 🌸 Nika Browser Installer Script
# Fast and automated installer for Linux (x86_64)
# GitHub: https://github.com/MrDemonc/Nika-Browser
# ==============================================================================

set -e

# Terminal formatting colors
BOLD='\033[1m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

REPO="MrDemonc/Nika-Browser"
DEFAULT_TAG="v0.1.0-alpha.1"

echo -e "${PURPLE}${BOLD}"
echo "  ┌────────────────────────────────────────────────────────┐"
echo "  │               🌸 Nika Browser Installer               │"
echo "  │        Modern, private and fast Linux browser          │"
echo "  └────────────────────────────────────────────────────────┘"
echo -e "${NC}"

# 1. Verify operating system
OS="$(uname -s)"
if [ "$OS" != "Linux" ]; then
    echo -e "${RED}❌ Error: Nika Browser is built exclusively for Linux systems.${NC}"
    exit 1
fi

# 2. Verify architecture
ARCH="$(uname -m)"
if [ "$ARCH" != "x86_64" ]; then
    echo -e "${YELLOW}⚠️ Warning: Official precompiled release binaries target x86_64 (your machine is $ARCH).${NC}"
    echo -e "You can compile Nika Browser from source by running:"
    echo -e "  git clone https://github.com/${REPO}.git"
    echo -e "  cd Nika-Browser && cargo build --release"
    exit 1
fi

# 3. Check for required command-line tools
for cmd in curl tar; do
    if ! command -v "$cmd" >/dev/null 2>&1; then
        echo -e "${RED}❌ Error: Required tool '$cmd' is not installed. Please install it to continue.${NC}"
        exit 1
    fi
done

# 4. Set up user target directories
BIN_DIR="${HOME}/.local/bin"
APPS_DIR="${HOME}/.local/share/applications"
ICONS_DIR="${HOME}/.local/share/icons/hicolor/512x512/apps"

mkdir -p "$BIN_DIR"
mkdir -p "$APPS_DIR"
mkdir -p "$ICONS_DIR"

# 5. Fetch latest release tag from GitHub
echo -e "${CYAN}🔍 Fetching latest release information from GitHub (${REPO})...${NC}"
TAG=""
if command -v gh >/dev/null 2>&1 && gh auth status >/dev/null 2>&1; then
    TAG="$(gh release view -R "$REPO" --json tagName -q .tagName 2>/dev/null || true)"
fi

if [ -z "$TAG" ]; then
    TAG="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases" 2>/dev/null | grep -m1 '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/' || true)"
fi

if [ -z "$TAG" ]; then
    TAG="$DEFAULT_TAG"
fi

echo -e "${GREEN}📦 Downloading Nika Browser (${TAG})...${NC}"

# URL of the release binary
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${TAG}/nika-browser"
TEMP_FILE="$(mktemp)"

if ! curl -fL --progress-bar "$DOWNLOAD_URL" -o "$TEMP_FILE"; then
    echo -e "${YELLOW}⚠️ Could not download asset from ${DOWNLOAD_URL}.${NC}"
    echo -e "Attempting fallback to default alpha release (${DEFAULT_TAG})..."
    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${DEFAULT_TAG}/nika-browser"
    if ! curl -fL --progress-bar "$DOWNLOAD_URL" -o "$TEMP_FILE"; then
        echo -e "${RED}❌ Error: Failed to download release asset from GitHub.${NC}"
        rm -f "$TEMP_FILE"
        exit 1
    fi
fi

# 6. Install executable binary
TARGET_BIN="${BIN_DIR}/nika-browser"
mv "$TEMP_FILE" "$TARGET_BIN"
chmod +x "$TARGET_BIN"
echo -e "${GREEN}✓ Binary installed to: ${TARGET_BIN}${NC}"

# 7. Download and install high-resolution application icon
ICON_URL="https://raw.githubusercontent.com/${REPO}/main/nika_browser.png"
TARGET_ICON="${ICONS_DIR}/nika-browser.png"
echo -e "${CYAN}🖼️  Installing official desktop icon...${NC}"
curl -fsSL "$ICON_URL" -o "$TARGET_ICON" 2>/dev/null || true
echo -e "${GREEN}✓ Icon installed to: ${TARGET_ICON}${NC}"

# 7b. Install bundled resources (icons, styles, privacy filter rules)
RESOURCES_DIR="${HOME}/.local/share/nika-browser/resources"
echo -e "${CYAN}📦 Installing resources (icons, styles, privacy rules)...${NC}"
RES_TMP="$(mktemp -d)"
if curl -fsSL "https://github.com/${REPO}/archive/refs/tags/${TAG}.tar.gz" -o "${RES_TMP}/src.tar.gz" 2>/dev/null \
    && tar -xzf "${RES_TMP}/src.tar.gz" -C "$RES_TMP" 2>/dev/null; then
    SRC_ROOT="$(find "$RES_TMP" -maxdepth 1 -mindepth 1 -type d -name 'Nika-Browser-*' | head -n1)"
    if [ -n "$SRC_ROOT" ] && [ -d "${SRC_ROOT}/resources" ]; then
        mkdir -p "$RESOURCES_DIR"
        cp -R "${SRC_ROOT}/resources/." "$RESOURCES_DIR/"
        echo -e "${GREEN}✓ Resources installed to: ${RESOURCES_DIR}${NC}"
    else
        echo -e "${YELLOW}⚠️ Could not locate resources in the release archive; using built-in defaults.${NC}"
    fi
else
    echo -e "${YELLOW}⚠️ Could not download resource bundle; using built-in defaults.${NC}"
fi
rm -rf "$RES_TMP"

# 8. Create desktop launcher entry (.desktop file)
DESKTOP_FILE="${APPS_DIR}/nika-browser.desktop"
echo -e "${CYAN}🖥️  Configuring desktop application launcher...${NC}"

cat <<EOF > "$DESKTOP_FILE"
[Desktop Entry]
Name=Nika Browser
GenericName=Web Browser
Comment=Modern, minimalist, fast and privacy-focused Linux browser in Rust
Exec=${TARGET_BIN} %U
Icon=nika-browser
Terminal=false
Type=Application
Categories=Network;WebBrowser;
MimeType=text/html;text/xml;application/xhtml+xml;x-scheme-handler/http;x-scheme-handler/https;x-scheme-handler/nika;
StartupWMClass=org.nika.browser
Keywords=web;browser;internet;privacy;nika;
EOF

chmod +x "$DESKTOP_FILE"

# 9. Update desktop application and icon databases
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$APPS_DIR" 2>/dev/null || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t "${HOME}/.local/share/icons/hicolor" 2>/dev/null || true
fi

# 10. Check whether ~/.local/bin is present in $PATH
PATH_NOTE=""
if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
    PATH_NOTE="${YELLOW}💡 Note: Add ~/.local/bin to your PATH to run 'nika-browser' from any terminal:\n   echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.bashrc (or ~/.zshrc)${NC}\n"
fi

echo -e "\n${GREEN}${BOLD}🎉 Nika Browser installed successfully!${NC}\n"
echo -e "  • Installed version : ${BOLD}${TAG}${NC}"
echo -e "  • Executable binary : ${CYAN}${TARGET_BIN}${NC}"
echo -e "  • Desktop launcher  : ${CYAN}${DESKTOP_FILE}${NC}"
echo -e ""
if [ -n "$PATH_NOTE" ]; then
    echo -e "$PATH_NOTE"
fi
echo -e "To launch Nika Browser:"
echo -e "  Run ${BOLD}nika-browser${NC} or launch it from your application menu (Rofi, Wofi, GNOME, KDE).\n"
