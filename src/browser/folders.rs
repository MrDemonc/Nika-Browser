use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

pub const DEFAULT_FOLDER_COLORS: &[(&str, &str)] = &[
    ("Cyan", "#00d2d3"),
    ("Menta", "#1dd1a1"),
    ("Lavanda", "#a29bfe"),
    ("Rosa", "#fd79a8"),
    ("Coral", "#ff7675"),
    ("Ámbar", "#ffeaa7"),
];

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TabFolderItem {
    pub id: String,
    pub title: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TabFolder {
    pub id: String,
    pub name: String,
    pub color: String,
    pub is_expanded: bool,
    pub tabs: Vec<TabFolderItem>,
}

fn default_true() -> bool {
    true
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TabFoldersState {
    #[serde(default = "default_true")]
    pub is_section_expanded: bool,
    pub folders: Vec<TabFolder>,
}

impl Default for TabFoldersState {
    fn default() -> Self {
        Self {
            is_section_expanded: true,
            folders: Vec::new(),
        }
    }
}

pub struct TabFoldersManager {
    pub folders: Rc<RefCell<Vec<TabFolder>>>,
    pub is_section_expanded: Rc<RefCell<bool>>,
    config_path: PathBuf,
}

fn generate_id(prefix: &str) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("{}-{}-{}", prefix, now, glib::random_int())
}

impl TabFoldersManager {
    pub fn new() -> Self {
        let dir = glib::user_config_dir().join("nika-browser");
        let _ = fs::create_dir_all(&dir);
        let config_path = dir.join("tab_folders.json");

        let (is_expanded, loaded_folders) = Self::load_from_path(&config_path);

        Self {
            folders: Rc::new(RefCell::new(loaded_folders)),
            is_section_expanded: Rc::new(RefCell::new(is_expanded)),
            config_path,
        }
    }

    fn load_from_path(path: &PathBuf) -> (bool, Vec<TabFolder>) {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(state) = serde_json::from_str::<TabFoldersState>(&content) {
                    return (state.is_section_expanded, state.folders);
                }
            }
        }
        (true, Vec::new())
    }

    pub fn save(&self) {
        let state = TabFoldersState {
            is_section_expanded: *self.is_section_expanded.borrow(),
            folders: self.folders.borrow().clone(),
        };
        if let Ok(json) = serde_json::to_string_pretty(&state) {
            let _ = fs::write(&self.config_path, json);
        }
    }

    pub fn is_section_expanded(&self) -> bool {
        *self.is_section_expanded.borrow()
    }

    pub fn toggle_section_expanded(&self) -> bool {
        let mut exp = self.is_section_expanded.borrow_mut();
        *exp = !*exp;
        let val = *exp;
        drop(exp);
        self.save();
        val
    }

    pub fn get_folders(&self) -> Vec<TabFolder> {
        self.folders.borrow().clone()
    }

    pub fn create_folder(&self, name: &str, color: &str) -> TabFolder {
        let trimmed_name = if name.trim().is_empty() {
            "Nueva Carpeta"
        } else {
            name.trim()
        };
        let valid_color = if color.trim().is_empty() {
            "#00d2d3"
        } else {
            color.trim()
        };

        let folder = TabFolder {
            id: generate_id("folder"),
            name: trimmed_name.to_string(),
            color: valid_color.to_string(),
            is_expanded: true,
            tabs: Vec::new(),
        };

        self.folders.borrow_mut().push(folder.clone());
        self.save();
        folder
    }

    pub fn delete_folder(&self, folder_id: &str) -> bool {
        let mut folders = self.folders.borrow_mut();
        if let Some(pos) = folders.iter().position(|f| f.id == folder_id) {
            folders.remove(pos);
            drop(folders);
            self.save();
            true
        } else {
            false
        }
    }

    pub fn rename_folder(&self, folder_id: &str, new_name: &str) -> bool {
        let mut folders = self.folders.borrow_mut();
        if let Some(folder) = folders.iter_mut().find(|f| f.id == folder_id) {
            let clean = new_name.trim();
            if !clean.is_empty() {
                folder.name = clean.to_string();
                drop(folders);
                self.save();
                return true;
            }
        }
        false
    }

    pub fn set_folder_color(&self, folder_id: &str, color: &str) -> bool {
        let mut folders = self.folders.borrow_mut();
        if let Some(folder) = folders.iter_mut().find(|f| f.id == folder_id) {
            folder.color = color.to_string();
            drop(folders);
            self.save();
            return true;
        }
        false
    }

    pub fn toggle_expanded(&self, folder_id: &str) -> bool {
        let mut folders = self.folders.borrow_mut();
        if let Some(folder) = folders.iter_mut().find(|f| f.id == folder_id) {
            folder.is_expanded = !folder.is_expanded;
            let current = folder.is_expanded;
            drop(folders);
            self.save();
            return current;
        }
        false
    }

    pub fn add_tab(&self, folder_id: &str, title: &str, url: &str) -> Option<TabFolderItem> {
        let trimmed_url = url.trim();
        if trimmed_url.is_empty() {
            return None;
        }

        let display_title = if title.trim().is_empty() {
            trimmed_url
        } else {
            title.trim()
        };

        let mut folders = self.folders.borrow_mut();
        if let Some(folder) = folders.iter_mut().find(|f| f.id == folder_id) {
            // Evitar duplicados idénticos consecutivos en la misma carpeta
            if let Some(existing) = folder.tabs.iter().find(|t| t.url == trimmed_url) {
                return Some(existing.clone());
            }

            let item = TabFolderItem {
                id: generate_id("tab"),
                title: display_title.to_string(),
                url: trimmed_url.to_string(),
            };
            folder.tabs.push(item.clone());
            drop(folders);
            self.save();
            Some(item)
        } else {
            None
        }
    }

    pub fn remove_tab(&self, folder_id: &str, tab_id: &str) -> bool {
        let mut folders = self.folders.borrow_mut();
        if let Some(folder) = folders.iter_mut().find(|f| f.id == folder_id) {
            if let Some(pos) = folder.tabs.iter().position(|t| t.id == tab_id) {
                folder.tabs.remove(pos);
                drop(folders);
                self.save();
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_folders_state_serialization() {
        let mut state = TabFoldersState::default();
        state.folders.push(TabFolder {
            id: "f1".to_string(),
            name: "Trabajo".to_string(),
            color: "#00d2d3".to_string(),
            is_expanded: true,
            tabs: vec![TabFolderItem {
                id: "t1".to_string(),
                title: "GitHub".to_string(),
                url: "https://github.com".to_string(),
            }],
        });

        let json = serde_json::to_string(&state).expect("serialize");
        let decoded: TabFoldersState = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(state.folders.len(), decoded.folders.len());
        assert_eq!(state.folders[0].name, decoded.folders[0].name);
        assert_eq!(state.folders[0].tabs.len(), decoded.folders[0].tabs.len());
        assert_eq!(state.folders[0].tabs[0].url, "https://github.com");
    }

    #[test]
    fn test_folders_manager_lifecycle() {
        let temp_dir = std::env::temp_dir().join(format!("nika_test_folders_{}", glib::random_int()));
        let _ = fs::create_dir_all(&temp_dir);
        let path = temp_dir.join("tab_folders.json");

        let manager = TabFoldersManager {
            folders: Rc::new(RefCell::new(Vec::new())),
            is_section_expanded: Rc::new(RefCell::new(true)),
            config_path: path.clone(),
        };

        // Crear carpeta
        let f1 = manager.create_folder("Estudio", "#1dd1a1");
        assert_eq!(f1.name, "Estudio");
        assert_eq!(f1.color, "#1dd1a1");
        assert_eq!(manager.get_folders().len(), 1);

        // Añadir pestañas
        let tab1 = manager.add_tab(&f1.id, "Rust Book", "https://doc.rust-lang.org/book/").expect("add tab 1");
        assert_eq!(tab1.title, "Rust Book");
        assert_eq!(manager.get_folders()[0].tabs.len(), 1);

        let tab2 = manager.add_tab(&f1.id, "Arch Wiki", "https://wiki.archlinux.org/").expect("add tab 2");
        assert_eq!(tab2.title, "Arch Wiki");
        assert_eq!(manager.get_folders()[0].tabs.len(), 2);

        // Renombrar carpeta
        assert!(manager.rename_folder(&f1.id, "Programación"));
        assert_eq!(manager.get_folders()[0].name, "Programación");

        // Cambiar color
        assert!(manager.set_folder_color(&f1.id, "#a29bfe"));
        assert_eq!(manager.get_folders()[0].color, "#a29bfe");

        // Toggle expandido
        let expanded = manager.toggle_expanded(&f1.id);
        assert!(!expanded);
        assert_eq!(manager.get_folders()[0].is_expanded, false);

        // Quitar una pestaña
        assert!(manager.remove_tab(&f1.id, &tab1.id));
        assert_eq!(manager.get_folders()[0].tabs.len(), 1);
        assert_eq!(manager.get_folders()[0].tabs[0].url, "https://wiki.archlinux.org/");

        // Eliminar carpeta
        assert!(manager.delete_folder(&f1.id));
        assert_eq!(manager.get_folders().len(), 0);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
