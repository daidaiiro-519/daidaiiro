//! 記録にある、書いた作成 ・ 更新 ・ 削除（記録の行の形）。
//!
//! 書くモジュールは追記し、前の依頼と間に入った変更を引く。検証のモジュールは、SAS を通らない変更を知らせるのに引く。
//! 値は文字列と数で持つ（記録の置き場所が読み書きする形）。

/// 作成 ・ 更新で変わった欄1つ（記録の行）。値は JSON の文字列で、None は値が無いこと。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldRow {
    /// JSON Pointer
    pub pointer: String,
    /// add ・ remove ・ replace
    pub change: String,
    /// 前の値
    pub before: Option<String>,
    /// 後の値
    pub after: Option<String>,
    /// 前の値が取れなかった（SAS を通らない変更の取り込みで、前の中身のコミットが無い）。true なら before は None
    pub before_unavailable: bool,
}

/// 書いた作成 ・ 更新 ・ 削除1件（記録の行）。断った操作は記録しない（理由はその場で返す）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeRow {
    /// create ・ update ・ delete
    pub kind: String,
    /// sas（SAS が書いた）か outside（SAS を通らない変更を取り込んだ）
    pub source: String,
    /// リポジトリの根からの経路
    pub target_path: String,
    /// やり直しで同じ操作を2回残さないための ID
    pub request_id: Option<String>,
    /// 事前のハッシュ（作成では None）
    pub before: Option<String>,
    /// 事後のハッシュ（削除では None）
    pub after: Option<String>,
    /// 理由（なぜその操作をしたか。呼ぶ側が渡したときだけ）
    pub reason: Option<String>,
    /// 書いた文書が従う schema の $id（削除は消す前の文書のもの。記録を読むときに返す。版5より前の行は None）
    pub schema: Option<String>,
    /// 変わった欄
    pub fields: Vec<FieldRow>,
}

/// 記録にある作成 ・ 更新 ・ 削除の操作。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredChange {
    /// 操作の ID
    pub operation: String,
    /// 打ったセッションの ID（人なら None）
    pub session: Option<String>,
    /// 行
    pub row: ChangeRow,
}

/// 記録にある、対象を書いた操作（作成か更新）。古い順に並べて使う。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteRecord {
    /// 操作の ID
    pub operation: String,
    /// 打ったセッションの ID（人なら None）
    pub session: Option<String>,
    /// 記録した日時
    pub at: String,
    /// 行
    pub row: ChangeRow,
}

/// いまの中身が SAS を通らずに変わったなら、比べた最後の事後のハッシュを返す。記録が無い ・ 記録のどれかの事後と同じなら None。
/// writes は経路の対象を書いた操作（記録した順）。
///
/// DB はワークツリーで1つなので、ほかのワークツリーが後から書いた事後があっても、いまの中身がこの経路の記録の
/// どれかの事後と同じなら SAS を通った中身とみなす（最後の事後だけと比べると、ワークツリーごとに外の変更に見える）。
#[must_use]
pub fn unrecorded_since(writes: &[WriteRecord], current: &str) -> Option<String> {
    if writes
        .iter()
        .any(|w| w.row.after.as_deref() == Some(current))
    {
        return None;
    }
    writes.last().and_then(|w| w.row.after.clone())
}

#[cfg(test)]
mod tests {
    use super::{ChangeRow, WriteRecord, unrecorded_since};

    fn write(n: usize, before: &str, after: &str) -> WriteRecord {
        WriteRecord {
            operation: format!("o-{n}"),
            session: None,
            at: "t".into(),
            row: ChangeRow {
                kind: "update".into(),
                source: "sas".into(),
                target_path: "a.json".into(),
                request_id: None,
                before: Some(before.into()),
                after: Some(after.into()),
                reason: None,
                schema: None,
                fields: Vec::new(),
            },
        }
    }

    #[test]
    fn any_recorded_after_is_tracked() {
        assert_eq!(unrecorded_since(&[], "x"), None);
        let writes = [write(1, "a", "b"), write(2, "b", "c")];
        // ほかのワークツリーが c まで書いても、このワークツリーの b は SAS を通った中身
        assert_eq!(unrecorded_since(&writes, "b"), None);
        assert_eq!(unrecorded_since(&writes, "c"), None);
        // 記録のどの事後でもない中身は、最後の事後と比べた外の変更
        assert_eq!(unrecorded_since(&writes, "d").as_deref(), Some("c"));
    }
}
