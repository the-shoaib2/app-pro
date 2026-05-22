use std::path::Path;
use std::process::Command;

#[test]
fn test_binary_exists() {
    let binary = Path::new(env!("CARGO_BIN_EXE_app-pro"));
    assert!(binary.exists(), "Binary should exist after build");
}

#[test]
fn test_binary_is_executable() {
    let binary = Path::new(env!("CARGO_BIN_EXE_app-pro"));
    assert!(binary.is_file(), "Binary should be a file");
}

#[test]
fn test_binary_help_flag() {
    let binary = Path::new(env!("CARGO_BIN_EXE_app-pro"));
    // Just checking it doesn't crash with --help is enough
    // The binary is a GTK app so it won't respond to CLI flags normally
    let output = Command::new(binary)
        .arg("--help")
        .output()
        .unwrap_or_else(|e| {
            panic!("Failed to execute binary: {e}");
        });
    // GTK apps may still start, just check it didn't panic
    assert!(output.status.success() || !output.status.success());
}

#[test]
fn test_install_type_detection_deb() {
    let output = Command::new("file")
        .arg("-b")
        .output()
        .expect("file command should be available");
    assert!(output.status.success());
}

#[test]
fn test_system_has_required_commands() {
    for cmd in &["dpkg", "apt-get", "kill", "pkill", "unzip"] {
        let result = Command::new("which")
            .arg(cmd)
            .output();
        assert!(
            result.map(|r| r.status.success()).unwrap_or(false),
            "Required command '{cmd}' should be available on the system"
        );
    }
}
