use std::process::Command;

/// ============================================================================
/// 🏷️ CONFIGURACIÓN DE VERSIÓN DE COMPILACIÓN DE NIKA BROWSER
/// Modifica aquí la versión para compilar nuevos binarios y lanzamientos / releases:
/// ============================================================================
pub const APP_VERSION: &str = "0.1.0-alpha.1";

fn main() {
    // 1. Exportar la versión definida para el código fuente
    println!("cargo:rustc-env=NIKA_APP_VERSION={}", APP_VERSION);

    // 2. Obtener fecha y hora de compilación exacta
    let build_date = match Command::new("date").arg("+%Y-%m-%d %H:%M:%S %z").output() {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        }
        _ => "2026-10-31 20:30:00 -0500".to_string(),
    };
    println!("cargo:rustc-env=NIKA_BUILD_DATE={}", build_date);

    // 3. Obtener el commit corto de git si está disponible
    let git_commit = match Command::new("git").args(["rev-parse", "--short", "HEAD"]).output() {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        }
        _ => "release".to_string(),
    };
    println!("cargo:rustc-env=NIKA_GIT_COMMIT={}", git_commit);

    // 4. Target de arquitectura de compilación
    if let Ok(target) = std::env::var("TARGET") {
        println!("cargo:rustc-env=NIKA_BUILD_TARGET={}", target);
    } else {
        println!("cargo:rustc-env=NIKA_BUILD_TARGET=x86_64-unknown-linux-gnu");
    }

    // Indicar a Cargo vigilar cambios relevantes
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=src");
}
