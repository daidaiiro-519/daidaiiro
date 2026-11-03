//! 記録のモジュールが外に求めるもの（記録を読む UC7 と、入口が使う「誰が打ったかを決めて登録する」手続きが使うポートだけ）。
//! 検証と変更を追記するポートは、追記する書く ・ 検証 ・ 描画のモジュールが、それぞれの ports.rs に宣言する。

use validation_domain::values::ValidateRow;
use writing_domain::values::{Caller, ChangeRow};

/// 記録にあるセッション1つ（sessions の行）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionRow {
    /// セッション ID
    pub id: String,
    /// ツールの名前
    pub tool: String,
    /// 初めて見たときに打ったフォルダ
    pub folder: String,
    /// ID をどこから知ったか（env ・ flag ・ init）
    pub via: String,
}

/// 引数と環境変数から、呼び出し元を見分ける（入口が実装する）。
pub trait IdentifyCaller {
    /// いまの呼び出し元。--session も環境変数も無ければ、initialized（このフォルダで semantic-agent-schema init が最後に登録したセッション）を使い、
    /// それも無ければ人。2つ以上のツールの環境変数に値があって決められないときは、そのツールの名前を並べて返す。
    ///
    /// # Errors
    /// どのツールが打ったかを決められないとき。
    fn identify(&self, initialized: Option<SessionRow>) -> Result<Caller, Vec<String>>;
}

/// 打ったセッションの置き場所（sessions）。追記だけを受け付ける。
pub trait SessionLog {
    /// 打ったセッションの ID。初めて見たセッションなら、sessions に1行足す（行は書き換えない）。人なら None。
    ///
    /// # Errors
    /// 置き場所を読めない ・ 書けないとき。
    fn session(&mut self, caller: &Caller) -> Result<Option<String>, String>;

    /// 記録にあるセッションの行。無ければ None。
    ///
    /// # Errors
    /// 置き場所を読めないとき。
    fn session_row(&self, id: &str) -> Result<Option<SessionRow>, String>;

    /// フォルダで semantic-agent-schema init が最後に登録したセッション（via = init）。無ければ None。
    ///
    /// # Errors
    /// 置き場所を読めないとき。
    fn initialized(&self, folder: &str) -> Result<Option<SessionRow>, String>;

    /// 新しいセッション ID（semantic-agent-schema init で、ツールから ID が取れないとき）。一意で、時刻だけから作らない。
    fn new_session_id(&self) -> String;
}

/// 記録を読むときの絞り方（UC7）。どれも「かつ」で、None ・ 空は絞らない。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OperationFilter {
    /// 対象の経路（どれかに当たる操作）
    pub paths: Vec<String>,
    /// 打ったセッションの ID
    pub by: Option<String>,
    /// 打ったセッションが初めて見えたフォルダ（sessions の folder。正規化した絶対の経路）
    pub folder: Option<String>,
    /// この時刻（UTC の RFC 3339。前の部分だけでもよい）より後か同じ
    pub since: Option<String>,
    /// 新しいものから何件まで
    pub limit: Option<usize>,
}

/// 記録にある操作1件の要約（一覧の1行）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationSummary {
    /// 操作の ID
    pub operation: String,
    /// 記録した日時
    pub at: String,
    /// create ・ update ・ delete ・ validate
    pub op: String,
    /// 対象の経路
    pub path: String,
    /// 打ったセッション（人なら None）
    pub session: Option<SessionRow>,
    /// 理由（変更で、呼ぶ側が渡したときだけ）
    pub reason: Option<String>,
    /// 検証なら、判定の区分ごとの件数（区分の名前 → 件数）。変更なら空
    pub counts: Vec<(String, usize)>,
}

/// 記録にある操作1件の中身。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationDetail {
    /// 作成 ・ 更新 ・ 削除
    Change {
        /// 操作の ID
        operation: String,
        /// 記録した日時
        at: String,
        /// 打ったセッションの ID（人なら None）
        session: Option<String>,
        /// 行（変わった欄も）
        row: ChangeRow,
    },
    /// 検証
    Validate {
        /// 操作の ID
        operation: String,
        /// 記録した日時
        at: String,
        /// 打ったセッションの ID（人なら None）
        session: Option<String>,
        /// 行（rule ごとの結果も）
        row: ValidateRow,
    },
}

/// 操作の記録を読む（UC7）。読むだけで、何も足さない。
pub trait OperationReader {
    /// 絞り方に当たる操作（変更と検証）を、新しい順に。
    ///
    /// # Errors
    /// 置き場所を読めないとき。
    fn operations(&self, filter: &OperationFilter) -> Result<Vec<OperationSummary>, String>;

    /// 操作1件の中身。無い ID は None。
    ///
    /// # Errors
    /// 置き場所を読めないとき。
    fn operation(&self, id: &str) -> Result<Option<OperationDetail>, String>;
}
