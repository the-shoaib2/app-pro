use crate::core::SystemExec;
use crate::db::AppEntry;
use crate::installer::InstallResult;
use std::path::{Path, PathBuf};
use std::fs;

pub struct AppImageInstaller;

impl AppImageInstaller {
    pub fn install<P: AsRef<Path>>(path: P) -> InstallResult {
        let path = path.as_ref();
        let filename = path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let app_name = Self::extract_name(&filename);
        let app_dir = Self::get_apps_dir().join(&app_name);
        let target_path = app_dir.join(&filename);

        log::info!("Installing AppImage: {} -> {}", path.display(), target_path.display());

        // Create app directory
        if let Err(e) = fs::create_dir_all(&app_dir) {
            return InstallResult {
                success: false,
                message: format!("Failed to create app directory: {}", e),
                app_name,
                install_path: String::new(),
                icon_path: None,
                version: None,
                size_bytes: 0,
            };
        }

        // Copy AppImage to apps directory
        if let Err(e) = fs::copy(path, &target_path) {
            return InstallResult {
                success: false,
                message: format!("Failed to copy AppImage: {}", e),
                app_name,
                install_path: String::new(),
                icon_path: None,
                version: None,
                size_bytes: 0,
            };
        }

        // Make executable
        use std::os::unix::fs::PermissionsExt;
        if let Err(e) = fs::set_permissions(&target_path, fs::Permissions::from_mode(0o755)) {
            return InstallResult {
                success: false,
                message: format!("Failed to set permissions: {}", e),
                app_name,
                install_path: target_path.to_string_lossy().to_string(),
                icon_path: None,
                version: None,
                size_bytes: 0,
            };
        }

        // Extract icon from AppImage if possible
        let icon_path = Self::extract_icon(&target_path, &app_name);

        // Create .desktop entry
        let desktop_result = Self::create_desktop_entry(&app_name, &target_path, &icon_path);

        let size = path.metadata().map(|m| m.len() as i64).unwrap_or(0);
        let install_path_str = target_path.to_string_lossy().to_string();

        let message = match &desktop_result {
            Ok(_) => format!("Successfully installed {}", app_name),
            Err(e) => format!("Installed {} but desktop entry creation failed: {}", app_name, e),
        };

        InstallResult {
            success: true,
            message,
            app_name,
            install_path: install_path_str,
            icon_path,
            version: None,
            size_bytes: size,
        }
    }

    pub fn uninstall(app: &AppEntry) -> InstallResult {
        log::info!("Uninstalling AppImage: {}", app.name);

        let app_dir = Self::get_apps_dir().join(&app.name);

        // Remove app directory
        if app_dir.exists() {
            if let Err(e) = fs::remove_dir_all(&app_dir) {
                return InstallResult {
                    success: false,
                    message: format!("Failed to remove app directory: {}", e),
                    app_name: app.name.clone(),
                    install_path: String::new(),
                    icon_path: None,
                    version: None,
                    size_bytes: 0,
                };
            }
        }

        // Remove desktop file
        let desktop_path = Self::get_desktop_path(&app.name);
        if desktop_path.exists() {
            fs::remove_file(&desktop_path).ok();
        }

        // Remove icon
        if let Some(ref icon) = app.icon_path {
            let icon_path = Path::new(icon);
            if icon_path.exists() {
                fs::remove_file(icon_path).ok();
            }
        }

        InstallResult {
            success: true,
            message: format!("Successfully uninstalled {}", app.name),
            app_name: app.name.clone(),
            install_path: String::new(),
            icon_path: None,
            version: None,
            size_bytes: 0,
        }
    }

    fn get_apps_dir() -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        home.join(".local").join("share").join("app-pro").join("apps")
    }

    fn get_desktop_path(app_name: &str) -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        home.join(".local").join("share").join("applications")
            .join(format!("app-pro-{}.desktop", app_name))
    }

    fn extract_name(filename: &str) -> String {
        let name = filename
            .replace(".AppImage", "")
            .replace("-x86_64", "")
            .replace("-amd64", "")
            .replace("-linux", "")
            .replace("_", "-");
        // Get just the app name part (before version numbers)
        let parts: Vec<&str> = name.split('-').collect();
        if parts.len() > 1 && parts.last().map_or(false, |p| p.chars().next().map_or(false, |c| c.is_ascii_digit())) {
            parts[..parts.len()-1].join("-")
        } else {
            name
        }
    }

    fn extract_icon(appimage_path: &Path, app_name: &str) -> Option<String> {
        // Try to extract icon embedded in AppImage using --appimage-extract
        let _extract_dir = PathBuf::from("/tmp").join(format!("appimage-extract-{}", app_name));

        let result = SystemExec::run(appimage_path.to_str().unwrap_or(""), &["--appimage-extract"]);
        if result.is_ok() && result.unwrap().success {
            // AppImage extracts to ./squashfs-root
            let squashfs = PathBuf::from("squashfs-root");
            if squashfs.exists() {
                let icon = Self::find_icon_in_dir(&squashfs, app_name);
                if let Some(icon_path) = icon {
                    let dest = Self::get_apps_dir().join(&format!("{}.png", app_name));
                    fs::create_dir_all(dest.parent().unwrap()).ok();
                    fs::copy(&icon_path, &dest).ok();
                    fs::remove_dir_all(&squashfs).ok();
                    return Some(dest.to_string_lossy().to_string());
                }
                fs::remove_dir_all(&squashfs).ok();
            }
        }

        None
    }

    fn find_icon_in_dir(dir: &Path, app_name: &str) -> Option<PathBuf> {
        // Look for .png or .svg icons in common locations
        let search_patterns = [
            format!("{}.png", app_name),
            format!("{}.svg", app_name),
            format!("{}.xpm", app_name),
            "icon.png".to_string(),
            "logo.png".to_string(),
        ];

        for _pattern in &search_patterns {
            let icon_path = dir.join(".DirIcon");
            if icon_path.exists() {
                return Some(icon_path);
            }
            // Search recursively in usr/share/icons
            let icons_dir = dir.join("usr/share/icons");
            if icons_dir.exists() {
                if let Ok(entries) = fs::read_dir(&icons_dir) {
                    for entry in entries.flatten() {
                        let p = entry.path();
                        if p.is_dir() {
                            if let Ok(subs) = fs::read_dir(&p) {
                                for sub in subs.flatten() {
                                    let sp = sub.path();
                                    if sp.is_dir() {
                                        if let Ok(subsubs) = fs::read_dir(&sp) {
                                            for subsub in subsubs.flatten() {
                                                let ssp = subsub.path();
                                                if ssp.is_dir() {
                                                    if let Ok(app_icons) = fs::read_dir(&ssp) {
                                                        for app_icon in app_icons.flatten() {
                                                            let name = app_icon.path().file_name()
                                                                .and_then(|n| n.to_str())
                                                                .unwrap_or("")
                                                                .to_lowercase();
                                                            if name.contains(&app_name.to_lowercase()) {
                                                                return Some(app_icon.path());
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }

    fn create_desktop_entry(app_name: &str, exec_path: &Path, icon_path: &Option<String>) -> Result<(), String> {
        let desktop_path = Self::get_desktop_path(app_name);
        if let Some(parent) = desktop_path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        let icon_line = match icon_path {
            Some(icon) => format!("Icon={}", icon),
            None => "Icon=application-x-executable".to_string(),
        };

        let content = format!(
            "[Desktop Entry]\n\
             Type=Application\n\
             Name={name}\n\
             Exec={exec}\n\
             {icon}\n\
             Terminal=false\n\
             Categories=Utility;\n\
             Comment={name} - Installed via App Pro\n\
             X-AppPro-Managed=true\n",
            name = app_name,
            exec = exec_path.display(),
            icon = icon_line,
        );

        fs::write(&desktop_path, content).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_installed_apps() -> Vec<(String, String, u64)> {
        let apps_dir = Self::get_apps_dir();
        if !apps_dir.exists() {
            return Vec::new();
        }

        let mut apps = Vec::new();
        if let Ok(entries) = fs::read_dir(&apps_dir) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    if let Ok(mut dir_entries) = fs::read_dir(entry.path()) {
                        if let Some(first_file) = dir_entries.next() {
                            if let Ok(f) = first_file {
                                let path = f.path();
                                let size = path.metadata().map(|m| m.len()).unwrap_or(0);
                                apps.push((
                                    entry.file_name().to_string_lossy().to_string(),
                                    path.to_string_lossy().to_string(),
                                    size,
                                ));
                            }
                        }
                    }
                }
            }
        }
        apps
    }
}
