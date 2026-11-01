//! 操作の記録へ追記する。

use crate::ports::{ChangeRow, IdentifyCaller, OperationLog, ValidateRow, WriteRecord};

/// 追記した結果：呼んだ親セッションと、足した操作の ID（足した順）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recorded {
    /// 呼んだ親セッションの ID（人なら None）
    pub parent: Option<String>,
    /// 操作の ID
    pub operations: Vec<String>,
}

/// 呼んだ親セッションを決め、検証を1件ずつ足す。
///
/// # Errors
/// 置き場所が親セッションの行を書けない ・ 行を拒んだとき。それまでに足した行は消さない。
pub fn record_validations(
    who: &dyn IdentifyCaller,
    log: &mut dyn OperationLog,
    rows: &[ValidateRow],
) -> Result<Recorded, String> {
    let parent = log.parent_session(&who.identify())?;
    let mut operations = Vec::new();
    for row in rows {
        operations.push(log.append_validate(parent.as_deref(), row)?);
    }
    Ok(Recorded { parent, operations })
}

/// 呼んだ親セッションを決め、書いた作成 ・ 更新 ・ 削除を1件足す。断った操作は記録しない。
///
/// # Errors
/// 置き場所が親セッションの行を書けない ・ 行を拒んだとき。
pub fn record_change(
    who: &dyn IdentifyCaller,
    log: &mut dyn OperationLog,
    row: &ChangeRow,
) -> Result<Recorded, String> {
    let parent = log.parent_session(&who.identify())?;
    let operation = log.append_change(parent.as_deref(), row)?;
    Ok(Recorded {
        parent,
        operations: vec![operation],
    })
}

/// 書く前に、呼ぶ側が前提にした指紋といまの指紋が違ったときに返す「間に入った変更」。
/// 呼ぶ側（人か AI）は、間に入った操作の意図と変わった欄を読んで、直し方を考え直す。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Interleaved {
    /// 呼ぶ側が前提にした指紋（--if-match か、最後に記録した書いた操作の事後の指紋）
    pub expected: String,
    /// いまの中身の指紋
    pub current: String,
    /// 前提の指紋のあとに記録された、対象を書いた操作（古い順）
    pub operations: Vec<WriteRecord>,
    /// いまの中身が、この経路の記録の事後のどれでもない（SAS を通らない変更があった）
    pub untracked: bool,
    /// 最後に記録した、対象を書いた操作（無ければ None）
    pub last_recorded: Option<WriteRecord>,
}

/// 対象を最後に書いた操作の事後の指紋。SAS が一度も書いていなければ None。
///
/// # Errors
/// 置き場所を読めないとき。
pub fn last_written(log: &dyn OperationLog, target_path: &str) -> Result<Option<String>, String> {
    Ok(log
        .writes(target_path)?
        .last()
        .and_then(|w| w.row.after.clone()))
}

/// いまの中身が SAS を通らずに変わったなら、比べた最後の事後の指紋を返す。記録が無い ・ 記録のどれかの事後と同じなら None。
///
/// DB はワークツリーで1つなので、ほかのワークツリーが後から書いた事後があっても、いまの中身がこの経路の記録の
/// どれかの事後と同じなら SAS を通った中身とみなす（最後の事後だけと比べると、ワークツリーごとに外の変更に見える）。
///
/// # Errors
/// 置き場所を読めないとき。
pub fn untracked_since(
    log: &dyn OperationLog,
    target_path: &str,
    current: &str,
) -> Result<Option<String>, String> {
    let writes = log.writes(target_path)?;
    if writes
        .iter()
        .any(|w| w.row.after.as_deref() == Some(current))
    {
        return Ok(None);
    }
    Ok(writes.last().and_then(|w| w.row.after.clone()))
}

/// 前提の指紋からいまの指紋までに記録された操作を、事前と事後の指紋をたどって組む。
/// 前提の指紋が記録のどこにも無いときは、記録した操作をすべて返す。
///
/// # Errors
/// 置き場所を読めないとき。
pub fn interleaved(
    log: &dyn OperationLog,
    target_path: &str,
    expected: &str,
    current: &str,
) -> Result<Interleaved, String> {
    let writes = log.writes(target_path)?;
    let start = writes
        .iter()
        .rposition(|w| w.row.after.as_deref() == Some(expected))
        .map(|i| i + 1)
        .or_else(|| {
            writes
                .iter()
                .position(|w| w.row.before.as_deref() == Some(expected))
        })
        .unwrap_or(0);
    let operations: Vec<WriteRecord> = writes[start.min(writes.len())..].to_vec();
    // いまの中身が記録のどれかの事後なら、SAS を通った中身（ほかのワークツリーが書いた事後を含む）
    let reached = writes
        .iter()
        .any(|w| w.row.after.as_deref() == Some(current));
    Ok(Interleaved {
        expected: expected.to_string(),
        current: current.to_string(),
        untracked: !reached,
        last_recorded: writes.last().cloned(),
        operations,
    })
}
