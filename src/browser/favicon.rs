use std::cell::RefCell;
use std::collections::HashMap;
use url::Url;

thread_local! {
    static FAVICON_CACHE: RefCell<HashMap<String, gtk4::gdk::Texture>> = RefCell::new(HashMap::new());
}

pub struct FaviconManager;

impl FaviconManager {
    /// Aplica el favicon cacheado para una URL, si ya fue obtenido por WebKit.
    /// No realiza peticiones de red: WebKit emite `favicon` de forma nativa.
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

        let cached = FAVICON_CACHE.with(|cache| cache.borrow().get(&host).cloned());
        if let Some(texture) = cached {
            icon.set_paintable(Some(&texture));
        }
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
