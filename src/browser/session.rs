use super::tab::BrowserTab;
use libadwaita as adw;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionState {
    pub urls: Vec<String>,
    pub active_index: usize,
}

/// Guarda el estado de la sesión activa en session.json excluyendo pestañas de incógnito
pub fn save_session_data(tabs: &Rc<RefCell<Vec<BrowserTab>>>, tab_view: &adw::TabView) {
    let tabs_b = tabs.borrow();
    let mut urls = Vec::new();
    let mut active_index = 0;
    let selected_page = tab_view.selected_page();

    for tab in tabs_b.iter().filter(|t| !t.is_incognito) {
        let url = tab.current_url.borrow().clone();
        if url.is_empty() {
            continue;
        }
        if let Some(ref sel) = selected_page
            && tab.page == *sel
        {
            active_index = urls.len();
        }
        urls.push(url);
    }

    if urls.is_empty() {
        urls.push("nika://home".to_string());
    }

    let state = SessionState { urls, active_index };

    let dir = glib::user_config_dir().join("nika-browser");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("session.json");
    if let Ok(json) = serde_json::to_string_pretty(&state) {
        let _ = std::fs::write(path, json);
    }
}
