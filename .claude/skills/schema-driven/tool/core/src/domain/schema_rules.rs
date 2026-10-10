//! ドメインサービス 具体のスキーマを検査する。
//! 構造の契約は、スキーマの $schema が示す版のメタスキーマ（references/meta-schema.json）で、注釈の形（すべての深さ）・ 文の型 ・ 申告していない注釈 ・
//! 参照される種類の id は、この処理で見る。契約はビルドのときに取り込むので、結果はツールの版で決まる。

use crate::domain::contract::{self, Version};
use crate::domain::schema::Schema;
use serde_json::{json, Value};
use std::collections::BTreeSet;

const ANNOTATIONS: &str = include_str!("../../../../references/annotations.schema.json");
const VIEW: &str = include_str!("../../../../references/view.schema.json");

/// 基盤の注釈。これと、根の x-annotations に申告したもののほかは、違反にする。
const BASE_ANNOTATIONS: [&str; 7] = [
    "x-abstract",
    "x-prompt",
    "x-view",
    "x-ref",
    "x-derive",
    "x-generates",
    "x-annotations",
];

/// 中を検査しないキー（値の例や定数で、スキーマの構造ではない）。
const OPAQUE: [&str; 4] = ["examples", "const", "default", "enum"];

/// 値が「名前 → スキーマ」の対応になるキー。この下のキーは名前で、注釈ではない。
const NAME_MAPS: [&str; 4] = [
    "$defs",
    "properties",
    "patternProperties",
    "dependentSchemas",
];

/// 検査の結果1件。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaFinding {
    /// 具体のスキーマの名前
    pub schema: String,
    /// スキーマの中の場所（JSON Pointer）
    pub at: String,
    /// 破った契約
    pub rule: String,
    pub message: String,
}

fn parse_json(text: &str) -> Value {
    serde_json::from_str(text).unwrap_or(Value::Null)
}

fn pointer_part(key: &str) -> String {
    key.replace('~', "~0").replace('/', "~1")
}

/// 文の型から {…} の式を取り出す。{{ と }} は文字。
fn expressions(template: &str) -> Vec<String> {
    let chars: Vec<char> = template.chars().collect();
    let (mut out, mut i) = (Vec::new(), 0);
    while i < chars.len() {
        let character = chars[i];
        if (character == '{' || character == '}') && chars.get(i + 1) == Some(&character) {
            i += 2;
            continue;
        }
        if character != '{' {
            i += 1;
            continue;
        }
        let (mut depth, mut quote, mut expr) = (1, None::<char>, String::new());
        i += 1;
        while i < chars.len() {
            let inner = chars[i];
            match quote {
                Some(open_quote) if inner == open_quote => quote = None,
                Some(_) => {}
                None if "'`\"".contains(inner) => quote = Some(inner),
                None if inner == '{' => depth += 1,
                None if inner == '}' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                None => {}
            }
            expr.push(inner);
            i += 1;
        }
        out.push(expr.trim().to_owned());
        i += 1;
    }
    out
}

/// x-view が持つ文の型と式（text ・ cases ・ parts ・ label）。式はそのまま {…} で包んで返す。
fn templates(view: &Value) -> Vec<String> {
    let mut found = Vec::new();
    let case = |case: &Value, found: &mut Vec<String>| {
        if let Some(text) = case.get("text").and_then(Value::as_str) {
            found.push(text.to_owned());
        }
        if let Some(when) = case.get("when").and_then(Value::as_str) {
            found.push(format!("{{{when}}}"));
        }
    };
    if let Some(text) = view.get("text").and_then(Value::as_str) {
        found.push(text.to_owned());
    }
    for each_case in view
        .get("cases")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        case(each_case, &mut found);
    }
    for (_, part) in view
        .get("parts")
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
    {
        match part {
            Value::String(template) => found.push(template.clone()),
            Value::Array(cases) => cases
                .iter()
                .for_each(|each_case| case(each_case, &mut found)),
            _ => {}
        }
    }
    if let Some(label) = view.get("label").and_then(Value::as_str) {
        found.push(format!("{{{label}}}"));
    }
    found
}

/// パイプの右が関数の呼び出しでない（|name のような古い絞りの書き方）。
fn old_filter(expr: &str) -> bool {
    let mut rest = expr;
    while let Some(pipe) = rest.find('|') {
        let right = rest[pipe + 1..].trim_start();
        if let Some(after_or) = right.strip_prefix('|') {
            // || は「または」
            rest = after_or;
            continue;
        }
        let id: String = right
            .chars()
            .take_while(|character| character.is_ascii_alphanumeric() || *character == '_')
            .collect();
        if !id.is_empty() {
            let after = right[id.len()..].trim_start();
            if !after.starts_with('(') && !after.starts_with('.') && !after.starts_with('[') {
                return true;
            }
        }
        rest = &rest[pipe + 1..];
    }
    false
}

struct Shapes {
    view: Schema,
    xref: Schema,
    derive: Schema,
    /// v1 の x-prompt の形
    prompt: Schema,
}

fn first_error(shape: &Schema, annotation: &Value) -> Option<String> {
    let validation = shape.validate(annotation).ok()?;
    validation
        .errors
        .first()
        .map(|error| {
            format!("{} {}", error.property(), error.reason())
                .trim()
                .to_owned()
        })
        .or_else(|| {
            validation
                .unfilled
                .first()
                .map(|unfilled| format!("{} が無い", unfilled.property()))
        })
}

#[allow(clippy::too_many_arguments)]
fn walk(
    value: &Value,
    at: &str,
    name: &str,
    declared: &BTreeSet<String>,
    shapes: &Shapes,
    names: bool,
    parse: &dyn Fn(&str) -> Result<(), String>,
    out: &mut Vec<SchemaFinding>,
) {
    let mut push = |at: String, rule: &str, message: String| {
        out.push(SchemaFinding {
            schema: name.to_owned(),
            at,
            rule: rule.to_owned(),
            message,
        })
    };
    match value {
        Value::Object(object) => {
            let mut children = Vec::new();
            for (key, child) in object {
                let here = format!("{at}/{}", pointer_part(key));
                if names {
                    children.push((here, child, false));
                    continue;
                }
                if !key.starts_with("x-") {
                    if !OPAQUE.contains(&key.as_str()) {
                        children.push((here, child, NAME_MAPS.contains(&key.as_str())));
                    }
                    continue;
                }
                let shape = match key.as_str() {
                    "x-view" => Some(&shapes.view),
                    "x-ref" => Some(&shapes.xref),
                    "x-derive" => Some(&shapes.derive),
                    _ => None,
                };
                if let Some(shape) = shape {
                    if let Some(error) = first_error(shape, child) {
                        push(here.clone(), "注釈の形", error);
                    }
                }
                if key == "x-prompt" {
                    if let Some(error) = first_error(&shapes.prompt, child) {
                        push(here.clone(), "注釈の形", error);
                    }
                    let mut seen = BTreeSet::new();
                    let ids = child
                        .get("write")
                        .and_then(Value::as_array)
                        .into_iter()
                        .flatten()
                        .filter_map(|rule| rule.get("id").and_then(Value::as_str));
                    for id in ids {
                        if !seen.insert(id) {
                            push(
                                here.clone(),
                                "決まりの id の重なり",
                                format!("{id} が2回ある"),
                            );
                        }
                    }
                }
                if key == "x-view" && child.is_object() {
                    for template in templates(child) {
                        for expr in expressions(&template) {
                            if old_filter(&expr) {
                                push(here.clone(), "古い絞りの書き方", format!("{{{expr}}} ── パイプの右は関数の呼び出しにする（例：name(@)）"));
                            } else if let Err(error) = parse(&expr) {
                                push(
                                    here.clone(),
                                    "文の型",
                                    format!("{{{expr}}} ── {}", error.lines().next().unwrap_or("")),
                                );
                            }
                        }
                    }
                }
                if !BASE_ANNOTATIONS.contains(&key.as_str()) && !declared.contains(key) {
                    push(
                        here,
                        "申告していない注釈",
                        format!("{key} は基盤の注釈でなく、x-annotations にも無い"),
                    );
                }
            }
            for (here, child, names) in children {
                walk(child, &here, name, declared, shapes, names, parse, out);
            }
        }
        Value::Array(elements) => {
            for (position, element) in elements.iter().enumerate() {
                walk(
                    element,
                    &format!("{at}/{position}"),
                    name,
                    declared,
                    shapes,
                    false,
                    parse,
                    out,
                );
            }
        }
        _ => {}
    }
}

/// x-ref が指す種類を集める。
fn referenced_kinds(value: &Value, out: &mut BTreeSet<String>) {
    match value {
        Value::Object(object) => {
            for (key, child) in object {
                if key == "x-ref" {
                    match child.get("to") {
                        Some(Value::String(kind)) => {
                            out.insert(kind.clone());
                        }
                        Some(Value::Array(kinds)) => {
                            out.extend(kinds.iter().filter_map(Value::as_str).map(str::to_owned))
                        }
                        _ => {}
                    }
                } else if !OPAQUE.contains(&key.as_str()) {
                    referenced_kinds(child, out);
                }
            }
        }
        Value::Array(elements) => elements
            .iter()
            .for_each(|element| referenced_kinds(element, out)),
        _ => {}
    }
}

/// 具体のスキーマの集まりを検査する。parse は、式が JMESPath として読めるかを返す（ポート Query の parse）。
/// 結果は、渡したスキーマの順に、メタスキーマ ・ 注釈 ・ 参照される種類の id の順で並ぶ。
pub fn check_schemas(
    schemas: &[(String, Value)],
    parse: &dyn Fn(&str) -> Result<(), String>,
) -> Vec<SchemaFinding> {
    let meta_v1 = Schema::new("meta-schema.json", parse_json(contract::META_V1), vec![]);
    let sib = vec![
        (
            "annotations.schema.json".to_owned(),
            parse_json(ANNOTATIONS),
        ),
        ("view.schema.json".to_owned(), parse_json(VIEW)),
    ];
    let shape =
        |reference: &str| Schema::new("shape.json", json!({"$ref": reference}), sib.clone());
    let shapes = Shapes {
        view: shape("schema-driven/view@v1#/$defs/x-view"),
        xref: shape("schema-driven/annotations@v1#/$defs/x-ref"),
        derive: shape("schema-driven/annotations@v1#/$defs/x-derive"),
        prompt: Schema::new(
            "shape.json",
            json!({"$ref": format!("{}#/$defs/prompt", contract::V1)}),
            vec![],
        ),
    };
    let mut referenced = BTreeSet::new();
    for (_, schema) in schemas {
        referenced_kinds(schema, &mut referenced);
    }
    referenced.remove("self");
    let mut out = Vec::new();
    for (name, schema) in schemas {
        let version = contract::version_of(schema);
        let meta = match &version {
            Version::V1 => &meta_v1,
            Version::Missing => {
                out.push(SchemaFinding {
                    schema: name.clone(),
                    at: "/x-abstract".into(),
                    rule: "版が無い".into(),
                    message: format!("x-abstract に、従う版 {} を書く", contract::V1),
                });
                continue;
            }
            Version::Unknown(written) => {
                out.push(SchemaFinding {
                    schema: name.clone(),
                    at: "/x-abstract".into(),
                    rule: "知らない版".into(),
                    message: format!(
                        "{written} はこのエンジンが扱う版ではない（扱うのは {}）。新しい版なら、エンジンを新しくする",
                        contract::V1
                    ),
                });
                continue;
            }
        };
        match meta.validate(schema) {
            Ok(validation) => {
                for unfilled in &validation.unfilled {
                    out.push(SchemaFinding {
                        schema: name.clone(),
                        at: unfilled.property().to_owned(),
                        rule: "メタスキーマ".into(),
                        message: format!("{} が無い", unfilled.property()),
                    });
                }
                for error in &validation.errors {
                    out.push(SchemaFinding {
                        schema: name.clone(),
                        at: error.property().to_owned(),
                        rule: "メタスキーマ".into(),
                        message: error.reason().to_owned(),
                    });
                }
            }
            Err(error) => out.push(SchemaFinding {
                schema: name.clone(),
                at: String::new(),
                rule: "メタスキーマ".into(),
                message: format!("検査できない（{}）", error.0),
            }),
        }
        let declared: BTreeSet<String> = schema
            .get("x-annotations")
            .and_then(Value::as_object)
            .map(|annotations| annotations.keys().cloned().collect())
            .unwrap_or_default();
        walk(schema, "", name, &declared, &shapes, false, parse, &mut out);
        let kind = schema
            .pointer("/properties/kind/const")
            .and_then(Value::as_str);
        if let Some(kind) = kind {
            if schema.get("x-generates").is_some()
                && referenced.contains(kind)
                && schema.pointer("/properties/id").is_none()
            {
                out.push(SchemaFinding {
                    schema: name.clone(),
                    at: "/properties/id".into(),
                    rule: "参照される種類の id".into(),
                    message: format!("{kind} はほかのスキーマの x-ref が指すので、id を持つ"),
                });
            }
        }
    }
    out
}
