use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::browser::permissions::SitePermissionsManager;
use crate::i18n::tr;

pub struct PermissionsDialog {
    parent_window: adw::ApplicationWindow,
    manager: SitePermissionsManager,
    current_url: Rc<RefCell<String>>,
    current_host: Rc<RefCell<String>>,
    current_lang: Rc<RefCell<String>>,
    clear_callback: Rc<RefCell<Option<Box<dyn Fn(&str)>>>>,
    active_dialog: Rc<RefCell<Option<adw::Dialog>>>,
}

pub type PermissionsPopover = PermissionsDialog;

impl PermissionsDialog {
    pub fn new(parent_window: &adw::ApplicationWindow, manager: SitePermissionsManager) -> Self {
        Self {
            parent_window: parent_window.clone(),
            manager,
            current_url: Rc::new(RefCell::new(String::new())),
            current_host: Rc::new(RefCell::new(String::new())),
            current_lang: Rc::new(RefCell::new("es".to_string())),
            clear_callback: Rc::new(RefCell::new(None)),
            active_dialog: Rc::new(RefCell::new(None)),
        }
    }

    pub fn set_on_clear_data<F: Fn(&str) + 'static>(&self, f: F) {
        *self.clear_callback.borrow_mut() = Some(Box::new(f));
    }

    /// Actualiza la URL e idioma activos
    pub fn update_for_url(&self, url: &str, lang: &str) {
        *self.current_url.borrow_mut() = url.to_string();
        let host = SitePermissionsManager::extract_host(url);
        *self.current_host.borrow_mut() = host;
        *self.current_lang.borrow_mut() = lang.to_string();
    }

    /// Abre la ventana centrada con el botón X de cerrar
    pub fn popup(&self) {
        self.present();
    }

    pub fn present(&self) {
        if let Some(old) = self.active_dialog.borrow_mut().take() {
            if old.parent().is_some() {
                old.close();
            }
        }

        let url = self.current_url.borrow().clone();
        let host = self.current_host.borrow().clone();
        let lang = self.current_lang.borrow().clone();
        let is_en = lang == "en";
        let is_https = url.starts_with("https://");
        let is_http = url.starts_with("http://");
        let is_internal = !is_https && !is_http;

        let dialog = adw::Dialog::builder()
            .title(if is_en { "Site Permissions" } else { "Permisos del sitio" })
            .content_width(320)
            .follows_content_size(true)
            .presentation_mode(adw::DialogPresentationMode::Floating)
            .can_close(true)
            .build();
        dialog.add_css_class("permissions-dialog");

        // Barra superior de la ventana con título y botón 'X' para cerrar
        let header_bar = adw::HeaderBar::new();
        header_bar.add_css_class("flat");
        header_bar.add_css_class("perm-header-bar");
        header_bar.set_show_end_title_buttons(false);
        header_bar.set_show_start_title_buttons(false);

        let title_label = gtk4::Label::new(Some(if is_en { "Site Permissions" } else { "Permisos del sitio" }));
        title_label.add_css_class("perm-dialog-title");
        header_bar.set_title_widget(Some(&title_label));

        let btn_close = gtk4::Button::from_icon_name("window-close-symbolic");
        btn_close.add_css_class("flat");
        btn_close.add_css_class("circular");
        btn_close.add_css_class("perm-close-btn");
        btn_close.set_tooltip_text(Some(if is_en { "Close" } else { "Cerrar" }));

        let d_close = dialog.clone();
        btn_close.connect_clicked(move |_| {
            d_close.close();
        });
        header_bar.pack_end(&btn_close);

        // Contenedor principal del contenido
        let content_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        content_box.set_margin_top(4);
        content_box.set_margin_bottom(12);
        content_box.set_margin_start(16);
        content_box.set_margin_end(16);

        // 1. Tarjeta con estado de seguridad y host (SIN el texto "Nika Browser")
        let host_card = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        host_card.add_css_class("perm-host-card");
        host_card.set_valign(gtk4::Align::Center);

        let icon_name = if is_https {
            "security-high-symbolic"
        } else if is_http {
            "dialog-warning-symbolic"
        } else {
            "security-high-symbolic"
        };
        let icon = gtk4::Image::from_icon_name(icon_name);
        icon.set_icon_size(gtk4::IconSize::Normal);
        if is_https {
            icon.add_css_class("perm-icon-secure");
        } else if is_http {
            icon.add_css_class("perm-icon-warning");
        }
        host_card.append(&icon);

        let host_display = if is_internal {
            if is_en { "System Page" } else { "Página del sistema" }
        } else {
            &host
        };
        let host_label = gtk4::Label::new(Some(host_display));
        host_label.add_css_class("perm-host-title");
        host_label.set_halign(gtk4::Align::Start);
        host_label.set_hexpand(true);
        host_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        host_card.append(&host_label);

        let (badge_text, badge_class) = if is_https {
            (tr(&lang, "perm_badge_secure"), "perm-badge-secure")
        } else if is_http {
            (tr(&lang, "perm_badge_insecure"), "perm-badge-warning")
        } else {
            (tr(&lang, "perm_badge_internal"), "perm-badge-sys")
        };
        let badge_label = gtk4::Label::new(Some(badge_text));
        badge_label.add_css_class("perm-badge");
        badge_label.add_css_class(badge_class);
        host_card.append(&badge_label);

        content_box.append(&host_card);

        // Separador sutil
        let sep1 = gtk4::Separator::new(gtk4::Orientation::Horizontal);
        sep1.add_css_class("perm-separator");
        content_box.append(&sep1);

        // 2. Encabezado de sección
        let sec_label = gtk4::Label::new(Some(tr(&lang, "site_permissions")));
        sec_label.add_css_class("perm-section-heading");
        sec_label.set_halign(gtk4::Align::Start);
        content_box.append(&sec_label);

        // 3. Filas de permisos compactas y minimalistas
        let current_perms = self.manager.get_permissions(&host);

        let rows_data = [
            (
                "preferences-system-notifications-symbolic",
                tr(&lang, "perm_notifications"),
                current_perms.notifications,
                0,
            ),
            (
                "audio-input-microphone-symbolic",
                tr(&lang, "perm_mic"),
                current_perms.microphone,
                1,
            ),
            (
                "camera-video-symbolic",
                tr(&lang, "perm_camera"),
                current_perms.camera,
                2,
            ),
            (
                "find-location-symbolic",
                tr(&lang, "perm_location"),
                current_perms.geolocation,
                3,
            ),
            (
                "edit-paste-symbolic",
                tr(&lang, "perm_clipboard"),
                current_perms.clipboard,
                4,
            ),
            (
                "drive-harddisk-symbolic",
                tr(&lang, "perm_storage"),
                current_perms.storage,
                5,
            ),
        ];

        for (icon_name, title, state, perm_id) in rows_data {
            let row_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
            row_box.add_css_class("perm-row");

            let img = gtk4::Image::from_icon_name(icon_name);
            img.set_icon_size(gtk4::IconSize::Normal);
            img.add_css_class("perm-row-icon");
            row_box.append(&img);

            let t_label = gtk4::Label::new(Some(title));
            t_label.add_css_class("perm-row-title");
            t_label.set_halign(gtk4::Align::Start);
            t_label.set_hexpand(true);
            row_box.append(&t_label);

            let sw = gtk4::Switch::new();
            sw.set_active(state);
            sw.set_valign(gtk4::Align::Center);
            sw.add_css_class("perm-switch");

            let mgr = self.manager.clone();
            let h = host.clone();
            sw.connect_state_set(move |_, new_state| {
                let mut p = mgr.get_permissions(&h);
                match perm_id {
                    0 => p.notifications = new_state,
                    1 => p.microphone = new_state,
                    2 => p.camera = new_state,
                    3 => p.geolocation = new_state,
                    4 => p.clipboard = new_state,
                    5 => p.storage = new_state,
                    _ => {}
                }
                mgr.set_permissions(&h, p);
                glib::Propagation::Proceed
            });

            row_box.append(&sw);
            content_box.append(&row_box);
        }

        // Separador sutil
        let sep2 = gtk4::Separator::new(gtk4::Orientation::Horizontal);
        sep2.add_css_class("perm-separator");
        content_box.append(&sep2);

        // 4. Botones de acción compactos
        let actions_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        actions_box.set_margin_top(4);

        let btn_reset = gtk4::Button::builder()
            .label(tr(&lang, "perm_btn_reset"))
            .icon_name("view-refresh-symbolic")
            .hexpand(true)
            .build();
        btn_reset.add_css_class("perm-action-btn");

        let mgr_reset = self.manager.clone();
        let h_reset = host.clone();
        let d_reset = dialog.clone();
        btn_reset.connect_clicked(move |_| {
            mgr_reset.reset_permissions(&h_reset);
            d_reset.close();
        });

        let btn_clear = gtk4::Button::builder()
            .label(tr(&lang, "perm_btn_clear"))
            .icon_name("user-trash-symbolic")
            .hexpand(true)
            .build();
        btn_clear.add_css_class("perm-action-btn");
        btn_clear.add_css_class("perm-btn-danger");

        let cb_ref = self.clear_callback.clone();
        let h_clear = host.clone();
        let btn_clear_clone = btn_clear.clone();
        let lang_clear = lang.clone();
        btn_clear.connect_clicked(move |_| {
            if let Some(cb) = cb_ref.borrow().as_ref() {
                cb(&h_clear);
            }
            btn_clear_clone.set_label(tr(&lang_clear, "perm_cleared"));
        });

        actions_box.append(&btn_reset);
        actions_box.append(&btn_clear);
        content_box.append(&actions_box);

        // Estructura con ToolbarView
        let toolbar_view = adw::ToolbarView::new();
        toolbar_view.add_top_bar(&header_bar);
        toolbar_view.set_content(Some(&content_box));

        dialog.set_child(Some(&toolbar_view));

        let active_ref = self.active_dialog.clone();
        dialog.connect_closed(move |_| {
            *active_ref.borrow_mut() = None;
        });

        dialog.present(Some(&self.parent_window));

        *self.active_dialog.borrow_mut() = Some(dialog);
    }
}
