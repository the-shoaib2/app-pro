use gtk4::prelude::*;
use gtk4::{self, Box, Label, Button, ScrolledWindow, ListBox, ListBoxRow};
use std::sync::Arc;

use crate::manager::AppManager;

pub struct InstalledAppsPage {
    pub container: Box,
    list_box: ListBox,
    manager: Arc<AppManager>,
    status_label: Label,
}

impl InstalledAppsPage {
    pub fn new(manager: Arc<AppManager>) -> Self {
        let container = Box::new(gtk4::Orientation::Vertical, 0);

        let header = Box::new(gtk4::Orientation::Horizontal, 0);
        header.set_css_classes(&["page-header"]);
        header.set_hexpand(true);

        let title = Label::new(Some("Installed Applications"));
        title.set_css_classes(&["page-title"]);
        title.set_hexpand(true);
        header.append(&title);

        let refresh_button = Button::with_label("Refresh");
        refresh_button.set_css_classes(&["action-button"]);
        header.append(&refresh_button);
        container.append(&header);

        let scrolled = ScrolledWindow::new();
        scrolled.set_vexpand(true);
        scrolled.set_hexpand(true);

        let list_box = ListBox::new();
        list_box.set_css_classes(&["app-list"]);
        scrolled.set_child(Some(&list_box));

        container.append(&scrolled);

        let status_label = Label::new(None);
        status_label.set_css_classes(&["status-label"]);
        container.append(&status_label);

        let page = InstalledAppsPage {
            container,
            list_box,
            manager,
            status_label,
        };

        page.refresh_list();

        let manager = page.manager.clone();
        let list = page.list_box.clone();
        let status = page.status_label.clone();
        refresh_button.connect_clicked(move |_| {
            Self::populate_list(&manager, &list, &status);
        });

        page
    }

    pub fn refresh_list(&self) {
        Self::populate_list(&self.manager, &self.list_box, &self.status_label);
    }

    fn populate_list(manager: &Arc<AppManager>, list_box: &ListBox, status: &Label) {
        // Remove existing children
        while let Some(child) = list_box.first_child() {
            list_box.remove(&child);
        }

        let apps = manager.get_installed_apps();

        if apps.is_empty() {
            let row = ListBoxRow::new();
            let label = Label::new(Some("No applications installed via App Pro yet."));
            label.set_margin_top(20);
            label.set_margin_bottom(20);
            label.set_css_classes(&["empty-label"]);
            row.set_child(Some(&label));
            list_box.append(&row);
            status.set_text("");
            return;
        }

        for app in &apps {
            let row = ListBoxRow::new();
            let hbox = Box::new(gtk4::Orientation::Horizontal, 12);
            hbox.set_margin_top(8);
            hbox.set_margin_bottom(8);
            hbox.set_margin_start(12);
            hbox.set_margin_end(12);

            // App icon placeholder
            let icon_label = Label::new(Some(match app.install_type.as_str() {
                "deb" => "📦",
                "appimage" => "🖥️",
                "zip" => "📁",
                "flatpak" => "📦",
                _ => "❓",
            }));
            icon_label.set_css_classes(&["app-icon"]);

            // App info
            let info_box = Box::new(gtk4::Orientation::Vertical, 2);
            let name_label = Label::new(Some(&app.name));
            name_label.set_halign(gtk4::Align::Start);
            name_label.set_css_classes(&["app-name"]);

            let meta = format!("{}  |  {}  |  {}", app.install_type.to_uppercase(), app.version.as_deref().unwrap_or("v1.0"), app.installed_at);
            let meta_label = Label::new(Some(&meta));
            meta_label.set_halign(gtk4::Align::Start);
            meta_label.set_css_classes(&["app-meta"]);

            info_box.append(&name_label);
            info_box.append(&meta_label);

            // Uninstall button
            let uninstall_btn = Button::with_label("Uninstall");
            uninstall_btn.set_css_classes(&["uninstall-button"]);
            uninstall_btn.set_valign(gtk4::Align::Center);

            let app_clone = app.clone();
            let manager = manager.clone();
            let list = list_box.clone();
            let s = status.clone();
            uninstall_btn.connect_clicked(move |_| {
                s.set_text(&format!("Uninstalling {}...", app_clone.name));
                let result = manager.uninstall_app(&app_clone);
                s.set_text(&result.message);
                Self::populate_list(&manager, &list, &s);
            });

            hbox.append(&icon_label);
            hbox.append(&info_box);
            hbox.append(&uninstall_btn);
            row.set_child(Some(&hbox));
            list_box.append(&row);
        }

        status.set_text(&format!("Total: {} applications", apps.len()));
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }
}
