//! ポート SkillsCli の実装。文脈 skills の公開された言葉（skills コマンドの validate の結果）を、skills を PATH から探して
//! 外の実行ファイルとして呼んで読む。skills の crate には依存しない。出力は書き換えずに返す。

use agents_application::ports::{Run, SkillsCli};
use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;

/// skills を呼ぶ出口。
#[derive(Debug, Clone)]
pub struct Skills {
    program: PathBuf,
}

impl Skills {
    /// PATH の skills。PATH に無ければ None。
    #[must_use]
    pub fn locate() -> Option<Self> {
        std::env::var_os("PATH")
            .and_then(|paths| {
                std::env::split_paths(&paths)
                    .map(|d| d.join("skills"))
                    .find(|p| p.is_file())
            })
            .map(|program| Self { program })
    }
}

impl SkillsCli for Skills {
    fn validate(&self, folder: &str) -> Result<Run, String> {
        let out = Command::new(&self.program)
            .args(["validate", folder])
            .output()
            .map_err(|e| format!("{} を起動できない：{e}", self.program.display()))?;
        let text = String::from_utf8_lossy(&out.stdout);
        let output: Value = serde_json::from_str(text.trim())
            .map_err(|e| format!("skills の出力が JSON でない（{e}）：{text}"))?;
        Ok(Run {
            exit: out.status.code().unwrap_or(-1),
            output,
        })
    }
}
