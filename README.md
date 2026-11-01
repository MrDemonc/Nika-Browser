<p align="center">
  <img src="nika_browser.png" width="128" height="128" alt="Nika Browser Logo" style="border-radius: 28px;" />
</p>

<h1 align="center">🌸 Nika Browser</h1>

<p align="center">
  <strong>Navegador web moderno, ultra-privado, minimalista y rápido desarrollado en Rust con GTK4, Libadwaita y WebKitGTK 6.0 para Linux.</strong>
</p>

<p align="center">
  <a href="https://github.com/MrDemonc/Nika-Browser/releases"><img src="https://img.shields.io/github/v/release/MrDemonc/Nika-Browser?color=c084fc&label=Release&style=flat-square" alt="Release"></a>
  <img src="https://img.shields.io/badge/Rust-2024%20Edition-orange?style=flat-square&logo=rust" alt="Rust 2024">
  <img src="https://img.shields.io/badge/GTK4%20%2B%20Libadwaita-1.6-blue?style=flat-square&logo=gnome" alt="GTK4 Libadwaita">
  <img src="https://img.shields.io/badge/WebKitGTK-6.0-green?style=flat-square&logo=webkit" alt="WebKitGTK 6">
  <img src="https://img.shields.io/badge/Platform-Linux%20x86__64-purple?style=flat-square&logo=linux" alt="Linux x86_64">
  <img src="https://img.shields.io/badge/License-MIT-lightgrey?style=flat-square" alt="License MIT">
</p>

---

## ⚡ Instalación Rápida (Linux x86_64)

Puedes instalar **Nika Browser** directamente con un solo comando en tu terminal mediante `curl`:

```bash
curl -fsSL https://raw.githubusercontent.com/MrDemonc/Nika-Browser/main/install.sh | bash
```

> **¿Qué hace este script?**
> - Descarga el binario optimizado de la última versión precompilada (`nika-browser`).
> - Lo instala en `~/.local/bin/nika-browser`.
> - Instala el icono oficial en alta resolución en `~/.local/share/icons/hicolor/512x512/apps/`.
> - Configura el lanzador de aplicaciones de escritorio (`.desktop`) para que aparezca inmediatamente en tu lanzador (Rofi, Wofi, GNOME, KDE, etc.).

---

## ✨ Características Principales

### 🛡️ 1. Privacidad y Seguridad por Diseño
* **Sin Telemetría Comercial**: 0% código de analíticas, perfiles de usuario o rastreo publicitario.
* **Escudo WebKit Nativo**: Bloqueo de anuncios y rastreadores invasivos (Google Analytics, Facebook Pixel, Hotjar, telemetría) compilado directamente en WebKit mediante reglas de filtrado de contenido.
* **Limpiador Automático de URLs**: Elimina parámetros espía (`utm_*`, `fbclid`, `gclid`, `igshid`, `yclid`, etc.) antes de que se envíen por red.
* **Eliminación Segura de Caché al Cerrar**: Opción para purgar automáticamente memoria caché, cookies temporales y datos WebKit cada vez que cierras el navegador.

### 🔒 2. Gestor Local de Contraseñas Cifrado (Bóveda Segura)
* **Cifrado Militar Local**: Protegido mediante **AES-256-GCM** y derivación de clave con **PBKDF2** (600,000 iteraciones con sal criptográfica de 16 bytes).
* **Contraseña Maestra**: Se configura durante el asistente inicial o desde Configuración. Tus credenciales se almacenan estrictamente en tu equipo en `~/.config/nika-browser/passwords_vault.json`, sin servidores externos ni nube.
* **Desbloqueo y Bloqueo en un Clic**: Interfaz integrada para consultar, copiar, añadir o eliminar cuentas y contraseñas.

### 🌐 3. DNS Seguro (DNS-over-HTTPS / DoH)
* **Cifrado de Consultas DNS**: Impide que tu proveedor de internet (ISP) o atacantes espíen los dominios que visitas.
* **Proveedores Preconfigurados con 1 Clic**:
  * 🛡️ **AdGuard DNS**: Bloqueo de publicidad a nivel de resolución de nombres.
  * ⚡ **Cloudflare DNS** (`1.1.1.1`): Máxima velocidad y privacidad.
  * 🔒 **Quad9 DNS** (`9.9.9.9`): Bloqueo de dominios maliciosos y malware.
  * 🌐 **Google Public DNS** (`8.8.8.8`).
  * ⚙️ **Endpoint DoH Personalizado**: Ingresa cualquier URL RFC 8484 HTTPS personalizada.

### 📑 4. Barra Lateral de Pestañas Estilo Arc (Vertical)
* **Dos Modos de Visualización**:
  * **Modo Fijado (Docked)**: Se acopla a la izquierda sin tapar la página web, separada por una línea divisoria nítida.
  * **Modo Auto-ocultar (Hover)**: Permanece oculta y se desliza suavemente al pasar el ratón por el borde izquierdo de la pantalla.
* Pestañas en cápsula minimalista con indicador activo, icono de sitio (favicon dinámico), título adaptable y botón de cierre individual.
* Soporte nativo para pestañas privadas / incógnito en sección separada.

### 🗂️ 5. Centro de Permisos y Seguridad Web
* Ventana modal compacta y centrada accesible desde el icono de candado en la barra de búsqueda.
* Gestión individual de permisos para el sitio actual:
  * 📷 Cámara y Captura de Video
  * 🎤 Micrófono y Audio (ideal para ChatGPT Voice, conferencias y apps web)
  * 🔔 Notificaciones de escritorio
  * 📍 Geolocalización
  * 💾 Almacenamiento Local y Base de Datos
* **Botón de Limpieza Rápida**: Elimina instantáneamente las cookies y almacenamiento de un sitio en concreto sin afectar al resto de tu navegación.

### ⏱️ 6. Nueva Pestaña y Personalización
* Reloj minimalista integrado con selector de formato de **12 horas (con indicador AM/PM)** o **24 horas**.
* Foco automático en el buscador: abre una pestaña y escribe directamente sin tener que hacer clic.
* Accesos rápidos a sitios populares.
* **Soporte Multilenguaje**: Español e Inglés con cambio en tiempo real.

---

## 🛠️ Compilación y Gestión de Versiones

### 🏷️ Cómo Modificar la Versión de la Aplicación
El proyecto cuenta con un apartado dedicado en el código de compilación para editar la versión de manera centralizada.

1. Abre el archivo de compilación [build.rs](build.rs):
   ```rust
   /// ============================================================================
   /// 🏷️ CONFIGURACIÓN DE VERSIÓN DE COMPILACIÓN DE NIKA BROWSER
   /// Modifica aquí la versión para compilar nuevos binarios y lanzamientos:
   /// ============================================================================
   pub const APP_VERSION: &str = "0.1.0-alpha.1";
   ```
2. Modifica el valor de `APP_VERSION` (por ejemplo `"0.1.0-alpha.2"` o `"0.2.0"`).
3. Actualiza la versión en [Cargo.toml](Cargo.toml):
   ```toml
   [package]
   version = "0.1.0-alpha.1"
   ```

### 📅 Metadatos de Compilación en el Binario
Cada vez que compilas, el script `build.rs` inyecta automáticamente:
* La versión de la aplicación.
* La fecha, hora y zona horaria exacta en la que se generó el binario.
* El hash de commit de git y el target de compilación (`x86_64-unknown-linux-gnu`).

Para verificar los metadatos desde la terminal:
```bash
nika-browser --version
```
Salida de ejemplo:
```text
Nika Browser v0.1.0-alpha.1
Compilado el: 2026-10-31 20:31:39 -0500
Target: x86_64-unknown-linux-gnu (git: 0cd57ff)
Licencia: MIT
```

---

## 💻 Compilación Manual desde Código Fuente

### Requisitos previos (Arch Linux / Manjaro)
```bash
sudo pacman -S base-devel rust cargo webkitgtk-6.0 libadwaita gtk4 gstreamer gst-plugins-base gst-plugins-good gst-plugins-bad gst-libav
```

### Requisitos previos (Ubuntu 24.04+ / Debian 13+)
```bash
sudo apt update
sudo apt install build-essential cargo libwebkitgtk-6.0-dev libadwaita-1-dev libgtk-4-dev
```

### Compilar y ejecutar
```bash
# Clonar el repositorio
git clone https://github.com/MrDemonc/Nika-Browser.git
cd Nika-Browser

# Compilar en modo optimizado Release
cargo build --release

# Iniciar el navegador
./target/release/nika-browser
```

O usando el script de ejecución incluido:
```bash
./run.sh
```

---

## ⌨️ Atajos de Teclado

| Atajo | Acción |
| :--- | :--- |
| <kbd>Ctrl</kbd> + <kbd>T</kbd> | Abrir nueva pestaña |
| <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>N</kbd> | Abrir pestaña de incógnito / privada |
| <kbd>Ctrl</kbd> + <kbd>W</kbd> | Cerrar pestaña activa |
| <kbd>Ctrl</kbd> + <kbd>L</kbd> | Enfocar barra de direcciones (Omnibox) |
| <kbd>Ctrl</kbd> + <kbd>B</kbd> | Alternar / fijar barra lateral de pestañas |
| <kbd>Ctrl</kbd> + <kbd>R</kbd> / <kbd>F5</kbd> | Recargar página |
| <kbd>Ctrl</kbd> + <kbd>J</kbd> | Abrir panel lateral de descargas |
| <kbd>Ctrl</kbd> + <kbd>F</kbd> | Abrir barra de búsqueda en página |
| <kbd>Ctrl</kbd> + <kbd>,</kbd> | Abrir pestaña de Configuración (`nika://settings`) |
| <kbd>Alt</kbd> + <kbd>←</kbd> | Retroceder en el historial |
| <kbd>Alt</kbd> + <kbd>→</kbd> | Avanzar en el historial |
| <kbd>Ctrl</kbd> + <kbd>+</kbd> / <kbd>-</kbd> / <kbd>0</kbd> | Aumentar, disminuir y restablecer zoom |

---

## 📄 Licencia

Este proyecto está distribuido bajo la licencia [MIT](LICENSE).
Creado y mantenido por **[MrDemonc](https://github.com/MrDemonc)**.
