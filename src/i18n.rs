/// Sistema de Internacionalización (i18n) para Nika Browser (Español e Inglés)

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Es,
    En,
}

impl Language {
    pub fn from_code(code: &str) -> Self {
        match code.trim().to_lowercase().as_str() {
            "en" | "en-us" | "en-gb" | "english" => Self::En,
            _ => Self::Es,
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::Es => "es",
            Self::En => "en",
        }
    }
}

/// Función de traducción con clave e idioma
pub fn tr(lang: &str, key: &'static str) -> &'static str {
    let is_en = Language::from_code(lang) == Language::En;
    match key {
        // ==========================================
        // Barra Superior / HeaderBar
        // ==========================================
        "nav_back" => if is_en { "Back" } else { "Atrás" },
        "nav_forward" => if is_en { "Forward" } else { "Adelante" },
        "nav_reload" => if is_en { "Reload page" } else { "Recargar página" },
        "nav_stop" => if is_en { "Stop loading" } else { "Detener carga" },
        "nav_home" => if is_en { "Home" } else { "Inicio" },
        "nav_toggle_sidebar" => if is_en { "Toggle tabs sidebar (Ctrl+B)" } else { "Alternar barra de pestañas (Ctrl+B)" },
        "nav_downloads" => if is_en { "Downloads (Ctrl+J)" } else { "Descargas (Ctrl+J)" },
        "nav_shield" => if is_en { "Nika Shield - Privacy Active" } else { "Escudo Nika - Privacidad Activa" },
        "nav_settings" => if is_en { "Settings (Ctrl+,)" } else { "Configuración (Ctrl+,)" },
        "omnibox_placeholder" => if is_en { "Search with Google or enter URL..." } else { "Buscar con Google o ingresa una URL..." },
        "copy_url" => if is_en { "Copy URL to clipboard" } else { "Copiar enlace al portapapeles" },
        "copied" => if is_en { "Copied to clipboard!" } else { "¡Enlace copiado!" },

        // ==========================================
        // Panel Lateral de Pestañas / Sidebar
        // ==========================================
        "sidebar_title" => if is_en { "Tabs" } else { "Pestañas" },
        "sidebar_pin" => if is_en { "Panel pinned (Click to auto-hide)" } else { "Panel fijado permanentemente (Clic para auto-ocultar)" },
        "sidebar_auto_hide" => if is_en { "Auto-hide active (Hover edge to show)" } else { "Auto-ocultar activo (Pasa el cursor por el borde para ver)" },
        "new_tab" => if is_en { "New tab (Ctrl+T)" } else { "Nueva pestaña (Ctrl+T)" },
        "new_incognito_tab" => if is_en { "New incognito tab (Ctrl+Shift+N)" } else { "Nueva pestaña de incógnito (Ctrl+Shift+N)" },
        "nav_translate" => if is_en { "Translate page (Click: translate, Right click: options)" } else { "Traducir página (Clic: traducir, Clic derecho: opciones)" },
        "translate_title" => if is_en { "Google Translate" } else { "Traducción de Google" },
        "translate_auto_suggest" => if is_en { "Auto-suggest translation" } else { "Sugerir traducción automáticamente" },
        "translate_target_lang" => if is_en { "Translate to:" } else { "Traducir al idioma:" },
        "translate_now" => if is_en { "Translate now" } else { "Traducir ahora" },
        "translate_restore" => if is_en { "Show original" } else { "Ver original" },
        "translate_active_tooltip" => if is_en { "Page translated (Click to restore original)" } else { "Página traducida (Clic para ver original)" },
        "collapse_sidebar" => if is_en { "Collapse tabs sidebar (Ctrl+B)" } else { "Ocultar barra de pestañas (Ctrl+B)" },
        "incognito_section" => if is_en { "Incognito" } else { "Incógnito" },
        "home_tab_title" => if is_en { "Home" } else { "Inicio" },
        "private_tab_title" => if is_en { "Private tab" } else { "Pestaña privada" },
        "close_tab" => if is_en { "Close tab" } else { "Cerrar pestaña" },
        "duplicate_tab" => if is_en { "Duplicate tab" } else { "Duplicar pestaña" },
        "close_other_tabs" => if is_en { "Close other tabs" } else { "Cerrar otras pestañas" },
        "media_now_playing" => if is_en { "NOW PLAYING" } else { "REPRODUCIENDO" },
        "media_paused" => if is_en { "PAUSED" } else { "PAUSADO" },
        "media_play" => if is_en { "Play" } else { "Reproducir" },
        "media_pause" => if is_en { "Pause" } else { "Pausar" },
        "media_prev" => if is_en { "Rewind 10s / Previous" } else { "Retroceder 10s / Anterior" },
        "media_next" => if is_en { "Fast-forward 10s / Next" } else { "Adelantar 10s / Siguiente" },
        "media_mute" => if is_en { "Mute" } else { "Silenciar" },
        "media_unmute" => if is_en { "Unmute" } else { "Reactivar sonido" },
        "media_switch_tab" => if is_en { "Click to switch to tab" } else { "Clic para ir a la pestaña" },
        "media_hide" => if is_en { "Hide player" } else { "Ocultar reproductor" },


        // ==========================================
        // Gestor de Descargas / Downloads
        // ==========================================
        "downloads_title" => if is_en { "Downloads" } else { "Descargas" },
        "downloads_empty" => if is_en { "No downloads yet" } else { "No hay descargas" },
        "open_folder" => if is_en { "Open downloads folder" } else { "Abrir carpeta de descargas" },
        "close" => if is_en { "Close" } else { "Cerrar" },
        "downloading" => if is_en { "Downloading..." } else { "Descargando..." },
        "download_completed" => if is_en { "Download completed" } else { "Descarga completada" },
        "download_completed_body" => if is_en { "Downloaded: " } else { "Descargado: " },
        "download_in_progress" => if is_en { "Downloading file... (Click or Ctrl+J to view)" } else { "Descargando archivo... (Clic o Ctrl+J para ver)" },
        "completed" => if is_en { "Completed" } else { "Completado" },
        "failed" => if is_en { "Failed" } else { "Fallido" },
        "cancelled" => if is_en { "Cancelled" } else { "Cancelado" },
        "cancel" => if is_en { "Cancel" } else { "Cancelar" },
        "delete_entry" => if is_en { "Delete from list" } else { "Eliminar de la lista" },

        // ==========================================
        // Barra de Búsqueda / FindBar
        // ==========================================
        "find_placeholder" => if is_en { "Find in page..." } else { "Buscar en la página..." },
        "find_prev" => if is_en { "Previous match (Shift+Enter)" } else { "Coincidencia anterior (Shift+Enter)" },
        "find_next" => if is_en { "Next match (Enter)" } else { "Siguiente coincidencia (Enter)" },
        "find_close" => if is_en { "Close search (Esc)" } else { "Cerrar búsqueda (Esc)" },
        "find_no_results" => if is_en { "No matches" } else { "Sin resultados" },
        "find_result" => if is_en { "1 match" } else { "1 resultado" },
        "find_results" => if is_en { "matches" } else { "resultados" },

        // ==========================================
        // Diálogo del Escudo Nika
        // ==========================================
        "shield_heading" => if is_en { "Nika Shield - Active Privacy" } else { "Escudo Nika - Privacidad Activa" },
        "shield_body" => if is_en {
            "This browser actively protects you by blocking:\n• Third-party telemetry and ads\n• Tracking parameters in links (UTM, fbclid, etc.)\n• Insecure cross-site tracking cookies\n• Mixed content and WebRTC IP leaks\n\nYour browsing leaves no traces and stays entirely local."
        } else {
            "Este navegador te protege activamente bloqueando:\n• Rastreadores y publicidad invasiva de terceros\n• Parámetros de rastreo espía en enlaces (UTM, fbclid, etc.)\n• Cookies de seguimiento entre sitios\n• Contenido mixto inseguro y fugas de IP por WebRTC\n\nTu navegación no deja huellas y permanece 100% local."
        },
        "open_settings" => if is_en { "Open Settings" } else { "Abrir Configuración" },
        "understood" => if is_en { "Understood" } else { "Entendido" },

        // ==========================================
        // Bóveda de Contraseñas / Vault
        // ==========================================
        "vault_saved_prompt" => if is_en { "Save password for this site?" } else { "¿Guardar contraseña para este sitio?" },
        "save" => if is_en { "Save" } else { "Guardar" },
        "never" => if is_en { "Never" } else { "Nunca" },

        // ==========================================
        // Permisos del Sitio / Site Permissions
        // ==========================================
        "site_permissions" => if is_en { "Site Permissions" } else { "Permisos del sitio" },
        "site_permissions_tooltip" => if is_en { "Site permissions and security" } else { "Permisos y seguridad del sitio" },
        "perm_sec_https" => if is_en { "Secure Connection (HTTPS)" } else { "Conexión Segura Cifrada (HTTPS)" },
        "perm_sec_http" => if is_en { "Insecure Connection (HTTP)" } else { "Conexión No Segura (HTTP)" },
        "perm_sec_internal" => if is_en { "System Page" } else { "Página del sistema" },
        "perm_notifications" => if is_en { "Notifications" } else { "Notificaciones" },
        "perm_notif_sub" => if is_en { "Desktop notifications and alerts" } else { "Avisos y alertas en el escritorio" },
        "perm_mic" => if is_en { "Microphone" } else { "Micrófono" },
        "perm_mic_sub" => if is_en { "Voice input and audio devices" } else { "Grabación y entrada de voz" },
        "perm_camera" => if is_en { "Camera" } else { "Cámara" },
        "perm_camera_sub" => if is_en { "Video capture and webcam" } else { "Captura de vídeo y cámara web" },
        "perm_location" => if is_en { "Location" } else { "Ubicación" },
        "perm_location_sub" => if is_en { "Geographic position sharing" } else { "Acceso a posición geográfica" },
        "perm_storage" => if is_en { "Storage & Cookies" } else { "Almacenamiento y Cookies" },
        "perm_storage_sub" => if is_en { "Persistent cookies and session data" } else { "Cookies y datos locales del sitio" },
        "perm_clipboard" => if is_en { "Clipboard" } else { "Portapapeles" },
        "perm_clip_sub" => if is_en { "Read and write clipboard data" } else { "Lectura y copia al portapapeles" },
        "perm_reset" => if is_en { "Reset permissions" } else { "Restablecer permisos" },
        "perm_clear_data" => if is_en { "Clear site cookies & data" } else { "Borrar cookies y datos" },
        "perm_cleared" => if is_en { "Site data cleared!" } else { "¡Datos del sitio borrados!" },
        "perm_badge_secure" => if is_en { "Secure" } else { "Seguro" },
        "perm_badge_insecure" => if is_en { "Not secure" } else { "No seguro" },
        "perm_badge_internal" => if is_en { "System" } else { "Sistema" },
        "perm_btn_reset" => if is_en { "Reset" } else { "Restablecer" },
        "perm_btn_clear" => if is_en { "Clear data" } else { "Borrar datos" },

        _ => key,
    }
}
