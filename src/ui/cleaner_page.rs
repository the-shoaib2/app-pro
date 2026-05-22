use gtk4::prelude::*;
use gtk4::{self, Box, Label, Button, ScrolledWindow, ListBox, ListBoxRow};
use std::sync::Arc;

use crate::cleaner::CleanupManager;
use crate::cleaner::cache::CacheAnalyzer;

pub struct CleanerPage {
    pub container: Box,
    list_box: ListBox,
    status_label: Label,
    clean_all_btn: Button,
    cleaner: Arc<CleanupManager>,
}

impl CleanerPage {
    pub fn new(cleaner: Arc<CleanupManager>) -> Self {
        let container = Box::new(gtk4::Orientation::Vertical, 0);

        let header = Box::new(gtk4::Orientation::Vertical, 2);
        header.set_css_classes(&["page-header"]);

        let title = Label::new(Some("System Cleaner"));
        title.set_css_classes(&["page-title"]);
        header.append(&title);

        let desc = Label::new(Some("Clean up system caches and free disk space."));
        desc.set_css_classes(&["page-description"]);
        header.append(&desc);
        container.append(&header);

        let scrolled = ScrolledWindow::new();
        scrolled.set_vexpand(true);
        scrolled.set_hexpand(true);

        let list_box = ListBox::new();
        list_box.set_css_classes(&["cleaner-list"]);
        scrolled.set_child(Some(&list_box));

        container.append(&scrolled);

        let bottom = Box::new(gtk4::Orientation::Vertical, 6);
        bottom.set_margin_top(4);
        bottom.set_margin_bottom(4);

        let status_label = Label::new(None);
        status_label.set_css_classes(&["status-label"]);
        bottom.append(&status_label);

        let clean_all_btn = Button::with_label("Clean All");
        clean_all_btn.set_css_classes(&["primary-button"]);
        clean_all_btn.set_halign(gtk4::Align::Center);
        bottom.append(&clean_all_btn);

        container.append(&bottom);

        let page = CleanerPage {
            container,
            list_box,
            status_label,
            clean_all_btn,
            cleaner,
        };

        page.setup_clean_all();
        page.refresh();

        page
    }

    fn setup_clean_all(&self) {
        let cleaner = self.cleaner.clone();
        let list = self.list_box.clone();
        let status = self.status_label.clone();
        self.clean_all_btn.connect_clicked(move |_| {
            status.set_text("Running all cleaners...");
            let results = [cleaner.clean_user_cache(),
                cleaner.clean_apt_cache(),
                cleaner.clean_thumbnails(),
                cleaner.clean_orphan_packages()];
            let total: u64 = results.iter().map(|r| r.bytes_freed).sum();
            let total_str = if total > 1024 * 1024 * 1024 {
                format!("{:.2} GB", total as f64 / (1024.0 * 1024.0 * 1024.0))
            } else if total > 1024 * 1024 {
                format!("{:.2} MB", total as f64 / (1024.0 * 1024.0))
            } else {
                format!("{} bytes", total)
            };
            status.set_text(&format!("Clean complete! Total space freed: {}", total_str));
            Self::populate_list(&cleaner, &list, &status);
        });
    }

    pub fn refresh(&self) {
        Self::populate_list(&self.cleaner, &self.list_box, &self.status_label);
    }

    fn populate_list(cleaner: &Arc<CleanupManager>, list_box: &ListBox, status: &Label) {
        while let Some(child) = list_box.first_child() {
            list_box.remove(&child);
        }

        let cache_sizes = CacheAnalyzer::get_all_cache_sizes();

        for (name, path, size) in &cache_sizes {
            let row = ListBoxRow::new();
            let hbox = Box::new(gtk4::Orientation::Horizontal, 12);
            hbox.set_margin_top(10);
            hbox.set_margin_bottom(10);
            hbox.set_margin_start(16);
            hbox.set_margin_end(16);

            let size_str = if *size > 1024 * 1024 * 1024 {
                format!("{:.2} GB", *size as f64 / (1024.0 * 1024.0 * 1024.0))
            } else if *size > 1024 * 1024 {
                format!("{:.2} MB", *size as f64 / (1024.0 * 1024.0))
            } else if *size > 1024 {
                format!("{:.1} KB", *size as f64 / 1024.0)
            } else {
                format!("{} B", size)
            };

            let info_box = Box::new(gtk4::Orientation::Vertical, 2);
            let name_label = Label::new(Some(name));
            name_label.set_halign(gtk4::Align::Start);
            name_label.set_css_classes(&["cleaner-name"]);

            let path_label = Label::new(Some(path));
            path_label.set_halign(gtk4::Align::Start);
            path_label.set_css_classes(&["cleaner-path"]);
            path_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);

            let size_label = Label::new(Some(&size_str));
            size_label.set_halign(gtk4::Align::Start);
            size_label.set_css_classes(&["cleaner-size"]);

            info_box.append(&name_label);
            info_box.append(&path_label);
            info_box.append(&size_label);

            let clean_btn = Button::with_label("Clean");
            clean_btn.set_css_classes(&["clean-button"]);
            clean_btn.set_valign(gtk4::Align::Center);

            let c = cleaner.clone();
            let list = list_box.clone();
            let s = status.clone();
            let name_clone = name.clone();

            clean_btn.connect_clicked(move |_| {
                s.set_text(&format!("Cleaning {}...", name_clone));
                let result = match name_clone.as_str() {
                    "User Cache" => c.clean_user_cache(),
                    "APT Cache" => c.clean_apt_cache(),
                    "Thumbnails" => c.clean_thumbnails(),
                    _ => {
                        s.set_text(&format!("Unknown cache type: {}", name_clone));
                        return;
                    }
                };

                let msg = if result.success {
                    let freed = if result.bytes_freed > 1024 * 1024 {
                        format!("{:.2} MB", result.bytes_freed as f64 / (1024.0 * 1024.0))
                    } else {
                        format!("{} bytes", result.bytes_freed)
                    };
                    format!("{}: Freed {}", result.description, freed)
                } else {
                    result.message.clone()
                };
                s.set_text(&msg);
                Self::populate_list(&c, &list, &s);
            });

            hbox.append(&info_box);
            hbox.append(&clean_btn);
            row.set_child(Some(&hbox));
            list_box.append(&row);
        }

        // Orphan packages row
        let orphan_row = ListBoxRow::new();
        let orphan_box = Box::new(gtk4::Orientation::Horizontal, 12);
        orphan_box.set_margin_top(10);
        orphan_box.set_margin_bottom(10);
        orphan_box.set_margin_start(16);
        orphan_box.set_margin_end(16);

        let orphan_info = Box::new(gtk4::Orientation::Vertical, 2);
        let orphan_name = Label::new(Some("Orphaned Packages"));
        orphan_name.set_halign(gtk4::Align::Start);
        orphan_name.set_css_classes(&["cleaner-name"]);

        let orphan_desc = Label::new(Some("Remove unused dependencies (autoremove)"));
        orphan_desc.set_halign(gtk4::Align::Start);
        orphan_desc.set_css_classes(&["cleaner-path"]);

        orphan_info.append(&orphan_name);
        orphan_info.append(&orphan_desc);

        let orphan_btn = Button::with_label("Clean");
        orphan_btn.set_valign(gtk4::Align::Center);
        orphan_btn.set_css_classes(&["clean-button"]);

        let cleaner_orphan = cleaner.clone();
        let list_orphan = list_box.clone();
        let s_orphan = status.clone();
        orphan_btn.connect_clicked(move |_| {
            s_orphan.set_text("Removing orphaned packages...");
            let result = cleaner_orphan.clean_orphan_packages();
            s_orphan.set_text(&result.message);
            Self::populate_list(&cleaner_orphan, &list_orphan, &s_orphan);
        });

        orphan_box.append(&orphan_info);
        orphan_box.append(&orphan_btn);
        orphan_row.set_child(Some(&orphan_box));
        list_box.append(&orphan_row);
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }
}
