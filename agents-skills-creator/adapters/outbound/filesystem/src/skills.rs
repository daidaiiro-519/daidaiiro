//! ポート SkillFiles ・ Destinations を、作業ディレクトリのファイルとシンボリックリンクで実装する。ポート GeneratedFiles で書くのは
//! creator が作るファイル（SKILL.md）だけで、skill.json は書かない（Semantic Agent Schema が書く）。

use skills_application::ports::{Destinations, GeneratedFiles, SkillFiles};
use std::path::{Path, PathBuf};

/// 作業ディレクトリ（経路の根）。
#[derive(Debug, Clone)]
pub struct WorkingTree {
    root: PathBuf,
}

impl WorkingTree {
    /// root を根にする。
    #[must_use]
    pub const fn new(root: PathBuf) -> Self {
        Self { root }
    }
}

/// 経路の親と最後の部分（/ で区切る）。
fn split(path: &str) -> (&str, &str) {
    match path.rsplit_once('/') {
        Some((parent, last)) => (parent, last),
        None => ("", path),
    }
}

fn join(parent: &str, name: &str) -> String {
    if parent.is_empty() {
        name.to_string()
    } else {
        format!("{parent}/{name}")
    }
}

/// フォルダの中を降りて、先の無いシンボリックリンクを集める（リンクの先のフォルダの中へは降りない）。
fn walk(dir: &Path, rel: &str, out: &mut Vec<String>) -> Result<(), String> {
    let entries = std::fs::read_dir(dir).map_err(|e| format!("{rel} を読めない：{e}"))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("{rel} を読めない：{e}"))?;
        let name = entry.file_name().to_string_lossy().to_string();
        let here = join(rel, &name);
        let kind = entry
            .file_type()
            .map_err(|e| format!("{here} を読めない：{e}"))?;
        if kind.is_symlink() {
            if std::fs::metadata(entry.path()).is_err() {
                out.push(here);
            }
        } else if kind.is_dir() {
            walk(&entry.path(), &here, out)?;
        }
    }
    Ok(())
}

impl SkillFiles for WorkingTree {
    fn exists(&self, path: &str) -> bool {
        self.root.join(path).exists()
    }

    fn read(&self, path: &str) -> Result<String, String> {
        std::fs::read_to_string(self.root.join(path)).map_err(|e| format!("{path} を読めない：{e}"))
    }

    fn siblings(&self, folder: &str) -> Result<Vec<String>, String> {
        let (parent, me) = split(folder);
        let dir = if parent.is_empty() {
            self.root.clone()
        } else {
            self.root.join(parent)
        };
        let entries = std::fs::read_dir(&dir).map_err(|e| {
            format!(
                "{} を読めない：{e}",
                if parent.is_empty() { "." } else { parent }
            )
        })?;
        let mut out: Vec<String> = entries
            .filter_map(Result::ok)
            .map(|e| e.file_name().to_string_lossy().to_string())
            .filter(|n| n != me && dir.join(n).is_dir())
            .map(|n| join(parent, &n))
            .collect();
        out.sort();
        Ok(out)
    }

    fn dangling_links(&self, folder: &str) -> Result<Vec<String>, String> {
        let mut out = Vec::new();
        walk(&self.root.join(folder), folder, &mut out)?;
        out.sort();
        Ok(out)
    }
}

impl GeneratedFiles for WorkingTree {
    fn write(&self, path: &str, text: &str) -> Result<(), String> {
        // 一時ファイルに書いてから置き換える（途中で止まっても、前の中身か新しい中身のどちらかが残る）
        let to = self.root.join(path);
        let (parent, name) = split(path);
        let tmp = self.root.join(join(parent, &format!(".{name}.asc-tmp")));
        std::fs::write(&tmp, text).map_err(|e| format!("{path} を書けない：{e}"))?;
        std::fs::rename(&tmp, &to).map_err(|e| {
            let _ = std::fs::remove_file(&tmp);
            format!("{path} を置き換えられない：{e}")
        })
    }
}

impl Destinations for WorkingTree {
    fn home(&self) -> Option<String> {
        std::env::var_os("HOME")
            .filter(|h| !h.is_empty())
            .map(|h| h.to_string_lossy().to_string())
    }

    fn canonical(&self, path: &str) -> Option<String> {
        std::fs::canonicalize(self.root.join(path))
            .ok()
            .map(|p| p.to_string_lossy().to_string())
    }

    fn occupied(&self, path: &str) -> bool {
        std::fs::symlink_metadata(self.root.join(path)).is_ok()
    }

    fn link(&self, path: &str, target: &str) -> Result<(), String> {
        let at = self.root.join(path);
        if let Some(parent) = at.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("{path} の親のフォルダを作れない：{e}"))?;
        }
        symlink(target, &at).map_err(|e| format!("{path} にリンクを置けない：{e}"))
    }

    fn is_link(&self, path: &str) -> bool {
        std::fs::symlink_metadata(self.root.join(path)).is_ok_and(|m| m.file_type().is_symlink())
    }

    fn unlink(&self, path: &str) -> Result<(), String> {
        if !self.is_link(path) {
            return Err(format!("{path} はシンボリックリンクでないので外さない"));
        }
        remove_link(&self.root.join(path)).map_err(|e| format!("{path} のリンクを外せない：{e}"))
    }
}

#[cfg(unix)]
fn remove_link(at: &Path) -> std::io::Result<()> {
    std::fs::remove_file(at)
}

#[cfg(windows)]
fn remove_link(at: &Path) -> std::io::Result<()> {
    std::fs::remove_dir(at)
}

#[cfg(unix)]
fn symlink(target: &str, at: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(target, at)
}

#[cfg(windows)]
fn symlink(target: &str, at: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_dir(target, at)
}
