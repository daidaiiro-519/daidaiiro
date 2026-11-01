//! この業務領域が外に求めるもの。

/// 打ったセッションの ID をどこから知ったか。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Via {
    /// ツールの環境変数（callers.json の表）
    Env,
    /// --session
    Flag,
    /// このフォルダで sas init が登録したセッション
    Init,
}

impl Via {
    /// 記録と出力に書く名前。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Env => "env",
            Self::Flag => "flag",
            Self::Init => "init",
        }
    }
}

/// 操作を呼んだのは誰か。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Caller {
    /// sas を打ったセッション（AI のエージェント ・ CI ・ オーケストレーター）。--session ・ ツールの環境変数 ・ sas init で分かった
    Session {
        /// セッション ID（ツールのものか、sas init で SAS が決めたもの）
        id: String,
        /// ツールの名前（claude-code など。分からなければ unknown）
        tool: String,
        /// 打ったフォルダ（Git の中ならリポジトリの根、外なら作業フォルダ）
        folder: String,
        /// ID をどこから知ったか
        via: Via,
    },
    /// --session も環境変数も sas init の登録も無い呼び出し。人とみなす
    Human,
}

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

/// 呼び出し元を見分ける。
pub trait IdentifyCaller {
    /// いまの呼び出し元。
    fn identify(&self) -> Caller;
}

/// 検証の1件の rule の結果（記録の行）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleRow {
    /// rule の id
    pub rule: String,
    /// 対象の中の当たる所
    pub pointer: String,
    /// machine ・ ai
    pub by: String,
    /// 判定の5つの区分のどれか
    pub result: String,
    /// 理由
    pub reason: String,
    /// 判定が読んだ所のハッシュ（読んだ所と値から作る）
    pub reads: String,
}

/// 検証1件（記録の行）。検証は対象を変えないので、ハッシュは1つ。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ValidateRow {
    /// リポジトリの根からの経路
    pub target_path: String,
    /// 判定した中身のハッシュ
    pub hash: String,
    /// rule ごとの結果
    pub results: Vec<RuleRow>,
}

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
    /// 意図（なぜその操作をしたか。呼ぶ側が渡したときだけ）
    pub intent: Option<String>,
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

/// 前の検証の判定1件。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Judgment {
    /// 判定を記録した検証の ID
    pub operation: String,
    /// 判定
    pub rule: RuleRow,
}

/// 操作の記録の置き場所。追記だけを受け付ける。ID と時刻は置き場所が作る。
pub trait OperationLog {
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

    /// フォルダで sas init が最後に登録したセッション（via = init）。無ければ None。
    ///
    /// # Errors
    /// 置き場所を読めないとき。
    fn initialized(&self, folder: &str) -> Result<Option<SessionRow>, String>;

    /// 新しいセッション ID（sas init で、ツールから ID が取れないとき）。一意で、時刻だけから作らない。
    fn new_session_id(&self) -> String;

    /// 検証を1件足し（rule ごとの結果も一緒に）、その ID を返す。
    ///
    /// # Errors
    /// 置き場所が拒んだとき（決まりに反する行）。
    fn append_validate(
        &mut self,
        session: Option<&str>,
        row: &ValidateRow,
    ) -> Result<String, String>;

    /// 書いた作成 ・ 更新 ・ 削除を1件足し（変わった欄も一緒に）、その ID を返す。
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
    /// 入力が変わった rule だけを検証すると、検証ごとに判定した rule が違うので、検証ではなく rule ごとに引く。
    /// その経路を消した操作があれば、最後に消したより後の検証だけ（消して作り直した文書に前の判定を当てない）。
    ///
    /// # Errors
    /// 置き場所を読めないとき。
    fn last_judgments(&self, target_path: &str) -> Result<Vec<Judgment>, String>;
}
