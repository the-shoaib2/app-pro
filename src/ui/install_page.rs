use gtk4::prelude::*;
use gtk4::{self, gio, Box, Label, Button, Frame, Entry, Window};
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
        let container = Box::new(gtk4::Orientation::Vertical, 0);

        let header = Box::new(gtk4::Orientation::Vertical, 2);
        header.set_css_classes(&["page-header"]);
        let title = Label::new(Some("Install Application"));
        title.set_css_classes(&["page-title"]);
        header.append(&title);
        let desc = Label::new(Some("Select a .deb, .AppImage, or .zip file to install."));
        desc.set_css_classes(&["page-description"]);
        header.append(&desc);
        container.append(&header);

        let content = Box::new(gtk4::Orientation::Vertical, 0);
        content.set_css_classes(&["content-area"]);
        content.set_vexpand(true);

        let card = Frame::new(None);
        card.set_css_classes(&["install-card"]);

        let card_box = Box::new(gtk4::Orientation::Vertical, 0);
        card_box.set_margin_start(12);
        card_box.set_margin_end(12);
        card_box.set_margin_top(12);
        card_box.set_margin_bottom(12);

        let drop_zone = Frame::new(None);
        drop_zone.set_css_classes(&["drop-zone"]);
        drop_zone.set_hexpand(true);
        drop_zone.set_vexpand(true);
        drop_zone.set_margin_bottom(16);

        let drop_content = Box::new(gtk4::Orientation::Vertical, 6);
        drop_content.set_valign(gtk4::Align::Center);
        drop_content.set_halign(gtk4::Align::Center);

        let drop_icon = Label::new(Some("⬆"));
        drop_icon.set_css_classes(&["drop-icon"]);
        drop_content.append(&drop_icon);

        let drop_text = Label::new(Some("Drop file here"));
        drop_text.set_css_classes(&["drop-text"]);
        drop_content.append(&drop_text);

        let drop_hint = Label::new(Some("or click Browse to select a file"));
        drop_hint.set_css_classes(&["drop-hint"]);
        drop_content.append(&drop_hint);

        drop_zone.set_child(Some(&drop_content));
        card_box.append(&drop_zone);

        let file_row = Box::new(gtk4::Orientation::Horizontal, 8);

        let file_entry = Entry::new();
        file_entry.set_placeholder_text(Some("Choose a file..."));
        file_entry.set_hexpand(true);
        file_entry.set_css_classes(&["file-entry"]);

        let browse_button = Button::with_label("Browse");
        browse_button.set_css_classes(&["browse-button"]);

        file_row.append(&file_entry);
        file_row.append(&browse_button);
        card_box.append(&file_row);

        let sep = Frame::new(None);
        sep.set_css_classes(&["install-separator"]);
        sep.set_margin_top(20);
        sep.set_margin_bottom(12);
        sep.set_size_request(-1, 1);
        card_box.append(&sep);

        let install_button = Button::with_label("Install");
        install_button.set_css_classes(&["primary-button"]);
        install_button.set_halign(gtk4::Align::Center);
        card_box.append(&install_button);

        card.set_child(Some(&card_box));

        let card_wrap = Box::new(gtk4::Orientation::Vertical, 0);
        card_wrap.set_margin_start(12);
        card_wrap.set_margin_end(12);
        card_wrap.set_margin_top(6);
        card_wrap.append(&card);

        content.append(&card_wrap);
        container.append(&content);

        let status_label = Label::new(None);
        status_label.set_css_classes(&["status-label"]);
        status_label.set_wrap(true);
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

        let entry = self.file_path_entry.clone();
        browse_button.connect_clicked(move |btn| {
            if let Some(window) = btn.root().and_then(|r| r.dynamic_cast::<Window>().ok()) {
                let filter = gtk4::FileFilter::new();
                filter.add_pattern("*.deb");
                filter.add_pattern("*.AppImage");
                filter.add_pattern("*.zip");
                let dialog = gtk4::FileDialog::new();
                dialog.set_title("Select Installer File");
                dialog.set_default_filter(Some(&filter));
                let entry = entry.clone();
                dialog.open(Some(&window), None::<&gio::Cancellable>, move |result| {
                    if let Ok(file) = result {
                        let path_str = file.path()
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_default();
                        entry.set_text(&path_str);
                    }
                });
            }
        });
    }

    pub(crate) fn set_file_path(&self, path: &str) {
        self.file_path_entry.set_text(path);
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }
}
