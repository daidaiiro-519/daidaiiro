//! port の Files の実装。本物のファイルシステムを使う。

use skills_creator_core::ports::Files;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

pub struct RealFiles;

fn said(path: &Path, error: std::io::Error) -> String {
    format!("{}：{error}", path.display())
}

impl Files for RealFiles {
    fn read(&self, path: &Path) -> Result<String, String> {
        fs::read_to_string(path).map_err(|error| said(path, error))
    }
    fn read_bytes(&self, path: &Path) -> Result<Vec<u8>, String> {
        fs::read(path).map_err(|error| said(path, error))
    }
    fn write(&self, path: &Path, text: &str) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| said(parent, error))?;
        }
        fs::write(path, text).map_err(|error| said(path, error))
    }
    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }
    fn list(&self, dir: &Path) -> Vec<PathBuf> {
        let mut found: Vec<PathBuf> = fs::read_dir(dir)
            .map(|entries| entries.flatten().map(|entry| entry.path()).collect())
            .unwrap_or_default();
        found.sort();
        found
    }
    fn create_dir(&self, dir: &Path) -> Result<(), String> {
        fs::create_dir_all(dir).map_err(|error| said(dir, error))
    }
    fn copy(&self, from: &Path, to: &Path) -> Result<(), String> {
        fs::copy(from, to)
            .map(|_| ())
            .map_err(|error| said(to, error))
    }
    fn rename(&self, from: &Path, to: &Path) -> Result<(), String> {
        fs::rename(from, to).map_err(|error| said(to, error))
    }
    fn remove_dir(&self, dir: &Path) -> Result<(), String> {
        fs::remove_dir_all(dir).map_err(|error| said(dir, error))
    }
    fn remove_file(&self, path: &Path) -> Result<(), String> {
        fs::remove_file(path).map_err(|error| said(path, error))
    }
    #[cfg(unix)]
    fn set_executable(&self, path: &Path) -> Result<(), String> {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o755))
            .map_err(|error| said(path, error))
    }
    #[cfg(not(unix))]
    fn set_executable(&self, _path: &Path) -> Result<(), String> {
        Ok(())
    }
    fn temp_dir(&self) -> Result<PathBuf, String> {
        let dir = std::env::temp_dir().join(format!(
            "skills-creator-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).map_err(|error| said(&dir, error))?;
        Ok(dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn written_text_is_read_back_and_listed() {
        let files = RealFiles;
        let dir = files.temp_dir().unwrap();
        files.write(&dir.join("a/b.txt"), "中身").unwrap();
        assert_eq!(files.read(&dir.join("a/b.txt")), Ok("中身".to_owned()));
        assert_eq!(files.list(&dir.join("a")), vec![dir.join("a/b.txt")]);
        files.remove_dir(&dir).unwrap();
        assert!(!files.exists(&dir));
    }
}
