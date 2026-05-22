use gtk4::prelude::*;
use gtk4::{self, Box, Label, Frame};
use std::path::Path;

use crate::cleaner::cache::CacheAnalyzer;

pub struct InfoPage {
    pub container: Box,
}

impl InfoPage {
    pub fn new() -> Self {
        let container = Box::new(gtk4::Orientation::Vertical, 12);

        let title = Label::new(Some("System Information"));
        title.set_css_classes(&["page-title"]);
        container.append(&title);

        let scrolled = gtk4::ScrolledWindow::new();
        scrolled.set_vexpand(true);
        scrolled.set_hexpand(true);

        let content = Box::new(gtk4::Orientation::Vertical, 12);
        content.set_margin_start(24);
        content.set_margin_end(24);
        content.set_margin_top(12);
        content.set_margin_bottom(12);

        // System Info
        content.append(&Self::create_info_section("System", &[
            ("OS", std::env::consts::OS),
            ("Arch", std::env::consts::ARCH),
            ("Host", &whoami_host()),
            ("Kernel", &read_os_release("kernel-version").unwrap_or_else(|| get_kernel())),
            ("Desktop", &std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_else(|_| "Unknown".to_string())),
        ]));

        // Hardware
        let mem_total = get_mem_info("MemTotal");
        let mem_avail = get_mem_info("MemAvailable");
        let mem_used = mem_total.saturating_sub(mem_avail);
        let mem_pct = if mem_total > 0 { (mem_used as f64 / mem_total as f64 * 100.0) as u64 } else { 0 };

        content.append(&Self::create_info_section("Memory", &[
            ("Total", &format_memory(mem_total)),
            ("Used", &format_memory(mem_used)),
            ("Available", &format_memory(mem_avail)),
            ("Usage", &format!("{}%", mem_pct)),
        ]));

        // CPU info
        let cpu_count = num_cpus();
        let cpu_model = get_cpu_model();
        content.append(&Self::create_info_section("CPU", &[
            ("Model", &cpu_model),
            ("Cores", &cpu_count.to_string()),
        ]));

        // Storage - root
        let root_path = Path::new("/");
        let total_space = fs_total_bytes(root_path);
        let free_space = fs_free_bytes(root_path);
        let used_space = total_space.saturating_sub(free_space);
        let storage_pct = if total_space > 0 { (used_space as f64 / total_space as f64 * 100.0) as u64 } else { 0 };

        content.append(&Self::create_info_section("Storage (/)", &[
            ("Total", &format_memory(total_space)),
            ("Used", &format_memory(used_space)),
            ("Free", &format_memory(free_space)),
            ("Usage", &format!("{}%", storage_pct)),
        ]));

        // Cache info
        let cache_size = CacheAnalyzer::get_user_cache_size();
        let apt_cache = CacheAnalyzer::get_apt_cache_size();
        content.append(&Self::create_info_section("Cache Sizes", &[
            ("~/.cache", &format_memory(cache_size)),
            ("APT Cache", &format_memory(apt_cache)),
        ]));

        // App Pro info
        let app_dir = dirs::data_dir()
            .unwrap_or_else(|| Path::new("/tmp").to_path_buf())
            .join("app-pro");
        let app_size = if app_dir.exists() {
            crate::core::SystemExec::get_size(&app_dir)
        } else {
            0
        };

        content.append(&Self::create_info_section("App Pro", &[
            ("Version", "1.0.0"),
            ("Data Directory", &app_dir.to_string_lossy()),
            ("Data Size", &format_memory(app_size)),
            ("Database", &app_dir.join("app_pro.db").to_string_lossy()),
        ]));

        scrolled.set_child(Some(&content));
        container.append(&scrolled);

        InfoPage { container }
    }

    fn create_info_section(title: &str, entries: &[(&str, &str)]) -> Frame {
        let frame = Frame::new(Some(title));
        frame.set_css_classes(&["info-frame"]);

        let vbox = Box::new(gtk4::Orientation::Vertical, 4);
        vbox.set_margin_start(12);
        vbox.set_margin_end(12);
        vbox.set_margin_top(8);
        vbox.set_margin_bottom(8);

        for (key, value) in entries {
            let row = Box::new(gtk4::Orientation::Horizontal, 8);

            let key_label = Label::new(Some(key));
            key_label.set_css_classes(&["info-key"]);
            key_label.set_halign(gtk4::Align::Start);
            key_label.set_width_request(120);

            let value_label = Label::new(Some(value));
            value_label.set_css_classes(&["info-value"]);
            value_label.set_halign(gtk4::Align::Start);
            value_label.set_wrap(true);
            value_label.set_hexpand(true);

            row.append(&key_label);
            row.append(&value_label);
            vbox.append(&row);
        }

        frame.set_child(Some(&vbox));
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

fn fs_total_bytes(path: &Path) -> u64 {
    #[cfg(target_os = "linux")]
    {
        use std::mem::MaybeUninit;
        unsafe {
            let mut stat: MaybeUninit<libc::statvfs> = MaybeUninit::uninit();
            if libc::statvfs(
                path.to_str().unwrap_or("/").as_ptr() as *const libc::c_char,
                stat.as_mut_ptr(),
            ) == 0
            {
                let s = stat.assume_init();
                return s.f_blocks * s.f_bsize as u64;
            }
        }
        0
    }
    #[cfg(not(target_os = "linux"))]
    {
        0
    }
}

fn fs_free_bytes(path: &Path) -> u64 {
    #[cfg(target_os = "linux")]
    {
        use std::mem::MaybeUninit;
        unsafe {
            let mut stat: MaybeUninit<libc::statvfs> = MaybeUninit::uninit();
            if libc::statvfs(
                path.to_str().unwrap_or("/").as_ptr() as *const libc::c_char,
                stat.as_mut_ptr(),
            ) == 0
            {
                let s = stat.assume_init();
                return s.f_bfree * s.f_bsize as u64;
            }
        }
        0
    }
    #[cfg(not(target_os = "linux"))]
    {
        0
    }
}
