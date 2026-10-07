//! 出ていく側のポート。core はファイルにも JMESPath の実装にも直接触れず、この trait を通す。

use crate::domain::values::Hash;

/// ファイルを読めなかった理由（失敗の種類「読めない」）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadError(pub String);

/// ファイルを書けなかった理由。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriteError {
    /// 書く直前の内容のハッシュ値が、期待した値と違う（拒否の理由「ほかの更新と競合した」）
    Conflict,
    /// 失敗の種類「書けない」
    Unwritable(String),
}

/// 書く条件。作成は「まだ無いこと」、更新は「読んだ時点のハッシュ値のままであること」を求める。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriteIf {
    Absent,
    Unchanged(Hash),
}

/// ファイルシステム（支援アクター）。
pub trait Files: Send + Sync {
    fn exists(&self, path: &str) -> bool;
    fn read(&self, path: &str) -> Result<String, ReadError>;
    /// ディレクトリの直下のファイルのパスを、名前の順に返す。
    fn list(&self, dir: &str) -> Result<Vec<String>, ReadError>;
    fn write(&self, path: &str, content: &str, cond: WriteIf) -> Result<(), WriteError>;
    fn remove(&self, path: &str) -> Result<(), WriteError>;
}

/// スキーマの供給元。スキーマが `$ref` で指す先のスキーマを、ファイル名と内容の組で返す。
/// 基盤のアダプタは同じディレクトリから読む。外のスキーマを使いたい Skill は、自分のアダプタで取得して渡す
/// （基盤はネットワークに出ない。ACDR 0122）。
pub trait Schemas: Send + Sync {
    fn referenced(&self, schema_path: &str) -> Result<Vec<(String, String)>, ReadError>;
}

/// JMESPath 式が読めない。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryError(pub String);

/// JMESPath に足す基盤の関数（view.schema.json の functions、ACDR 0129 ・ 0132）。
pub const FUNCTIONS: [&str; 6] = ["name", "label", "map", "view", "part", "quote"];

/// JMESPath に足す関数の名前と中身。基盤の6つは core が持ち、具体はこれを実装して自分の関数を足す（ACDR 0132）。
/// Query のアダプタは、names() の名前を登録して呼び出しを渡すだけにする。
pub trait Functions: Send + Sync {
    fn names(&self) -> Vec<String>;
    fn call(&self, name: &str, args: &[serde_json::Value]) -> Result<serde_json::Value, String>;
}

/// 取得（JMESPath 式で値を取り出す）。差し替えられるように trait にする（ACDR 0122）。
pub trait Query: Send + Sync {
    /// 式を JSON の値に当て、結果を JSON の値で返す。何も指さなければ `null` を返す。
    fn search(
        &self,
        expression: &str,
        json: &serde_json::Value,
    ) -> Result<serde_json::Value, QueryError>;
    /// 式が JMESPath として読めるかだけを見る（評価はしない）。
    fn parse(&self, expression: &str) -> Result<(), QueryError>;
    /// functions の関数（基盤の関数と、具体が足した関数）を追加して、式を JSON の値で評価する。
    fn evaluate(
        &self,
        expression: &str,
        json: &serde_json::Value,
        functions: std::sync::Arc<dyn Functions>,
    ) -> Result<serde_json::Value, QueryError>;
}

/// ページのレイアウトに渡すもの（3c）。題は文字、ほかは基盤が生成した HTML。
#[derive(Debug, Clone, Default)]
pub struct Frame {
    pub title: String,
    pub badges: crate::domain::values::Html,
    pub lead: crate::domain::values::Html,
    pub sections: crate::domain::values::Html,
}

/// 基盤がコンポーネントに渡す API（3c）。式の評価 ・ 要素の描画 ・ プレースホルダーの置換 ・ エスケープを持つ。
pub trait Renderer {
    /// 今の位置に式を当てた値。{"text": …} なら、その文字。
    fn value(&mut self, expr: &serde_json::Value) -> Result<serde_json::Value, String>;
    /// ページテンプレートの要素を、今の位置で描画する。
    fn node(&mut self, node: &serde_json::Value) -> Result<crate::domain::values::Html, String>;
    /// ページテンプレートの要素を、at を今の位置にして描画する（表の列を行ごとに評価するときなど）。
    fn node_at(
        &mut self,
        node: &serde_json::Value,
        at: &serde_json::Value,
    ) -> Result<crate::domain::values::Html, String>;
    /// 部品のプレースホルダーを置換する。プレースホルダーと、渡した名前が1つでも合わなければエラー。
    fn part(
        &self,
        id: &str,
        slots: &[(&str, crate::domain::values::Html)],
    ) -> Result<crate::domain::values::Html, String>;
    /// 文字をエスケープする。
    fn text(&self, text: &str) -> crate::domain::values::Html;
    /// SVG の文字列をページに埋め込む。script ・ イベントの属性（on…）・ javascript: を含むなら断る。
    fn svg(&self, svg: &str) -> Result<crate::domain::values::Html, String>;
    /// ページテンプレートの tones の表を引く。
    fn tone(&self, table: &str, value: &serde_json::Value) -> Option<String>;
}

/// 具体が実装を渡すデザイン（3c。ボード schema-driven-build の論点5、ACDR 0132）。
/// コンポーネントの一覧 ・ 部品の HTML ・ ページと節のレイアウトを具体が持ち、基盤は中身を知らない。
pub trait Design: Send + Sync {
    /// コンポーネントの名前と、入力の形（JSON Schema）。
    fn components(&self) -> Vec<(String, serde_json::Value)>;
    /// 部品の HTML（<template id="…"> の並び。プレースホルダーは {{名前}}）。
    fn parts(&self) -> &str;
    /// ページのレイアウト。
    fn page(
        &self,
        frame: &Frame,
        renderer: &dyn Renderer,
    ) -> Result<crate::domain::values::Html, String>;
    /// 節のレイアウト。
    fn section(
        &self,
        heading: &str,
        body: crate::domain::values::Html,
        renderer: &dyn Renderer,
    ) -> Result<crate::domain::values::Html, String>;
    /// コンポーネント1つを描画する。show と each は基盤が適用してから呼び出す。
    fn render(
        &self,
        name: &str,
        input: &serde_json::Value,
        renderer: &mut dyn Renderer,
    ) -> Result<crate::domain::values::Html, String>;
}
