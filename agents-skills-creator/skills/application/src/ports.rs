//! UC1 作る ・ UC4 確かめるが外に求めるもの（使う操作だけを宣言する。実装は adapters/outbound）。

use serde_json::Value;

/// SAS で Skill の項目（skill.json）を検証した結果。SAS の出力を書き換えずに持つ。
#[derive(Debug, Clone, PartialEq)]
pub struct SasRun {
    /// semantic-agent-schema の終了コード（0 は violated と error が無い）
    pub exit: i32,
    /// semantic-agent-schema が標準出力に出した JSON（1行なら1つ、複数の行なら行の並び）
    pub output: Value,
}

/// SAS（共用サービス）で、Skill の項目を検証する。変換を挟まずに従う。
pub trait SkillValidator {
    /// skill.json の経路（打った場所からの経路）を semantic-agent-schema validate に渡し、その結果を返す。
    ///
    /// # Errors
    /// semantic-agent-schema を呼べない ・ 出力が JSON でないとき。
    fn validate(&self, skill_json: &str) -> Result<SasRun, String>;

    /// SAS が、打った場所の中にある Concrete として、この $id（URN）を引けるか。
    ///
    /// # Errors
    /// semantic-agent-schema を呼べない ・ 出力が JSON でないとき。
    fn knows_concrete(&self, urn: &str) -> Result<bool, String>;
}

/// SAS（共用サービス）に、Skill の項目を書かせる。結果は書き換えずに返す。
pub trait SkillWriter {
    /// semantic-agent-schema create <経路> --concrete <URN> --semantic：意味で書く手順を受け取る（何も書かない）。
    ///
    /// # Errors
    /// semantic-agent-schema を呼べない ・ 出力が JSON でないとき。
    fn create_semantic(&self, path: &str, concrete: &str) -> Result<SasRun, String>;

    /// semantic-agent-schema create <経路> --concrete <URN> --value <JSON>：値を書かせる。
    ///
    /// # Errors
    /// semantic-agent-schema を呼べない ・ 出力が JSON でないとき。
    fn create_value(&self, path: &str, concrete: &str, value: &str) -> Result<SasRun, String>;

    /// semantic-agent-schema update <経路> --semantic --task <依頼>：意味で直す手順を受け取る（何も書かない）。
    ///
    /// # Errors
    /// semantic-agent-schema を呼べない ・ 出力が JSON でないとき。
    fn update_semantic(&self, path: &str, task: &str) -> Result<SasRun, String>;

    /// semantic-agent-schema update <経路> --patch <JSON Patch> --if-match <ハッシュ> --reason <理由>：直しを書かせる。
    ///
    /// # Errors
    /// semantic-agent-schema を呼べない ・ 出力が JSON でないとき。
    fn update_patch(
        &self,
        path: &str,
        patch: &str,
        if_match: &str,
        reason: &str,
    ) -> Result<SasRun, String>;
}

/// SAS の記録を読む（書かない）。差分は SAS が記録したものを使い、creator で作らない。
pub trait SkillHistory {
    /// semantic-agent-schema log <経路>：その文書の操作の並び（新しい順）。
    ///
    /// # Errors
    /// semantic-agent-schema を呼べない ・ 出力が JSON でないとき。
    fn log(&self, path: &str) -> Result<SasRun, String>;

    /// semantic-agent-schema log --operation <ID>：操作1件の中身（書いた変更なら所ごとの前と後）。
    ///
    /// # Errors
    /// semantic-agent-schema を呼べない ・ 出力が JSON でないとき。
    fn operation(&self, id: &str) -> Result<SasRun, String>;
}

/// ふつうの JSON Schema（2020-12）で、値を確かめる。
pub trait JsonSchemaChecker {
    /// value を schema に照らし、外れた所（JSON Pointer）と理由を返す（従えば空）。
    ///
    /// # Errors
    /// schema を JSON Schema として組めないとき。
    fn check(&self, schema: &Value, value: &Value) -> Result<Vec<(String, String)>, String>;
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

/// creator が作るファイル（SKILL.md）を書く。skill.json は書かない（SAS が書く）。
pub trait GeneratedFiles {
    /// ファイルを書く。途中で止まっても、前の中身か新しい中身のどちらかが残る。
    ///
    /// # Errors
    /// 書けないとき。
    fn write(&self, path: &str, text: &str) -> Result<(), String>;
}

/// 配布先（ツールの置き場所）を読み、Skill のフォルダへのシンボリックリンクを置く。置き場所のほかのものは変えない。
pub trait Destinations {
    /// 利用者の全体の置き場所の根（HOME）。分からなければ None。
    fn home(&self) -> Option<String>;

    /// 経路の先の、リンクをたどった絶対の経路。先が無ければ None。
    fn canonical(&self, path: &str) -> Option<String>;

    /// 経路に何かがあるか（先の無いシンボリックリンクも、あるとみなす）。
    fn occupied(&self, path: &str) -> bool;

    /// 経路に、target を指すシンボリックリンクを置く（親のフォルダが無ければ作る）。
    ///
    /// # Errors
    /// 置けないとき。
    fn link(&self, path: &str, target: &str) -> Result<(), String>;

    /// 経路がシンボリックリンクか（先が無くても）。
    fn is_link(&self, path: &str) -> bool;

    /// 経路のシンボリックリンクを外す（リンクの先は変えない。リンクでなければ外さない）。
    ///
    /// # Errors
    /// リンクでない ・ 外せないとき。
    fn unlink(&self, path: &str) -> Result<(), String>;
}
