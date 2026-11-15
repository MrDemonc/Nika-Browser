#!/usr/bin/env bash
# ==============================================================================
# 🌸 Nika Browser - Uninstaller Script
# Desinstalador oficial y limpio para Linux
# GitHub: https://github.com/MrDemonc/Nika-Browser
# ==============================================================================

set -euo pipefail

# Colores para la salida en terminal
BOLD='\033[1m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m' # Sin color

AUTO_CONFIRM=false
PURGE_DATA=false
KEEP_DATA=false

# Procesar argumentos de línea de comandos
while [[ $# -gt 0 ]]; do
    case "$1" in
        -y|--yes)
            AUTO_CONFIRM=true
            shift
            ;;
        -p|--purge|--all)
            PURGE_DATA=true
            shift
            ;;
        -c|--keep-data)
            KEEP_DATA=true
            shift
            ;;
        -h|--help)
            echo -e "${BOLD}Uso:${NC} $0 [OPCIONES]"
            echo ""
            echo "Desinstala Nika Browser de tu sistema Linux de forma limpia."
            echo ""
            echo -e "${BOLD}Opciones:${NC}"
            echo "  -y, --yes          Modo no interactivo; asume 'sí' a la confirmación de desinstalación"
            echo "  -p, --purge, --all Elimina también los datos de usuario (configuración, contraseñas, historial y caché)"
            echo "  -c, --keep-data    Conserva explícitamente los datos de usuario y configuración sin preguntar"
            echo "  -h, --help         Muestra esta ayuda y termina"
            echo ""
            echo -e "${BOLD}Ejemplos:${NC}"
            echo "  ./uninstall.sh                # Desinstalación interactiva"
            echo "  ./uninstall.sh --purge        # Desinstala y borra también todos los datos y cachés"
            echo "  ./uninstall.sh -y --keep-data # Desinstalación automática conservando tus datos"
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
echo "  │              🗑️  Nika Browser - Uninstaller             │"
echo "  │           Desinstalador oficial para Linux             │"
echo "  └────────────────────────────────────────────────────────┘"
echo -e "${NC}"

# Función para leer entrada interactiva de forma segura incluso en tuberías (pipes)
prompt_user() {
    local prompt_msg="$1"
    local default_ans="$2"
    local user_ans=""

    if [ -t 0 ]; then
        read -r -p "$prompt_msg " user_ans
    elif [ -r /dev/tty ]; then
        read -r -p "$prompt_msg " user_ans < /dev/tty
    else
        user_ans="$default_ans"
    fi

    echo "${user_ans:-$default_ans}"
}

# 1. Comprobar si Nika Browser está actualmente en ejecución
if pgrep -x "nika-browser" >/dev/null 2>&1; then
    echo -e "${YELLOW}⚠️ Nika Browser se encuentra actualmente en ejecución.${NC}"
    if [ "$AUTO_CONFIRM" = true ]; then
        echo -e "${CYAN}Cerrando procesos de Nika Browser...${NC}"
        pkill -x "nika-browser" 2>/dev/null || true
        sleep 1
    else
        ans=$(prompt_user "¿Deseas cerrar Nika Browser ahora para continuar con la desinstalación? [S/n]:" "s")
        if [[ "$ans" =~ ^[SsYy]$ ]]; then
            echo -e "${CYAN}Cerrando procesos de Nika Browser...${NC}"
            pkill -x "nika-browser" 2>/dev/null || true
            sleep 1
        else
            echo -e "${RED}❌ Desinstalación cancelada. Cierra Nika Browser e inténtalo nuevamente.${NC}"
            exit 1
        fi
    fi
fi

# 2. Confirmación previa de desinstalación
if [ "$AUTO_CONFIRM" = false ]; then
    confirm=$(prompt_user "¿Estás seguro de que deseas desinstalar Nika Browser? [s/N]:" "n")
    if [[ ! "$confirm" =~ ^[SsYy]$ ]]; then
        echo -e "${YELLOW}Desinstalación cancelada por el usuario.${NC}"
        exit 0
    fi
fi

# 3. Preguntar sobre la eliminación de datos personales si no se especificó por bandera
if [ "$PURGE_DATA" = false ] && [ "$KEEP_DATA" = false ]; then
    if [ "$AUTO_CONFIRM" = false ]; then
        echo ""
        echo -e "${CYAN}Tus datos personales incluyen configuración, contraseñas guardadas, pestañas y caché.${NC}"
        purge_ans=$(prompt_user "¿Deseas ELIMINAR también todos tus datos personales y configuraciones? [s/N]:" "n")
        if [[ "$purge_ans" =~ ^[SsYy]$ ]]; then
            PURGE_DATA=true
        fi
    fi
fi

echo ""
echo -e "${CYAN}🚀 Iniciando desinstalación de Nika Browser...${NC}"

# Rutas estándar del usuario
BIN_USER="${HOME}/.local/bin/nika-browser"
DESKTOP_USER="${HOME}/.local/share/applications/nika-browser.desktop"
ICON_USER="${HOME}/.local/share/icons/hicolor/512x512/apps/nika-browser.png"
SHARE_USER="${HOME}/.local/share/nika-browser"
CONFIG_USER="${HOME}/.config/nika-browser"
CACHE_USER="${HOME}/.cache/nika-browser"

REMOVED_ANY=false

# 4. Eliminar binario ejecutable
if [ -f "$BIN_USER" ]; then
    echo -e "  Eliminando binario: ${BIN_USER}"
    rm -f "$BIN_USER"
    echo -e "  ${GREEN}✓ Binario eliminado.${NC}"
    REMOVED_ANY=true
fi

# Comprobar también binarios en /usr/local/bin si son escribibles por el usuario
if [ -w "/usr/local/bin/nika-browser" ]; then
    echo -e "  Eliminando binario del sistema: /usr/local/bin/nika-browser"
    rm -f "/usr/local/bin/nika-browser"
    REMOVED_ANY=true
fi

# 5. Eliminar lanzador de escritorio (.desktop)
if [ -f "$DESKTOP_USER" ]; then
    echo -e "  Eliminando lanzador de escritorio: ${DESKTOP_USER}"
    rm -f "$DESKTOP_USER"
    echo -e "  ${GREEN}✓ Lanzador de escritorio eliminado.${NC}"
    REMOVED_ANY=true
fi

if [ -w "/usr/local/share/applications/nika-browser.desktop" ]; then
    rm -f "/usr/local/share/applications/nika-browser.desktop"
    REMOVED_ANY=true
fi

# 6. Eliminar iconos de aplicación
if [ -f "$ICON_USER" ]; then
    echo -e "  Eliminando icono de aplicación: ${ICON_USER}"
    rm -f "$ICON_USER"
    echo -e "  ${GREEN}✓ Icono eliminado.${NC}"
    REMOVED_ANY=true
fi

# Limpiar posibles iconos en otras resoluciones hicolor del usuario
for icon_path in "${HOME}/.local/share/icons/hicolor"/*/apps/nika-browser.png; do
    if [ -f "$icon_path" ]; then
        rm -f "$icon_path"
        REMOVED_ANY=true
    fi
done

# 7. Eliminar recursos del programa en ~/.local/share/nika-browser/resources
if [ -d "${SHARE_USER}/resources" ]; then
    echo -e "  Eliminando recursos del programa: ${SHARE_USER}/resources"
    rm -rf "${SHARE_USER}/resources"
    echo -e "  ${GREEN}✓ Recursos del programa eliminados.${NC}"
    REMOVED_ANY=true
fi

# 8. Tratar datos personales y caché
if [ "$PURGE_DATA" = true ]; then
    echo -e "${YELLOW}🧹 Eliminando datos personales, configuración y cachés...${NC}"

    if [ -d "$CONFIG_USER" ]; then
        echo -e "  Eliminando configuración: ${CONFIG_USER}"
        rm -rf "$CONFIG_USER"
        echo -e "  ${GREEN}✓ Configuración y contraseñas eliminadas.${NC}"
    fi

    if [ -d "$CACHE_USER" ]; then
        echo -e "  Eliminando caché: ${CACHE_USER}"
        rm -rf "$CACHE_USER"
        echo -e "  ${GREEN}✓ Caché eliminada.${NC}"
    fi

    if [ -d "$SHARE_USER" ]; then
        echo -e "  Eliminando datos de navegación (cookies, almacenamiento): ${SHARE_USER}"
        rm -rf "$SHARE_USER"
        echo -e "  ${GREEN}✓ Datos de navegación eliminados.${NC}"
    fi

    rm -f /tmp/nika_cookies.sqlite* 2>/dev/null || true
    echo -e "  ${GREEN}✓ Todos los datos personales han sido purgados por completo.${NC}"
else
    # Si no se purgan datos, comprobar si quedó vacío ~/.local/share/nika-browser
    if [ -d "$SHARE_USER" ] && [ -z "$(ls -A "$SHARE_USER" 2>/dev/null)" ]; then
        rmdir "$SHARE_USER" 2>/dev/null || true
    fi

    if [ -d "$CONFIG_USER" ]; then
        echo -e "${CYAN}ℹ️  Se han conservado tus datos y configuraciones en:${NC}"
        echo -e "    ${CONFIG_USER}"
        echo -e "   (Si deseas borrarlos más adelante, usa: ./uninstall.sh --purge)"
    fi
fi

# 9. Actualizar bases de datos de escritorio y caché de iconos
APPS_DIR="${HOME}/.local/share/applications"
ICONS_BASE="${HOME}/.local/share/icons/hicolor"

if command -v update-desktop-database >/dev/null 2>&1 && [ -d "$APPS_DIR" ]; then
    update-desktop-database "$APPS_DIR" 2>/dev/null || true
fi

if command -v gtk-update-icon-cache >/dev/null 2>&1 && [ -d "$ICONS_BASE" ]; then
    gtk-update-icon-cache -f -t "$ICONS_BASE" 2>/dev/null || true
fi

echo ""
if [ "$REMOVED_ANY" = true ] || [ "$PURGE_DATA" = true ]; then
    echo -e "${GREEN}${BOLD}✨ ¡Nika Browser ha sido desinstalado exitosamente!${NC}"
else
    echo -e "${YELLOW}⚠️ No se encontraron archivos de instalación de Nika Browser en las rutas estándar.${NC}"
fi
echo ""
