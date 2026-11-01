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

    /// value を、$id が schema のスキーマに照らし、形が通った所の注釈を返す。
    /// 1つの要素は、インスタンスの所（JSON Pointer）と、そこに当たったサブスキーマの注釈（キーワード → 値）。
    /// 入れ子 ・ $ref の先 ・ items ・ if/then の先の注釈も、当たったインスタンスの所付きで返る（Core 7.7.1）。
    ///
    /// # Errors
    /// スキーマを組めないとき。
    fn annotations(
        &self,
        schema: &str,
        value: &Value,
        more: &[(String, Value)],
    ) -> Result<Vec<(String, Value)>, String>;
}

/// SAS が起動した子セッションの返り。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reply {
    /// 答えた（tool_ref はツールの出力から読んだ子セッションの ID。answer は文字列のまま）
    Answered {
        /// 子セッション ID
        tool_ref: String,
        /// 答え（SAS が JSON として読んで確かめる）
        answer: String,
    },
    /// 時間内に答えなかった
    Timeout {
        /// 分かれば子セッション ID
        tool_ref: Option<String>,
    },
    /// 同じ子セッションが処理中
    Busy,
    /// 起動できない ・ 失敗した（理由）
    Failed(String),
}

/// 意味を読ませる：子セッションを起動するか続けて、手順のプロンプトを渡し、答えを受け取る。
/// 子セッションを起動する前に、親セッション ID の環境変数を外す（子セッションの中で打たれたものが親セッションの操作に見えないように）。
pub trait ReadMeaning {
    /// ツールの名前（sessions.tool に書く）
    fn tool(&self) -> String;
    /// モデル（分かれば）
    fn model(&self) -> Option<String>;
    /// 新しい子セッションの起動の設定（sessions.launch に写し、--continue で同じものを渡す）
    fn launch(&self) -> Value;
    /// キャッシュの有効時間の目安（秒）。過ぎたら新しい子セッションを勧める
    fn cache_ttl_s(&self) -> Option<u64>;
    /// 新しい子セッションを起動して、プロンプトを渡す。
    fn start(&self, prompt: &str, answer_schema: &Value) -> Reply;
    /// 子セッションを続けて、プロンプトを渡す（launch は起動したときの設定）。
    fn resume(&self, tool_ref: &str, launch: &Value, prompt: &str, answer_schema: &Value) -> Reply;
}
