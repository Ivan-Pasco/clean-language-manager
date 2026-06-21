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
/// Replaces the destination's inode rather than mutating in place. On unix,
/// `unix_mode` is applied to the temp file before the rename so the
/// destination appears atomically with the correct permissions.
///
/// **macOS provenance caveat:** if the destination is a regular file
/// carrying `com.apple.provenance`, the rename itself is rejected with
/// EPERM by the kernel — the lock covers the path, not just in-place
/// mutation. For shim-style files that must overwrite a possibly-locked
/// destination, use [`atomic_replace_symlink`] instead: symlinks (unlike
/// interpreted-script regular files) are not subject to the lock and
/// rename-replace cleanly.
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

/// True if `path` is a regular file carrying `com.apple.provenance`.
///
/// On macOS Sequoia, an interpreted-script regular file with this xattr
/// becomes immutable to user-level operations (rm, chmod, xattr -c,
/// rename-over, even via sudo). Symlinks and Mach-O binaries with the
/// same xattr are NOT affected — only interpreted-script regular files
/// trigger the kernel-level lock. Always false on non-macOS targets.
pub fn has_provenance_lock(path: &Path) -> bool {
    #[cfg(target_os = "macos")]
    {
        let Ok(meta) = std::fs::symlink_metadata(path) else {
            return false;
        };
        if !meta.file_type().is_file() {
            return false;
        }
        std::process::Command::new("/usr/bin/xattr")
            .arg("-p")
            .arg("com.apple.provenance")
            .arg(path)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = path;
        false
    }
}

/// Atomically replace `path` with a symlink pointing at `target`.
///
/// The symlink is created at a sibling temp name and renamed over the
/// destination. Symlinks are not subject to the macOS-Sequoia provenance
/// lock that blocks rename-over on script files, so this is the safe
/// shim primitive: once the path is a symlink, every subsequent `cleen
/// use` can replace it atomically.
#[cfg(unix)]
pub fn atomic_replace_symlink(path: &Path, target: &Path) -> Result<()> {
    let parent = path.parent().ok_or_else(|| CleenError::IoError {
        message: format!("path has no parent: {}", path.display()),
    })?;
    ensure_dir_exists(parent)?;
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("file");
    let tmp = parent.join(format!(".{file_name}.cleen-tmp.{}", std::process::id()));
    let _ = std::fs::remove_file(&tmp);

    std::os::unix::fs::symlink(target, &tmp)?;
    std::fs::rename(&tmp, path).inspect_err(|_| {
        let _ = std::fs::remove_file(&tmp);
    })?;
    Ok(())
}

/// Evict provenance-locked regular files out of `bin_dir` by shuffling
/// the directory in place. Returns `Ok(true)` if a shuffle ran.
///
/// macOS Sequoia's `com.apple.provenance` xattr on a script-style
/// regular file blocks unlink, chmod, xattr clear, and rename-over —
/// even from sudo. The escape hatch is to rename the *parent* directory:
/// the lock is on the file's identity within its directory, not on the
/// containing directory itself, so `mv bin bin.locked` succeeds and the
/// locked file is left frozen inside the graveyard. After the shuffle a
/// fresh `bin_dir` contains everything that was salvageable, minus the
/// listed names — the caller then recreates those names as symlinks via
/// [`atomic_replace_symlink`].
///
/// No-op on non-macOS targets.
pub fn evict_locked_shims(bin_dir: &Path, names: &[&str]) -> Result<bool> {
    #[cfg(target_os = "macos")]
    {
        let any_locked = names.iter().any(|n| has_provenance_lock(&bin_dir.join(n)));
        if !any_locked {
            return Ok(false);
        }

        let parent = bin_dir.parent().ok_or_else(|| CleenError::IoError {
            message: format!("bin dir has no parent: {}", bin_dir.display()),
        })?;
        let bin_name = bin_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("bin");

        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let staging = parent.join(format!("{bin_name}.new.{}", std::process::id()));
        let graveyard = parent.join(format!("{bin_name}.locked-{ts}"));

        let _ = std::fs::remove_dir_all(&staging);
        ensure_dir_exists(&staging)?;

        // Salvage every child except the explicitly-locked shim names.
        // Other locked files (if any) would fail to copy; let that surface
        // rather than guessing.
        for entry in std::fs::read_dir(bin_dir)? {
            let entry = entry?;
            let src = entry.path();
            let file_name = entry.file_name();
            let name_str = file_name.to_string_lossy().to_string();

            if names.iter().any(|n| *n == name_str) && has_provenance_lock(&src) {
                continue;
            }

            let dst = staging.join(&file_name);
            if src.is_symlink() {
                let target = std::fs::read_link(&src)?;
                std::os::unix::fs::symlink(&target, &dst)?;
            } else if src.is_dir() {
                copy_dir_recursive(&src, &dst)?;
            } else {
                std::fs::copy(&src, &dst)?;
                use std::os::unix::fs::PermissionsExt;
                let meta = std::fs::metadata(&src)?;
                std::fs::set_permissions(
                    &dst,
                    std::fs::Permissions::from_mode(meta.permissions().mode()),
                )?;
            }
        }

        std::fs::rename(bin_dir, &graveyard)?;
        if let Err(e) = std::fs::rename(&staging, bin_dir) {
            let _ = std::fs::rename(&graveyard, bin_dir);
            return Err(e.into());
        }

        eprintln!(
            "  Evicted provenance-locked shim — old directory preserved at {}",
            graveyard.display()
        );
        Ok(true)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (bin_dir, names);
        Ok(false)
    }
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

        #[cfg(unix)]
        {
            // Inode replacement is the load-bearing property: it's what drops
            // any xattrs the kernel pinned to the old file. Only meaningful on
            // unix — Windows has no notion of an inode here.
            assert_ne!(inode_of(&target), old_inode);

            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(&target).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o755);
        }
        #[cfg(not(unix))]
        {
            let _ = old_inode;
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

    #[cfg(unix)]
    #[test]
    fn atomic_replace_symlink_replaces_existing_symlink() {
        let tmp =
            std::env::temp_dir().join(format!("cleen-fs-symlink-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        let target_a = tmp.join("target-a");
        let target_b = tmp.join("target-b");
        fs::write(&target_a, b"a").unwrap();
        fs::write(&target_b, b"b").unwrap();
        let link = tmp.join("cln");

        atomic_replace_symlink(&link, &target_a).unwrap();
        assert!(fs::symlink_metadata(&link).unwrap().file_type().is_symlink());
        assert_eq!(fs::read(&link).unwrap(), b"a");

        // Replacing the existing symlink with one pointing at a different
        // target is the load-bearing operation: `cleen use <other>` should
        // succeed against an existing symlink.
        atomic_replace_symlink(&link, &target_b).unwrap();
        assert!(fs::symlink_metadata(&link).unwrap().file_type().is_symlink());
        assert_eq!(fs::read_link(&link).unwrap(), target_b);
        assert_eq!(fs::read(&link).unwrap(), b"b");

        // No leftover temp files.
        let leftovers: Vec<_> = fs::read_dir(&tmp)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().starts_with(".cln.cleen-tmp."))
            .collect();
        assert!(leftovers.is_empty(), "found leftover temp symlinks");

        fs::remove_dir_all(&tmp).unwrap();
    }
}
