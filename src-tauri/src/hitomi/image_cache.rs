use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;

static WRITE_SEQ: AtomicU64 = AtomicU64::new(0);

fn key(hash: &str, thumb: bool) -> String {
    format!("{hash}-{}", if thumb { "t" } else { "f" })
}

fn paths(dir: &Path, hash: &str, thumb: bool) -> (PathBuf, PathBuf) {
    let shard = &hash[..hash.len().min(2)];
    let base = dir.join(shard);
    let k = key(hash, thumb);
    let data = base.join(&k);
    let meta = base.join(format!("{k}.ct"));
    (data, meta)
}

pub fn read(dir: &Path, hash: &str, thumb: bool) -> Option<(Vec<u8>, String)> {
    let (data, meta) = paths(dir, hash, thumb);
    let bytes = fs::read(&data).ok()?;
    if bytes.is_empty() {
        return None;
    }
    let content_type = fs::read_to_string(&meta).unwrap_or_else(|_| "image/webp".to_string());
    Some((bytes, content_type))
}

pub fn write(dir: &Path, hash: &str, thumb: bool, bytes: &[u8], content_type: &str) {
    let (data, meta) = paths(dir, hash, thumb);
    if let Some(parent) = data.parent() {
        if fs::create_dir_all(parent).is_err() {
            return;
        }
    }
    if write_atomic(&data, bytes).is_err() {
        return;
    }
    let _ = write_atomic(&meta, content_type.as_bytes());
}

fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let seq = WRITE_SEQ.fetch_add(1, Ordering::Relaxed);
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(format!(".tmp-{}-{}", std::process::id(), seq));
    let tmp = PathBuf::from(tmp);
    fs::write(&tmp, bytes)?;
    if let Err(e) = fs::rename(&tmp, path) {
        let _ = fs::remove_file(&tmp);
        return Err(e);
    }
    Ok(())
}

pub fn clear(dir: &Path) {
    let _ = fs::remove_dir_all(dir);
}

pub fn size(dir: &Path) -> u64 {
    fn walk(p: &Path) -> u64 {
        let mut total = 0;
        if let Ok(entries) = fs::read_dir(p) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    total += walk(&path);
                } else if let Ok(meta) = entry.metadata() {
                    total += meta.len();
                }
            }
        }
        total
    }
    walk(dir)
}

pub fn enforce_limit(dir: &Path, max_bytes: u64) {
    if max_bytes == 0 {
        return;
    }

    let mut files: Vec<(PathBuf, u64, SystemTime)> = Vec::new();
    let mut total: u64 = 0;
    fn collect(p: &Path, out: &mut Vec<(PathBuf, u64, SystemTime)>, total: &mut u64) {
        if let Ok(entries) = fs::read_dir(p) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    collect(&path, out, total);
                } else if let Ok(meta) = entry.metadata() {
                    let mtime = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                    *total += meta.len();
                    out.push((path, meta.len(), mtime));
                }
            }
        }
    }
    collect(dir, &mut files, &mut total);

    if total <= max_bytes {
        return;
    }

    files.sort_by_key(|(_, _, mtime)| *mtime);
    let target = max_bytes / 10 * 9;
    for (path, sz, _) in files {
        if total <= target {
            break;
        }
        if fs::remove_file(&path).is_ok() {
            total = total.saturating_sub(sz);
        }
    }
}

pub fn ext_from_content_type(ct: &str) -> &'static str {
    match ct.split(';').next().unwrap_or("").trim() {
        "image/webp" => "webp",
        "image/avif" => "avif",
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/gif" => "gif",
        "image/jxl" => "jxl",
        _ => "img",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_dir(tag: &str) -> PathBuf {
        let seq = WRITE_SEQ.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("vista-cache-test-{}-{}-{}", std::process::id(), tag, seq))
    }

    const HASH_A: &str = "0123456789abcdef0123456789abcdef01234567";
    const HASH_B: &str = "fedcba9876543210fedcba9876543210fedcba98";

    #[test]
    fn write_read_roundtrip() {
        let dir = tmp_dir("roundtrip");
        write(&dir, HASH_A, false, b"hello", "image/avif");
        let (bytes, ct) = read(&dir, HASH_A, false).expect("cache hit expected");
        assert_eq!(bytes, b"hello");
        assert_eq!(ct, "image/avif");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn thumb_and_full_are_separate_keys() {
        let dir = tmp_dir("variants");
        write(&dir, HASH_A, false, b"full", "image/webp");
        assert!(read(&dir, HASH_A, true).is_none());
        write(&dir, HASH_A, true, b"thumb", "image/webp");
        assert_eq!(read(&dir, HASH_A, true).unwrap().0, b"thumb");
        assert_eq!(read(&dir, HASH_A, false).unwrap().0, b"full");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_read_is_none() {
        let dir = tmp_dir("missing");
        assert!(read(&dir, HASH_A, false).is_none());
    }

    #[test]
    fn write_leaves_no_temp_files() {
        let dir = tmp_dir("notmp");
        write(&dir, HASH_A, false, b"payload", "image/webp");
        let shard = dir.join(&HASH_A[..2]);
        let leftover = fs::read_dir(&shard)
            .unwrap()
            .flatten()
            .any(|e| e.file_name().to_string_lossy().contains(".tmp-"));
        assert!(!leftover, "temp files must be renamed away");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn enforce_limit_reduces_total_under_limit() {
        let dir = tmp_dir("limit");
        let blob = vec![0u8; 1000];
        for i in 0..6 {
            let hash = format!("{HASH_A}{i:02x}");
            write(&dir, &hash, false, &blob, "image/webp");
        }
        assert!(size(&dir) > 3000);
        enforce_limit(&dir, 3000);
        assert!(size(&dir) <= 3000, "enforce_limit should bring total under the cap");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn ext_from_content_type_maps_known_and_unknown() {
        assert_eq!(ext_from_content_type("image/webp"), "webp");
        assert_eq!(ext_from_content_type("image/avif; charset=binary"), "avif");
        assert_eq!(ext_from_content_type("image/jpeg"), "jpg");
        assert_eq!(ext_from_content_type("application/octet-stream"), "img");
    }

    #[test]
    fn clear_removes_everything() {
        let dir = tmp_dir("clear");
        write(&dir, HASH_B, false, b"x", "image/webp");
        assert!(size(&dir) > 0);
        clear(&dir);
        assert_eq!(size(&dir), 0);
    }
}
