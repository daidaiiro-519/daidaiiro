//! 書くモジュールのポート（外の技術の側が実装する）。対象の読み書きと形の検査は、検証のモジュールのポートを使う。

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
