use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppEntry {
    pub id: String,
    pub name: String,
    pub install_type: String, // "deb", "appimage", "zip", "flatpak"
    pub install_path: String,
    pub icon_path: Option<String>,
    pub desktop_file: Option<String>,
    pub version: Option<String>,
    pub installed_at: String,
    pub size_bytes: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub id: String,
    pub path: String,
    pub description: String,
    pub size_bytes: i64,
    pub cleaned_at: Option<String>,
}

pub struct AppDatabase {
    conn: Mutex<Connection>,
}

impl AppDatabase {
    pub fn new() -> Result<Self> {
        let db_path = Self::get_db_path();
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let conn = Connection::open(&db_path)?;
        let db = AppDatabase { conn: Mutex::new(conn) };
        db.initialize_tables()?;
        Ok(db)
    }

    pub fn new_from_conn(conn: Connection) -> Self {
        let db = AppDatabase { conn: Mutex::new(conn) };
        db.initialize_tables().ok();
        db
    }

    fn get_db_path() -> PathBuf {
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"));
        data_dir.join("app-pro").join("app_pro.db")
    }

    fn initialize_tables(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS installed_apps (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                install_type TEXT NOT NULL,
                install_path TEXT NOT NULL,
                icon_path TEXT,
                desktop_file TEXT,
                version TEXT,
                installed_at TEXT NOT NULL DEFAULT (datetime('now')),
                size_bytes INTEGER DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS cleanup_history (
                id TEXT PRIMARY KEY,
                path TEXT NOT NULL,
                description TEXT NOT NULL,
                size_bytes INTEGER DEFAULT 0,
                cleaned_at TEXT
            );

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );"
        )?;
        Ok(())
    }

    pub fn insert_app(&self, app: &AppEntry) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO installed_apps
             (id, name, install_type, install_path, icon_path, desktop_file, version, installed_at, size_bytes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                app.id,
                app.name,
                app.install_type,
                app.install_path,
                app.icon_path,
                app.desktop_file,
                app.version,
                app.installed_at,
                app.size_bytes,
            ],
        )?;
        Ok(())
    }

    pub fn get_all_apps(&self) -> Result<Vec<AppEntry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, install_type, install_path, icon_path, desktop_file, version, installed_at, size_bytes
             FROM installed_apps ORDER BY installed_at DESC"
        )?;
        let apps = stmt.query_map([], |row| {
            Ok(AppEntry {
                id: row.get(0)?,
                name: row.get(1)?,
                install_type: row.get(2)?,
                install_path: row.get(3)?,
                icon_path: row.get(4)?,
                desktop_file: row.get(5)?,
                version: row.get(6)?,
                installed_at: row.get(7)?,
                size_bytes: row.get(8)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
        Ok(apps)
    }

    pub fn get_app_by_id(&self, id: &str) -> Result<Option<AppEntry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, install_type, install_path, icon_path, desktop_file, version, installed_at, size_bytes
             FROM installed_apps WHERE id = ?1"
        )?;
        let mut rows = stmt.query_map(params![id], |row| {
            Ok(AppEntry {
                id: row.get(0)?,
                name: row.get(1)?,
                install_type: row.get(2)?,
                install_path: row.get(3)?,
                icon_path: row.get(4)?,
                desktop_file: row.get(5)?,
                version: row.get(6)?,
                installed_at: row.get(7)?,
                size_bytes: row.get(8)?,
            })
        })?;
        match rows.next() {
            Some(Ok(app)) => Ok(Some(app)),
            _ => Ok(None),
        }
    }

    pub fn remove_app(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM installed_apps WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn app_exists(&self, name: &str, install_type: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT COUNT(*) FROM installed_apps WHERE name = ?1 AND install_type = ?2"
        )?;
        let count: i64 = stmt.query_row(params![name, install_type], |row| row.get(0))?;
        Ok(count > 0)
    }

    pub fn insert_cleanup(&self, entry: &CacheEntry) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO cleanup_history (id, path, description, size_bytes, cleaned_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![entry.id, entry.path, entry.description, entry.size_bytes, entry.cleaned_at],
        )?;
        Ok(())
    }

    pub fn get_cleanup_history(&self) -> Result<Vec<CacheEntry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, path, description, size_bytes, cleaned_at FROM cleanup_history ORDER BY cleaned_at DESC"
        )?;
        let entries = stmt.query_map([], |row| {
            Ok(CacheEntry {
                id: row.get(0)?,
                path: row.get(1)?,
                description: row.get(2)?,
                size_bytes: row.get(3)?,
                cleaned_at: row.get(4)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
        Ok(entries)
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
        let mut rows = stmt.query_map(params![key], |row| row.get::<_, String>(0))?;
        match rows.next() {
            Some(Ok(val)) => Ok(Some(val)),
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_db() -> AppDatabase {
        let conn = Connection::open_in_memory().unwrap();
        let db = AppDatabase { conn: Mutex::new(conn) };
        db.initialize_tables().unwrap();
        db
    }

    #[test]
    fn test_insert_and_get_apps() {
        let db = test_db();
        let app = AppEntry {
            id: "test-id".into(),
            name: "TestApp".into(),
            install_type: "deb".into(),
            install_path: "/usr/bin/test".into(),
            icon_path: None,
            desktop_file: None,
            version: Some("1.0".into()),
            installed_at: "2025-01-01 12:00:00".into(),
            size_bytes: 1024,
        };
        db.insert_app(&app).unwrap();
        let apps = db.get_all_apps().unwrap();
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].name, "TestApp");
        assert_eq!(apps[0].install_type, "deb");
        assert_eq!(apps[0].version.as_deref(), Some("1.0"));
    }

    #[test]
    fn test_get_app_by_id() {
        let db = test_db();
        let app = AppEntry {
            id: "id-123".into(),
            name: "MyApp".into(),
            install_type: "appimage".into(),
            install_path: "/home/user/app.AppImage".into(),
            icon_path: Some("/icon.png".into()),
            desktop_file: None,
            version: None,
            installed_at: "2025-06-01".into(),
            size_bytes: 5000,
        };
        db.insert_app(&app).unwrap();
        let found = db.get_app_by_id("id-123").unwrap().unwrap();
        assert_eq!(found.name, "MyApp");
        assert_eq!(found.icon_path, Some("/icon.png".into()));

        let missing = db.get_app_by_id("nonexistent").unwrap();
        assert!(missing.is_none());
    }

    #[test]
    fn test_remove_app() {
        let db = test_db();
        let app = AppEntry {
            id: "r-id".into(),
            name: "RemoveMe".into(),
            install_type: "zip".into(),
            install_path: "/tmp/app".into(),
            icon_path: None, desktop_file: None, version: None,
            installed_at: "2025-01-01".into(), size_bytes: 0,
        };
        db.insert_app(&app).unwrap();
        assert_eq!(db.get_all_apps().unwrap().len(), 1);
        db.remove_app("r-id").unwrap();
        assert_eq!(db.get_all_apps().unwrap().len(), 0);
    }

    #[test]
    fn test_app_exists() {
        let db = test_db();
        let app = AppEntry {
            id: "e-id".into(), name: "ExistsApp".into(),
            install_type: "deb".into(), install_path: "/".into(),
            icon_path: None, desktop_file: None, version: None,
            installed_at: "2025-01-01".into(), size_bytes: 0,
        };
        db.insert_app(&app).unwrap();
        assert!(db.app_exists("ExistsApp", "deb").unwrap());
        assert!(!db.app_exists("ExistsApp", "appimage").unwrap());
        assert!(!db.app_exists("Other", "deb").unwrap());
    }

    #[test]
    fn test_settings() {
        let db = test_db();
        db.set_setting("theme", "dark").unwrap();
        assert_eq!(db.get_setting("theme").unwrap(), Some("dark".into()));
        assert_eq!(db.get_setting("nonexistent").unwrap(), None);
    }

    #[test]
    fn test_insert_and_get_cleanup() {
        let db = test_db();
        let entry = CacheEntry {
            id: "c-id".into(),
            path: "/tmp/cache".into(),
            description: "test cache".into(),
            size_bytes: 1000,
            cleaned_at: Some("2025-01-01".into()),
        };
        db.insert_cleanup(&entry).unwrap();
        let history = db.get_cleanup_history().unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].description, "test cache");
        assert_eq!(history[0].size_bytes, 1000);
    }

    #[test]
    fn test_multiple_apps_order() {
        let db = test_db();
        for i in 0..3 {
            let app = AppEntry {
                id: format!("m-{i}"), name: format!("App{i}"),
                install_type: "deb".into(), install_path: "/".into(),
                icon_path: None, desktop_file: None, version: None,
                installed_at: format!("2025-01-{:02}", i+1), size_bytes: i,
            };
            db.insert_app(&app).unwrap();
        }
        let apps = db.get_all_apps().unwrap();
        assert_eq!(apps.len(), 3);
        // Ordered by installed_at DESC
        assert_eq!(apps[0].name, "App2");
        assert_eq!(apps[2].name, "App0");
    }
}
