use gtk4::prelude::*;
use gtk4::{self, Application, ApplicationWindow, Stack, Paned};
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
    main_stack: Stack,
    sidebar: Sidebar,
    install_page: InstallPage,
    installed_page: InstalledAppsPage,
    processes_page: ProcessesPage,
    cleaner_page: CleanerPage,
    info_page: InfoPage,
    manager: Arc<AppManager>,
    cleaner: Arc<CleanupManager>,
}

impl AppProUI {
    pub fn new(app: &Application, manager: Arc<AppManager>, cleaner: Arc<CleanupManager>) -> Self {

        // Build pages
        let install_page = InstallPage::new(manager.clone());
        let installed_page = InstalledAppsPage::new(manager.clone());
        let processes_page = ProcessesPage::new();
        let cleaner_page = CleanerPage::new(cleaner.clone());
        let info_page = InfoPage::new();

        // Main stack
        let main_stack = Stack::new();
        main_stack.set_transition_type(gtk4::StackTransitionType::Crossfade);
        main_stack.set_transition_duration(200);

        main_stack.add_titled(install_page.widget(), Some("install"), "Install");
        main_stack.add_titled(installed_page.widget(), Some("installed"), "Installed");
        main_stack.add_titled(processes_page.widget(), Some("processes"), "Processes");
        main_stack.add_titled(cleaner_page.widget(), Some("cleaner"), "Cleaner");
        main_stack.add_titled(info_page.widget(), Some("info"), "System Info");

        // Sidebar
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

        // Main layout using Paned
        let paned = Paned::new(gtk4::Orientation::Horizontal);
        paned.set_start_child(Some(sidebar.widget()));
        paned.set_end_child(Some(&main_stack));
        paned.set_position(220);
        paned.set_wide_handle(true);

        // Window
        let window = ApplicationWindow::new(app);
        window.set_title(Some("App Pro - System Utility"));
        window.set_default_size(1000, 700);
        window.set_child(Some(&paned));

        // Apply CSS
        Self::apply_css(&window);

        AppProUI {
            window,
            main_stack,
            sidebar,
            install_page,
            installed_page,
            processes_page,
            cleaner_page,
            info_page,
            manager,
            cleaner,
        }
    }

    pub fn show(&self) {
        self.window.present();
    }

    fn apply_css(_window: &ApplicationWindow) {
        let provider = gtk4::CssProvider::new();
        provider.load_from_string(include_str!("style.css"));

        // Apply to display
        if let Some(display) = gtk4::gdk::Display::default() {
            gtk4::StyleContext::add_provider_for_display(
                &display,
                &provider,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
    }

    pub fn window(&self) -> &ApplicationWindow {
        &self.window
    }
}
