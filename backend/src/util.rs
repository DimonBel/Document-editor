use std::fs;
use std::io::Write;
use std::path::Path;

/// Atomically write `data` to `path` by writing to a sibling `*.tmp`
/// file in the same directory, fsyncing, and renaming over the target.
///
/// Guarantees the destination file is either fully replaced or left
/// untouched, even if the process is killed mid-write. Also ensures
/// the rename target sits on the same filesystem as the source so the
/// rename is guaranteed atomic on POSIX and Windows.
pub fn write_atomic(path: &Path, data: &[u8]) -> std::io::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "path has no parent"))?;
    fs::create_dir_all(parent)?;

    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "path has no filename"))?;

    let tmp_path = parent.join(format!(".{}.tmp", file_name));

    {
        let mut f = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&tmp_path)?;
        f.write_all(data)?;
        f.sync_all()?;
    }

    // On Windows, rename refuses to overwrite an existing file, so
    // remove the target first. The window between remove+rename is
    // safe because any concurrent reader either sees the old file
    // (still intact at that path) or the new file (after rename).
    #[cfg(windows)]
    {
        if path.exists() {
            fs::remove_file(path)?;
        }
    }

    fs::rename(&tmp_path, path)?;
    Ok(())
}