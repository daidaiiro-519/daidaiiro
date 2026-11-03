//! ポート AgentValidator ・ AgentWriter ・ AgentHistory を、SAS の semantic-agent-schema コマンドを外の実行ファイルとして呼んで実装する。
//! 出力は書き換えずに返す。semantic-agent-schema は PATH だけで探す。打った場所と環境変数はそのまま渡す。

use agents_application::ports::{AgentHistory, AgentValidator, AgentWriter, Run};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

/// 外の実行ファイルを打ち、終了コードと出力の JSON（1行なら1つ、複数の行なら行の並び）を返す。
///
/// # Errors
/// 起動できない ・ 出力が JSON でない ・ 何も出力しないとき。
pub fn run(program: &Path, args: &[&str]) -> Result<Run, String> {
    let out = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| format!("{} を起動できない：{e}", program.display()))?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<Value> = stdout
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            serde_json::from_str(l.trim())
                .map_err(|e| format!("{} の出力が JSON でない（{e}）：{l}", program.display()))
        })
        .collect::<Result<_, _>>()?;
    let output = match <[Value; 1]>::try_from(lines) {
        Ok([one]) => one,
        Err(lines) if lines.is_empty() => {
            return Err(format!(
                "{} が何も出力しなかった：{}",
                program.display(),
                String::from_utf8_lossy(&out.stderr)
            ));
        }
        Err(lines) => Value::Array(lines),
    };
    Ok(Run {
        exit: out.status.code().unwrap_or(-1),
        output,
    })
}

/// PATH から実行ファイルを探す。
#[must_use]
pub fn locate(name: &str) -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .map(|d| d.join(name))
            .find(|p| p.is_file())
    })
}

/// semantic-agent-schema を呼ぶ出口。
#[derive(Debug, Clone)]
pub struct Sas {
    program: PathBuf,
}

impl Sas {
    /// PATH の semantic-agent-schema。PATH に無ければ None。
    #[must_use]
    pub fn locate() -> Option<Self> {
        locate("semantic-agent-schema").map(|program| Self { program })
    }
}

impl AgentValidator for Sas {
    fn validate(&self, path: &str) -> Result<Run, String> {
        run(&self.program, &["validate", path])
    }
}

impl AgentWriter for Sas {
    fn create_semantic(&self, path: &str, concrete: &str) -> Result<Run, String> {
        run(
            &self.program,
            &["create", path, "--concrete", concrete, "--semantic"],
        )
    }

    fn create_value(&self, path: &str, concrete: &str, value: &str) -> Result<Run, String> {
        run(
            &self.program,
            &["create", path, "--concrete", concrete, "--value", value],
        )
    }

    fn update_semantic(&self, path: &str, task: &str) -> Result<Run, String> {
        run(
            &self.program,
            &["update", path, "--semantic", "--task", task],
        )
    }

    fn update_patch(
        &self,
        path: &str,
        patch: &str,
        if_match: &str,
        reason: &str,
    ) -> Result<Run, String> {
        run(
            &self.program,
            &[
                "update",
                path,
                "--patch",
                patch,
                "--if-match",
                if_match,
                "--reason",
                reason,
            ],
        )
    }
}

impl AgentHistory for Sas {
    fn log(&self, path: &str) -> Result<Run, String> {
        run(&self.program, &["log", path])
    }

    fn operation(&self, id: &str) -> Result<Run, String> {
        run(&self.program, &["log", "--operation", id])
    }
}
