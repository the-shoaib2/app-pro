use gtk4::prelude::*;
use gtk4::{self, Application, ApplicationWindow, Stack, Box, HeaderBar, Label as GtkLabel};
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

        let header_bar = HeaderBar::new();
        header_bar.set_show_title_buttons(true);
        let title_widget = GtkLabel::new(Some("App Pro - System Utility"));
        title_widget.set_css_classes(&["window-title"]);
        header_bar.set_title_widget(Some(&title_widget));

        let layout = Box::new(gtk4::Orientation::Vertical, 0);
        layout.append(&header_bar);
        layout.append(sidebar.widget());
        layout.append(&main_stack);
        main_stack.set_vexpand(true);

        let window = ApplicationWindow::new(app);
        window.set_title(Some("App Pro - System Utility"));
        window.set_default_size(680, 560);
        window.set_resizable(true);
        window.set_child(Some(&layout));

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
}
