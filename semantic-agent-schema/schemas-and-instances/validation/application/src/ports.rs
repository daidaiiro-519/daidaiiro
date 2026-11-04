//! 検証のモジュールが外に求めるもの（検証のユースケースが使うポートと、そのうち使うメソッドだけを宣言する。実装は adapters/outbound）。

use serde_json::Value;
use validation_domain::values::{Judgment, ValidateRow};
use writing_domain::values::WriteRecord;

/// 渡された経路の文書を読む。経路は Git の根からの相対で、区切りは /。全部を並べる手段は持たない。
pub trait DocumentStore {
    /// 文書の中身。
    ///
    /// # Errors
    /// 無い ・ 読めないとき。
    fn read(&self, path: &str) -> Result<String, String>;
}

/// JSON Schema 2020-12 のバリデーター。Semantic Agent Schema が入れて出すスキーマ（Abstract の本体 ・ 方言 ・ 語彙）はバリデーターの側が持つ。
pub trait SchemaValidator {
    /// 入れて出すスキーマの文書。無い名前は None。
    fn shipped(&self, id: &str) -> Option<Value>;

    /// value を、$id が schema のスキーマに照らし、外れた所（JSON Pointer）と理由を返す。
    /// more は渡された置き場所から引いた文書（Concrete と $ref の先。$id と文書）で、schema の名前と $ref の先として引ける。
    ///
    /// # Errors
    /// スキーマを組めないとき。
    fn check(
        &self,
        schema: &str,
        value: &Value,
        more: &[(String, Value)],
    ) -> Result<Vec<(String, String)>, String>;
}

/// 操作の記録のうち、検証のユースケースが使うもの。追記だけを受け付け、ID と時刻は置き場所が作る。
/// 打ったセッションの行（sessions）は入口が先に足すので、ここでは扱わない。
pub trait ValidationLog {
    /// 検証を1件足し（rule ごとの結果も一緒に）、その ID を返す。session は打ったセッションの ID（人なら None）。
    ///
    /// # Errors
    /// 置き場所が拒んだとき（決まりに反する行）。
    fn append_validate(
        &mut self,
        session: Option<&str>,
        row: &ValidateRow,
    ) -> Result<String, String>;

    /// 経路の対象を書いた操作（作成か更新）を、記録した順に。
    /// その経路を消した操作があれば、最後に消したより後のものだけ（消すと記録のつながりが切れる）。
    /// Semantic Agent Schema を通らない変更を知らせるのに使う。
    ///
    /// # Errors
    /// 置き場所を読めないとき。
    fn writes(&self, target_path: &str) -> Result<Vec<WriteRecord>, String>;

    /// 経路の対象の、rule ごとの最後の判定（その rule を最後に判定した検証の行）。検証が無ければ空。
    /// 入力が変わった rule だけを検証すると、検証ごとに判定した rule が違うので、検証ではなく rule ごとに引く。
    /// その経路を消した操作があれば、最後に消したより後の検証だけ（消して作り直した文書に前の判定を当てない）。
    ///
    /// # Errors
    /// 置き場所を読めないとき。
    fn last_judgments(&self, target_path: &str) -> Result<Vec<Judgment>, String>;
}
