use crate::browser::session::save_session_data;
use crate::browser::tab::BrowserTab;
use crate::settings::BrowserSettings;
use glib::clone;
use gtk4::prelude::*;
use libadwaita as adw;
use std::cell::RefCell;
use std::rc::Rc;

pub struct SidebarComponents {
    pub container: gtk4::Box,
    pub edge_trigger: gtk4::Box,
    pub revealer: gtk4::Revealer,
    pub sidebar_card: gtk4::Box,
    pub pin_btn: gtk4::Button,
    pub new_tab_btn: gtk4::Button,
    pub new_incognito_tab_btn: gtk4::Button,
    pub sidebar_collapse_btn: gtk4::Button,
    pub normal_tabs_box: gtk4::Box,
    pub incognito_section: gtk4::Box,
    pub incognito_tabs_box: gtk4::Box,
    pub auto_hide_enabled: Rc<RefCell<bool>>,
    pub on_pin_toggled: Rc<RefCell<Option<Box<dyn Fn(bool)>>>>,
}

impl SidebarComponents {
    pub fn new(settings: Rc<RefCell<BrowserSettings>>) -> Self {
        let initial_auto_hide = settings.borrow().sidebar_auto_hide;
        let auto_hide_enabled = Rc::new(RefCell::new(initial_auto_hide));
        let on_pin_toggled: Rc<RefCell<Option<Box<dyn Fn(bool)>>>> = Rc::new(RefCell::new(None));

        let revealer = gtk4::Revealer::new();
        revealer.set_transition_type(gtk4::RevealerTransitionType::SlideRight);
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
        sidebar_card.set_size_request(180, -1);

        // Cabecera del panel lateral
        let sidebar_header = gtk4::Box::new(gtk4::Orientation::Horizontal, 2);
        sidebar_header.add_css_class("sidebar-header");

        let sidebar_title = gtk4::Label::new(Some("Pestañas"));
        sidebar_title.add_css_class("sidebar-title");
        sidebar_title.set_hexpand(true);
        sidebar_title.set_halign(gtk4::Align::Start);

        let pin_btn = gtk4::Button::from_icon_name("nika-pin-symbolic");
        pin_btn.add_css_class("sidebar-action-btn");
        if initial_auto_hide {
            pin_btn.set_tooltip_text(Some("Auto-ocultar activo (Pasa el cursor por el borde para ver)"));
        } else {
            pin_btn.add_css_class("pinned");
            pin_btn.set_tooltip_text(Some("Panel fijado permanentemente (Clic para auto-ocultar)"));
        }

        let new_tab_btn = gtk4::Button::from_icon_name("nika-plus-symbolic");
        new_tab_btn.add_css_class("sidebar-action-btn");
        new_tab_btn.set_tooltip_text(Some("Nueva pestaña (Ctrl+T)"));

        let new_incognito_tab_btn = gtk4::Button::from_icon_name("nika-incognito-symbolic");
        new_incognito_tab_btn.add_css_class("sidebar-action-btn");
        new_incognito_tab_btn.set_tooltip_text(Some("Nueva pestaña de incógnito (Ctrl+Shift+N)"));

        let sidebar_collapse_btn = gtk4::Button::from_icon_name("nika-back-symbolic");
        sidebar_collapse_btn.add_css_class("sidebar-action-btn");
        sidebar_collapse_btn.set_tooltip_text(Some("Ocultar barra de pestañas (Ctrl+B)"));

        sidebar_header.append(&sidebar_title);
        sidebar_header.append(&pin_btn);
        sidebar_header.append(&new_tab_btn);
        sidebar_header.append(&new_incognito_tab_btn);
        sidebar_header.append(&sidebar_collapse_btn);

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
        tabs_box.set_size_request(168, -1);

        let normal_tabs_box = gtk4::Box::new(gtk4::Orientation::Vertical, 3);
        normal_tabs_box.set_hexpand(true);

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

        tabs_box.append(&normal_tabs_box);
        tabs_box.append(&incognito_section);
        tabs_scroll.set_child(Some(&tabs_box));

        sidebar_card.append(&sidebar_header);
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

        // Motion controller para auto-ocultar
        let motion_sidebar = gtk4::EventControllerMotion::new();
        let auto_hide_clone = auto_hide_enabled.clone();
        let revealer_leave = revealer.clone();
        motion_sidebar.connect_leave(move |_| {
            if *auto_hide_clone.borrow() {
                let r_clone = revealer_leave.clone();
                let a_clone = auto_hide_clone.clone();
                glib::timeout_add_local_once(std::time::Duration::from_millis(300), move || {
                    if *a_clone.borrow() {
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
                    pin_btn.set_tooltip_text(Some("Auto-ocultar activo (Pasa el cursor por el borde para ver)"));
                    edge_trigger_clone.set_visible(true);
                    revealer.set_reveal_child(false);
                    s_card_clone.remove_css_class("docked-sidebar");
                    s_card_clone.add_css_class("floating-sidebar");
                } else {
                    pin_btn.add_css_class("pinned");
                    pin_btn.set_tooltip_text(Some("Panel fijado permanentemente (Clic para auto-ocultar)"));
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

        Self {
            container,
            edge_trigger,
            revealer,
            sidebar_card,
            pin_btn,
            new_tab_btn,
            new_incognito_tab_btn,
            sidebar_collapse_btn,
            normal_tabs_box,
            incognito_section,
            incognito_tabs_box,
            auto_hide_enabled,
            on_pin_toggled,
        }
    }

    /// Permite conectar un callback cuando se conmuta la fijación del panel lateral
    pub fn set_on_pin_toggled<F: Fn(bool) + 'static>(&self, callback: F) {
        *self.on_pin_toggled.borrow_mut() = Some(Box::new(callback));
    }

    /// Actualiza el modo auto-hide programáticamente (ej. desde ajustes)
    pub fn set_auto_hide(&self, is_auto: bool) {
        *self.auto_hide_enabled.borrow_mut() = is_auto;
        if is_auto {
            self.pin_btn.remove_css_class("pinned");
            self.pin_btn.set_tooltip_text(Some("Auto-ocultar activo (Pasa el cursor por el borde para ver)"));
            self.edge_trigger.set_visible(true);
            self.revealer.set_reveal_child(false);
            self.sidebar_card.remove_css_class("docked-sidebar");
            self.sidebar_card.add_css_class("floating-sidebar");
        } else {
            self.pin_btn.add_css_class("pinned");
            self.pin_btn.set_tooltip_text(Some("Panel fijado permanentemente (Clic para auto-ocultar)"));
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
        self.new_incognito_tab_btn.set_tooltip_text(Some(tr(lang, "new_incognito_tab")));
        self.sidebar_collapse_btn.set_tooltip_text(Some(tr(lang, "collapse_sidebar")));
        if *self.auto_hide_enabled.borrow() {
            self.pin_btn.set_tooltip_text(Some(tr(lang, "sidebar_auto_hide")));
        } else {
            self.pin_btn.set_tooltip_text(Some(tr(lang, "sidebar_pin")));
        }
    }

    /// Conecta soporte de arrastre vertical y menú contextual
    pub fn attach_tab_drag_and_context_menu(
        tab: &BrowserTab,
        tabs: Rc<RefCell<Vec<BrowserTab>>>,
        normal_tabs_box: gtk4::Box,
        incognito_tabs_box: gtk4::Box,
        tab_view: adw::TabView,
    ) {
        let pill = &tab.tab_pill;
        let page = &tab.page;

        let drag_gesture = gtk4::GestureDrag::new();
        let last_offset = Rc::new(RefCell::new(0.0));

        drag_gesture.connect_drag_begin(clone!(
            #[weak]
            pill,
            #[strong]
            last_offset,
            move |_, _, _| {
                *last_offset.borrow_mut() = 0.0;
                pill.add_css_class("dragging");
            }
        ));

        drag_gesture.connect_drag_update(clone!(
            #[strong]
            tabs,
            #[strong]
            normal_tabs_box,
            #[strong]
            incognito_tabs_box,
            #[strong]
            tab_view,
            #[weak]
            page,
            #[strong]
            last_offset,
            move |_, _, offset_y| {
                let prev = *last_offset.borrow();
                if offset_y - prev > 34.0 {
                    move_tab_internal(&tabs, &normal_tabs_box, &incognito_tabs_box, &tab_view, &page, 1);
                    *last_offset.borrow_mut() = offset_y;
                } else if offset_y - prev < -34.0 {
                    move_tab_internal(&tabs, &normal_tabs_box, &incognito_tabs_box, &tab_view, &page, -1);
                    *last_offset.borrow_mut() = offset_y;
                }
            }
        ));

        drag_gesture.connect_drag_end(clone!(
            #[weak]
            pill,
            move |_, _, _| {
                pill.remove_css_class("dragging");
            }
        ));
        pill.add_controller(drag_gesture);

        let right_click = gtk4::GestureClick::new();
        right_click.set_button(3);
        right_click.connect_pressed(clone!(
            #[strong]
            tabs,
            #[strong]
            normal_tabs_box,
            #[strong]
            incognito_tabs_box,
            #[strong]
            tab_view,
            #[weak]
            page,
            #[weak]
            pill,
            move |_, _, _, _| {
                let popover = gtk4::Popover::new();
                popover.add_css_class("tab-menu-popover");
                popover.set_parent(&pill);

                let menu_box = gtk4::Box::new(gtk4::Orientation::Vertical, 2);

                let btn_up = gtk4::Button::with_label("▲ Mover hacia arriba");
                btn_up.add_css_class("tab-menu-btn");
                btn_up.connect_clicked(clone!(
                    #[strong]
                    tabs,
                    #[strong]
                    normal_tabs_box,
                    #[strong]
                    incognito_tabs_box,
                    #[strong]
                    tab_view,
                    #[weak]
                    page,
                    #[weak]
                    popover,
                    move |_| {
                        move_tab_internal(&tabs, &normal_tabs_box, &incognito_tabs_box, &tab_view, &page, -1);
                        popover.popdown();
                    }
                ));

                let btn_down = gtk4::Button::with_label("▼ Mover hacia abajo");
                btn_down.add_css_class("tab-menu-btn");
                btn_down.connect_clicked(clone!(
                    #[strong]
                    tabs,
                    #[strong]
                    normal_tabs_box,
                    #[strong]
                    incognito_tabs_box,
                    #[strong]
                    tab_view,
                    #[weak]
                    page,
                    #[weak]
                    popover,
                    move |_| {
                        move_tab_internal(&tabs, &normal_tabs_box, &incognito_tabs_box, &tab_view, &page, 1);
                        popover.popdown();
                    }
                ));

                let sep = gtk4::Separator::new(gtk4::Orientation::Horizontal);

                let btn_close = gtk4::Button::with_label("✖ Cerrar pestaña");
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

                menu_box.append(&btn_up);
                menu_box.append(&btn_down);
                menu_box.append(&sep);
                menu_box.append(&btn_close);

                popover.set_child(Some(&menu_box));
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
        let pos = tabs_borrow.iter().position(|t| t.page == *page);
        if let Some(current) = pos {
            let target = (current as i32 + delta).clamp(0, (tabs_borrow.len() - 1) as i32) as usize;
            (current, target, tabs_borrow[current].is_incognito)
        } else {
            return;
        }
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
        let sibling_idx = siblings.iter().position(|t| t.page == target_tab.page).unwrap_or(0);
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
