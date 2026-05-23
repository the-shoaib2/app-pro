use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{self, gio, Box, Label, Button, Frame, Window, ProgressBar};
use std::path::Path;
use std::sync::Arc;

use crate::installer::InstallResult;
use crate::manager::AppManager;

#[allow(dead_code)]
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
    cancel_button: Button,
    progress_bar: ProgressBar,
    log_scroll: gtk4::ScrolledWindow,
    log_view: gtk4::TextView,
    status_label: Label,
    manager: Arc<AppManager>,
    installed_name: std::rc::Rc<std::cell::RefCell<String>>,
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
        card_box.set_margin_start(10);
        card_box.set_margin_end(10);
        card_box.set_margin_top(10);
        card_box.set_margin_bottom(10);

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

        let log_scroll = gtk4::ScrolledWindow::new();
        log_scroll.set_min_content_height(100);
        log_scroll.set_max_content_height(150);
        log_scroll.set_css_classes(&["install-log-scroll"]);
        log_scroll.set_visible(false);

        let log_view = gtk4::TextView::new();
        log_view.set_editable(false);
        log_view.set_cursor_visible(false);
        log_view.set_wrap_mode(gtk4::WrapMode::Word);
        log_view.set_css_classes(&["install-log-view"]);
        log_scroll.set_child(Some(&log_view));
        card_box.append(&log_scroll);

        let action_box = Box::new(gtk4::Orientation::Horizontal, 8);
        action_box.set_halign(gtk4::Align::Center);
        action_box.set_margin_top(8);
        action_box.set_margin_bottom(4);

        let install_button = Button::with_label("Install");
        install_button.set_css_classes(&["primary-button"]);
        install_button.set_visible(false);

        let open_button = Button::with_label("Open");
        open_button.set_css_classes(&["primary-button"]);
        open_button.set_visible(false);

        let cancel_button = Button::with_label("Cancel");
        cancel_button.set_css_classes(&["secondary-button"]);
        cancel_button.set_visible(false);

        action_box.append(&install_button);
        action_box.append(&open_button);
        action_box.append(&cancel_button);
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
        let installed_name: std::rc::Rc<std::cell::RefCell<String>> =
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
            let sl1 = status_label.clone();
            let sl2 = status_label.clone();
            let in1 = installed_name.clone();
            let in2 = installed_name.clone();
            let cb1 = cancel_button.clone();
            let cb2 = cancel_button.clone();
            let ls1 = log_scroll.clone();
            let ls2 = log_scroll.clone();

            browse_button.connect_clicked(move |_| {
                open_dialog(&p1, &fi1, &dz1, &ficon1, &fname1, &fpath1, &ib1, &ob1, &pb1, &sl1, &in1, &cb1, &ls1);
            });

            let gesture = gtk4::GestureClick::new();
            gesture.connect_pressed(move |_, _, _, _| {
                open_dialog(&p2, &fi2, &dz2, &ficon2, &fname2, &fpath2, &ib2, &ob2, &pb2, &sl2, &in2, &cb2, &ls2);
            });
            drop_zone.add_controller(gesture);
        }

        let path = current_path.clone();
        let pb = progress_bar.clone();
        let status = status_label.clone();
        let ib = install_button.clone();
        let ob = open_button.clone();
        let mgr = manager.clone();
        let installed_name2 = installed_name.clone();
        let status2 = status.clone();
        let installed_name_for_install = installed_name.clone();
        let log_scroll_c = log_scroll.clone();
        let log_view_c = log_view.clone();

        install_button.connect_clicked(move |_| {
            let p = path.borrow().clone();
            if p.is_empty() {
                return;
            }
            ib.set_sensitive(false);
            pb.set_visible(true);
            pb.set_fraction(0.05);
            pb.set_text(Some("Installing..."));

            // Clear and show log panel
            log_view_c.buffer().set_text("");
            log_scroll_c.set_visible(true);

            let mgr = mgr.clone();
            let (tx, rx) = std::sync::mpsc::channel::<InstallResult>();
            let (log_tx, log_rx) = std::sync::mpsc::channel::<String>();

            std::thread::spawn(move || {
                let result = mgr.install_file(&p, Some(log_tx));
                tx.send(result).ok();
            });

            let rx = std::sync::Mutex::new(rx);
            let log_rx = std::sync::Mutex::new(log_rx);
            let pb2 = pb.clone();
            let status2 = status.clone();
            let ib2 = ib.clone();
            let ob2 = ob.clone();
            let name = installed_name_for_install.clone();
            let received_result = std::rc::Rc::new(std::cell::RefCell::new(None));
            let log_scroll2 = log_scroll_c.clone();
            let log_view2 = log_view_c.clone();

            glib::timeout_add_local(std::time::Duration::from_millis(80), {
                let received_result = received_result.clone();
                move || {
                    // Read pending log messages
                    let mut added_lines = false;
                    while let Ok(line) = log_rx.lock().unwrap().try_recv() {
                        let buffer = log_view2.buffer();
                        let mut iter = buffer.end_iter();
                        buffer.insert(&mut iter, &format!("{}\n", line));
                        added_lines = true;
                    }

                    if added_lines {
                        // Scroll to bottom
                        let adj = log_scroll2.vadjustment();
                        adj.set_value(adj.upper() - adj.page_size());
                    }

                    // Try to receive the result if we haven't yet
                    if received_result.borrow().is_none() {
                        if let Ok(result) = rx.lock().unwrap().try_recv() {
                            *received_result.borrow_mut() = Some(result);
                        }
                    }

                    // If we have received the result, animate the progress to 100%
                    if let Some(ref result) = *received_result.borrow() {
                        let current = pb2.fraction();
                        if current < 0.95 {
                            let next = (current + 0.15).min(1.0);
                            pb2.set_fraction(next);
                            pb2.set_text(Some(&format!("Installing... {}%", (next * 100.0) as i32)));
                            glib::ControlFlow::Continue
                        } else {
                            pb2.set_fraction(1.0);
                            pb2.set_text(Some("Done ✓"));
                            status2.set_text(&result.message);
                            ib2.set_sensitive(true);
                            ib2.set_visible(false);
                            *name.borrow_mut() = result.app_name.clone();
                            ob2.set_visible(true);
                            glib::ControlFlow::Break
                        }
                    } else {
                        // Still installing: advance progress bar slowly with a smooth decay curve capping at 90%
                        let current = pb2.fraction();
                        let limit = 0.90;
                        if current < limit {
                            let step = (limit - current) * 0.015; // Smooth asymptotic easing (1.5% of remaining distance per 80ms)
                            pb2.set_fraction(current + step);
                        }
                        let pct = (pb2.fraction() * 100.0) as i32;
                        pb2.set_text(Some(&format!("Installing... {}%", pct)));
                        glib::ControlFlow::Continue
                    }
                }
            });
        });

        open_button.connect_clicked(move |_| {
            let app_name = installed_name2.borrow().clone();
            if app_name.is_empty() {
                status2.set_text("App name not found.");
                return;
            }
            let launched = launch_app(&app_name);
            if launched {
                status2.set_text(&format!("Launched {}", app_name));
            } else {
                status2.set_text(&format!("Could not launch {}", app_name));
            }
        });

        let cb_c = cancel_button.clone();
        let ib_c = install_button.clone();
        let ob_c = open_button.clone();
        let pb_c = progress_bar.clone();
        let fi_c = file_info.clone();
        let dz_c = drop_zone.clone();
        let path_c = current_path.clone();
        let name_c = installed_name.clone();
        let status_c = status_label.clone();
        let log_scroll_c2 = log_scroll.clone();

        cancel_button.connect_clicked(move |_| {
            *path_c.borrow_mut() = String::new();
            *name_c.borrow_mut() = String::new();
            fi_c.set_visible(false);
            pb_c.set_visible(false);
            log_scroll_c2.set_visible(false);
            ib_c.set_visible(false);
            ob_c.set_visible(false);
            cb_c.set_visible(false);
            status_c.set_text("");
            dz_c.set_visible(true);
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
            cancel_button,
            progress_bar,
            log_scroll,
            log_view,
            status_label,
            manager,
            installed_name,
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
            &self.cancel_button,
            &self.progress_bar,
            &self.log_scroll,
            &self.status_label,
            &self.installed_name,
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
    status_label: &Label,
    installed_name: &std::rc::Rc<std::cell::RefCell<String>>,
    cancel_button: &Button,
    log_scroll: &gtk4::ScrolledWindow,
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
    let sl = status_label.clone();
    let in_name = installed_name.clone();
    let cb = cancel_button.clone();
    let ls = log_scroll.clone();

    dialog.open(Some(&window), None::<&gio::Cancellable>, move |result| {
        if let Ok(file) = result {
            if let Some(p) = file.path() {
                *path.borrow_mut() = p.to_string_lossy().to_string();
                select_file_inner(&p, &path, &dz, &fi, &fi_icon, &fi_name, &fi_path, &ib, &ob, &cb, &pb, &ls, &sl, &in_name);
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
    cancel_button: &Button,
    progress_bar: &ProgressBar,
    log_scroll: &gtk4::ScrolledWindow,
    status_label: &Label,
    installed_name: &std::rc::Rc<std::cell::RefCell<String>>,
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
    progress_bar.set_visible(false);
    log_scroll.set_visible(false);
    cancel_button.set_visible(true);

    let app_name = get_app_name_from_path(path);
    *installed_name.borrow_mut() = app_name.clone();

    if app_name.eq_ignore_ascii_case("app-pro") || app_name.eq_ignore_ascii_case("app_pro") || app_name.eq_ignore_ascii_case("apppro") {
        install_button.set_visible(true);
        install_button.set_sensitive(false);
        install_button.set_label("Install");
        open_button.set_visible(false);
        status_label.set_text("App Pro is already running and cannot install itself.");
    } else if is_app_installed(&app_name, &path.to_string_lossy()) {
        install_button.set_visible(true);
        install_button.set_sensitive(true);
        install_button.set_label("Reinstall");
        open_button.set_visible(true);
        status_label.set_text("Application is already installed.");
    } else {
        install_button.set_visible(true);
        install_button.set_sensitive(true);
        install_button.set_label("Install");
        open_button.set_visible(false);
        status_label.set_text("");
    }
}

fn get_app_name_from_path(path: &Path) -> String {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    match ext.as_str() {
        "deb" => {
            let (name, _) = crate::installer::deb::DebInstaller::get_deb_info(path);
            name
        }
        "appimage" => {
            let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("unknown");
            crate::installer::appimage::AppImageInstaller::extract_name(filename)
        }
        "zip" => {
            let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("unknown");
            filename.strip_suffix(".zip").unwrap_or(filename).to_string()
        }
        _ => {
            path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown").to_string()
        }
    }
}

fn is_app_installed(app_name: &str, file_path: &str) -> bool {
    let ext = std::path::Path::new(file_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    if ext == "deb" {
        if let Ok(output) = std::process::Command::new("dpkg-query")
            .args(["-W", "-f=${Status}", app_name])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("install ok installed") {
                return true;
            }
        }
    }

    let search_dirs = [
        std::path::PathBuf::from("/usr/share/applications"),
        dirs::data_dir().map(|d| d.join("applications")).unwrap_or_default(),
        std::path::PathBuf::from(
            std::env::var("HOME").unwrap_or_default(),
        )
        .join(".local/share/applications"),
    ];

    for dir in &search_dirs {
        if !dir.exists() {
            continue;
        }
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.extension().and_then(|e| e.to_str()) != Some("desktop") {
                    continue;
                }
                let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                let mut match_found = stem.eq_ignore_ascii_case(app_name)
                    || stem.eq_ignore_ascii_case(&format!("app-pro-{}", app_name));

                let mut desktop_name = String::new();
                let mut exec_name = String::new();

                if let Ok(content) = std::fs::read_to_string(&p) {
                    for line in content.lines() {
                        let line_trimmed = line.trim();
                        if line_trimmed.starts_with("Name=") {
                            desktop_name = line_trimmed.trim_start_matches("Name=").to_string();
                        } else if line_trimmed.starts_with("Exec=") {
                            let exec_line = line_trimmed.trim_start_matches("Exec=");
                            let cmd = if exec_line.starts_with('"') {
                                if let Some(end) = exec_line[1..].find('"') {
                                    &exec_line[1..end + 1]
                                } else {
                                    exec_line
                                }
                            } else {
                                exec_line.split_whitespace().next().unwrap_or("")
                            };
                            exec_name = std::path::Path::new(cmd)
                                .file_name()
                                .and_then(|s| s.to_str())
                                .unwrap_or("")
                                .to_string();
                        }
                    }
                }

                if !match_found {
                    match_found = desktop_name.eq_ignore_ascii_case(app_name)
                        || exec_name.eq_ignore_ascii_case(app_name)
                        || exec_name.starts_with(app_name);
                }

                if match_found {
                    return true;
                }
            }
        }
    }

    false
}

fn launch_app(app_name: &str) -> bool {
    let search_dirs = [
        std::path::PathBuf::from("/usr/share/applications"),
        dirs::data_dir().map(|d| d.join("applications")).unwrap_or_default(),
        std::path::PathBuf::from(
            std::env::var("HOME").unwrap_or_default(),
        )
        .join(".local/share/applications"),
    ];

    for dir in &search_dirs {
        if !dir.exists() {
            continue;
        }
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.extension().and_then(|e| e.to_str()) != Some("desktop") {
                    continue;
                }
                let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                let mut match_found = stem.eq_ignore_ascii_case(app_name)
                    || stem.eq_ignore_ascii_case(&format!("app-pro-{}", app_name));

                let mut desktop_name = String::new();
                let mut exec_name = String::new();
                let mut exec_cmd = String::new();

                if let Ok(content) = std::fs::read_to_string(&p) {
                    for line in content.lines() {
                        let line_trimmed = line.trim();
                        if line_trimmed.starts_with("Name=") {
                            desktop_name = line_trimmed.trim_start_matches("Name=").to_string();
                        } else if line_trimmed.starts_with("Exec=") {
                            exec_cmd = line_trimmed.trim_start_matches("Exec=").to_string();
                            let cmd = if exec_cmd.starts_with('"') {
                                if let Some(end) = exec_cmd[1..].find('"') {
                                    &exec_cmd[1..end + 1]
                                } else {
                                    &exec_cmd
                                }
                            } else {
                                exec_cmd.split_whitespace().next().unwrap_or("")
                            };
                            exec_name = std::path::Path::new(cmd)
                                .file_name()
                                .and_then(|s| s.to_str())
                                .unwrap_or("")
                                .to_string();
                        }
                    }
                }

                if !match_found {
                    match_found = desktop_name.eq_ignore_ascii_case(app_name)
                        || exec_name.eq_ignore_ascii_case(app_name)
                        || exec_name.starts_with(app_name);
                }

                if match_found {
                    log::info!("Found matching desktop file for {}: {}", app_name, p.display());
                    
                    // If we parsed an Exec command, clean and launch it directly in the background
                    if !exec_cmd.is_empty() {
                        let mut cleaned = exec_cmd.clone();
                        for placeholder in &["%U", "%u", "%F", "%f", "%i", "%c", "%k"] {
                            cleaned = cleaned.replace(placeholder, "");
                        }
                        let cleaned = cleaned.trim();
                        log::info!("Executing parsed desktop launch command: {}", cleaned);
                        if std::process::Command::new("sh")
                            .args(["-c", &format!("nohup {} >/dev/null 2>&1 &", cleaned)])
                            .spawn()
                            .is_ok()
                        {
                            return true;
                        }
                    }

                    // Fallbacks
                    let base = p.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                    if std::process::Command::new("gtk-launch")
                        .arg(base)
                        .spawn()
                        .is_ok()
                    {
                        return true;
                    }
                    if std::process::Command::new("xdg-open")
                        .arg(&p)
                        .spawn()
                        .is_ok()
                    {
                        return true;
                    }
                }
            }
        }
    }
    false
}
