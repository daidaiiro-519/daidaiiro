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
}
