//! 描画のモジュールが外に求めるもの（描画と、デザインシステムの検証のユースケースが使うポートと、そのうち使うメソッドだけを宣言する。実装は adapters/outbound）。
//! 文書を読む ・ 形を照らすポートは、検証のモジュールにも同じ名前のものがあるが、変わる理由がモジュールごとなので、ここにも宣言する。

use serde_json::Value;

/// 渡された経路の文書を読む。経路は Git の根からの相対で、区切りは /。全部を並べる手段は持たない。
pub trait DocumentStore {
    /// 文書の中身。
    ///
    /// # Errors
    /// 無い ・ 読めないとき。
    fn read(&self, path: &str) -> Result<String, String>;
}

/// JSON Schema 2020-12 のバリデーター。デザインシステムを受け取りの型に照らす。
pub trait SchemaValidator {
    /// value を、$id が schema のスキーマに照らし、外れた所（JSON Pointer）と理由を返す。
    /// more は照らすスキーマ（$id と文書）で、schema の名前と $ref の先として引ける。
    ///
    /// # Errors
    /// スキーマを組めないとき。
    fn check(
        &self,
        schema: &str,
        value: &Value,
        more: &[(String, Value)],
    ) -> Result<Vec<(String, String)>, String>;
}

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

/// テンプレートの書き方は Mustache（mustache/spec の必須の部分）。ロジックを持たず、値を置く ・ 繰り返す ・
/// 有無で出し分けるだけ。UI ・ UX のロジックはテンプレートの JS が持つ。
pub trait TemplateRenderer {
    /// templates（種類の名前とテンプレートの本文）を部分テンプレートとして使えるようにし、entry の種類のテンプレートに
    /// context を入れて描く。値は HTML としてエスケープする（{{{ }}} と {{& }} はテンプレートが引き受ける）。
    ///
    /// # Errors
    /// テンプレートの書き方の誤り ・ 部分テンプレートの読み込みが深すぎるとき。
    fn render(
        &self,
        templates: &[(String, String)],
        entry: &str,
        context: &Value,
    ) -> Result<String, String>;

    /// テンプレートを描かずに読み、値を置く所 ・ セクション ・ 部分テンプレートの木を返す。
    ///
    /// # Errors
    /// テンプレートの書き方の誤りのとき。
    fn inspect(&self, source: &str) -> Result<Vec<TemplateNode>, String>;
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

/// JS を読んで分かったこと。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScriptFacts {
    /// 値の入口（semantic-agent-schema-data）から読んでいるか
    pub reads_entry: bool,
    /// 値を読んだ所
    pub reads: Vec<DataRead>,
}

/// テンプレートの JS を構文解析する道具。JS のロジックは見ず、値の入口からどの欄を読んでいるかだけを取り出す。
pub trait ScriptReader {
    /// source（1つの <script> の中身）を読み、entry_id の要素（<script type="application/json" id=…>）から
    /// JSON.parse した値をたどって読んでいる所を返す。
    ///
    /// # Errors
    /// JS として読めないとき。
    fn data_reads(&self, source: &str, entry_id: &str) -> Result<ScriptFacts, String>;
}

/// ページを書き出せなかった理由。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PageRefusal {
    /// 書き出すページが、入力のファイル（実体の経路で比べる）と同じ所を指す
    Overlaps {
        /// ページ（書き出す先からの経路）
        page: String,
        /// 重なった入力のファイル
        input: String,
    },
    /// 書き出す先を作れない ・ 書けない
    Failed(String),
}

/// 描いたページの書き出し先。
pub trait PageWriter {
    /// out の下に、ページ（out からの経路と本文）をすべて置く。1つでも置けなければ、どれも置かず、
    /// 前から在ったページも変えない。ページが inputs（Git の根からの経路）のどれかと同じファイルを
    /// 指せば、何も書かずに断る。
    ///
    /// # Errors
    /// 入力と重なる ・ 書き出す先を作れない ・ 書けないとき。
    fn write_pages(
        &self,
        out: &str,
        pages: &[(String, String)],
        inputs: &[String],
    ) -> Result<(), PageRefusal>;
}
