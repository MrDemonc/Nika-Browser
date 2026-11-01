use crate::settings::BrowserSettings;
use glib::clone;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub const SUPPORTED_LANGUAGES: &[(&str, &str)] = &[
    ("es", "Español"),
    ("en", "English"),
    ("fr", "Français"),
    ("de", "Deutsch"),
    ("it", "Italiano"),
    ("pt", "Português"),
    ("zh-CN", "中文 (Chino)"),
    ("ja", "日本語 (Japonés)"),
    ("ru", "Русский (Ruso)"),
    ("ar", "العربية (Árabe)"),
    ("ko", "한국어 (Coreano)"),
    ("hi", "हिन्दी (Hindi)"),
    ("nl", "Nederlands"),
    ("pl", "Polski"),
    ("tr", "Türkçe"),
];

#[allow(dead_code)]
pub struct TranslationPopover {
    pub popover: gtk4::Popover,
    pub auto_translate_switch: gtk4::Switch,
    pub lang_dropdown: gtk4::DropDown,
    pub title_label: gtk4::Label,
    pub auto_label: gtk4::Label,
    pub lang_label: gtk4::Label,
    pub translate_now_btn: gtk4::Button,
    pub restore_btn: gtk4::Button,
    pub settings: Rc<RefCell<BrowserSettings>>,
    on_translate: Rc<RefCell<Option<Box<dyn Fn(String)>>>>,
    on_restore: Rc<RefCell<Option<Box<dyn Fn()>>>>,
}

impl TranslationPopover {
    pub fn new(parent_btn: &gtk4::Button, settings: Rc<RefCell<BrowserSettings>>) -> Self {
        let popover = gtk4::Popover::new();
        popover.set_parent(parent_btn);
        popover.add_css_class("translation-popover");
        popover.set_position(gtk4::PositionType::Right);

        let content_box = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
        content_box.add_css_class("translation-content-box");
        content_box.set_size_request(240, -1);

        // Cabecera del popover
        let header_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        header_row.add_css_class("translation-header-row");

        let icon = gtk4::Image::from_icon_name("nika-translate-symbolic");
        icon.set_pixel_size(16);

        let lang_code = settings.borrow().language.clone();
        let title_label = gtk4::Label::new(Some(crate::i18n::tr(&lang_code, "translate_title")));
        title_label.add_css_class("translation-title");
        title_label.set_hexpand(true);
        title_label.set_halign(gtk4::Align::Start);

        let close_btn = gtk4::Button::from_icon_name("nika-close-symbolic");
        close_btn.add_css_class("sidebar-action-btn");
        close_btn.set_tooltip_text(Some(crate::i18n::tr(&lang_code, "close")));
        close_btn.connect_clicked(clone!(
            #[weak]
            popover,
            move |_| {
                popover.popdown();
            }
        ));

        header_row.append(&icon);
        header_row.append(&title_label);
        header_row.append(&close_btn);

        let sep = gtk4::Separator::new(gtk4::Orientation::Horizontal);

        // Opción 1: Switch sugerir automáticamente
        let auto_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        auto_row.set_valign(gtk4::Align::Center);

        let auto_label = gtk4::Label::new(Some(crate::i18n::tr(&lang_code, "translate_auto_suggest")));
        auto_label.add_css_class("translation-label");
        auto_label.set_hexpand(true);
        auto_label.set_halign(gtk4::Align::Start);
        auto_label.set_wrap(true);
        auto_label.set_max_width_chars(20);

        let auto_translate_switch = gtk4::Switch::new();
        auto_translate_switch.set_active(settings.borrow().auto_translate);
        auto_translate_switch.set_valign(gtk4::Align::Center);

        let s_switch = settings.clone();
        auto_translate_switch.connect_active_notify(move |sw| {
            s_switch.borrow_mut().auto_translate = sw.is_active();
            let _ = s_switch.borrow().save();
        });

        auto_row.append(&auto_label);
        auto_row.append(&auto_translate_switch);

        // Opción 2: Selector de idioma objetivo
        let lang_row = gtk4::Box::new(gtk4::Orientation::Vertical, 4);

        let lang_label = gtk4::Label::new(Some(crate::i18n::tr(&lang_code, "translate_target_lang")));
        lang_label.add_css_class("translation-label");
        lang_label.set_halign(gtk4::Align::Start);

        let lang_names: Vec<&str> = SUPPORTED_LANGUAGES.iter().map(|(_, name)| *name).collect();
        let lang_dropdown = gtk4::DropDown::from_strings(&lang_names);
        lang_dropdown.add_css_class("translation-dropdown");

        let current_target = settings.borrow().translate_target_lang.clone();
        if let Some(pos) = SUPPORTED_LANGUAGES.iter().position(|(code, _)| *code == current_target) {
            lang_dropdown.set_selected(pos as u32);
        } else {
            lang_dropdown.set_selected(0);
        }

        let s_drop = settings.clone();
        let on_translate_ref: Rc<RefCell<Option<Box<dyn Fn(String)>>>> = Rc::new(RefCell::new(None));
        let on_trans_clone = on_translate_ref.clone();
        lang_dropdown.connect_selected_notify(move |dd| {
            let idx = dd.selected() as usize;
            if idx < SUPPORTED_LANGUAGES.len() {
                let code = SUPPORTED_LANGUAGES[idx].0.to_string();
                s_drop.borrow_mut().translate_target_lang = code.clone();
                let _ = s_drop.borrow().save();
                if let Some(ref cb) = *on_trans_clone.borrow() {
                    cb(code);
                }
            }
        });

        lang_row.append(&lang_label);
        lang_row.append(&lang_dropdown);

        // Botones de acción rápida
        let actions_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        actions_row.set_halign(gtk4::Align::End);

        let translate_now_btn = gtk4::Button::with_label(crate::i18n::tr(&lang_code, "translate_now"));
        translate_now_btn.add_css_class("suggested-action");
        translate_now_btn.add_css_class("translation-action-btn");

        let restore_btn = gtk4::Button::with_label(crate::i18n::tr(&lang_code, "translate_restore"));
        restore_btn.add_css_class("translation-action-btn");

        actions_row.append(&restore_btn);
        actions_row.append(&translate_now_btn);

        content_box.append(&header_row);
        content_box.append(&sep);
        content_box.append(&auto_row);
        content_box.append(&lang_row);
        content_box.append(&actions_row);

        popover.set_child(Some(&content_box));

        let on_restore_ref: Rc<RefCell<Option<Box<dyn Fn()>>>> = Rc::new(RefCell::new(None));

        let on_trans_btn = on_translate_ref.clone();
        let s_btn = settings.clone();
        let pop_close = popover.clone();
        translate_now_btn.connect_clicked(move |_| {
            let code = s_btn.borrow().translate_target_lang.clone();
            if let Some(ref cb) = *on_trans_btn.borrow() {
                cb(code);
            }
            pop_close.popdown();
        });

        let on_rest_btn = on_restore_ref.clone();
        let pop_close2 = popover.clone();
        restore_btn.connect_clicked(move |_| {
            if let Some(ref cb) = *on_rest_btn.borrow() {
                cb();
            }
            pop_close2.popdown();
        });

        Self {
            popover,
            auto_translate_switch,
            lang_dropdown,
            title_label,
            auto_label,
            lang_label,
            translate_now_btn,
            restore_btn,
            settings,
            on_translate: on_translate_ref,
            on_restore: on_restore_ref,
        }
    }

    pub fn set_on_translate<F: Fn(String) + 'static>(&self, callback: F) {
        *self.on_translate.borrow_mut() = Some(Box::new(callback));
    }

    pub fn set_on_restore<F: Fn() + 'static>(&self, callback: F) {
        *self.on_restore.borrow_mut() = Some(Box::new(callback));
    }

    pub fn popup(&self) {
        self.popover.popup();
    }

    pub fn update_language(&self, lang: &str) {
        use crate::i18n::tr;
        self.title_label.set_text(tr(lang, "translate_title"));
        self.auto_label.set_text(tr(lang, "translate_auto_suggest"));
        self.lang_label.set_text(tr(lang, "translate_target_lang"));
        self.translate_now_btn.set_label(tr(lang, "translate_now"));
        self.restore_btn.set_label(tr(lang, "translate_restore"));
    }
}
