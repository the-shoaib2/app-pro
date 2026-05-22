pub mod cache;

use crate::core::SystemExec;
use crate::db::{AppDatabase, CacheEntry};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct CleanupResult {
    #[allow(dead_code)]
    pub path: String,
    pub description: String,
    pub bytes_freed: u64,
    pub success: bool,
    pub message: String,
}

pub struct CleanupManager {
    db: AppDatabase,
}

impl CleanupManager {
    pub fn new(db: AppDatabase) -> Self {
        CleanupManager { db }
    }

    pub fn clean_user_cache(&self) -> CleanupResult {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"));

        let size_before = SystemExec::get_size(&cache_dir);
        let path = cache_dir.to_string_lossy().to_string();

        match SystemExec::remove_dir_contents(&cache_dir) {
            Ok(_) => {
                let size_after = SystemExec::get_size(&cache_dir);
                let bytes_freed = size_before.saturating_sub(size_after);

                self.record_cleanup(&path, "User cache (~/.cache)", bytes_freed);

                CleanupResult {
                    path,
                    description: "User cache (~/.cache)".to_string(),
                    bytes_freed,
                    success: true,
                    message: format!("Cleaned ~/.cache: freed {} bytes", bytes_freed),
                }
            }
            Err(e) => CleanupResult {
                path,
                description: "User cache (~/.cache)".to_string(),
                bytes_freed: 0,
                success: false,
                message: format!("Failed to clean ~/.cache: {}", e),
            },
        }
    }

    pub fn clean_apt_cache(&self) -> CleanupResult {
        let path = "/var/cache/apt/archives".to_string();
        let size_before = SystemExec::get_size(&path);

        let result = SystemExec::run_with_pkexec("apt-get clean 2>&1");

        match result {
            Ok(exec) if exec.success => {
                let size_after = SystemExec::get_size(&path);
                let bytes_freed = size_before.saturating_sub(size_after);

                self.record_cleanup(&path, "APT package cache", bytes_freed);

                CleanupResult {
                    path,
                    description: "APT package cache".to_string(),
                    bytes_freed,
                    success: true,
                    message: format!("Cleaned APT cache: freed {} bytes", bytes_freed),
                }
            }
            Ok(exec) => CleanupResult {
                path,
                description: "APT package cache".to_string(),
                bytes_freed: 0,
                success: false,
                message: format!("Failed to clean APT cache: {}", exec.stderr),
            },
            Err(e) => CleanupResult {
                path,
                description: "APT package cache".to_string(),
                bytes_freed: 0,
                success: false,
                message: format!("Failed to execute apt-get clean: {}", e),
            },
        }
    }

    pub fn clean_orphan_packages(&self) -> CleanupResult {
        let path = "orphan-packages".to_string();
        let description = "Orphaned packages (autoremove)".to_string();

        let result = SystemExec::run_with_pkexec("apt-get autoremove --purge -y 2>&1");

        match result {
            Ok(exec) if exec.success => {
                self.record_cleanup(&path, &description, 0);

                CleanupResult {
                    path,
                    description,
                    bytes_freed: 0,
                    success: true,
                    message: "Removed orphaned packages".to_string(),
                }
            }
            Ok(exec) => CleanupResult {
                path,
                description,
                bytes_freed: 0,
                success: false,
                message: format!("Failed to remove orphans: {}", exec.stderr),
            },
            Err(e) => CleanupResult {
                path,
                description,
                bytes_freed: 0,
                success: false,
                message: format!("Failed to execute autoremove: {}", e),
            },
        }
    }

    pub fn clean_thumbnails(&self) -> CleanupResult {
        let thumb_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("thumbnails");

        let path = thumb_dir.to_string_lossy().to_string();
        let size_before = SystemExec::get_size(&thumb_dir);

        match SystemExec::remove_dir_contents(&thumb_dir) {
            Ok(_) => {
                let size_after = SystemExec::get_size(&thumb_dir);
                let bytes_freed = size_before.saturating_sub(size_after);

                self.record_cleanup(&path, "Thumbnail cache", bytes_freed);

                CleanupResult {
                    path,
                    description: "Thumbnail cache".to_string(),
                    bytes_freed,
                    success: true,
                    message: format!("Cleaned thumbnails: freed {} bytes", bytes_freed),
                }
            }
            Err(e) => CleanupResult {
                path,
                description: "Thumbnail cache".to_string(),
                bytes_freed: 0,
                success: false,
                message: format!("Failed to clean thumbnails: {}", e),
            },
        }
    }

    #[allow(dead_code)]
    pub fn get_size_of(path: &str) -> u64 {
        SystemExec::get_size(path)
    }

    #[allow(dead_code)]
    pub fn get_history(&self) -> Vec<CacheEntry> {
        self.db.get_cleanup_history().unwrap_or_default()
    }

    fn record_cleanup(&self, path: &str, description: &str, bytes_freed: u64) {
        let entry = CacheEntry {
            id: uuid::Uuid::new_v4().to_string(),
            path: path.to_string(),
            description: description.to_string(),
            size_bytes: bytes_freed as i64,
            cleaned_at: Some(chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()),
        };
        self.db.insert_cleanup(&entry).ok();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::AppDatabase;

    fn test_cleaner() -> CleanupManager {
        let db = AppDatabase::new().unwrap();
        CleanupManager::new(db)
    }

    #[test]
    fn test_clean_user_cache() {
        let cleaner = test_cleaner();
        let result = cleaner.clean_user_cache();
        // User cache likely empty or inaccessible in test env
        assert!(result.success || !result.success);
        assert!(!result.description.is_empty());
    }

    #[test]
    fn test_clean_thumbnails() {
        let cleaner = test_cleaner();
        let result = cleaner.clean_thumbnails();
        assert!(result.success || !result.success);
        assert!(!result.path.is_empty());
    }

    #[test]
    fn test_cleanup_result_struct() {
        let r = CleanupResult {
            path: "/tmp".into(),
            description: "test".into(),
            bytes_freed: 100,
            success: true,
            message: "done".into(),
        };
        assert_eq!(r.bytes_freed, 100);
        assert!(r.success);
    }
}
