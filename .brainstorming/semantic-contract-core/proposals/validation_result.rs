//! ルール1件を検証した結果（案）。どの validator（rule の by が machine ・ ai ・ human）も、この5つのどれかを返す。

/// ルール1件の結果。
pub enum ValidationResult {
    /// 合格。検証した validator の範囲での合格に限る
    Verified,
    /// 違反。場所と、根拠にした値の引用を持つ
    Violated { at: String, quote: String },
    /// 合否を決められなかった。根拠が渡した値に無い AI の答えと、割れた答えもここ
    Unverified { reason: String },
    /// validator が無い（agent の割り当てが無いなど）。必要な capability を返す
    Unsupported { needs: Vec<String> },
    /// 時間切れ ・ 失敗 ・ 形を満たさない答えなど、処理の失敗
    Error { reason: String },
}

/// ルールが必須かどうか。
pub struct Required(pub bool);

/// 必須のルールに合格以外が1件でもあれば、全体を成功にしない。
pub fn overall(results: &[(Required, ValidationResult)]) -> bool {
    results
        .iter()
        .all(|(required, result)| !required.0 || matches!(result, ValidationResult::Verified))
}
