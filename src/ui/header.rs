use gtk4::prelude::*;

#[derive(Clone)]
pub struct HeaderComponents {
    pub header_bar: gtk4::Box,
    pub sidebar_toggle_btn: gtk4::Button,
    pub btn_back: gtk4::Button,
    pub btn_forward: gtk4::Button,
    pub reload_btn: gtk4::Button,
    pub btn_home: gtk4::Button,
    pub omnibox: gtk4::Entry,
    pub tools_row: gtk4::Box,
    pub media_revealer: gtk4::Revealer,
    pub btn_media_prev: gtk4::Button,
    pub btn_media_play: gtk4::Button,
    pub btn_media_next: gtk4::Button,
    pub shield_btn: gtk4::Button,
    pub settings_btn: gtk4::Button,
}

impl HeaderComponents {
    pub fn new(
        download_btn: &gtk4::Button,
        translate_btn: &gtk4::Button,
    ) -> Self {
        // Cabecera superior integrada en el panel lateral (exacto a la imagen de Arc Browser)
        let header_bar = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
        header_bar.add_css_class("arc-sidebar-top");

        // Fila 1: Botón de alternar barra a la izquierda y botones de navegación a la derecha
        let nav_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 2);
        nav_row.add_css_class("arc-sidebar-nav-row");

        let sidebar_toggle_btn = gtk4::Button::from_icon_name("nika-sidebar-symbolic");
        sidebar_toggle_btn.add_css_class("sidebar-action-btn");
        sidebar_toggle_btn.add_css_class("arc-toggle-btn");
        sidebar_toggle_btn.set_tooltip_text(Some("Alternar barra lateral (Ctrl+B)"));

        let spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        spacer.set_hexpand(true);

        let btn_back = gtk4::Button::from_icon_name("nika-back-symbolic");
        btn_back.add_css_class("sidebar-action-btn");
        btn_back.add_css_class("arc-nav-btn");
        btn_back.set_tooltip_text(Some("Atrás"));

        let btn_forward = gtk4::Button::from_icon_name("nika-forward-symbolic");
        btn_forward.add_css_class("sidebar-action-btn");
        btn_forward.add_css_class("arc-nav-btn");
        btn_forward.set_tooltip_text(Some("Adelante"));

        let reload_btn = gtk4::Button::from_icon_name("nika-reload-symbolic");
        reload_btn.add_css_class("sidebar-action-btn");
        reload_btn.add_css_class("arc-nav-btn");
        reload_btn.set_tooltip_text(Some("Recargar página"));

        let btn_home = gtk4::Button::from_icon_name("nika-home-symbolic");
        btn_home.add_css_class("sidebar-action-btn");
        btn_home.add_css_class("arc-nav-btn");
        btn_home.set_tooltip_text(Some("Inicio"));

        nav_row.append(&sidebar_toggle_btn);
        nav_row.append(&spacer);
        nav_row.append(&btn_back);
        nav_row.append(&btn_forward);
        nav_row.append(&reload_btn);
        nav_row.append(&btn_home);

        // Fila 2: Cápsula de URL / Omnibox en el panel lateral (exacto como Arc en la imagen)
        let omnibox = gtk4::Entry::builder()
            .placeholder_text("Buscar o ingresar URL...")
            .hexpand(true)
            .halign(gtk4::Align::Fill)
            .primary_icon_name("security-high-symbolic")
            .primary_icon_tooltip_text("Permisos y seguridad del sitio (Clic para ver detalles)")
            .primary_icon_activatable(true)
            .secondary_icon_name("nika-copy-symbolic")
            .secondary_icon_tooltip_text("Copiar URL al portapapeles")
            .secondary_icon_activatable(true)
            .build();
        omnibox.add_css_class("omnibox");
        omnibox.add_css_class("arc-sidebar-omnibox");

        header_bar.append(&nav_row);
        header_bar.append(&omnibox);

        // Barra inferior de herramientas (ubicada al pie del panel lateral)
        let tools_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 2);
        tools_row.add_css_class("arc-sidebar-footer");

        let shield_btn = gtk4::Button::from_icon_name("nika-shield-symbolic");
        shield_btn.add_css_class("shield-pill");
        shield_btn.add_css_class("arc-tool-btn");
        shield_btn.set_tooltip_text(Some("Escudo de privacidad activo (Clic para detalles)"));

        let settings_btn = gtk4::Button::from_icon_name("nika-settings-symbolic");
        settings_btn.add_css_class("settings-btn");
        settings_btn.add_css_class("arc-tool-btn");
        settings_btn.set_tooltip_text(Some("Configuración (Ctrl+,) • DNS Seguro, Descargas y Red"));

        download_btn.add_css_class("arc-tool-btn");
        translate_btn.add_css_class("arc-tool-btn");

        // Reproductor multimedia compacto integrado
        let media_revealer = gtk4::Revealer::new();
        media_revealer.set_transition_type(gtk4::RevealerTransitionType::SlideRight);
        media_revealer.set_transition_duration(240);
        media_revealer.set_reveal_child(false);

        let media_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 2);
        media_box.add_css_class("header-media-box");
        media_box.add_css_class("arc-media-box");

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

        tools_row.append(&shield_btn);
        tools_row.append(download_btn);
        tools_row.append(translate_btn);
        tools_row.append(&settings_btn);

        let tools_spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        tools_spacer.set_hexpand(true);
        tools_row.append(&tools_spacer);
        tools_row.append(&media_revealer);

        Self {
            header_bar,
            sidebar_toggle_btn,
            btn_back,
            btn_forward,
            reload_btn,
            btn_home,
            omnibox,
            tools_row,
            media_revealer,
            btn_media_prev,
            btn_media_play,
            btn_media_next,
            shield_btn,
            settings_btn,
        }
    }

    pub fn set_media_playing_state(&self, playing: bool) {
        if playing {
            self.btn_media_play.set_icon_name("nika-pause-symbolic");
            self.btn_media_play.set_tooltip_text(Some("Pausar"));
        } else {
            self.btn_media_play.set_icon_name("nika-play-symbolic");
            self.btn_media_play.set_tooltip_text(Some("Reproducir"));
        }
    }

    pub fn is_media_playing(&self) -> bool {
        self.btn_media_play
            .icon_name()
            .map(|n| n == "nika-pause-symbolic")
            .unwrap_or(false)
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
