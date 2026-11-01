use std::cell::RefCell;
use std::collections::HashMap;
use url::Url;

thread_local! {
    static FAVICON_CACHE: RefCell<HashMap<String, gtk4::gdk::Texture>> = RefCell::new(HashMap::new());
    static PENDING_ICONS: RefCell<HashMap<u64, (String, gtk4::Image)>> = RefCell::new(HashMap::new());
    static NEXT_REQ_ID: RefCell<u64> = RefCell::new(1);
}

pub struct FaviconManager;

impl FaviconManager {
    /// Carga el favicon para una URL en el widget Image.
    /// Si está en caché, lo aplica de inmediato; de lo contrario, lo descarga de fondo.
    pub fn load_for_url(url_str: &str, icon: &gtk4::Image) {
        let trimmed = url_str.trim();
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

        let origin = format!("{}://{}", parsed.scheme(), host);

        // 1. Comprobar caché en memoria
        let cached = FAVICON_CACHE.with(|cache| cache.borrow().get(&host).cloned());
        if let Some(texture) = cached {
            icon.set_paintable(Some(&texture));
            return;
        }

        // 2. Registrar en la lista local del hilo principal y descargar en hilo secundario
        let req_id = NEXT_REQ_ID.with(|id| {
            let val = *id.borrow();
            *id.borrow_mut() = val.wrapping_add(1);
            val
        });

        PENDING_ICONS.with(|m| {
            m.borrow_mut().insert(req_id, (host.clone(), icon.clone()));
        });

        let host_clone = host.clone();
        std::thread::spawn(move || {
            let bytes_opt = fetch_favicon(&origin, &host_clone);
            glib::idle_add_once(move || {
                if let Some((stored_host, img)) = PENDING_ICONS.with(|m| m.borrow_mut().remove(&req_id)) {
                    if let Some(bytes) = bytes_opt {
                        let glib_bytes = glib::Bytes::from_owned(bytes);
                        if let Ok(texture) = gtk4::gdk::Texture::from_bytes(&glib_bytes) {
                            FAVICON_CACHE.with(|cache| {
                                cache.borrow_mut().insert(stored_host, texture.clone());
                            });
                            img.set_paintable(Some(&texture));
                        }
                    }
                }
            });
        });
    }

    /// Guarda una textura (ej. de WebKit) en la caché
    pub fn cache_texture(host: &str, texture: &gtk4::gdk::Texture) {
        let h = host.to_lowercase();
        let tex = texture.clone();
        FAVICON_CACHE.with(|cache| {
            cache.borrow_mut().insert(h, tex);
        });
    }
}

/// Descarga bytes de favicon intentando primero /favicon.ico y luego Google S2 favicon service
fn fetch_favicon(origin: &str, host: &str) -> Option<Vec<u8>> {
    // Intento 1: favicon.ico directo del dominio
    let direct_url = format!("{}/favicon.ico", origin);
    if let Some(data) = curl_get(&direct_url) {
        if !data.is_empty() && (data.starts_with(b"\x89PNG") || data.starts_with(b"\x00\x00\x01\x00") || data.starts_with(b"<svg") || data.starts_with(b"GIF") || data.starts_with(b"\xff\xd8\xff")) {
            return Some(data);
        }
    }

    // Intento 2: Google Favicons API (alta confiabilidad y formato PNG garantizado)
    let google_url = format!("https://www.google.com/s2/favicons?domain={}&sz=32", host);
    if let Some(data) = curl_get(&google_url) {
        if !data.is_empty() {
            return Some(data);
        }
    }

    None
}

fn curl_get(url: &str) -> Option<Vec<u8>> {
    let output = std::process::Command::new("curl")
        .args(&["-s", "-L", "--max-time", "3", url])
        .output()
        .ok()?;
    if output.status.success() && !output.stdout.is_empty() {
        Some(output.stdout)
    } else {
        None
    }
}
