use gtk4::prelude::*;
use libadwaita as adw;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Light,
    Dark,
}

pub struct ThemeManager {
    pub current_mode: ThemeMode,
    pub custom_color: String,
    pub dynamic_provider: gtk4::CssProvider,
}

impl ThemeManager {
    pub fn new(initial_mode: &str, initial_color: &str) -> Self {
        let current_mode = if initial_mode.eq_ignore_ascii_case("light") {
            ThemeMode::Light
        } else {
            ThemeMode::Dark
        };

        let dynamic_provider = gtk4::CssProvider::new();
        if let Some(display) = gtk4::gdk::Display::default() {
            gtk4::style_context_add_provider_for_display(
                &display,
                &dynamic_provider,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION + 20,
            );
        }

        Self {
            current_mode,
            custom_color: initial_color.to_string(),
            dynamic_provider,
        }
    }

    /// Aplica el modo y el color personalizado a la ventana
    pub fn apply_to_window(&self, window: &adw::ApplicationWindow) {
        let style_manager = adw::StyleManager::default();

        match self.current_mode {
            ThemeMode::Light => {
                style_manager.set_color_scheme(adw::ColorScheme::ForceLight);
                window.add_css_class("light-theme");
                window.remove_css_class("dark-theme");
            }
            ThemeMode::Dark => {
                style_manager.set_color_scheme(adw::ColorScheme::ForceDark);
                window.add_css_class("dark-theme");
                window.remove_css_class("light-theme");
            }
        }

        let css = self.generate_theme_css();
        self.dynamic_provider.load_from_string(&css);
    }

    /// Cambia el modo de tema (Light / Dark) y actualiza la ventana
    #[allow(dead_code)]
    pub fn set_theme_mode(&mut self, window: &adw::ApplicationWindow, mode: ThemeMode) {
        self.current_mode = mode;
        self.apply_to_window(window);
    }

    /// Alterna entre modo claro y oscuro
    #[allow(dead_code)]
    pub fn toggle_theme(&mut self, window: &adw::ApplicationWindow) -> ThemeMode {
        let next_mode = match self.current_mode {
            ThemeMode::Light => ThemeMode::Dark,
            ThemeMode::Dark => ThemeMode::Light,
        };
        self.set_theme_mode(window, next_mode);
        self.current_mode
    }

    /// Establece un color de personalización para el navegador (hexadecimal como #ff7675)
    pub fn set_theme_color(&mut self, window: &adw::ApplicationWindow, color_hex: &str) {
        self.custom_color = color_hex.trim().to_string();
        self.apply_to_window(window);
    }

    /// Genera las reglas CSS dinámicas para el color personalizado seleccionado
    fn generate_theme_css(&self) -> String {
        let hex = self.custom_color.trim();
        if hex.is_empty() {
            return String::new();
        }

        let Some((r, g, b)) = parse_hex_color(hex) else {
            return String::new();
        };

        let lum = 0.299 * (r as f32) + 0.587 * (g as f32) + 0.114 * (b as f32);
        let is_bg_light = lum > 145.0;

        let clean_hex = format!("#{:02x}{:02x}{:02x}", r, g, b);

        if is_bg_light {
            format!(
                r#"
window.nika-window,
.docked-sidebar,
.floating-sidebar {{
    background-color: {clean_hex} !important;
}}

.sidebar-header .sidebar-title,
.sidebar-title,
.tab-folder-name,
.tab-menu-section-header,
.folders-section-header {{
    color: #1a1a1e !important;
}}

.arc-new-tab-btn {{
    color: #2c2c32 !important;
}}
.arc-new-tab-btn:hover {{
    color: #000000 !important;
    background-color: rgba(0, 0, 0, 0.08) !important;
}}

.arc-nav-btn,
.arc-toggle-btn,
.arc-tool-btn {{
    color: #333338 !important;
}}
.arc-nav-btn:hover,
.arc-toggle-btn:hover,
.arc-tool-btn:hover {{
    color: #000000 !important;
    background-color: rgba(0, 0, 0, 0.08) !important;
}}

.arc-sidebar-omnibox {{
    background-color: rgba(255, 255, 255, 0.65) !important;
    border-color: rgba(0, 0, 0, 0.12) !important;
    color: #121212 !important;
}}
.arc-sidebar-omnibox:focus {{
    background-color: #ffffff !important;
    border-color: #121212 !important;
}}

.vertical-tabs-box .tab-pill {{
    color: #2b2b2e !important;
}}
.vertical-tabs-box .tab-pill:hover {{
    background-color: rgba(0, 0, 0, 0.08) !important;
    color: #000000 !important;
}}
.vertical-tabs-box .tab-pill.active {{
    background-color: #ffffff !important;
    color: #121212 !important;
    border-radius: 9px !important;
    border: 1px solid rgba(0, 0, 0, 0.08) !important;
    box-shadow: 0 2px 10px rgba(0, 0, 0, 0.16) !important;
}}
.vertical-tabs-box .tab-pill.active .tab-title,
.vertical-tabs-box .tab-pill.active .tab-icon {{
    color: #121212 !important;
}}
.vertical-tabs-box .tab-pill.active .tab-close-btn {{
    color: #555558 !important;
}}

.arc-sidebar-footer {{
    border-top-color: rgba(0, 0, 0, 0.1) !important;
}}
"#
            )
        } else {
            format!(
                r#"
window.nika-window,
.docked-sidebar,
.floating-sidebar {{
    background-color: {clean_hex} !important;
}}

.sidebar-header .sidebar-title,
.sidebar-title,
.tab-folder-name,
.tab-menu-section-header,
.folders-section-header {{
    color: #f2f2f6 !important;
}}

.arc-new-tab-btn {{
    color: #d0d0d8 !important;
}}
.arc-new-tab-btn:hover {{
    color: #ffffff !important;
    background-color: rgba(255, 255, 255, 0.1) !important;
}}

.arc-nav-btn,
.arc-toggle-btn,
.arc-tool-btn {{
    color: #d6d6dc !important;
}}
.arc-nav-btn:hover,
.arc-toggle-btn:hover,
.arc-tool-btn:hover {{
    color: #ffffff !important;
    background-color: rgba(255, 255, 255, 0.12) !important;
}}

.arc-sidebar-omnibox {{
    background-color: rgba(0, 0, 0, 0.28) !important;
    border-color: rgba(255, 255, 255, 0.14) !important;
    color: #ffffff !important;
}}
.arc-sidebar-omnibox:focus {{
    background-color: rgba(0, 0, 0, 0.45) !important;
    border-color: #48dbfb !important;
}}

.vertical-tabs-box .tab-pill {{
    color: #dddddd !important;
}}
.vertical-tabs-box .tab-pill:hover {{
    background-color: rgba(255, 255, 255, 0.1) !important;
    color: #ffffff !important;
}}
.vertical-tabs-box .tab-pill.active {{
    background-color: #ffffff !important;
    color: #121212 !important;
    border-radius: 9px !important;
    box-shadow: 0 2px 12px rgba(0, 0, 0, 0.4) !important;
}}
.vertical-tabs-box .tab-pill.active .tab-title,
.vertical-tabs-box .tab-pill.active .tab-icon {{
    color: #121212 !important;
}}
.vertical-tabs-box .tab-pill.active .tab-close-btn {{
    color: #555558 !important;
}}

.arc-sidebar-footer {{
    border-top-color: rgba(255, 255, 255, 0.1) !important;
}}
"#
            )
        }
    }

    /// Carga y aplica el CSS personalizado de tonos pasteles
    pub fn load_css(css_path: &std::path::Path) {
        let provider = gtk4::CssProvider::new();

        if css_path.exists() {
            provider.load_from_path(css_path);
        } else {
            // CSS fallback integrado en caso de ejecución portable
            provider.load_from_string(include_str!("../resources/styles/pastel.css"));
        }

        if let Some(display) = gtk4::gdk::Display::default() {
            gtk4::style_context_add_provider_for_display(
                &display,
                &provider,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
    }
}

fn parse_hex_color(hex: &str) -> Option<(u8, u8, u8)> {
    let s = hex.trim().trim_start_matches('#');
    if s.len() == 6 {
        let r = u8::from_str_radix(&s[0..2], 16).ok()?;
        let g = u8::from_str_radix(&s[2..4], 16).ok()?;
        let b = u8::from_str_radix(&s[4..6], 16).ok()?;
        Some((r, g, b))
    } else {
        None
    }
}
