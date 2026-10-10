//! abstract スキーマの版。concrete スキーマは `x-abstract` に版の名前を書いて、使う版を示す。
//! `$schema` には、JSON Schema の仕様どおり標準の 2020-12 の URI を書く。

use serde_json::Value;

/// abstract スキーマ v1 の名前（meta-schema の `$id`）。
pub const V1: &str = "schema-driven/abstract@v1";

/// 版の名前の前半。これで始まるのに知らない版は、検査で知らない版として返す。
const PREFIX: &str = "schema-driven/abstract@";

/// v1 のメタスキーマ。
pub(crate) const META_V1: &str = include_str!("../../../../references/meta-schema.json");

/// 固有のツールの宣言の形の名前（tools.schema.json の `$id`）。
pub const TOOLS_V1: &str = "schema-driven/tools@v1";

/// 固有のツールの宣言の形。
pub(crate) const TOOLS_SCHEMA_V1: &str = include_str!("../../../../references/tools.schema.json");

/// concrete スキーマが従う版。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Version {
    V1,
    /// 版を示さない（`x-abstract` が無いか、abstract の版の名前ではない）
    Missing,
    /// 版の名前の形をしているが、このエンジンは扱えない
    Unknown(String),
}

/// スキーマの `x-abstract` から版を決める。
pub fn version_of(schema: &Value) -> Version {
    match schema.get("x-abstract").and_then(Value::as_str) {
        Some(V1) => Version::V1,
        Some(other) if other.starts_with(PREFIX) => Version::Unknown(other.to_owned()),
        _ => Version::Missing,
    }
}

/// 検証器の Registry に入れる、版ごとのメタスキーマ（ファイル名と内容）。
pub(crate) fn meta_schemas() -> Vec<(&'static str, Value)> {
    vec![(
        "meta-schema.json",
        serde_json::from_str(META_V1).unwrap_or(Value::Null),
    )]
}

/// エンジンが持つスキーマ（名前 ・ ファイル名 ・ 中身）。インスタンスの `$schema` がこの名前なら、--skill を渡さなくても、ファイルではなくこれで検証する。
const BUILTINS: [(&str, &str, &str); 9] = [
    (TOOLS_V1, "tools.schema.json", TOOLS_SCHEMA_V1),
    (
        "schema-driven/annotations@v1",
        "annotations.schema.json",
        include_str!("../../../../references/annotations.schema.json"),
    ),
    (
        "schema-driven/catalog@v1",
        "catalog.schema.json",
        include_str!("../../../../references/catalog.schema.json"),
    ),
    (
        "schema-driven/design-system@v1",
        "design-system.schema.json",
        include_str!("../../../../references/design-system.schema.json"),
    ),
    (
        "schema-driven/document@v1",
        "document.schema.json",
        include_str!("../../../../references/document.schema.json"),
    ),
    (
        "schema-driven/implementation@v1",
        "implementation.schema.json",
        include_str!("../../../../references/implementation.schema.json"),
    ),
    (
        "schema-driven/page@v1",
        "page.schema.json",
        include_str!("../../../../references/page.schema.json"),
    ),
    (
        "schema-driven/published@v1",
        "published.schema.json",
        include_str!("../../../../references/published.schema.json"),
    ),
    (
        "schema-driven/view@v1",
        "view.schema.json",
        include_str!("../../../../references/view.schema.json"),
    ),
];

/// エンジンが持つスキーマを、名前で返す。
pub fn abstract_schema(name: &str) -> Option<Value> {
    BUILTINS
        .iter()
        .find(|(builtin, _, _)| *builtin == name)
        .and_then(|(_, _, text)| serde_json::from_str(text).ok())
}

/// エンジンが持つスキーマのファイル名と中身。同じフォルダの $ref を解くために、まとめて渡す。
pub(crate) fn builtin_files() -> Vec<(String, Value)> {
    BUILTINS
        .iter()
        .filter_map(|(_, file, text)| {
            serde_json::from_str(text)
                .ok()
                .map(|value| ((*file).to_owned(), value))
        })
        .collect()
}

/// エンジンが持つスキーマのファイル名。名前でなければ None。
pub(crate) fn builtin_file(name: &str) -> Option<&'static str> {
    BUILTINS
        .iter()
        .find(|(builtin, _, _)| *builtin == name)
        .map(|(_, file, _)| *file)
}
