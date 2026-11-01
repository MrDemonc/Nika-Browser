# 🌸 Nika Browser

**Nika Browser** es un navegador web moderno, minimalista y ultra-seguro desarrollado en **Rust** para Linux (optimizado nativamente para Arch Linux con GTK4, Libadwaita y WebKitGTK 6.0).

---

## ✨ Características Principales

1. **🛡️ 100% Seguro y Sin Rastreadores**:
   - **Escudo WebKit Nativo**: Bloquea telemetría, scripts de analíticas (Google Analytics, Facebook Pixel, Hotjar, etc.) y redes publicitarias mediante filtros compilados directamente en el motor WebKit.
   - **Limpiador Automático de URLs**: Elimina los parámetros de rastreo invasivos (`utm_*`, `fbclid`, `gclid`, `igshid`, `yclid`, etc.) antes de que la petición se envíe.
   - **Bloqueo de Cookies de Terceros**: Impide que sitios externos de publicidad y rastreo perfilen tu historial.
   - **Actualización Forzada a HTTPS**: Todas las conexiones no cifradas se elevan automáticamente a HTTPS seguro.
   - **Anti-Beacons**: Deshabilita la auditoría de hipervínculos (`<a ping>`) y ventanas emergentes automáticas.

2. **📑 Barra de Pestañas Moderna y Minimalista**:
   - Pestañas con estilo cápsula (*pills*), animaciones fluidas, indicador activo y soporte para arrastrar, cerrar y reordenar pestañas (`adw::TabBar` + `adw::TabView`).
   - Botón de nueva pestaña (+) y botón de cerrar individuales.
   - Si cierras la última pestaña, se restaura automáticamente la página de bienvenida en lugar de cerrar el navegador.

3. **🎨 Paleta de Colores Pasteles y Modos Claro / Oscuro**:
   - **Modo Claro Pastel**: Tonos suaves lavanda (`#C8B6FF`), menta salvia (`#BBF2C9`), melocotón (`#FFD6A5`) y cielo (`#A0C4FF`) sobre un fondo crema relajante (`#F8F6F0`).
   - **Modo Oscuro Pastel**: Fondos pizarra profundo (`#1A1B26` / `#24283B`) con acentos pasteles descansados para la vista (`#BB9AF7`, `#73DACA`, `#FF9E64`).
   - **Alternancia Instantánea**: Botón en la barra de navegación para alternar con un clic entre ☀️ Claro y 🌙 Oscuro.

- **🔎 Motor de Búsqueda Exclusivo (Chromium)**:
  - Todo término de búsqueda ingresado se procesa a través del motor de Chromium (`&pws=0` despersonalizado y limpio), sin telemetría ni rastreadores.

- **🎬 Reproducción de Videos de YouTube y Medios HTML5**:
  - Habilitado soporte para **MediaSource Extensions (MSE)**, **WebAudio**, **Media Streams**, **DRM/EME (Encrypted Media)** y codecs de video/audio VP9/VP8/H.264/WebM.
  - User-Agent de Chromium Linux moderno para forzar a YouTube a reproducir sin trabas.

- **🖤 Modo Oscuro Negro Puro (Pitch Black / OLED)**:
  - Fondo negro absoluto (`#000000`), barras en tonos ébano (`#08080A`, `#101014`) y detalles en acentos pasteles descansados para la vista (`#C084FC`, `#6EE7B7`).

- **✨ Barra de Pestañas y Navegación Ultra-Compactas**:
  - Altura reducida a **28px**, pestañas delgadas y discretas, botones de navegación sutiles y Omnibox ajustado.

5. **🏠 Pantalla de Inicio Minimalista (*New Tab*)**:
   - Reloj minimalista en tiempo real con saludo dinámico.
   - Barra de búsqueda central en cápsula pastel.
   - Accesos directos rápidos estilizados (DuckDuckGo, Brave, Wikipedia, GitHub, Arch Linux).
   - Indicador de estado del Escudo de Privacidad.

---

## 🚀 Cómo Ejecutar

Para iniciar el navegador:

```bash
# Opción 1: Ejecución directa con cargo
cd /home/demonc/Documentos/proyectos/nika-browser
cargo run --release

# Opción 2: Usar el script de lanzamiento
./run.sh

# Opción 3: Desde el menú de aplicaciones de tu escritorio
# Busca "Nika Browser" en tu lanzador de aplicaciones (GNOME, KDE, Waybar, Rofi, etc.)
```

---

## 🏗️ Estructura del Proyecto

```
nika-browser/
├── Cargo.toml                          # Dependencias: gtk4 (v4_14), libadwaita (v1_6), webkit6, url, regex
├── run.sh                              # Script de ejecución rápida
├── resources/
│   ├── rules/
│   │   └── privacy-rules.json          # Reglas del motor de bloqueo WebKit
│   ├── styles/
│   │   └── pastel.css                  # Hoja de estilos con variables pasteles y temas
│   └── home/
│       └── index.html                  # Pantalla de inicio minimalista en pastel
└── src/
    ├── main.rs                         # Punto de entrada de la aplicación Libadwaita
    ├── app.rs                          # Ventana principal, barra de navegación y pestañas
    ├── browser_tab.rs                  # Controlador de pestañas WebKit y gestión de carga
    ├── privacy.rs                      # Sanitizador de URLs y compilador de filtros WebKit
    ├── search.rs                       # Motores de búsqueda y resolución de consultas
    └── theme.rs                        # Gestor de temas claro / oscuro pastel
```

---

## 🔒 Privacidad y Motores de Búsqueda (Nota Técnica)

- **Chromium vs WebKit en Rust**: En el ecosistema Rust nativo en Linux, **WebKitGTK 6.0** proporciona la mejor integración gráfica con aceleración por hardware vía Wayland/GTK4 y cuenta con la API oficial de **Content Blocking** (la misma tecnología utilizada por Safari y Brave en iOS).
- **Motores de Búsqueda**: Chromium es el proyecto base de navegadores (Blink engine), mientras que los motores de búsqueda para consultar la web de forma 100% privada son DuckDuckGo, Brave Search y SearXNG, todos integrados en la barra de direcciones de Nika.
