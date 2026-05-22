use gtk4::prelude::*;
use gtk4::{self, Box, Label, Frame};
use std::path::Path;

use crate::cleaner::cache::CacheAnalyzer;

pub struct InfoPage {
    pub container: Box,
}

impl InfoPage {
    pub fn new() -> Self {
        let container = Box::new(gtk4::Orientation::Vertical, 0);

        let header = Box::new(gtk4::Orientation::Vertical, 2);
        header.set_css_classes(&["page-header"]);

        let title = Label::new(Some("System Information"));
        title.set_css_classes(&["page-title"]);
        header.append(&title);
        container.append(&header);

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

        let cache_size = CacheAnalyzer::get_user_cache_size();
        let apt_cache = CacheAnalyzer::get_apt_cache_size();
        content.append(&Self::section("CACHES", &[
            ("User Cache", &format_memory(cache_size)),
            ("APT Cache", &format_memory(apt_cache)),
        ]));

        let app_dir = dirs::data_dir()
            .unwrap_or_else(|| Path::new("/tmp").to_path_buf())
            .join("app-pro");
        let app_size = if app_dir.exists() {
            crate::core::SystemExec::get_size(&app_dir)
        } else {
            0
        };

        content.append(&Self::section("APP PRO", &[
            ("Version", "1.0.0"),
            ("Data Size", &format_memory(app_size)),
        ]));

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
