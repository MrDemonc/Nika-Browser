use gtk4::prelude::*;

pub struct FindBar {
    pub revealer: gtk4::Revealer,
    pub entry: gtk4::SearchEntry,
    pub count_label: gtk4::Label,
    pub prev_btn: gtk4::Button,
    pub next_btn: gtk4::Button,
    pub close_btn: gtk4::Button,
}

impl FindBar {
    pub fn new() -> Self {
        let revealer = gtk4::Revealer::new();
        revealer.set_transition_type(gtk4::RevealerTransitionType::SlideDown);
        revealer.set_reveal_child(false);
        revealer.set_valign(gtk4::Align::Start);
        revealer.set_halign(gtk4::Align::End);
        revealer.set_margin_top(8);
        revealer.set_margin_end(16);

        let card = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
        card.add_css_class("floating-find-card");

        let entry = gtk4::SearchEntry::new();
        entry.add_css_class("find-entry");
        entry.set_placeholder_text(Some("Buscar en la página..."));
        entry.set_size_request(200, -1);

        let count_label = gtk4::Label::new(Some(""));
        count_label.add_css_class("find-count-label");

        let prev_btn = gtk4::Button::from_icon_name("go-up-symbolic");
        prev_btn.add_css_class("find-btn");
        prev_btn.set_tooltip_text(Some("Anterior (Shift+Enter)"));

        let next_btn = gtk4::Button::from_icon_name("go-down-symbolic");
        next_btn.add_css_class("find-btn");
        next_btn.set_tooltip_text(Some("Siguiente (Enter)"));

        let close_btn = gtk4::Button::from_icon_name("nika-close-symbolic");
        close_btn.add_css_class("find-btn");
        close_btn.set_tooltip_text(Some("Cerrar búsqueda (Esc)"));

        card.append(&entry);
        card.append(&count_label);
        card.append(&prev_btn);
        card.append(&next_btn);
        card.append(&close_btn);

        revealer.set_child(Some(&card));

        Self {
            revealer,
            entry,
            count_label,
            prev_btn,
            next_btn,
            close_btn,
        }
    }

    pub fn update_language(&self, lang: &str) {
        use crate::i18n::tr;
        self.entry.set_placeholder_text(Some(tr(lang, "find_placeholder")));
        self.prev_btn.set_tooltip_text(Some(tr(lang, "find_prev")));
        self.next_btn.set_tooltip_text(Some(tr(lang, "find_next")));
        self.close_btn.set_tooltip_text(Some(tr(lang, "find_close")));
    }
}
