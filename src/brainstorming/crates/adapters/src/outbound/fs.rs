//! port の Files の実装。本物のファイルシステムを使う。

use brainstorming_core::ports::Files;
use std::fs;
use std::path::{Path, PathBuf};

pub struct RealFiles;

fn said(path: &Path, error: std::io::Error) -> String {
    format!("{}：{error}", path.display())
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut paths: Vec<PathBuf> = entries.flatten().map(|entry| entry.path()).collect();
    paths.sort();
    for path in paths {
        if path.is_dir() {
            walk(&path, out);
        } else {
            out.push(path);
        }
    }
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
    fn write_bytes(&self, path: &Path, bytes: &[u8]) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| said(parent, error))?;
        }
        fs::write(path, bytes).map_err(|error| said(path, error))
    }
    fn create_new(&self, path: &Path, text: &str) -> Result<bool, String> {
        use std::io::Write;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| said(parent, error))?;
        }
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
        {
            Ok(mut file) => file
                .write_all(text.as_bytes())
                .map(|_| true)
                .map_err(|error| said(path, error)),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(false),
            Err(error) => Err(said(path, error)),
        }
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
    fn tree(&self, dir: &Path) -> Vec<PathBuf> {
        let mut out = Vec::new();
        walk(dir, &mut out);
        out
    }
    fn remove_file(&self, path: &Path) -> Result<(), String> {
        fs::remove_file(path).map_err(|error| said(path, error))
    }
    fn remove_dir(&self, dir: &Path) -> Result<(), String> {
        match dir.exists() {
            true => fs::remove_dir_all(dir).map_err(|error| said(dir, error)),
            false => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("brainstorming-fs-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn writing_makes_folders_and_tree_lists_every_file_in_order() {
        let dir = temp("tree");
        RealFiles.write(&dir.join("b/c.txt"), "1").unwrap();
        RealFiles.write(&dir.join("a.txt"), "2").unwrap();
        assert_eq!(
            RealFiles.tree(&dir),
            vec![dir.join("a.txt"), dir.join("b/c.txt")]
        );
        assert_eq!(RealFiles.list(&dir), vec![dir.join("a.txt"), dir.join("b")]);
        RealFiles.remove_dir(&dir).unwrap();
        assert!(!RealFiles.exists(&dir));
        assert!(RealFiles.remove_dir(&dir).is_ok());
    }
}
