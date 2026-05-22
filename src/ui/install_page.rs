use gtk4::prelude::*;
use gtk4::{self, Box, Label, Button, Frame, Entry, ResponseType, Window};
use std::path::Path;
use std::sync::Arc;

use crate::manager::AppManager;

pub struct InstallPage {
    pub container: Box,
    status_label: Label,
    file_path_entry: Entry,
    install_button: Button,
    manager: Arc<AppManager>,
}

impl InstallPage {
    pub fn new(manager: Arc<AppManager>) -> Self {
        let container = Box::new(gtk4::Orientation::Vertical, 12);

        let title = Label::new(Some("Install Application"));
        title.set_css_classes(&["page-title"]);
        container.append(&title);

        let desc = Label::new(Some("Select a .deb, .AppImage, or .zip file to install."));
        desc.set_css_classes(&["page-description"]);
        container.append(&desc);

        // Drop zone
        let drop_zone = Frame::new(None);
        drop_zone.set_css_classes(&["drop-zone"]);
        drop_zone.set_hexpand(true);
        drop_zone.set_vexpand(true);
        drop_zone.set_margin_top(12);
        drop_zone.set_margin_bottom(12);
        drop_zone.set_margin_start(24);
        drop_zone.set_margin_end(24);

        let drop_content = Box::new(gtk4::Orientation::Vertical, 6);
        drop_content.set_valign(gtk4::Align::Center);
        drop_content.set_halign(gtk4::Align::Center);

        let drop_icon = Label::new(Some("📁"));
        drop_icon.set_css_classes(&["drop-icon"]);
        drop_content.append(&drop_icon);

        let drop_text = Label::new(Some("Drop file here or click Browse below"));
        drop_text.set_css_classes(&["drop-text"]);
        drop_content.append(&drop_text);

        let drop_hint = Label::new(Some("Supported: .deb, .AppImage, .zip"));
        drop_hint.set_css_classes(&["drop-hint"]);
        drop_content.append(&drop_hint);

        drop_zone.set_child(Some(&drop_content));

        // File path entry + browse
        let file_row = Box::new(gtk4::Orientation::Horizontal, 8);
        file_row.set_margin_start(24);
        file_row.set_margin_end(24);

        let file_entry = Entry::new();
        file_entry.set_placeholder_text(Some("Select file path..."));
        file_entry.set_hexpand(true);
        file_entry.set_css_classes(&["file-entry"]);

        let browse_button = Button::with_label("Browse");
        browse_button.set_css_classes(&["action-button"]);

        file_row.append(&file_entry);
        file_row.append(&browse_button);

        // Install button
        let install_button = Button::with_label("Install");
        install_button.set_css_classes(&["primary-button"]);
        install_button.set_halign(gtk4::Align::Center);
        install_button.set_margin_top(12);

        // Status area
        let status_label = Label::new(None);
        status_label.set_css_classes(&["status-label"]);
        status_label.set_wrap(true);

        container.append(&drop_zone);
        container.append(&file_row);
        container.append(&install_button);
        container.append(&status_label);

        let page = InstallPage {
            container,
            status_label,
            file_path_entry: file_entry,
            install_button,
            manager,
        };

        page.setup_signals(browse_button);

        page
    }

    fn setup_signals(&self, browse_button: Button) {
        let manager = self.manager.clone();
        let entry = self.file_path_entry.clone();
        let status = self.status_label.clone();

        self.install_button.connect_clicked(move |_| {
            let path = entry.text().to_string();
            if path.is_empty() {
                status.set_text("Please select a file first.");
                return;
            }
            if !Path::new(&path).exists() {
                status.set_text("File does not exist.");
                return;
            }
            status.set_text(&format!("Installing {}...", path));
            let result = manager.install_file(&path);
            status.set_text(&result.message);
        });

        let entry_clone = self.file_path_entry.clone();
        browse_button.connect_clicked(move |btn| {
            if let Some(window) = btn.root().and_then(|r| r.dynamic_cast::<Window>().ok()) {
                let dialog = gtk4::Dialog::builder()
                    .title("Select Installer File")
                    .transient_for(&window)
                    .modal(true)
                    .build();

                // Use FileDialog instead for modern GTK4
                let _entry = entry_clone.clone();
                dialog.connect_response(move |dialog, response| {
                    if response == ResponseType::Accept || response == ResponseType::Ok {
                        // In a real implementation, use FileDialog
                    }
                    dialog.close();
                });
                dialog.add_button("Cancel", ResponseType::Cancel);
                dialog.add_button("Open", ResponseType::Accept);
                dialog.show();
            }
        });
    }

    pub fn set_file_path(&self, path: &str) {
        self.file_path_entry.set_text(path);
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }
}
