use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{self, gio, Box, Label, Button, Frame, Window, ProgressBar};
use std::path::Path;
use std::sync::Arc;

use crate::installer::InstallResult;
use crate::manager::AppManager;

pub struct InstallPage {
    pub container: Box,
    file_path_entry: std::rc::Rc<std::cell::RefCell<String>>,
    drop_zone: Frame,
    file_info: Box,
    file_icon: Label,
    file_name: Label,
    file_path: Label,
    install_button: Button,
    open_button: Button,
    progress_bar: ProgressBar,
    status_label: Label,
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

        let drop_icon = Label::new(Some("📦"));
        drop_icon.set_css_classes(&["drop-icon"]);
        drop_content.append(&drop_icon);

        let drop_text = Label::new(Some("Drag & drop or click to browse"));
        drop_text.set_css_classes(&["drop-text"]);
        drop_content.append(&drop_text);

        let browse_button = Button::with_label("Browse Files");
        browse_button.set_css_classes(&["primary-button"]);
        browse_button.set_halign(gtk4::Align::Center);
        browse_button.set_margin_top(8);
        drop_content.append(&browse_button);

        drop_zone.set_child(Some(&drop_content));
        card_box.append(&drop_zone);

        let file_info = Box::new(gtk4::Orientation::Horizontal, 10);
        file_info.set_css_classes(&["file-info"]);
        file_info.set_margin_bottom(16);
        file_info.set_visible(false);

        let file_icon = Label::new(None);
        file_icon.set_css_classes(&["file-info-icon"]);

        let file_details = Box::new(gtk4::Orientation::Vertical, 2);
        let file_name = Label::new(None);
        file_name.set_css_classes(&["file-info-name"]);
        file_name.set_halign(gtk4::Align::Start);
        let file_path = Label::new(None);
        file_path.set_css_classes(&["file-info-path"]);
        file_path.set_halign(gtk4::Align::Start);
        file_path.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        file_details.append(&file_name);
        file_details.append(&file_path);

        file_info.append(&file_icon);
        file_info.append(&file_details);
        card_box.append(&file_info);

        let progress_bar = ProgressBar::new();
        progress_bar.set_css_classes(&["install-progress"]);
        progress_bar.set_show_text(true);
        progress_bar.set_fraction(0.0);
        progress_bar.set_visible(false);
        card_box.append(&progress_bar);

        let sep = Frame::new(None);
        sep.set_css_classes(&["install-separator"]);
        sep.set_margin_top(20);
        sep.set_margin_bottom(12);
        sep.set_size_request(-1, 1);
        card_box.append(&sep);

        let action_box = Box::new(gtk4::Orientation::Horizontal, 8);
        action_box.set_halign(gtk4::Align::Center);

        let install_button = Button::with_label("Install");
        install_button.set_css_classes(&["primary-button"]);
        install_button.set_visible(false);

        let open_button = Button::with_label("Open");
        open_button.set_css_classes(&["primary-button"]);
        open_button.set_visible(false);

        action_box.append(&install_button);
        action_box.append(&open_button);
        card_box.append(&action_box);
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

        let current_path: std::rc::Rc<std::cell::RefCell<String>> =
            std::rc::Rc::new(std::cell::RefCell::new(String::new()));

        {
            let p1 = current_path.clone();
            let p2 = current_path.clone();
            let fi1 = file_info.clone();
            let fi2 = file_info.clone();
            let dz1 = drop_zone.clone();
            let dz2 = drop_zone.clone();
            let ficon1 = file_icon.clone();
            let ficon2 = file_icon.clone();
            let fname1 = file_name.clone();
            let fname2 = file_name.clone();
            let fpath1 = file_path.clone();
            let fpath2 = file_path.clone();
            let ib1 = install_button.clone();
            let ib2 = install_button.clone();
            let ob1 = open_button.clone();
            let ob2 = open_button.clone();
            let pb1 = progress_bar.clone();
            let pb2 = progress_bar.clone();

            browse_button.connect_clicked(move |_| {
                open_dialog(&p1, &fi1, &dz1, &ficon1, &fname1, &fpath1, &ib1, &ob1, &pb1);
            });

            let gesture = gtk4::GestureClick::new();
            gesture.connect_pressed(move |_, _, _, _| {
                open_dialog(&p2, &fi2, &dz2, &ficon2, &fname2, &fpath2, &ib2, &ob2, &pb2);
            });
            drop_zone.add_controller(gesture);
        }

        let path = current_path.clone();
        let pb = progress_bar.clone();
        let status = status_label.clone();
        let ib = install_button.clone();
        let ob = open_button.clone();
        let mgr = manager.clone();

        install_button.connect_clicked(move |_| {
            let p = path.borrow().clone();
            if p.is_empty() {
                return;
            }
            ib.set_sensitive(false);
            pb.set_visible(true);
            pb.set_fraction(0.0);
            pb.set_text(Some("Installing..."));

            let mgr = mgr.clone();
            let (tx, rx) = std::sync::mpsc::channel::<InstallResult>();

            std::thread::spawn(move || {
                let result = mgr.install_file(&p);
                tx.send(result).ok();
            });

            let rx = std::sync::Mutex::new(rx);
            let pb2 = pb.clone();
            let status2 = status.clone();
            let ib2 = ib.clone();
            let ob2 = ob.clone();

            glib::timeout_add_local(std::time::Duration::from_millis(120), move || {
                if let Ok(result) = rx.lock().unwrap().try_recv() {
                    pb2.set_fraction(1.0);
                    pb2.set_text(Some("Done ✓"));
                    status2.set_text(&result.message);
                    ib2.set_sensitive(true);
                    ib2.set_visible(false);
                    ob2.set_visible(true);
                    glib::ControlFlow::Break
                } else {
                    let frac = pb2.fraction();
                    let next = (frac + 0.07).min(0.9);
                    pb2.set_fraction(next);
                    pb2.set_text(Some(&format!("Installing... {}%", (next * 100.0) as i32)));
                    glib::ControlFlow::Continue
                }
            });
        });

        let path = current_path.clone();
        let mgr = manager.clone();
        let status = status_label.clone();
        open_button.connect_clicked(move |_| {
            let p = path.borrow().clone();
            let result = mgr.install_file(&p);
            status.set_text(&format!("Launching... {}", result.message));
        });

        InstallPage {
            container,
            file_path_entry: current_path,
            drop_zone,
            file_info,
            file_icon,
            file_name,
            file_path,
            install_button,
            open_button,
            progress_bar,
            status_label,
            manager,
        }
    }

    pub(crate) fn set_file_path(&self, path: &str) {
        select_file_inner(
            Path::new(path),
            &self.file_path_entry,
            &self.drop_zone,
            &self.file_info,
            &self.file_icon,
            &self.file_name,
            &self.file_path,
            &self.install_button,
            &self.open_button,
            &self.progress_bar,
        );
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }
}

fn open_dialog(
    current_path: &std::rc::Rc<std::cell::RefCell<String>>,
    file_info: &Box,
    drop_zone: &Frame,
    file_icon: &Label,
    file_name: &Label,
    file_path: &Label,
    install_button: &Button,
    open_button: &Button,
    progress_bar: &ProgressBar,
) {
    let window = match drop_zone.root().and_then(|r| r.dynamic_cast::<Window>().ok()) {
        Some(w) => w,
        None => return,
    };
    let filter = gtk4::FileFilter::new();
    filter.add_pattern("*.deb");
    filter.add_pattern("*.AppImage");
    filter.add_pattern("*.zip");
    let dialog = gtk4::FileDialog::new();
    dialog.set_title("Select Installer File");
    dialog.set_default_filter(Some(&filter));

    let path = current_path.clone();
    let fi = file_info.clone();
    let dz = drop_zone.clone();
    let fi_icon = file_icon.clone();
    let fi_name = file_name.clone();
    let fi_path = file_path.clone();
    let ib = install_button.clone();
    let ob = open_button.clone();
    let pb = progress_bar.clone();

    dialog.open(Some(&window), None::<&gio::Cancellable>, move |result| {
        if let Ok(file) = result {
            if let Some(p) = file.path() {
                *path.borrow_mut() = p.to_string_lossy().to_string();
                select_file_inner(&p, &path, &dz, &fi, &fi_icon, &fi_name, &fi_path, &ib, &ob, &pb);
            }
        }
    });
}

fn select_file_inner(
    path: &Path,
    current_path: &std::rc::Rc<std::cell::RefCell<String>>,
    drop_zone: &Frame,
    file_info: &Box,
    file_icon: &Label,
    file_name: &Label,
    file_path: &Label,
    install_button: &Button,
    open_button: &Button,
    progress_bar: &ProgressBar,
) {
    let name = path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let ext = path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let icon = match ext.as_str() {
        "deb" => "📦",
        "appimage" => "⚡",
        "zip" => "🗜️",
        _ => "📄",
    };

    *current_path.borrow_mut() = path.to_string_lossy().to_string();
    file_icon.set_text(icon);
    file_name.set_text(&name);
    file_path.set_text(&path.to_string_lossy());

    drop_zone.set_visible(false);
    file_info.set_visible(true);
    install_button.set_visible(true);
    open_button.set_visible(false);
    progress_bar.set_visible(false);
}
