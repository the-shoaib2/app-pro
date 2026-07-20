use gtk4::prelude::*;
use gtk4::{self, Box, Label, Button, ScrolledWindow, ListBox, ListBoxRow};

use crate::manager::processes::ProcessManager;

pub struct ProcessesPage {
    pub container: Box,
    list_box: ListBox,
    status_label: Label,
    search_entry: gtk4::SearchEntry,
}

impl ProcessesPage {
    pub fn new() -> Self {
        let container = Box::new(gtk4::Orientation::Vertical, 0);

        let header = Box::new(gtk4::Orientation::Horizontal, 6);
        header.set_css_classes(&["page-header"]);

        // Search Entry
        let search_entry = gtk4::SearchEntry::new();
        search_entry.set_placeholder_text(Some("Search by PID or name..."));
        search_entry.set_width_request(180);
        header.append(&search_entry);

        // Spacer to push buttons to the right
        let spacer = Box::new(gtk4::Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        header.append(&spacer);

        // Refresh button with icon only
        let refresh_btn = Button::from_icon_name("view-refresh-symbolic");
        refresh_btn.set_css_classes(&["action-button"]);

        let kill_btn = Button::with_label("Kill");
        kill_btn.set_css_classes(&["danger-button"]);

        let force_kill_btn = Button::with_label("Force Kill");
        force_kill_btn.set_css_classes(&["danger-button"]);

        header.append(&refresh_btn);
        header.append(&kill_btn);
        header.append(&force_kill_btn);
        container.append(&header);

        let scrolled = ScrolledWindow::new();
        scrolled.set_vexpand(true);
        scrolled.set_hexpand(true);

        let list_box = ListBox::new();
        list_box.set_css_classes(&["process-list"]);
        scrolled.set_child(Some(&list_box));

        container.append(&scrolled);

        let status_label = Label::new(None);
        status_label.set_css_classes(&["status-label"]);
        container.append(&status_label);

        let page = ProcessesPage {
            container,
            list_box,
            status_label,
            search_entry: search_entry.clone(),
        };

        page.refresh();

        // Signal connections
        let list = page.list_box.clone();
        let status = page.status_label.clone();
        let search = page.search_entry.clone();
        refresh_btn.connect_clicked(move |_| {
            Self::populate_list(&list, &status, &search);
        });

        let list = page.list_box.clone();
        let status = page.status_label.clone();
        let search = page.search_entry.clone();
        search_entry.connect_changed(move |_| {
            Self::populate_list(&list, &status, &search);
        });

        let list = page.list_box.clone();
        let status = page.status_label.clone();
        let search = page.search_entry.clone();
        kill_btn.connect_clicked(move |_| {
            if let Some(row) = list.selected_row() {
                if let Some(child) = row.child() {
                    // Find PID label in children
                    if let Some(hbox) = child.dynamic_cast_ref::<Box>() {
                        if let Some(pid_label) = hbox.first_child() {
                            if let Some(label) = pid_label.dynamic_cast_ref::<Label>() {
                                if let Ok(pid) = label.text().parse::<u32>() {
                                    match ProcessManager::kill_process(pid, false) {
                                        Ok(msg) => status.set_text(&msg),
                                        Err(e) => status.set_text(&e),
                                    }
                                    Self::populate_list(&list, &status, &search);
                                }
                            }
                        }
                    }
                }
            } else {
                status.set_text("Select a process first.");
            }
        });

        let list = page.list_box.clone();
        let status = page.status_label.clone();
        let search = page.search_entry.clone();
        force_kill_btn.connect_clicked(move |_| {
            if let Some(row) = list.selected_row() {
                if let Some(child) = row.child() {
                    if let Some(hbox) = child.dynamic_cast_ref::<Box>() {
                        if let Some(pid_label) = hbox.first_child() {
                            if let Some(label) = pid_label.dynamic_cast_ref::<Label>() {
                                if let Ok(pid) = label.text().parse::<u32>() {
                                    match ProcessManager::kill_process(pid, true) {
                                        Ok(msg) => status.set_text(&msg),
                                        Err(e) => status.set_text(&e),
                                    }
                                    Self::populate_list(&list, &status, &search);
                                }
                            }
                        }
                    }
                }
            } else {
                status.set_text("Select a process first.");
            }
        });

        page
    }

    pub fn refresh(&self) {
        Self::populate_list(&self.list_box, &self.status_label, &self.search_entry);
    }

    fn populate_list(list_box: &ListBox, status: &Label, search_entry: &gtk4::SearchEntry) {
        // Remove existing children
        while let Some(child) = list_box.first_child() {
            list_box.remove(&child);
        }

        let query = search_entry.text().to_string().to_lowercase();
        let processes = ProcessManager::list_user_processes();
        let filtered: Vec<_> = processes.into_iter()
            .filter(|p| {
                if query.is_empty() {
                    true
                } else {
                    p.name.to_lowercase().contains(&query) || p.pid.to_string().contains(&query)
                }
            })
            .collect();

        // Header row
        let header_row = ListBoxRow::new();
        let header_box = Box::new(gtk4::Orientation::Horizontal, 12);
        header_box.set_margin_top(8);
        header_box.set_margin_bottom(8);
        header_box.set_margin_start(12);
        header_box.set_margin_end(12);

        let widths: [i32; 5] = [55, 140, 80, 55, 70];
        for (i, text) in ["PID", "Name", "Memory", "State", "User"].iter().enumerate() {
            let h = Label::new(Some(text));
            h.set_css_classes(&["process-header"]);
            h.set_halign(gtk4::Align::Start);
            h.set_width_request(widths[i]);
            header_box.append(&h);
        }
        header_row.set_child(Some(&header_box));
        header_row.set_css_classes(&["process-header-row"]);
        list_box.append(&header_row);

        for proc in &filtered {
            let row = ListBoxRow::new();
            let hbox = Box::new(gtk4::Orientation::Horizontal, 12);
            hbox.set_css_classes(&["process-row"]);

            let pid_label = Label::new(Some(&proc.pid.to_string()));
            pid_label.set_css_classes(&["process-pid"]);
            pid_label.set_width_request(55);
            pid_label.set_halign(gtk4::Align::Start);

            let name_label = Label::new(Some(&proc.name));
            name_label.set_css_classes(&["process-name"]);
            name_label.set_hexpand(true);
            name_label.set_halign(gtk4::Align::Start);
            name_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);

            let mem_str = if proc.memory_bytes > 1024 * 1024 * 1024 {
                format!("{:.1} GB", proc.memory_bytes as f64 / (1024.0 * 1024.0 * 1024.0))
            } else if proc.memory_bytes > 1024 * 1024 {
                format!("{:.1} MB", proc.memory_bytes as f64 / (1024.0 * 1024.0))
            } else {
                format!("{} KB", proc.memory_bytes / 1024)
            };
            let mem_label = Label::new(Some(&mem_str));
            mem_label.set_css_classes(&["process-mem"]);
            mem_label.set_width_request(80);
            mem_label.set_halign(gtk4::Align::End);

            let state_label = Label::new(Some(&proc.state));
            state_label.set_css_classes(&["process-state"]);
            state_label.set_width_request(55);
            state_label.set_halign(gtk4::Align::Center);

            let user_label = Label::new(Some(&proc.user));
            user_label.set_css_classes(&["process-user"]);
            user_label.set_width_request(70);
            user_label.set_halign(gtk4::Align::Start);

            hbox.append(&pid_label);
            hbox.append(&name_label);
            hbox.append(&mem_label);
            hbox.append(&state_label);
            hbox.append(&user_label);
            row.set_child(Some(&hbox));
            list_box.append(&row);
        }

        if query.is_empty() {
            status.set_text(&format!("Total processes: {}", filtered.len()));
        } else {
            status.set_text(&format!("Total processes: {} (filtered: {})", filtered.len(), filtered.len()));
        }
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }
}
