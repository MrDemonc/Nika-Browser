use gdk_pixbuf::prelude::*;
use gio::prelude::FileExt;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use url::Url;

thread_local! {
    static FAVICON_CACHE: RefCell<HashMap<String, gtk4::gdk::Texture>> = RefCell::new(HashMap::new());
    static IN_FLIGHT: RefCell<HashSet<String>> = RefCell::new(HashSet::new());
}

pub struct FaviconManager;

impl FaviconManager {
    fn cache_dir() -> PathBuf {
        let dir = glib::user_cache_dir().join("nika-browser").join("favicons");
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    fn safe_host_filename(host: &str) -> String {
        let sanitized: String = host
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '.' || c == '-' { c } else { '_' })
            .collect();
        format!("{}.png", sanitized)
    }

    /// Obtiene el favicon cacheado (primero en memoria, luego en disco).
    pub fn get_cached(host: &str) -> Option<gtk4::gdk::Texture> {
        let h = host.to_lowercase();
        // 1. Memoria
        if let Some(tex) = FAVICON_CACHE.with(|c| c.borrow().get(&h).cloned()) {
            return Some(tex);
        }

        // 2. Disco
        let file_path = Self::cache_dir().join(Self::safe_host_filename(&h));
        if file_path.exists() {
            if let Ok(pixbuf) = gdk_pixbuf::Pixbuf::from_file(&file_path) {
                let scaled = if pixbuf.width() > 24 || pixbuf.height() > 24 {
                    pixbuf
                        .scale_simple(18, 18, gdk_pixbuf::InterpType::Bilinear)
                        .unwrap_or(pixbuf)
                } else {
                    pixbuf
                };
                let texture = gtk4::gdk::Texture::for_pixbuf(&scaled);
                FAVICON_CACHE.with(|c| {
                    c.borrow_mut().insert(h, texture.clone());
                });
                return Some(texture);
            }
        }

        None
    }

    /// Aplica el favicon cacheado para una URL si ya existe en caché de memoria o disco.
    /// Retorna true si se aplicó un favicon cacheado.
    pub fn load_for_url(url_str: &str, icon: &gtk4::Image) -> bool {
        let trimmed = url_str.trim();
        if trimmed.starts_with("nika://")
            || trimmed.starts_with("file://")
            || trimmed == "about:blank"
            || trimmed.is_empty()
        {
            return false;
        }

        let parsed = match Url::parse(trimmed) {
            Ok(u) => u,
            Err(_) => return false,
        };

        let host = match parsed.host_str() {
            Some(h) => h.to_lowercase(),
            None => return false,
        };

        if let Some(texture) = Self::get_cached(&host) {
            icon.set_icon_name(None);
            icon.set_paintable(Some(&texture));
            return true;
        }

        false
    }

    /// Guarda una textura en la caché de memoria
    pub fn cache_texture(host: &str, texture: &gtk4::gdk::Texture) {
        let h = host.to_lowercase();
        let tex = texture.clone();
        FAVICON_CACHE.with(|cache| {
            cache.borrow_mut().insert(h, tex);
        });
    }

    /// Guarda el pixbuf en el disco para persistencia entre sesiones
    pub fn save_pixbuf_to_disk(host: &str, pixbuf: &gdk_pixbuf::Pixbuf) {
        let h = host.to_lowercase();
        let file_path = Self::cache_dir().join(Self::safe_host_filename(&h));
        let _ = pixbuf.savev(&file_path, "png", &[]);
    }

    /// Descarga y decodifica el favicon asíncronamente para un host dado.
    /// Utiliza deduplicación de peticiones concurrentes para evitar saturación de red/procesos.
    pub fn fetch_for_url(
        page_url: &str,
        candidate_icon_url: Option<String>,
        icon: &gtk4::Image,
    ) {
        let trimmed = page_url.trim();
        if trimmed.starts_with("nika://")
            || trimmed.starts_with("file://")
            || trimmed == "about:blank"
            || trimmed.is_empty()
        {
            return;
        }

        let parsed = match Url::parse(trimmed) {
            Ok(u) => u,
            Err(_) => return,
        };

        let host = match parsed.host_str() {
            Some(h) => h.to_lowercase(),
            None => return,
        };

        // Si ya está en caché, aplicarlo de inmediato
        if let Some(texture) = Self::get_cached(&host) {
            icon.set_icon_name(None);
            icon.set_paintable(Some(&texture));
            return;
        }

        // Evitar múltiples descargas en vuelo para el mismo host
        let already_in_flight = IN_FLIGHT.with(|set| !set.borrow_mut().insert(host.clone()));
        if already_in_flight {
            return;
        }

        // Construir candidatos: URL explícita del HTML -> Google S2 Favicon API
        let mut candidates = Vec::new();

        if let Some(u) = candidate_icon_url {
            let u_trimmed = u.trim();
            if !u_trimmed.is_empty() {
                if let Ok(resolved) = parsed.join(u_trimmed) {
                    let s = resolved.to_string();
                    if s.starts_with("http://") || s.starts_with("https://") {
                        candidates.push(s);
                    }
                }
            }
        }

        // Fallback global de alta resolución (funciona para el 99.9% de sitios incluyendo YouTube)
        candidates.push(format!("https://www.google.com/s2/favicons?domain={}&sz=64", host));

        Self::try_fetch_next(candidates, 0, host, icon.downgrade());
    }

    fn finish_fetch(host: &str) {
        IN_FLIGHT.with(|set| {
            set.borrow_mut().remove(host);
        });
    }

    fn try_fetch_next(
        candidates: Vec<String>,
        idx: usize,
        host: String,
        icon_weak: glib::WeakRef<gtk4::Image>,
    ) {
        if idx >= candidates.len() {
            Self::finish_fetch(&host);
            return;
        }

        let target_url = candidates[idx].clone();
        let file = gio::File::for_uri(&target_url);
        let host_for_cb = host.clone();

        file.load_bytes_async(
            None::<&gio::Cancellable>,
            move |result| match result {
                Ok((bytes, _)) => {
                    let slice = bytes.as_ref();
                    if slice.len() > 10 {
                        let loader = gdk_pixbuf::PixbufLoader::new();
                        if loader.write(slice).is_ok() && loader.close().is_ok() {
                            if let Some(pixbuf) = loader.pixbuf() {
                                let scaled = pixbuf
                                    .scale_simple(18, 18, gdk_pixbuf::InterpType::Bilinear)
                                    .unwrap_or(pixbuf);
                                let texture = gtk4::gdk::Texture::for_pixbuf(&scaled);

                                Self::cache_texture(&host_for_cb, &texture);
                                Self::save_pixbuf_to_disk(&host_for_cb, &scaled);

                                if let Some(icon) = icon_weak.upgrade() {
                                    icon.set_icon_name(None);
                                    icon.set_paintable(Some(&texture));
                                }
                                Self::finish_fetch(&host_for_cb);
                                return;
                            }
                        }
                    }
                    // Si falló decodificar este candidato, intentar siguiente
                    Self::try_fetch_next(candidates, idx + 1, host_for_cb, icon_weak);
                }
                Err(_) => {
                    Self::try_fetch_next(candidates, idx + 1, host_for_cb, icon_weak);
                }
            },
        );
    }
}
