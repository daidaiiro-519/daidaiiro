//! 出ていく側の port。application が定義し、adapters が実装する。

use crate::domain::agent::Plan;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// ジェネリクスで使う。ファイルとフォルダの読み書き。
pub trait Files {
    fn read(&self, path: &Path) -> Result<String, String>;
    fn read_bytes(&self, path: &Path) -> Result<Vec<u8>, String>;
    /// 書く。親のフォルダが無ければ作る。
    fn write(&self, path: &Path, text: &str) -> Result<(), String>;
    /// 無いときだけ書く（あるか無いかの確かめと書くことを一度に行う）。書いたら true、すでにあれば false。
    fn create_new(&self, path: &Path, text: &str) -> Result<bool, String>;
    fn exists(&self, path: &Path) -> bool;
    /// フォルダの直下のファイルとフォルダ。名前の順に並べる。無ければ空。
    fn list(&self, dir: &Path) -> Vec<PathBuf>;
    /// フォルダの下のすべてのファイル（フォルダは含めない）。名前の順に並べる。無ければ空。
    fn tree(&self, dir: &Path) -> Vec<PathBuf>;
    fn remove_file(&self, path: &Path) -> Result<(), String>;
    /// フォルダを中身ごと消す。無ければ何もしない。
    fn remove_dir(&self, dir: &Path) -> Result<(), String>;
}

/// 起動した結果。
#[derive(Clone, Debug, PartialEq)]
pub struct Run {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// ジェネリクスで使う。終わるまで待つ実行ファイルの起動（基盤のエンジン ・ git）。
pub trait Programs {
    fn run(&self, program: &Path, args: &[String], cwd: Option<&Path>) -> Result<Run, String>;
}

/// ジェネリクスで使う。時刻と ID。
pub trait Clock {
    /// いまの時刻（RFC 3339、UTC）
    fn now(&self) -> String;
    /// きょうの日付（YYYY-MM-DD）
    fn today(&self) -> String;
    /// 起動してからの経過（ミリ秒）。時間切れを数えるのに使う
    fn elapsed_ms(&self) -> u64;
    /// 新しい UUID（v4）
    fn uuid(&self) -> String;
    /// いまの時刻（1970-01-01 からの秒）。ロックの古さを数えるのに使う
    fn epoch_secs(&self) -> u64;
}

/// 起動した AI の CLI から次に来るもの。
#[derive(Clone, Debug, PartialEq)]
pub enum Next {
    /// 標準出力の1行
    Line(String),
    /// 待つあいだに何も来なかった
    Quiet,
    /// 終わった（終了コードと、標準エラーの末尾）
    Ended(i32, String),
}

/// ジェネリクスで使う。動いている AI の CLI。
pub trait Running {
    /// wait まで待って、次の1行か、静かだったことか、終わったことを返す。
    fn next(&mut self, wait: Duration) -> Next;
    /// 子孫のプロセスまで止める。
    fn stop(&mut self);
}

/// ジェネリクスで使う。AI の CLI の起動。
pub trait Agents {
    type Started: Running;
    /// シェルを通さずに起動する。cwd はリポジトリの根。
    fn start(&self, plan: &Plan, cwd: &Path) -> Result<Self::Started, String>;
    /// その実行ファイルが PATH にあるか。
    fn found(&self, program: &str) -> bool;
}
