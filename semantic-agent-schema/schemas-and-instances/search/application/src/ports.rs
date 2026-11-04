//! 検索のモジュールが外に求めるもの（検索のユースケースが使うポートだけを宣言する。実装は adapters/outbound）。
//! 検証のモジュールにも同じ名前のポートがあるが、変わる理由がモジュールごとなので、ここにも宣言する。

use serde_json::Value;

/// 渡された経路の文書を読む。経路はリポジトリの根（作業ディレクトリ）からの相対で、区切りは /。全部を並べる手段は持たない。
pub trait DocumentStore {
    /// 文書の中身。
    ///
    /// # Errors
    /// 無い ・ 読めないとき。
    fn read(&self, path: &str) -> Result<String, String>;
}

/// JSON Schema 2020-12 のバリデーター。意味の答えを、答えの形に照らす。
pub trait SchemaValidator {
    /// value を、$id が schema のスキーマに照らし、外れた所（JSON Pointer）と理由を返す。
    /// more は照らすスキーマ（$id と文書）で、schema の名前と $ref の先として引ける。
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
