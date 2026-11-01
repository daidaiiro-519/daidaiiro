//! 誰が打ったかを決めて sessions に登録し（入口だけが呼ぶ）、記録を読む（UC7）。

use crate::ports::{
    IdentifyCaller, OperationDetail, OperationFilter, OperationReader, OperationSummary,
    SessionLog, SessionRow,
};
use writing_domain::values::{Caller, Via};

/// 誰が打ったかを決められなかった理由。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unidentified {
    /// 2つ以上のツールの環境変数に値があり、どれが打ったかを決められない（ツールの名前）
    Ambiguous(Vec<String>),
    /// 置き場所を読めない ・ 書けない
    Log(String),
}

/// 誰が打ったかを決める：--session → ツールの環境変数 → このフォルダ（folder）で sas init が最後に登録したセッション → 人。
/// sessions には足さない（足すのは register）。
///
/// # Errors
/// どのツールが打ったかを決められない ・ 置き場所を読めないとき。
pub fn identify(
    who: &dyn IdentifyCaller,
    folder: &str,
    log: &dyn SessionLog,
) -> Result<Caller, Unidentified> {
    let initialized = log.initialized(folder).map_err(Unidentified::Log)?;
    who.identify(initialized).map_err(Unidentified::Ambiguous)
}

/// 記録をつけるコマンドで、打ったセッションを sessions に登録する。初めて見たセッションだけ1行足す（行は書き換えない）。
/// 人なら何も足さない。
///
/// # Errors
/// 置き場所を読めない ・ 書けないとき。
pub fn register(caller: &Caller, log: &mut dyn SessionLog) -> Result<(), String> {
    log.session(caller).map(|_| ())
}

/// sas init：いまのセッションを、このフォルダの記録の相手として sessions に登録する（via = init）。
/// 呼び出し元のセッション ID（--session か環境変数）があればそれを、無ければ記録の置き場所が ID を決める。
/// 前の登録は使わない。すでに記録にある ID なら、行を書き換えずに、その行を返す（追記だけ）。
///
/// # Errors
/// どのツールが打ったかを決められない ・ 置き場所を読めない ・ 書けないとき。
pub fn register_session(
    who: &dyn IdentifyCaller,
    folder: &str,
    log: &mut dyn SessionLog,
) -> Result<SessionRow, Unidentified> {
    let (id, tool) = match who.identify(None).map_err(Unidentified::Ambiguous)? {
        Caller::Session { id, tool, .. } => (id, tool),
        Caller::Human => (log.new_session_id(), "unknown".to_string()),
    };
    log.session(&Caller::Session {
        id: id.clone(),
        tool,
        folder: folder.to_string(),
        via: Via::Init,
    })
    .map_err(Unidentified::Log)?;
    log.session_row(&id)
        .map_err(Unidentified::Log)?
        .ok_or_else(|| Unidentified::Log(format!("セッション {id} を登録できなかった")))
}

/// UC7 操作の記録を読む：絞り方に当たる操作（変更と検証）を新しい順に返す。読むだけで、セッションの行も足さない。
///
/// # Errors
/// 置き場所を読めないとき。
pub fn read_operations(
    log: &dyn OperationReader,
    filter: &OperationFilter,
) -> Result<Vec<OperationSummary>, String> {
    log.operations(filter)
}

/// UC7 の1件：操作の中身（変わった欄 ・ rule ごとの結果）。無い ID は None（呼ぶ側が「無い」と返す）。
///
/// # Errors
/// 置き場所を読めないとき。
pub fn read_operation(
    log: &dyn OperationReader,
    id: &str,
) -> Result<Option<OperationDetail>, String> {
    log.operation(id)
}
