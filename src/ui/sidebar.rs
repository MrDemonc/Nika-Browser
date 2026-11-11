use crate::browser::session::save_session_data;
use crate::browser::tab::BrowserTab;
use crate::browser::folders::{TabFolder, TabFoldersManager, DEFAULT_FOLDER_COLORS};
use crate::settings::BrowserSettings;
use glib::clone;
use gtk4::prelude::*;
use libadwaita as adw;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct SidebarComponents {
    pub container: gtk4::Box,
    pub edge_trigger: gtk4::Box,
    pub revealer: gtk4::Revealer,
    pub sidebar_card: gtk4::Box,
    pub sidebar_toggle_btn: gtk4::Button,
    pub pin_btn: gtk4::Button,
    pub new_tab_btn: gtk4::Button,
    pub new_incognito_tab_btn: gtk4::Button,
    pub translate_btn: gtk4::Button,
    pub folders_manager: Rc<TabFoldersManager>,
    #[allow(dead_code)]
    pub folders_section: gtk4::Box,
    pub folders_header: gtk4::Box,
    pub folders_toggle_chevron: gtk4::Image,
    pub folders_list_box: gtk4::Box,
    pub add_folder_btn: gtk4::Button,
    pub normal_tabs_box: gtk4::Box,
    pub incognito_section: gtk4::Box,
    pub incognito_tabs_box: gtk4::Box,
    pub auto_hide_enabled: Rc<RefCell<bool>>,
    pub on_pin_toggled: Rc<RefCell<Option<Box<dyn Fn(bool)>>>>,
    pub is_popover_open: Rc<RefCell<bool>>,
    pub is_pointer_inside: Rc<RefCell<bool>>,
    pub on_open_tab: Rc<RefCell<Option<Box<dyn Fn(&str)>>>>,
    pub get_active_tab: Rc<RefCell<Option<Box<dyn Fn() -> Option<(String, String)>>>>>,
    pub settings: Rc<RefCell<BrowserSettings>>,
}

impl SidebarComponents {
    pub fn new(settings: Rc<RefCell<BrowserSettings>>) -> Self {
        let initial_auto_hide = settings.borrow().sidebar_auto_hide;
        let auto_hide_enabled = Rc::new(RefCell::new(initial_auto_hide));
        let on_pin_toggled: Rc<RefCell<Option<Box<dyn Fn(bool)>>>> = Rc::new(RefCell::new(None));
        let is_popover_open = Rc::new(RefCell::new(false));
        let is_pointer_inside = Rc::new(RefCell::new(false));

        let folders_manager = Rc::new(TabFoldersManager::new());
        let on_open_tab: Rc<RefCell<Option<Box<dyn Fn(&str)>>>> = Rc::new(RefCell::new(None));
        let get_active_tab: Rc<RefCell<Option<Box<dyn Fn() -> Option<(String, String)>>>>> =
            Rc::new(RefCell::new(None));

        let initial_lang = settings.borrow().language.clone();

        let revealer = gtk4::Revealer::new();
        revealer.set_transition_type(gtk4::RevealerTransitionType::SlideRight);
        revealer.set_transition_duration(280);
        revealer.set_reveal_child(!initial_auto_hide);
        revealer.set_hexpand(false);
        revealer.set_halign(gtk4::Align::Start);
        revealer.set_valign(gtk4::Align::Fill);

        let sidebar_card = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        if initial_auto_hide {
            sidebar_card.add_css_class("floating-sidebar");
        } else {
            sidebar_card.add_css_class("docked-sidebar");
        }
        sidebar_card.set_hexpand(false);
        sidebar_card.set_size_request(210, -1);

        let sidebar_toggle_btn = gtk4::Button::from_icon_name("nika-sidebar-symbolic");
        sidebar_toggle_btn.add_css_class("sidebar-action-btn");
        sidebar_toggle_btn.set_tooltip_text(Some("Alternar barra lateral (Ctrl+B)"));

        let new_tab_btn = gtk4::Button::with_label("+ Nueva pestaña");
        new_tab_btn.add_css_class("arc-new-tab-btn");
        new_tab_btn.set_tooltip_text(Some("Nueva pestaña (Ctrl+T)"));

        let new_incognito_tab_btn = gtk4::Button::from_icon_name("nika-incognito-symbolic");
        new_incognito_tab_btn.add_css_class("sidebar-action-btn");
        new_incognito_tab_btn.set_tooltip_text(Some("Nueva pestaña de incógnito (Ctrl+Shift+N)"));

        let translate_btn = gtk4::Button::from_icon_name("nika-translate-symbolic");
        translate_btn.add_css_class("sidebar-action-btn");
        translate_btn.add_css_class("translate-btn");
        translate_btn.set_tooltip_text(Some(crate::i18n::tr(&initial_lang, "nav_translate")));

        let pin_btn = gtk4::Button::new();

        let tabs_scroll = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .propagate_natural_height(true)
            .vexpand(true)
            .hexpand(false)
            .build();
        tabs_scroll.add_css_class("tabs-scroller");

        let tabs_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        tabs_box.add_css_class("vertical-tabs-box");
        tabs_box.set_hexpand(false);
        tabs_box.set_size_request(195, -1);

        // ====================================================================
        // Sección de Carpetas de Pestañas (Situada al principio)
        // ====================================================================
        let folders_section = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
        folders_section.add_css_class("tab-folders-section");

        let is_section_exp = folders_manager.is_section_expanded();

        let folders_header = gtk4::Box::new(gtk4::Orientation::Horizontal, 2);
        folders_header.add_css_class("folders-section-header");

        let folders_toggle_chevron = gtk4::Image::from_icon_name(if is_section_exp {
            "pan-down-symbolic"
        } else {
            "pan-end-symbolic"
        });
        folders_toggle_chevron.add_css_class("folders-toggle-chevron");

        let folders_icon = gtk4::Image::from_icon_name("nika-folder-symbolic");
        folders_icon.add_css_class("folders-header-icon");

        let folders_title = gtk4::Label::new(Some("Carpetas"));
        folders_title.add_css_class("folders-section-title");
        folders_title.set_hexpand(true);
        folders_title.set_halign(gtk4::Align::Start);

        let add_folder_btn = gtk4::Button::from_icon_name("nika-plus-symbolic");
        add_folder_btn.add_css_class("sidebar-action-btn");
        add_folder_btn.add_css_class("add-folder-btn");
        add_folder_btn.set_tooltip_text(Some("Nueva carpeta de pestañas"));

        folders_header.append(&folders_toggle_chevron);
        folders_header.append(&folders_icon);
        folders_header.append(&folders_title);
        folders_header.append(&add_folder_btn);

        let folders_list_box = gtk4::Box::new(gtk4::Orientation::Vertical, 3);
        folders_list_box.add_css_class("folders-list-box");
        folders_list_box.set_visible(is_section_exp);

        let folders_divider = gtk4::Separator::new(gtk4::Orientation::Horizontal);
        folders_divider.add_css_class("folders-divider");

        folders_section.append(&folders_header);
        folders_section.append(&folders_list_box);
        folders_section.append(&folders_divider);

        // Pestañas normales
        let normal_tabs_box = gtk4::Box::new(gtk4::Orientation::Vertical, 3);
        normal_tabs_box.set_hexpand(true);

        // Pestañas de incógnito
        let incognito_section = gtk4::Box::new(gtk4::Orientation::Vertical, 3);
        incognito_section.add_css_class("incognito-sidebar-section");
        incognito_section.set_visible(false);

        let incognito_divider = gtk4::Separator::new(gtk4::Orientation::Horizontal);
        incognito_divider.add_css_class("incognito-divider");

        let incognito_header = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
        incognito_header.add_css_class("incognito-header-row");

        let incog_icon = gtk4::Image::from_icon_name("nika-incognito-symbolic");
        incog_icon.add_css_class("incognito-icon");

        let incog_label = gtk4::Label::new(Some("Incógnito"));
        incog_label.add_css_class("incognito-label");
        incog_label.set_hexpand(true);
        incog_label.set_halign(gtk4::Align::Start);

        incognito_header.append(&incog_icon);
        incognito_header.append(&incog_label);

        let incognito_tabs_box = gtk4::Box::new(gtk4::Orientation::Vertical, 3);
        incognito_tabs_box.set_hexpand(true);

        incognito_section.append(&incognito_divider);
        incognito_section.append(&incognito_header);
        incognito_section.append(&incognito_tabs_box);

        // Ubicación de las carpetas y nueva pestaña estilo Arc Browser
        tabs_box.append(&folders_section);
        tabs_box.append(&new_tab_btn);
        tabs_box.append(&normal_tabs_box);
        tabs_box.append(&incognito_section);
        tabs_scroll.set_child(Some(&tabs_box));

        sidebar_card.append(&tabs_scroll);
        revealer.set_child(Some(&sidebar_card));

        // Franja sensible en el extremo izquierdo
        let edge_trigger = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        edge_trigger.add_css_class("sidebar-edge-trigger");
        edge_trigger.set_size_request(6, -1);
        edge_trigger.set_vexpand(true);
        edge_trigger.set_hexpand(false);
        edge_trigger.set_visible(initial_auto_hide);

        // Motion controller para auto-mostrar
        let motion_edge = gtk4::EventControllerMotion::new();
        motion_edge.connect_enter(clone!(
            #[strong]
            auto_hide_enabled,
            #[weak]
            revealer,
            move |_, _, _| {
                if *auto_hide_enabled.borrow() && !revealer.reveals_child() {
                    revealer.set_reveal_child(true);
                }
            }
        ));
        edge_trigger.add_controller(motion_edge);

        // Motion controller para auto-ocultar (respeta menús y popovers abiertos)
        let motion_sidebar = gtk4::EventControllerMotion::new();
        let auto_hide_clone = auto_hide_enabled.clone();
        let revealer_leave = revealer.clone();
        let is_pop_clone = is_popover_open.clone();
        let is_ptr_leave = is_pointer_inside.clone();

        let is_ptr_enter = is_pointer_inside.clone();
        motion_sidebar.connect_enter(move |_, _, _| {
            *is_ptr_enter.borrow_mut() = true;
        });

        motion_sidebar.connect_leave(move |_| {
            *is_ptr_leave.borrow_mut() = false;
            if *auto_hide_clone.borrow() {
                let r_clone = revealer_leave.clone();
                let a_clone = auto_hide_clone.clone();
                let pop_clone = is_pop_clone.clone();
                let ptr_clone = is_ptr_leave.clone();
                glib::timeout_add_local_once(std::time::Duration::from_millis(350), move || {
                    if *a_clone.borrow() && !*pop_clone.borrow() && !*ptr_clone.borrow() {
                        r_clone.set_reveal_child(false);
                    }
                });
            }
        });
        sidebar_card.add_controller(motion_sidebar);

        // Conectar botón pin
        let s_clone_pin = settings.clone();
        let on_pin_clone = on_pin_toggled.clone();
        let edge_trigger_clone = edge_trigger.clone();
        let s_card_clone = sidebar_card.clone();
        pin_btn.connect_clicked(clone!(
            #[strong]
            auto_hide_enabled,
            #[weak]
            revealer,
            #[weak]
            pin_btn,
            #[strong]
            s_clone_pin,
            #[strong]
            on_pin_clone,
            #[strong]
            edge_trigger_clone,
            #[strong]
            s_card_clone,
            move |_| {
                let is_auto = {
                    let mut mode = auto_hide_enabled.borrow_mut();
                    *mode = !*mode;
                    *mode
                };
                s_clone_pin.borrow_mut().sidebar_auto_hide = is_auto;
                let _ = s_clone_pin.borrow().save();
                if is_auto {
                    pin_btn.remove_css_class("pinned");
                    pin_btn.set_tooltip_text(Some(
                        "Auto-ocultar activo (Pasa el cursor por el borde para ver)",
                    ));
                    edge_trigger_clone.set_visible(true);
                    revealer.set_reveal_child(false);
                    s_card_clone.remove_css_class("docked-sidebar");
                    s_card_clone.add_css_class("floating-sidebar");
                } else {
                    pin_btn.add_css_class("pinned");
                    pin_btn.set_tooltip_text(Some(
                        "Panel fijado permanentemente (Clic para auto-ocultar)",
                    ));
                    edge_trigger_clone.set_visible(false);
                    revealer.set_reveal_child(true);
                    s_card_clone.remove_css_class("floating-sidebar");
                    s_card_clone.add_css_class("docked-sidebar");
                }
                if let Some(ref cb) = *on_pin_clone.borrow() {
                    cb(is_auto);
                }
            }
        ));

        let container = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        container.set_vexpand(true);
        container.set_hexpand(false);
        container.set_halign(gtk4::Align::Start);
        container.set_valign(gtk4::Align::Fill);
        container.append(&edge_trigger);
        container.append(&revealer);

        let sidebar = Self {
            container,
            edge_trigger,
            revealer,
            sidebar_card,
            sidebar_toggle_btn,
            pin_btn,
            new_tab_btn,
            new_incognito_tab_btn,
            translate_btn,
            folders_manager,
            folders_section,
            folders_header,
            folders_toggle_chevron,
            folders_list_box,
            add_folder_btn,
            normal_tabs_box,
            incognito_section,
            incognito_tabs_box,
            auto_hide_enabled,
            on_pin_toggled,
            is_popover_open,
            is_pointer_inside,
            on_open_tab,
            get_active_tab,
            settings: settings.clone(),
        };

        // Conectar botón de alternar barra lateral (ocultar / mostrar panel)
        let s_rev_toggle = sidebar.revealer.clone();
        sidebar.sidebar_toggle_btn.connect_clicked(move |_| {
            let cur = s_rev_toggle.reveals_child();
            s_rev_toggle.set_reveal_child(!cur);
        });

        // Conectar clic derecho sobre el panel lateral para mostrar opciones (Ocultar automáticamente con checkbox)
        let right_click_sidebar = gtk4::GestureClick::new();
        right_click_sidebar.set_button(3);
        let s_right = sidebar.clone();
        right_click_sidebar.connect_pressed(move |_, _, x, y| {
            s_right.show_sidebar_options_menu(s_right.sidebar_card.upcast_ref(), x, y);
        });
        sidebar.sidebar_card.add_controller(right_click_sidebar);

        // Conectar botón '+' de crear carpeta
        let s_add_btn = sidebar.clone();
        sidebar.add_folder_btn.connect_clicked(move |btn| {
            s_add_btn.show_create_folder_popover(btn.upcast_ref(), None);
        });

        // Conectar clic en la cabecera para alternar la visibilidad de todas las carpetas (ocultar/mostrar)
        let header_toggle = gtk4::GestureClick::new();
        let s_hdr_toggle = sidebar.clone();
        header_toggle.connect_pressed(move |_, _, _, _| {
            let is_exp = s_hdr_toggle.folders_manager.toggle_section_expanded();
            s_hdr_toggle.folders_list_box.set_visible(is_exp);
            s_hdr_toggle.folders_toggle_chevron.set_icon_name(Some(if is_exp {
                "pan-down-symbolic"
            } else {
                "pan-end-symbolic"
            }));
        });
        sidebar.folders_header.add_controller(header_toggle);

        // Cargar vista inicial de carpetas
        sidebar.refresh_folders_ui();

        sidebar
    }

    /// Permite conectar callbacks para abrir URLs y obtener la pestaña activa
    pub fn setup_folders_callbacks<O, G>(&self, open_tab: O, get_active_tab: G)
    where
        O: Fn(&str) + 'static,
        G: Fn() -> Option<(String, String)> + 'static,
    {
        *self.on_open_tab.borrow_mut() = Some(Box::new(open_tab));
        *self.get_active_tab.borrow_mut() = Some(Box::new(get_active_tab));
    }

    /// Regenera la interfaz de carpetas de pestañas según el estado actual
    pub fn refresh_folders_ui(&self) {
        while let Some(child) = self.folders_list_box.first_child() {
            self.folders_list_box.remove(&child);
        }

        let is_exp = self.folders_manager.is_section_expanded();
        self.folders_list_box.set_visible(is_exp);
        self.folders_toggle_chevron.set_icon_name(Some(if is_exp {
            "pan-down-symbolic"
        } else {
            "pan-end-symbolic"
        }));

        let folders = self.folders_manager.get_folders();
        if folders.is_empty() {
            let empty_banner = gtk4::Button::builder()
                .label("＋ Crear primera carpeta")
                .has_frame(false)
                .build();
            empty_banner.add_css_class("folders-empty-banner");

            let s_self = self.clone();
            empty_banner.connect_clicked(move |btn| {
                s_self.show_create_folder_popover(btn.upcast_ref(), None);
            });

            self.folders_list_box.append(&empty_banner);
            return;
        }

        for folder in folders {
            let folder_card = self.build_folder_card(&folder);
            self.folders_list_box.append(&folder_card);
        }
    }

    /// Construye la tarjeta visual para una carpeta individual
    fn build_folder_card(&self, folder: &TabFolder) -> gtk4::Box {
        let folder_card = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
        folder_card.add_css_class("tab-folder-card");
        folder_card.set_widget_name(&format!("folder-card-{}", folder.id));

        // Fila de encabezado
        let header_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
        header_row.add_css_class("tab-folder-header");

        let chevron = gtk4::Image::from_icon_name(if folder.is_expanded {
            "pan-down-symbolic"
        } else {
            "pan-end-symbolic"
        });
        chevron.add_css_class("tab-folder-chevron");

        // El icono de la carpeta toma DIRECTAMENTE el color pastel elegido (sin punto feo)
        let folder_icon = gtk4::Image::from_icon_name("nika-folder-symbolic");
        folder_icon.add_css_class("tab-folder-icon");
        folder_icon.add_css_class(&color_to_class(&folder.color));

        let name_label = gtk4::Label::new(Some(&folder.name));
        name_label.add_css_class("tab-folder-name");
        name_label.set_hexpand(true);
        name_label.set_halign(gtk4::Align::Start);
        name_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);

        let count_label = gtk4::Label::new(Some(&format!("{}", folder.tabs.len())));
        count_label.add_css_class("tab-folder-count");

        header_row.append(&chevron);
        header_row.append(&folder_icon);
        header_row.append(&name_label);
        header_row.append(&count_label);

        // Clic izquierdo en header para colapsar/expandir
        let left_click = gtk4::GestureClick::new();
        left_click.set_button(1);
        let s_toggle = self.clone();
        let f_id_toggle = folder.id.clone();
        left_click.connect_pressed(move |_, _, _, _| {
            s_toggle.folders_manager.toggle_expanded(&f_id_toggle);
            s_toggle.refresh_folders_ui();
        });
        header_row.add_controller(left_click);

        // Clic derecho en header para editar carpeta (renombrar, eliminar, cambiar color, etc.)
        let right_click = gtk4::GestureClick::new();
        right_click.set_button(3);
        let s_menu = self.clone();
        let folder_clone = folder.clone();
        let header_ref = header_row.clone();
        right_click.connect_pressed(move |_, _, _, _| {
            s_menu.show_folder_options_popover(header_ref.upcast_ref(), &folder_clone);
        });
        header_row.add_controller(right_click);

        folder_card.append(&header_row);

        // Soporte de DropTarget nativo para soltar URLs o texto en la carpeta
        let drop_target = gtk4::DropTarget::new(glib::Type::STRING, gtk4::gdk::DragAction::COPY);
        let s_drop = self.clone();
        let f_id_drop = folder.id.clone();
        drop_target.connect_drop(move |_, val, _, _| {
            if let Ok(text) = val.get::<String>() {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    let title = if let Ok(u) = url::Url::parse(trimmed) {
                        u.host_str().unwrap_or(trimmed).to_string()
                    } else {
                        trimmed.to_string()
                    };
                    s_drop.folders_manager.add_tab(&f_id_drop, &title, trimmed);
                    s_drop.refresh_folders_ui();
                    return true;
                }
            }
            false
        });
        folder_card.add_controller(drop_target);

        // Si está expandida, mostrar la lista de pestañas dentro de la carpeta
        if folder.is_expanded {
            let tabs_box = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
            tabs_box.add_css_class("tab-folder-tabs-box");

            // Opción principal: Guardar pestaña activa al principio
            let add_active_top = gtk4::Button::with_label("＋ Guardar pestaña activa");
            add_active_top.add_css_class("folder-add-active-top-btn");
            let s_add = self.clone();
            let f_id_add = folder.id.clone();
            add_active_top.connect_clicked(move |_| {
                if let Some(ref get_act) = *s_add.get_active_tab.borrow() {
                    if let Some((t, u)) = get_act() {
                        s_add.folders_manager.add_tab(&f_id_add, &t, &u);
                        s_add.refresh_folders_ui();
                    }
                }
            });
            tabs_box.append(&add_active_top);

            for tab in &folder.tabs {
                let tab_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
                tab_row.add_css_class("folder-tab-row");
                tab_row.set_tooltip_text(Some(&format!("{}\n{}", tab.title, tab.url)));

                let tab_icon = gtk4::Image::from_icon_name("web-browser-symbolic");
                tab_icon.add_css_class("folder-tab-icon");
                crate::browser::FaviconManager::load_for_url(&tab.url, &tab_icon);

                let title_lbl = gtk4::Label::new(Some(&tab.title));
                title_lbl.add_css_class("folder-tab-title");
                title_lbl.set_hexpand(true);
                title_lbl.set_halign(gtk4::Align::Start);
                title_lbl.set_ellipsize(gtk4::pango::EllipsizeMode::End);

                let del_btn = gtk4::Button::from_icon_name("nika-close-symbolic");
                del_btn.add_css_class("folder-tab-remove-btn");
                del_btn.set_tooltip_text(Some("Quitar de la carpeta"));

                let s_del = self.clone();
                let f_id_del = folder.id.clone();
                let t_id_del = tab.id.clone();
                del_btn.connect_clicked(move |_| {
                    s_del.folders_manager.remove_tab(&f_id_del, &t_id_del);
                    s_del.refresh_folders_ui();
                });

                // Clic para abrir pestaña
                let tab_click = gtk4::GestureClick::new();
                let s_open = self.clone();
                let url_open = tab.url.clone();
                tab_click.connect_pressed(move |_, _, _, _| {
                    if let Some(ref opener) = *s_open.on_open_tab.borrow() {
                        opener(&url_open);
                    }
                });
                tab_row.add_controller(tab_click);

                // Clic derecho para copiar enlace o abrir
                let tab_right_click = gtk4::GestureClick::new();
                tab_right_click.set_button(3);
                let tab_row_ref = tab_row.clone();
                let url_copy = tab.url.clone();
                let s_ctx = self.clone();
                let f_id_ctx = folder.id.clone();
                let t_id_ctx = tab.id.clone();
                tab_right_click.connect_pressed(move |_, _, _, _| {
                    let pop = gtk4::Popover::new();
                    pop.add_css_class("tab-menu-popover");
                    pop.set_parent(&tab_row_ref);

                    let mbox = gtk4::Box::new(gtk4::Orientation::Vertical, 2);

                    let b_open = gtk4::Button::with_label("🌐 Abrir pestaña");
                    b_open.add_css_class("tab-menu-btn");
                    let s_op = s_ctx.clone();
                    let u_op = url_copy.clone();
                    let p_op = pop.clone();
                    b_open.connect_clicked(move |_| {
                        if let Some(ref opener) = *s_op.on_open_tab.borrow() {
                            opener(&u_op);
                        }
                        p_op.popdown();
                    });

                    let b_copy = gtk4::Button::with_label("📋 Copiar enlace");
                    b_copy.add_css_class("tab-menu-btn");
                    let u_cp = url_copy.clone();
                    let p_cp = pop.clone();
                    b_copy.connect_clicked(move |_| {
                        if let Some(display) = gtk4::gdk::Display::default() {
                            display.clipboard().set_text(&u_cp);
                        }
                        p_cp.popdown();
                    });

                    let b_rem = gtk4::Button::with_label("✖ Quitar de carpeta");
                    b_rem.add_css_class("tab-menu-btn");
                    b_rem.add_css_class("tab-menu-btn-danger");
                    let s_rem = s_ctx.clone();
                    let f_r = f_id_ctx.clone();
                    let t_r = t_id_ctx.clone();
                    let p_rem = pop.clone();
                    b_rem.connect_clicked(move |_| {
                        s_rem.folders_manager.remove_tab(&f_r, &t_r);
                        s_rem.refresh_folders_ui();
                        p_rem.popdown();
                    });

                    mbox.append(&b_open);
                    mbox.append(&b_copy);
                    mbox.append(&b_rem);

                    pop.set_child(Some(&mbox));
                    s_ctx.on_popover_opened();
                    let s_p = s_ctx.clone();
                    pop.connect_closed(move |_| s_p.on_popover_closed());
                    pop.popup();
                });
                tab_row.add_controller(tab_right_click);

                tab_row.append(&tab_icon);
                tab_row.append(&title_lbl);
                tab_row.append(&del_btn);

                tabs_box.append(&tab_row);
            }

            folder_card.append(&tabs_box);
        }

        folder_card
    }

    /// Muestra el menú contextual para una carpeta (clic derecho)
    fn show_folder_options_popover(&self, parent: &gtk4::Widget, folder: &TabFolder) {
        let popover = gtk4::Popover::new();
        popover.add_css_class("folder-menu-popover");
        popover.set_has_arrow(false);
        popover.set_position(gtk4::PositionType::Right);
        popover.set_offset(10, 0);
        popover.set_parent(parent);

        let menu_box = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
        menu_box.set_size_request(190, -1);

        let make_btn = |text: &str, is_primary: bool, is_danger: bool| -> gtk4::Button {
            let btn = gtk4::Button::new();
            btn.add_css_class("tab-menu-btn");
            if is_primary {
                btn.add_css_class("tab-menu-btn-primary");
            }
            if is_danger {
                btn.add_css_class("tab-menu-btn-danger");
            }
            btn.set_halign(gtk4::Align::Fill);
            btn.set_hexpand(true);
            let lbl = gtk4::Label::new(Some(text));
            lbl.set_halign(gtk4::Align::Start);
            lbl.set_xalign(0.0);
            lbl.set_hexpand(true);
            btn.set_child(Some(&lbl));
            btn
        };

        // 1. Guardar pestaña activa aquí (Opción principal destacada al inicio)
        let btn_save_act = make_btn("Guardar pestaña activa", true, false);
        let s_save_act = self.clone();
        let f_id_save = folder.id.clone();
        let pop_save = popover.clone();
        btn_save_act.connect_clicked(move |_| {
            if let Some(ref get_act) = *s_save_act.get_active_tab.borrow() {
                if let Some((t, u)) = get_act() {
                    s_save_act.folders_manager.add_tab(&f_id_save, &t, &u);
                    s_save_act.refresh_folders_ui();
                }
            }
            pop_save.popdown();
        });

        // 2. Abrir todas las pestañas
        let btn_open_all = make_btn("Abrir todas las pestañas", false, false);
        let s_open_all = self.clone();
        let f_tabs = folder.tabs.clone();
        let pop_all = popover.clone();
        btn_open_all.connect_clicked(move |_| {
            if let Some(ref opener) = *s_open_all.on_open_tab.borrow() {
                for t in &f_tabs {
                    opener(&t.url);
                }
            }
            pop_all.popdown();
        });

        // 3. Renombrar carpeta
        let btn_rename = make_btn("Renombrar carpeta", false, false);
        let s_rename = self.clone();
        let f_id_rename = folder.id.clone();
        let f_name_rename = folder.name.clone();
        let p_widget = parent.clone();
        let pop_ren = popover.clone();
        btn_rename.connect_clicked(move |_| {
            pop_ren.popdown();
            s_rename.show_rename_folder_popover(&p_widget, &f_id_rename, &f_name_rename);
        });

        // 4. Cambiar color
        let btn_color = make_btn("Cambiar color", false, false);
        let s_color = self.clone();
        let f_id_color = folder.id.clone();
        let p_color_widget = parent.clone();
        let pop_col = popover.clone();
        btn_color.connect_clicked(move |_| {
            pop_col.popdown();
            s_color.show_color_picker_popover(&p_color_widget, &f_id_color);
        });

        let sep = gtk4::Separator::new(gtk4::Orientation::Horizontal);
        sep.add_css_class("tab-menu-sep");

        // 5. Eliminar carpeta
        let btn_delete = make_btn("Eliminar carpeta", false, true);
        let s_del_f = self.clone();
        let f_id_del_f = folder.id.clone();
        let pop_del = popover.clone();
        btn_delete.connect_clicked(move |_| {
            s_del_f.folders_manager.delete_folder(&f_id_del_f);
            s_del_f.refresh_folders_ui();
            pop_del.popdown();
        });

        menu_box.append(&btn_save_act);
        menu_box.append(&btn_open_all);
        menu_box.append(&btn_rename);
        menu_box.append(&btn_color);
        menu_box.append(&sep);
        menu_box.append(&btn_delete);

        popover.set_child(Some(&menu_box));

        self.on_popover_opened();
        let s_pop = self.clone();
        popover.connect_closed(move |_| {
            s_pop.on_popover_closed();
        });

        popover.popup();
    }

    /// Muestra el popover para crear una nueva carpeta estilo Arc / Traducción (sin flecha y diseño limpio)
    pub fn show_create_folder_popover(
        &self,
        parent: &gtk4::Widget,
        initial_tab: Option<(String, String)>,
    ) {
        let popover = gtk4::Popover::new();
        popover.add_css_class("folder-dialog-popover");
        popover.set_has_arrow(false);
        popover.set_position(gtk4::PositionType::Right);
        popover.set_offset(10, 0);
        popover.set_parent(parent);

        let content = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
        content.add_css_class("translation-content-box");
        content.set_size_request(220, -1);

        let header_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        header_row.add_css_class("translation-header-row");

        let folder_ico = gtk4::Image::from_icon_name("nika-folder-symbolic");
        folder_ico.set_pixel_size(16);

        let title_lbl = gtk4::Label::new(Some("Nueva Carpeta"));
        title_lbl.add_css_class("translation-title");
        title_lbl.set_hexpand(true);
        title_lbl.set_halign(gtk4::Align::Start);

        let close_btn = gtk4::Button::from_icon_name("nika-close-symbolic");
        close_btn.add_css_class("sidebar-action-btn");
        close_btn.set_tooltip_text(Some("Cerrar"));
        let pop_c_btn = popover.clone();
        close_btn.connect_clicked(move |_| {
            pop_c_btn.popdown();
        });

        header_row.append(&folder_ico);
        header_row.append(&title_lbl);
        header_row.append(&close_btn);

        let sep = gtk4::Separator::new(gtk4::Orientation::Horizontal);

        let entry = gtk4::Entry::builder()
            .placeholder_text("Nombre de carpeta...")
            .hexpand(true)
            .build();
        entry.add_css_class("folder-name-entry");

        let colors_lbl = gtk4::Label::new(Some("Color"));
        colors_lbl.add_css_class("translation-label");
        colors_lbl.set_halign(gtk4::Align::Start);

        let colors_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);

        let selected_color = Rc::new(RefCell::new("#00d2d3".to_string()));
        let dot_buttons: Rc<RefCell<Vec<gtk4::Button>>> = Rc::new(RefCell::new(Vec::new()));

        for (_name, hex) in DEFAULT_FOLDER_COLORS {
            let btn = gtk4::Button::new();
            btn.add_css_class("color-picker-dot");
            btn.add_css_class(&color_to_class(hex));

            if *hex == "#00d2d3" {
                btn.add_css_class("active");
            }

            let sel_col = selected_color.clone();
            let hex_val = hex.to_string();
            let all_dots = dot_buttons.clone();
            btn.connect_clicked(clone!(
                #[weak]
                btn,
                move |_| {
                    *sel_col.borrow_mut() = hex_val.clone();
                    for d in all_dots.borrow().iter() {
                        d.remove_css_class("active");
                    }
                    btn.add_css_class("active");
                }
            ));

            dot_buttons.borrow_mut().push(btn.clone());
            colors_box.append(&btn);
        }

        let btns_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        btns_box.set_halign(gtk4::Align::End);
        btns_box.set_margin_top(4);

        let cancel_btn = gtk4::Button::with_label("Cancelar");
        cancel_btn.add_css_class("folder-dialog-btn-cancel");
        let pop_cancel = popover.clone();
        cancel_btn.connect_clicked(move |_| {
            pop_cancel.popdown();
        });

        let create_btn = gtk4::Button::with_label("Crear");
        create_btn.add_css_class("folder-dialog-btn-create");

        let s_create = self.clone();
        let pop_create = popover.clone();
        let entry_clone = entry.clone();
        let sel_col_create = selected_color.clone();
        let init_tab_clone = initial_tab.clone();

        let do_create = move || {
            let name = entry_clone.text();
            let color = sel_col_create.borrow().clone();
            let folder = s_create.folders_manager.create_folder(&name, &color);
            if let Some((tab_title, tab_url)) = init_tab_clone.clone() {
                s_create.folders_manager.add_tab(&folder.id, &tab_title, &tab_url);
            }
            s_create.refresh_folders_ui();
            pop_create.popdown();
        };

        let do_create_click = do_create.clone();
        create_btn.connect_clicked(move |_| {
            do_create_click();
        });

        entry.connect_activate(move |_| {
            do_create();
        });

        btns_box.append(&cancel_btn);
        btns_box.append(&create_btn);

        content.append(&header_row);
        content.append(&sep);
        content.append(&entry);
        content.append(&colors_lbl);
        content.append(&colors_box);
        content.append(&btns_box);

        popover.set_child(Some(&content));

        self.on_popover_opened();
        let s_self = self.clone();
        popover.connect_closed(move |_| {
            s_self.on_popover_closed();
        });

        popover.popup();
        entry.grab_focus();
    }

    /// Popover rediseñado para renombrar una carpeta existente (sin flecha, estilo traducción)
    fn show_rename_folder_popover(&self, parent: &gtk4::Widget, folder_id: &str, current_name: &str) {
        let popover = gtk4::Popover::new();
        popover.add_css_class("folder-dialog-popover");
        popover.set_has_arrow(false);
        popover.set_position(gtk4::PositionType::Right);
        popover.set_offset(10, 0);
        popover.set_parent(parent);

        let content = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
        content.add_css_class("translation-content-box");
        content.set_size_request(220, -1);

        let header_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        header_row.add_css_class("translation-header-row");

        let icon = gtk4::Image::from_icon_name("nika-folder-symbolic");
        icon.set_pixel_size(16);

        let title_lbl = gtk4::Label::new(Some("Renombrar Carpeta"));
        title_lbl.add_css_class("translation-title");
        title_lbl.set_hexpand(true);
        title_lbl.set_halign(gtk4::Align::Start);

        let close_btn = gtk4::Button::from_icon_name("nika-close-symbolic");
        close_btn.add_css_class("sidebar-action-btn");
        close_btn.set_tooltip_text(Some("Cerrar"));
        let pop_c = popover.clone();
        close_btn.connect_clicked(move |_| pop_c.popdown());

        header_row.append(&icon);
        header_row.append(&title_lbl);
        header_row.append(&close_btn);

        let sep = gtk4::Separator::new(gtk4::Orientation::Horizontal);

        let entry = gtk4::Entry::builder().text(current_name).build();
        entry.add_css_class("folder-name-entry");

        let btns = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        btns.set_halign(gtk4::Align::End);
        btns.set_margin_top(4);

        let cancel = gtk4::Button::with_label("Cancelar");
        cancel.add_css_class("folder-dialog-btn-cancel");
        let pop_cancel = popover.clone();
        cancel.connect_clicked(move |_| pop_cancel.popdown());

        let save = gtk4::Button::with_label("Guardar");
        save.add_css_class("folder-dialog-btn-create");

        let s_save = self.clone();
        let f_id = folder_id.to_string();
        let entry_cl = entry.clone();
        let pop_s = popover.clone();
        let do_save = move || {
            let new_name = entry_cl.text();
            s_save.folders_manager.rename_folder(&f_id, &new_name);
            s_save.refresh_folders_ui();
            pop_s.popdown();
        };

        let do_save_c = do_save.clone();
        save.connect_clicked(move |_| do_save_c());
        entry.connect_activate(move |_| do_save());

        btns.append(&cancel);
        btns.append(&save);

        content.append(&header_row);
        content.append(&sep);
        content.append(&entry);
        content.append(&btns);

        popover.set_child(Some(&content));
        self.on_popover_opened();
        let s_pop = self.clone();
        popover.connect_closed(move |_| s_pop.on_popover_closed());
        popover.popup();
        entry.grab_focus();
    }

    /// Popover rediseñado para cambiar el color de una carpeta (sin flecha, estilo traducción)
    fn show_color_picker_popover(&self, parent: &gtk4::Widget, folder_id: &str) {
        let popover = gtk4::Popover::new();
        popover.add_css_class("folder-dialog-popover");
        popover.set_has_arrow(false);
        popover.set_position(gtk4::PositionType::Right);
        popover.set_offset(10, 0);
        popover.set_parent(parent);

        let content = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
        content.add_css_class("translation-content-box");
        content.set_size_request(200, -1);

        let header_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        header_row.add_css_class("translation-header-row");

        let icon = gtk4::Image::from_icon_name("nika-folder-symbolic");
        icon.set_pixel_size(16);

        let title_lbl = gtk4::Label::new(Some("Selecciona Color"));
        title_lbl.add_css_class("translation-title");
        title_lbl.set_hexpand(true);
        title_lbl.set_halign(gtk4::Align::Start);

        let close_btn = gtk4::Button::from_icon_name("nika-close-symbolic");
        close_btn.add_css_class("sidebar-action-btn");
        close_btn.set_tooltip_text(Some("Cerrar"));
        let pop_c = popover.clone();
        close_btn.connect_clicked(move |_| pop_c.popdown());

        header_row.append(&icon);
        header_row.append(&title_lbl);
        header_row.append(&close_btn);

        let sep = gtk4::Separator::new(gtk4::Orientation::Horizontal);

        let colors_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
        for (_name, hex) in DEFAULT_FOLDER_COLORS {
            let btn = gtk4::Button::new();
            btn.add_css_class("color-picker-dot");
            btn.add_css_class(&color_to_class(hex));

            let s_col = self.clone();
            let f_id = folder_id.to_string();
            let hex_val = hex.to_string();
            let pop_cl = popover.clone();
            btn.connect_clicked(move |_| {
                s_col.folders_manager.set_folder_color(&f_id, &hex_val);
                s_col.refresh_folders_ui();
                pop_cl.popdown();
            });

            colors_box.append(&btn);
        }

        content.append(&header_row);
        content.append(&sep);
        content.append(&colors_box);

        popover.set_child(Some(&content));
        self.on_popover_opened();
        let s_pop = self.clone();
        popover.connect_closed(move |_| s_pop.on_popover_closed());
        popover.popup();
    }

    /// Adjunta la cabecera (navegación y omnibox) al inicio y la barra de herramientas al pie
    pub fn attach_header(&self, header_widget: &gtk4::Widget, tools_widget: &gtk4::Widget) {
        self.sidebar_card.prepend(header_widget);
        self.sidebar_card.append(tools_widget);
    }

    /// Muestra el menú de opciones de la barra lateral con clic derecho (incluye checkbox de auto-ocultar)
    pub fn show_sidebar_options_menu(&self, parent: &gtk4::Widget, x: f64, y: f64) {
        let popover = gtk4::Popover::new();
        popover.add_css_class("folder-menu-popover");
        popover.set_has_arrow(false);
        popover.set_parent(parent);

        let menu_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        menu_box.set_size_request(210, -1);
        menu_box.set_margin_start(6);
        menu_box.set_margin_end(6);
        menu_box.set_margin_top(6);
        menu_box.set_margin_bottom(6);

        // Título de sección sutil
        let title_lbl = gtk4::Label::new(Some("Barra lateral"));
        title_lbl.add_css_class("tab-menu-section-header");
        title_lbl.set_halign(gtk4::Align::Start);
        menu_box.append(&title_lbl);

        // Opción con casilla de verificación: Ocultar automáticamente
        let check_auto_hide = gtk4::CheckButton::with_label("Ocultar automáticamente");
        check_auto_hide.add_css_class("sidebar-check-opt");
        check_auto_hide.set_active(*self.auto_hide_enabled.borrow());

        let s_check = self.clone();
        let pop_check = popover.clone();
        check_auto_hide.connect_toggled(move |btn| {
            let is_auto = btn.is_active();
            *s_check.auto_hide_enabled.borrow_mut() = is_auto;
            s_check.settings.borrow_mut().sidebar_auto_hide = is_auto;
            let _ = s_check.settings.borrow().save();

            if is_auto {
                s_check.sidebar_card.remove_css_class("docked-sidebar");
                s_check.sidebar_card.add_css_class("floating-sidebar");
            } else {
                s_check.sidebar_card.remove_css_class("floating-sidebar");
                s_check.sidebar_card.add_css_class("docked-sidebar");
            }

            if let Some(ref cb) = *s_check.on_pin_toggled.borrow() {
                cb(is_auto);
            }

            let p = pop_check.clone();
            glib::timeout_add_local_once(std::time::Duration::from_millis(220), move || {
                p.popdown();
            });
        });
        menu_box.append(&check_auto_hide);

        let sep = gtk4::Separator::new(gtk4::Orientation::Horizontal);
        menu_box.append(&sep);

        let make_item_btn = |text: &str| -> gtk4::Button {
            let btn = gtk4::Button::new();
            btn.add_css_class("tab-menu-btn");
            btn.set_halign(gtk4::Align::Fill);
            btn.set_hexpand(true);
            let lbl = gtk4::Label::new(Some(text));
            lbl.set_halign(gtk4::Align::Start);
            lbl.set_xalign(0.0);
            lbl.set_hexpand(true);
            btn.set_child(Some(&lbl));
            btn
        };

        let btn_new_tab = make_item_btn("Nueva pestaña");
        let s_nt = self.clone();
        let pop_nt = popover.clone();
        btn_new_tab.connect_clicked(move |_| {
            s_nt.new_tab_btn.emit_clicked();
            pop_nt.popdown();
        });
        menu_box.append(&btn_new_tab);

        let btn_new_incog = make_item_btn("Nueva pestaña de incógnito");
        let s_ni = self.clone();
        let pop_ni = popover.clone();
        btn_new_incog.connect_clicked(move |_| {
            s_ni.new_incognito_tab_btn.emit_clicked();
            pop_ni.popdown();
        });
        menu_box.append(&btn_new_incog);

        let btn_new_folder = make_item_btn("Nueva carpeta");
        let s_nf = self.clone();
        let pop_nf = popover.clone();
        let p_widget = parent.clone();
        btn_new_folder.connect_clicked(move |_| {
            pop_nf.popdown();
            s_nf.show_create_folder_popover(&p_widget, None);
        });
        menu_box.append(&btn_new_folder);

        popover.set_child(Some(&menu_box));

        let rect = gtk4::gdk::Rectangle::new(x as i32, y as i32, 1, 1);
        popover.set_pointing_to(Some(&rect));

        self.on_popover_opened();
        let s_pop = self.clone();
        popover.connect_closed(move |_| {
            s_pop.on_popover_closed();
        });

        popover.popup();
    }

    /// Permite conectar un callback cuando se conmuta la fijación del panel lateral
    pub fn set_on_pin_toggled<F: Fn(bool) + 'static>(&self, callback: F) {
        *self.on_pin_toggled.borrow_mut() = Some(Box::new(callback));
    }

    pub fn on_popover_opened(&self) {
        *self.is_popover_open.borrow_mut() = true;
    }

    pub fn on_popover_closed(&self) {
        *self.is_popover_open.borrow_mut() = false;
        if *self.auto_hide_enabled.borrow() && !*self.is_pointer_inside.borrow() {
            let r = self.revealer.clone();
            let a = self.auto_hide_enabled.clone();
            let ptr = self.is_pointer_inside.clone();
            let pop = self.is_popover_open.clone();
            glib::timeout_add_local_once(std::time::Duration::from_millis(350), move || {
                if *a.borrow() && !*ptr.borrow() && !*pop.borrow() {
                    r.set_reveal_child(false);
                }
            });
        }
    }

    /// Establece el modo de auto-ocultación del panel
    pub fn set_auto_hide(&self, is_auto: bool) {
        *self.auto_hide_enabled.borrow_mut() = is_auto;
        if is_auto {
            self.pin_btn.remove_css_class("pinned");
            self.pin_btn.set_tooltip_text(Some(
                "Auto-ocultar activo (Pasa el cursor por el borde para ver)",
            ));
            self.edge_trigger.set_visible(true);
            self.revealer.set_reveal_child(false);
            self.sidebar_card.remove_css_class("docked-sidebar");
            self.sidebar_card.add_css_class("floating-sidebar");
        } else {
            self.pin_btn.add_css_class("pinned");
            self.pin_btn.set_tooltip_text(Some(
                "Panel fijado permanentemente (Clic para auto-ocultar)",
            ));
            self.edge_trigger.set_visible(false);
            self.revealer.set_reveal_child(true);
            self.sidebar_card.remove_css_class("floating-sidebar");
            self.sidebar_card.add_css_class("docked-sidebar");
        }
        if let Some(ref cb) = *self.on_pin_toggled.borrow() {
            cb(is_auto);
        }
    }

    /// Actualiza tooltips del panel lateral según el idioma
    pub fn update_language(&self, lang: &str) {
        use crate::i18n::tr;
        self.new_tab_btn.set_tooltip_text(Some(tr(lang, "new_tab")));
        self.new_incognito_tab_btn
            .set_tooltip_text(Some(tr(lang, "new_incognito_tab")));
        self.translate_btn
            .set_tooltip_text(Some(tr(lang, "nav_translate")));
        self.add_folder_btn
            .set_tooltip_text(Some(tr(lang, "new_folder")));
        if *self.auto_hide_enabled.borrow() {
            self.pin_btn
                .set_tooltip_text(Some(tr(lang, "sidebar_auto_hide")));
        } else {
            self.pin_btn.set_tooltip_text(Some(tr(lang, "sidebar_pin")));
        }
    }

    /// Conecta soporte de arrastre vertical con hueco visual (slot) y menú contextual,
    /// incluyendo soporte completo para arrastrar pestañas y soltarlas directamente en carpetas.
    pub fn attach_tab_drag_and_context_menu(
        tab: &BrowserTab,
        tabs: Rc<RefCell<Vec<BrowserTab>>>,
        sidebar: &SidebarComponents,
        tab_view: adw::TabView,
    ) {
        let pill = &tab.tab_pill;
        let page = &tab.page;
        let is_incog = tab.is_incognito;
        let url_rc = tab.current_url.clone();
        let title_lbl = tab.title_label.clone();

        let drag_gesture = gtk4::GestureDrag::new();
        let target_slot_idx: Rc<RefCell<Option<usize>>> = Rc::new(RefCell::new(None));
        let initial_tab_idx: Rc<RefCell<usize>> = Rc::new(RefCell::new(0));
        let hovered_folder_id: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));

        let drop_indicator = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        drop_indicator.add_css_class("tab-drop-slot");
        drop_indicator.set_height_request(28);

        let parent_box = if is_incog {
            sidebar.incognito_tabs_box.clone()
        } else {
            sidebar.normal_tabs_box.clone()
        };

        let normal_box_clone = sidebar.normal_tabs_box.clone();
        let incog_box_clone = sidebar.incognito_tabs_box.clone();
        let folders_list_clone = sidebar.folders_list_box.clone();

        drag_gesture.connect_drag_begin(clone!(
            #[weak]
            pill,
            #[strong]
            tabs,
            #[weak]
            page,
            #[strong]
            initial_tab_idx,
            #[strong]
            target_slot_idx,
            #[strong]
            hovered_folder_id,
            move |_, _, _| {
                *target_slot_idx.borrow_mut() = None;
                *hovered_folder_id.borrow_mut() = None;
                let tabs_borrow = tabs.borrow();
                let initial = tabs_borrow
                    .iter()
                    .filter(|t| t.is_incognito == is_incog)
                    .position(|t| t.page == page)
                    .unwrap_or(0);
                *initial_tab_idx.borrow_mut() = initial;
                pill.add_css_class("tab-dragging");
            }
        ));

        drag_gesture.connect_drag_update(clone!(
            #[weak]
            pill,
            #[strong]
            tabs,
            #[strong]
            parent_box,
            #[strong]
            drop_indicator,
            #[strong]
            initial_tab_idx,
            #[strong]
            target_slot_idx,
            #[strong]
            hovered_folder_id,
            #[strong]
            folders_list_clone,
            move |gesture, offset_x, offset_y| {
                let (start_x, start_y) = gesture.start_point().unwrap_or((0.0, 0.0));
                let cur_pt = gtk4::graphene::Point::new((start_x + offset_x) as f32, (start_y + offset_y) as f32);

                // Comprobar si el cursor está sobre la lista de carpetas
                let mut folder_targeted: Option<(String, gtk4::Widget)> = None;
                if folders_list_clone.is_visible() {
                    if let Some(f_pt) = pill.compute_point(&folders_list_clone, &cur_pt) {
                        let fx = f_pt.x();
                        let fy = f_pt.y();
                        let box_w = folders_list_clone.width() as f32;
                        let box_h = folders_list_clone.height() as f32;

                        if fx >= -30.0 && fx <= box_w + 30.0 && fy >= -5.0 && fy <= box_h + 5.0 {
                            let mut child = folders_list_clone.first_child();
                            while let Some(w) = child {
                                if let Some(card_pt) = w.compute_point(&folders_list_clone, &gtk4::graphene::Point::new(0.0, 0.0)) {
                                    let cy = card_pt.y();
                                    let ch = w.height() as f32;
                                    if fy >= cy && fy <= cy + ch {
                                        let name = w.widget_name();
                                        if let Some(fid) = name.as_str().strip_prefix("folder-card-") {
                                            folder_targeted = Some((fid.to_string(), w.clone()));
                                            break;
                                        }
                                    }
                                }
                                child = w.next_sibling();
                            }
                        }
                    }
                }

                if let Some((fid, card_w)) = folder_targeted {
                    // Limpiar hover de otras tarjetas
                    let mut child = folders_list_clone.first_child();
                    while let Some(w) = child {
                        if w != card_w {
                            w.remove_css_class("tab-folder-drop-hover");
                        }
                        child = w.next_sibling();
                    }
                    card_w.add_css_class("tab-folder-drop-hover");
                    *hovered_folder_id.borrow_mut() = Some(fid);

                    if drop_indicator.parent().is_some() {
                        parent_box.remove(&drop_indicator);
                    }
                    *target_slot_idx.borrow_mut() = None;
                    return;
                }

                // Si no está sobre una carpeta, quitar resaltado de carpetas
                let mut child = folders_list_clone.first_child();
                while let Some(w) = child {
                    w.remove_css_class("tab-folder-drop-hover");
                    child = w.next_sibling();
                }
                *hovered_folder_id.borrow_mut() = None;

                // Comportamiento normal de reordenación entre pestañas hermanas
                let initial = *initial_tab_idx.borrow();
                let sibling_pills: Vec<gtk4::Box> = {
                    let tabs_borrow = tabs.borrow();
                    tabs_borrow
                        .iter()
                        .filter(|t| t.is_incognito == is_incog)
                        .map(|t| t.tab_pill.clone())
                        .collect()
                };
                let n = sibling_pills.len();
                if n <= 1 {
                    return;
                }
                let steps = (offset_y / 34.0).round() as i32;
                let target = (initial as i32 + steps).clamp(0, (n - 1) as i32) as usize;

                if target != initial {
                    *target_slot_idx.borrow_mut() = Some(target);
                    if drop_indicator.parent().is_none() {
                        parent_box.append(&drop_indicator);
                    }
                    if target > initial {
                        parent_box
                            .reorder_child_after(&drop_indicator, Some(&sibling_pills[target]));
                    } else if target == 0 {
                        parent_box.reorder_child_after(&drop_indicator, None::<&gtk4::Widget>);
                    } else {
                        parent_box
                            .reorder_child_after(&drop_indicator, Some(&sibling_pills[target - 1]));
                    }
                } else {
                    *target_slot_idx.borrow_mut() = None;
                    if drop_indicator.parent().is_some() {
                        parent_box.remove(&drop_indicator);
                    }
                }
            }
        ));

        let sidebar_ref = sidebar.clone();
        drag_gesture.connect_drag_end(clone!(
            #[weak]
            pill,
            #[strong]
            tabs,
            #[strong]
            normal_box_clone,
            #[strong]
            incog_box_clone,
            #[strong]
            tab_view,
            #[weak]
            page,
            #[strong]
            drop_indicator,
            #[strong]
            parent_box,
            #[strong]
            initial_tab_idx,
            #[strong]
            target_slot_idx,
            #[strong]
            hovered_folder_id,
            #[strong]
            folders_list_clone,
            #[strong]
            url_rc,
            #[strong]
            title_lbl,
            move |_, _, _| {
                pill.remove_css_class("tab-dragging");
                pill.remove_css_class("dragging");
                if drop_indicator.parent().is_some() {
                    parent_box.remove(&drop_indicator);
                }

                // Limpiar resaltados visuales de carpetas
                let mut child = folders_list_clone.first_child();
                while let Some(w) = child {
                    w.remove_css_class("tab-folder-drop-hover");
                    child = w.next_sibling();
                }

                // Si se soltó sobre una carpeta, GUARDAR la pestaña en esa carpeta
                if let Some(folder_id) = hovered_folder_id.borrow_mut().take() {
                    let u = url_rc.borrow().clone();
                    let t = title_lbl.text().to_string();
                    if !u.is_empty() && u != "about:blank" {
                        sidebar_ref.folders_manager.add_tab(&folder_id, &t, &u);
                        sidebar_ref.refresh_folders_ui();
                    }
                    *target_slot_idx.borrow_mut() = None;
                    return;
                }

                // De lo contrario, reordenar pestañas si hubo desplazamiento
                if let Some(target) = *target_slot_idx.borrow() {
                    let initial = *initial_tab_idx.borrow();
                    if target != initial {
                        let delta = target as i32 - initial as i32;
                        move_tab_internal(
                            &tabs,
                            &normal_box_clone,
                            &incog_box_clone,
                            &tab_view,
                            &page,
                            delta,
                        );
                    }
                }
                *target_slot_idx.borrow_mut() = None;
            }
        ));
        pill.add_controller(drag_gesture);

        let right_click = gtk4::GestureClick::new();
        right_click.set_button(3);
        let normal_box_rc = sidebar.normal_tabs_box.clone();
        let incog_box_rc = sidebar.incognito_tabs_box.clone();
        let is_pop_open = sidebar.is_popover_open.clone();
        let is_ptr_inside = sidebar.is_pointer_inside.clone();
        let auto_hide_en = sidebar.auto_hide_enabled.clone();
        let rev = sidebar.revealer.clone();
        let sidebar_clone = sidebar.clone();

        right_click.connect_pressed(clone!(
            #[strong]
            tabs,
            #[strong]
            normal_box_rc,
            #[strong]
            incog_box_rc,
            #[strong]
            tab_view,
            #[weak]
            page,
            #[weak]
            pill,
            #[strong]
            is_pop_open,
            #[strong]
            is_ptr_inside,
            #[strong]
            auto_hide_en,
            #[strong]
            rev,
            #[strong]
            sidebar_clone,
            #[strong]
            url_rc,
            #[strong]
            title_lbl,
            move |_, _, _, _| {
                let popover = gtk4::Popover::new();
                popover.add_css_class("folder-menu-popover");
                popover.set_has_arrow(false);
                popover.set_parent(&pill);

                let menu_box = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
                menu_box.set_size_request(180, -1);

                let make_tab_btn = |text: &str, is_danger: bool| -> gtk4::Button {
                    let btn = gtk4::Button::new();
                    btn.add_css_class("tab-menu-btn");
                    if is_danger {
                        btn.add_css_class("tab-menu-btn-danger");
                    }
                    btn.set_halign(gtk4::Align::Fill);
                    btn.set_hexpand(true);
                    let lbl = gtk4::Label::new(Some(text));
                    lbl.set_halign(gtk4::Align::Start);
                    lbl.set_xalign(0.0);
                    lbl.set_hexpand(true);
                    btn.set_child(Some(&lbl));
                    btn
                };

                let btn_up = make_tab_btn("Mover hacia arriba", false);
                btn_up.connect_clicked(clone!(
                    #[strong]
                    tabs,
                    #[strong]
                    normal_box_rc,
                    #[strong]
                    incog_box_rc,
                    #[strong]
                    tab_view,
                    #[weak]
                    page,
                    #[weak]
                    popover,
                    move |_| {
                        move_tab_internal(
                            &tabs,
                            &normal_box_rc,
                            &incog_box_rc,
                            &tab_view,
                            &page,
                            -1,
                        );
                        popover.popdown();
                    }
                ));

                let btn_down = make_tab_btn("Mover hacia abajo", false);
                btn_down.connect_clicked(clone!(
                    #[strong]
                    tabs,
                    #[strong]
                    normal_box_rc,
                    #[strong]
                    incog_box_rc,
                    #[strong]
                    tab_view,
                    #[weak]
                    page,
                    #[weak]
                    popover,
                    move |_| {
                        move_tab_internal(
                            &tabs,
                            &normal_box_rc,
                            &incog_box_rc,
                            &tab_view,
                            &page,
                            1,
                        );
                        popover.popdown();
                    }
                ));

                menu_box.append(&btn_up);
                menu_box.append(&btn_down);

                // Sección: Guardar en carpeta
                let sep1 = gtk4::Separator::new(gtk4::Orientation::Horizontal);
                menu_box.append(&sep1);

                let lbl_folders = gtk4::Label::new(Some("Guardar en carpeta:"));
                lbl_folders.add_css_class("tab-menu-section-header");
                lbl_folders.set_halign(gtk4::Align::Start);
                menu_box.append(&lbl_folders);

                let existing_folders = sidebar_clone.folders_manager.get_folders();
                for f in existing_folders {
                    let btn_f = make_tab_btn(&format!("Guardar en {}", f.name), false);
                    let s_f = sidebar_clone.clone();
                    let f_id = f.id.clone();
                    let u_save = url_rc.clone();
                    let t_save = title_lbl.clone();
                    let pop_f = popover.clone();
                    btn_f.connect_clicked(move |_| {
                        let u = u_save.borrow().clone();
                        let t = t_save.text().to_string();
                        if !u.is_empty() && u != "about:blank" {
                            s_f.folders_manager.add_tab(&f_id, &t, &u);
                            s_f.refresh_folders_ui();
                        }
                        pop_f.popdown();
                    });
                    menu_box.append(&btn_f);
                }

                let btn_new_folder = make_tab_btn("Nueva carpeta...", false);
                let s_new = sidebar_clone.clone();
                let u_save_new = url_rc.clone();
                let t_save_new = title_lbl.clone();
                let pop_new = popover.clone();
                let pill_ref = pill.clone();
                btn_new_folder.connect_clicked(move |_| {
                    pop_new.popdown();
                    let u = u_save_new.borrow().clone();
                    let t = t_save_new.text().to_string();
                    let init_tab = if !u.is_empty() && u != "about:blank" {
                        Some((t, u))
                    } else {
                        None
                    };
                    s_new.show_create_folder_popover(pill_ref.upcast_ref(), init_tab);
                });
                menu_box.append(&btn_new_folder);

                let sep2 = gtk4::Separator::new(gtk4::Orientation::Horizontal);
                menu_box.append(&sep2);

                let btn_close = make_tab_btn("Cerrar pestaña", true);
                btn_close.add_css_class("tab-menu-btn");
                btn_close.connect_clicked(clone!(
                    #[strong]
                    tab_view,
                    #[weak]
                    page,
                    #[weak]
                    popover,
                    move |_| {
                        tab_view.close_page(&page);
                        popover.popdown();
                    }
                ));
                menu_box.append(&btn_close);

                popover.set_child(Some(&menu_box));

                *is_pop_open.borrow_mut() = true;
                popover.connect_closed(clone!(
                    #[strong]
                    is_pop_open,
                    #[strong]
                    is_ptr_inside,
                    #[strong]
                    auto_hide_en,
                    #[strong]
                    rev,
                    move |_| {
                        *is_pop_open.borrow_mut() = false;
                        if *auto_hide_en.borrow() && !*is_ptr_inside.borrow() {
                            let r = rev.clone();
                            let a = auto_hide_en.clone();
                            let ptr = is_ptr_inside.clone();
                            let pop = is_pop_open.clone();
                            glib::timeout_add_local_once(
                                std::time::Duration::from_millis(350),
                                move || {
                                    if *a.borrow() && !*ptr.borrow() && !*pop.borrow() {
                                        r.set_reveal_child(false);
                                    }
                                },
                            );
                        }
                    }
                ));

                popover.popup();
            }
        ));
        pill.add_controller(right_click);
    }
}

pub fn move_tab_internal(
    tabs: &Rc<RefCell<Vec<BrowserTab>>>,
    normal_tabs_box: &gtk4::Box,
    incognito_tabs_box: &gtk4::Box,
    tab_view: &adw::TabView,
    page: &adw::TabPage,
    delta: i32,
) {
    let (from_idx, to_idx, is_incog) = {
        let tabs_borrow = tabs.borrow();
        let Some(current) = tabs_borrow.iter().position(|t| t.page == *page) else {
            return;
        };
        let is_incognito = tabs_borrow[current].is_incognito;
        let group: Vec<usize> = tabs_borrow
            .iter()
            .enumerate()
            .filter(|(_, t)| t.is_incognito == is_incognito)
            .map(|(idx, _)| idx)
            .collect();
        let Some(group_pos) = group.iter().position(|&idx| idx == current) else {
            return;
        };
        let target_pos = (group_pos as i32 + delta).clamp(0, (group.len() - 1) as i32) as usize;
        (current, group[target_pos], is_incognito)
    };

    if from_idx != to_idx {
        let mut tabs_mut = tabs.borrow_mut();
        if from_idx >= tabs_mut.len() || to_idx >= tabs_mut.len() {
            return;
        }

        let tab = tabs_mut.remove(from_idx);
        tabs_mut.insert(to_idx, tab);

        let target_tab = &tabs_mut[to_idx];
        tab_view.reorder_page(&target_tab.page, to_idx as i32);

        let parent_box = if is_incog {
            incognito_tabs_box
        } else {
            normal_tabs_box
        };

        let siblings: Vec<_> = tabs_mut
            .iter()
            .filter(|t| t.is_incognito == is_incog)
            .collect();
        let sibling_idx = siblings
            .iter()
            .position(|t| t.page == target_tab.page)
            .unwrap_or(0);
        if sibling_idx == 0 {
            parent_box.reorder_child_after(&target_tab.tab_pill, None::<&gtk4::Widget>);
        } else {
            let prev_sibling = &siblings[sibling_idx - 1];
            parent_box.reorder_child_after(&target_tab.tab_pill, Some(&prev_sibling.tab_pill));
        }

        drop(tabs_mut);
        save_session_data(tabs, tab_view);
    }
}

fn color_to_class(hex: &str) -> String {
    format!("color-dot-{}", hex.trim_start_matches('#').to_lowercase())
}
