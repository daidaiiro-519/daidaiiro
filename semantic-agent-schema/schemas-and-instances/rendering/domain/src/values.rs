//! 値：テンプレートの書き方の契約のうち描画が使う所 ・ テンプレートの木 ・ JS が値の入口から読んだ所（アダプターが読んで渡し、描画の決まりがたどる）。

use serde_json::Value;

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

/// JS が値の入口から読んだ所の、1つの段。
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

/// テンプレートの書き方の契約（template-language.json）のうち、描画が使う所。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateLanguage {
    /// テンプレートに渡す名前（x-names のキー）
    pub names: Vec<String>,
    /// JS が値を読む入口（x-data-entry）
    pub entry: DataEntry,
}

/// JS が値を読む入口の要素（<script type=… id=…>中身</script>）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataEntry {
    /// type 属性
    pub media_type: String,
    /// id 属性
    pub id: String,
    /// 中身（テンプレートに書く形）
    pub content: String,
}

impl TemplateLanguage {
    /// 契約の文書から読む。x-names か x-data-entry が無い ・ 形が違えば None。
    #[must_use]
    pub fn read(contract: &Value) -> Option<Self> {
        let names = contract
            .get("x-names")?
            .as_object()?
            .keys()
            .cloned()
            .collect();
        let entry = contract.get("x-data-entry")?;
        let text = |k: &str| {
            entry
                .get(k)
                .and_then(Value::as_str)
                .map(ToString::to_string)
        };
        if text("element")?.as_str() != "script" {
            return None;
        }
        Some(Self {
            names,
            entry: DataEntry {
                media_type: text("type")?,
                id: text("id")?,
                content: text("content")?,
            },
        })
    }
}
