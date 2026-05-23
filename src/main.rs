mod core;
mod installer;
mod manager;
mod cleaner;
mod db;
mod ui;
mod updater;

use gtk4::prelude::*;
use gtk4::Application;
use std::env;

fn main() {
    env_logger::init();

    // Initialize database
    let database = match db::AppDatabase::new() {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Failed to initialize database: {}", e);
            std::process::exit(1);
        }
    };

    let app_manager = std::sync::Arc::new(manager::AppManager::new(database));
    let cleaner_manager = {
        let db = match db::AppDatabase::new() {
            Ok(db) => db,
            Err(e) => {
                eprintln!("Failed to initialize cleaner database: {}", e);
                std::process::exit(1);
            }
        };
        std::sync::Arc::new(cleaner::CleanupManager::new(db))
    };

    ensure_desktop_integration();

    let args: Vec<String> = env::args().collect();
    let mut file_to_install = None;

    if args.len() > 1 {
        let first_arg = args[1].as_str();
        match first_arg {
            "update" => {
                let current_ver = core::app_version();
                println!("App Pro v{}", current_ver);
                println!("Checking for updates...");
                match updater::check_for_updates(current_ver) {
                    Ok(Some(release)) => {
                        println!("New version available: {}", release.tag_name);
                        if let Some(body) = &release.body {
                            println!("\n{}", body);
                        }
                        match updater::perform_update(&release) {
                            Ok(_) => {
                                println!("\n✓ Update complete! Restart App Pro to use the new version.");
                                std::process::exit(0);
                            }
                            Err(e) => {
                                eprintln!("\n✗ Update failed: {}", e);
                                std::process::exit(1);
                            }
                        }
                    }
                    Ok(None) => {
                        println!("✓ You're up to date!");
                        std::process::exit(0);
                    }
                    Err(e) => {
                        eprintln!("✗ Error: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            "-v" | "--version" => {
                println!("App Pro Version: {}", core::app_version());
                std::process::exit(0);
            }
            "-h" | "--help" => {
                println!("App Pro - Production-grade Linux System Utility");
                println!("\nUsage:");
                println!("  app-pro                    Launch the GTK4 GUI Dashboard");
                println!("  app-pro update             Check for updates and install them");
                println!("  app-pro <file.deb>         Pre-select a package file to install");
                println!("  app-pro -v, --version      Show App Pro version");
                println!("  app-pro -h, --help         Show help information");
                std::process::exit(0);
            }
            other => {
                // If it looks like a file or path, assume we want to open it in GUI
                if other.ends_with(".deb") || other.ends_with(".AppImage") || other.ends_with(".zip") || std::path::Path::new(other).exists() {
                    file_to_install = Some(other.to_string());
                } else {
                    eprintln!("Unknown argument: {}", other);
                    eprintln!("Run 'app-pro --help' for usage.");
                    std::process::exit(1);
                }
            }
        }
    }

    let app = Application::builder()
        .application_id("com.app-pro.utility")
        .build();

    app.connect_activate(move |app| {
        let ui = ui::AppProUI::new(app, app_manager.clone(), cleaner_manager.clone());

        if let Some(ref file_path) = file_to_install {
            ui.set_file_path(file_path);
        }

        ui.show();
    });

    app.run();
}

fn ensure_desktop_integration() {
    let desktop_entry = r#"[Desktop Entry]
Name=App Pro
Comment=Linux system utility: install, manage, clean, monitor
Exec=/usr/local/bin/app-pro
Icon=app-pro
Terminal=false
Type=Application
Categories=Utility;System;GTK;
StartupNotify=true
StartupWMClass=app-pro
"#;

    let svg_icon = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" width="512" height="512">
  <defs>
    <linearGradient id="bg" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#2ecc71"/>
      <stop offset="100%" stop-color="#27ae60"/>
    </linearGradient>
  </defs>
  <rect width="512" height="512" rx="96" fill="url(#bg)"/>
  <g transform="translate(96, 120)">
    <rect x="0" y="40" width="320" height="240" rx="24" fill="none" stroke="white" stroke-width="20" opacity=".9"/>
    <rect x="40" y="0" width="240" height="80" rx="16" fill="white" opacity=".2"/>
    <line x1="160" y1="80" x2="160" y2="280" stroke="white" stroke-width="12" opacity=".7"/>
    <line x1="40" y1="160" x2="280" y2="160" stroke="white" stroke-width="12" opacity=".7"/>
  </g>
  <text x="256" y="440" font-family="system-ui,-apple-system,sans-serif" font-size="96" font-weight="800" fill="white" text-anchor="middle" opacity=".95">AP</text>
</svg>"##;

    let install = |path: &std::path::Path, content: &str| -> bool {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::write(path, content).is_ok()
    };

    // Try system-wide locations first, fall back to user-local
    let desktop_sys = std::path::Path::new("/usr/share/applications/app-pro.desktop");
    let desktop_user = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("~/.local/share"))
        .join("applications/app-pro.desktop");
    let icon_sys = std::path::Path::new("/usr/share/icons/hicolor/scalable/apps/app-pro.svg");
    let icon_user = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("~/.local/share"))
        .join("icons/hicolor/scalable/apps/app-pro.svg");

    if !install(desktop_sys, desktop_entry) {
        install(&desktop_user, desktop_entry);
    }
    if !install(icon_sys, svg_icon) {
        install(&icon_user, svg_icon);
    }
}
