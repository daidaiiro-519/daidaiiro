//! 書くモジュールが外に求めるもの（書くユースケースが使うポートと、そのうち使うメソッドだけを宣言する。実装は adapters/outbound）。
//! 文書の読み書きと形の検査は、検証のモジュールにも同じ名前のポートがあるが、変わる理由がモジュールごとなので、ここにも宣言する。

use serde_json::Value;
use validation_domain::values::Judgment;
use writing_domain::values::{ChangeRow, StoredChange, WriteRecord};

/// 渡された経路の文書。経路はリポジトリの根（作業ディレクトリ）からの相対で、区切りは /。全部を並べる手段は持たない。
pub trait DocumentStore {
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

/// コミットにある中身を読む（Git）。読むだけで、版を残す ・ コミットするなど Git への書き込みはしない。
/// SAS を通らない変更を取り込むとき、記録の最後の事後と同じ中身のコミットを探して、欄ごとの前の値に使う。
pub trait CommitReader {
    /// 経路の対象を変えたコミットの名前を、新しい順に max 件まで。リポジトリでない ・ コミットが無ければ空。
    ///
    /// # Errors
    /// Git を読めないとき。
    fn commits(&self, path: &str, max: usize) -> Result<Vec<String>, String>;

    /// コミット rev にある経路の対象の中身（作業ツリーに置いたときと同じ変換を当てたバイト列）。そのコミットに無ければ None。
    ///
    /// # Errors
    /// Git を読めないとき。
    fn read(&self, rev: &str, path: &str) -> Result<Option<Vec<u8>>, String>;
}

/// 操作の記録のうち、書くユースケースが使うもの。追記だけを受け付け、ID と時刻は置き場所が作る。
/// 打ったセッションの行（sessions）は入口が先に足すので、ここでは扱わない。
pub trait ChangeLog {
    /// 書いた作成 ・ 更新 ・ 削除を1件足し（変わった欄も一緒に）、その ID を返す。session は打ったセッションの ID（人なら None）。
    ///
    /// # Errors
    /// 置き場所が拒んだとき（同じ request_id ・ 決まりに反する行）。
    fn append_change(&mut self, session: Option<&str>, row: &ChangeRow) -> Result<String, String>;

    /// request_id で足した作成 ・ 更新 ・ 削除。無ければ None。
    ///
    /// # Errors
    /// 置き場所を読めないとき。
    fn find_request(&self, request_id: &str) -> Result<Option<StoredChange>, String>;

    /// 経路の対象を書いた操作（作成か更新）を、記録した順に。
    /// その経路を消した操作があれば、最後に消したより後のものだけ（消すと記録のつながりが切れる）。
    ///
    /// # Errors
    /// 置き場所を読めないとき。
    fn writes(&self, target_path: &str) -> Result<Vec<WriteRecord>, String>;

    /// 経路の対象の、rule ごとの最後の判定（その rule を最後に判定した検証の行）。検証が無ければ空。
    /// 更新のあと、前の判定のうち読んだ所が変わったものを決めるのに使う。
    /// その経路を消した操作があれば、最後に消したより後の検証だけ（消して作り直した文書に前の判定を当てない）。
    ///
    /// # Errors
    /// 置き場所を読めないとき。
    fn last_judgments(&self, target_path: &str) -> Result<Vec<Judgment>, String>;

    /// 記録の最後の変更で、$id が schema の Concrete に従うと書いた文書の経路（消した文書は除く）。
    /// Concrete を直す ・ 消すとき、渡された範囲と合わせて、従うインスタンスを確かめるのに使う。
    ///
    /// # Errors
    /// 置き場所を読めないとき。
    fn instances_of(&self, schema: &str) -> Result<Vec<String>, String>;
}
