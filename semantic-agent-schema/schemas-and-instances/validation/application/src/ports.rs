//! この業務領域が外に求めるもの。

use serde_json::Value;

/// リポジトリの中の文書。経路はリポジトリの根（作業ディレクトリ）からの相対で、区切りは /。
pub trait DocumentStore {
    /// 検証の材料になりうる JSON の文書の経路（並びは経路の順）。
    ///
    /// # Errors
    /// 置き場所を読めないとき。
    fn list(&self) -> Result<Vec<String>, String>;

    /// 文書の中身。
    ///
    /// # Errors
    /// 無い ・ 読めないとき。
    fn read(&self, path: &str) -> Result<String, String>;

    /// 文書を書く。途中で止まっても、前の中身か新しい中身のどちらかが残る（一時ファイルに書いて置き換える）。
    /// 書くのは SAS の use_cases だけ。
    ///
    /// # Errors
    /// 経路がリポジトリの外か SAS の記録を指す ・ 書けないとき。
    fn write(&self, path: &str, text: &str) -> Result<(), String>;

    /// 文書を消す。消すのは SAS の use_cases だけ。
    ///
    /// # Errors
    /// 経路がリポジトリの外か SAS の記録を指す ・ 無い ・ 消せないとき。
    fn remove(&self, path: &str) -> Result<(), String>;
}

/// JSON Schema 2020-12 のバリデーター。SAS が入れて出すスキーマ（Abstract の本体 ・ 方言 ・ 語彙）はバリデーターの側が持つ。
pub trait SchemaValidator {
    /// 入れて出すスキーマの文書。無い名前は None。
    fn shipped(&self, id: &str) -> Option<Value>;

    /// value を、$id が schema のスキーマに照らし、外れた所（JSON Pointer）と理由を返す。
    /// more はリポジトリの Concrete（$id と文書）で、schema の名前と $ref の先として引ける。
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
