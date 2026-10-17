//! 決まり1件を確かめた結果（案）。どの担い手（エンジン ・ 決まりごとの検査 ・ AI）も、この5つのどれかを返す。

/// 決まり1件の結果。
pub enum Outcome {
    /// 合格。確かめた担い手の範囲での合格に限る
    Verified,
    /// 違反。決まりの id ・ 場所 ・ 引用を、基盤が機械で確かめたものだけ
    Violated { at: String, quote: String },
    /// 担い手はいるが、合否を決められなかった。AI の「守れている」もここ
    Unverified { reason: String },
    /// 確かめる担い手が無い。要る能力を返す
    Unsupported { needs: Vec<String> },
    /// 時間切れ ・ 読めない応答など、処理の失敗。記録を作らない
    Error { reason: String },
}

/// 決まりが必須かどうか。
pub struct Required(pub bool);

/// 必須の決まりに合格以外が1件でもあれば、全体を成功にしない。
pub fn overall(results: &[(Required, Outcome)]) -> bool {
    results
        .iter()
        .all(|(required, outcome)| !required.0 || matches!(outcome, Outcome::Verified))
}
