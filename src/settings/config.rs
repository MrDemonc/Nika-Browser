use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Proveedores de DNS seguro preconfigurados
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DnsProvider {
    System = 0,
    Cloudflare = 1,
    Google = 2,
    Quad9 = 3,
    AdGuard = 4,
    Custom = 5,
}

impl Default for DnsProvider {
    fn default() -> Self {
        Self::Cloudflare
    }
}

#[allow(dead_code)]
impl DnsProvider {
    pub fn name(&self) -> &'static str {
        match self {
            Self::System => "Sistema (Predeterminado)",
            Self::Cloudflare => "Cloudflare DNS (1.1.1.1 - Rápido y privado)",
            Self::Google => "Google Public DNS (8.8.8.8 - Global)",
            Self::Quad9 => "Quad9 (9.9.9.9 - Bloqueo de malware)",
            Self::AdGuard => "AdGuard DNS (94.140.14.14 - Bloqueo de anuncios)",
            Self::Custom => "DNS Personalizado",
        }
    }

    pub fn primary_ip(&self) -> &'static str {
        match self {
            Self::System => "Predeterminado de tu conexión",
            Self::Cloudflare => "1.1.1.1 / 1.0.0.1",
            Self::Google => "8.8.8.8 / 8.8.4.4",
            Self::Quad9 => "9.9.9.9 / 149.112.112.112",
            Self::AdGuard => "94.140.14.14 / 94.140.15.15",
            Self::Custom => "Personalizado",
        }
    }

    pub fn doh_url(&self) -> &'static str {
        match self {
            Self::System => "",
            Self::Cloudflare => "https://cloudflare-dns.com/dns-query",
            Self::Google => "https://dns.google/dns-query",
            Self::Quad9 => "https://dns.quad9.net/dns-query",
            Self::AdGuard => "https://dns.adguard-dns.com/dns-query",
            Self::Custom => "",
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_lang() -> String {
    "es".to_string()
}

fn default_time_format() -> String {
    "24h".to_string()
}

fn default_target_lang() -> String {
    "es".to_string()
}

/// Configuración persistente del navegador Nika
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserSettings {
    #[serde(default)]
    pub dns_provider: DnsProvider,
    #[serde(default)]
    pub custom_dns: String,
    #[serde(default = "default_true")]
    pub doh_enabled: bool,
    #[serde(default = "default_true")]
    pub tracker_blocking: bool,
    #[serde(default = "default_true")]
    pub restore_session: bool,
    #[serde(default)]
    pub sidebar_auto_hide: bool,
    #[serde(default)]
    pub download_dir: String,
    #[serde(default = "default_lang")]
    pub language: String,
    #[serde(default = "default_time_format")]
    pub time_format: String,
    #[serde(default)]
    pub onboarding_completed: bool,
    #[serde(default = "default_true")]
    pub block_insecure_content: bool,
    #[serde(default = "default_true")]
    pub webrtc_leak_protection: bool,
    #[serde(default)]
    pub clear_cache_on_exit: bool,
    #[serde(default)]
    pub auto_translate: bool,
    #[serde(default = "default_target_lang")]
    pub translate_target_lang: String,
}

impl Default for BrowserSettings {
    fn default() -> Self {
        Self {
            dns_provider: DnsProvider::Cloudflare,
            custom_dns: "https://cloudflare-dns.com/dns-query".to_string(),
            doh_enabled: true,
            tracker_blocking: true,
            restore_session: true,
            sidebar_auto_hide: false,
            download_dir: String::new(),
            language: "es".to_string(),
            time_format: "24h".to_string(),
            onboarding_completed: false,
            block_insecure_content: true,
            webrtc_leak_protection: true,
            clear_cache_on_exit: false,
            auto_translate: false,
            translate_target_lang: "es".to_string(),
        }
    }
}

impl BrowserSettings {
    /// Ruta del archivo de configuración JSON en el directorio de usuario
    fn config_path() -> PathBuf {
        let dir = glib::user_config_dir().join("nika-browser");
        let _ = fs::create_dir_all(&dir);
        dir.join("settings.json")
    }

    /// Carga la configuración desde disco o genera la predeterminada
    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(settings) = serde_json::from_str::<Self>(&content) {
                    return settings;
                }
            }
        }
        let def = Self::default();
        let _ = def.save();
        def
    }

    /// Guarda la configuración actual en disco
    pub fn save(&self) -> Result<(), std::io::Error> {
        let path = Self::config_path();
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fs::write(path, json)
    }

    /// Devuelve el directorio efectivo para descargas
    pub fn get_download_dir(&self) -> PathBuf {
        let trimmed = self.download_dir.trim();
        if !trimmed.is_empty() {
            let p = PathBuf::from(trimmed);
            if p.exists() || fs::create_dir_all(&p).is_ok() {
                return p;
            }
        }
        glib::user_special_dir(glib::UserDirectory::Downloads)
            .unwrap_or_else(|| glib::home_dir().join("Descargas"))
    }

    /// Aplica DNS prefetching según el proveedor configurado
    pub fn apply_dns(&self) {
        if let Some(session) = webkit6::NetworkSession::default() {
            match self.dns_provider {
                DnsProvider::Cloudflare => {
                    session.prefetch_dns("1.1.1.1");
                    session.prefetch_dns("cloudflare-dns.com");
                }
                DnsProvider::Google => {
                    session.prefetch_dns("8.8.8.8");
                    session.prefetch_dns("dns.google");
                }
                DnsProvider::Quad9 => {
                    session.prefetch_dns("9.9.9.9");
                    session.prefetch_dns("dns.quad9.net");
                }
                DnsProvider::AdGuard => {
                    session.prefetch_dns("94.140.14.14");
                    session.prefetch_dns("dns.adguard-dns.com");
                }
                _ => {}
            }
        }
    }
}
