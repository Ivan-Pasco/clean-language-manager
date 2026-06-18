use crate::error::{CleenError, Result};
use std::path::Path;

pub fn ensure_dir_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path).map_err(|e| match e.kind() {
            std::io::ErrorKind::PermissionDenied => CleenError::PermissionDenied {
                path: path.to_path_buf(),
            },
            _ => CleenError::from(e),
        })?;
    }
    Ok(())
}

pub fn remove_dir_recursive(path: &Path) -> Result<()> {
    if path.exists() {
        std::fs::remove_dir_all(path).map_err(|e| match e.kind() {
            std::io::ErrorKind::PermissionDenied => CleenError::PermissionDenied {
                path: path.to_path_buf(),
            },
            _ => CleenError::from(e),
        })?;
    }
    Ok(())
}

/// Remove whatever is at `path` — file, directory, or symlink (even a broken one).
///
/// `Path::exists()` follows symlinks, so it reports `false` for a dangling
/// symlink. The caller then skips the remove and the next `symlink()` /
/// `create_dir()` call hits `EEXIST` ("File exists", os error 17). This
/// helper uses `symlink_metadata()` so it sees the link itself and removes it.
pub fn remove_path_if_exists(path: &Path) -> Result<()> {
    let meta = match std::fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e.into()),
    };

    let res = if meta.file_type().is_dir() {
        std::fs::remove_dir_all(path)
    } else {
        std::fs::remove_file(path)
    };
    res.map_err(|e| match e.kind() {
        std::io::ErrorKind::PermissionDenied => CleenError::PermissionDenied {
            path: path.to_path_buf(),
        },
        _ => CleenError::from(e),
    })?;
    Ok(())
}

pub fn copy_file(from: &Path, to: &Path) -> Result<()> {
    if let Some(parent) = to.parent() {
        ensure_dir_exists(parent)?;
    }

    std::fs::copy(from, to)?;
    Ok(())
}

pub fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    ensure_dir_exists(dst)?;

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

pub fn is_executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        path.metadata()
            .map(|m| m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }

    #[cfg(windows)]
    {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("exe"))
            .unwrap_or(false)
    }
}

pub fn make_executable(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(path)?.permissions();
        perms.set_mode(perms.mode() | 0o755);
        std::fs::set_permissions(path, perms)?;
    }

    // On Windows, executable permission is determined by file extension
    #[cfg(windows)]
    {
        let _ = path; // Suppress unused warning
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[cfg(unix)]
    #[test]
    fn remove_path_if_exists_clears_broken_symlink() {
        let tmp =
            std::env::temp_dir().join(format!("cleen-fs-broken-symlink-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        let nonexistent_target = tmp.join("does-not-exist");
        let link = tmp.join("dangling-link");
        std::os::unix::fs::symlink(&nonexistent_target, &link).unwrap();

        // `exists()` follows symlinks and returns false here — this is the
        // bug class the helper exists to handle.
        assert!(!link.exists());
        // But the link itself does exist.
        assert!(link.symlink_metadata().is_ok());

        remove_path_if_exists(&link).unwrap();
        assert!(link.symlink_metadata().is_err());

        // Calling again on a now-missing path is a no-op, not an error.
        remove_path_if_exists(&link).unwrap();

        fs::remove_dir_all(&tmp).unwrap();
    }

    #[test]
    fn remove_path_if_exists_clears_regular_file() {
        let tmp = std::env::temp_dir().join(format!("cleen-fs-file-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        let file = tmp.join("a.txt");
        fs::write(&file, "x").unwrap();
        assert!(file.exists());

        remove_path_if_exists(&file).unwrap();
        assert!(!file.exists());

        fs::remove_dir_all(&tmp).unwrap();
    }

    #[test]
    fn remove_path_if_exists_clears_directory() {
        let tmp = std::env::temp_dir().join(format!("cleen-fs-dir-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        let sub = tmp.join("sub");
        fs::create_dir_all(sub.join("inner")).unwrap();
        fs::write(sub.join("inner").join("f"), "y").unwrap();

        remove_path_if_exists(&sub).unwrap();
        assert!(!sub.exists());

        fs::remove_dir_all(&tmp).unwrap();
    }
}
