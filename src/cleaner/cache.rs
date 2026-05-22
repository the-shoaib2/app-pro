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
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_all_cache_sizes_returns_three() {
        let sizes = CacheAnalyzer::get_all_cache_sizes();
        assert_eq!(sizes.len(), 3);
        assert_eq!(sizes[0].0, "User Cache");
        assert_eq!(sizes[1].0, "APT Cache");
        assert_eq!(sizes[2].0, "Thumbnails");
    }

    #[test]
    fn test_cache_sizes_non_empty_paths() {
        let sizes = CacheAnalyzer::get_all_cache_sizes();
        for (_, path, _) in &sizes {
            assert!(!path.is_empty());
        }
    }
}
