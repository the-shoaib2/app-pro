use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::db::AppEntry;

#[derive(Debug, Clone)]
pub struct DesktopAppInfo {
    pub name: String,
    pub exec: String,
    pub icon: Option<String>,
    pub comment: Option<String>,
    pub categories: Option<String>,
    pub desktop_path: PathBuf,
    pub is_app_pro: bool,
    pub origin: AppOrigin,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppOrigin {
    System,
    User,
    AppPro,
}

impl AppOrigin {
    pub fn as_str(&self) -> &'static str {
        match self {
            AppOrigin::System => "System",
            AppOrigin::User => "User",
            AppOrigin::AppPro => "App Pro",
        }
    }
}

pub fn scan_desktop_apps(app_pro_apps: &[AppEntry]) -> Vec<DesktopAppInfo> {
    let pro_names: HashSet<&str> = app_pro_apps.iter().map(|a| a.name.as_str()).collect();
    let pro_paths: HashSet<&str> = app_pro_apps
        .iter()
        .filter_map(|a| a.desktop_file.as_deref())
        .collect();

    let mut apps = Vec::new();

    // Scan system directory
    if let Ok(entries) = std::fs::read_dir("/usr/share/applications") {
        for entry in entries.flatten() {
            if let Some(info) = parse_desktop_file(&entry.path(), &pro_names, &pro_paths) {
                apps.push(info);
            }
        }
    }

    // Scan user directory
    if let Some(user_dir) = dirs::data_dir().map(|d| d.join("applications")) {
        if let Ok(entries) = std::fs::read_dir(&user_dir) {
            for entry in entries.flatten() {
                if let Some(mut info) = parse_desktop_file(&entry.path(), &pro_names, &pro_paths) {
                    if info.origin == AppOrigin::System {
                        info.origin = AppOrigin::User;
                    }
                    apps.push(info);
                }
            }
        }
    }

    // Also check ~/.local/share/applications directly
    let alt_user = PathBuf::from(
        std::env::var("HOME").unwrap_or_default(),
    )
    .join(".local/share/applications");
    if alt_user != dirs::data_dir().map(|d| d.join("applications")).unwrap_or_default() {
        if let Ok(entries) = std::fs::read_dir(&alt_user) {
            for entry in entries.flatten() {
                if let Some(mut info) = parse_desktop_file(&entry.path(), &pro_names, &pro_paths) {
                    if info.origin == AppOrigin::System {
                        info.origin = AppOrigin::User;
                    }
                    apps.push(info);
                }
            }
        }
    }

    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    apps.dedup_by(|a, b| a.name.to_lowercase() == b.name.to_lowercase());
    apps
}

fn parse_desktop_file(
    path: &Path,
    pro_names: &HashSet<&str>,
    pro_paths: &HashSet<&str>,
) -> Option<DesktopAppInfo> {
    if path.extension().and_then(|e| e.to_str()) != Some("desktop") {
        return None;
    }

    let content = std::fs::read_to_string(path).ok()?;

    let mut name = None;
    let mut exec = None;
    let mut icon = None;
    let mut comment = None;
    let mut categories = None;
    let mut no_display = false;
    let mut is_app_pro = false;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.starts_with('[') {
            continue;
        }
        if let Some(val) = line.strip_prefix("Name=") {
            name = Some(val.to_string());
        } else if let Some(val) = line.strip_prefix("Exec=") {
            exec = Some(val.to_string());
        } else if let Some(val) = line.strip_prefix("Icon=") {
            icon = Some(val.to_string());
        } else if let Some(val) = line.strip_prefix("Comment=") {
            comment = Some(val.to_string());
        } else if let Some(val) = line.strip_prefix("Categories=") {
            categories = Some(val.to_string());
        } else if let Some(val) = line.strip_prefix("NoDisplay=") {
            no_display = val.trim() == "true";
        } else if let Some(val) = line.strip_prefix("X-AppPro-Managed=") {
            is_app_pro = val.trim() == "true";
        }
    }

    let name = name?;
    if no_display {
        return None;
    }

    let is_pro_name = pro_names.contains(name.as_str());
    let pro_path_str = path.to_string_lossy();
    let is_pro_path_matched = pro_paths.contains(pro_path_str.as_ref());
    let is_app_pro = is_app_pro || is_pro_name || is_pro_path_matched;

    let origin = if is_app_pro {
        AppOrigin::AppPro
    } else {
        AppOrigin::System
    };

    Some(DesktopAppInfo {
        name,
        exec: exec.unwrap_or_default(),
        icon,
        comment,
        categories,
        desktop_path: path.to_path_buf(),
        is_app_pro,
        origin,
    })
}
