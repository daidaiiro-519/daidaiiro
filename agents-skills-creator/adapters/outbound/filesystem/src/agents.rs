//! ポート AgentFiles ・ GeneratedFiles ・ Destinations を、作業ディレクトリのファイルとシンボリックリンクで実装する。
//! 配布先には、Agent の定義のファイルを写して置く。
//! GeneratedFiles で書くのは creator が作るファイル（AGENT.md ・ codex.toml）だけで、agent.json は書かない（SAS が書く）。

use agents_application::ports::{AgentFiles, Destinations, GeneratedFiles};
use std::path::PathBuf;

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

impl AgentFiles for WorkingTree {
    fn read(&self, path: &str) -> Result<String, String> {
        std::fs::read_to_string(self.root.join(path)).map_err(|e| format!("{path} を読めない：{e}"))
    }

    fn entries(&self, folder: &str) -> Vec<String> {
        let mut out: Vec<String> = std::fs::read_dir(self.root.join(folder))
            .into_iter()
            .flatten()
            .filter_map(Result::ok)
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        out.sort();
        out
    }

    fn resolve(&self, path: &str) -> Option<String> {
        let root = std::fs::canonicalize(&self.root).ok()?;
        let at = std::fs::canonicalize(self.root.join(path)).ok()?;
        let rel = at.strip_prefix(&root).ok()?;
        Some(rel.to_string_lossy().replace('\\', "/"))
    }
}

impl GeneratedFiles for WorkingTree {
    fn write(&self, path: &str, text: &str) -> Result<(), String> {
        let to = self.root.join(path);
        let tmp = to.with_extension("asc-tmp");
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

    fn occupied(&self, path: &str) -> bool {
        std::fs::symlink_metadata(self.root.join(path)).is_ok()
    }

    fn absolute(&self, path: &str) -> Option<String> {
        std::fs::canonicalize(self.root.join(path))
            .ok()
            .map(|p| p.to_string_lossy().replace('\\', "/"))
    }

    fn is_file(&self, path: &str) -> bool {
        std::fs::symlink_metadata(self.root.join(path)).is_ok_and(|m| m.file_type().is_file())
    }

    fn place(&self, path: &str, text: &str) -> Result<(), String> {
        let at = self.root.join(path);
        if let Some(parent) = at.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("{path} の親のフォルダを作れない：{e}"))?;
        }
        std::fs::write(&at, text).map_err(|e| format!("{path} を書けない：{e}"))
    }

    fn remove(&self, path: &str) -> Result<(), String> {
        let at = self.root.join(path);
        if !std::fs::symlink_metadata(&at).is_ok_and(|m| m.file_type().is_file()) {
            return Err(format!("{path} はふつうのファイルでないので消さない"));
        }
        std::fs::remove_file(at).map_err(|e| format!("{path} を消せない：{e}"))
    }
}
