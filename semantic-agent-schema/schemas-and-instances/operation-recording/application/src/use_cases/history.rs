//! 操作の記録へ追記する。

use crate::ports::{ChangeRow, IdentifyCaller, OperationLog, ValidateRow, WriteRecord};

/// 追記した結果：呼び出し元のセッションと、足した操作の ID（足した順）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recorded {
    /// セッションの ID
    pub session: String,
    /// 操作の ID
    pub operations: Vec<String>,
}

/// 呼び出し元のセッションを決め、検証の操作を1件ずつ足す。
///
/// # Errors
/// 置き場所がセッションを作れない ・ 行を拒んだとき。それまでに足した行は消さない。
pub fn record_validations(
    who: &dyn IdentifyCaller,
    log: &mut dyn OperationLog,
    rows: &[ValidateRow],
) -> Result<Recorded, String> {
    let session = log.caller_session(&who.identify())?;
    let mut operations = Vec::new();
    for row in rows {
        if row.before != row.after {
            return Err(format!("検証の前後で {} の指紋が違う", row.target_path));
        }
        operations.push(log.append_validate(&session, row)?);
    }
    Ok(Recorded {
        session,
        operations,
    })
}

/// 呼び出し元のセッションを決め、作成か更新の操作を1件足す。
///
/// # Errors
/// 置き場所がセッションを作れない ・ 行を拒んだとき。断った操作の事前と事後の指紋が違うとき。
pub fn record_change(
    who: &dyn IdentifyCaller,
    log: &mut dyn OperationLog,
    row: &ChangeRow,
) -> Result<Recorded, String> {
    if row.refusal.is_some() && row.before != row.after {
        return Err(format!(
            "断った操作で {} の指紋が変わっている",
            row.target_path
        ));
    }
    let session = log.caller_session(&who.identify())?;
    let operation = log.append_change(&session, row)?;
    Ok(Recorded {
        session,
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
    /// いまの中身が、記録した操作の事後のどれでもない（SAS を通らない変更があった）
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
    let reached = operations
        .last()
        .is_some_and(|w| w.row.after.as_deref() == Some(current));
    Ok(Interleaved {
        expected: expected.to_string(),
        current: current.to_string(),
        untracked: !reached,
        last_recorded: writes.last().cloned(),
        operations,
    })
}
