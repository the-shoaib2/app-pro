mod core;
mod installer;
mod manager;
mod cleaner;
mod db;
mod ui;

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

    let args: Vec<String> = env::args().collect();
    let file_to_install = args.get(1).cloned();

    let app = Application::builder()
        .application_id("com.app-pro.utility")
        .build();

    app.connect_activate(move |app| {
        let ui = ui::AppProUI::new(app, app_manager.clone(), cleaner_manager.clone());

        if let Some(ref file_path) = file_to_install {
            log::info!("File to install: {}", file_path);
        }

        ui.show();
    });

    app.run();
}
