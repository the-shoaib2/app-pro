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
    // Suppress GTK theme asset warnings (Kali's adw-gtk3-dark theme has missing assets)
    std::env::set_var("GTK_DEBUG", "discard-theme");
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

    let args: Vec<String> = env::args().collect();
    let mut file_to_install = None;

    if args.len() > 1 {
        let first_arg = args[1].as_str();
        match first_arg {
            "update" => {
                let current_ver = env!("CARGO_PKG_VERSION");
                println!("Checking for updates... (Current version: {})", current_ver);
                match updater::check_for_updates(current_ver) {
                    Ok(Some(release)) => {
                        println!("New release found: {}", release.tag_name);
                        if let Some(body) = &release.body {
                            println!("\nRelease Notes:\n{}", body);
                        }
                        println!("\nUpdating App Pro...");
                        match updater::perform_update(&release) {
                            Ok(_) => {
                                println!("Update completed successfully! Please restart App Pro.");
                                std::process::exit(0);
                            }
                            Err(e) => {
                                eprintln!("Update failed: {}", e);
                                std::process::exit(1);
                            }
                        }
                    }
                    Ok(None) => {
                        println!("App Pro is already up to date.");
                        std::process::exit(0);
                    }
                    Err(e) => {
                        eprintln!("Error checking for updates: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            "-v" | "--version" => {
                println!("App Pro Version: {}", env!("CARGO_PKG_VERSION"));
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
