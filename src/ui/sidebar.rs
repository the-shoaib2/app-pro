use gtk4::prelude::*;
use gtk4::{self, ListBox, ListBoxRow, Label};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Install,
    InstalledApps,
    RunningProcesses,
    Cleaner,
    SystemInfo,
}

pub struct Sidebar {
    pub container: ListBox,
    pages: Vec<(Page, String)>,
}

impl Sidebar {
    pub fn new() -> Self {
        let sidebar = ListBox::new();
        sidebar.set_css_classes(&["sidebar"]);

        let pages: Vec<(Page, String)> = vec![
            (Page::Install, "📥  Install".to_string()),
            (Page::InstalledApps, "📦  Installed".to_string()),
            (Page::RunningProcesses, "⚙️  Processes".to_string()),
            (Page::Cleaner, "🧹  Cleaner".to_string()),
            (Page::SystemInfo, "ℹ️  System Info".to_string()),
        ];

        for (_, label_text) in &pages {
            let row = ListBoxRow::new();
            let label = Label::new(Some(label_text));
            label.set_halign(gtk4::Align::Start);
            label.set_margin_start(16);
            label.set_margin_end(16);
            label.set_margin_top(8);
            label.set_margin_bottom(8);
            label.set_css_classes(&["sidebar-label"]);
            row.set_child(Some(&label));
            row.set_css_classes(&["sidebar-row"]);
            sidebar.append(&row);
        }

        // Select first item
        if let Some(first_row) = sidebar.first_child() {
            if let Some(list_box_row) = first_row.dynamic_cast_ref::<ListBoxRow>() {
                sidebar.select_row(Some(list_box_row));
            }
        }

        sidebar.set_activate_on_single_click(true);

        Sidebar { container: sidebar, pages }
    }

    pub fn connect_page_changed<F: Fn(Page) + 'static>(&self, callback: F) {
        let pages = self.pages.clone();
        self.container.connect_row_activated(move |_, row| {
            let index = row.index();
            if index >= 0 {
                if let Some(page_data) = pages.get(index as usize) {
                    callback(page_data.0);
                }
            }
        });
    }

    pub fn widget(&self) -> &ListBox {
        &self.container
    }
}
