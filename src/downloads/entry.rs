use std::path::PathBuf;
use webkit6::Download;

#[derive(Clone, Debug, PartialEq)]
pub enum DownloadState {
    InProgress,
    Completed,
    Failed(String),
    Cancelled,
}

#[allow(dead_code)]
pub struct DownloadEntry {
    pub id: u64,
    pub filename: String,
    pub path: PathBuf,
    pub progress: f64,
    pub state: DownloadState,
    pub download: Download,
}

pub fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

pub fn get_unique_download_path(dir: &std::path::Path, suggested_name: &str) -> PathBuf {
    let target = dir.join(suggested_name);
    if !target.exists() {
        return target;
    }

    let p = std::path::Path::new(suggested_name);
    let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("descarga");
    let ext = p.extension().and_then(|e| e.to_str());

    let mut counter = 1;
    loop {
        let new_name = match ext {
            Some(extension) => format!("{} ({}).{}", stem, counter, extension),
            None => format!("{} ({})", stem, counter),
        };
        let candidate = dir.join(&new_name);
        if !candidate.exists() {
            return candidate;
        }
        counter += 1;
    }
}
