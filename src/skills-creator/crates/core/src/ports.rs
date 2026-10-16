//! 出ていく側の port。application が定義し、adapters が実装する。

use std::path::{Path, PathBuf};

/// ジェネリクスで使う。ファイルとフォルダの読み書き。
pub trait Files {
    fn read(&self, path: &Path) -> Result<String, String>;
    fn read_bytes(&self, path: &Path) -> Result<Vec<u8>, String>;
    fn write(&self, path: &Path, text: &str) -> Result<(), String>;
    fn exists(&self, path: &Path) -> bool;
    /// フォルダの直下のファイルとフォルダ。名前の順に並べる。無ければ空。
    fn list(&self, dir: &Path) -> Vec<PathBuf>;
    fn create_dir(&self, dir: &Path) -> Result<(), String>;
    fn copy(&self, from: &Path, to: &Path) -> Result<(), String>;
    /// 同じファイルシステムの中で移す。移す先が在れば置き換える。
    fn rename(&self, from: &Path, to: &Path) -> Result<(), String>;
    fn remove_dir(&self, dir: &Path) -> Result<(), String>;
    fn remove_file(&self, path: &Path) -> Result<(), String>;
    /// 実行できるファイルにする（Windows では何もしない）。
    fn set_executable(&self, path: &Path) -> Result<(), String>;
    /// 作業用の新しい空のフォルダ。
    fn temp_dir(&self) -> Result<PathBuf, String>;
}

/// 起動した結果。
#[derive(Clone, Debug, PartialEq)]
pub struct Run {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// ジェネリクスで使う。実行ファイルの起動。
pub trait Programs {
    fn run(&self, program: &Path, args: &[String]) -> Result<Run, String>;
}

/// ジェネリクスで使う。公開された基盤の配布物の取得 ・ 証明の照合 ・ 展開。
pub trait Releases {
    /// tag の Release から、names のファイルを into へ取る。
    fn download(&self, tag: &str, names: &[String], into: &Path) -> Result<(), String>;
    /// file の証明を、bundle で照合する。署名したワークフローと tag まで縛る。
    fn attest(&self, file: &Path, bundle: &Path, tag: &str) -> Result<(), String>;
    /// 配布物（tar.gz か zip）を into へ展開する。
    fn extract(&self, archive: &Path, into: &Path) -> Result<(), String>;
}
