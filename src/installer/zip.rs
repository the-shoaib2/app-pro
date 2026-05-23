use crate::core::SystemExec;
use crate::db::AppEntry;
use crate::installer::InstallResult;
use std::path::{Path, PathBuf};
use std::fs;

pub struct ZipInstaller;

impl ZipInstaller {
    pub fn install<P: AsRef<Path>>(path: P, log_tx: Option<std::sync::mpsc::Sender<String>>) -> InstallResult {
        let path = path.as_ref();
        let filename = path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let app_name = filename
            .strip_suffix(".zip")
            .unwrap_or(&filename)
            .to_string();

        let apps_dir = Self::get_apps_dir();
        let target_dir = apps_dir.join(&app_name);

        log::info!("Installing zip bundle: {} -> {}", path.display(), target_dir.display());

        if let Some(ref tx) = log_tx {
            tx.send(format!("Starting installation of zip bundle: {}", app_name)).ok();
            tx.send(format!("Creating extraction directory...")).ok();
        }

        // Create temporary extraction directory
        let temp_dir = PathBuf::from("/tmp").join(format!("apppro-extract-{}", &app_name));
        fs::create_dir_all(&temp_dir).ok();

        if let Some(ref tx) = log_tx {
            tx.send(format!("Extracting files using unzip...")).ok();
        }

        let accumulated_logs = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
        let accum_clone = accumulated_logs.clone();
        let tx_clone = log_tx.clone();

        // Extract zip using unzip with streaming
        let result = SystemExec::run_streaming("unzip", [
            "-o",
            path.to_str().unwrap_or(""),
            "-d",
            temp_dir.to_str().unwrap_or("")
        ], move |line| {
            if let Ok(mut accum) = accum_clone.lock() {
                accum.push_str(&line);
                accum.push('\n');
            }
            if let Some(ref tx) = tx_clone {
                tx.send(line).ok();
            }
        });

        let logs = accumulated_logs.lock().unwrap().clone();

        match result {
            Ok(mut exec) => {
                exec.stdout = logs.clone();
                exec.stderr = logs;

                if exec.success {
                    if let Some(ref tx) = log_tx {
                        tx.send("Extraction completed successfully.".to_string()).ok();
                        tx.send("Finding main executable...".to_string()).ok();
                    }

                    // Find the main executable
                    let main_exe = Self::find_main_executable(&temp_dir, &app_name);

                    if let Some(ref tx) = log_tx {
                        tx.send("Copying extracted files to application folder...".to_string()).ok();
                    }

                    // Create app directory
                    fs::create_dir_all(&target_dir).ok();

                    // Copy extracted files
                    if let Err(e) = SystemExec::copy_recursively(&temp_dir, &target_dir) {
                        fs::remove_dir_all(&temp_dir).ok();
                        let err_msg = format!("Failed to copy files: {}", e);
                        if let Some(ref tx) = log_tx {
                            tx.send(err_msg.clone()).ok();
                        }
                        return InstallResult {
                            success: false,
                            message: err_msg,
                            app_name,
                            install_path: String::new(),
                            icon_path: None,
                            version: None,
                            size_bytes: 0,
                        };
                    }

                    // Make main executable executable
                    if let Some(ref exe) = main_exe {
                        let exe_path = target_dir.join(exe);
                        if exe_path.exists() {
                            if let Some(ref tx) = log_tx {
                                tx.send(format!("Setting permissions (0755) on executable: {}", exe.display())).ok();
                            }
                            use std::os::unix::fs::PermissionsExt;
                            fs::set_permissions(&exe_path, fs::Permissions::from_mode(0o755)).ok();
                        }
                    }

                    // Find icon
                    let icon_path = Self::find_icon_in_dir(&target_dir, &app_name);

                    // Create .desktop entry
                    if let Some(ref tx) = log_tx {
                        tx.send("Creating desktop application launcher entry...".to_string()).ok();
                    }
                    let exec_path = match &main_exe {
                        Some(exe) => target_dir.join(exe),
                        None => target_dir.join(&app_name),
                    };
                    Self::create_desktop_entry(&app_name, &exec_path, &icon_path).ok();

                    // Calculate total size
                    let size = SystemExec::get_size(&target_dir) as i64;

                    // Cleanup temp
                    fs::remove_dir_all(&temp_dir).ok();

                    let message = format!("Successfully installed {}", app_name);
                    if let Some(ref tx) = log_tx {
                        tx.send(message.clone()).ok();
                        tx.send(format!("Installation of {} finished!", app_name)).ok();
                    }

                    InstallResult {
                        success: true,
                        message,
                        app_name,
                        install_path: target_dir.to_string_lossy().to_string(),
                        icon_path,
                        version: None,
                        size_bytes: size,
                    }
                } else {
                    fs::remove_dir_all(&temp_dir).ok();
                    let err_msg = format!("Extraction failed: {}", exec.stderr);
                    if let Some(ref tx) = log_tx {
                        tx.send(err_msg.clone()).ok();
                    }
                    InstallResult {
                        success: false,
                        message: err_msg,
                        app_name,
                        install_path: String::new(),
                        icon_path: None,
                        version: None,
                        size_bytes: 0,
                    }
                }
            }
            Err(e) => {
                fs::remove_dir_all(&temp_dir).ok();
                let err_msg = format!("Failed to run unzip: {}", e);
                if let Some(ref tx) = log_tx {
                    tx.send(err_msg.clone()).ok();
                }
                InstallResult {
                    success: false,
                    message: err_msg,
                    app_name,
                    install_path: String::new(),
                    icon_path: None,
                    version: None,
                    size_bytes: 0,
                }
            }
        }
    }

    pub fn uninstall(app: &AppEntry) -> InstallResult {
        log::info!("Uninstalling zip app: {}", app.name);

        let apps_dir = Self::get_apps_dir();
        let target_dir = apps_dir.join(&app.name);

        if target_dir.exists() {
            if let Err(e) = fs::remove_dir_all(&target_dir) {
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

    fn find_main_executable(dir: &Path, app_name: &str) -> Option<PathBuf> {
        // Known binary locations
        let candidates = [
            format!("{}/{}", dir.display(), app_name),
            format!("{}/bin/{}", dir.display(), app_name),
            format!("{}/app", dir.display()),
            format!("{}/start.sh", dir.display()),
            format!("{}/launcher", dir.display()),
            format!("{}/run", dir.display()),
        ];

        for cand in &candidates {
            let p = Path::new(cand);
            if p.exists() && !p.is_dir() {
                return Some(p.to_path_buf());
            }
        }

        // Fallback: look for executable files
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    use std::os::unix::fs::PermissionsExt;
                    if let Ok(meta) = path.metadata() {
                        if meta.permissions().mode() & 0o111 != 0 {
                            // Find relative path
                            if let Ok(rel) = path.strip_prefix(dir) {
                                return Some(rel.to_path_buf());
                            }
                            return Some(path);
                        }
                    }
                }
            }
        }

        None
    }

    fn find_icon_in_dir(dir: &Path, app_name: &str) -> Option<String> {
        for pattern in &[
            format!("{}.png", app_name),
            format!("{}.svg", app_name),
            "icon.png".to_string(),
            "logo.png".to_string(),
            "icon.svg".to_string(),
            "logo.svg".to_string(),
        ] {
            let icon_path = dir.join(pattern);
            if icon_path.exists() {
                return Some(icon_path.to_string_lossy().to_string());
            }
        }

        let sub_dirs = ["usr/share/icons", "share/icons", "icons", "resources"];
        for sub in &sub_dirs {
            let icon_dir = dir.join(sub);
            if !icon_dir.exists() { continue; }
            if let Ok(entries) = fs::read_dir(&icon_dir) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if !p.is_dir() { continue; }
                    if let Ok(subs) = fs::read_dir(&p) {
                        for sub in subs.flatten() {
                            let sp = sub.path();
                            if !sp.is_dir() { continue; }
                            if let Ok(icons) = fs::read_dir(&sp) {
                                for icon in icons.flatten() {
                                    let ip = icon.path();
                                    let name = ip.file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or("")
                                        .to_lowercase();
                                    if name.contains(&app_name.to_lowercase())
                                        && (name.ends_with(".png") || name.ends_with(".svg"))
                                    {
                                        return Some(ip.to_string_lossy().to_string());
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
            None => "Icon=application-x-archive".to_string(),
        };

        let content = format!(
            "[Desktop Entry]\n\
             Type=Application\n\
             Name={name}\n\
             Exec={exec}\n\
             Path={path}\n\
             {icon}\n\
             Terminal=false\n\
             Categories=Utility;\n\
             Comment={name} - Installed via App Pro\n\
             X-AppPro-Managed=true\n",
            name = app_name,
            exec = exec_path.display(),
            path = exec_path.parent().map(|p| p.display()).unwrap_or_else(|| std::path::Path::new(".").display()),
            icon = icon_line,
        );

        fs::write(&desktop_path, content).map_err(|e| e.to_string())?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_installed_apps() -> Vec<(String, String, u64)> {
        let apps_dir = Self::get_apps_dir();
        if !apps_dir.exists() {
            return Vec::new();
        }

        let mut apps = Vec::new();
        if let Ok(entries) = fs::read_dir(&apps_dir) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    let size = SystemExec::get_size(entry.path());
                    apps.push((
                        entry.file_name().to_string_lossy().to_string(),
                        entry.path().to_string_lossy().to_string(),
                        size,
                    ));
                }
            }
        }
        apps
    }
}
