use gtk4::prelude::*;
use gtk4::{self, Box, Label, Button, ScrolledWindow, ListBox, ListBoxRow};
use std::sync::Arc;

use crate::manager::AppManager;
use crate::manager::desktop_scanner::{DesktopAppInfo, AppOrigin};

#[derive(Clone, Copy, PartialEq)]
enum Filter {
    All,
    User,
    AppPro,
}

pub struct InstalledAppsPage {
    pub container: Box,
    list_box: ListBox,
    status_label: Label,
    manager: Arc<AppManager>,
    filter: std::rc::Rc<std::cell::RefCell<Filter>>,
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

        let filter_bar = Box::new(gtk4::Orientation::Horizontal, 4);
        filter_bar.set_css_classes(&["filter-bar"]);
        filter_bar.set_margin_start(12);
        filter_bar.set_margin_end(12);
        filter_bar.set_margin_top(8);
        filter_bar.set_margin_bottom(4);

        let all_btn = Button::with_label("All Apps");
        all_btn.set_css_classes(&["filter-button"]);
        let user_btn = Button::with_label("User");
        user_btn.set_css_classes(&["filter-button"]);
        let pro_btn = Button::with_label("App Pro");
        pro_btn.set_css_classes(&["filter-button", "filter-active"]);

        filter_bar.append(&all_btn);
        filter_bar.append(&user_btn);
        filter_bar.append(&pro_btn);
        container.append(&filter_bar);

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

        let filter = std::rc::Rc::new(std::cell::RefCell::new(Filter::AppPro));

        let list = list_box.clone();
        let status = status_label.clone();
        let mgr = manager.clone();
        let filt = filter.clone();

        refresh_button.connect_clicked(move |_| {
            let f = *filt.borrow();
            Self::populate_list(&mgr, &list, &status, f);
        });

        let filter_buttons = [all_btn, user_btn, pro_btn];

        for (i, btn) in filter_buttons.iter().enumerate() {
            let list = list_box.clone();
            let status = status_label.clone();
            let mgr = manager.clone();
            let fbs: [Button; 3] = [
                filter_buttons[0].clone(),
                filter_buttons[1].clone(),
                filter_buttons[2].clone(),
            ];
            let filt = filter.clone();

            btn.connect_clicked(move |_| {
                for (j, fb) in fbs.iter().enumerate() {
                    fb.set_css_classes(if j == i {
                        &["filter-button", "filter-active"]
                    } else {
                        &["filter-button"]
                    });
                }
                let f = match i {
                    0 => Filter::All,
                    1 => Filter::User,
                    _ => Filter::AppPro,
                };
                *filt.borrow_mut() = f;
                Self::populate_list(&mgr, &list, &status, f);
            });
        }

        let page = InstalledAppsPage {
            container,
            list_box,
            status_label,
            manager,
            filter,
        };

        page.refresh_list();
        page
    }

    pub fn refresh_list(&self) {
        let f = *self.filter.borrow();
        Self::populate_list(&self.manager, &self.list_box, &self.status_label, f);
    }

    fn populate_list(manager: &Arc<AppManager>, list_box: &ListBox, status: &Label, filter: Filter) {
        while let Some(child) = list_box.first_child() {
            list_box.remove(&child);
        }

        let apps = manager.scan_all_desktop_apps();

        let filtered: Vec<&DesktopAppInfo> = match filter {
            Filter::All => apps.iter().collect(),
            Filter::User => apps.iter().filter(|a| a.origin != AppOrigin::System).collect(),
            Filter::AppPro => apps.iter().filter(|a| matches!(a.origin, AppOrigin::AppPro)).collect(),
        };

        if filtered.is_empty() {
            let row = ListBoxRow::new();
            let label = Label::new(Some(match filter {
                Filter::All => "No applications found.",
                Filter::User => "No user-installed applications found.",
                Filter::AppPro => "No applications installed via App Pro yet.",
            }));
            label.set_margin_top(20);
            label.set_margin_bottom(20);
            label.set_css_classes(&["empty-label"]);
            row.set_child(Some(&label));
            list_box.append(&row);
            status.set_text("");
            return;
        }

        let pro_apps = manager.get_installed_apps();

        for app in &filtered {
            let row = ListBoxRow::new();
            let hbox = Box::new(gtk4::Orientation::Horizontal, 10);
            hbox.set_margin_top(6);
            hbox.set_margin_bottom(6);
            hbox.set_margin_start(12);
            hbox.set_margin_end(12);

            let icon_label = Label::new(Some("📄"));
            icon_label.set_css_classes(&["app-icon"]);

            let info_box = Box::new(gtk4::Orientation::Vertical, 1);
            let name_label = Label::new(Some(&app.name));
            name_label.set_halign(gtk4::Align::Start);
            name_label.set_css_classes(&["app-name"]);

            let origin_str = app.origin.as_str();
            let meta = if let Some(comment) = &app.comment {
                format!("{}  ·  {}", origin_str, comment)
            } else {
                origin_str.to_string()
            };
            let meta_label = Label::new(Some(&meta));
            meta_label.set_halign(gtk4::Align::Start);
            meta_label.set_css_classes(&["app-meta"]);

            info_box.append(&name_label);
            info_box.append(&meta_label);

            hbox.append(&icon_label);
            hbox.append(&info_box);

            // Origin badge
            let badge = Label::new(Some(match app.origin {
                AppOrigin::AppPro => "App Pro",
                AppOrigin::User => "User",
                AppOrigin::System => "System",
            }));
            badge.set_valign(gtk4::Align::Center);
            badge.set_css_classes(&["origin-badge", match app.origin {
                AppOrigin::AppPro => "badge-pro",
                AppOrigin::User => "badge-user",
                AppOrigin::System => "badge-system",
            }]);
            hbox.append(&badge);

            let action_btn = Button::new();
            if app.is_app_pro {
                action_btn.set_label("Uninstall");
                action_btn.set_css_classes(&["uninstall-button"]);
                if let Some(db_app) = pro_apps.iter().find(|a| a.name == app.name) {
                    let db_app = db_app.clone();
                    let mgr = manager.clone();
                    let list = list_box.clone();
                    let s = status.clone();
                    action_btn.connect_clicked(move |_| {
                        s.set_text(&format!("Uninstalling {}...", db_app.name));
                        let result = mgr.uninstall_app(&db_app);
                        s.set_text(&result.message);
                        Self::populate_list(&mgr, &list, &s, filter);
                    });
                }
            } else {
                action_btn.set_label("Open");
                action_btn.set_css_classes(&["action-button"]);
                let exec = app.exec.clone();
                action_btn.connect_clicked(move |_| {
                    let _bin = exec.split_whitespace().next().unwrap_or(&exec);
                    std::process::Command::new("sh")
                        .args(["-c", &exec])
                        .spawn()
                        .ok();
                });
            }
            action_btn.set_valign(gtk4::Align::Center);
            hbox.append(&action_btn);

            row.set_child(Some(&hbox));
            list_box.append(&row);
        }

        status.set_text(&format!("Showing {} of {} apps", filtered.len(), apps.len()));
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }
}
