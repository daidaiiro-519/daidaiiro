//! port の Programs の実装。実行ファイルを起動し、終わるまで待って終了コードと出力を返す。

use brainstorming_core::ports::{Programs, Run};
use std::path::Path;
use std::process::{Command, Stdio};

pub struct RealPrograms;

impl Programs for RealPrograms {
    fn run(&self, program: &Path, args: &[String], cwd: Option<&Path>) -> Result<Run, String> {
        let mut command = Command::new(program);
        command.args(args).stdin(Stdio::null());
        if let Some(dir) = cwd {
            command.current_dir(dir);
        }
        let output = command
            .output()
            .map_err(|error| format!("{} を起動できない：{error}", program.display()))?;
        Ok(Run {
            code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }
}
