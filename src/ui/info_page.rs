use gtk4::prelude::*;
use gtk4::{self, Box, Label, Frame};
use std::path::Path;

pub struct InfoPage {
    pub container: Box,
}

impl InfoPage {
    pub fn new() -> Self {
        let container = Box::new(gtk4::Orientation::Vertical, 0);


        let scrolled = gtk4::ScrolledWindow::new();
        scrolled.set_vexpand(true);
        scrolled.set_hexpand(true);

        let content = Box::new(gtk4::Orientation::Vertical, 4);
        content.set_margin_top(8);
        content.set_margin_bottom(8);

        content.append(&Self::section("SYSTEM", &[
            ("OS", std::env::consts::OS),
            ("Architecture", std::env::consts::ARCH),
            ("Hostname", &whoami_host()),
            ("Kernel", &read_os_release("kernel-version").unwrap_or_else(get_kernel)),
            ("Desktop", &std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_else(|_| "Unknown".to_string())),
        ]));

        let mem_total = get_mem_info("MemTotal");
        let mem_avail = get_mem_info("MemAvailable");
        let mem_used = mem_total.saturating_sub(mem_avail);
        let mem_pct = if mem_total > 0 { (mem_used as f64 / mem_total as f64 * 100.0) as u64 } else { 0 };

        content.append(&Self::section("MEMORY", &[
            ("Total", &format_memory(mem_total)),
            ("Used", &format_memory(mem_used)),
            ("Available", &format_memory(mem_avail)),
            ("Usage", &format!("{}%", mem_pct)),
        ]));

        let cpu_count = num_cpus();
        let cpu_model = get_cpu_model();
        content.append(&Self::section("CPU", &[
            ("Model", &cpu_model),
            ("Cores", &cpu_count.to_string()),
        ]));

        for mount in list_mounts() {
            let mp = Path::new(&mount);
            let (total, free) = statvfs_size(mp);
            if total == 0 { continue; }
            let used = total.saturating_sub(free);
            let pct = if total > 0 { (used as f64 / total as f64 * 100.0) as u64 } else { 0 };
            let label = if mount == "/" { "System (/) ".to_string() } else { mount.clone() };
            content.append(&Self::section(&label, &[
                ("Total", &format_memory(total)),
                ("Used", &format_memory(used)),
                ("Free", &format_memory(free)),
                ("Usage", &format!("{}%", pct)),
            ]));
        }

        let app_dir = dirs::data_dir()
            .unwrap_or_else(|| Path::new("/tmp").to_path_buf())
            .join("app-pro");
        let app_size = if app_dir.exists() {
            crate::core::SystemExec::get_size(&app_dir)
        } else {
            0
        };

        // Custom APP PRO section with Interactive Updater
        let app_frame = Frame::new(None);
        app_frame.set_css_classes(&["info-section"]);

        let app_card = Box::new(gtk4::Orientation::Vertical, 0);
        app_card.set_css_classes(&["info-section-card"]);

        let app_title = Label::new(Some("APP PRO"));
        app_title.set_css_classes(&["info-section-title"]);
        app_title.set_halign(gtk4::Align::Start);
        app_title.set_margin_start(10);
        app_title.set_margin_end(10);
        app_title.set_margin_top(10);
        app_card.append(&app_title);

        let app_inner = Box::new(gtk4::Orientation::Vertical, 0);
        app_inner.set_margin_start(10);
        app_inner.set_margin_end(10);
        app_inner.set_margin_bottom(8);

        // Version Row
        let ver_row = Box::new(gtk4::Orientation::Horizontal, 0);
        ver_row.set_css_classes(&["info-row"]);
        ver_row.set_margin_top(5);
        ver_row.set_margin_bottom(5);

        let ver_key = Label::new(Some("Version"));
        ver_key.set_css_classes(&["info-key"]);
        ver_key.set_halign(gtk4::Align::Start);
        ver_key.set_hexpand(true);

        let ver_val = Label::new(Some(crate::core::app_version()));
        ver_val.set_css_classes(&["info-value"]);
        ver_val.set_halign(gtk4::Align::End);

        ver_row.append(&ver_key);
        ver_row.append(&ver_val);
        app_inner.append(&ver_row);

        let sep1 = Box::new(gtk4::Orientation::Horizontal, 0);
        sep1.set_css_classes(&["info-separator"]);
        app_inner.append(&sep1);

        // Data Size Row
        let size_row = Box::new(gtk4::Orientation::Horizontal, 0);
        size_row.set_css_classes(&["info-row"]);
        size_row.set_margin_top(5);
        size_row.set_margin_bottom(5);

        let size_key = Label::new(Some("Data Size"));
        size_key.set_css_classes(&["info-key"]);
        size_key.set_halign(gtk4::Align::Start);
        size_key.set_hexpand(true);

        let size_val = Label::new(Some(&format_memory(app_size)));
        size_val.set_css_classes(&["info-value"]);
        size_val.set_halign(gtk4::Align::End);

        size_row.append(&size_key);
        size_row.append(&size_val);
        app_inner.append(&size_row);

        let sep2 = Box::new(gtk4::Orientation::Horizontal, 0);
        sep2.set_css_classes(&["info-separator"]);
        app_inner.append(&sep2);

        // Update Controller UI
        let update_row = Box::new(gtk4::Orientation::Horizontal, 10);
        update_row.set_margin_top(8);
        update_row.set_margin_bottom(4);

        let check_btn = gtk4::Button::with_label("Check for Updates");
        check_btn.set_css_classes(&["primary-button"]);
        check_btn.set_valign(gtk4::Align::Center);

        let update_status = Label::new(None);
        update_status.set_css_classes(&["app-meta"]);
        update_status.set_halign(gtk4::Align::Start);
        update_status.set_hexpand(true);
        update_status.set_wrap(true);
        update_status.set_max_width_chars(35);

        update_row.append(&check_btn);
        update_row.append(&update_status);
        app_inner.append(&update_row);

        app_card.append(&app_inner);
        app_frame.set_child(Some(&app_card));
        content.append(&app_frame);

        enum UpdateMessage {
            Checking,
            NoUpdate,
            UpdateAvailable(crate::updater::ReleaseInfo),
            CheckFailed(String),
            Installing,
            UpdateSuccess,
            UpdateFailed(String),
        }

        let current_version = crate::core::app_version();
        let update_state = std::sync::Arc::new(std::sync::Mutex::new(None));

        let btn_clone = check_btn.clone();
        let status_clone = update_status.clone();
        let state_clone = update_state.clone();

        // Setup Main Thread channel receiver using std::sync::mpsc and glib timeout polling
        let (sender, receiver) = std::sync::mpsc::channel();
        
        gtk4::glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
            while let Ok(msg) = receiver.try_recv() {
                match msg {
                    UpdateMessage::Checking => {
                        status_clone.set_text("Checking for updates...");
                        btn_clone.set_sensitive(false);
                    }
                    UpdateMessage::NoUpdate => {
                        status_clone.set_text("App Pro is up to date.");
                        btn_clone.set_label("Check for Updates");
                        btn_clone.set_sensitive(true);
                    }
                    UpdateMessage::UpdateAvailable(release) => {
                        status_clone.set_text(&format!("New version {} available!", release.tag_name));
                        btn_clone.set_label("Install Update");
                        btn_clone.set_sensitive(true);
                        let mut guard = state_clone.lock().unwrap();
                        *guard = Some(release);
                    }
                    UpdateMessage::CheckFailed(err) => {
                        status_clone.set_text(&format!("Check failed: {}", err));
                        btn_clone.set_label("Check for Updates");
                        btn_clone.set_sensitive(true);
                    }
                    UpdateMessage::Installing => {
                        status_clone.set_text("Installing update... Please wait.");
                        btn_clone.set_sensitive(false);
                    }
                    UpdateMessage::UpdateSuccess => {
                        status_clone.set_text("Update successful! Restart the application to apply.");
                        btn_clone.set_label("Restart Required");
                        btn_clone.set_sensitive(false);
                    }
                    UpdateMessage::UpdateFailed(err) => {
                        status_clone.set_text(&format!("Update failed: {}", err));
                        btn_clone.set_label("Check for Updates");
                        btn_clone.set_sensitive(true);
                    }
                }
            }
            gtk4::glib::ControlFlow::Continue
        });

        let sender_clone = sender.clone();
        let state_clone2 = update_state.clone();

        check_btn.connect_clicked(move |_| {
            let mut state_guard = state_clone2.lock().unwrap();
            if let Some(release) = state_guard.take() {
                let tx = sender_clone.clone();
                tx.send(UpdateMessage::Installing).ok();
                std::thread::spawn(move || {
                    match crate::updater::perform_update(&release) {
                        Ok(_) => {
                            tx.send(UpdateMessage::UpdateSuccess).ok();
                        }
                        Err(e) => {
                            tx.send(UpdateMessage::UpdateFailed(e)).ok();
                        }
                    }
                });
            } else {
                let tx = sender_clone.clone();
                let version_str = current_version.to_string();
                tx.send(UpdateMessage::Checking).ok();
                std::thread::spawn(move || {
                    match crate::updater::check_for_updates(&version_str) {
                        Ok(Some(release)) => {
                            tx.send(UpdateMessage::UpdateAvailable(release)).ok();
                        }
                        Ok(None) => {
                            tx.send(UpdateMessage::NoUpdate).ok();
                        }
                        Err(e) => {
                            tx.send(UpdateMessage::CheckFailed(e)).ok();
                        }
                    }
                });
            }
        });

        scrolled.set_child(Some(&content));
        container.append(&scrolled);

        InfoPage { container }
    }

    fn section(title: &str, entries: &[(&str, &str)]) -> Frame {
        let frame = Frame::new(None);
        frame.set_css_classes(&["info-section"]);

        let card = Box::new(gtk4::Orientation::Vertical, 0);
        card.set_css_classes(&["info-section-card"]);

        let title_label = Label::new(Some(title));
        title_label.set_css_classes(&["info-section-title"]);
        title_label.set_halign(gtk4::Align::Start);
        title_label.set_margin_start(10);
        title_label.set_margin_end(10);
        title_label.set_margin_top(10);
        card.append(&title_label);

        let inner = Box::new(gtk4::Orientation::Vertical, 0);
        inner.set_margin_start(10);
        inner.set_margin_end(10);
        inner.set_margin_bottom(8);

        for (i, (key, value)) in entries.iter().enumerate() {
            if i > 0 {
                let sep = Box::new(gtk4::Orientation::Horizontal, 0);
                sep.set_css_classes(&["info-separator"]);
                inner.append(&sep);
            }

            let row = Box::new(gtk4::Orientation::Horizontal, 0);
            row.set_css_classes(&["info-row"]);
            row.set_margin_top(5);
            row.set_margin_bottom(5);

            let key_label = Label::new(Some(key));
            key_label.set_css_classes(&["info-key"]);
            key_label.set_halign(gtk4::Align::Start);
            key_label.set_hexpand(true);

            let value_label = Label::new(Some(value));
            value_label.set_css_classes(&["info-value"]);
            value_label.set_halign(gtk4::Align::End);
            value_label.set_wrap(true);
            value_label.set_max_width_chars(40);

            row.append(&key_label);
            row.append(&value_label);
            inner.append(&row);
        }

        card.append(&inner);
        frame.set_child(Some(&card));
        frame
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }
}

fn whoami_host() -> String {
    std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("HOST"))
        .unwrap_or_else(|_| {
            std::fs::read_to_string("/etc/hostname")
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|_| "unknown".to_string())
        })
}

fn get_kernel() -> String {
    std::fs::read_to_string("/proc/sys/kernel/osrelease")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}

fn read_os_release(key: &str) -> Option<String> {
    let content = std::fs::read_to_string("/etc/os-release").ok()?;
    for line in content.lines() {
        if line.starts_with(key) {
            let val = line.split('=').nth(1)?;
            return Some(val.trim_matches('"').to_string());
        }
    }
    None
}

fn get_mem_info(key: &str) -> u64 {
    let content = match std::fs::read_to_string("/proc/meminfo") {
        Ok(c) => c,
        Err(_) => return 0,
    };
    for line in content.lines() {
        if line.starts_with(key) {
            if let Some(val) = line.split_whitespace().nth(1) {
                return val.parse::<u64>().unwrap_or(0) * 1024;
            }
        }
    }
    0
}

fn num_cpus() -> usize {
    std::thread::available_parallelism().map(|n| n.get()).unwrap_or(0)
}

fn get_cpu_model() -> String {
    std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("model name"))
                .and_then(|l| l.split(':').nth(1))
                .map(|s| s.trim().to_string())
        })
        .unwrap_or_else(|| "Unknown".to_string())
}

fn format_memory(bytes: u64) -> String {
    if bytes > 1024 * 1024 * 1024 {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes > 1024 * 1024 {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes > 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

fn statvfs_size(path: &Path) -> (u64, u64) {
    #[cfg(target_os = "linux")]
    {
        use std::ffi::CString;
        use std::mem::MaybeUninit;
        let c_path = match CString::new(path.to_str().unwrap_or("/")) {
            Ok(p) => p,
            Err(_) => return (0, 0),
        };
        unsafe {
            let mut stat: MaybeUninit<libc::statvfs> = MaybeUninit::uninit();
            if libc::statvfs(c_path.as_ptr(), stat.as_mut_ptr()) == 0 {
                let s = stat.assume_init();
                let total = s.f_blocks * s.f_frsize;
                let free = s.f_bfree * s.f_frsize;
                return (total, free);
            }
        }
        (0, 0)
    }
    #[cfg(not(target_os = "linux"))]
    {
        (0, 0)
    }
}

fn list_mounts() -> Vec<String> {
    let mut mounts = vec!["/".to_string()];
    if let Ok(content) = std::fs::read_to_string("/proc/mounts") {
        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let mount_point = parts[1];
                let fs_type = parts.get(2).unwrap_or(&"");
                if mount_point.starts_with("/")
                    && mount_point.len() > 1
                    && !mount_point.starts_with("/dev")
                    && !mount_point.starts_with("/sys")
                    && !mount_point.starts_with("/proc")
                    && !mount_point.starts_with("/run")
                    && !mount_point.starts_with("/tmp")
                    && !fs_type.contains("overlay")
                    && !mount_point.contains("docker")
                {
                    let mp = mount_point.to_string();
                    if !mounts.contains(&mp) {
                        mounts.push(mp);
                    }
                }
            }
        }
    }
    mounts
}
