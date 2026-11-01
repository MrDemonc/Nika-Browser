#!/usr/bin/env bash
# ==============================================================================
# 🌸 Nika Browser Installer Script
# Instalador automático para Linux (x86_64)
# GitHub: https://github.com/MrDemonc/Nika-Browser
# ==============================================================================

set -e

# Colores para la terminal
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
echo "  │        Navegador web moderno, privado y rápido         │"
echo "  └────────────────────────────────────────────────────────┘"
echo -e "${NC}"

# 1. Comprobar sistema operativo
OS="$(uname -s)"
if [ "$OS" != "Linux" ]; then
    echo -e "${RED}❌ Error: Nika Browser está optimizado exclusivamente para sistemas Linux.${NC}"
    exit 1
fi

# 2. Comprobar arquitectura
ARCH="$(uname -m)"
if [ "$ARCH" != "x86_64" ]; then
    echo -e "${YELLOW}⚠️ Advertencia: Las versiones precompiladas oficiales son para arquitectura x86_64 (tu sistema es $ARCH).${NC}"
    echo -e "Puedes compilar Nika Browser directamente desde el código fuente ejecutando:"
    echo -e "  git clone https://github.com/${REPO}.git"
    echo -e "  cd Nika-Browser && cargo build --release"
    exit 1
fi

# 3. Comprobar herramientas básicas requeridas
for cmd in curl tar; do
    if ! command -v "$cmd" >/dev/null 2>&1; then
        echo -e "${RED}❌ Error: '$cmd' no está instalado. Por favor instálalo para continuar.${NC}"
        exit 1
    fi
done

# 4. Determinar rutas de instalación en el directorio de usuario
BIN_DIR="${HOME}/.local/bin"
APPS_DIR="${HOME}/.local/share/applications"
ICONS_DIR="${HOME}/.local/share/icons/hicolor/512x512/apps"

mkdir -p "$BIN_DIR"
mkdir -p "$APPS_DIR"
mkdir -p "$ICONS_DIR"

# 5. Obtener información de la versión a descargar
echo -e "${CYAN}🔍 Buscando última versión disponible en GitHub (${REPO})...${NC}"
TAG=""
if command -v gh >/dev/null 2>&1 && gh auth status >/dev/null 2>&1; then
    TAG="$(gh release view -R "$REPO" --json tagName -q .tagName 2>/dev/null || true)"
fi

if [ -z "$TAG" ]; then
    # Consultar API pública de GitHub o usar tag por defecto
    TAG="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases" 2>/dev/null | grep -m1 '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/' || true)"
fi

if [ -z "$TAG" ]; then
    TAG="$DEFAULT_TAG"
fi

echo -e "${GREEN}📦 Descargando Nika Browser (${TAG})...${NC}"

# URL del binario de release
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${TAG}/nika-browser"
TEMP_FILE="$(mktemp)"

if ! curl -fL --progress-bar "$DOWNLOAD_URL" -o "$TEMP_FILE"; then
    echo -e "${YELLOW}⚠️ No se pudo descargar el binario desde ${DOWNLOAD_URL}.${NC}"
    echo -e "Intentando descargar la versión alpha por defecto (${DEFAULT_TAG})..."
    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${DEFAULT_TAG}/nika-browser"
    if ! curl -fL --progress-bar "$DOWNLOAD_URL" -o "$TEMP_FILE"; then
        echo -e "${RED}❌ Error: No se pudo descargar el archivo de lanzamiento.${NC}"
        rm -f "$TEMP_FILE"
        exit 1
    fi
fi

# 6. Instalar binario ejecutable
TARGET_BIN="${BIN_DIR}/nika-browser"
mv "$TEMP_FILE" "$TARGET_BIN"
chmod +x "$TARGET_BIN"
echo -e "${GREEN}✓ Binario instalado en: ${TARGET_BIN}${NC}"

# 7. Descargar e instalar icono de alta resolución
ICON_URL="https://raw.githubusercontent.com/${REPO}/main/nika_browser.png"
TARGET_ICON="${ICONS_DIR}/nika-browser.png"
echo -e "${CYAN}🖼️  Instalando icono oficial...${NC}"
curl -fsSL "$ICON_URL" -o "$TARGET_ICON" 2>/dev/null || true
echo -e "${GREEN}✓ Icono instalado en: ${TARGET_ICON}${NC}"

# 8. Crear archivo de acceso directo de escritorio (.desktop)
DESKTOP_FILE="${APPS_DIR}/nika-browser.desktop"
echo -e "${CYAN}🖥️  Configurando lanzador de aplicaciones de escritorio...${NC}"

cat <<EOF > "$DESKTOP_FILE"
[Desktop Entry]
Name=Nika Browser
GenericName=Web Browser
Comment=Navegador web moderno, minimalista, privado y rápido en Rust
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

# 9. Actualizar base de datos de escritorio e iconos
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$APPS_DIR" 2>/dev/null || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t "${HOME}/.local/share/icons/hicolor" 2>/dev/null || true
fi

# 10. Comprobar si ~/.local/bin está en el PATH
PATH_NOTE=""
if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
    PATH_NOTE="${YELLOW}💡 Tip: Añade ~/.local/bin a tu PATH para ejecutarlo directamente escribiendo 'nika-browser':\n   echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.bashrc (o ~/.zshrc)${NC}\n"
fi

echo -e "\n${GREEN}${BOLD}🎉 ¡Nika Browser se ha instalado exitosamente!${NC}\n"
echo -e "  • Versión instalada : ${BOLD}${TAG}${NC}"
echo -e "  • Ejecutable        : ${CYAN}${TARGET_BIN}${NC}"
echo -e "  • Acceso directo    : ${CYAN}${DESKTOP_FILE}${NC}"
echo -e ""
if [ -n "$PATH_NOTE" ]; then
    echo -e "$PATH_NOTE"
fi
echo -e "Para iniciar Nika Browser:"
echo -e "  ${BOLD}nika-browser${NC}  o búscalo en tu menú de aplicaciones (Rofi, Wofi, GNOME, KDE).\n"
