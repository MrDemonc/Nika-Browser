use gtk4::prelude::*;

pub struct ZoomToast {
    pub revealer: gtk4::Revealer,
    pub label: gtk4::Label,
}

impl ZoomToast {
    pub fn new() -> Self {
        let revealer = gtk4::Revealer::new();
        revealer.set_transition_type(gtk4::RevealerTransitionType::Crossfade);
        revealer.set_reveal_child(false);
        revealer.set_valign(gtk4::Align::Start);
        revealer.set_halign(gtk4::Align::Center);
        revealer.set_margin_top(16);

        let label = gtk4::Label::new(Some("100%"));
        label.add_css_class("zoom-toast-label");
        revealer.set_child(Some(&label));

        Self { revealer, label }
    }

    pub fn show(&self, zoom: f64) {
        let pct = (zoom * 100.0).round() as i32;
        self.label.set_text(&format!("{}%", pct));
        self.revealer.set_reveal_child(true);
        let rev = self.revealer.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(1200), move || {
            rev.set_reveal_child(false);
        });
    }
}
