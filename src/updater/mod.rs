use std::process::Command;
use serde::{Deserialize, Serialize};
use crate::core::SystemExec;

const REPO: &str = "the-shoaib2/app-pro";

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ReleaseInfo {
    pub tag_name: String,
    pub html_url: String,
    pub body: Option<String>,
    pub assets: Vec<ReleaseAsset>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

/// Checks GitHub Releases API for the latest version.
/// Returns `Ok(Some(ReleaseInfo))` if a new release is available.
pub fn check_for_updates(current_version: &str) -> Result<Option<ReleaseInfo>, String> {
    log::info!("Checking for updates. Current version: {}", current_version);
    
    let url = format!("https://api.github.com/repos/{}/releases/latest", REPO);
    
    let output = Command::new("curl")
        .args(["-sSL", "-H", "User-Agent: app-pro", &url])
        .output()
        .map_err(|e| format!("Failed to execute curl: {}", e))?;
        
    if !output.status.success() {
        return Err(format!(
            "Failed to fetch release info (curl exit code {:?})", 
            output.status.code()
        ));
    }
    
    let response_body = String::from_utf8_lossy(&output.stdout);
    
    let release: ReleaseInfo = serde_json::from_str(&response_body)
        .map_err(|e| format!("Failed to parse release JSON: {}. Response: {}", e, response_body))?;
        
    if is_newer(current_version, &release.tag_name) {
        Ok(Some(release))
    } else {
        Ok(None)
    }
}

/// Helper function to compare two semantic version strings.
pub fn is_newer(current: &str, latest: &str) -> bool {
    let current_clean = current.trim_start_matches('v');
    let latest_clean = latest.trim_start_matches('v');

    let current_parts: Vec<u32> = current_clean.split('.')
        .map(|s| s.parse().unwrap_or(0))
        .collect();
    let latest_parts: Vec<u32> = latest_clean.split('.')
        .map(|s| s.parse().unwrap_or(0))
        .collect();

    for i in 0..std::cmp::max(current_parts.len(), latest_parts.len()) {
        let cur = current_parts.get(i).cloned().unwrap_or(0);
        let lat = latest_parts.get(i).cloned().unwrap_or(0);
        if lat > cur {
            return true;
        } else if cur > lat {
            return false;
        }
    }
    false
}

/// Downloads and applies the update.
pub fn perform_update(release: &ReleaseInfo) -> Result<(), String> {
    let arch = std::env::consts::ARCH;
    let suffix = match arch {
        "x86_64" => "linux-x86_64",
        "aarch64" => "linux-arm64",
        _ => return Err(format!("Unsupported architecture: {}", arch)),
    };
    
    let asset_name = format!("app-pro-{}", suffix);
    let asset = release.assets.iter()
        .find(|a| a.name == asset_name)
        .ok_or_else(|| format!("Could not find binary asset named '{}' in the release", asset_name))?;
        
    log::info!("Downloading asset: {} from {}", asset.name, asset.browser_download_url);
    
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join("app-pro-update-download");
    
    // Download release binary
    let download_status = Command::new("curl")
        .args(["-sSL", "-o", temp_file.to_str().unwrap(), &asset.browser_download_url])
        .status()
        .map_err(|e| format!("Failed to launch curl for download: {}", e))?;
        
    if !download_status.success() {
        return Err("Failed to download the new binary version via curl".to_string());
    }
    
    // Make it executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&temp_file)
            .map_err(|e| format!("Failed to read metadata of temp file: {}", e))?
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&temp_file, perms)
            .map_err(|e| format!("Failed to set executable permissions on temp file: {}", e))?;
    }
    
    // Replace current executable
    let current_exe = std::env::current_exe()
        .map_err(|e| format!("Failed to determine current running binary location: {}", e))?;
        
    // Check write permissions for current binary path
    let needs_pkexec = if let Some(parent) = current_exe.parent() {
        std::fs::metadata(parent).map(|m| m.permissions().readonly()).unwrap_or(true)
    } else {
        true
    };
    
    if needs_pkexec {
        log::info!("Elevated privileges needed. Replacing binary using pkexec...");
        let script = format!(
            "cp '{}' '{}' && rm -f '{}'",
            temp_file.display(),
            current_exe.display(),
            temp_file.display()
        );
        let res = SystemExec::run_with_pkexec(&script)
            .map_err(|e| format!("Failed to run installer with elevated privileges: {}", e))?;
            
        if !res.success {
            return Err(format!("Authorization or installation failed: {}", res.stderr));
        }
    } else {
        log::info!("Replacing binary directly...");
        // Rename/replace safely by moving existing one first to handle running executable lock on unix
        let backup_exe = current_exe.with_extension("bak");
        std::fs::rename(&current_exe, &backup_exe)
            .map_err(|e| format!("Failed to backup existing binary: {}", e))?;
            
        if let Err(e) = std::fs::copy(&temp_file, &current_exe) {
            // Rollback if copying new binary failed
            std::fs::rename(&backup_exe, &current_exe).ok();
            return Err(format!("Failed to copy new binary: {}", e));
        }
        
        // Clean up
        std::fs::remove_file(backup_exe).ok();
        std::fs::remove_file(temp_file).ok();
    }
    
    log::info!("Update to {} completed successfully!", release.tag_name);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_newer() {
        assert!(is_newer("1.0.0", "1.1.0"));
        assert!(is_newer("1.0.0", "2.0.0"));
        assert!(is_newer("1.1.2", "1.1.3"));
        assert!(is_newer("v1.0.0", "v1.0.1"));
        assert!(is_newer("1.0", "1.1"));
        
        assert!(!is_newer("1.1.0", "1.1.0"));
        assert!(!is_newer("1.1.0", "1.0.0"));
        assert!(!is_newer("2.0.1", "2.0.0"));
        assert!(!is_newer("v1.2.0", "v1.1.0"));
    }
}
