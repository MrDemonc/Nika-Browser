use url::Url;

/// Motor de búsqueda único: Chromium (Google Search despersonalizado y limpio)
pub const CHROMIUM_SEARCH_BASE: &str = "https://www.google.com/search?q=";

/// Construye la URL de búsqueda en Chromium para una consulta dada
pub fn build_chromium_search_url(query: &str) -> String {
    let encoded = urlencoding_encode(query);
    format!("{}{}&pws=0", CHROMIUM_SEARCH_BASE, encoded)
}

/// Codificador de parámetros URL
fn urlencoding_encode(s: &str) -> String {
    let mut result = String::new();
    for byte in s.bytes() {
        match byte {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            b' ' => result.push('+'),
            _ => {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}

/// Resuelve si el texto ingresado en el Omnibox es una URL directa o una búsqueda con Chromium
pub fn resolve_query_or_url(input: &str) -> String {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return "about:blank".to_string();
    }

    // Esquemas explícitos (nika, http, https, file, about)
    if trimmed.starts_with("nika://")
        || trimmed.starts_with("http://")
        || trimmed.starts_with("https://")
        || trimmed.starts_with("file://")
        || trimmed.starts_with("about:")
    {
        return trimmed.to_string();
    }

    // Comprobar si parece un dominio de internet (e.g. youtube.com, reddit.com, localhost:3000)
    let has_spaces = trimmed.contains(' ');
    let parts: Vec<&str> = trimmed.split('/').collect();
    let host_part = parts[0];

    let is_localhost = host_part.starts_with("localhost");
    let has_valid_tld = host_part.contains('.') && !host_part.ends_with('.');

    if !has_spaces && (is_localhost || has_valid_tld) {
        if Url::parse(&format!("https://{}", trimmed)).is_ok() {
            return format!("https://{}", trimmed);
        }
    }

    // Buscar con Chromium
    build_chromium_search_url(trimmed)
}
