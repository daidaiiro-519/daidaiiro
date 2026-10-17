//! ルール1件を検証した結果（案）。どの Interpreter（エンジン ・ ルールごとの検査 ・ AI）も、この5つのどれかを返す。

/// ルール1件の結果。
pub enum Outcome {
    /// 合格。検証した Interpreter の範囲での合格に限る
    Verified,
    /// 違反。ルールの id ・ 場所 ・ 引用を、基盤が機械で検証したものだけ
    Violated { at: String, quote: String },
    /// Interpreter はあるが、合否を決められなかった。AI の「守れている」もここ
    Unverified { reason: String },
    /// Interpreter が無い。必要な Capability を返す
    Unsupported { needs: Vec<String> },
    /// 時間切れ ・ 読めない応答など、処理の失敗。記録を作らない
    Error { reason: String },
}

/// ルールが必須かどうか。
pub struct Required(pub bool);

/// 必須のルールに合格以外が1件でもあれば、全体を成功にしない。
pub fn overall(results: &[(Required, Outcome)]) -> bool {
    results
        .iter()
        .all(|(required, outcome)| !required.0 || matches!(outcome, Outcome::Verified))
}
