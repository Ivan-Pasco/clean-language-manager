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

/// Strip extended attributes from `path` (macOS only, best-effort, no-op elsewhere).
///
/// macOS Sequoia inherits `com.apple.provenance` onto files created by a
/// process whose own binary carries that xattr — typical for `cleen` itself
/// when it was installed via `curl | sh`. The xattr blocks later in-place
/// mutation of those files, which is what breaks `cleen use` when it tries
/// to update the shim.
///
/// Errors are intentionally swallowed: some attributes are kernel-protected
/// and the in-place strip is not the load-bearing fix — [`atomic_write`] is.
/// This just keeps directory listings tidy.
pub fn strip_macos_xattrs(path: &Path) {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("/usr/bin/xattr")
            .arg("-c")
            .arg(path)
            .output();
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = path;
    }
}

/// Recursive variant of [`strip_macos_xattrs`] for use after archive extraction.
pub fn strip_macos_xattrs_recursive(path: &Path) {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("/usr/bin/xattr")
            .arg("-cr")
            .arg(path)
            .output();
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = path;
    }
}

/// Atomically write `contents` to `path` via temp-file + rename.
///
/// The destination's inode is replaced, not mutated in place. Any extended
/// attributes on the old inode (e.g. `com.apple.provenance` that blocks
/// modification) go away with it — the new inode starts fresh. On unix,
/// `unix_mode` is applied to the temp file before the rename so the
/// destination appears atomically with the correct permissions.
pub fn atomic_write(path: &Path, contents: &[u8], unix_mode: Option<u32>) -> Result<()> {
    let parent = path.parent().ok_or_else(|| CleenError::IoError {
        message: format!("path has no parent: {}", path.display()),
    })?;
    ensure_dir_exists(parent)?;
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("file");
    let tmp = parent.join(format!(".{file_name}.cleen-tmp.{}", std::process::id()));
    let _ = std::fs::remove_file(&tmp);

    std::fs::write(&tmp, contents)?;

    #[cfg(unix)]
    {
        if let Some(m) = unix_mode {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(m))?;
        }
    }
    #[cfg(not(unix))]
    {
        let _ = unix_mode;
    }

    strip_macos_xattrs(&tmp);
    std::fs::rename(&tmp, path).inspect_err(|_| {
        let _ = std::fs::remove_file(&tmp);
    })?;
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

    #[test]
    fn atomic_write_replaces_existing_file_and_applies_mode() {
        let tmp = std::env::temp_dir().join(format!("cleen-fs-atomic-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        let target = tmp.join("cln");

        // Pre-existing file we'd normally try to overwrite in place.
        fs::write(&target, b"old").unwrap();
        let old_inode = inode_of(&target);

        atomic_write(&target, b"new", Some(0o755)).unwrap();

        assert_eq!(fs::read(&target).unwrap(), b"new");
        // Inode replacement is the load-bearing property: it's what drops
        // any xattrs the kernel pinned to the old file.
        assert_ne!(inode_of(&target), old_inode);

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(&target).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o755);
        }

        // No leftover temp files in the parent.
        let leftovers: Vec<_> = fs::read_dir(&tmp)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_string_lossy()
                    .starts_with(".cln.cleen-tmp.")
            })
            .collect();
        assert!(leftovers.is_empty(), "found leftover temp files");

        fs::remove_dir_all(&tmp).unwrap();
    }

    #[cfg(unix)]
    fn inode_of(path: &Path) -> u64 {
        use std::os::unix::fs::MetadataExt;
        fs::metadata(path).unwrap().ino()
    }

    #[cfg(not(unix))]
    fn inode_of(_path: &Path) -> u64 {
        0
    }
}
