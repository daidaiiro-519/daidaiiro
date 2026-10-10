//! 基盤のエンジン（schema-driven）を、実行ファイルとして呼ぶ。ライブラリとして取り込まない。

use serde_json::Value;
use std::path::Path;
use std::process::Command;

/// エンジンを起動し、標準出力の JSON を返す。起動できない ・ JSON でないときは理由を返す。
pub fn call(engine: &Path, args: &[&str]) -> Result<Value, String> {
    let output = Command::new(engine).args(args).output().map_err(|error| {
        format!(
            "基盤のエンジン {} を起動できない：{error}",
            engine.display()
        )
    })?;
    serde_json::from_slice(&output.stdout).map_err(|_| {
        format!(
            "基盤のエンジンの出力が JSON でない：{}",
            String::from_utf8_lossy(&output.stdout)
                .chars()
                .take(200)
                .collect::<String>()
        )
    })
}
