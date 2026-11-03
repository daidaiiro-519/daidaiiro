//! ポート SkillsCli の実装。文脈 skills の公開された言葉（agents-skills-creator skills validate の結果）を、同じ実行ファイル
//! （agents-skills-creator。無ければ PATH のもの）を外のプロセスとして呼んで読む。skills の crate には依存しない。
//! 出力は書き換えずに返す。

use agents_application::ports::{Run, SkillsCli};
use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;

/// 実行ファイルの名前。
const PROGRAM: &str = "agents-skills-creator";

/// agents-skills-creator skills を呼ぶ出口。
#[derive(Debug, Clone)]
pub struct Skills {
    program: PathBuf,
}

impl Skills {
    /// 使う実行ファイル：いま動いている agents-skills-creator を先に使い、無ければ PATH の agents-skills-creator。
    #[must_use]
    pub fn locate() -> Option<Self> {
        let me = std::env::current_exe()
            .ok()
            .filter(|p| p.file_stem().is_some_and(|n| n == PROGRAM));
        me.or_else(|| {
            std::env::var_os("PATH").and_then(|paths| {
                std::env::split_paths(&paths)
                    .map(|d| d.join(PROGRAM))
                    .find(|p| p.is_file())
            })
        })
        .map(|program| Self { program })
    }
}

impl SkillsCli for Skills {
    fn validate(&self, folder: &str) -> Result<Run, String> {
        let out = Command::new(&self.program)
            .args(["skills", "validate", folder])
            .output()
            .map_err(|e| format!("{} を起動できない：{e}", self.program.display()))?;
        let text = String::from_utf8_lossy(&out.stdout);
        let output: Value = serde_json::from_str(text.trim()).map_err(|e| {
            format!("agents-skills-creator skills の出力が JSON でない（{e}）：{text}")
        })?;
        Ok(Run {
            exit: out.status.code().unwrap_or(-1),
            output,
        })
    }
}
