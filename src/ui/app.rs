use gtk4::prelude::*;
use gtk4::{self, Application, ApplicationWindow, Stack, Box};
use std::sync::Arc;

use crate::manager::AppManager;
use crate::cleaner::CleanupManager;
use crate::ui::sidebar::{Sidebar, Page};
use crate::ui::install_page::InstallPage;
use crate::ui::installed_page::InstalledAppsPage;
use crate::ui::processes_page::ProcessesPage;
use crate::ui::cleaner_page::CleanerPage;
use crate::ui::info_page::InfoPage;

pub struct AppProUI {
    window: ApplicationWindow,
    #[allow(dead_code)]
    main_stack: Stack,
    #[allow(dead_code)]
    install_page: InstallPage,
    #[allow(dead_code)]
    installed_page: InstalledAppsPage,
    #[allow(dead_code)]
    processes_page: ProcessesPage,
    #[allow(dead_code)]
    cleaner_page: CleanerPage,
    #[allow(dead_code)]
    info_page: InfoPage,
}

impl AppProUI {
    pub fn new(app: &Application, manager: Arc<AppManager>, cleaner: Arc<CleanupManager>) -> Self {

        let install_page = InstallPage::new(manager.clone());
        let installed_page = InstalledAppsPage::new(manager.clone());
        let processes_page = ProcessesPage::new();
        let cleaner_page = CleanerPage::new(cleaner.clone());
        let info_page = InfoPage::new();

        let main_stack = Stack::new();
        main_stack.set_transition_type(gtk4::StackTransitionType::Crossfade);
        main_stack.set_transition_duration(200);

        main_stack.add_titled(install_page.widget(), Some("install"), "Install");
        main_stack.add_titled(installed_page.widget(), Some("installed"), "Installed");
        main_stack.add_titled(processes_page.widget(), Some("processes"), "Processes");
        main_stack.add_titled(cleaner_page.widget(), Some("cleaner"), "Cleaner");
        main_stack.add_titled(info_page.widget(), Some("info"), "System Info");

        let sidebar = Sidebar::new();
        sidebar.connect_page_changed({
            let stack = main_stack.clone();
            move |page| {
                let name = match page {
                    Page::Install => "install",
                    Page::InstalledApps => "installed",
                    Page::RunningProcesses => "processes",
                    Page::Cleaner => "cleaner",
                    Page::SystemInfo => "info",
                };
                stack.set_visible_child_name(name);
            }
        });

        let layout = Box::new(gtk4::Orientation::Vertical, 0);
        layout.append(sidebar.widget());
        layout.append(&main_stack);
        main_stack.set_vexpand(true);

        let window = ApplicationWindow::new(app);
        window.set_title(Some("App Pro - System Utility"));
        window.set_default_size(680, 560);
        window.set_resizable(true);
        window.set_child(Some(&layout));

        // Set application icon from assets
        if let Some(display) = gtk4::gdk::Display::default() {
            let icon_theme = gtk4::IconTheme::for_display(&display);
            if let Ok(current_dir) = std::env::current_dir() {
                icon_theme.add_search_path(current_dir.join("assets"));
            }
            icon_theme.add_search_path("/home/kali/Desktop/app-pro/assets");
            window.set_icon_name(Some("image"));
        }
        Self::setup_theme_sync();

        // Follow system dark theme preference
        if let Some(settings) = gtk4::Settings::default() {
            settings.set_gtk_application_prefer_dark_theme(true);
        }

        Self::apply_css(&window);

        AppProUI {
            window,
            main_stack,
            install_page,
            installed_page,
            processes_page,
            cleaner_page,
            info_page,
        }
    }

    pub fn show(&self) {
        self.window.present();
    }

    pub fn set_file_path(&self, path: &str) {
        self.install_page.set_file_path(path);
    }

    fn apply_css(_window: &ApplicationWindow) {
        let provider = gtk4::CssProvider::new();
        provider.load_from_string(include_str!("style.css"));

        if let Some(display) = gtk4::gdk::Display::default() {
            gtk4::style_context_add_provider_for_display(
                &display,
                &provider,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
    }

    fn is_system_dark_mode() -> bool {
        // 1. Try reading from GSettings interface schema if available
        let schema_exists = gtk4::gio::SettingsSchemaSource::default()
            .and_then(|source| source.lookup("org.gnome.desktop.interface", true))
            .is_some();

        if schema_exists {
            let settings = gtk4::gio::Settings::new("org.gnome.desktop.interface");
            let color_scheme = settings.string("color-scheme");
            if color_scheme.contains("dark") {
                return true;
            }
            let theme_name = settings.string("gtk-theme");
            if theme_name.to_lowercase().contains("dark") {
                return true;
            }
        }

        // 2. Fallback to executing gsettings CLI commands
        if let Ok(output) = std::process::Command::new("gsettings")
            .args(["get", "org.gnome.desktop.interface", "color-scheme"])
            .output()
        {
            let scheme = String::from_utf8_lossy(&output.stdout).to_lowercase();
            if scheme.contains("dark") {
                return true;
            }
        }

        if let Ok(output) = std::process::Command::new("gsettings")
            .args(["get", "org.gnome.desktop.interface", "gtk-theme"])
            .output()
        {
            let theme = String::from_utf8_lossy(&output.stdout).to_lowercase();
            if theme.contains("dark") {
                return true;
            }
        }

        // 3. Fallback to GtkSettings defaults
        if let Some(gtk_settings) = gtk4::Settings::default() {
            if let Some(theme_name) = gtk_settings.gtk_theme_name() {
                if theme_name.to_lowercase().contains("dark") {
                    return true;
                }
            }
        }

        false
    }

    fn setup_theme_sync() {
        let schema_exists = gtk4::gio::SettingsSchemaSource::default()
            .and_then(|source| source.lookup("org.gnome.desktop.interface", true))
            .is_some();

        if schema_exists {
            let gsettings = gtk4::gio::Settings::new("org.gnome.desktop.interface");
            let gtk_settings = gtk4::Settings::default().unwrap();
            
            // Set initial state
            gtk_settings.set_gtk_application_prefer_dark_theme(Self::is_system_dark_mode());

            // Connect signal to listen to changes dynamically
            gsettings.connect_changed(None, move |_, key| {
                if key == "color-scheme" || key == "gtk-theme" {
                    if let Some(s) = gtk4::Settings::default() {
                        s.set_gtk_application_prefer_dark_theme(Self::is_system_dark_mode());
                    }
                }
            });
        } else {
            // Fallback for non-GNOME/XFCE environments where schema is missing
            if let Some(gtk_settings) = gtk4::Settings::default() {
                gtk_settings.set_gtk_application_prefer_dark_theme(Self::is_system_dark_mode());

                gtk_settings.connect_gtk_theme_name_notify(move |s| {
                    if let Some(theme_name) = s.gtk_theme_name() {
                        let is_dark = theme_name.to_lowercase().contains("dark");
                        s.set_gtk_application_prefer_dark_theme(is_dark);
                    }
                });
            }
        }
    }
}
