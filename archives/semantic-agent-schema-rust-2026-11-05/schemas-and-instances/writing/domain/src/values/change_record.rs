//! 記録にある、書いた作成 ・ 更新 ・ 削除（記録の行の形）。
//!
//! 書くモジュールは追記し、間に入った変更を引く。検証のモジュールは、Semantic Agent Schema を通らない変更を知らせるのに引く。
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
    /// 前の値が無い（Semantic Agent Schema を通らない変更の取り込みで、--base を渡さなかった）。true なら before は None
    pub before_unavailable: bool,
}

/// 書いた作成 ・ 更新 ・ 削除1件（記録の行）。断った操作は記録しない（理由はその場で返す）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeRow {
    /// create ・ update ・ delete
    pub kind: String,
    /// semantic-agent-schema（Semantic Agent Schema が書いた）か outside（Semantic Agent Schema を通らない変更を取り込んだ）
    pub source: String,
    /// リポジトリの根からの経路
    pub target_path: String,
    /// 事前のハッシュ（作成では None）
    pub before: Option<String>,
    /// 事後のハッシュ（削除では None）
    pub after: Option<String>,
    /// 理由（なぜその操作をしたか。呼ぶ側が渡したときだけ）
    pub reason: Option<String>,
    /// 書いた文書が従う schema の $id（削除は消す前の文書のもの。記録を読むときに返す）
    pub schema: Option<String>,
    /// 変わった欄
    pub fields: Vec<FieldRow>,
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
    /// いま打っているワークツリー（作業ツリーの根）で記録した操作か。ワークツリーの無い行は false
    pub same_worktree: bool,
    /// 行
    pub row: ChangeRow,
}

/// いまの中身が Semantic Agent Schema を通らずに変わったなら、比べた記録の操作（その事後が比べたハッシュ）を返す。外の変更が無ければ None。
/// writes は経路の対象を書いた操作（記録した順）。
///
/// 記録はワークツリーで1つなので、ほかのワークツリーの書き込みとは比べない：
/// - このワークツリーで書いた操作があれば、その最後の事後と比べる（前の事後へ戻した中身も外の変更）
/// - まだ書いていなければ（作ったばかりのワークツリー ・ ブランチを移った直後）、中身は Git から来たので、
///   その経路の記録のどれかの中身（事前か事後）と同じなら外の変更でない。違えば、記録の最後の操作と比べた外の変更
#[must_use]
pub fn unrecorded_since<'a>(writes: &'a [WriteRecord], current: &str) -> Option<&'a WriteRecord> {
    if let Some(own) = writes.iter().rev().find(|w| w.same_worktree) {
        return (own.row.after.as_deref() != Some(current)).then_some(own);
    }
    let seen = writes.iter().any(|w| {
        w.row.after.as_deref() == Some(current) || w.row.before.as_deref() == Some(current)
    });
    if seen { None } else { writes.last() }
}

#[cfg(test)]
mod tests {
    use super::{ChangeRow, WriteRecord, unrecorded_since};

    fn write(n: usize, before: &str, after: &str, same_worktree: bool) -> WriteRecord {
        WriteRecord {
            operation: format!("o-{n}"),
            session: None,
            at: "t".into(),
            same_worktree,
            row: ChangeRow {
                kind: "update".into(),
                source: "semantic-agent-schema".into(),
                target_path: "a.json".into(),
                before: Some(before.into()),
                after: Some(after.into()),
                reason: None,
                schema: None,
                fields: Vec::new(),
            },
        }
    }

    fn compared(writes: &[WriteRecord], current: &str) -> Option<String> {
        unrecorded_since(writes, current).and_then(|w| w.row.after.clone())
    }

    #[test]
    fn this_worktree_compares_with_its_last_after() {
        assert_eq!(compared(&[], "x"), None);
        // このワークツリーで b、ほかのワークツリーで c まで書いた
        let writes = [write(1, "a", "b", true), write(2, "b", "c", false)];
        assert_eq!(compared(&writes, "b"), None);
        // ほかのワークツリーの事後でも、このワークツリーの最後の事後と違えば外の変更
        assert_eq!(compared(&writes, "c").as_deref(), Some("b"));
        // 前の事後へ戻した中身も外の変更
        let writes = [write(1, "a", "b", true), write(2, "b", "c", true)];
        assert_eq!(compared(&writes, "b").as_deref(), Some("c"));
    }

    #[test]
    fn a_worktree_without_writes_compares_with_any_recorded_content() {
        // ほかのワークツリーだけが a から b へ書いた。Git から来た a ・ b は外の変更でない
        let writes = [write(1, "a", "b", false)];
        assert_eq!(compared(&writes, "a"), None);
        assert_eq!(compared(&writes, "b"), None);
        assert_eq!(compared(&writes, "d").as_deref(), Some("b"));
    }
}
