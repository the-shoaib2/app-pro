use std::ffi::OsStr;
use std::path::Path;
use std::process::{Command, Output};
use std::io;

pub struct SystemExec;

#[derive(Debug, Clone)]
pub struct ExecResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

impl SystemExec {
    pub fn run<I, S>(cmd: &str, args: I) -> io::Result<ExecResult>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let output = Command::new(cmd).args(args).output()?;
        Ok(Self::from_output(output))
    }

    pub fn run_with_sudo<I, S>(cmd: &str, args: I) -> io::Result<ExecResult>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let mut full_args: Vec<String> = Vec::new();
        full_args.push(cmd.to_string());
        for a in args {
            full_args.push(a.as_ref().to_string_lossy().to_string());
        }
        let output = Command::new("pkexec").args(&full_args).output()?;
        Ok(Self::from_output(output))
    }

    pub fn run_with_pkexec(script: &str) -> io::Result<ExecResult> {
        let output = Command::new("pkexec")
            .arg("sh")
            .arg("-c")
            .arg(script)
            .output()?;
        Ok(Self::from_output(output))
    }

    pub fn run_script(script: &str) -> io::Result<ExecResult> {
        let output = Command::new("sh").arg("-c").arg(script).output()?;
        Ok(Self::from_output(output))
    }

    fn from_output(output: Output) -> ExecResult {
        ExecResult {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code(),
        }
    }

    pub fn file_exists<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().exists()
    }

    pub fn copy_recursively(src: &Path, dst: &Path) -> io::Result<()> {
        if src.is_dir() {
            if !dst.exists() {
                std::fs::create_dir_all(dst)?;
            }
            for entry in std::fs::read_dir(src)? {
                let entry = entry?;
                let file_type = entry.file_type()?;
                let src_path = entry.path();
                let dst_path = dst.join(entry.file_name());
                if file_type.is_dir() {
                    Self::copy_recursively(&src_path, &dst_path)?;
                } else {
                    std::fs::copy(&src_path, &dst_path)?;
                }
            }
            Ok(())
        } else {
            if let Some(parent) = dst.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(src, dst)?;
            Ok(())
        }
    }

    pub fn remove_dir_contents<P: AsRef<Path>>(path: P) -> io::Result<()> {
        let path = path.as_ref();
        if path.exists() {
            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    std::fs::remove_dir_all(&entry_path)?;
                } else {
                    std::fs::remove_file(&entry_path)?;
                }
            }
        }
        Ok(())
    }

    pub fn get_size<P: AsRef<Path>>(path: P) -> u64 {
        fn dir_size(dir: &Path) -> io::Result<u64> {
            let mut total = 0u64;
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_symlink() {
                    continue;
                }
                if path.is_dir() {
                    total += dir_size(&path)?;
                } else {
                    total += entry.metadata()?.len();
                }
            }
            Ok(total)
        }
        dir_size(path.as_ref()).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_run_echo() {
        let result = SystemExec::run("echo", ["hello"]).unwrap();
        assert!(result.success);
        assert_eq!(result.stdout.trim(), "hello");
    }

    #[test]
    fn test_run_false() {
        let result = SystemExec::run("false", std::iter::empty::<&str>()).unwrap();
        assert!(!result.success);
        assert_eq!(result.exit_code, Some(1));
    }

    #[test]
    fn test_file_exists() {
        assert!(SystemExec::file_exists("/tmp"));
        assert!(!SystemExec::file_exists("/nonexistent_path_xyz"));
    }

    #[test]
    fn test_get_size() {
        let dir = "/tmp/apppro-test-size";
        fs::create_dir_all(dir).unwrap();
        fs::write(format!("{dir}/a.txt"), "hello").unwrap();
        fs::write(format!("{dir}/b.txt"), "world").unwrap();
        let size = SystemExec::get_size(dir);
        assert_eq!(size, 10);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn test_copy_recursively() {
        let src = "/tmp/apppro-test-copy-src";
        let dst = "/tmp/apppro-test-copy-dst";
        fs::create_dir_all(format!("{src}/sub")).unwrap();
        fs::write(format!("{src}/sub/f.txt"), "data").unwrap();
        SystemExec::copy_recursively(Path::new(src), Path::new(dst)).unwrap();
        assert!(SystemExec::file_exists(format!("{dst}/sub/f.txt")));
        assert_eq!(fs::read_to_string(format!("{dst}/sub/f.txt")).unwrap(), "data");
        fs::remove_dir_all(src).unwrap();
        fs::remove_dir_all(dst).unwrap();
    }

    #[test]
    fn test_remove_dir_contents() {
        let dir = "/tmp/apppro-test-remove";
        fs::create_dir_all(dir).unwrap();
        fs::write(format!("{dir}/x.txt"), "").unwrap();
        SystemExec::remove_dir_contents(dir).unwrap();
        assert!(SystemExec::file_exists(dir));
        assert_eq!(fs::read_dir(dir).unwrap().count(), 0);
        fs::remove_dir(dir).unwrap();
    }

    #[test]
    fn test_script_true() {
        let r = SystemExec::run_script("exit 0").unwrap();
        assert!(r.success);
    }

    #[test]
    fn test_script_false() {
        let r = SystemExec::run_script("exit 42").unwrap();
        assert!(!r.success);
        assert_eq!(r.exit_code, Some(42));
    }
}
