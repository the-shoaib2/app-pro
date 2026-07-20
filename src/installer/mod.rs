pub mod deb;
pub mod appimage;
pub mod zip;
pub mod tar;

use std::path::Path;

#[derive(Debug, Clone)]
pub struct InstallResult {
    pub success: bool,
    pub message: String,
    pub app_name: String,
    pub install_path: String,
    pub icon_path: Option<String>,
    pub version: Option<String>,
    pub size_bytes: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InstallType {
    Deb,
    AppImage,
    Zip,
    TarGz,
    Unknown,
}

impl InstallType {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
        let name = path.as_ref().file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
            return InstallType::TarGz;
        }
        match path.as_ref().extension().and_then(|e| e.to_str()) {
            Some("deb") => InstallType::Deb,
            Some("AppImage") | Some("appimage") => InstallType::AppImage,
            Some("zip") => InstallType::Zip,
            _ => {
                if name.contains(".AppImage") || name.ends_with("AppImage") {
                    InstallType::AppImage
                } else {
                    InstallType::Unknown
                }
            }
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            InstallType::Deb => "deb",
            InstallType::AppImage => "appimage",
            InstallType::Zip => "zip",
            InstallType::TarGz => "targz",
            InstallType::Unknown => "unknown",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_install_type_deb() {
        assert_eq!(InstallType::from_path(Path::new("foo.deb")), InstallType::Deb);
        assert_eq!(InstallType::from_path(Path::new("/path/to/package.deb")), InstallType::Deb);
    }

    #[test]
    fn test_install_type_appimage() {
        assert_eq!(InstallType::from_path(Path::new("App.AppImage")), InstallType::AppImage);
        assert_eq!(InstallType::from_path(Path::new("app.appimage")), InstallType::AppImage);
        assert_eq!(InstallType::from_path(Path::new("MyApp-x86_64.AppImage")), InstallType::AppImage);
    }

    #[test]
    fn test_install_type_zip() {
        assert_eq!(InstallType::from_path(Path::new("archive.zip")), InstallType::Zip);
        assert_eq!(InstallType::from_path(Path::new("/tmp/data.zip")), InstallType::Zip);
    }

    #[test]
    fn test_install_type_targz() {
        assert_eq!(InstallType::from_path(Path::new("archive.tar.gz")), InstallType::TarGz);
        assert_eq!(InstallType::from_path(Path::new("app-1.0.tar.gz")), InstallType::TarGz);
        assert_eq!(InstallType::from_path(Path::new("bundle.tgz")), InstallType::TarGz);
        assert_eq!(InstallType::from_path(Path::new("/tmp/pkg.tar.gz")), InstallType::TarGz);
    }

    #[test]
    fn test_install_type_unknown() {
        assert_eq!(InstallType::from_path(Path::new("readme.txt")), InstallType::Unknown);
        assert_eq!(InstallType::from_path(Path::new("script.sh")), InstallType::Unknown);
        assert_eq!(InstallType::from_path(Path::new("")), InstallType::Unknown);
    }

    #[test]
    fn test_install_type_no_extension_appimage() {
        assert_eq!(InstallType::from_path(Path::new("editor.AppImage")), InstallType::AppImage);
        assert_eq!(InstallType::from_path(Path::new("some.AppImage")), InstallType::AppImage);
    }

    #[test]
    fn test_as_str() {
        assert_eq!(InstallType::Deb.as_str(), "deb");
        assert_eq!(InstallType::AppImage.as_str(), "appimage");
        assert_eq!(InstallType::Zip.as_str(), "zip");
        assert_eq!(InstallType::TarGz.as_str(), "targz");
        assert_eq!(InstallType::Unknown.as_str(), "unknown");
    }

    #[test]
    fn test_install_result_defaults() {
        let r = InstallResult {
            success: true,
            message: "ok".into(),
            app_name: "test".into(),
            install_path: "/tmp".into(),
            icon_path: None,
            version: None,
            size_bytes: 100,
        };
        assert!(r.success);
        assert_eq!(r.app_name, "test");
        assert_eq!(r.size_bytes, 100);
    }
}
