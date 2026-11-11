use super::entry::{format_bytes, get_unique_download_path, DownloadEntry, DownloadState};
use crate::settings::BrowserSettings;
use gio::prelude::*;
use glib::clone;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use webkit6::Download;

pub struct DownloadManager {
    pub button: gtk4::Button,
    pub active_dot: gtk4::Label,
    pub badge_label: gtk4::Label,
    pub revealer: gtk4::Revealer,
    pub items_box: gtk4::Box,
    pub empty_label: gtk4::Label,
    pub title_label: gtk4::Label,
    pub open_folder_btn: gtk4::Button,
    pub close_btn: gtk4::Button,
    pub downloads: Rc<RefCell<Vec<Rc<RefCell<DownloadEntry>>>>>,
    pub settings: Rc<RefCell<BrowserSettings>>,
    next_id: RefCell<u64>,
    pub completed_count: Rc<RefCell<u32>>,
}

impl DownloadManager {
    pub fn new(settings: Rc<RefCell<BrowserSettings>>) -> Rc<Self> {
        let button = gtk4::Button::new();
        button.add_css_class("download-btn");
        button.set_tooltip_text(Some("Descargas (Ctrl+J)"));
        button.set_valign(gtk4::Align::Center);
        button.set_halign(gtk4::Align::Center);

        let btn_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        btn_box.set_valign(gtk4::Align::Center);
        btn_box.set_halign(gtk4::Align::Center);

        let icon_img = gtk4::Image::from_icon_name("nika-download-symbolic");
        icon_img.set_pixel_size(14);

        let active_dot = gtk4::Label::new(Some("●"));
        active_dot.add_css_class("download-active-dot");
        active_dot.set_valign(gtk4::Align::Center);
        active_dot.set_halign(gtk4::Align::Center);
        active_dot.set_visible(false);

        let badge_label = gtk4::Label::new(None);
        badge_label.add_css_class("download-badge");
        badge_label.set_visible(false);

        btn_box.append(&icon_img);
        btn_box.append(&active_dot);
        btn_box.append(&badge_label);
        button.set_child(Some(&btn_box));

        // Panel lateral flotante de descargas
        let revealer = gtk4::Revealer::new();
        revealer.set_transition_type(gtk4::RevealerTransitionType::SlideLeft);
        revealer.set_transition_duration(280);
        revealer.set_reveal_child(false);
        revealer.set_hexpand(false);
        revealer.set_halign(gtk4::Align::End);
        revealer.set_valign(gtk4::Align::Fill);

        let card = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        card.add_css_class("floating-sidebar");
        card.add_css_class("floating-downloads-panel");
        card.set_hexpand(false);
        card.set_size_request(260, -1);

        // Cabecera del panel de descargas
        let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 2);
        header.add_css_class("sidebar-header");

        let title = gtk4::Label::new(Some("Descargas"));
        title.add_css_class("sidebar-title");
        title.set_hexpand(true);
        title.set_halign(gtk4::Align::Start);

        let open_folder_btn = gtk4::Button::from_icon_name("nika-folder-symbolic");
        open_folder_btn.add_css_class("sidebar-action-btn");
        open_folder_btn.set_tooltip_text(Some("Abrir carpeta de descargas"));

        let s_folder = settings.clone();
        open_folder_btn.connect_clicked(move |_| {
            let dl_dir = s_folder.borrow().get_download_dir();
            if let Ok(uri) = glib::filename_to_uri(&dl_dir, None) {
                let _ = gio::AppInfo::launch_default_for_uri(&uri, None::<&gio::AppLaunchContext>);
            }
        });

        let close_btn = gtk4::Button::from_icon_name("nika-close-symbolic");
        close_btn.add_css_class("sidebar-action-btn");
        close_btn.set_tooltip_text(Some("Cerrar panel de descargas (Ctrl+J)"));

        header.append(&title);
        header.append(&open_folder_btn);
        header.append(&close_btn);

        let scroll = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .propagate_natural_height(true)
            .vexpand(true)
            .hexpand(false)
            .build();
        scroll.add_css_class("tabs-scroller");

        let items_box = gtk4::Box::new(gtk4::Orientation::Vertical, 3);
        items_box.add_css_class("vertical-tabs-box");
        items_box.set_hexpand(false);
        items_box.set_size_request(248, -1);

        let empty_label = gtk4::Label::new(Some("No hay descargas"));
        empty_label.add_css_class("downloads-empty-label");
        empty_label.set_vexpand(true);
        empty_label.set_valign(gtk4::Align::Center);
        items_box.append(&empty_label);

        scroll.set_child(Some(&items_box));
        card.append(&header);
        card.append(&scroll);
        revealer.set_child(Some(&card));

        let completed_count = Rc::new(RefCell::new(0));

        let mgr = Rc::new(Self {
            button: button.clone(),
            active_dot,
            badge_label,
            revealer: revealer.clone(),
            items_box,
            empty_label,
            title_label: title,
            open_folder_btn,
            close_btn: close_btn.clone(),
            downloads: Rc::new(RefCell::new(Vec::new())),
            settings,
            next_id: RefCell::new(1),
            completed_count,
        });

        let mgr_weak = Rc::downgrade(&mgr);
        button.connect_clicked(clone!(
            #[weak]
            revealer,
            move |_| {
                let cur = revealer.reveals_child();
                revealer.set_reveal_child(!cur);
                if !cur {
                    if let Some(m) = mgr_weak.upgrade() {
                        m.clear_badge();
                    }
                }
            }
        ));

        close_btn.connect_clicked(clone!(
            #[weak]
            revealer,
            move |_| {
                revealer.set_reveal_child(false);
            }
        ));

        mgr
    }

    /// Limpia el número badge de descargas completadas
    pub fn clear_badge(&self) {
        *self.completed_count.borrow_mut() = 0;
        self.badge_label.set_visible(false);
    }

    /// Actualiza el estado visual del botón según si hay descargas activas (iluminado fijo y punto indicador)
    pub fn update_active_state(&self) {
        let any_in_progress = self.downloads.borrow().iter().any(|e| {
            if let Ok(entry) = e.try_borrow() {
                entry.state == DownloadState::InProgress
            } else {
                false
            }
        });

        let lang = self.settings.borrow().language.clone();
        if any_in_progress {
            self.button.add_css_class("download-active");
            self.active_dot.set_visible(true);
            self.button.set_tooltip_text(Some(crate::i18n::tr(&lang, "download_in_progress")));
        } else {
            self.button.remove_css_class("download-active");
            self.active_dot.set_visible(false);
            self.button.set_tooltip_text(Some(crate::i18n::tr(&lang, "nav_downloads")));
        }
    }

    /// Incrementa el contador badge y envía notificación de escritorio del sistema
    pub fn notify_completed(&self, filename: &str) {
        *self.completed_count.borrow_mut() += 1;
        let count = *self.completed_count.borrow();
        self.badge_label.set_text(&count.to_string());
        self.badge_label.set_visible(true);

        let lang = self.settings.borrow().language.clone();
        let title = crate::i18n::tr(&lang, "download_completed");
        let prefix = crate::i18n::tr(&lang, "download_completed_body");
        let body = format!("{}{}", prefix, filename);

        let gnotif = gio::Notification::new(title);
        gnotif.set_body(Some(&body));
        if let Some(app) = gio::Application::default() {
            app.send_notification(None, &gnotif);
        }
    }

    /// Actualiza textos y tooltips del gestor de descargas según el idioma
    pub fn update_language(&self, lang: &str) {
        use crate::i18n::tr;
        self.button.set_tooltip_text(Some(tr(lang, "nav_downloads")));
        self.title_label.set_text(tr(lang, "downloads_title"));
        self.empty_label.set_text(tr(lang, "downloads_empty"));
        self.open_folder_btn.set_tooltip_text(Some(tr(lang, "open_folder")));
        self.close_btn.set_tooltip_text(Some(tr(lang, "close")));
    }

    pub fn handle_new_download(self: &Rc<Self>, download: &Download) {
        let mgr = self.clone();

        download.connect_decide_destination(clone!(
            #[strong]
            mgr,
            move |d, suggested_name| {
                let dl_dir = mgr.settings.borrow().get_download_dir();
                let _ = std::fs::create_dir_all(&dl_dir);

                let target_path = get_unique_download_path(&dl_dir, suggested_name);
                let final_name = target_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(suggested_name)
                    .to_string();

                let path_str = target_path.to_string_lossy();
                d.set_destination(&path_str);
                d.set_allow_overwrite(true);

                mgr.add_download_item(d, &final_name, target_path);
                true
            }
        ));
    }

    fn add_download_item(self: &Rc<Self>, download: &Download, filename: &str, path: PathBuf) {
        let id = {
            let mut next = self.next_id.borrow_mut();
            let current = *next;
            *next += 1;
            current
        };

        let entry = Rc::new(RefCell::new(DownloadEntry {
            id,
            filename: filename.to_string(),
            path: path.clone(),
            progress: 0.0,
            state: DownloadState::InProgress,
            download: download.clone(),
        }));

        self.downloads.borrow_mut().push(entry.clone());
        self.empty_label.set_visible(false);
        self.update_active_state();

        let row = gtk4::Box::new(gtk4::Orientation::Vertical, 3);
        row.add_css_class("tab-pill");
        row.add_css_class("download-pill");

        let top_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);

        let file_icon = gtk4::Image::from_icon_name("nika-download-symbolic");
        file_icon.add_css_class("tab-icon");

        let name_label = gtk4::Label::new(Some(filename));
        name_label.add_css_class("tab-title");
        name_label.set_hexpand(true);
        name_label.set_halign(gtk4::Align::Start);
        name_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        name_label.set_max_width_chars(18);

        let cancel_btn = gtk4::Button::from_icon_name("nika-close-symbolic");
        cancel_btn.add_css_class("tab-close-btn");
        cancel_btn.set_tooltip_text(Some("Cancelar descarga"));

        let open_btn = gtk4::Button::from_icon_name("nika-folder-symbolic");
        open_btn.add_css_class("tab-close-btn");
        open_btn.set_tooltip_text(Some("Abrir archivo"));
        open_btn.set_visible(false);

        let delete_btn = gtk4::Button::from_icon_name("nika-close-symbolic");
        delete_btn.add_css_class("tab-close-btn");
        delete_btn.set_tooltip_text(Some("Eliminar de la lista"));
        delete_btn.set_visible(false);

        top_row.append(&file_icon);
        top_row.append(&name_label);
        top_row.append(&open_btn);
        top_row.append(&cancel_btn);
        top_row.append(&delete_btn);

        let status_label = gtk4::Label::new(Some("Iniciando descarga..."));
        status_label.add_css_class("download-status-label");
        status_label.set_halign(gtk4::Align::Start);
        status_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        status_label.set_max_width_chars(28);

        let progress_bar = gtk4::ProgressBar::new();
        progress_bar.add_css_class("download-progress-bar");
        progress_bar.set_fraction(0.0);

        row.append(&top_row);
        row.append(&status_label);
        row.append(&progress_bar);

        let del_btn_for_cancel = delete_btn.clone();
        let mgr_cancel = Rc::downgrade(self);
        cancel_btn.connect_clicked(clone!(
            #[strong]
            entry,
            #[weak]
            status_label,
            #[weak]
            progress_bar,
            move |btn| {
                let dl = if let Ok(mut e) = entry.try_borrow_mut() {
                    e.state = DownloadState::Cancelled;
                    Some(e.download.clone())
                } else {
                    None
                };
                if let Some(dl) = dl {
                    dl.cancel();
                }
                status_label.set_text("Cancelado");
                progress_bar.set_visible(false);
                btn.set_visible(false);
                del_btn_for_cancel.set_visible(true);
                if let Some(m) = mgr_cancel.upgrade() {
                    m.update_active_state();
                }
            }
        ));

        let dls_for_delete = self.downloads.clone();
        let ibox_for_delete = self.items_box.clone();
        let empty_lbl_for_delete = self.empty_label.clone();
        let mgr_delete = Rc::downgrade(self);
        delete_btn.connect_clicked(clone!(
            #[weak]
            row,
            move |_| {
                ibox_for_delete.remove(&row);
                dls_for_delete.borrow_mut().retain(|d| {
                    if let Ok(e) = d.try_borrow() {
                        e.id != id
                    } else {
                        true
                    }
                });
                if dls_for_delete.borrow().is_empty() {
                    empty_lbl_for_delete.set_visible(true);
                }
                if let Some(m) = mgr_delete.upgrade() {
                    m.update_active_state();
                }
            }
        ));

        let path_clone = path.clone();
        open_btn.connect_clicked(move |_| {
            if let Ok(uri) = glib::filename_to_uri(&path_clone, None) {
                let _ = gio::AppInfo::launch_default_for_uri(&uri, None::<&gio::AppLaunchContext>);
            }
        });

        self.items_box.prepend(&row);

        let last_progress_update = Rc::new(RefCell::new(std::time::Instant::now()));
        download.connect_estimated_progress_notify(clone!(
            #[strong]
            entry,
            #[weak]
            progress_bar,
            #[weak]
            status_label,
            #[strong]
            last_progress_update,
            move |dl| {
                let now = std::time::Instant::now();
                if now.duration_since(*last_progress_update.borrow()) < std::time::Duration::from_millis(150) {
                    return;
                }
                *last_progress_update.borrow_mut() = now;

                let frac = dl.estimated_progress();
                if let Ok(mut e) = entry.try_borrow_mut() {
                    e.progress = frac;
                }
                progress_bar.set_fraction(frac);
                let pct = (frac * 100.0) as u32;
                let bytes = dl.received_data_length();
                let size_str = format_bytes(bytes);
                if pct > 0 && pct < 100 {
                    status_label.set_text(&format!("{} • {}%", size_str, pct));
                } else if pct >= 100 {
                    status_label.set_text(&format!("{} • 100%", size_str));
                } else {
                    status_label.set_text(&format!("Descargando... {}", size_str));
                }
            }
        ));

        let del_btn_finish = delete_btn.clone();
        let mgr_finish = Rc::downgrade(self);
        download.connect_finished(clone!(
            #[strong]
            entry,
            #[weak]
            progress_bar,
            #[weak]
            status_label,
            #[weak]
            cancel_btn,
            #[weak]
            open_btn,
            #[weak]
            del_btn_finish,
            move |dl| {
                let is_cancelled_or_failed = if let Ok(e) = entry.try_borrow() {
                    e.state == DownloadState::Cancelled || matches!(e.state, DownloadState::Failed(_))
                } else {
                    false
                };

                if !is_cancelled_or_failed {
                    if let Ok(mut e) = entry.try_borrow_mut() {
                        e.state = DownloadState::Completed;
                    }
                    progress_bar.set_fraction(1.0);
                    progress_bar.set_visible(false);
                    let bytes = dl.received_data_length();
                    let lang = if let Some(m) = mgr_finish.upgrade() {
                        m.settings.borrow().language.clone()
                    } else {
                        "es".to_string()
                    };
                    let completed_text = crate::i18n::tr(&lang, "completed");
                    status_label.set_text(&format!("{} ({})", completed_text, format_bytes(bytes)));
                    cancel_btn.set_visible(false);
                    open_btn.set_visible(true);
                    del_btn_finish.set_visible(true);

                    let fname = if let Ok(e) = entry.try_borrow() {
                        e.filename.clone()
                    } else {
                        "archivo".to_string()
                    };
                    if let Some(m) = mgr_finish.upgrade() {
                        m.notify_completed(&fname);
                    }
                }

                if let Some(m) = mgr_finish.upgrade() {
                    m.update_active_state();
                }
            }
        ));

        let del_btn_fail = delete_btn;
        let mgr_fail = Rc::downgrade(self);
        download.connect_failed(clone!(
            #[strong]
            entry,
            #[weak]
            progress_bar,
            #[weak]
            status_label,
            #[weak]
            cancel_btn,
            move |_, err| {
                if let Ok(mut e) = entry.try_borrow_mut() {
                    e.state = DownloadState::Failed(err.to_string());
                }
                progress_bar.set_visible(false);
                status_label.set_text(&format!("Error: {}", err));
                cancel_btn.set_visible(false);
                del_btn_fail.set_visible(true);
                if let Some(m) = mgr_fail.upgrade() {
                    m.update_active_state();
                }
            }
        ));
    }
}
