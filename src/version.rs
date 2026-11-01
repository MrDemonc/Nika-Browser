/// Módulo de versión y metadatos de compilación de Nika Browser

pub const APP_NAME: &str = "Nika Browser";
pub const APP_ID: &str = "org.nika.browser";

/// Versión configurada en tiempo de compilación (desde build.rs)
pub const VERSION: &str = env!("NIKA_APP_VERSION");

/// Fecha y hora exacta en la que se compiló este binario
pub const BUILD_DATE: &str = env!("NIKA_BUILD_DATE");

/// Hash corto del commit git al momento de compilar
pub const GIT_COMMIT: &str = env!("NIKA_GIT_COMMIT");

/// Arquitectura y target de la compilación
pub const BUILD_TARGET: &str = env!("NIKA_BUILD_TARGET");

/// Devuelve el banner formateado completo de información de versión
pub fn version_info_string() -> String {
    format!(
        "{} v{}\nCompilado el: {}\nTarget: {} (git: {})\nLicencia: GNU GPL v3.0",
        APP_NAME, VERSION, BUILD_DATE, BUILD_TARGET, GIT_COMMIT
    )
}
