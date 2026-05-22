use gtk4::prelude::*;
use gtk4::{self, Box, Button};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Install,
    InstalledApps,
    RunningProcesses,
    Cleaner,
    SystemInfo,
}

pub struct Sidebar {
    pub container: Box,
    #[allow(dead_code)]
    active_page: Page,
}

impl Sidebar {
    pub fn new() -> Self {
        let container = Box::new(gtk4::Orientation::Horizontal, 0);
        container.set_css_classes(&["tab-bar"]);

        let page_labels = vec![
            (Page::Install, "Install"),
            (Page::InstalledApps, "Installed"),
            (Page::RunningProcesses, "Processes"),
            (Page::Cleaner, "Cleaner"),
            (Page::SystemInfo, "System Info"),
        ];

        for (_, label) in &page_labels {
            let btn = Button::with_label(label);
            btn.set_css_classes(&["tab-button"]);
            btn.set_can_focus(false);
            if let Some(cursor) = gtk4::gdk::Cursor::from_name("pointer", None) {
                btn.set_cursor(Some(&cursor));
            }
            container.append(&btn);
        }

        if let Some(child) = container.first_child() {
            if let Some(btn) = child.dynamic_cast_ref::<Button>() {
                btn.set_css_classes(&["tab-button", "tab-active"]);
            }
        }

        Sidebar {
            container,
            active_page: Page::Install,
        }
    }

    pub fn connect_page_changed<F: Fn(Page) + 'static + Clone>(&self, callback: F) {
        let tab_bar = self.container.clone();
        let mut index = 0;
        let mut child = self.container.first_child();
        while let Some(widget) = child {
            if let Some(btn) = widget.dynamic_cast_ref::<Button>() {
                let page = match index {
                    0 => Page::Install,
                    1 => Page::InstalledApps,
                    2 => Page::RunningProcesses,
                    3 => Page::Cleaner,
                    4 => Page::SystemInfo,
                    _ => Page::Install,
                };
                let cb = callback.clone();
                let bar = tab_bar.clone();
                let btn_clone = btn.clone();
                btn.connect_clicked(move |_| {
                    let mut c = bar.first_child();
                    while let Some(w) = c {
                        if let Some(b) = w.dynamic_cast_ref::<Button>() {
                            b.set_css_classes(&["tab-button"]);
                        }
                        c = w.next_sibling();
                    }
                    btn_clone.set_css_classes(&["tab-button", "tab-active"]);
                    cb(page);
                });
                index += 1;
            }
            child = widget.next_sibling();
        }
        callback(Page::Install);
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }
}
