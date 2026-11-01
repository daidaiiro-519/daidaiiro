//! 記録にある検証と、rule ごとの判定（記録の行の形）。
//!
//! 検証のモジュールと、デザインシステムを検証する描画のモジュールが追記する。入力が変わった rule だけの検証と、
//! 書くモジュールの更新のあとの判定し直しが要るものは、rule ごとの最後の判定（Judgment）を引いて決める。
//! 値は文字列で持つ（記録の置き場所が読み書きする形）。

/// 検証の1件の rule の結果（記録の行）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleRow {
    /// rule の id
    pub rule: String,
    /// 対象の中の当たる所
    pub pointer: String,
    /// machine ・ ai
    pub by: String,
    /// 判定の5つの区分のどれか
    pub result: String,
    /// 理由
    pub reason: String,
    /// 判定が読んだ所のハッシュ（読んだ所と値から作る）
    pub reads: String,
}

/// 検証1件（記録の行）。検証は対象を変えないので、ハッシュは1つ。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ValidateRow {
    /// リポジトリの根からの経路
    pub target_path: String,
    /// 判定した中身のハッシュ
    pub hash: String,
    /// 検証した文書が従う schema の $id（記録を読むときに返す。版5より前の行は None）
    pub schema: Option<String>,
    /// rule ごとの結果
    pub results: Vec<RuleRow>,
}

/// 前の検証の判定1件。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Judgment {
    /// 判定を記録した検証の ID
    pub operation: String,
    /// 判定を記録した検証が判定した中身のハッシュ
    pub hash: String,
    /// 判定
    pub rule: RuleRow,
}
