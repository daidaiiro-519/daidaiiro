//! ポート DocumentStore を、作業ディレクトリのファイルで実装する。書かない。

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
}
