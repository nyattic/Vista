use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

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
    let _ = fs::write(&data, bytes);
    let _ = fs::write(&meta, content_type);
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
