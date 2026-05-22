use crate::core::SystemExec;
use crate::db::AppEntry;
use crate::installer::InstallResult;
use std::path::Path;

pub struct DebInstaller;

impl DebInstaller {
    pub fn install<P: AsRef<Path>>(path: P) -> InstallResult {
        let path = path.as_ref();
        let _filename = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Extract package name and version from .deb metadata
        let info = Self::get_deb_info(path);

        log::info!("Installing .deb package: {}", path.display());

        let result = SystemExec::run_with_pkexec(&format!(
            "dpkg -i '{}' 2>&1; if [ $? -ne 0 ]; then apt-get install -f -y 2>&1; fi",
            path.display()
        ));

        match result {
            Ok(exec) => {
                let success = exec.success;
                let message = if success {
                    format!("Successfully installed {}", info.0)
                } else {
                    format!("Installation failed: {}", exec.stderr)
                };

                let install_path = format!("/usr/share/{}", info.0);
                let size = path.metadata().map(|m| m.len() as i64).unwrap_or(0);

                let app_name = info.0;
                let version = info.1;
                InstallResult {
                    success,
                    message,
                    app_name: app_name.clone(),
                    install_path,
                    icon_path: Self::find_icon(&app_name),
                    version: Some(version),
                    size_bytes: size,
                }
            }
            Err(e) => InstallResult {
                success: false,
                message: format!("Failed to execute dpkg: {}", e),
                app_name: info.0,
                install_path: String::new(),
                icon_path: None,
                version: Some(info.1),
                size_bytes: 0,
            },
        }
    }

    pub fn uninstall(app: &AppEntry) -> InstallResult {
        log::info!("Uninstalling .deb package: {}", app.name);

        let result = SystemExec::run_with_pkexec(&format!("apt-get remove -y '{}' 2>&1", app.name));

        match result {
            Ok(exec) => {
                InstallResult {
                    success: exec.success,
                    message: if exec.success {
                        format!("Successfully uninstalled {}", app.name)
                    } else {
                        format!("Uninstall failed: {}", exec.stderr)
                    },
                    app_name: app.name.clone(),
                    install_path: String::new(),
                    icon_path: None,
                    version: None,
                    size_bytes: 0,
                }
            }
            Err(e) => InstallResult {
                success: false,
                message: format!("Failed to execute apt-get: {}", e),
                app_name: app.name.clone(),
                install_path: String::new(),
                icon_path: None,
                version: None,
                size_bytes: 0,
            },
        }
    }

    fn get_deb_info(path: &Path) -> (String, String) {
        let name = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let version = "1.0".to_string();

        // Try to extract actual package info
        if let Ok(result) = SystemExec::run("dpkg-deb", ["--show", "--showformat=${Package}::${Version}", path.to_str().unwrap_or("")]) {
            if result.success {
                let parts: Vec<&str> = result.stdout.trim().split("::").collect();
                if parts.len() == 2 {
                    return (parts[0].to_string(), parts[1].to_string());
                }
            }
        }

        (name, version)
    }

    fn find_icon(package_name: &str) -> Option<String> {
        let icon_paths = [
            format!("/usr/share/icons/hicolor/128x128/apps/{}.png", package_name),
            format!("/usr/share/icons/hicolor/64x64/apps/{}.png", package_name),
            format!("/usr/share/icons/hicolor/48x48/apps/{}.png", package_name),
            format!("/usr/share/pixmaps/{}.png", package_name),
            format!("/usr/share/pixmaps/{}.xpm", package_name),
        ];
        for p in &icon_paths {
            if SystemExec::file_exists(p) {
                return Some(p.clone());
            }
        }
        None
    }
}
