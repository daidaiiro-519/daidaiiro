//! ポート SkillValidator を、SAS の sas コマンドを外の実行ファイルとして呼んで実装する。出力は書き換えずに返す。
//!
//! sas は PATH だけで探す（sas は環境に1つ。Skill は使う版の範囲だけを宣言する）。打った場所（作業ディレクトリ）と
//! 環境変数はそのまま渡す（sas は打った場所の中から Concrete を探し、打ったセッションを環境変数で決める）。

use serde_json::Value;
use skills_application::ports::{SasRun, SkillValidator};
use std::path::{Path, PathBuf};
use std::process::Command;

/// sas を呼ぶ出口。
#[derive(Debug, Clone)]
pub struct Sas {
    program: PathBuf,
}

impl Sas {
    /// PATH の sas。PATH に無ければ None。
    #[must_use]
    pub fn locate() -> Option<Self> {
        std::env::var_os("PATH")
            .and_then(|paths| {
                std::env::split_paths(&paths)
                    .map(|d| d.join("sas"))
                    .find(|p| p.is_file())
            })
            .map(|program| Self { program })
    }

    /// 使う sas の経路。
    #[must_use]
    pub fn program(&self) -> &Path {
        &self.program
    }
}

impl Sas {
    /// sas を打ち、終了コードと出力の JSON（1行なら1つ、複数の行なら行の並び）を返す。
    fn run(&self, args: &[&str]) -> Result<SasRun, String> {
        let out = Command::new(&self.program)
            .args(args)
            .output()
            .map_err(|e| format!("{} を起動できない：{e}", self.program.display()))?;
        let stdout = String::from_utf8_lossy(&out.stdout);
        let lines: Vec<Value> = stdout
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| {
                serde_json::from_str(l.trim())
                    .map_err(|e| format!("sas の出力が JSON でない（{e}）：{l}"))
            })
            .collect::<Result<_, _>>()?;
        let output = match <[Value; 1]>::try_from(lines) {
            Ok([one]) => one,
            Err(lines) if lines.is_empty() => {
                return Err(format!(
                    "sas が何も出力しなかった：{}",
                    String::from_utf8_lossy(&out.stderr)
                ));
            }
            Err(lines) => Value::Array(lines),
        };
        Ok(SasRun {
            exit: out.status.code().unwrap_or(-1),
            output,
        })
    }
}

impl SkillValidator for Sas {
    fn validate(&self, skill_json: &str) -> Result<SasRun, String> {
        self.run(&["validate", skill_json])
    }

    fn knows_concrete(&self, urn: &str) -> Result<bool, String> {
        // その Concrete に従う文書を、範囲を Concrete にして構造で探す。SAS が Concrete を引けなければ unknown_schema で断る
        let r = self.run(&[
            "search",
            "--concrete",
            urn,
            "--where",
            &format!("/$schema={urn}"),
        ])?;
        match (
            r.output.get("kind").and_then(Value::as_str),
            r.output.get("code").and_then(Value::as_str),
        ) {
            (Some("search"), _) => Ok(true),
            (Some("error"), Some("unknown_schema" | "schema_not_listed")) => Ok(false),
            _ => Err(format!("sas search の結果を読めない：{}", r.output)),
        }
    }
}
