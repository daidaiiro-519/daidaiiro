//! スキーマ（VO-2 が指す JSON Schema）による検証と、x-prompt の取り出し。入出力を持たない。

use crate::domain::values::{Unfilled, ValidationError};
use serde_json::Value;
use std::fmt;

/// 検証結果（情報の別名 TERM-9）。未記入は検証エラーと分けて返す（ACDR 0118）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Validation {
    pub errors: Vec<ValidationError>,
    pub unfilled: Vec<Unfilled>,
}

/// スキーマそのものが壊れていて、検証器を作れない。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaError(pub String);

impl fmt::Display for SchemaError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "スキーマが壊れている：{}", self.0)
    }
}

impl std::error::Error for SchemaError {}

/// 読み込んだスキーマ。本体と、`$ref` の先になる同じディレクトリのスキーマを持つ。
pub struct Schema {
    name: String,
    root: Value,
    siblings: Vec<(String, Value)>,
}

const BASE: &str = "file:///schemas/";

/// 値を、JSON の書き方のまま短い文字にする。
fn shown(value: &Value) -> String {
    value.to_string()
}

/// 並びを「 ・ 」でつなぐ。
fn joined(items: impl IntoIterator<Item = String>) -> String {
    items.into_iter().collect::<Vec<_>>().join(" ・ ")
}

/// 検証エラーの理由を、日本語の文にする。キーワードと、スキーマが決めた値（上限 ・ 候補 ・ 型）は、そのまま残す。
/// エラーの種類は jsonschema の ValidationErrorKind の全部を並べる。版が上がって種類が増えたら、コンパイルが止まる。
fn reason_of(error: &jsonschema::ValidationError) -> String {
    use jsonschema::error::{TypeKind, ValidationErrorKind as Kind};
    let value = shown(error.instance());
    match error.kind() {
        Kind::Type {
            kind: TypeKind::Single(expected),
        } => {
            format!("型が {expected} でない（値：{value}）")
        }
        Kind::Type {
            kind: TypeKind::Multiple(expected),
        } => format!(
            "型が {} のどれでもない（値：{value}）",
            joined(expected.iter().map(|json_type| json_type.to_string()))
        ),
        Kind::Enum { options } => format!(
            "enum の候補（{}）のどれでもない（値：{value}）",
            joined(options.as_array().into_iter().flatten().map(shown))
        ),
        Kind::Constant { expected_value } => {
            format!("const の {} と違う（値：{value}）", shown(expected_value))
        }
        Kind::Required { property } => format!("必須のプロパティ {} が無い", shown(property)),
        Kind::MinLength { limit } => format!("文字数が minLength の {limit} より少ない"),
        Kind::MaxLength { limit } => format!("文字数が maxLength の {limit} より多い"),
        Kind::Minimum { limit } => format!("値が minimum の {limit} より小さい（値：{value}）"),
        Kind::Maximum { limit } => format!("値が maximum の {limit} より大きい（値：{value}）"),
        Kind::ExclusiveMinimum { limit } => {
            format!("値が exclusiveMinimum の {limit} 以下（値：{value}）")
        }
        Kind::ExclusiveMaximum { limit } => {
            format!("値が exclusiveMaximum の {limit} 以上（値：{value}）")
        }
        Kind::MultipleOf { multiple_of } => {
            format!("値が multipleOf の {multiple_of} の倍数でない（値：{value}）")
        }
        Kind::Pattern { pattern } => format!("pattern の {pattern} に合わない（値：{value}）"),
        Kind::Format { format } => format!("format の {format} に合わない（値：{value}）"),
        Kind::MinItems { limit } => format!("要素の数が minItems の {limit} より少ない"),
        Kind::MaxItems { limit } => format!("要素の数が maxItems の {limit} より多い"),
        Kind::AdditionalItems { limit } => {
            format!("要素の数が items で決めた {limit} 個を超えている（additionalItems）")
        }
        Kind::UniqueItems => "同じ要素が2つ以上ある（uniqueItems）".to_owned(),
        Kind::Contains => "contains のスキーマに合う要素が無い".to_owned(),
        Kind::MinProperties { limit } => {
            format!("プロパティの数が minProperties の {limit} より少ない")
        }
        Kind::MaxProperties { limit } => {
            format!("プロパティの数が maxProperties の {limit} より多い")
        }
        Kind::AdditionalProperties { unexpected } => format!(
            "スキーマに無いプロパティがある：{}（additionalProperties）",
            joined(unexpected.iter().cloned())
        ),
        Kind::UnevaluatedProperties { unexpected } => format!(
            "どのスキーマにも無いプロパティがある：{}（unevaluatedProperties）",
            joined(unexpected.iter().cloned())
        ),
        Kind::UnevaluatedItems { unexpected } => format!(
            "どのスキーマにも合わない要素がある：{}（unevaluatedItems）",
            joined(unexpected.iter().cloned())
        ),
        Kind::PropertyNames { error } => {
            format!(
                "プロパティの名前が propertyNames に合わない：{}",
                reason_of(error)
            )
        }
        Kind::AnyOf { .. } => "anyOf のどのスキーマにも合わない".to_owned(),
        Kind::OneOfNotValid { .. } => "oneOf のどのスキーマにも合わない".to_owned(),
        Kind::OneOfMultipleValid { .. } => "oneOf のスキーマの2つ以上に合う".to_owned(),
        Kind::Not { .. } => "not のスキーマに合う（合ってはいけない）".to_owned(),
        Kind::FalseSchema => "スキーマが false なので、どの値も受け付けない".to_owned(),
        Kind::ContentEncoding { content_encoding } => {
            format!("contentEncoding の {content_encoding} に合わない")
        }
        Kind::ContentMediaType { content_media_type } => {
            format!("contentMediaType の {content_media_type} に合わない")
        }
        Kind::FromUtf8 { .. } => "UTF-8 として読めない".to_owned(),
        Kind::Custom { keyword, message } => format!("{keyword}：{message}"),
        Kind::BacktrackLimitExceeded { error } => format!("pattern を評価できない：{error}"),
        Kind::RegexEngineFailure { message } => format!("pattern を評価できない：{message}"),
        Kind::Referencing(error) => format!("$ref の先を読めない：{error}"),
    }
}

impl Schema {
    /// `name` はスキーマのファイル名、`siblings` は同じディレクトリのスキーマ（ファイル名と内容）。
    pub fn new(name: &str, root: Value, siblings: Vec<(String, Value)>) -> Self {
        Self {
            name: name.to_owned(),
            root,
            siblings,
        }
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn root(&self) -> &Value {
        &self.root
    }

    /// インスタンスを検証する。必須のプロパティが無いことは未記入として分ける。
    pub fn validate(&self, instance: &Value) -> Result<Validation, SchemaError> {
        let mut registry = jsonschema::Registry::new();
        for (name, value) in &self.siblings {
            registry = registry
                .add(format!("{BASE}{name}"), value.clone())
                .map_err(|error| SchemaError(error.to_string()))?;
        }
        let registry = registry
            .prepare()
            .map_err(|error| SchemaError(error.to_string()))?;
        let validator = jsonschema::options()
            .offline()
            .with_registry(&registry)
            .with_base_uri(format!("{BASE}{}", self.name))
            .build(&self.root)
            .map_err(|error| SchemaError(error.to_string()))?;
        let mut out = Validation::default();
        for error in validator.iter_errors(instance) {
            let at = error.instance_path().to_string();
            if let jsonschema::error::ValidationErrorKind::Required { property } = error.kind() {
                let name = property
                    .as_str()
                    .map(str::to_owned)
                    .unwrap_or_else(|| property.to_string());
                let pointer = format!("{at}/{}", name.replace('~', "~0").replace('/', "~1"));
                if let Ok(unfilled) = Unfilled::new(&pointer) {
                    out.unfilled.push(unfilled);
                }
            } else if let Ok(validation_error) = ValidationError::new(&at, &reason_of(&error)) {
                out.errors.push(validation_error);
            }
        }
        out.unfilled
            .sort_by(|left, right| left.property().cmp(right.property()));
        Ok(out)
    }

    /// プロパティ（JSON Pointer）の x-prompt を返す。プロパティがスキーマに無いか、x-prompt を持たなければ None。
    pub fn prompt(&self, pointer: &str) -> Option<Value> {
        let mut node = &self.root;
        let mut doc = &self.root;
        for raw in pointer.split('/').skip(1) {
            let segment = raw.replace("~1", "/").replace("~0", "~");
            let (resolved_node, resolved_doc) = self.resolve(node, doc);
            node = resolved_node;
            doc = resolved_doc;
            node = if segment.chars().all(|character| character.is_ascii_digit())
                && node.get("items").is_some()
            {
                node.get("items")?
            } else {
                node.get("properties")?.get(&segment)?
            };
        }
        if pointer.is_empty() {
            return None;
        }
        if let Some(prompt) = node.get("x-prompt") {
            return Some(prompt.clone());
        }
        let (resolved_node, _) = self.resolve(node, doc);
        resolved_node.get("x-prompt").cloned()
    }

    /// `$ref` をたどる。同じファイル（`#/…`）と、同じディレクトリのスキーマ（`名前#/…`）だけを解く。
    pub(crate) fn resolve<'schema>(
        &'schema self,
        mut node: &'schema Value,
        mut doc: &'schema Value,
    ) -> (&'schema Value, &'schema Value) {
        for _ in 0..16 {
            let Some(reference) = node.get("$ref").and_then(Value::as_str) else {
                break;
            };
            let (file, frag) = reference.split_once('#').unwrap_or((reference, ""));
            if !file.is_empty() {
                match self
                    .siblings
                    .iter()
                    .find(|(file_name, _)| file_name == file)
                {
                    Some((_, sibling)) => doc = sibling,
                    None => break,
                }
            }
            match doc.pointer(frag) {
                Some(target) => node = target,
                None => break,
            }
        }
        (node, doc)
    }
}
