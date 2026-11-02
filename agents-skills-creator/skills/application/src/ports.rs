//! UC4 確かめるが外に求めるもの（使う操作だけを宣言する。実装は adapters/outbound）。

use serde_json::Value;

/// SAS で Skill の項目（skill.json）を検証した結果。SAS の出力を書き換えずに持つ。
#[derive(Debug, Clone, PartialEq)]
pub struct SasRun {
    /// sas の終了コード（0 は violated と error が無い）
    pub exit: i32,
    /// sas が標準出力に出した JSON（1行なら1つ、複数の行なら行の並び）
    pub output: Value,
}

/// SAS（共用サービス）で、Skill の項目を検証する。変換を挟まずに従う。
pub trait SkillValidator {
    /// skill.json の経路（打った場所からの経路）を sas validate に渡し、その結果を返す。
    ///
    /// # Errors
    /// sas を呼べない ・ 出力が JSON でないとき。
    fn validate(&self, skill_json: &str) -> Result<SasRun, String>;
}

/// Skill のフォルダとファイルとリンクを読む（書かない）。経路は打った場所からの経路で、区切りは /。
pub trait SkillFiles {
    /// 経路の先があるか（シンボリックリンクはたどる。先の無いリンクは無い）。
    fn exists(&self, path: &str) -> bool;

    /// ファイルの中身。
    ///
    /// # Errors
    /// 無い ・ 読めないとき。
    fn read(&self, path: &str) -> Result<String, String>;

    /// 同じ親フォルダにある、ほかのフォルダの経路（自分は除く。経路の順）。
    ///
    /// # Errors
    /// 親フォルダを読めないとき。
    fn siblings(&self, folder: &str) -> Result<Vec<String>, String>;

    /// フォルダの中にある、先の無いシンボリックリンクの経路（経路の順。リンクの先のフォルダの中へは降りない）。
    ///
    /// # Errors
    /// フォルダを読めないとき。
    fn dangling_links(&self, folder: &str) -> Result<Vec<String>, String>;
}
