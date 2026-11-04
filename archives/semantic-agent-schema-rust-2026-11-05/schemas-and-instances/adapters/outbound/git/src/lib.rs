//! ポート CommitReader を、git コマンドで実装する。読むだけで、Git に書き込む操作（コミット ・ 参照の作成）はしない。
//!
//! - 中身：`git cat-file --filters <コミット>:./<経路>`（渡されたコミット（--base）の、作業ツリーに置いたときと同じ変換を
//!   当てたバイト列。`./` で始まる経路は git を動かすフォルダからの相対として読まれるので、入口は Git の根で動かす）
//!
//! - 記録の置き場所に使う共通のフォルダ：`git rev-parse --path-format=absolute --git-common-dir`
//!   （ワークツリーをいくつ作っても同じ1つ。$GIT_COMMON_DIR があればそれ、無ければ $GIT_DIR）
//! - セッションを打ったフォルダ：`git rev-parse --show-toplevel`（作業ツリーの根。ワークツリーごとに違う）
//!
//! リポジトリでない ・ コミットが無い ・ git が無いときは、前の中身のコミットが無いとして空を返す。

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use writing_application::ports::CommitReader;

/// Git の根の Git を読む。
#[derive(Debug, Clone)]
pub struct GitCommits {
    root: PathBuf,
}

impl GitCommits {
    /// root で git を動かす（入口は Git の根を渡す。根を引く前は作業フォルダ）。
    #[must_use]
    pub const fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// git を動かす。git が無ければ None。
    fn git(&self, args: &[&str]) -> Result<Option<Output>, String> {
        match Command::new("git")
            .args(args)
            .current_dir(&self.root)
            .output()
        {
            Ok(o) => Ok(Some(o)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(format!("git を動かせない：{e}")),
        }
    }

    /// ワークツリーで共通の Git のフォルダ（絶対の経路）。リポジトリでない ・ git が無ければ None。
    #[must_use]
    pub fn common_dir(&self) -> Option<PathBuf> {
        let out = self
            .git(&["rev-parse", "--path-format=absolute", "--git-common-dir"])
            .ok()??;
        if !out.status.success() {
            return None;
        }
        let dir = String::from_utf8_lossy(&out.stdout).trim().to_string();
        (!dir.is_empty()).then(|| Path::new(&dir).to_path_buf())
    }

    /// 作業ツリーの根（絶対の経路）。リポジトリでない ・ git が無ければ None。
    #[must_use]
    pub fn toplevel(&self) -> Option<PathBuf> {
        let out = self.git(&["rev-parse", "--show-toplevel"]).ok()??;
        if !out.status.success() {
            return None;
        }
        let dir = String::from_utf8_lossy(&out.stdout).trim().to_string();
        (!dir.is_empty()).then(|| Path::new(&dir).to_path_buf())
    }
}

fn spec(rev: &str, path: &str) -> String {
    format!("{rev}:./{path}")
}

impl CommitReader for GitCommits {
    fn read(&self, rev: &str, path: &str) -> Result<Option<Vec<u8>>, String> {
        let object = spec(rev, path);
        // 無いものは -e が 0 以外で終わる
        let Some(exists) = self.git(&["cat-file", "-e", &object])? else {
            return Ok(None);
        };
        if !exists.status.success() {
            return Ok(None);
        }
        let Some(out) = self.git(&["cat-file", "--filters", &object])? else {
            return Ok(None);
        };
        if !out.status.success() {
            return Err(format!(
                "git cat-file が失敗した：{}",
                String::from_utf8_lossy(&out.stderr).trim()
            ));
        }
        Ok(Some(out.stdout))
    }
}
