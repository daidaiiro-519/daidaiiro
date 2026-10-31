//! この業務領域が外に求めるもの。

/// 操作を呼んだのは誰か。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Caller {
    /// 自分のセッションを名乗ったツール（AI のエージェント ・ CI ・ オーケストレーター）。tool_ref はそのツールが付けた名前
    Tool {
        /// ツールの側のセッションの名前
        tool_ref: String,
    },
    /// 名乗らなかった呼び出し。人とみなす
    Human,
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
    /// machine ・ ai ・ human
    pub by: String,
    /// 5つの値のどれか
    pub result: String,
    /// 理由
    pub reason: String,
    /// 判定が読んだ所の指紋（読んだ所と値から作る）。ステップ2 までの記録には無い
    pub reads: Option<String>,
}

/// 検証の操作1件（記録の行）。検証は対象を変えないので、事前と事後の指紋は同じ。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidateRow {
    /// concrete か instance
    pub target_kind: String,
    /// リポジトリの根からの経路
    pub target_path: String,
    /// 事前の指紋
    pub before: String,
    /// 事後の指紋
    pub after: String,
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

/// 作成 ・ 更新 ・ 削除の操作1件（記録の行）。断ったときは refusal に決まった名前と理由が入り、対象は変わらない。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeRow {
    /// create ・ update ・ delete
    pub kind: String,
    /// sas（SAS が書いた）か outside（SAS を通らない変更を取り込んだ）
    pub source: String,
    /// concrete か instance
    pub target_kind: String,
    /// リポジトリの根からの経路
    pub target_path: String,
    /// 従う Concrete（Concrete なら Abstract の版）の $id
    pub concrete: String,
    /// やり直しで同じ操作を2回残さないための ID
    pub request_id: Option<String>,
    /// 事前の指紋（作成では None）
    pub before: Option<String>,
    /// 事後の指紋（断った作成と、削除では None）
    pub after: Option<String>,
    /// 断ったときの決まった名前と理由
    pub refusal: Option<(String, String)>,
    /// 意図（なぜその操作をしたか。呼ぶ側が渡したときだけ）
    pub intent: Option<String>,
    /// この操作を引き起こした操作の ID（削除に伴う参照元の書き換えのとき、その削除）
    pub cause: Option<String>,
    /// 変わった欄
    pub fields: Vec<FieldRow>,
}

/// 記録にある作成 ・ 更新 ・ 削除の操作。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredChange {
    /// 操作の ID
    pub operation: String,
    /// 呼び出し元のセッションの ID
    pub session: String,
    /// 行
    pub row: ChangeRow,
}

/// 記録にある、対象を書いた操作（作成か更新で、断っていないもの）。古い順に並べて使う。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteRecord {
    /// 操作の ID
    pub operation: String,
    /// 呼び出し元のセッションの ID
    pub session: String,
    /// 記録した日時
    pub at: String,
    /// 行
    pub row: ChangeRow,
}

/// 前の検証の判定1件。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Judgment {
    /// 判定を記録した検証の操作の ID
    pub operation: String,
    /// 判定
    pub rule: RuleRow,
}

/// 操作の記録の置き場所。追記だけを受け付ける。ID と時刻は置き場所が作る。
pub trait OperationLog {
    /// 呼び出し元のセッションの ID。開いているものが無ければ作る。
    ///
    /// # Errors
    /// 置き場所を読めない ・ 書けないとき。
    fn caller_session(&mut self, caller: &Caller) -> Result<String, String>;

    /// 検証の操作を1件足し、その ID を返す。
    ///
    /// # Errors
    /// 置き場所が拒んだとき（閉じたセッション ・ 決まりに反する行）。
    fn append_validate(&mut self, session: &str, row: &ValidateRow) -> Result<String, String>;

    /// 作成 ・ 更新 ・ 削除の操作を1件足し（変わった欄も一緒に）、その ID を返す。
    ///
    /// # Errors
    /// 置き場所が拒んだとき（閉じたセッション ・ 同じ request_id ・ 決まりに反する行）。
    fn append_change(&mut self, session: &str, row: &ChangeRow) -> Result<String, String>;

    /// request_id で足した作成 ・ 更新 ・ 削除の操作。無ければ None。
    ///
    /// # Errors
    /// 置き場所を読めないとき。
    fn find_request(&self, request_id: &str) -> Result<Option<StoredChange>, String>;

    /// 操作 operation を原因とする操作（削除に伴う参照元の書き換え）を、記録した順に。
    ///
    /// # Errors
    /// 置き場所を読めないとき。
    fn caused_by(&self, operation: &str) -> Result<Vec<StoredChange>, String>;

    /// 経路の対象を書いた操作（作成か更新で、断っていないもの）を、記録した順に。
    /// その経路を消した操作があれば、最後に消したより後のものだけ（消すと記録のつながりが切れる）。
    ///
    /// # Errors
    /// 置き場所を読めないとき。
    fn writes(&self, target_path: &str) -> Result<Vec<WriteRecord>, String>;

    /// 経路の対象の、rule ごとの最後の判定（その rule を最後に判定した検証の操作の行）。検証が無ければ空。
    /// 入力が変わった rule だけを検証すると、検証の操作ごとに判定した rule が違うので、操作ではなく rule ごとに引く。
    ///
    /// # Errors
    /// 置き場所を読めないとき。
    fn last_judgments(&self, target_path: &str) -> Result<Vec<Judgment>, String>;
}
