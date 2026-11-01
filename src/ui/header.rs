use gtk4::prelude::*;
use libadwaita as adw;

#[derive(Clone)]
pub struct HeaderComponents {
    pub header_bar: adw::HeaderBar,
    pub sidebar_toggle_btn: gtk4::Button,
    pub btn_back: gtk4::Button,
    pub btn_forward: gtk4::Button,
    pub reload_btn: gtk4::Button,
    pub btn_home: gtk4::Button,
    pub media_revealer: gtk4::Revealer,
    pub btn_media_prev: gtk4::Button,
    pub btn_media_play: gtk4::Button,
    pub btn_media_next: gtk4::Button,
    pub omnibox: gtk4::Entry,
    pub shield_btn: gtk4::Button,
    pub settings_btn: gtk4::Button,
}

impl HeaderComponents {
    pub fn new(download_btn: &gtk4::Button) -> Self {
        let header_bar = adw::HeaderBar::new();
        header_bar.add_css_class("nika-header");
        header_bar.add_css_class("flat");

        // Botones de navegación pequeños, compactos y refinados
        let sidebar_toggle_btn = gtk4::Button::from_icon_name("nika-sidebar-symbolic");
        sidebar_toggle_btn.add_css_class("nav-button");
        sidebar_toggle_btn.set_tooltip_text(Some("Alternar barra de pestañas (Ctrl+B)"));

        let btn_back = gtk4::Button::from_icon_name("nika-back-symbolic");
        btn_back.add_css_class("nav-button");
        btn_back.set_tooltip_text(Some("Atrás"));

        let btn_forward = gtk4::Button::from_icon_name("nika-forward-symbolic");
        btn_forward.add_css_class("nav-button");
        btn_forward.set_tooltip_text(Some("Adelante"));

        let reload_btn = gtk4::Button::from_icon_name("nika-reload-symbolic");
        reload_btn.add_css_class("nav-button");
        reload_btn.set_tooltip_text(Some("Recargar página"));

        let btn_home = gtk4::Button::from_icon_name("nika-home-symbolic");
        btn_home.add_css_class("nav-button");
        btn_home.set_tooltip_text(Some("Inicio"));

        // Controles multimedia compactos a la derecha del botón Inicio
        let media_revealer = gtk4::Revealer::new();
        media_revealer.set_transition_type(gtk4::RevealerTransitionType::SlideRight);
        media_revealer.set_transition_duration(250);
        media_revealer.set_reveal_child(false);

        let media_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 1);
        media_box.add_css_class("header-media-box");

        let btn_media_prev = gtk4::Button::from_icon_name("nika-prev-symbolic");
        btn_media_prev.add_css_class("nav-button");
        btn_media_prev.add_css_class("header-media-btn");
        btn_media_prev.set_tooltip_text(Some("Pista anterior"));

        let btn_media_play = gtk4::Button::from_icon_name("nika-play-symbolic");
        btn_media_play.add_css_class("nav-button");
        btn_media_play.add_css_class("header-media-btn");
        btn_media_play.add_css_class("header-media-play-btn");
        btn_media_play.set_tooltip_text(Some("Reproducir / Pausar"));

        let btn_media_next = gtk4::Button::from_icon_name("nika-next-symbolic");
        btn_media_next.add_css_class("nav-button");
        btn_media_next.add_css_class("header-media-btn");
        btn_media_next.set_tooltip_text(Some("Siguiente pista"));

        media_box.append(&btn_media_prev);
        media_box.append(&btn_media_play);
        media_box.append(&btn_media_next);
        media_revealer.set_child(Some(&media_box));

        let nav_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 1);
        nav_box.append(&sidebar_toggle_btn);
        nav_box.append(&btn_back);
        nav_box.append(&btn_forward);
        nav_box.append(&reload_btn);
        nav_box.append(&btn_home);
        nav_box.append(&media_revealer);
        header_bar.pack_start(&nav_box);

        // Omnibox compacto horizontal y verticalmente con icono de permisos y seguridad
        let omnibox = gtk4::Entry::builder()
            .placeholder_text("Buscar con Chromium o ingresa una URL...")
            .hexpand(false)
            .halign(gtk4::Align::Center)
            .width_chars(38)
            .primary_icon_name("security-high-symbolic")
            .primary_icon_tooltip_text("Permisos y seguridad del sitio (Clic para ver detalles)")
            .primary_icon_activatable(true)
            .secondary_icon_name("nika-copy-symbolic")
            .secondary_icon_tooltip_text("Copiar URL al portapapeles")
            .secondary_icon_activatable(true)
            .build();
        omnibox.add_css_class("omnibox");

        header_bar.set_title_widget(Some(&omnibox));

        // Acciones del lado derecho: Descargas, Escudo Nika, Configuración
        let shield_btn = gtk4::Button::from_icon_name("nika-shield-symbolic");
        shield_btn.add_css_class("shield-pill");
        shield_btn.set_tooltip_text(Some("Escudo de privacidad activo (Clic para detalles)"));
        shield_btn.set_valign(gtk4::Align::Center);
        shield_btn.set_halign(gtk4::Align::Center);

        let settings_btn = gtk4::Button::from_icon_name("nika-settings-symbolic");
        settings_btn.add_css_class("settings-btn");
        settings_btn.set_tooltip_text(Some("Configuración (Ctrl+,) • DNS Seguro, Descargas y Red"));
        settings_btn.set_valign(gtk4::Align::Center);

        header_bar.pack_end(&settings_btn);
        header_bar.pack_end(&shield_btn);
        header_bar.pack_end(download_btn);

        Self {
            header_bar,
            sidebar_toggle_btn,
            btn_back,
            btn_forward,
            reload_btn,
            btn_home,
            media_revealer,
            btn_media_prev,
            btn_media_play,
            btn_media_next,
            omnibox,
            shield_btn,
            settings_btn,
        }
    }

    pub fn set_media_playing_state(&self, playing: bool) {
        if playing {
            self.btn_media_play.set_icon_name("nika-pause-symbolic");
        } else {
            self.btn_media_play.set_icon_name("nika-play-symbolic");
        }
    }

    pub fn show_media_controls(&self) {
        if !self.media_revealer.reveals_child() {
            self.media_revealer.set_reveal_child(true);
        }
    }

    pub fn hide_media_controls(&self) {
        if self.media_revealer.reveals_child() {
            self.media_revealer.set_reveal_child(false);
        }
    }

    /// Actualiza los textos y tooltips de los componentes de cabecera según el idioma
    pub fn update_language(&self, lang: &str) {
        use crate::i18n::tr;
        let is_en = lang == "en";
        self.sidebar_toggle_btn.set_tooltip_text(Some(tr(lang, "nav_toggle_sidebar")));
        self.btn_back.set_tooltip_text(Some(tr(lang, "nav_back")));
        self.btn_forward.set_tooltip_text(Some(tr(lang, "nav_forward")));
        self.reload_btn.set_tooltip_text(Some(tr(lang, "nav_reload")));
        self.btn_home.set_tooltip_text(Some(tr(lang, "nav_home")));
        self.btn_media_prev.set_tooltip_text(Some(if is_en { "Previous track" } else { "Pista anterior" }));
        self.btn_media_play.set_tooltip_text(Some(if is_en { "Play / Pause" } else { "Reproducir / Pausar" }));
        self.btn_media_next.set_tooltip_text(Some(if is_en { "Next track" } else { "Siguiente pista" }));
        self.omnibox.set_placeholder_text(Some(tr(lang, "omnibox_placeholder")));
        self.omnibox.set_primary_icon_tooltip_text(Some(tr(lang, "site_permissions_tooltip")));
        self.omnibox.set_secondary_icon_tooltip_text(Some(tr(lang, "copy_url")));
        self.shield_btn.set_tooltip_text(Some(tr(lang, "nav_shield")));
        self.settings_btn.set_tooltip_text(Some(tr(lang, "nav_settings")));
    }
}
