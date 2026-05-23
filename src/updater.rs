use std::path::PathBuf;

pub struct ReleaseInfo {
    pub tag_name: String,
    pub name: String,
    pub body: Option<String>,
    pub download_url: String,
}

fn current_exe_path() -> PathBuf {
    std::env::current_exe().unwrap_or_else(|_| PathBuf::from("/usr/local/bin/app-pro"))
}

fn parse_version(tag: &str) -> Vec<u32> {
    tag.trim_start_matches('v')
        .split('.')
        .filter_map(|s| s.parse::<u32>().ok())
        .collect()
}

fn is_newer(latest: &str, current: &str) -> bool {
    parse_version(latest) > parse_version(current)
}

pub fn check_for_updates(current_version: &str) -> Result<Option<ReleaseInfo>, String> {
    let url = "https://api.github.com/repos/the-shoaib2/app-pro/releases/latest";
    let resp = std::process::Command::new("curl")
        .args(["-fsSL", "-H", "Accept: application/json", url])
        .output()
        .map_err(|e| format!("Failed to check updates: {}", e))?;

    if !resp.status.success() {
        let stderr = String::from_utf8_lossy(&resp.stderr);
        return Err(format!("GitHub API error: {}", stderr));
    }

    let body: serde_json::Value =
        serde_json::from_slice(&resp.stdout).map_err(|e| format!("Parse error: {}", e))?;

    let tag_name = body["tag_name"]
        .as_str()
        .ok_or("Missing tag_name")?
        .to_string();

    if !is_newer(&tag_name, current_version) {
        return Ok(None);
    }

    let name = body["name"].as_str().unwrap_or(&tag_name).to_string();
    let body_text = body["body"].as_str().map(|s| s.to_string());

    let assets = body["assets"].as_array().ok_or("No assets found")?;
    let download_url = assets
        .iter()
        .find(|a| {
            a["name"]
                .as_str()
                .map(|n| n.contains("linux") || n.contains("x86_64"))
                .unwrap_or(false)
        })
        .and_then(|a| a["browser_download_url"].as_str().map(|s| s.to_string()))
        .ok_or("No compatible asset found")?;

    Ok(Some(ReleaseInfo {
        tag_name,
        name,
        body: body_text,
        download_url,
    }))
}

pub fn perform_update(release: &ReleaseInfo) -> Result<(), String> {
    let target = current_exe_path();
    let tmp_path = std::env::temp_dir().join("app-pro-update");

    let status = std::process::Command::new("curl")
        .args([
            "-fsSL",
            "-o",
            &tmp_path.to_string_lossy(),
            "-L",
            &release.download_url,
        ])
        .status()
        .map_err(|e| format!("Download failed: {}", e))?;

    if !status.success() {
        return Err("Download failed".to_string());
    }

    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&tmp_path, std::fs::Permissions::from_mode(0o755))
        .map_err(|e| format!("Failed to set permissions: {}", e))?;

    std::fs::rename(&tmp_path, &target)
        .or_else(|_rename_err: std::io::Error| {
            std::fs::copy(&tmp_path, &target).map(|_| {
                std::fs::remove_file(&tmp_path).ok();
            })
        })
        .map_err(|e: std::io::Error| format!("Failed to install update: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version() {
        assert_eq!(parse_version("v1.0.0"), vec![1, 0, 0]);
        assert_eq!(parse_version("v2.3.4"), vec![2, 3, 4]);
        assert_eq!(parse_version("1.0.0"), vec![1, 0, 0]);
    }

    #[test]
    fn test_is_newer() {
        assert!(is_newer("v1.1.0", "v1.0.0"));
        assert!(is_newer("v2.0.0", "v1.9.9"));
        assert!(!is_newer("v1.0.0", "v1.0.0"));
        assert!(!is_newer("v0.9.0", "v1.0.0"));
    }

    #[test]
    fn test_current_exe_path() {
        let p = current_exe_path();
        assert!(p.as_os_str().len() > 0);
    }
}
