#![allow(dead_code)]
use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

/// Widget de control y visualización de reproducción multimedia en la barra de pestañas lateral
pub struct SidebarMediaPlayer {
    pub container: gtk4::Revealer,
    pub title_label: gtk4::Label,
    pub artist_label: gtk4::Label,
    pub status_label: gtk4::Label,
    pub play_btn: gtk4::Button,
    pub prev_btn: gtk4::Button,
    pub next_btn: gtk4::Button,
    pub mute_btn: gtk4::Button,
    pub dismiss_btn: gtk4::Button,
    pub info_box: gtk4::Box,
    pub is_playing: Rc<RefCell<bool>>,
    pub is_muted: Rc<RefCell<bool>>,
    current_lang: Rc<RefCell<String>>,
}

impl SidebarMediaPlayer {
    pub fn new(initial_lang: &str) -> Self {
        let container = gtk4::Revealer::builder()
            .transition_type(gtk4::RevealerTransitionType::SlideUp)
            .transition_duration(220)
            .reveal_child(false)
            .build();
        container.add_css_class("sidebar-media-container");

        let card = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        card.add_css_class("sidebar-media-card");

        // Fila de encabezado: Icono de música + estado + botón cerrar
        let header_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
        header_row.add_css_class("media-header-row");

        let badge_icon = gtk4::Image::from_icon_name("nika-music-symbolic");
        badge_icon.add_css_class("media-badge-icon");

        let is_en = initial_lang == "en";
        let status_label = gtk4::Label::new(Some(if is_en { "NOW PLAYING" } else { "REPRODUCIENDO" }));
        status_label.add_css_class("media-badge-title");
        status_label.set_hexpand(true);
        status_label.set_xalign(0.0);

        let dismiss_btn = gtk4::Button::from_icon_name("nika-close-symbolic");
        dismiss_btn.add_css_class("media-dismiss-btn");
        dismiss_btn.set_tooltip_text(Some(if is_en { "Hide player" } else { "Ocultar reproductor" }));

        header_row.append(&badge_icon);
        header_row.append(&status_label);
        header_row.append(&dismiss_btn);

        // Caja de título y artista / sitio (clicable para ir a la pestaña)
        let info_box = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
        info_box.add_css_class("media-info-box");
        info_box.set_tooltip_text(Some(if is_en { "Click to switch to tab" } else { "Clic para ir a la pestaña" }));

        let title_label = gtk4::Label::new(Some(if is_en { "Media Track" } else { "Pista multimedia" }));
        title_label.add_css_class("media-track-title");
        title_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        title_label.set_xalign(0.0);
        title_label.set_lines(1);

        let artist_label = gtk4::Label::new(Some("Web Media"));
        artist_label.add_css_class("media-track-artist");
        artist_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        artist_label.set_xalign(0.0);
        artist_label.set_lines(1);

        info_box.append(&title_label);
        info_box.append(&artist_label);

        // Fila de controles de reproducción
        let controls_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        controls_row.add_css_class("media-controls-row");
        controls_row.set_halign(gtk4::Align::Center);

        let prev_btn = gtk4::Button::from_icon_name("nika-prev-symbolic");
        prev_btn.add_css_class("media-control-btn");
        prev_btn.set_tooltip_text(Some(if is_en { "Rewind 10s / Previous" } else { "Retroceder 10s / Anterior" }));

        let play_btn = gtk4::Button::from_icon_name("nika-pause-symbolic");
        play_btn.add_css_class("media-play-btn");
        play_btn.set_tooltip_text(Some(if is_en { "Pause" } else { "Pausar" }));

        let next_btn = gtk4::Button::from_icon_name("nika-next-symbolic");
        next_btn.add_css_class("media-control-btn");
        next_btn.set_tooltip_text(Some(if is_en { "Fast-forward 10s / Next" } else { "Adelantar 10s / Siguiente" }));

        let mute_btn = gtk4::Button::from_icon_name("nika-volume-symbolic");
        mute_btn.add_css_class("media-control-btn");
        mute_btn.set_tooltip_text(Some(if is_en { "Mute" } else { "Silenciar" }));

        controls_row.append(&prev_btn);
        controls_row.append(&play_btn);
        controls_row.append(&next_btn);
        controls_row.append(&mute_btn);

        card.append(&header_row);
        card.append(&info_box);
        card.append(&controls_row);

        container.set_child(Some(&card));

        let is_playing = Rc::new(RefCell::new(true));
        let is_muted = Rc::new(RefCell::new(false));
        let current_lang = Rc::new(RefCell::new(initial_lang.to_string()));

        // Conectar dismiss btn por defecto para ocultar la tarjeta
        let c_clone = container.clone();
        dismiss_btn.connect_clicked(move |_| {
            c_clone.set_reveal_child(false);
        });

        Self {
            container,
            title_label,
            artist_label,
            status_label,
            play_btn,
            prev_btn,
            next_btn,
            mute_btn,
            dismiss_btn,
            info_box,
            is_playing,
            is_muted,
            current_lang,
        }
    }

    pub fn set_info(&self, title: &str, artist: &str, is_paused: bool, muted: bool) {
        if !title.is_empty() {
            self.title_label.set_text(title);
            self.title_label.set_tooltip_text(Some(title));
        }
        if !artist.is_empty() {
            self.artist_label.set_text(artist);
        }
        self.set_playing_state(!is_paused);
        self.set_muted_state(muted);
    }

    pub fn set_playing_state(&self, playing: bool) {
        *self.is_playing.borrow_mut() = playing;
        let is_en = *self.current_lang.borrow() == "en";
        if playing {
            self.play_btn.set_icon_name("nika-pause-symbolic");
            self.play_btn.set_tooltip_text(Some(if is_en { "Pause" } else { "Pausar" }));
            self.status_label.set_text(if is_en { "NOW PLAYING" } else { "REPRODUCIENDO" });
        } else {
            self.play_btn.set_icon_name("nika-play-symbolic");
            self.play_btn.set_tooltip_text(Some(if is_en { "Play" } else { "Reproducir" }));
            self.status_label.set_text(if is_en { "PAUSED" } else { "PAUSADO" });
        }
    }

    pub fn set_muted_state(&self, muted: bool) {
        *self.is_muted.borrow_mut() = muted;
        let is_en = *self.current_lang.borrow() == "en";
        if muted {
            self.mute_btn.set_icon_name("nika-volume-mute-symbolic");
            self.mute_btn.set_tooltip_text(Some(if is_en { "Unmute" } else { "Reactivar sonido" }));
        } else {
            self.mute_btn.set_icon_name("nika-volume-symbolic");
            self.mute_btn.set_tooltip_text(Some(if is_en { "Mute" } else { "Silenciar" }));
        }
    }

    pub fn show(&self) {
        self.container.set_reveal_child(true);
    }

    pub fn hide(&self) {
        self.container.set_reveal_child(false);
    }

    #[allow(dead_code)]
    pub fn is_revealed(&self) -> bool {
        self.container.reveals_child()
    }

    pub fn update_language(&self, lang: &str) {
        *self.current_lang.borrow_mut() = lang.to_string();
        let is_en = lang == "en";
        let playing = *self.is_playing.borrow();
        if playing {
            self.status_label.set_text(if is_en { "NOW PLAYING" } else { "REPRODUCIENDO" });
            self.play_btn.set_tooltip_text(Some(if is_en { "Pause" } else { "Pausar" }));
        } else {
            self.status_label.set_text(if is_en { "PAUSED" } else { "PAUSADO" });
            self.play_btn.set_tooltip_text(Some(if is_en { "Play" } else { "Reproducir" }));
        }
        self.prev_btn.set_tooltip_text(Some(if is_en { "Rewind 10s / Previous" } else { "Retroceder 10s / Anterior" }));
        self.next_btn.set_tooltip_text(Some(if is_en { "Fast-forward 10s / Next" } else { "Adelantar 10s / Siguiente" }));
        self.dismiss_btn.set_tooltip_text(Some(if is_en { "Hide player" } else { "Ocultar reproductor" }));
        self.info_box.set_tooltip_text(Some(if is_en { "Click to switch to tab" } else { "Clic para ir a la pestaña" }));

        let muted = *self.is_muted.borrow();
        if muted {
            self.mute_btn.set_tooltip_text(Some(if is_en { "Unmute" } else { "Reactivar sonido" }));
        } else {
            self.mute_btn.set_tooltip_text(Some(if is_en { "Mute" } else { "Silenciar" }));
        }
    }
}
