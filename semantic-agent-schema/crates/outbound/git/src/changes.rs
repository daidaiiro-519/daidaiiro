//! 前後（前回のコミットと今の作業ツリー）と、worktree が共有する置き場所

use semantic_agent_schema_application::ports::changes::{Changes, FileChange};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Git の CLI で、作業の根の下の変更を読む
pub struct Git {
    root: PathBuf,
}

impl Git {
    /// 作業の根を渡して作る
    pub fn new(root: PathBuf) -> Self {
        Git { root }
    }

    /// git を作業の根で動かし、成功したときの標準出力を返す。失敗なら None
    fn run(&self, args: &[&str]) -> Result<Option<Vec<u8>>, String> {
        let out = Command::new("git")
            .arg("-C")
            .arg(&self.root)
            .args(args)
            .output()
            .map_err(|e| format!("git を呼べない：{e}"))?;
        Ok(out.status.success().then_some(out.stdout))
    }

    fn text(&self, args: &[&str]) -> Result<Option<String>, String> {
        Ok(self
            .run(args)?
            .map(|b| String::from_utf8_lossy(&b).trim_end().to_string()))
    }
}

impl Changes for Git {
    fn head(&self) -> Result<Option<String>, String> {
        self.text(&["rev-parse", "--verify", "--quiet", "HEAD"])
    }

    fn changed(&self) -> Result<Vec<FileChange>, String> {
        let head = self.head()?;
        let prefix = self
            .text(&["rev-parse", "--show-prefix"])?
            .ok_or("Git のリポジトリの中でない")?;
        let status = self
            .run(&[
                "status",
                "--porcelain",
                "-z",
                "--untracked-files=all",
                "--no-renames",
                "--",
                ".",
            ])?
            .ok_or("git status が失敗した")?;
        let mut out = Vec::new();
        for entry in status.split(|b| *b == 0).filter(|e| e.len() > 3) {
            let code = &entry[..2];
            let whole = String::from_utf8_lossy(&entry[3..]).to_string();
            let Some(path) = whole.strip_prefix(&prefix) else {
                continue;
            };
            let before = match (&head, code == b"??") {
                (Some(_), false) => self
                    .run(&["show", &format!("HEAD:./{path}")])?
                    .map(|b| String::from_utf8_lossy(&b).to_string()),
                _ => None,
            };
            let after = std::fs::read(self.root.join(path))
                .ok()
                .map(|b| String::from_utf8_lossy(&b).to_string());
            out.push(FileChange {
                path: path.to_string(),
                before,
                after,
            });
        }
        Ok(out)
    }

    fn workspace(&self) -> Result<Option<String>, String> {
        let (Some(own), Some(common)) = (
            self.text(&["rev-parse", "--absolute-git-dir"])?,
            self.text(&["rev-parse", "--path-format=absolute", "--git-common-dir"])?,
        ) else {
            return Ok(None);
        };
        // メインのワークツリーは「main」、追加したワークツリーは共有の置き場所からの相対（worktrees/<名前>）
        Ok(Some(match Path::new(&own).strip_prefix(&common) {
            Ok(rel) if rel.as_os_str().is_empty() => "main".to_string(),
            Ok(rel) => rel.to_string_lossy().replace('\\', "/"),
            Err(_) => own,
        }))
    }

    fn shared_dir(&self) -> Option<PathBuf> {
        let dir = self.text(&["rev-parse", "--git-common-dir"]).ok()??;
        let p = Path::new(&dir);
        Some(if p.is_absolute() {
            p.to_path_buf()
        } else {
            self.root.join(p)
        })
    }
}
