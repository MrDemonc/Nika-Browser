use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use url::Url;

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

/// Ajustes de permisos específicos para un dominio o sitio web
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SitePermissionSettings {
    #[serde(default = "default_false")]
    pub notifications: bool,
    #[serde(default = "default_false")]
    pub microphone: bool,
    #[serde(default = "default_false")]
    pub camera: bool,
    #[serde(default = "default_false")]
    pub geolocation: bool,
    #[serde(default = "default_true")]
    pub storage: bool,
    #[serde(default = "default_false")]
    pub clipboard: bool,
}

impl Default for SitePermissionSettings {
    fn default() -> Self {
        Self {
            notifications: false,
            microphone: false,
            camera: false,
            geolocation: false,
            storage: true,
            clipboard: false,
        }
    }
}

/// Gestor central de permisos persistentes por sitio web
#[derive(Debug, Clone)]
pub struct SitePermissionsManager {
    sites: Rc<RefCell<HashMap<String, SitePermissionSettings>>>,
}

impl SitePermissionsManager {
    fn file_path() -> PathBuf {
        let dir = glib::user_config_dir().join("nika-browser");
        let _ = fs::create_dir_all(&dir);
        dir.join("site_permissions.json")
    }

    pub fn new() -> Self {
        let path = Self::file_path();
        let map = if path.exists() {
            fs::read_to_string(&path)
                .ok()
                .and_then(|data| serde_json::from_str::<HashMap<String, SitePermissionSettings>>(&data).ok())
                .unwrap_or_default()
        } else {
            HashMap::new()
        };

        Self {
            sites: Rc::new(RefCell::new(map)),
        }
    }

    /// Extrae el dominio o identificador limpio de una URL
    pub fn extract_host(url_str: &str) -> String {
        let trimmed = url_str.trim();
        if trimmed.starts_with("nika://") || trimmed.starts_with("file://") || trimmed.is_empty() {
            return "nika://internal".to_string();
        }
        if let Ok(u) = Url::parse(trimmed) {
            if let Some(host) = u.host_str() {
                return host.to_lowercase();
            }
        }
        "default".to_string()
    }

    /// Comprueba si ya existe una configuración explícita guardada para este host
    pub fn has_site(&self, host: &str) -> bool {
        let clean = host.to_lowercase();
        self.sites.borrow().contains_key(&clean)
    }

    /// Obtiene los permisos configurados para un host (o los predeterminados)
    pub fn get_permissions(&self, host: &str) -> SitePermissionSettings {
        let clean = host.to_lowercase();
        self.sites
            .borrow()
            .get(&clean)
            .cloned()
            .unwrap_or_default()
    }

    /// Modifica y persiste los permisos para un host
    pub fn set_permissions(&self, host: &str, perms: SitePermissionSettings) {
        let clean = host.to_lowercase();
        self.sites.borrow_mut().insert(clean, perms);
        self.save();
    }

    /// Restablece los permisos de un host a sus valores predeterminados
    pub fn reset_permissions(&self, host: &str) {
        let clean = host.to_lowercase();
        self.sites.borrow_mut().remove(&clean);
        self.save();
    }

    /// Guarda la base de datos de permisos en disco
    pub fn save(&self) {
        let path = Self::file_path();
        if let Ok(json) = serde_json::to_string_pretty(&*self.sites.borrow()) {
            let _ = fs::write(path, json);
        }
    }
}
