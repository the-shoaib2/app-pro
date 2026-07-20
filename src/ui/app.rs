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
        window.set_default_size(520, 420);
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

        // Auto update check on startup in background
        {
            let (sender, receiver) = std::sync::mpsc::channel();
            std::thread::spawn(move || {
                let current_version = crate::core::app_version();
                if let Ok(Some(release)) = crate::updater::check_for_updates(current_version) {
                    sender.send(release).ok();
                }
            });

            let window_clone = window.clone();
            gtk4::glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
                if let Ok(release) = receiver.try_recv() {
                    Self::show_update_dialog(&window_clone, release);
                    gtk4::glib::ControlFlow::Break
                } else {
                    gtk4::glib::ControlFlow::Continue
                }
            });
        }

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
        let css = format!("{}{}", include_str!("style.css"), include_str!("button.css"));
        provider.load_from_string(&css);

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

    fn show_update_dialog(parent: &ApplicationWindow, release: crate::updater::ReleaseInfo) {
        let dialog = gtk4::MessageDialog::new(
            Some(parent),
            gtk4::DialogFlags::MODAL,
            gtk4::MessageType::Question,
            gtk4::ButtonsType::YesNo,
            "Update Available",
        );
        dialog.set_secondary_text(Some(&format!(
            "A new version ({}) of App Pro is available.\n\nWould you like to download and install the update now?",
            release.tag_name
        )));

        dialog.connect_response(move |dialog, response| {
            dialog.close();
            if response == gtk4::ResponseType::Yes {
                let rel = release.clone();
                std::thread::spawn(move || {
                    match crate::updater::perform_update(&rel) {
                        Ok(_) => {
                            gtk4::glib::idle_add_local(move || {
                                let success_dialog = gtk4::MessageDialog::new(
                                    None::<&gtk4::Window>,
                                    gtk4::DialogFlags::MODAL,
                                    gtk4::MessageType::Info,
                                    gtk4::ButtonsType::Ok,
                                    "Update Complete",
                                );
                                success_dialog.set_secondary_text(Some("✓ Update complete! Please restart App Pro to use the new version."));
                                success_dialog.connect_response(|d, _| {
                                    d.close();
                                });
                                success_dialog.show();
                                gtk4::glib::ControlFlow::Break
                            });
                        }
                        Err(e) => {
                            gtk4::glib::idle_add_local(move || {
                                let err_dialog = gtk4::MessageDialog::new(
                                    None::<&gtk4::Window>,
                                    gtk4::DialogFlags::MODAL,
                                    gtk4::MessageType::Error,
                                    gtk4::ButtonsType::Ok,
                                    "Update Failed",
                                );
                                err_dialog.set_secondary_text(Some(&format!("✗ Update failed: {}", e)));
                                err_dialog.connect_response(|d, _| {
                                    d.close();
                                });
                                err_dialog.show();
                                gtk4::glib::ControlFlow::Break
                            });
                        }
                    };
                });
            }
        });
        dialog.show();
    }
}
