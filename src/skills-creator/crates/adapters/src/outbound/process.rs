//! port の Programs の実装。実行ファイルを起動し、終了コードと出力を返す。

use skills_creator_core::ports::{Programs, Run};
use std::path::Path;
use std::process::Command;

pub struct RealPrograms;

impl Programs for RealPrograms {
    fn run(&self, program: &Path, args: &[String]) -> Result<Run, String> {
        let output = Command::new(program)
            .args(args)
            .output()
            .map_err(|error| format!("{} を起動できない：{error}", program.display()))?;
        Ok(Run {
            code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }
}
