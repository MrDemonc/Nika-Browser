#!/usr/bin/env bash
# ==============================================================================
# 🌸 Nika Browser - Local Build & Install Script
# Compiles and installs Nika Browser directly from local source code.
# ==============================================================================

set -euo pipefail

# Terminal formatting colors
BOLD='\033[1m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m'

# Resolve repository root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}❌ Error: No se encontró Cargo.toml. Ejecuta este script desde el directorio del proyecto.${NC}"
    exit 1
fi

BUILD_MODE="release"
RUN_AFTER=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        -d|--debug)
            BUILD_MODE="debug"
            shift
            ;;
        -r|--run)
            RUN_AFTER=true
            shift
            ;;
        -h|--help)
            echo "Uso: $0 [OPCIONES]"
            echo ""
            echo "Opciones:"
            echo "  -d, --debug    Compilar en modo debug (más rápido, sin optimizaciones de release)"
            echo "  -r, --run      Ejecutar Nika Browser inmediatamente después de instalar"
            echo "  -h, --help     Mostrar esta ayuda"
            exit 0
            ;;
        *)
            echo -e "${YELLOW}⚠️ Opción desconocida: $1${NC}"
            shift
            ;;
    esac
done

echo -e "${PURPLE}${BOLD}"
echo "  ┌────────────────────────────────────────────────────────┐"
echo "  │        🌸 Nika Browser - Instalador Local (Source)     │"
echo "  │        Compilación e instalación local optimizada      │"
echo "  └────────────────────────────────────────────────────────┘"
echo -e "${NC}"

# 1. Comprobar que cargo esté instalado
if ! command -v cargo >/dev/null 2>&1; then
    echo -e "${RED}❌ Error: Rust/Cargo no está instalado o no se encuentra en el PATH.${NC}"
    echo -e "Instálalo mediante: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# 2. Configurar directorios de destino en el espacio del usuario (~/.local)
BIN_DIR="${HOME}/.local/bin"
APPS_DIR="${HOME}/.local/share/applications"
ICONS_DIR="${HOME}/.local/share/icons/hicolor/512x512/apps"
DATA_DIR="${HOME}/.local/share/nika-browser"
RESOURCES_DIR="${DATA_DIR}/resources"

mkdir -p "$BIN_DIR"
mkdir -p "$APPS_DIR"
mkdir -p "$ICONS_DIR"
mkdir -p "$RESOURCES_DIR"

# 3. Compilación del proyecto
if [ "$BUILD_MODE" = "release" ]; then
    echo -e "${CYAN}🔨 Compilando Nika Browser en modo RELEASE (máximo rendimiento y optimización)...${NC}"
    cargo build --release
    SOURCE_BIN="target/release/nika-browser"
else
    echo -e "${CYAN}🔨 Compilando Nika Browser en modo DEBUG (desarrollo rápido)...${NC}"
    cargo build
    SOURCE_BIN="target/debug/nika-browser"
fi

if [ ! -f "$SOURCE_BIN" ]; then
    echo -e "${RED}❌ Error: El binario no fue generado en $SOURCE_BIN.${NC}"
    exit 1
fi

# 4. Instalación del binario en ~/.local/bin
TARGET_BIN="${BIN_DIR}/nika-browser"
echo -e "${CYAN}📦 Instalando binario en ${TARGET_BIN}...${NC}"
# Usar install para reemplazar el inodo de forma atómica y segura incluso si la app está en ejecución
install -m 755 "$SOURCE_BIN" "$TARGET_BIN"
echo -e "${GREEN}✓ Binario instalado correctamente.${NC}"

# 5. Instalación del icono oficial de escritorio
if [ -f "nika_browser.png" ]; then
    echo -e "${CYAN}🖼️  Instalando icono en ${ICONS_DIR}...${NC}"
    install -m 644 "nika_browser.png" "${ICONS_DIR}/nika-browser.png"
    echo -e "${GREEN}✓ Icono instalado correctamente.${NC}"
fi

# 6. Sincronización de recursos locales (estilos, iconos, home, settings, reglas de privacidad)
if [ -d "resources" ]; then
    echo -e "${CYAN}📂 Sincronizando recursos locales a ${RESOURCES_DIR}...${NC}"
    cp -R resources/. "$RESOURCES_DIR/"
    echo -e "${GREEN}✓ Recursos sincronizados correctamente.${NC}"
fi

# 7. Creación/actualización del lanzador de escritorio (.desktop)
DESKTOP_FILE="${APPS_DIR}/nika-browser.desktop"
echo -e "${CYAN}🖥️  Actualizando lanzador de escritorio en ${DESKTOP_FILE}...${NC}"

cat <<EOF > "$DESKTOP_FILE"
[Desktop Entry]
Name=Nika Browser
GenericName=Web Browser
GenericName[es]=Navegador Web
Comment=Modern, minimalist, fast and privacy-focused Linux browser in Rust
Comment[es]=Navegador web moderno, minimalista y privado para Linux
Exec=${TARGET_BIN} %U
Icon=nika-browser
Terminal=false
Type=Application
Categories=Network;WebBrowser;
MimeType=text/html;text/xml;application/xhtml+xml;x-scheme-handler/http;x-scheme-handler/https;x-scheme-handler/nika;
StartupWMClass=org.nika.browser
Keywords=web;browser;internet;privacy;nika;navegador;
EOF

chmod 644 "$DESKTOP_FILE"
echo -e "${GREEN}✓ Lanzador de escritorio configurado.${NC}"

# 8. Actualizar base de datos de escritorio y caché de iconos si están disponibles
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$APPS_DIR" 2>/dev/null || true
fi

if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t "${HOME}/.local/share/icons/hicolor" 2>/dev/null || true
fi

# 9. Comprobar si ~/.local/bin está en el PATH
if [[ ":$PATH:" != *":${BIN_DIR}:"* ]]; then
    echo -e "${YELLOW}⚠️ Aviso: ${BIN_DIR} no parece estar en tu variable PATH actual.${NC}"
    echo -e "Para ejecutar 'nika-browser' directamente desde cualquier terminal, añade a tu ~/.bashrc o ~/.zshrc:"
    echo -e "  export PATH=\"\$HOME/.local/bin:\$PATH\""
fi

echo -e "\n${GREEN}${BOLD}✨ ¡Instalación local completada con éxito!${NC}"
echo -e "Puedes abrir Nika Browser desde tu lanzador de aplicaciones o escribiendo en la terminal:"
echo -e "  ${BOLD}nika-browser${NC}\n"

# 10. Ejecutar si se solicitó con --run
if [ "$RUN_AFTER" = true ]; then
    echo -e "${CYAN}🚀 Iniciando Nika Browser...${NC}"
    "$TARGET_BIN" &
fi
