//! abstract スキーマの版。concrete スキーマは `x-abstract` に版の名前を書いて、使う版を示す。
//! `$schema` には、JSON Schema の仕様どおり標準の 2020-12 の URI を書く。

use serde_json::Value;

/// abstract スキーマ v1 の名前（meta-schema の `$id`）。
pub const V1: &str = "schema-driven/abstract@v1";

/// 版の名前の前半。これで始まるのに知らない版は、検査で知らない版として返す。
const PREFIX: &str = "schema-driven/abstract@";

/// concrete スキーマの `$schema` に書く JSON Schema の dialect。
pub const DIALECT: &str = "https://json-schema.org/draft/2020-12/schema";

/// 文書が従うスキーマの名前。concrete スキーマは `$schema` に dialect を、`x-abstract` に abstract の版を書くので、`x-abstract` を返す。
pub fn schema_of(value: &Value) -> Option<&str> {
    let named = value.get("$schema")?.as_str()?;
    if named == DIALECT {
        value.get("x-abstract")?.as_str()
    } else {
        Some(named)
    }
}

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
        serde_json::from_str(crate::embedded::schema_text(V1)).unwrap_or(Value::Null),
    )]
}

/// エンジンが持つスキーマを、名前で返す。インスタンスの `$schema` がこの名前なら、--skill を渡さなくても、ファイルではなくこれで検証する。
pub fn abstract_schema(name: &str) -> Option<Value> {
    crate::embedded::schema(name).and_then(|schema| serde_json::from_str(schema.text).ok())
}

/// エンジンが持つスキーマのファイル名と中身。同じフォルダの $ref を解くために、まとめて渡す。
pub(crate) fn builtin_files() -> Vec<(String, Value)> {
    crate::embedded::SCHEMAS
        .iter()
        .filter_map(|schema| {
            serde_json::from_str(schema.text)
                .ok()
                .map(|value| (schema.file.to_owned(), value))
        })
        .collect()
}

/// エンジンが持つスキーマのファイル名。名前でなければ None。
pub(crate) fn builtin_file(name: &str) -> Option<&'static str> {
    crate::embedded::schema(name).map(|schema| schema.file)
}
