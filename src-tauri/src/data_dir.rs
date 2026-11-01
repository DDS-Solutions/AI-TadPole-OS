//! @docs ARCHITECTURE:DesktopShell
//!
//! ### AI Assist Note
//! Resolves writable desktop state before token, database or PID initialization.
//! Platform and environment inputs are explicit so failure paths can be tested.
//!
//! ### 🔍 Debugging & Observability
//! I/O errors propagate to the desktop startup diagnostic; no bundle fallback on macOS.

use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};

fn select_data_dir(
    install_dir: &Path,
    custom: Option<OsString>,
    macos: bool,
    home: Option<OsString>,
) -> io::Result<PathBuf> {
    let path = if let Some(custom) = custom.filter(|value| !value.is_empty()) {
        PathBuf::from(custom)
    } else if macos {
        let home = home.filter(|value| !value.is_empty()).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "HOME is unavailable; set an absolute TADPOLE_DATA_DIR",
            )
        })?;
        PathBuf::from(home).join("Library/Application Support/com.tadpole.os")
    } else {
        install_dir.to_path_buf()
    };
    if !path.is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Desktop data directory must be absolute (check TADPOLE_DATA_DIR or HOME)",
        ));
    }
    Ok(path)
}

pub fn prepare_data_dir(
    install_dir: &Path,
    custom: Option<OsString>,
    macos: bool,
    home: Option<OsString>,
) -> io::Result<PathBuf> {
    let path = select_data_dir(install_dir, custom, macos, home)?;
    let mut builder = std::fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    if macos {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(&path)?;
    #[cfg(unix)]
    if macos {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700))?;
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root() -> PathBuf {
        std::env::temp_dir()
    }

    #[test]
    fn macos_uses_application_support_outside_the_bundle() {
        let home = root().join("user with spaces");
        let install = root().join("Applications/App.app/Contents/MacOS");
        assert_eq!(
            select_data_dir(&install, None, true, Some(home.clone().into_os_string())).unwrap(),
            home.join("Library/Application Support/com.tadpole.os")
        );
    }

    #[test]
    fn missing_or_empty_home_cannot_fall_back_to_bundle() {
        for home in [None, Some(OsString::new())] {
            assert_eq!(
                select_data_dir(&root(), None, true, home)
                    .unwrap_err()
                    .kind(),
                io::ErrorKind::NotFound
            );
        }
    }

    #[test]
    fn explicit_absolute_override_does_not_require_home() {
        let custom = root().join("portable state");
        assert_eq!(
            select_data_dir(&root(), Some(custom.clone().into_os_string()), true, None).unwrap(),
            custom
        );
    }

    #[test]
    fn relative_override_is_rejected_before_sidecar_changes_directory() {
        assert_eq!(
            select_data_dir(&root(), Some("relative-state".into()), true, None)
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidInput
        );
    }

    #[test]
    fn relative_home_is_rejected() {
        assert_eq!(
            select_data_dir(&root(), None, true, Some("relative-home".into()))
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidInput
        );
    }

    #[test]
    fn other_platforms_preserve_portable_installation_default() {
        assert_eq!(
            select_data_dir(&root(), Some(OsString::new()), false, None).unwrap(),
            root()
        );
    }

    #[test]
    fn creates_data_directory_and_propagates_filesystem_errors() {
        let fixture = root().join(format!("tadpole-data-dir-test-{}", std::process::id()));
        std::fs::create_dir(&fixture).unwrap();
        let target = fixture.join("data");
        assert_eq!(
            prepare_data_dir(&fixture, Some(target.clone().into_os_string()), true, None).unwrap(),
            target
        );
        assert!(target.is_dir());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(&target).unwrap().permissions().mode() & 0o777,
                0o700
            );
        }
        let blocked = fixture.join("file");
        std::fs::write(&blocked, b"occupied").unwrap();
        assert!(prepare_data_dir(&fixture, Some(blocked.into_os_string()), true, None).is_err());
        // Only this test's newly created directory is removed.
        std::fs::remove_file(fixture.join("file")).unwrap();
        std::fs::remove_dir(target).unwrap();
        std::fs::remove_dir(fixture).unwrap();
    }
}
