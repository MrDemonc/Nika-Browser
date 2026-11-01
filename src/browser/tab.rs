use super::favicon::FaviconManager;
use crate::downloads::DownloadManager;
use crate::privacy::PrivacyEngine;
use crate::settings::BrowserSettings;
use gio::prelude::*;
use glib::clone;
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use webkit6::prelude::*;
use webkit6::{LoadEvent, PolicyDecisionType, ResponsePolicyDecision, UserContentManager, WebView};

pub const HOME_HTML: &str = include_str!("../../resources/home/index.html");
pub const SETTINGS_HTML: &str = include_str!("../../resources/settings/index.html");
pub const WELCOME_HTML: &str = include_str!("../../resources/welcome/index.html");

pub struct BrowserTab {
    pub webview: WebView,
    pub page: adw::TabPage,
    pub current_url: Rc<RefCell<String>>,
    pub tab_pill: gtk4::Box,
    pub spinner: gtk4::Spinner,
    pub icon: gtk4::Image,
    pub title_label: gtk4::Label,
    pub is_loading: Rc<RefCell<bool>>,
    pub is_dark: Rc<RefCell<bool>>,
    pub settings: Rc<RefCell<BrowserSettings>>,
    pub password_vault: Rc<crate::passwords::PasswordVault>,
    pub is_incognito: bool,
    pub zoom_level: Rc<RefCell<f64>>,
    pub is_translated: Rc<RefCell<bool>>,
    pub original_url: Rc<RefCell<Option<String>>>,
}

impl BrowserTab {
    pub fn new(
        tab_view: &adw::TabView,
        home_url: &str,
        progress_bar: &gtk4::ProgressBar,
        omnibox: &gtk4::Entry,
        reload_btn: &gtk4::Button,
        window: &adw::ApplicationWindow,
        header_bar: &adw::HeaderBar,
        sidebar: &gtk4::Revealer,
        content_manager: &UserContentManager,
        download_manager: &Rc<DownloadManager>,
        settings: Rc<RefCell<BrowserSettings>>,
        password_vault: &Rc<crate::passwords::PasswordVault>,
        persistent_session: &webkit6::NetworkSession,
        find_count_label: &gtk4::Label,
        permissions_manager: &crate::browser::permissions::SitePermissionsManager,
        is_dark: bool,
        is_incognito: bool,
    ) -> Self {
        let session = if is_incognito {
            webkit6::NetworkSession::new_ephemeral()
        } else {
            persistent_session.clone()
        };

        let webview = WebView::builder()
            .user_content_manager(content_manager)
            .network_session(&session)
            .vexpand(true)
            .hexpand(true)
            .build();
        webview.set_background_color(&gtk4::gdk::RGBA::new(0.0, 0.0, 0.0, 1.0));

        if let Some(s) = webkit6::prelude::WebViewExt::settings(&webview) {
            let webrtc_protect = settings.borrow().webrtc_leak_protection;
            PrivacyEngine::configure_security_settings(&s, webrtc_protect);
        }

        // Conectar solicitudes de permisos (Notificaciones, Almacenamiento, Portapapeles, Micrófono, etc.)
        let window_weak = window.downgrade();
        let perms_mgr = permissions_manager.clone();
        let settings_perms = settings.clone();
        webview.connect_permission_request(move |view, request| {
            let uri = view.uri().unwrap_or_default();
            let host = crate::browser::permissions::SitePermissionsManager::extract_host(&uri);
            let has_site = perms_mgr.has_site(&host);
            let perms = perms_mgr.get_permissions(&host);

            // 1. Notificaciones
            if let Ok(n) = request.clone().downcast::<webkit6::NotificationPermissionRequest>() {
                if has_site {
                    if perms.notifications { n.allow(); } else { n.deny(); }
                    return true;
                }
                // Si aún no se ha configurado, solicitar al usuario
                if let Some(win) = window_weak.upgrade() {
                    let is_en = settings_perms.borrow().language == "en";
                    let heading = if is_en {
                        format!("Allow \"{}\" to send notifications?", host)
                    } else {
                        format!("¿Permitir a \"{}\" enviar notificaciones?", host)
                    };
                    let body = if is_en {
                        "This website would like to show notifications on your desktop.".to_string()
                    } else {
                        "Este sitio web desea mostrarte notificaciones y avisos en el escritorio.".to_string()
                    };
                    let dialog = adw::AlertDialog::builder()
                        .heading(&heading)
                        .body(&body)
                        .build();
                    dialog.add_response("deny", if is_en { "Block" } else { "Bloquear" });
                    dialog.set_response_appearance("deny", adw::ResponseAppearance::Destructive);
                    dialog.add_response("allow", if is_en { "Allow" } else { "Permitir" });
                    dialog.set_response_appearance("allow", adw::ResponseAppearance::Suggested);
                    dialog.set_default_response(Some("allow"));

                    let p_mgr = perms_mgr.clone();
                    let h = host.clone();
                    let req = n.clone();
                    dialog.connect_response(None, move |_, resp| {
                        let allow = resp == "allow";
                        let mut p = p_mgr.get_permissions(&h);
                        p.notifications = allow;
                        p_mgr.set_permissions(&h, p);
                        if allow { req.allow(); } else { req.deny(); }
                    });
                    dialog.present(Some(&win));
                    return true;
                }
                n.deny();
                return true;
            }

            // 2. Ubicación / Geolocalización
            if let Ok(g) = request.clone().downcast::<webkit6::GeolocationPermissionRequest>() {
                if has_site {
                    if perms.geolocation { g.allow(); } else { g.deny(); }
                    return true;
                }
                if let Some(win) = window_weak.upgrade() {
                    let is_en = settings_perms.borrow().language == "en";
                    let heading = if is_en {
                        format!("Allow \"{}\" to access your location?", host)
                    } else {
                        format!("¿Permitir a \"{}\" acceder a tu ubicación?", host)
                    };
                    let body = if is_en {
                        "This website would like to access your geographic location.".to_string()
                    } else {
                        "Este sitio web ha solicitado permiso para conocer tu posición geográfica.".to_string()
                    };
                    let dialog = adw::AlertDialog::builder()
                        .heading(&heading)
                        .body(&body)
                        .build();
                    dialog.add_response("deny", if is_en { "Block" } else { "Bloquear" });
                    dialog.set_response_appearance("deny", adw::ResponseAppearance::Destructive);
                    dialog.add_response("allow", if is_en { "Allow" } else { "Permitir" });
                    dialog.set_response_appearance("allow", adw::ResponseAppearance::Suggested);
                    dialog.set_default_response(Some("allow"));

                    let p_mgr = perms_mgr.clone();
                    let h = host.clone();
                    let req = g.clone();
                    dialog.connect_response(None, move |_, resp| {
                        let allow = resp == "allow";
                        let mut p = p_mgr.get_permissions(&h);
                        p.geolocation = allow;
                        p_mgr.set_permissions(&h, p);
                        if allow { req.allow(); } else { req.deny(); }
                    });
                    dialog.present(Some(&win));
                    return true;
                }
                g.deny();
                return true;
            }

            // 3. Micrófono y Cámara
            if let Ok(u) = request.clone().downcast::<webkit6::UserMediaPermissionRequest>() {
                let is_audio = webkit6::functions::user_media_permission_is_for_audio_device(&u);
                let is_video = webkit6::functions::user_media_permission_is_for_video_device(&u);
                if has_site {
                    let mut allow = true;
                    if is_audio && !perms.microphone { allow = false; }
                    if is_video && !perms.camera { allow = false; }
                    if allow { u.allow(); } else { u.deny(); }
                    return true;
                }
                if let Some(win) = window_weak.upgrade() {
                    let is_en = settings_perms.borrow().language == "en";
                    let (service_name, desc) = if is_audio && is_video {
                        (
                            if is_en { "camera and microphone" } else { "la cámara y el micrófono" },
                            if is_en { "access your camera and microphone" } else { "acceder a tu cámara web y micrófono" }
                        )
                    } else if is_video {
                        (
                            if is_en { "camera" } else { "la cámara" },
                            if is_en { "access your camera / webcam" } else { "acceder a tu cámara web" }
                        )
                    } else {
                        (
                            if is_en { "microphone" } else { "el micrófono" },
                            if is_en { "access your microphone" } else { "grabar audio o acceder al micrófono" }
                        )
                    };
                    let heading = if is_en {
                        format!("Allow \"{}\" to use {}?", host, service_name)
                    } else {
                        format!("¿Permitir a \"{}\" usar {}?", host, service_name)
                    };
                    let body = if is_en {
                        format!("This website would like to {}.", desc)
                    } else {
                        format!("Este sitio web ha solicitado {}.", desc)
                    };
                    let dialog = adw::AlertDialog::builder()
                        .heading(&heading)
                        .body(&body)
                        .build();
                    dialog.add_response("deny", if is_en { "Block" } else { "Bloquear" });
                    dialog.set_response_appearance("deny", adw::ResponseAppearance::Destructive);
                    dialog.add_response("allow", if is_en { "Allow" } else { "Permitir" });
                    dialog.set_response_appearance("allow", adw::ResponseAppearance::Suggested);
                    dialog.set_default_response(Some("allow"));

                    let p_mgr = perms_mgr.clone();
                    let h = host.clone();
                    let req = u.clone();
                    dialog.connect_response(None, move |_, resp| {
                        let allow = resp == "allow";
                        let mut p = p_mgr.get_permissions(&h);
                        if is_audio { p.microphone = allow; }
                        if is_video { p.camera = allow; }
                        p_mgr.set_permissions(&h, p);
                        if allow { req.allow(); } else { req.deny(); }
                    });
                    dialog.present(Some(&win));
                    return true;
                }
                u.deny();
                return true;
            }

            // 4. Enumeración de dispositivos
            if let Ok(d) = request.clone().downcast::<webkit6::DeviceInfoPermissionRequest>() {
                if perms.camera || perms.microphone {
                    d.allow();
                } else {
                    d.deny();
                }
                return true;
            }

            // 5. Almacenamiento y Cookies
            if let Ok(w) = request.clone().downcast::<webkit6::WebsiteDataAccessPermissionRequest>() {
                if perms.storage {
                    w.allow();
                } else {
                    w.deny();
                }
                return true;
            }

            request.deny();
            true
        });

        // Conectar notificaciones nativas de escritorio para aplicaciones web
        let window_weak = window.downgrade();
        webview.connect_show_notification(move |_view, notif| {
            if let Some(win) = window_weak.upgrade() {
                if let Some(app) = win.application() {
                    let title = notif.title().map(|s| s.to_string()).unwrap_or_else(|| "Nika Browser".to_string());
                    let gnotif = gio::Notification::new(&title);
                    if let Some(body) = notif.body() {
                        gnotif.set_body(Some(&body));
                    }
                    let w_click = win.downgrade();
                    notif.connect_clicked(move |_| {
                        if let Some(w) = w_click.upgrade() {
                            w.present();
                        }
                    });
                    app.send_notification(None, &gnotif);
                }
            }
            true
        });

        webview.connect_decide_policy(move |_view, decision, decision_type| {
            if decision_type == PolicyDecisionType::Response {
                if let Ok(resp_decision) = decision.clone().downcast::<ResponsePolicyDecision>() {
                    if !resp_decision.is_mime_type_supported() {
                        resp_decision.download();
                        return true;
                    }
                }
            }
            false
        });

        if let Some(session) = webview.network_session() {
            session.connect_download_started(clone!(
                #[strong]
                download_manager,
                move |_, download| {
                    download_manager.handle_new_download(download);
                }
            ));
        }

        webview.connect_enter_fullscreen(clone!(
            #[weak]
            window,
            #[weak]
            header_bar,
            #[weak]
            sidebar,
            #[weak]
            progress_bar,
            #[upgrade_or]
            false,
            move |_| {
                header_bar.set_visible(false);
                sidebar.set_visible(false);
                progress_bar.set_visible(false);
                window.fullscreen();
                false
            }
        ));

        webview.connect_leave_fullscreen(clone!(
            #[weak]
            window,
            #[weak]
            header_bar,
            #[weak]
            sidebar,
            #[upgrade_or]
            false,
            move |_| {
                header_bar.set_visible(true);
                sidebar.set_visible(true);
                window.unfullscreen();
                false
            }
        ));

        let current_url = Rc::new(RefCell::new(String::new()));
        let is_loading = Rc::new(RefCell::new(false));
        let is_dark_rc = Rc::new(RefCell::new(is_dark));
        let is_translated_rc = Rc::new(RefCell::new(false));
        let original_url_rc: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));

        let tab_pill = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        tab_pill.add_css_class("tab-pill");
        if is_incognito {
            tab_pill.add_css_class("incognito-tab");
        }
        tab_pill.set_hexpand(true);

        let spinner = gtk4::Spinner::new();
        spinner.add_css_class("tab-spinner");
        spinner.set_visible(false);

        let default_icon = if is_incognito {
            "nika-incognito-symbolic"
        } else {
            "nika-home-symbolic"
        };
        let icon = gtk4::Image::from_icon_name(default_icon);
        icon.add_css_class("tab-icon");
        icon.set_visible(true);

        let is_en = settings.borrow().language == "en";
        let initial_title = if is_incognito {
            if is_en { "Private tab" } else { "Pestaña privada" }
        } else {
            if is_en { "Home" } else { "Inicio" }
        };
        let title_label = gtk4::Label::new(Some(initial_title));
        title_label.add_css_class("tab-title");
        title_label.set_hexpand(true);
        title_label.set_halign(gtk4::Align::Start);
        title_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        title_label.set_max_width_chars(16);

        let close_btn = gtk4::Button::from_icon_name("nika-close-symbolic");
        close_btn.add_css_class("tab-close-btn");
        close_btn.set_tooltip_text(Some(if is_en { "Close tab" } else { "Cerrar pestaña" }));

        let page = tab_view.append(&webview);
        page.set_title(initial_title);

        close_btn.connect_clicked(clone!(
            #[weak]
            tab_view,
            #[weak]
            page,
            move |_| {
                tab_view.close_page(&page);
            }
        ));

        // Conectar cierre de pestaña solicitado por JavaScript (window.close)
        webview.connect_close(clone!(
            #[weak]
            tab_view,
            #[weak]
            page,
            move |_| {
                tab_view.close_page(&page);
            }
        ));

        let click_gesture = gtk4::GestureClick::new();
        click_gesture.connect_pressed(clone!(
            #[weak]
            tab_view,
            #[weak]
            page,
            move |_, _, _, _| {
                tab_view.set_selected_page(&page);
            }
        ));
        tab_pill.add_controller(click_gesture);

        tab_pill.append(&spinner);
        tab_pill.append(&icon);
        tab_pill.append(&title_label);
        tab_pill.append(&close_btn);

        if let Some(find) = webview.find_controller() {
            find.connect_counted_matches(clone!(
                #[weak]
                find_count_label,
                #[weak]
                tab_view,
                #[weak]
                page,
                move |_, count| {
                    if tab_view.selected_page() == Some(page.clone()) {
                        if count == 0 {
                            find_count_label.set_text("Sin resultados");
                        } else if count == 1 {
                            find_count_label.set_text("1 resultado");
                        } else {
                            find_count_label.set_text(&format!("{} resultados", count));
                        }
                    }
                }
            ));
            find.connect_failed_to_find_text(clone!(
                #[weak]
                find_count_label,
                #[weak]
                tab_view,
                #[weak]
                page,
                move |_| {
                    if tab_view.selected_page() == Some(page.clone()) {
                        find_count_label.set_text("Sin resultados");
                    }
                }
            ));
        }

        // Conectar favicon nativo de WebKit
        webview.connect_favicon_notify(clone!(
            #[weak]
            icon,
            move |view| {
                if let Some(texture) = view.favicon() {
                    icon.set_paintable(Some(&texture));
                    if let Some(uri) = view.uri() {
                        if let Ok(u) = url::Url::parse(&uri) {
                            if let Some(h) = u.host_str() {
                                FaviconManager::cache_texture(h, &texture);
                            }
                        }
                    }
                }
            }
        ));

        webview.connect_estimated_load_progress_notify(clone!(
            #[weak]
            progress_bar,
            move |view| {
                let progress = view.estimated_load_progress();
                progress_bar.set_fraction(progress);
            }
        ));

        webview.connect_title_notify(clone!(
            #[weak]
            title_label,
            #[weak]
            page,
            #[strong]
            settings,
            #[strong]
            is_incognito,
            move |view| {
                let uri = view.uri().unwrap_or_default();
                let is_home = uri == "nika://home" || uri.starts_with("file://") || uri.is_empty();
                let is_settings = uri.starts_with("nika://settings");
                let is_welcome = uri.starts_with("nika://welcome");
                let is_en = settings.borrow().language == "en";

                if is_home {
                    let t = if is_incognito {
                        if is_en { "Private tab" } else { "Pestaña privada" }
                    } else {
                        if is_en { "Home" } else { "Inicio" }
                    };
                    title_label.set_text(t);
                    page.set_title(t);
                } else if is_settings {
                    let t = if is_en { "Settings" } else { "Configuración" };
                    title_label.set_text(t);
                    page.set_title(t);
                } else if is_welcome {
                    let t = if is_en { "Welcome" } else { "Bienvenida" };
                    title_label.set_text(t);
                    page.set_title(t);
                } else if let Some(title) = view.title() {
                    if !title.is_empty() {
                        title_label.set_text(&title);
                        page.set_title(&title);
                    }
                }
            }
        ));

        webview.connect_uri_notify(clone!(
            #[weak]
            omnibox,
            #[weak]
            tab_view,
            #[weak]
            page,
            #[strong]
            current_url,
            move |view| {
                if let Some(uri) = view.uri() {
                    *current_url.borrow_mut() = uri.to_string();
                    if tab_view.selected_page() == Some(page.clone()) {
                        if !uri.starts_with("file://") && uri != "http://localhost/" && uri != "nika://home" {
                            omnibox.set_text(&uri);
                        } else {
                            omnibox.set_text("");
                        }
                    }
                }
            }
        ));

        webview.connect_load_changed(clone!(
            #[weak]
            tab_view,
            #[weak]
            page,
            #[weak]
            progress_bar,
            #[weak]
            omnibox,
            #[weak]
            reload_btn,
            #[weak]
            spinner,
            #[weak]
            icon,
            #[weak]
            title_label,
            #[strong]
            current_url,
            #[strong]
            is_loading,
            #[strong]
            is_dark_rc,
            #[strong]
            is_translated_rc,
            #[strong]
            original_url_rc,
            #[strong]
            settings,
            #[strong]
            is_incognito,
            move |view, event| {
                let is_active = tab_view.selected_page() == Some(page.clone());
                match event {
                    LoadEvent::Started => {
                        *is_loading.borrow_mut() = true;
                        spinner.start();
                        spinner.set_visible(true);
                        icon.set_visible(false);

                        if is_active {
                            reload_btn.set_icon_name("nika-close-symbolic");
                            reload_btn.set_tooltip_text(Some("Detener carga"));

                            progress_bar.set_visible(true);
                            progress_bar.set_fraction(0.15);
                        }
                    }
                    LoadEvent::Committed => {
                        if let Some(uri) = view.uri() {
                            let uri_str = uri.to_string();
                            *current_url.borrow_mut() = uri_str.clone();
                            if is_active {
                                if !uri_str.starts_with("file://") && uri_str != "http://localhost/" && uri_str != "nika://home" {
                                    let display_uri = if let Some(ref o) = *original_url_rc.borrow() {
                                        o.clone()
                                    } else {
                                        uri_str
                                    };
                                    omnibox.set_text(&display_uri);
                                } else {
                                    omnibox.set_text("");
                                }
                            }
                        }
                    }
                    LoadEvent::Finished => {
                        *is_loading.borrow_mut() = false;
                        spinner.stop();
                        spinner.set_visible(false);
                        icon.set_visible(true);

                        if is_active {
                            reload_btn.set_icon_name("nika-reload-symbolic");
                            reload_btn.set_tooltip_text(Some("Recargar página"));

                            progress_bar.set_fraction(1.0);
                            progress_bar.set_visible(false);
                        }

                        if let Some(uri) = view.uri() {
                            *current_url.borrow_mut() = uri.to_string();
                            let is_home = uri == "nika://home" || uri.starts_with("file://") || uri.is_empty();
                            let is_settings = uri.starts_with("nika://settings");
                            let is_welcome = uri.starts_with("nika://welcome");

                            if is_active {
                                if !uri.starts_with("file://") && uri != "http://localhost/" && !is_home {
                                    omnibox.set_text(&uri);
                                } else {
                                    omnibox.set_text("");
                                }

                                if uri.starts_with("https://") {
                                    omnibox.set_primary_icon_name(Some("channel-secure-symbolic"));
                                    omnibox.set_primary_icon_tooltip_text(Some("Conexión Segura Cifrada (HTTPS) • Clic para permisos"));
                                } else if uri.starts_with("http://") {
                                    omnibox.set_primary_icon_name(Some("dialog-warning-symbolic"));
                                    omnibox.set_primary_icon_tooltip_text(Some("Conexión No Segura (HTTP) • Clic para permisos"));
                                } else {
                                    omnibox.set_primary_icon_name(Some("security-high-symbolic"));
                                    omnibox.set_primary_icon_tooltip_text(Some("Página interna del sistema Nika • Clic para permisos"));
                                }
                            }

                            if is_home {
                                icon.set_paintable(None::<&gtk4::gdk::Paintable>);
                                icon.set_icon_name(Some(if is_incognito { "nika-incognito-symbolic" } else { "nika-home-symbolic" }));
                                let is_en = settings.borrow().language == "en";
                                let t = if is_incognito {
                                    if is_en { "Private tab" } else { "Pestaña privada" }
                                } else {
                                    if is_en { "Home" } else { "Inicio" }
                                };
                                title_label.set_text(t);
                                page.set_title(t);
                                if is_active {
                                    view.grab_focus();
                                    let _ = view.evaluate_javascript(
                                        "const inp = document.getElementById('search-input'); if (inp) { inp.focus(); }",
                                        None,
                                        None,
                                        None::<&gio::Cancellable>,
                                        |_| {},
                                    );
                                }
                            } else if is_settings {
                                icon.set_paintable(None::<&gtk4::gdk::Paintable>);
                                icon.set_icon_name(Some("nika-settings-symbolic"));
                                let is_en = settings.borrow().language == "en";
                                let t = if is_en { "Settings" } else { "Configuración" };
                                title_label.set_text(t);
                                page.set_title(t);
                            } else if is_welcome {
                                icon.set_paintable(None::<&gtk4::gdk::Paintable>);
                                icon.set_icon_name(Some("nika-shield-symbolic"));
                                let is_en = settings.borrow().language == "en";
                                let t = if is_en { "Welcome" } else { "Bienvenida" };
                                title_label.set_text(t);
                                page.set_title(t);
                            } else {
                                if let Some(texture) = view.favicon() {
                                    icon.set_paintable(Some(&texture));
                                } else {
                                    FaviconManager::load_for_url(&uri, &icon);
                                }
                                if let Some(title) = view.title() {
                                    if !title.is_empty() {
                                        title_label.set_text(&title);
                                        page.set_title(&title);
                                    }
                                }
                            }
                        }

                        let uri_str = view.uri().map(|u| u.to_string()).unwrap_or_default();
                        if uri_str.starts_with("nika://") || uri_str.starts_with("file://") || uri_str == "about:blank" {
                            let dark = *is_dark_rc.borrow();
                            let js = if dark {
                                "if (document.body) { document.body.classList.add('dark'); }"
                            } else {
                                "if (document.body) { document.body.classList.remove('dark'); }"
                            };
                            view.evaluate_javascript(js, None, None, None::<&gio::Cancellable>, |_| {});
                        } else if uri_str.contains(".translate.goog") || uri_str.starts_with("https://translate.google.com/translate") {
                            *is_translated_rc.borrow_mut() = true;
                        } else {
                            // Revisar sugerencia automática de traducción si está habilitada
                            let auto_trans = settings.borrow().auto_translate;
                            if auto_trans && !*is_translated_rc.borrow() {
                                let target_code = settings.borrow().translate_target_lang.clone();
                                let target_clone = target_code.clone();
                                let v_clone = view.clone();
                                let curr_clone = current_url.clone();
                                let is_trans_clone = is_translated_rc.clone();
                                let orig_url_clone = original_url_rc.clone();

                                view.evaluate_javascript(
                                    "(document.documentElement.lang || document.querySelector('meta[http-equiv=\"content-language\"]')?.content || '').toLowerCase()",
                                    None,
                                    None,
                                    None::<&gio::Cancellable>,
                                    move |res| {
                                        if let Ok(js_val) = res {
                                            let doc_lang = js_val.to_str().to_string().trim().to_lowercase();
                                            if !doc_lang.is_empty() && !doc_lang.starts_with(&target_clone.to_lowercase()) {
                                                let curr = curr_clone.borrow().clone();
                                                if !curr.is_empty() && !curr.starts_with("nika://") && !curr.starts_with("file://") {
                                                    *orig_url_clone.borrow_mut() = Some(curr.clone());
                                                    *is_trans_clone.borrow_mut() = true;
                                                    let encoded: String = url::form_urlencoded::byte_serialize(curr.as_bytes()).collect();
                                                    let trans_url = format!("https://translate.google.com/translate?sl=auto&tl={}&u={}", target_clone, encoded);
                                                    v_clone.load_uri(&trans_url);
                                                }
                                            }
                                        }
                                    }
                                );
                            }
                        }
                    }
                    _ => {}
                }
            }
        ));

        let tab = BrowserTab {
            webview,
            page,
            current_url,
            tab_pill,
            spinner,
            icon,
            title_label,
            is_loading,
            is_dark: is_dark_rc,
            settings,
            password_vault: password_vault.clone(),
            is_incognito,
            zoom_level: Rc::new(RefCell::new(1.0)),
            is_translated: is_translated_rc,
            original_url: original_url_rc,
        };

        tab.load_uri(home_url);
        tab.apply_theme_to_page(is_dark);

        tab
    }

    pub fn zoom_in(&self) -> f64 {
        let mut z = self.zoom_level.borrow_mut();
        *z = ((*z + 0.1) * 10.0).round() / 10.0;
        if *z > 3.0 {
            *z = 3.0;
        }
        let val = *z;
        self.webview.set_zoom_level(val);
        val
    }

    pub fn zoom_out(&self) -> f64 {
        let mut z = self.zoom_level.borrow_mut();
        *z = ((*z - 0.1) * 10.0).round() / 10.0;
        if *z < 0.4 {
            *z = 0.4;
        }
        let val = *z;
        self.webview.set_zoom_level(val);
        val
    }

    pub fn zoom_reset(&self) -> f64 {
        let mut z = self.zoom_level.borrow_mut();
        *z = 1.0;
        self.webview.set_zoom_level(1.0);
        1.0
    }

    pub fn load_uri(&self, url: &str) {
        let trimmed = url.trim();
        self.icon.set_paintable(None::<&gtk4::gdk::Paintable>);

        if trimmed == "nika://settings" || trimmed.starts_with("nika://settings#") {
            *self.current_url.borrow_mut() = trimmed.to_string();
            let is_en = self.settings.borrow().language == "en";
            self.title_label.set_text(if is_en { "Settings" } else { "Configuración" });
            self.icon.set_icon_name(Some("nika-settings-symbolic"));

            let current_json = serde_json::to_string(&*self.settings.borrow()).unwrap_or_default();
            let is_cfg = self.password_vault.is_configured();
            let is_unlocked = self.password_vault.is_unlocked();
            let count = self.password_vault.count_entries();
            let vault_status_json = serde_json::json!({
                "is_configured": is_cfg,
                "is_unlocked": is_unlocked,
                "entries_count": count
            }).to_string();

            let saved_passwords_json = if is_unlocked {
                serde_json::to_string(&self.password_vault.list_entries().unwrap_or_default()).unwrap_or_else(|_| "[]".to_string())
            } else {
                "[]".to_string()
            };

            let initial_tab = if let Some(pos) = trimmed.find('#') {
                &trimmed[pos + 1..]
            } else {
                ""
            };

            let html = SETTINGS_HTML
                .replace(
                    "/* __INITIAL_SETTINGS__ */",
                    &format!(
                        "window.__NIKA_SETTINGS = {};\n    window.__NIKA_INITIAL_TAB = {:?};\n    window.__NIKA_VERSION = {:?};\n    window.__NIKA_BUILD_DATE = {:?};",
                        current_json, initial_tab, crate::version::VERSION, crate::version::BUILD_DATE
                    ),
                )
                .replace(
                    "/* __INITIAL_VAULT__ */",
                    &format!(
                        "window.__NIKA_VAULT_STATUS = {};\n    window.__NIKA_SAVED_PASSWORDS = {};",
                        vault_status_json, saved_passwords_json
                    ),
                );
            self.webview.load_html(&html, Some(trimmed));
        } else if trimmed == "nika://welcome" {
            *self.current_url.borrow_mut() = "nika://welcome".to_string();
            let is_en = self.settings.borrow().language == "en";
            self.title_label.set_text(if is_en { "Welcome" } else { "Bienvenida" });
            self.icon.set_icon_name(Some("nika-shield-symbolic"));
            let current_json = serde_json::to_string(&*self.settings.borrow()).unwrap_or_default();
            let html = WELCOME_HTML.replace(
                "/* __INITIAL_SETTINGS__ */",
                &format!("window.__NIKA_SETTINGS = {};", current_json),
            );
            self.webview.load_html(&html, Some("nika://welcome"));
        } else if trimmed == "nika://home" || trimmed.starts_with("file://") || trimmed == "about:blank" || trimmed.is_empty() {
            *self.current_url.borrow_mut() = "nika://home".to_string();
            let is_en = self.settings.borrow().language == "en";
            let t = if self.is_incognito {
                if is_en { "Private tab" } else { "Pestaña privada" }
            } else {
                if is_en { "Home" } else { "Inicio" }
            };
            self.title_label.set_text(t);
            self.page.set_title(t);
            self.icon.set_icon_name(Some(if self.is_incognito { "nika-incognito-symbolic" } else { "nika-home-symbolic" }));
            let current_json = serde_json::to_string(&*self.settings.borrow()).unwrap_or_default();
            let html = HOME_HTML.replace(
                "/* __INITIAL_SETTINGS__ */",
                &format!("window.__NIKA_SETTINGS = {};", current_json),
            );
            self.webview.load_html(&html, Some("nika://home"));
        } else {
            if !trimmed.contains(".translate.goog") && !trimmed.starts_with("https://translate.google.com/translate") {
                *self.is_translated.borrow_mut() = false;
                *self.original_url.borrow_mut() = None;
            } else {
                *self.is_translated.borrow_mut() = true;
            }
            self.icon.set_icon_name(Some(if self.is_incognito { "nika-incognito-symbolic" } else { "web-browser-symbolic" }));
            FaviconManager::load_for_url(trimmed, &self.icon);
            let cleaned = PrivacyEngine::clean_url(trimmed);
            *self.current_url.borrow_mut() = cleaned.clone();
            self.webview.load_uri(&cleaned);
        }
    }

    pub fn apply_theme_to_page(&self, is_dark: bool) {
        *self.is_dark.borrow_mut() = is_dark;
        let uri = self.current_url.borrow().clone();
        if uri.starts_with("nika://") || uri.starts_with("file://") || uri == "about:blank" {
            let js = if is_dark {
                "if (document.body) { document.body.classList.add('dark'); }"
            } else {
                "if (document.body) { document.body.classList.remove('dark'); }"
            };
            self.webview.evaluate_javascript(js, None, None, None::<&gio::Cancellable>, |_| {});
        }
    }

    pub fn go_back(&self) {
        if self.webview.can_go_back() {
            self.webview.go_back();
        }
    }

    pub fn go_forward(&self) {
        if self.webview.can_go_forward() {
            self.webview.go_forward();
        }
    }

    pub fn reload_or_stop(&self) {
        if *self.is_loading.borrow() {
            self.webview.stop_loading();
        } else {
            let url = self.current_url.borrow().clone();
            if url.starts_with("nika://settings") {
                self.load_uri(&url);
            } else if url == "nika://welcome" {
                self.load_uri("nika://welcome");
            } else if url == "nika://home" {
                self.load_uri("nika://home");
            } else {
                self.webview.reload();
            }
        }
    }
}
