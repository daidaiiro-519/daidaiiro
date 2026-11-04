//! 誰が打ったかを決めて sessions に登録し（入口だけが呼ぶ）、記録を読む（UC7）。

use crate::ports::{
    IdentifyCaller, NotIdentified, OperationDetail, OperationFilter, OperationReader,
    OperationSummary, SessionLog, SessionRow,
};
use writing_domain::values::{Caller, Via};

/// 誰が打ったかを決められなかった理由。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unidentified {
    /// 2つ以上のツールの環境変数に値があり、どれが打ったかを決められない（ツールの名前）
    Ambiguous(Vec<String>),
    /// --session ・ ツールの環境変数 ・ 人の明示（--human）のどれも無い
    Missing,
    /// ツールの環境変数があるところで人を名乗った（ツールの名前）
    HumanInTool(Vec<String>),
    /// 置き場所を読めない ・ 書けない
    Log(String),
}

/// 誰が打ったかを決める：人の明示（--human）か、--session → ツールの環境変数。どれも無ければ断る（人とみなさない）。
/// sessions には足さない（足すのは register）。
///
/// # Errors
/// どのツールが打ったかを決められない ・ どれも無いとき。
pub fn identify(who: &dyn IdentifyCaller) -> Result<Caller, Unidentified> {
    who.identify()
        .map_err(not_identified)?
        .ok_or(Unidentified::Missing)
}

/// 呼び出し元を決められなかった理由を、手続きの理由にする。
fn not_identified(e: NotIdentified) -> Unidentified {
    match e {
        NotIdentified::Ambiguous(tools) => Unidentified::Ambiguous(tools),
        NotIdentified::HumanInTool(tools) => Unidentified::HumanInTool(tools),
    }
}

/// 記録をつけるコマンドで、打ったセッションを sessions に登録する。初めて見たセッションだけ1行足す（行は書き換えない）。
/// 人なら何も足さない。
///
/// # Errors
/// 置き場所を読めない ・ 書けないとき。
pub fn register(caller: &Caller, log: &mut dyn SessionLog) -> Result<(), String> {
    log.session(caller).map(|_| ())
}

/// semantic-agent-schema init：新しいセッションの ID を決めて sessions に登録し（via = init）、その行を返す。
/// あとで打つコマンドの既定にはしない（呼ぶ側は、返った ID を --session で渡す）。
///
/// # Errors
/// 置き場所を読めない ・ 書けないとき。
pub fn register_session(
    folder: &str,
    log: &mut dyn SessionLog,
) -> Result<SessionRow, Unidentified> {
    let id = log.new_session_id();
    let tool = "unknown".to_string();
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
