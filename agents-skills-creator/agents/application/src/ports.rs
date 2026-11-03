//! UC2 Agent を作るが外に求めるもの（使う操作だけを宣言する。実装は adapters/outbound）。

use serde_json::Value;

/// 外のコマンド（sas ・ skills）を打った結果。出力は書き換えずに持つ。
#[derive(Debug, Clone, PartialEq)]
pub struct Run {
    /// 終了コード
    pub exit: i32,
    /// 標準出力の JSON（1行なら1つ、複数の行なら行の並び）
    pub output: Value,
}

/// SAS（共用サービス）で Agent の項目を検証する。
pub trait AgentValidator {
    /// sas validate <経路>。
    ///
    /// # Errors
    /// sas を呼べない ・ 出力が JSON でないとき。
    fn validate(&self, path: &str) -> Result<Run, String>;
}

/// SAS に Agent の項目を書かせる。
pub trait AgentWriter {
    /// sas create <経路> --concrete <URN> --semantic。
    ///
    /// # Errors
    /// sas を呼べない ・ 出力が JSON でないとき。
    fn create_semantic(&self, path: &str, concrete: &str) -> Result<Run, String>;

    /// sas create <経路> --concrete <URN> --value <JSON>。
    ///
    /// # Errors
    /// sas を呼べない ・ 出力が JSON でないとき。
    fn create_value(&self, path: &str, concrete: &str, value: &str) -> Result<Run, String>;

    /// sas update <経路> --semantic --task <依頼>。
    ///
    /// # Errors
    /// sas を呼べない ・ 出力が JSON でないとき。
    fn update_semantic(&self, path: &str, task: &str) -> Result<Run, String>;

    /// sas update <経路> --patch <JSON Patch> --if-match <ハッシュ> --reason <理由>。
    ///
    /// # Errors
    /// sas を呼べない ・ 出力が JSON でないとき。
    fn update_patch(
        &self,
        path: &str,
        patch: &str,
        if_match: &str,
        reason: &str,
    ) -> Result<Run, String>;
}

/// SAS の記録を読む（書かない）。
pub trait AgentHistory {
    /// sas log <経路>（新しい順）。
    ///
    /// # Errors
    /// sas を呼べない ・ 出力が JSON でないとき。
    fn log(&self, path: &str) -> Result<Run, String>;

    /// sas log --operation <ID>。
    ///
    /// # Errors
    /// sas を呼べない ・ 出力が JSON でないとき。
    fn operation(&self, id: &str) -> Result<Run, String>;
}

/// 文脈 skills の公開された言葉を読む：agents-skills-creator skills validate <フォルダ> の結果（validate-result）。
pub trait SkillsCli {
    /// agents-skills-creator skills validate <フォルダ>。
    ///
    /// # Errors
    /// skills を呼べない ・ 出力が JSON でないとき。
    fn validate(&self, folder: &str) -> Result<Run, String>;
}

/// Agent のフォルダとファイルとリンクを読む（書かない）。経路は打った場所からの経路で、区切りは /。
pub trait AgentFiles {
    /// ファイルの中身。
    ///
    /// # Errors
    /// 無い ・ 読めないとき。
    fn read(&self, path: &str) -> Result<String, String>;

    /// フォルダの中の項目の名前（名前の順。フォルダが無ければ空）。
    fn entries(&self, folder: &str) -> Vec<String>;

    /// リンクをたどった先を、打った場所からの経路にしたもの。先が無いか、打った場所の外なら None。
    fn resolve(&self, path: &str) -> Option<String>;
}

/// creator が作るファイル（AGENT.md ・ codex.toml）を書く。agent.json は書かない（SAS が書く）。
pub trait GeneratedFiles {
    /// ファイルを書く。
    ///
    /// # Errors
    /// 書けないとき。
    fn write(&self, path: &str, text: &str) -> Result<(), String>;
}

/// 配布先（ツールの置き場所）を読み、Agent の定義のファイルを写して置き、外す。リンクにしない（ツールが
/// .claude/agents/ ・ .codex/agents/ のリンクのファイルを読むことは原文で確かめていない）。
pub trait Destinations {
    /// 利用者の全体の置き場所の根（HOME）。
    fn home(&self) -> Option<String>;

    /// 経路に何かがあるか（先の無いリンクも、あるとみなす）。
    fn occupied(&self, path: &str) -> bool;

    /// 経路を、リンクをたどって正規化した絶対の経路。先が無ければ None。
    fn absolute(&self, path: &str) -> Option<String>;

    /// 経路がふつうのファイルか（リンクやフォルダでない）。
    fn is_file(&self, path: &str) -> bool;

    /// 経路にふつうのファイルとして中身を書く（親のフォルダが無ければ作る）。
    ///
    /// # Errors
    /// 書けないとき。
    fn place(&self, path: &str, text: &str) -> Result<(), String>;

    /// 経路のふつうのファイルを消す（ふつうのファイルでなければ消さない）。
    ///
    /// # Errors
    /// ふつうのファイルでない ・ 消せないとき。
    fn remove(&self, path: &str) -> Result<(), String>;
}
