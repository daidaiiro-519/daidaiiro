//! ポート DocumentStore を、作業ディレクトリのファイルで実装する。
//! 書くときは一時ファイルに書いてから置き換える。リポジトリの外と、降りないフォルダには書かない。

use contracts_and_instances_conformance_application::ports::DocumentStore;
use std::path::{Path, PathBuf};

/// 降りないフォルダ（SAS の記録 ・ ビルドの出力 ・ Git の中身）。
const SKIP: [&str; 3] = [".sas", "target", ".git"];

/// 作業ディレクトリを根にした文書の置き場所。
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

fn walk(dir: &Path, rel: &str, out: &mut Vec<String>) -> Result<(), String> {
    let entries =
        std::fs::read_dir(dir).map_err(|e| format!("{} を読めない：{e}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().to_string();
        let here = if rel.is_empty() {
            name.clone()
        } else {
            format!("{rel}/{name}")
        };
        let kind = entry.file_type().map_err(|e| e.to_string())?;
        if kind.is_dir() {
            if !SKIP.contains(&name.as_str()) {
                walk(&entry.path(), &here, out)?;
            }
        } else if kind.is_file() && name.ends_with(".json") {
            out.push(here);
        }
    }
    Ok(())
}

impl DocumentStore for WorkingTree {
    fn list(&self) -> Result<Vec<String>, String> {
        let mut out = Vec::new();
        walk(&self.root, "", &mut out)?;
        out.sort();
        Ok(out)
    }

    fn read(&self, path: &str) -> Result<String, String> {
        std::fs::read_to_string(self.root.join(path)).map_err(|e| format!("{path} を読めない：{e}"))
    }

    fn write(&self, path: &str, text: &str) -> Result<(), String> {
        let parts: Vec<&str> = path.split('/').collect();
        if path.is_empty()
            || path.starts_with('/')
            || path.contains('\\')
            || parts
                .iter()
                .any(|p| p.is_empty() || *p == "." || *p == "..")
            || parts.iter().any(|p| SKIP.contains(p))
        {
            return Err(format!(
                "{path} には書かない（リポジトリの根からの相対の経路で、{} の中でないものだけ）",
                SKIP.join(" ・ ")
            ));
        }
        let to = self.root.join(path);
        if let Some(dir) = to.parent() {
            std::fs::create_dir_all(dir)
                .map_err(|e| format!("{} を作れない：{e}", dir.display()))?;
        }
        let tmp = to.with_file_name(format!(
            ".{}.sas-tmp",
            to.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default()
        ));
        std::fs::write(&tmp, text).map_err(|e| format!("{path} を書けない：{e}"))?;
        std::fs::rename(&tmp, &to).map_err(|e| {
            let _ = std::fs::remove_file(&tmp);
            format!("{path} を置き換えられない：{e}")
        })
    }
}
