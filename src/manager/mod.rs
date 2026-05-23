pub mod processes;
pub mod desktop_scanner;

use crate::db::{AppDatabase, AppEntry};
use crate::installer::{InstallResult, InstallType};
use crate::installer::{deb::DebInstaller, appimage::AppImageInstaller, zip::ZipInstaller};
use std::path::Path;

pub struct AppManager {
    db: AppDatabase,
}

impl AppManager {
    pub fn new(db: AppDatabase) -> Self {
        AppManager { db }
    }

    pub fn install_file<P: AsRef<Path>>(&self, path: P, log_tx: Option<std::sync::mpsc::Sender<String>>) -> InstallResult {
        let path = path.as_ref();
        let install_type = InstallType::from_path(path);

        let result = match install_type {
            InstallType::Deb => DebInstaller::install(path, log_tx),
            InstallType::AppImage => AppImageInstaller::install(path, log_tx),
            InstallType::Zip => ZipInstaller::install(path, log_tx),
            InstallType::Unknown => {
                return InstallResult {
                    success: false,
                    message: format!("Unknown file type: {}", path.display()),
                    app_name: String::new(),
                    install_path: String::new(),
                    icon_path: None,
                    version: None,
                    size_bytes: 0,
                };
            }
        };

        if result.success {
            let app_entry = AppEntry {
                id: uuid::Uuid::new_v4().to_string(),
                name: result.app_name.clone(),
                install_type: install_type.as_str().to_string(),
                install_path: result.install_path.clone(),
                icon_path: result.icon_path.clone(),
                desktop_file: None,
                version: result.version.clone(),
                installed_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                size_bytes: result.size_bytes,
            };
            self.db.insert_app(&app_entry).ok();
        }

        result
    }

    pub fn uninstall_app(&self, app: &AppEntry) -> InstallResult {
        let result = match app.install_type.as_str() {
            "deb" => DebInstaller::uninstall(app),
            "appimage" => AppImageInstaller::uninstall(app),
            "zip" => ZipInstaller::uninstall(app),
            _ => InstallResult {
                success: false,
                message: format!("Unknown install type: {}", app.install_type),
                app_name: app.name.clone(),
                install_path: String::new(),
                icon_path: None,
                version: None,
                size_bytes: 0,
            },
        };

        if result.success {
            self.db.remove_app(&app.id).ok();
        }

        result
    }

    pub fn get_installed_apps(&self) -> Vec<AppEntry> {
        self.db.get_all_apps().unwrap_or_default()
    }

    pub fn scan_all_desktop_apps(&self) -> Vec<desktop_scanner::DesktopAppInfo> {
        let pro_apps = self.get_installed_apps();
        desktop_scanner::scan_desktop_apps(&pro_apps)
    }

    #[allow(dead_code)]
    pub fn get_db(&self) -> &AppDatabase {
        &self.db
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::AppDatabase;
    use rusqlite::Connection;

    fn test_manager() -> AppManager {
        let conn = Connection::open_in_memory().unwrap();
        let db = AppDatabase::new_from_conn(conn);
        AppManager::new(db)
    }

    #[test]
    fn test_new_manager_empty() {
        let m = test_manager();
        let apps = m.get_installed_apps();
        assert!(apps.is_empty());
    }

    #[test]
    fn test_install_unknown_file() {
        let m = test_manager();
        let result = m.install_file("/tmp/nonexistent.foo", None);
        assert!(!result.success);
        assert!(result.message.contains("Unknown file type"));
    }

    #[test]
    fn test_install_nonexistent_file() {
        let m = test_manager();
        let result = m.install_file("/tmp/nonexistent.deb", None);
        assert!(!result.message.is_empty());
    }

    #[test]
    fn test_manager_stores_state() {
        let m = test_manager();
        assert!(m.get_installed_apps().is_empty());
    }
}
