use crate::core::SystemExec;

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    #[allow(dead_code)]
    pub cpu_percent: f32,
    pub memory_bytes: u64,
    #[allow(dead_code)]
    pub state: String,
    pub user: String,
    pub ports: Vec<u16>,
}

pub struct ProcessManager;

impl ProcessManager {
    pub fn list_processes() -> Vec<ProcessInfo> {
        let inode_map = Self::get_inode_to_port_map();
        let mut processes = Vec::new();

        // Read /proc for process information
        if let Ok(entries) = std::fs::read_dir("/proc") {
            for entry in entries.flatten() {
                let path = entry.path();
                let pid_str = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();

                // Only process numeric directories (PIDs)
                let pid: u32 = match pid_str.parse() {
                    Ok(p) => p,
                    Err(_) => continue,
                };

                let proc = Self::read_process_info(pid, &inode_map);
                if let Some(info) = proc {
                    processes.push(info);
                }
            }
        }

        // Sort by PID
        processes.sort_by_key(|p| p.pid);
        processes
    }

    pub fn list_user_processes() -> Vec<ProcessInfo> {
        let all = Self::list_processes();
        let current_user = whoami();
        all.into_iter()
            .filter(|p| p.user == current_user)
            .collect()
    }

    pub fn kill_process(pid: u32, force: bool) -> Result<String, String> {
        if force {
            let sig = if cfg!(target_os = "linux") { 9 } else { 15 };
            let result = SystemExec::run("kill", &[format!("-{}", sig), pid.to_string()]);
            match result {
                Ok(r) if r.success => Ok(format!("Process {} terminated (SIGKILL)", pid)),
                Ok(r) => Err(format!("Failed to kill process {}: {}", pid, r.stderr)),
                Err(e) => Err(format!("Failed to execute kill: {}", e)),
            }
        } else {
            let result = SystemExec::run("kill", &[pid.to_string()]);
            match result {
                Ok(r) if r.success => Ok(format!("Process {} terminated (SIGTERM)", pid)),
                Ok(r) => Err(format!("Failed to kill process {}: {}", pid, r.stderr)),
                Err(e) => Err(format!("Failed to execute kill: {}", e)),
            }
        }
    }

    #[allow(dead_code)]
    pub fn kill_process_by_name(name: &str, force: bool) -> Result<String, String> {
        let sig = if force { "-9" } else { "-15" };
        let result = SystemExec::run("pkill", [sig, name]);
        match result {
            Ok(r) if r.success => Ok(format!("Process '{}' terminated", name)),
            Ok(r) => Err(format!("Failed to kill '{}': {}", name, r.stderr)),
            Err(e) => Err(format!("Failed to execute pkill: {}", e)),
        }
    }

    fn get_inode_to_port_map() -> std::collections::HashMap<u64, Vec<u16>> {
        let mut map: std::collections::HashMap<u64, Vec<u16>> = std::collections::HashMap::new();
        let files = ["/proc/net/tcp", "/proc/net/tcp6", "/proc/net/udp", "/proc/net/udp6"];
        
        for file_path in &files {
            if let Ok(content) = std::fs::read_to_string(file_path) {
                for line in content.lines().skip(1) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 10 {
                        let local_addr = parts[1];
                        let state = parts[3];
                        let inode_str = parts[9];
                        
                        // For TCP, we only want listening connections (state "0A")
                        let is_tcp = file_path.contains("tcp");
                        if is_tcp && state != "0A" {
                            continue;
                        }
                        
                        if let Some(colon_pos) = local_addr.find(':') {
                            let port_hex = &local_addr[colon_pos + 1..];
                            if let Ok(port) = u16::from_str_radix(port_hex, 16) {
                                if let Ok(inode) = inode_str.parse::<u64>() {
                                    if inode > 0 {
                                        map.entry(inode).or_default().push(port);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        map
    }

    fn get_process_ports(pid: u32, inode_map: &std::collections::HashMap<u64, Vec<u16>>) -> Vec<u16> {
        let mut ports = Vec::new();
        let fd_dir = format!("/proc/{}/fd", pid);
        if let Ok(entries) = std::fs::read_dir(fd_dir) {
            for entry in entries.flatten() {
                if let Ok(target) = std::fs::read_link(entry.path()) {
                    let target_str = target.to_string_lossy();
                    if target_str.starts_with("socket:[") && target_str.ends_with(']') {
                        let inode_str = &target_str[8..target_str.len() - 1];
                        if let Ok(inode) = inode_str.parse::<u64>() {
                            if let Some(p_list) = inode_map.get(&inode) {
                                for &port in p_list {
                                    if !ports.contains(&port) {
                                        ports.push(port);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        ports.sort();
        ports
    }

    fn read_process_info(pid: u32, inode_map: &std::collections::HashMap<u64, Vec<u16>>) -> Option<ProcessInfo> {
        let proc_dir = std::path::PathBuf::from("/proc").join(pid.to_string());

        // Read /proc/[pid]/stat
        let stat_path = proc_dir.join("stat");
        let stat_content = std::fs::read_to_string(&stat_path).ok()?;

        // Parse stat (format is complex, we extract name and state)
        let fields: Vec<&str> = stat_content.split_whitespace().collect();
        if fields.len() < 3 {
            return None;
        }

        // Extract process name from parentheses in stat
        let name_start = stat_content.find('(')?;
        let name_end = stat_content.rfind(')')?;
        let name = stat_content[name_start + 1..name_end].to_string();
        let state = fields.get(2).unwrap_or(&"?").to_string();

        // Parse memory from /proc/[pid]/status
        let status_path = proc_dir.join("status");
        let memory_bytes = if let Ok(status_content) = std::fs::read_to_string(&status_path) {
            parse_memory_from_status(&status_content)
        } else {
            0
        };

        // Get user
        let uid_path = proc_dir.join("loginuid");
        let user = if let Ok(uid_str) = std::fs::read_to_string(&uid_path) {
            let uid: u32 = uid_str.trim().parse().unwrap_or(0);
            uid_to_username(uid)
        } else {
            "unknown".to_string()
        };

        // CPU usage (simplified - we read /proc/stat for total time)
        let cpu_percent = estimate_cpu_usage(pid);
        
        let ports = Self::get_process_ports(pid, inode_map);

        Some(ProcessInfo {
            pid,
            name,
            cpu_percent,
            memory_bytes,
            state,
            user,
            ports,
        })
    }
}

fn parse_memory_from_status(status: &str) -> u64 {
    for line in status.lines() {
        if line.starts_with("VmRSS:") {
            // Format: "VmRSS:    12345 kB"
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                if let Ok(kb) = parts[1].parse::<u64>() {
                    return kb * 1024; // Convert to bytes
                }
            }
        }
    }
    0
}

fn uid_to_username(uid: u32) -> String {
    // Try reading from /etc/passwd
    if let Ok(content) = std::fs::read_to_string("/etc/passwd") {
        for line in content.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 3 {
                if let Ok(u) = parts[2].parse::<u32>() {
                    if u == uid {
                        return parts[0].to_string();
                    }
                }
            }
        }
    }
    format!("uid:{}", uid)
}

fn estimate_cpu_usage(_pid: u32) -> f32 {
    // Simple estimation: read /proc/[pid]/stat for utime+stime
    // and compare with system uptime
    // For simplicity, return 0.0 and let the UI refresh
    0.0
}

fn whoami() -> String {
    if let Ok(content) = std::fs::read_to_string("/proc/self/loginuid") {
        if let Ok(uid) = content.trim().parse::<u32>() {
            return uid_to_username(uid);
        }
    }
    std::env::var("USER").unwrap_or_else(|_| "unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_processes_returns_at_least_one() {
        let procs = ProcessManager::list_processes();
        assert!(!procs.is_empty(), "Should list at least the current process");
    }

    #[test]
    fn test_current_process_exists() {
        let procs = ProcessManager::list_processes();
        let current_pid = std::process::id();
        let found = procs.iter().any(|p| p.pid == current_pid);
        assert!(found, "Current process {current_pid} should be in the list");
    }

    #[test]
    fn test_process_info_fields() {
        let procs = ProcessManager::list_processes();
        let proc = procs.iter().find(|p| p.name.contains("test")).unwrap_or(&procs[0]);
        assert!(proc.pid > 0);
        assert!(!proc.name.is_empty());
        assert!(!proc.state.is_empty());
    }

    #[test]
    fn test_user_processes_subset() {
        let all = ProcessManager::list_processes();
        let user = ProcessManager::list_user_processes();
        assert!(user.len() <= all.len());
    }

    #[test]
    fn test_kill_nonexistent_pid() {
        let result = ProcessManager::kill_process(999999, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_uid_to_username_root() {
        let name = uid_to_username(0);
        assert_eq!(name, "root");
    }

    #[test]
    fn test_uid_to_username_nobody() {
        let name = uid_to_username(65534);
        assert_eq!(name, "nobody");
    }

    #[test]
    fn test_process_info_memory() {
        let procs = ProcessManager::list_processes();
        for p in &procs {
            // Memory should be a sensible value (0 is ok for kernel threads)
            assert!(p.memory_bytes < 1024u64.pow(4), "Memory should be less than 1TB");
        }
    }
}
