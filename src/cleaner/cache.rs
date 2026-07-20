use crate::core::SystemExec;
use std::path::PathBuf;

pub struct CacheAnalyzer;

impl CacheAnalyzer {
    pub fn get_user_cache_size() -> u64 {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"));
        SystemExec::get_size(&cache_dir)
    }

    pub fn get_apt_cache_size() -> u64 {
        SystemExec::get_size("/var/cache/apt/archives")
    }

    pub fn get_thumbnail_cache_size() -> u64 {
        let thumb_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("thumbnails");
        SystemExec::get_size(&thumb_dir)
    }

    pub fn get_app_pro_cache_size() -> u64 {
        let mut total = 0;
        let cache_dir = dirs::cache_dir().unwrap_or_else(std::env::temp_dir);
        
        let check_file = cache_dir.join("app-pro-last-update-check");
        if check_file.exists() {
            if let Ok(metadata) = std::fs::metadata(&check_file) {
                total += metadata.len();
            }
        }
        
        let tmp_update = std::env::temp_dir().join("app-pro-update");
        if tmp_update.exists() {
            if let Ok(metadata) = std::fs::metadata(&tmp_update) {
                total += metadata.len();
            }
        }
        total
    }

    pub fn get_all_cache_sizes() -> Vec<(String, String, u64)> {
        vec![
            (
                "User Cache".to_string(),
                dirs::cache_dir()
                    .unwrap_or_else(|| PathBuf::from("/tmp"))
                    .to_string_lossy()
                    .to_string(),
                Self::get_user_cache_size(),
            ),
            (
                "APT Cache".to_string(),
                "/var/cache/apt/archives".to_string(),
                Self::get_apt_cache_size(),
            ),
            (
                "Thumbnails".to_string(),
                dirs::cache_dir()
                    .unwrap_or_else(|| PathBuf::from("/tmp"))
                    .join("thumbnails")
                    .to_string_lossy()
                    .to_string(),
                Self::get_thumbnail_cache_size(),
            ),
            (
                "App Pro Cache".to_string(),
                dirs::cache_dir()
                    .unwrap_or_else(|| PathBuf::from("/tmp"))
                    .join("app-pro-last-update-check")
                    .to_string_lossy()
                    .to_string(),
                Self::get_app_pro_cache_size(),
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_all_cache_sizes_returns_four() {
        let sizes = CacheAnalyzer::get_all_cache_sizes();
        assert_eq!(sizes.len(), 4);
        assert_eq!(sizes[0].0, "User Cache");
        assert_eq!(sizes[1].0, "APT Cache");
        assert_eq!(sizes[2].0, "Thumbnails");
        assert_eq!(sizes[3].0, "App Pro Cache");
    }

    #[test]
    fn test_cache_sizes_non_empty_paths() {
        let sizes = CacheAnalyzer::get_all_cache_sizes();
        for (_, path, _) in &sizes {
            assert!(!path.is_empty());
        }
    }
}
