use std::path::Path;
use url::Url;
use webkit6::{Settings, UserContentFilterStore, UserContentManager};

/// Parámetros de rastreo espía para eliminar en URLs ingresadas
const TRACKER_QUERY_PARAMS: &[&str] = &[
    "utm_source",
    "utm_medium",
    "utm_campaign",
    "utm_term",
    "utm_content",
    "utm_id",
    "fbclid",
    "gclid",
    "gclsrc",
    "dclid",
    "msclkid",
    "mc_eid",
    "igshid",
    "_hsenc",
    "_hsmi",
    "ref_src",
    "ref_url",
    "yclid",
    "spm",
    "twclid",
    "zanpid",
    "vero_id",
    "wickedid",
    "adgroupid",
    "campaignid",
];

const EMBEDDED_RULES: &[u8] = include_bytes!("../../resources/rules/privacy-rules.json");

pub struct PrivacyEngine;

impl PrivacyEngine {
    /// Limpia parámetros de rastreo espía en URLs y asegura HTTPS
    pub fn clean_url(input_url: &str) -> String {
        let trimmed = input_url.trim();

        if trimmed.starts_with("file://") || trimmed == "about:blank" {
            return trimmed.to_string();
        }

        if let Ok(mut url) = Url::parse(trimmed) {
            // Forzar HTTPS si es HTTP ordinario (excepto localhost)
            if url.scheme() == "http"
                && url.host_str() != Some("localhost")
                && url.host_str() != Some("127.0.0.1")
            {
                let _ = url.set_scheme("https");
            }

            // Filtrar query parameters de rastreo
            let query_pairs: Vec<(String, String)> = url
                .query_pairs()
                .filter(|(key, _)| {
                    !TRACKER_QUERY_PARAMS
                        .iter()
                        .any(|p| p.eq_ignore_ascii_case(key))
                })
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();

            if query_pairs.is_empty() {
                url.set_query(None);
            } else {
                url.query_pairs_mut().clear().extend_pairs(query_pairs);
            }

            url.to_string()
        } else {
            trimmed.to_string()
        }
    }

    /// Configura las preferencias de seguridad, privacidad y máxima fluidez/aceleración WebKit
    pub fn configure_security_settings(settings: &Settings, webrtc_leak_protection: bool) {
        // User-Agent: Safari moderno en Linux compatible con WebKit
        // Evita el bug de WebGL mismatch en Cloudflare Turnstile al no fingir ser Blink/Chrome
        settings.set_user_agent(Some(
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Safari/605.1.15",
        ));

        // =======================================================
        // Permisos y Capacidades Web Esenciales (ChatGPT, Web Apps)
        // =======================================================
        // Habilitar almacenamiento HTML5 (LocalStorage, IndexedDB y bases de datos locales)
        settings.set_enable_html5_local_storage(true);
        settings.set_enable_html5_database(true);
        settings.set_hardware_acceleration_policy(webkit6::HardwareAccelerationPolicy::Always);

        // El acceso al portapapeles requiere permiso explícito por sitio.
        settings.set_javascript_can_access_clipboard(false);

        // Permitir popups y ventanas de inicio de sesión (Google OAuth / ChatGPT)
        settings.set_javascript_can_open_windows_automatically(true);

        // =======================================================
        // Protección y Blindaje de Seguridad (Hardening)
        // =======================================================
        // Aislamiento de seguridad web estricto (SOP, CORS, certificados)
        settings.set_disable_web_security(false);

        // Bloqueo total de acceso a archivos del sistema desde URLs locales
        settings.set_allow_file_access_from_file_urls(false);
        settings.set_allow_universal_access_from_file_urls(false);

        // Bloqueo de redirecciones y phishing mediante data: URLs
        settings.set_allow_top_navigation_to_data_urls(false);

        // Prevenir envenenamiento o volcado de logs a la consola estándar
        settings.set_enable_write_console_messages_to_stdout(false);

        // Desactivar dispositivos de captura mock
        settings.set_enable_mock_capture_devices(false);

        // Protección contra fugas de IP real por WebRTC
        if webrtc_leak_protection {
            settings.set_enable_webrtc(false);
        } else {
            settings.set_enable_webrtc(true);
        }

        // Gestos de navegación hacia adelante y atrás
        settings.set_enable_back_forward_navigation_gestures(true);

        // Quirks de compatibilidad para sitios como YouTube, OpenAI y Google
        settings.set_enable_site_specific_quirks(true);

        // =======================================================
        // Optimización de Fluidez, Rendimiento y Caché de Memoria
        // =======================================================
        settings.set_enable_smooth_scrolling(true);
        settings.set_enable_webgl(true);
        // Desactivar page-cache en memoria para evitar acumulación desmedida de RAM por pestañas
        settings.set_enable_page_cache(false);

        // =======================================================
        // Soporte de Reproducción de Videos (YouTube, HTML5, MSE)
        // =======================================================
        settings.set_enable_media(true);
        settings.set_enable_mediasource(true);
        settings.set_enable_webaudio(true);
        settings.set_enable_media_stream(true);
        settings.set_enable_media_capabilities(true);
        settings.set_enable_encrypted_media(true);
        settings.set_media_playback_allows_inline(true);
        settings.set_media_playback_requires_user_gesture(false);
    }

    /// Carga y aplica el filtro de bloqueo de rastreadores nativo en el UserContentManager
    pub fn apply_content_filters(content_manager: &UserContentManager, rules_json_path: &Path) {
        let file_bytes = match std::fs::read(rules_json_path) {
            Ok(bytes) => bytes,
            Err(_) => {
                eprintln!(
                    "[Escudo Nika] Reglas externas no encontradas en {:?}; usando reglas embebidas.",
                    rules_json_path
                );
                EMBEDDED_RULES.to_vec()
            }
        };

        let cache_dir = glib::user_cache_dir().join("nika-browser").join("filters");
        let _ = std::fs::create_dir_all(&cache_dir);

        let store = UserContentFilterStore::new(cache_dir.to_str().unwrap_or("/tmp"));
        let glib_bytes = glib::Bytes::from_owned(file_bytes);
        let cm_clone = content_manager.clone();

        store.save("nika_anti_tracker", &glib_bytes, None::<&gio::Cancellable>, move |result| {
            match result {
                Ok(filter) => {
                    cm_clone.add_filter(&filter);
                    println!("[Escudo Nika] ✅ Filtro de bloqueo de rastreadores WebKit activado exitosamente.");
                }
                Err(err) => {
                    eprintln!("[Escudo Nika] ⚠️ Error al compilar reglas de bloqueo: {}", err);
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    #[test]
    fn test_privacy_rules_json_validity() {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
        let path = PathBuf::from(manifest_dir)
            .join("resources")
            .join("rules")
            .join("privacy-rules.json");
        let content = std::fs::read_to_string(&path).expect("privacy-rules.json must exist");
        let json: serde_json::Value =
            serde_json::from_str(&content).expect("privacy-rules.json must be valid JSON");
        assert!(json.is_array());
        let arr = json.as_array().unwrap();
        assert!(!arr.is_empty());
        for rule in arr {
            assert!(rule.get("trigger").is_some());
            assert!(rule.get("action").is_some());
        }
    }
}
