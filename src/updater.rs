use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ReleaseInfo {
    pub tag_name: String,
    #[allow(dead_code)]
    pub name: String,
    pub body: Option<String>,
    pub download_url: String,
}

fn current_exe_path() -> PathBuf {
    std::env::current_exe().unwrap_or_else(|_| PathBuf::from("/usr/local/bin/app-pro"))
}

fn parse_version(tag: &str) -> Vec<u32> {
    let cleaned = tag.trim_start_matches('v');
    cleaned.split('.')
        .filter_map(|s| {
            let num: String = s.chars().take_while(|c| c.is_ascii_digit()).collect();
            num.parse::<u32>().ok()
        })
        .collect()
}

fn is_newer(latest: &str, current: &str) -> bool {
    let lv = parse_version(latest);
    let cv = parse_version(current);
    for (l, c) in lv.iter().zip(cv.iter()) {
        if l != c {
            return l > c;
        }
    }
    if lv.len() > cv.len() {
        return true;
    }
    if lv.len() < cv.len() {
        return false;
    }
    let latest_stable = !latest.contains('-');
    let current_stable = !current.contains('-');
    latest_stable && !current_stable
}

fn is_valid_elf(path: &std::path::Path) -> bool {
    use std::io::Read;
    let mut f = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };
    let mut magic = [0u8; 4];
    f.read_exact(&mut magic).is_ok() && magic == [0x7f, 0x45, 0x4c, 0x46]
}

fn get_github_token() -> Option<String> {
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        if !token.trim().is_empty() {
            return Some(token.trim().to_string());
        }
    }

    if let Some(config_dir) = dirs::config_dir() {
        let token_file = config_dir.join("app-pro").join("github_token");
        if let Ok(token) = std::fs::read_to_string(token_file) {
            if !token.trim().is_empty() {
                return Some(token.trim().to_string());
            }
        }
    }

    if let Ok(content) = std::fs::read_to_string(".env") {
        for line in content.lines() {
            if line.starts_with("GITHUB_TOKEN=") {
                let token = line.trim_start_matches("GITHUB_TOKEN=").trim();
                let token = token.trim_matches('"').trim_matches('\'').trim();
                if !token.is_empty() {
                    return Some(token.to_string());
                }
            }
        }
    }

    None
}

fn get_latest_tag_via_redirect() -> Option<String> {
    let resp = std::process::Command::new("curl")
        .args(["-sI", "https://github.com/the-shoaib2/app-pro/releases/latest"])
        .output()
        .ok()?;

    if !resp.status.success() {
        return None;
    }

    let headers = String::from_utf8_lossy(&resp.stdout);
    for line in headers.lines() {
        if line.to_lowercase().starts_with("location:") {
            let parts: Vec<&str> = line.split("/tag/").collect();
            if parts.len() > 1 {
                let tag = parts[1].trim().to_string();
                if !tag.is_empty() {
                    return Some(tag);
                }
            }
        }
    }
    None
}

pub fn check_for_updates(current_version: &str) -> Result<Option<ReleaseInfo>, String> {
    let url = "https://api.github.com/repos/the-shoaib2/app-pro/releases/latest";
    let mut args = vec![
        "-sSL".to_string(),
        "-H".to_string(), "Accept: application/json".to_string(),
        "-H".to_string(), "User-Agent: App-Pro-Client".to_string(),
    ];

    if let Some(token) = get_github_token() {
        args.push("-H".to_string());
        args.push(format!("Authorization: Bearer {}", token));
    }

    args.push(url.to_string());

    let resp = std::process::Command::new("curl")
        .args(&args)
        .output()
        .map_err(|e| format!("Failed to check updates: {}", e))?;

    if !resp.status.success() {
        let stderr = String::from_utf8_lossy(&resp.stderr);
        return Err(format!("curl failed: {}", stderr));
    }

    let body: serde_json::Value =
        serde_json::from_slice(&resp.stdout).map_err(|e| format!("Parse error: {}", e))?;

    if let Some(msg) = body["message"].as_str() {
        if msg.contains("rate limit exceeded") || msg.contains("Rate limit exceeded") {
            if let Some(tag_name) = get_latest_tag_via_redirect() {
                if is_newer(&tag_name, current_version) {
                    let s = match std::env::consts::ARCH {
                        "x86_64" => "linux-x86_64",
                        "aarch64" => "linux-arm64",
                        _ => "linux-x86_64",
                    };
                    let download_url = format!(
                        "https://github.com/the-shoaib2/app-pro/releases/download/{}/app-pro-{}",
                        tag_name, s
                    );
                    return Ok(Some(ReleaseInfo {
                        tag_name: tag_name.clone(),
                        name: tag_name.clone(),
                        body: Some("Release notes are unavailable due to GitHub API rate limit, but you can still update safely.".to_string()),
                        download_url,
                    }));
                } else {
                    return Ok(None);
                }
            }
            return Err("GitHub API rate limit exceeded. Please try again later.".to_string());
        }
        if msg.contains("Not Found") {
            return Ok(None);
        }
        return Err(format!("GitHub API error: {}", msg));
    }

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
            a["name"].as_str().map(|n| {
                n.contains("app-pro") && (n.contains("linux") || n.contains("x86_64") || n.ends_with(".tar.gz"))
            }).unwrap_or(false)
        })
        .and_then(|a| a["browser_download_url"].as_str().map(|s| s.to_string()))
        .ok_or("No compatible asset found (expected asset with 'app-pro' in name)")?;

    Ok(Some(ReleaseInfo {
        tag_name,
        name,
        body: body_text,
        download_url,
    }))
}

pub fn should_auto_check() -> bool {
    let cache_dir = dirs::cache_dir().unwrap_or_else(std::env::temp_dir);
    let check_file = cache_dir.join("app-pro-last-update-check");
    if let Ok(content) = std::fs::read_to_string(&check_file) {
        if let Ok(last_time) = content.trim().parse::<u64>() {
            if let Ok(now) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
                if now.as_secs() > last_time && now.as_secs() - last_time < 12 * 3600 {
                    return false;
                }
            }
        }
    }
    true
}

pub fn update_last_check_timestamp() {
    let cache_dir = dirs::cache_dir().unwrap_or_else(std::env::temp_dir);
    let check_file = cache_dir.join("app-pro-last-update-check");
    if let Ok(now) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        std::fs::write(check_file, now.as_secs().to_string()).ok();
    }
}

pub fn perform_update(release: &ReleaseInfo) -> Result<(), String> {
    let target = current_exe_path();
    let tmp_path = std::env::temp_dir().join("app-pro-update");

    eprint!("Downloading {} ... ", release.tag_name);
    let status = std::process::Command::new("curl")
        .args([
            "-fsSL",
            "-o",
            &tmp_path.to_string_lossy(),
            "-L",
            &release.download_url,
        ])
        .status()
        .map_err(|e| format!("Failed to run curl: {}", e))?;

    if !status.success() {
        return Err("Download failed - check your internet connection".to_string());
    }
    eprintln!("done");

    if !is_valid_elf(&tmp_path) {
        std::fs::remove_file(&tmp_path).ok();
        return Err("Downloaded file is not a valid ELF binary - aborting".to_string());
    }

    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&tmp_path, std::fs::Permissions::from_mode(0o755))
        .map_err(|e| format!("Failed to set executable permissions: {}", e))?;

    let target_str = target.display();
    let copy_ok = std::process::Command::new("cp")
        .args(["-f", &tmp_path.to_string_lossy(), &target.to_string_lossy()])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if copy_ok {
        std::fs::remove_file(&tmp_path).ok();
    } else {
        eprint!("Escalating privileges... ");
        let pkexec_ok = std::process::Command::new("pkexec")
            .args(["cp", &tmp_path.to_string_lossy(), &target.to_string_lossy()])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        std::fs::remove_file(&tmp_path).ok();
        if pkexec_ok {
            eprintln!("done");
        } else {
            return Err(format!(
                "Permission denied: run 'sudo app-pro update' to update {}",
                target_str
            ));
        }
    }

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
        assert_eq!(parse_version("v1.0.0-rc1"), vec![1, 0, 0]);
        assert_eq!(parse_version("v1.0.0+build2"), vec![1, 0, 0]);
    }

    #[test]
    fn test_is_newer() {
        assert!(is_newer("v1.1.0", "v1.0.0"));
        assert!(is_newer("v2.0.0", "v1.9.9"));
        assert!(!is_newer("v1.0.0", "v1.0.0"));
        assert!(!is_newer("v0.9.0", "v1.0.0"));
        assert!(!is_newer("v0.9.9", "v1.0.0"));
        assert!(is_newer("v1.0.1", "v1.0.0"));
    }

    #[test]
    fn test_is_newer_pre_release() {
        assert!(is_newer("v1.1.0-rc1", "v1.0.0"));
        assert!(!is_newer("v1.0.0-rc1", "v1.0.0"));
    }

    #[test]
    fn test_current_exe_path() {
        let p = current_exe_path();
        assert!(p.as_os_str().len() > 0);
    }

    #[test]
    fn test_is_valid_elf_on_self() {
        let p = current_exe_path();
        assert!(is_valid_elf(&p));
    }

    #[test]
    fn test_is_valid_elf_on_tmpfile() {
        let p = std::env::temp_dir().join("test-not-elf");
        std::fs::write(&p, b"not an elf").ok();
        assert!(!is_valid_elf(&p));
        std::fs::remove_file(&p).ok();
    }
}
