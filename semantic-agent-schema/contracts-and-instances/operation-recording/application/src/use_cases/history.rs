//! 操作の記録へ追記する。

use crate::ports::{IdentifyCaller, OperationLog, ValidateRow};

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
