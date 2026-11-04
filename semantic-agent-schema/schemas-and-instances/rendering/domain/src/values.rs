//! 値：テンプレートの木と、JS が値の入口から読んだ所（アダプターが読んで渡し、描画の決まりがたどる）。

/// テンプレートの節（Mustache の木。文字列は持たない）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateNode {
    /// 値を置く（raw は {{{ }}} か {{& }}）
    Var {
        /// 名前（. か、点でつないだ名前）
        name: String,
        /// エスケープしないか
        raw: bool,
    },
    /// セクション（inverted は {{^ }}）
    Section {
        /// 名前
        name: String,
        /// 反転か
        inverted: bool,
        /// 中身
        children: Vec<Self>,
    },
    /// 部分テンプレートの読み込み
    Partial {
        /// 部分テンプレートの名前（templates の種類）
        name: String,
    },
}

/// JS が値の入口（semantic-agent-schema-data）から読んだ所の、1つの段。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathSeg {
    /// 名前で読む（data.goal の goal）
    Key(String),
    /// 番号で読む（data.calls[0] の 0）
    Index,
}

/// JS が値を読んだ1か所。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataRead {
    /// 入口の値からたどった道（data.calls[0].id → calls ・ 番号 ・ id）
    Path(Vec<PathSeg>),
    /// 読む所が実行するまで決まらない ・ 値をそのまま渡していて、その先を追えない（説明）
    Untraceable(String),
}
