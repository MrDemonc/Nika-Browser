mod app;
mod browser;
mod downloads;
pub mod i18n;
pub mod passwords;
mod privacy;
mod search;
mod security;
mod settings;
mod theme;
mod ui;
pub mod version;

use app::BrowserApp;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::path::PathBuf;
use theme::ThemeManager;

fn find_resources_dir() -> PathBuf {
    let mut candidates = Vec::new();
    candidates.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")));
    if let Ok(current) = std::env::current_dir() {
        candidates.push(current);
    }
    candidates.push(glib::user_data_dir().join("nika-browser"));
    if let Ok(exe) = std::env::current_exe()
        && let Some(parent) = exe.parent()
    {
        candidates.push(parent.to_path_buf());
    }

    for base in candidates {
        if base.join("resources").is_dir() {
            return base;
        }
    }

    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn main() -> glib::ExitCode {
    // Manejo de flags de línea de comandos (--version, -v, --help)
    let args: Vec<String> = std::env::args().collect();
    if args
        .iter()
        .any(|a| a == "--version" || a == "-v" || a == "-V")
    {
        println!("{}", version::version_info_string());
        return glib::ExitCode::SUCCESS;
    }
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("🌸 {} v{}", version::APP_NAME, version::VERSION);
        println!("Navegador web moderno, privado y rápido en Rust\n");
        println!("Uso: nika-browser [OPCIONES] [URL...]\n");
        println!("Opciones:");
        println!("  -v, --version    Muestra la versión y fecha de compilación del binario");
        println!("  -h, --help       Muestra esta ayuda de línea de comandos");
        println!("\nRepositorio: https://github.com/MrDemonc/Nika-Browser");
        return glib::ExitCode::SUCCESS;
    }

    println!(
        "[Nika] 🚀 {} v{} (compilado: {})",
        version::APP_NAME,
        version::VERSION,
        version::BUILD_DATE
    );

    // Configurar ruta de códecs GStreamer locales si existen (~/.local/lib/gstreamer-1.0)
    let local_gst_path = glib::home_dir().join(".local/lib/gstreamer-1.0");
    if local_gst_path.exists() {
        if let Some(existing) = std::env::var_os("GST_PLUGIN_PATH") {
            let mut paths = std::env::split_paths(&existing).collect::<Vec<_>>();
            if !paths.iter().any(|p| p == &local_gst_path) {
                paths.insert(0, local_gst_path);
            }
            if let Ok(new_val) = std::env::join_paths(paths) {
                unsafe {
                    std::env::set_var("GST_PLUGIN_PATH", new_val);
                }
            }
        } else {
            unsafe {
                std::env::set_var("GST_PLUGIN_PATH", &local_gst_path);
            }
        }
    }

    // Inicializar Libadwaita
    adw::init().expect("Error al inicializar Libadwaita");

    // Crear aplicación GTK4 / Libadwaita
    let app = adw::Application::builder()
        .application_id("org.nika.browser")
        .build();

    app.connect_startup(|_| {
        let project_dir = find_resources_dir();

        let css_path = project_dir.join("resources/styles/pastel.css");
        let icons_path = project_dir.join("resources/icons");

        // Cargar estilos personalizados
        ThemeManager::load_css(&css_path);

        // Registrar iconos vectoriales y logos personalizados de Nika Browser
        if let Some(display) = gtk4::gdk::Display::default() {
            let icon_theme = gtk4::IconTheme::for_display(&display);
            icon_theme.add_search_path(&icons_path);
            icon_theme.add_search_path(icons_path.join("hicolor"));
            let user_icons = glib::home_dir().join(".local/share/icons");
            if user_icons.exists() {
                icon_theme.add_search_path(&user_icons);
            }
        }

        // Establecer icono predeterminado de la aplicación
        gtk4::Window::set_default_icon_name("nika-browser");
    });

    app.connect_activate(|application| {
        let project_dir = find_resources_dir();
        let browser = BrowserApp::new(application, project_dir);
        browser.present();
    });

    app.run()
}
