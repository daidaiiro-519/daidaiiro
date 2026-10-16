//! 端から端までのテストの補助。作業用のフォルダと、実行ファイルの起動を、ここ1か所に置く。
#![allow(dead_code)]

use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// 試験の作業用のフォルダ。実行ごとに別の名前で作り、試験が終わると消す。
pub struct Scratch(PathBuf);

impl Scratch {
    pub fn new(name: &str) -> Self {
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("sc-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        Scratch(dir)
    }

    pub fn path(&self, rel: &str) -> PathBuf {
        self.0.join(rel)
    }

    pub fn put(&self, rel: &str, text: &str) {
        let path = self.path(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    pub fn read(&self, rel: &str) -> String {
        fs::read_to_string(self.path(rel)).unwrap()
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// skills-creator の実行ファイルを起動し、終了コードと出力の JSON を返す。
pub fn cli(args: &[&str]) -> (i32, Value) {
    let output = Command::new(env!("CARGO_BIN_EXE_skills-creator"))
        .args(args)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let out =
        serde_json::from_str(&stdout).unwrap_or_else(|_| panic!("出力が JSON でない：{stdout}"));
    (output.status.code().unwrap_or(-1), out)
}

/// check の結果を、項目の id と結果の組に並べる。
pub fn results(report: &Value) -> Vec<(String, String)> {
    report["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| {
            (
                item["id"].as_str().unwrap().to_owned(),
                item["result"].as_str().unwrap().to_owned(),
            )
        })
        .collect()
}
