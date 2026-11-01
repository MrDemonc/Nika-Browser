use gtk4::prelude::*;
use libadwaita as adw;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Light,
    Dark,
}

#[allow(dead_code)]
pub struct ThemeManager {
    pub current_mode: ThemeMode,
}

#[allow(dead_code)]
impl ThemeManager {
    pub fn new() -> Self {
        let style_manager = adw::StyleManager::default();
        style_manager.set_color_scheme(adw::ColorScheme::ForceDark);

        Self {
            current_mode: ThemeMode::Dark,
        }
    }

    /// Alterna entre Modo Claro Pastel y Modo Oscuro Pastel
    pub fn toggle_theme(&mut self, window: &adw::ApplicationWindow) -> ThemeMode {
        let style_manager = adw::StyleManager::default();

        match self.current_mode {
            ThemeMode::Light => {
                self.current_mode = ThemeMode::Dark;
                style_manager.set_color_scheme(adw::ColorScheme::ForceDark);
                window.add_css_class("dark-theme");
                window.remove_css_class("light-theme");
            }
            ThemeMode::Dark => {
                self.current_mode = ThemeMode::Light;
                style_manager.set_color_scheme(adw::ColorScheme::ForceLight);
                window.add_css_class("light-theme");
                window.remove_css_class("dark-theme");
            }
        }

        self.current_mode
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
