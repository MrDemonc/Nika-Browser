use crate::browser::permissions::SitePermissionsManager;
use crate::browser::{save_session_data, BrowserTab, SessionState};
use crate::downloads::DownloadManager;
use crate::passwords::PasswordVault;
use crate::privacy::PrivacyEngine;
use crate::search::resolve_query_or_url;
use crate::settings::{BrowserSettings, DnsProvider};
use crate::theme::{ThemeManager, ThemeMode};
use crate::ui::permissions_popover::PermissionsPopover;
use crate::ui::{FindBar, HeaderComponents, SidebarComponents, ZoomToast};
use glib::clone;
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use webkit6::prelude::*;
use webkit6::UserContentManager;

/// Estructura principal del navegador Nika
pub struct BrowserApp {
    pub window: adw::ApplicationWindow,
    pub header: HeaderComponents,
    pub sidebar: SidebarComponents,
    pub find_bar: FindBar,
    pub zoom_toast: ZoomToast,
    pub progress_bar: gtk4::ProgressBar,
    pub tab_view: adw::TabView,
    pub tabs: Rc<RefCell<Vec<BrowserTab>>>,
    pub theme_manager: Rc<RefCell<ThemeManager>>,
    pub download_manager: Rc<DownloadManager>,
    pub settings: Rc<RefCell<BrowserSettings>>,
    pub content_manager: UserContentManager,
    #[allow(dead_code)]
    pub password_vault: Rc<PasswordVault>,
    pub persistent_session: webkit6::NetworkSession,
    pub home_url: String,
    pub site_permissions: SitePermissionsManager,
    pub permissions_popover: Rc<PermissionsPopover>,
}

impl BrowserApp {
    pub fn new(app: &adw::Application, project_dir: PathBuf) -> Rc<Self> {
        let content_manager = UserContentManager::new();
        let rules_path = project_dir.join("resources").join("rules").join("privacy-rules.json");
        PrivacyEngine::apply_content_filters(&content_manager, &rules_path);

        let settings = Rc::new(RefCell::new(BrowserSettings::load()));
        settings.borrow().apply_dns();

        let data_dir = glib::user_data_dir().join("nika-browser");
        let cache_dir = glib::user_cache_dir().join("nika-browser");
        let _ = std::fs::create_dir_all(&data_dir);
        let _ = std::fs::create_dir_all(&cache_dir);

        let persistent_session = webkit6::NetworkSession::new(
            data_dir.to_str(),
            cache_dir.to_str(),
        );

        if let Some(cm) = persistent_session.cookie_manager() {
            let cookie_path = data_dir.join("cookies.sqlite");
            cm.set_persistent_storage(
                cookie_path.to_str().unwrap_or("/tmp/nika_cookies.sqlite"),
                webkit6::CookiePersistentStorage::Sqlite,
            );
            cm.set_accept_policy(webkit6::CookieAcceptPolicy::Always);
        }

        persistent_session.set_persistent_credential_storage_enabled(true);
        persistent_session.set_itp_enabled(false);

        if let Some(def_session) = webkit6::NetworkSession::default() {
            if let Some(cm) = def_session.cookie_manager() {
                let cookie_path = data_dir.join("cookies.sqlite");
                cm.set_persistent_storage(
                    cookie_path.to_str().unwrap_or("/tmp/nika_cookies.sqlite"),
                    webkit6::CookiePersistentStorage::Sqlite,
                );
                cm.set_accept_policy(webkit6::CookieAcceptPolicy::Always);
            }
            def_session.set_persistent_credential_storage_enabled(true);
            def_session.set_itp_enabled(false);
        }

        let home_url = "nika://home".to_string();

        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("Nika Browser")
            .default_width(1150)
            .default_height(780)
            .build();
        window.set_icon_name(Some("nika-browser"));
        window.add_css_class("nika-window");
        window.add_css_class("dark-theme");

        let download_manager = DownloadManager::new(settings.clone());
        let header = HeaderComponents::new(&download_manager.button);
        let sidebar = SidebarComponents::new(settings.clone());
        let find_bar = FindBar::new();
        let zoom_toast = ZoomToast::new();

        let progress_bar = gtk4::ProgressBar::new();
        progress_bar.add_css_class("pastel-progress");
        progress_bar.set_visible(false);

        let tab_view = adw::TabView::new();
        tab_view.set_vexpand(true);
        tab_view.set_hexpand(true);

        // Overlay principal sobre la vista web:
        // Contiene la vista de pestañas como base, y como capas flotantes:
        // gestor de descargas, barra de búsqueda en página y notificación de zoom.
        let overlay = gtk4::Overlay::new();
        overlay.add_css_class("nika-overlay");
        overlay.set_vexpand(true);
        overlay.set_hexpand(true);
        overlay.set_child(Some(&tab_view));

        overlay.add_overlay(&download_manager.revealer);
        overlay.add_overlay(&find_bar.revealer);
        overlay.add_overlay(&zoom_toast.revealer);

        // Contenedor horizontal:
        // Cuando el panel de pestañas está fijado (pinned), se acopla en content_box
        // a la izquierda de la página web, redimensionándola para que no quede tapada.
        // Cuando está en auto-ocultar, se superpone como capa flotante (overlay).
        let content_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        content_box.add_css_class("nika-content-box");
        content_box.set_vexpand(true);
        content_box.set_hexpand(true);
        content_box.append(&overlay);

        let initial_auto_hide = settings.borrow().sidebar_auto_hide;
        Self::apply_sidebar_docking(
            initial_auto_hide,
            &sidebar.container,
            &sidebar.edge_trigger,
            &content_box,
            &overlay,
        );

        let c_box_pin = content_box.clone();
        let overlay_pin = overlay.clone();
        let s_cont_pin = sidebar.container.clone();
        let s_edge_pin = sidebar.edge_trigger.clone();
        sidebar.set_on_pin_toggled(move |is_auto_hide| {
            Self::apply_sidebar_docking(
                is_auto_hide,
                &s_cont_pin,
                &s_edge_pin,
                &c_box_pin,
                &overlay_pin,
            );
        });

        let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        main_box.add_css_class("nika-main-box");
        main_box.set_vexpand(true);
        main_box.set_hexpand(true);
        main_box.append(&header.header_bar);
        main_box.append(&progress_bar);
        main_box.append(&content_box);

        window.set_content(Some(&main_box));

        let tabs = Rc::new(RefCell::new(Vec::new()));
        let theme_manager = Rc::new(RefCell::new(ThemeManager::new()));
        let password_vault = Rc::new(PasswordVault::new());
        let site_permissions = SitePermissionsManager::new();
        let permissions_popover = Rc::new(PermissionsPopover::new(&window, site_permissions.clone()));

        let browser = Rc::new(Self {
            window: window.clone(),
            header,
            sidebar,
            find_bar,
            zoom_toast,
            progress_bar,
            tab_view: tab_view.clone(),
            tabs: tabs.clone(),
            theme_manager,
            download_manager: download_manager.clone(),
            settings: settings.clone(),
            content_manager: content_manager.clone(),
            password_vault: password_vault.clone(),
            persistent_session: persistent_session.clone(),
            home_url: home_url.clone(),
            site_permissions: site_permissions.clone(),
            permissions_popover: permissions_popover.clone(),
        });

        // Aplicar idioma inicial configurado a toda la interfaz
        browser.update_language(&settings.borrow().language);

        // Script Message Handlers para comunicación bidireccional segura con la página de ajustes
        let s_save = settings.clone();
        let b_save = browser.clone();
        content_manager.register_script_message_handler("nikaSaveSettings", None);
        content_manager.connect_script_message_received(Some("nikaSaveSettings"), move |_, msg| {
            let json_str = msg.to_str();
            if let Ok(new_cfg) = serde_json::from_str::<BrowserSettings>(&json_str) {
                let old_auto_hide = s_save.borrow().sidebar_auto_hide;
                let old_lang = s_save.borrow().language.clone();
                *s_save.borrow_mut() = new_cfg.clone();
                let _ = new_cfg.save();
                new_cfg.apply_dns();
                if new_cfg.sidebar_auto_hide != old_auto_hide {
                    b_save.sidebar.set_auto_hide(new_cfg.sidebar_auto_hide);
                }
                if new_cfg.language != old_lang {
                    b_save.update_language(&new_cfg.language);
                }
            }
        });

        // Selector nativo de carpetas para descargas
        let s_pick = settings.clone();
        let win_pick = window.clone();
        let b_pick = browser.clone();
        content_manager.register_script_message_handler("nikaPickFolder", None);
        content_manager.connect_script_message_received(Some("nikaPickFolder"), move |_, _| {
            let file_dialog = gtk4::FileDialog::builder()
                .title("Seleccionar carpeta de descargas")
                .build();
            let s_inner = s_pick.clone();
            let b_inner = b_pick.clone();
            file_dialog.select_folder(Some(&win_pick), None::<&gio::Cancellable>, move |res| {
                if let Ok(folder) = res {
                    if let Some(path) = folder.path() {
                        let path_str = path.to_string_lossy().to_string();
                        s_inner.borrow_mut().download_dir = path_str.clone();
                        let _ = s_inner.borrow().save();

                        // Actualizar la pestaña de configuración web abierta
                        let tabs = b_inner.tabs.borrow();
                        for t in tabs.iter() {
                            let u = t.current_url.borrow();
                            if u.starts_with("nika://settings") {
                                let js = format!(
                                    "if (window.updateDownloadDir) {{ window.updateDownloadDir('{}'); }}",
                                    path_str.replace('\\', "/")
                                );
                                t.webview.evaluate_javascript(&js, None, None, None::<&gio::Cancellable>, |_| {});
                            }
                        }
                    }
                }
            });
        });

        // Handlers para el Gestor de Contraseñas y Onboarding
        #[derive(serde::Deserialize)]
        struct OnboardingPayload {
            language: Option<String>,
            dns_provider: Option<String>,
            doh_enabled: Option<bool>,
            tracker_blocking: Option<bool>,
            block_insecure_content: Option<bool>,
            webrtc_leak_protection: Option<bool>,
            master_password: Option<String>,
        }

        let pv_onboard = password_vault.clone();
        let b_onboard = browser.clone();
        content_manager.register_script_message_handler("nikaCompleteOnboarding", None);
        content_manager.connect_script_message_received(Some("nikaCompleteOnboarding"), move |_, msg| {
            let json_str = msg.to_str();
            if let Ok(payload) = serde_json::from_str::<OnboardingPayload>(&json_str) {
                let mut s = b_onboard.settings.borrow_mut();
                if let Some(l) = payload.language {
                    s.language = l;
                }
                if let Some(p) = payload.dns_provider {
                    s.dns_provider = match p.to_lowercase().as_str() {
                        "cloudflare" | "1" => DnsProvider::Cloudflare,
                        "google" | "2" => DnsProvider::Google,
                        "quad9" | "3" => DnsProvider::Quad9,
                        "adguard" | "4" => DnsProvider::AdGuard,
                        "system" | "0" => DnsProvider::System,
                        _ => DnsProvider::Cloudflare,
                    };
                }
                if let Some(d) = payload.doh_enabled {
                    s.doh_enabled = d;
                }
                if let Some(tb) = payload.tracker_blocking {
                    s.tracker_blocking = tb;
                }
                if let Some(bic) = payload.block_insecure_content {
                    s.block_insecure_content = bic;
                }
                if let Some(wlp) = payload.webrtc_leak_protection {
                    s.webrtc_leak_protection = wlp;
                }
                s.onboarding_completed = true;
                let _ = s.save();
                s.apply_dns();
                let lang = s.language.clone();
                drop(s);

                b_onboard.update_language(&lang);

                if let Some(mp) = payload.master_password {
                    if !mp.trim().is_empty() {
                        let _ = pv_onboard.setup_master_password(&mp);
                    }
                }

                let tabs = b_onboard.tabs.borrow();
                for t in tabs.iter() {
                    let cur = t.current_url.borrow().clone();
                    if cur == "nika://welcome" {
                        t.load_uri("nika://home");
                    }
                }
            }
        });

        let pv_status = password_vault.clone();
        let b_status = browser.clone();
        content_manager.register_script_message_handler("nikaGetVaultStatus", None);
        content_manager.connect_script_message_received(Some("nikaGetVaultStatus"), move |_, _| {
            let is_cfg = pv_status.is_configured();
            let is_unlocked = pv_status.is_unlocked();
            let count = pv_status.count_entries();
            let status_json = serde_json::json!({
                "is_configured": is_cfg,
                "is_unlocked": is_unlocked,
                "entries_count": count
            }).to_string();

            let tabs = b_status.tabs.borrow();
            for t in tabs.iter() {
                let u = t.current_url.borrow();
                if u.starts_with("nika://settings") {
                    let js = format!("if (window.onVaultStatus) {{ window.onVaultStatus({}); }}", status_json);
                    t.webview.evaluate_javascript(&js, None, None, None::<&gio::Cancellable>, |_| {});

                    if is_unlocked {
                        if let Ok(entries) = pv_status.list_entries() {
                            if let Ok(entries_json) = serde_json::to_string(&entries) {
                                let js_unlocked = format!("if (window.onVaultUnlocked) {{ window.onVaultUnlocked({}); }}", entries_json);
                                t.webview.evaluate_javascript(&js_unlocked, None, None, None::<&gio::Cancellable>, |_| {});
                            }
                        }
                    }
                }
            }
        });

        let pv_setup = password_vault.clone();
        let b_setup = browser.clone();
        content_manager.register_script_message_handler("nikaSetupVault", None);
        content_manager.connect_script_message_received(Some("nikaSetupVault"), move |_, msg| {
            let master_pass = msg.to_str();
            let res = pv_setup.setup_master_password(&master_pass);
            let tabs = b_setup.tabs.borrow();
            for t in tabs.iter() {
                let u = t.current_url.borrow();
                if u.starts_with("nika://settings") {
                    match res {
                        Ok(()) => {
                            let js = "if (window.onVaultUnlocked) { window.onVaultUnlocked([]); }";
                            t.webview.evaluate_javascript(js, None, None, None::<&gio::Cancellable>, |_| {});
                        }
                        Err(ref err) => {
                            let js = format!("if (window.onVaultAuthError) {{ window.onVaultAuthError('{}'); }}", err.replace('\'', "\\'"));
                            t.webview.evaluate_javascript(&js, None, None, None::<&gio::Cancellable>, |_| {});
                        }
                    }
                }
            }
        });

        let pv_unlock = password_vault.clone();
        let b_unlock = browser.clone();
        content_manager.register_script_message_handler("nikaUnlockVault", None);
        content_manager.connect_script_message_received(Some("nikaUnlockVault"), move |_, msg| {
            let master_pass = msg.to_str();
            let res = pv_unlock.unlock(&master_pass);
            let tabs = b_unlock.tabs.borrow();
            for t in tabs.iter() {
                let u = t.current_url.borrow();
                if u.starts_with("nika://settings") {
                    match res {
                        Ok(true) => {
                            if let Ok(entries) = pv_unlock.list_entries() {
                                if let Ok(entries_json) = serde_json::to_string(&entries) {
                                    let js = format!("if (window.onVaultUnlocked) {{ window.onVaultUnlocked({}); }}", entries_json);
                                    t.webview.evaluate_javascript(&js, None, None, None::<&gio::Cancellable>, |_| {});
                                }
                            }
                        }
                        Ok(false) => {
                            let is_en = b_unlock.settings.borrow().language == "en";
                            let err_msg = if is_en { "Incorrect master password." } else { "Contraseña maestra incorrecta." };
                            let js = format!("if (window.onVaultAuthError) {{ window.onVaultAuthError('{}'); }}", err_msg);
                            t.webview.evaluate_javascript(&js, None, None, None::<&gio::Cancellable>, |_| {});
                        }
                        Err(ref e) => {
                            let js = format!("if (window.onVaultAuthError) {{ window.onVaultAuthError('{}'); }}", e.replace('\'', "\\'"));
                            t.webview.evaluate_javascript(&js, None, None, None::<&gio::Cancellable>, |_| {});
                        }
                    }
                }
            }
        });

        let pv_lock = password_vault.clone();
        let b_lock = browser.clone();
        content_manager.register_script_message_handler("nikaLockVault", None);
        content_manager.connect_script_message_received(Some("nikaLockVault"), move |_, _| {
            pv_lock.lock();
            let tabs = b_lock.tabs.borrow();
            for t in tabs.iter() {
                let u = t.current_url.borrow();
                if u.starts_with("nika://settings") {
                    let js = "if (window.onVaultLocked) { window.onVaultLocked(); }";
                    t.webview.evaluate_javascript(js, None, None, None::<&gio::Cancellable>, |_| {});
                }
            }
        });

        #[derive(serde::Deserialize)]
        struct SavePassPayload {
            domain: String,
            username: String,
            password: String,
        }

        let pv_save_pass = password_vault.clone();
        let b_save_pass = browser.clone();
        content_manager.register_script_message_handler("nikaSavePassword", None);
        content_manager.connect_script_message_received(Some("nikaSavePassword"), move |_, msg| {
            let json_str = msg.to_str();
            if let Ok(payload) = serde_json::from_str::<SavePassPayload>(&json_str) {
                if pv_save_pass.add_or_update_entry(&payload.domain, &payload.username, &payload.password).is_ok() {
                    if let Ok(entries) = pv_save_pass.list_entries() {
                        if let Ok(entries_json) = serde_json::to_string(&entries) {
                            let tabs = b_save_pass.tabs.borrow();
                            for t in tabs.iter() {
                                let u = t.current_url.borrow();
                                if u.starts_with("nika://settings") {
                                    let js = format!("if (window.onVaultUnlocked) {{ window.onVaultUnlocked({}); }}", entries_json);
                                    t.webview.evaluate_javascript(&js, None, None, None::<&gio::Cancellable>, |_| {});
                                }
                            }
                        }
                    }
                }
            }
        });

        let pv_del_pass = password_vault.clone();
        let b_del_pass = browser.clone();
        content_manager.register_script_message_handler("nikaDeletePassword", None);
        content_manager.connect_script_message_received(Some("nikaDeletePassword"), move |_, msg| {
            let id = msg.to_str();
            if pv_del_pass.delete_entry(&id).is_ok() {
                if let Ok(entries) = pv_del_pass.list_entries() {
                    if let Ok(entries_json) = serde_json::to_string(&entries) {
                        let tabs = b_del_pass.tabs.borrow();
                        for t in tabs.iter() {
                            let u = t.current_url.borrow();
                            if u.starts_with("nika://settings") {
                                let js = format!("if (window.onVaultUnlocked) {{ window.onVaultUnlocked({}); }}", entries_json);
                                t.webview.evaluate_javascript(&js, None, None, None::<&gio::Cancellable>, |_| {});
                            }
                        }
                    }
                }
            }
        });

        // Conectar botones de pestañas
        browser.sidebar.new_tab_btn.connect_clicked(clone!(
            #[strong]
            browser,
            move |_| {
                browser.add_tab(&browser.home_url);
            }
        ));

        browser.sidebar.new_incognito_tab_btn.connect_clicked(clone!(
            #[strong]
            browser,
            move |_| {
                browser.add_tab_incognito(&browser.home_url);
            }
        ));

        // Eventos del buscador en página (Ctrl+F)
        let find_key = gtk4::EventControllerKey::new();
        find_key.connect_key_pressed(clone!(
            #[strong]
            browser,
            move |_, keyval, _, _| {
                if keyval == gtk4::gdk::Key::Escape {
                    browser.hide_find_bar();
                    return glib::Propagation::Stop;
                }
                glib::Propagation::Proceed
            }
        ));
        browser.find_bar.entry.add_controller(find_key);

        browser.find_bar.entry.connect_search_changed(clone!(
            #[strong]
            browser,
            move |entry| {
                let q = entry.text();
                if let Some(tab) = browser.get_active_tab() {
                    if let Some(find) = tab.webview.find_controller() {
                        if q.is_empty() {
                            find.search_finish();
                            browser.find_bar.count_label.set_text("");
                        } else {
                            find.search(&q, webkit6::FindOptions::CASE_INSENSITIVE.bits(), 100);
                            find.count_matches(&q, webkit6::FindOptions::CASE_INSENSITIVE.bits(), 100);
                        }
                    }
                }
            }
        ));

        browser.find_bar.prev_btn.connect_clicked(clone!(
            #[strong]
            browser,
            move |_| {
                if let Some(tab) = browser.get_active_tab() {
                    if let Some(find) = tab.webview.find_controller() {
                        find.search_previous();
                    }
                }
            }
        ));

        browser.find_bar.next_btn.connect_clicked(clone!(
            #[strong]
            browser,
            move |_| {
                if let Some(tab) = browser.get_active_tab() {
                    if let Some(find) = tab.webview.find_controller() {
                        find.search_next();
                    }
                }
            }
        ));

        browser.find_bar.close_btn.connect_clicked(clone!(
            #[strong]
            browser,
            move |_| {
                browser.hide_find_bar();
            }
        ));

        browser.find_bar.entry.connect_activate(clone!(
            #[strong]
            browser,
            move |_| {
                if let Some(tab) = browser.get_active_tab() {
                    if let Some(find) = tab.webview.find_controller() {
                        find.search_next();
                    }
                }
            }
        ));

        // Botón de Configuración
        browser.header.settings_btn.connect_clicked(clone!(
            #[strong]
            browser,
            move |_| {
                browser.open_or_switch_to_settings();
            }
        ));

        // Clic en icono principal del Omnibox (candado) para abrir permisos o secundario para copiar URL
        browser.header.omnibox.connect_icon_press(clone!(
            #[strong]
            browser,
            move |entry, icon_pos| {
                if icon_pos == gtk4::EntryIconPosition::Primary {
                    if let Some(tab) = browser.get_active_tab() {
                        let url = tab.current_url.borrow().clone();
                        let lang = browser.settings.borrow().language.clone();
                        browser.permissions_popover.update_for_url(&url, &lang);
                        browser.permissions_popover.popup();
                    }
                } else if icon_pos == gtk4::EntryIconPosition::Secondary {
                    let text = entry.text();
                    let url_to_copy = if !text.is_empty() {
                        text.to_string()
                    } else if let Some(tab) = browser.get_active_tab() {
                        tab.current_url.borrow().clone()
                    } else {
                        "nika://home".to_string()
                    };

                    entry.clipboard().set_text(&url_to_copy);
                    entry.set_icon_from_icon_name(gtk4::EntryIconPosition::Secondary, Some("emblem-ok-symbolic"));
                    entry.set_icon_tooltip_text(gtk4::EntryIconPosition::Secondary, Some("¡URL copiada!"));

                    glib::timeout_add_local_once(
                        std::time::Duration::from_millis(1500),
                        clone!(
                            #[weak]
                            entry,
                            move || {
                                entry.set_icon_from_icon_name(gtk4::EntryIconPosition::Secondary, Some("nika-copy-symbolic"));
                                entry.set_icon_tooltip_text(gtk4::EntryIconPosition::Secondary, Some("Copiar URL al portapapeles"));
                            }
                        ),
                    );
                }
            }
        ));

        // Limpieza de cookies y almacenamiento de un sitio web específico
        let ps_clear = persistent_session.clone();
        browser.permissions_popover.set_on_clear_data(move |host| {
            if let Some(wdm) = ps_clear.website_data_manager() {
                let types = webkit6::WebsiteDataTypes::COOKIES
                    | webkit6::WebsiteDataTypes::LOCAL_STORAGE
                    | webkit6::WebsiteDataTypes::INDEXEDDB_DATABASES
                    | webkit6::WebsiteDataTypes::DOM_CACHE;
                wdm.clear(types, glib::TimeSpan::from_seconds(0), None::<&gio::Cancellable>, |_| {});
            }
            println!("[Nika] 🍪 Datos y cookies borrados para el sitio: {}", host);
        });

        // Escudo Nika
        browser.header.shield_btn.connect_clicked(clone!(
            #[strong]
            browser,
            move |_| {
                let lang = browser.settings.borrow().language.clone();
                let is_en = lang == "en";
                let heading = if is_en { "Nika Shield - Active Privacy" } else { "Escudo Nika - Privacidad Activa" };
                let body = if is_en {
                    "This browser actively blocks:\n• Third-party trackers and telemetry\n• Surveillance query parameters in links (UTM, fbclid, etc.)\n• Cross-site tracking cookies\n• Local IP WebRTC leaks\n\nYour browsing leaves no traces and never shares your data."
                } else {
                    "Este navegador bloquea activamente:\n• Rastreadores y telemetría de terceros\n• Parámetros de rastreo espía en enlaces (UTM, fbclid, etc.)\n• Cookies de seguimiento entre sitios\n• Fugas de IP local por WebRTC\n\nTu navegación no deja huellas ni comparte tus datos."
                };
                let settings_label = if is_en { "Open Security Settings" } else { "Abrir Ajustes de Seguridad" };
                let ok_label = if is_en { "Got it" } else { "Entendido" };

                let dialog = adw::AlertDialog::builder()
                    .heading(heading)
                    .body(body)
                    .build();
                dialog.add_response("settings", settings_label);
                dialog.add_response("ok", ok_label);
                dialog.set_default_response(Some("ok"));
                let b_clone = browser.clone();
                dialog.connect_response(None, move |_, resp| {
                    if resp == "settings" {
                        b_clone.open_settings_section("security");
                    }
                });
                dialog.present(Some(&browser.window));
            }
        ));

        // Alternar barra lateral
        let s_rev = browser.sidebar.revealer.clone();
        browser.header.sidebar_toggle_btn.connect_clicked(move |_| {
            let cur = s_rev.reveals_child();
            s_rev.set_reveal_child(!cur);
        });

        let s_rev_col = browser.sidebar.revealer.clone();
        browser.sidebar.sidebar_collapse_btn.connect_clicked(move |_| {
            s_rev_col.set_reveal_child(false);
        });

        // Botones de navegación
        browser.header.btn_back.connect_clicked(clone!(
            #[strong]
            browser,
            move |_| {
                if let Some(tab) = browser.get_active_tab() {
                    tab.go_back();
                }
            }
        ));

        browser.header.btn_forward.connect_clicked(clone!(
            #[strong]
            browser,
            move |_| {
                if let Some(tab) = browser.get_active_tab() {
                    tab.go_forward();
                }
            }
        ));

        browser.header.reload_btn.connect_clicked(clone!(
            #[strong]
            browser,
            move |_| {
                if let Some(tab) = browser.get_active_tab() {
                    tab.reload_or_stop();
                }
            }
        ));

        browser.header.btn_home.connect_clicked(clone!(
            #[strong]
            browser,
            move |_| {
                if let Some(tab) = browser.get_active_tab() {
                    tab.load_uri(&browser.home_url);
                }
            }
        ));

        // Omnibox
        browser.header.omnibox.connect_activate(clone!(
            #[strong]
            browser,
            move |entry| {
                let query = entry.text();
                let target_url = resolve_query_or_url(&query);
                if let Some(tab) = browser.get_active_tab() {
                    tab.load_uri(&target_url);
                }
            }
        ));



        // Sincronización al cambiar de pestaña
        tab_view.connect_selected_page_notify(clone!(
            #[strong]
            browser,
            move |_| {
                browser.update_tabs_visual_state();
                browser.save_session();
                browser.update_header_security_state();
                if browser.find_bar.revealer.reveals_child() {
                    let q = browser.find_bar.entry.text();
                    if !q.is_empty() {
                        if let Some(tab) = browser.get_active_tab() {
                            if let Some(find) = tab.webview.find_controller() {
                                find.search(&q, webkit6::FindOptions::CASE_INSENSITIVE.bits(), 100);
                                find.count_matches(&q, webkit6::FindOptions::CASE_INSENSITIVE.bits(), 100);
                            }
                        }
                    }
                }
            }
        ));

        tab_view.connect_close_page(clone!(
            #[strong]
            browser,
            move |tab_view, page| {
                let n_pages = tab_view.n_pages();
                if n_pages > 1 {
                    let tab_info = {
                        let mut tabs = browser.tabs.borrow_mut();
                        if let Some(pos) = tabs.iter().position(|t| t.page == *page) {
                            let tab = tabs.remove(pos);
                            Some((tab.tab_pill, tab.is_incognito))
                        } else {
                            None
                        }
                    };

                    if let Some((pill, is_incognito)) = tab_info {
                        if is_incognito {
                            browser.sidebar.incognito_tabs_box.remove(&pill);
                        } else {
                            browser.sidebar.normal_tabs_box.remove(&pill);
                        }
                    }

                    let has_incognito = browser.tabs.borrow().iter().any(|t| t.is_incognito);
                    browser.sidebar.incognito_section.set_visible(has_incognito);

                    tab_view.close_page_finish(page, true);
                } else {
                    if let Some(active_tab) = browser.get_active_tab() {
                        active_tab.load_uri(&browser.home_url);
                    }
                    tab_view.close_page_finish(page, false);
                }
                browser.save_session();
                glib::Propagation::Stop
            }
        ));

        // Guardar sesión y limpiar caché al cerrar si está configurado
        window.connect_close_request(clone!(
            #[strong]
            browser,
            move |_| {
                browser.save_session();
                if browser.settings.borrow().clear_cache_on_exit {
                    browser.clear_cache();
                }
                glib::Propagation::Proceed
            }
        ));

        // Atajos globales
        let key_controller = gtk4::EventControllerKey::new();
        key_controller.connect_key_pressed(clone!(
            #[strong]
            browser,
            move |_, keyval, _, state| {
                let is_ctrl = state.contains(gtk4::gdk::ModifierType::CONTROL_MASK);
                let is_alt = state.contains(gtk4::gdk::ModifierType::ALT_MASK);
                let is_shift = state.contains(gtk4::gdk::ModifierType::SHIFT_MASK);

                // Escape: Ocultar búsqueda o salir de pantalla completa
                if keyval == gtk4::gdk::Key::Escape {
                    if browser.find_bar.revealer.reveals_child() {
                        browser.hide_find_bar();
                        return glib::Propagation::Stop;
                    }
                    if browser.window.is_fullscreen() {
                        browser.window.unfullscreen();
                        browser.header.header_bar.set_visible(true);
                        browser.sidebar.revealer.set_visible(true);
                        return glib::Propagation::Stop;
                    }
                }

                // F11: Pantalla completa
                if keyval == gtk4::gdk::Key::F11 {
                    if browser.window.is_fullscreen() {
                        browser.window.unfullscreen();
                        browser.header.header_bar.set_visible(true);
                        browser.sidebar.revealer.set_visible(true);
                    } else {
                        browser.window.fullscreen();
                        browser.header.header_bar.set_visible(false);
                        browser.sidebar.revealer.set_visible(false);
                    }
                    return glib::Propagation::Stop;
                }

                // Ctrl + F: Buscar
                if is_ctrl && (keyval == gtk4::gdk::Key::f || keyval == gtk4::gdk::Key::F) {
                    browser.show_find_bar();
                    return glib::Propagation::Stop;
                }

                // Ctrl + Shift + N: Incógnito
                if is_ctrl && is_shift && (keyval == gtk4::gdk::Key::n || keyval == gtk4::gdk::Key::N) {
                    browser.add_tab_incognito(&browser.home_url);
                    return glib::Propagation::Stop;
                }

                // Zoom In: Ctrl + Plus, Ctrl + =, KP_Add
                if is_ctrl && (keyval == gtk4::gdk::Key::plus || keyval == gtk4::gdk::Key::equal || keyval == gtk4::gdk::Key::KP_Add) {
                    if let Some(tab) = browser.get_active_tab() {
                        let z = tab.zoom_in();
                        browser.show_zoom_toast(z);
                    }
                    return glib::Propagation::Stop;
                }

                // Zoom Out: Ctrl + Minus, KP_Subtract
                if is_ctrl && (keyval == gtk4::gdk::Key::minus || keyval == gtk4::gdk::Key::KP_Subtract) {
                    if let Some(tab) = browser.get_active_tab() {
                        let z = tab.zoom_out();
                        browser.show_zoom_toast(z);
                    }
                    return glib::Propagation::Stop;
                }

                // Zoom Reset: Ctrl + 0
                if is_ctrl && (keyval == gtk4::gdk::Key::_0 || keyval == gtk4::gdk::Key::KP_0) {
                    if let Some(tab) = browser.get_active_tab() {
                        let z = tab.zoom_reset();
                        browser.show_zoom_toast(z);
                    }
                    return glib::Propagation::Stop;
                }

                // Ctrl + B: Alternar panel lateral
                if is_ctrl && (keyval == gtk4::gdk::Key::b || keyval == gtk4::gdk::Key::B) {
                    let cur = browser.sidebar.revealer.reveals_child();
                    browser.sidebar.revealer.set_reveal_child(!cur);
                    return glib::Propagation::Stop;
                }

                // Ctrl + J: Alternar descargas
                if is_ctrl && (keyval == gtk4::gdk::Key::j || keyval == gtk4::gdk::Key::J) {
                    let cur = browser.download_manager.revealer.reveals_child();
                    browser.download_manager.revealer.set_reveal_child(!cur);
                    return glib::Propagation::Stop;
                }

                // Ctrl + I: Permisos del sitio / Información de seguridad
                if is_ctrl && (keyval == gtk4::gdk::Key::i || keyval == gtk4::gdk::Key::I) {
                    if let Some(tab) = browser.get_active_tab() {
                        let url = tab.current_url.borrow().clone();
                        let lang = browser.settings.borrow().language.clone();
                        browser.permissions_popover.update_for_url(&url, &lang);
                        browser.permissions_popover.popup();
                    }
                    return glib::Propagation::Stop;
                }

                // Ctrl + , : Configuración
                if is_ctrl && (keyval == gtk4::gdk::Key::comma || keyval == gtk4::gdk::Key::period) {
                    browser.open_or_switch_to_settings();
                    return glib::Propagation::Stop;
                }

                // Ctrl + L: Enfocar Omnibox
                if is_ctrl && (keyval == gtk4::gdk::Key::l || keyval == gtk4::gdk::Key::L) {
                    browser.header.omnibox.grab_focus();
                    browser.header.omnibox.select_region(0, -1);
                    return glib::Propagation::Stop;
                }

                // Ctrl + T: Nueva pestaña
                if is_ctrl && (keyval == gtk4::gdk::Key::t || keyval == gtk4::gdk::Key::T) {
                    browser.add_tab(&browser.home_url);
                    if let Some(tab) = browser.get_active_tab() {
                        tab.webview.grab_focus();
                    }
                    return glib::Propagation::Stop;
                }

                // Ctrl + W: Cerrar pestaña
                if is_ctrl && (keyval == gtk4::gdk::Key::w || keyval == gtk4::gdk::Key::W) {
                    if let Some(page) = browser.tab_view.selected_page() {
                        browser.tab_view.close_page(&page);
                    }
                    return glib::Propagation::Stop;
                }

                // Reordenar pestaña hacia arriba
                if is_alt && is_shift && (keyval == gtk4::gdk::Key::Up || keyval == gtk4::gdk::Key::KP_Up || keyval == gtk4::gdk::Key::Left || keyval == gtk4::gdk::Key::KP_Left) {
                    if let Some(page) = browser.tab_view.selected_page() {
                        browser.move_tab_by_page(&page, -1);
                    }
                    return glib::Propagation::Stop;
                }

                // Reordenar pestaña hacia abajo
                if is_alt && is_shift && (keyval == gtk4::gdk::Key::Down || keyval == gtk4::gdk::Key::KP_Down || keyval == gtk4::gdk::Key::Right || keyval == gtk4::gdk::Key::KP_Right) {
                    if let Some(page) = browser.tab_view.selected_page() {
                        browser.move_tab_by_page(&page, 1);
                    }
                    return glib::Propagation::Stop;
                }

                // Ctrl + Tab o Ctrl + Page_Down: Siguiente pestaña
                if (is_ctrl && !is_shift && keyval == gtk4::gdk::Key::Tab) || (is_ctrl && (keyval == gtk4::gdk::Key::Page_Down || keyval == gtk4::gdk::Key::KP_Page_Down)) {
                    browser.select_adjacent_tab(1);
                    return glib::Propagation::Stop;
                }

                // Ctrl + Shift + Tab o Ctrl + Page_Up: Pestaña anterior
                if (is_ctrl && is_shift && (keyval == gtk4::gdk::Key::Tab || keyval == gtk4::gdk::Key::ISO_Left_Tab)) || (is_ctrl && (keyval == gtk4::gdk::Key::Page_Up || keyval == gtk4::gdk::Key::KP_Page_Up)) {
                    browser.select_adjacent_tab(-1);
                    return glib::Propagation::Stop;
                }

                glib::Propagation::Proceed
            }
        ));
        key_controller.set_propagation_phase(gtk4::PropagationPhase::Capture);
        window.add_controller(key_controller);

        // Si es el primer inicio, mostrar asistente de bienvenida/onboarding
        let is_first_run = !browser.settings.borrow().onboarding_completed;
        if is_first_run {
            browser.add_tab("nika://welcome");
        } else {
            // Restaurar sesión previa o abrir pestaña de inicio
            let should_restore = browser.settings.borrow().restore_session;
            let session_file = glib::user_config_dir().join("nika-browser").join("session.json");
            let mut restored = false;

            if should_restore && session_file.exists() {
                if let Ok(content) = std::fs::read_to_string(&session_file) {
                    if let Ok(state) = serde_json::from_str::<SessionState>(&content) {
                        if !state.urls.is_empty() {
                            for url in &state.urls {
                                browser.add_tab(url);
                            }
                            let tabs_len = browser.tabs.borrow().len();
                            if tabs_len > 0 {
                                let target_idx = state.active_index.min(tabs_len - 1);
                                let target_page = browser.tabs.borrow()[target_idx].page.clone();
                                browser.tab_view.set_selected_page(&target_page);
                            }
                            restored = true;
                        }
                    }
                }
            }

            if !restored {
                browser.add_tab(&home_url);
            }
        }

        browser
    }

    pub fn add_tab(&self, url: &str) {
        self.add_tab_internal(url, false);
    }

    pub fn add_tab_incognito(&self, url: &str) {
        self.add_tab_internal(url, true);
    }

    fn add_tab_internal(&self, url: &str, is_incognito: bool) {
        let is_dark = self.theme_manager.borrow().current_mode == ThemeMode::Dark;
        let new_tab = BrowserTab::new(
            &self.tab_view,
            url,
            &self.progress_bar,
            &self.header.omnibox,
            &self.header.reload_btn,
            &self.window,
            &self.header.header_bar,
            &self.sidebar.revealer,
            &self.content_manager,
            &self.download_manager,
            self.settings.clone(),
            &self.password_vault,
            &self.persistent_session,
            &self.find_bar.count_label,
            &self.site_permissions,
            is_dark,
            is_incognito,
        );

        SidebarComponents::attach_tab_drag_and_context_menu(
            &new_tab,
            self.tabs.clone(),
            self.sidebar.normal_tabs_box.clone(),
            self.sidebar.incognito_tabs_box.clone(),
            self.tab_view.clone(),
        );

        if is_incognito {
            self.sidebar.incognito_tabs_box.append(&new_tab.tab_pill);
            self.sidebar.incognito_section.set_visible(true);
        } else {
            self.sidebar.normal_tabs_box.append(&new_tab.tab_pill);
        }

        let is_home = url == "nika://home" || url.is_empty();
        if is_home {
            new_tab.webview.grab_focus();
        } else {
            self.header.omnibox.grab_focus();
            self.header.omnibox.select_region(0, -1);
        }

        self.tab_view.set_selected_page(&new_tab.page);
        self.tabs.borrow_mut().push(new_tab);
        self.update_tabs_visual_state();
        self.save_session();
    }

    pub fn show_find_bar(&self) {
        self.find_bar.revealer.set_reveal_child(true);
        self.find_bar.entry.grab_focus();
        let q = self.find_bar.entry.text();
        if !q.is_empty() {
            if let Some(tab) = self.get_active_tab() {
                if let Some(find) = tab.webview.find_controller() {
                    find.search(&q, webkit6::FindOptions::CASE_INSENSITIVE.bits(), 100);
                    find.count_matches(&q, webkit6::FindOptions::CASE_INSENSITIVE.bits(), 100);
                }
            }
        }
    }

    pub fn hide_find_bar(&self) {
        self.find_bar.revealer.set_reveal_child(false);
        if let Some(tab) = self.get_active_tab() {
            if let Some(find) = tab.webview.find_controller() {
                find.search_finish();
            }
        }
        self.find_bar.count_label.set_text("");
    }

    pub fn show_zoom_toast(&self, zoom: f64) {
        self.zoom_toast.show(zoom);
    }

    pub fn open_or_switch_to_settings(&self) {
        self.open_settings_section("");
    }

    pub fn open_settings_section(&self, section: &str) {
        let target_url = if section.is_empty() {
            "nika://settings".to_string()
        } else {
            format!("nika://settings#{}", section)
        };
        let tabs = self.tabs.borrow();
        for tab in tabs.iter() {
            let u = tab.current_url.borrow().clone();
            if u.starts_with("nika://settings") {
                if !section.is_empty() {
                    tab.load_uri(&target_url);
                }
                self.tab_view.set_selected_page(&tab.page);
                return;
            }
        }
        drop(tabs);
        self.add_tab(&target_url);
    }

    pub fn save_session(&self) {
        save_session_data(&self.tabs, &self.tab_view);
    }

    pub fn move_tab_by_page(&self, page: &adw::TabPage, delta: i32) {
        crate::ui::sidebar::move_tab_internal(
            &self.tabs,
            &self.sidebar.normal_tabs_box,
            &self.sidebar.incognito_tabs_box,
            &self.tab_view,
            page,
            delta,
        );
    }

    /// Selecciona la pestaña contigua (delta: +1 o -1) con ciclo circular
    pub fn select_adjacent_tab(&self, delta: i32) {
        let tabs = self.tabs.borrow();
        let len = tabs.len();
        if len <= 1 {
            return;
        }
        if let Some(cur_page) = self.tab_view.selected_page() {
            if let Some(idx) = tabs.iter().position(|t| t.page == cur_page) {
                let next_idx = if delta > 0 {
                    (idx + 1) % len
                } else {
                    (idx + len - 1) % len
                };
                let target_page = tabs[next_idx].page.clone();
                drop(tabs);
                self.tab_view.set_selected_page(&target_page);
            }
        }
    }

    pub fn update_tabs_visual_state(&self) {
        if let Some(selected_page) = self.tab_view.selected_page() {
            let tabs = self.tabs.borrow();
            let lang = self.settings.borrow().language.clone();
            for tab in tabs.iter() {
                if tab.page == selected_page {
                    tab.tab_pill.add_css_class("active");
                    let url = tab.current_url.borrow().clone();
                    if !url.starts_with("file://") && url != "nika://home" && url != "http://localhost/" {
                        self.header.omnibox.set_text(&url);
                    } else {
                        self.header.omnibox.set_text("");
                    }
                    if tab.is_incognito {
                        self.header.omnibox.set_primary_icon_name(Some("nika-incognito-symbolic"));
                        self.header.omnibox.set_primary_icon_tooltip_text(Some(crate::i18n::tr(&lang, "incognito_tab")));
                    } else if url.starts_with("https://") {
                        self.header.omnibox.set_primary_icon_name(Some("channel-secure-symbolic"));
                        self.header.omnibox.set_primary_icon_tooltip_text(Some(if lang == "en" { "Encrypted Secure Connection (HTTPS)" } else { "Conexión Segura Cifrada (HTTPS)" }));
                    } else if url.starts_with("http://") {
                        self.header.omnibox.set_primary_icon_name(Some("dialog-warning-symbolic"));
                        self.header.omnibox.set_primary_icon_tooltip_text(Some(if lang == "en" { "Insecure Connection (HTTP)" } else { "Conexión No Segura (HTTP)" }));
                    } else {
                        self.header.omnibox.set_primary_icon_name(Some("security-high-symbolic"));
                        self.header.omnibox.set_primary_icon_tooltip_text(Some(crate::i18n::tr(&lang, "omnibox_shield_tooltip")));
                    }

                    // Sincronizar estado del botón de recarga
                    if *tab.is_loading.borrow() {
                        self.header.reload_btn.set_icon_name("nika-close-symbolic");
                        self.header.reload_btn.set_tooltip_text(Some(crate::i18n::tr(&lang, "stop_loading")));
                    } else {
                        self.header.reload_btn.set_icon_name("nika-reload-symbolic");
                        self.header.reload_btn.set_tooltip_text(Some(crate::i18n::tr(&lang, "nav_reload")));
                    }
                } else {
                    tab.tab_pill.remove_css_class("active");
                }
            }
        }
    }

    /// Actualiza el idioma de todos los módulos de interfaz y pestañas activas
    pub fn update_language(&self, lang: &str) {
        self.header.update_language(lang);
        self.sidebar.update_language(lang);
        self.find_bar.update_language(lang);
        self.download_manager.update_language(lang);

        let tabs = self.tabs.borrow();
        for tab in tabs.iter() {
            let url = tab.current_url.borrow().clone();
            if url.starts_with("nika://settings") {
                let title = if lang == "en" { "Settings" } else { "Configuración" };
                tab.title_label.set_text(title);
                tab.page.set_title(title);
            } else if url == "nika://welcome" {
                let title = if lang == "en" { "Welcome" } else { "Bienvenida" };
                tab.title_label.set_text(title);
                tab.page.set_title(title);
            } else if url == "nika://home" || url.is_empty() {
                let title = if tab.is_incognito {
                    if lang == "en" { "Private tab" } else { "Pestaña privada" }
                } else {
                    if lang == "en" { "Home" } else { "Inicio" }
                };
                tab.title_label.set_text(title);
                tab.page.set_title(title);
            }
        }
    }

    pub fn get_active_tab(&self) -> Option<Rc<BrowserTab>> {
        if let Some(selected_page) = self.tab_view.selected_page() {
            let tabs = self.tabs.borrow();
            for tab in tabs.iter() {
                if tab.page == selected_page {
                    return Some(Rc::new(BrowserTab {
                        webview: tab.webview.clone(),
                        page: tab.page.clone(),
                        current_url: tab.current_url.clone(),
                        tab_pill: tab.tab_pill.clone(),
                        spinner: tab.spinner.clone(),
                        icon: tab.icon.clone(),
                        title_label: tab.title_label.clone(),
                        is_loading: tab.is_loading.clone(),
                        is_dark: tab.is_dark.clone(),
                        settings: tab.settings.clone(),
                        password_vault: tab.password_vault.clone(),
                        is_incognito: tab.is_incognito,
                        zoom_level: tab.zoom_level.clone(),
                    }));
                }
            }
        }
        None
    }

    pub fn present(&self) {
        self.window.present();
    }

    /// Alterna el acoplamiento del panel de pestañas:
    /// - Pinned (fijado): se inserta en content_box a la izquierda de la página web,
    ///   redimensionando el contenido web para que no quede tapado.
    /// - Auto-hide: se monta en overlay para deslizarse flotando sobre la página.
    pub fn apply_sidebar_docking(
        is_auto_hide: bool,
        container: &gtk4::Box,
        edge_trigger: &gtk4::Box,
        content_box: &gtk4::Box,
        overlay: &gtk4::Overlay,
    ) {
        if let Some(ref parent) = container.parent() {
            if parent == content_box.upcast_ref::<gtk4::Widget>() {
                content_box.remove(container);
            } else if parent == overlay.upcast_ref::<gtk4::Widget>() {
                overlay.remove_overlay(container);
            }
        }

        if is_auto_hide {
            overlay.add_overlay(container);
            edge_trigger.set_visible(true);
        } else {
            content_box.prepend(container);
            edge_trigger.set_visible(false);
        }
    }

    /// Elimina caché y almacenamiento temporal por motivos de seguridad y privacidad
    pub fn clear_cache(&self) {
        let cache_dirs = [
            glib::user_cache_dir().join("nika-browser").join("WebKitCache"),
            glib::user_cache_dir().join("nika-browser").join("CacheStorage"),
            glib::user_cache_dir().join("nika-browser").join("HSTS"),
        ];
        for d in &cache_dirs {
            if d.exists() {
                let _ = std::fs::remove_dir_all(d);
            }
        }

        if let Some(wdm) = self.persistent_session.website_data_manager() {
            let types = webkit6::WebsiteDataTypes::DISK_CACHE
                | webkit6::WebsiteDataTypes::MEMORY_CACHE
                | webkit6::WebsiteDataTypes::DOM_CACHE
                | webkit6::WebsiteDataTypes::OFFLINE_APPLICATION_CACHE;
            wdm.clear(types, glib::TimeSpan::from_seconds(0), None::<&gio::Cancellable>, |_| {});
        }
        println!("[Nika] 🧹 Caché y almacenamiento temporal eliminados por seguridad.");
    }

    /// Actualiza el icono de seguridad y permisos en la cabecera según la pestaña activa
    pub fn update_header_security_state(&self) {
        if let Some(tab) = self.get_active_tab() {
            let url = tab.current_url.borrow().clone();
            let lang = self.settings.borrow().language.clone();
            let is_en = lang == "en";
            if url.starts_with("https://") {
                self.header.omnibox.set_primary_icon_name(Some("security-high-symbolic"));
                self.header.omnibox.set_primary_icon_tooltip_text(Some(if is_en {
                    "Secure Connection (HTTPS) • Click for site permissions"
                } else {
                    "Conexión Segura Cifrada (HTTPS) • Clic para ver permisos"
                }));
            } else if url.starts_with("http://") {
                self.header.omnibox.set_primary_icon_name(Some("dialog-warning-symbolic"));
                self.header.omnibox.set_primary_icon_tooltip_text(Some(if is_en {
                    "Insecure Connection (HTTP) • Click for site permissions"
                } else {
                    "Conexión No Segura (HTTP) • Clic para ver permisos"
                }));
            } else {
                self.header.omnibox.set_primary_icon_name(Some("security-high-symbolic"));
                self.header.omnibox.set_primary_icon_tooltip_text(Some(if is_en {
                    "Nika System Page • Click for site permissions"
                } else {
                    "Página del sistema Nika • Clic para ver permisos"
                }));
            }
        }
    }
}
