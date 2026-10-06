//! ドメインサービス 具体のスキーマを検査する（ボード schema-driven-build の論点6 D）。
//! 構造の契約は references/meta-schema.json で、注釈の形（すべての深さ）・ 文の型 ・ 申告していない注釈 ・
//! 参照される種類の id は、この処理で見る。契約はビルドのときに取り込むので、結果はツールの版で決まる。

use crate::domain::schema::Schema;
use serde_json::{json, Value};
use std::collections::BTreeSet;

const META_SCHEMA: &str = include_str!("../../../../references/meta-schema.json");
const ANNOTATIONS: &str = include_str!("../../../../references/annotations.schema.json");
const VIEW: &str = include_str!("../../../../references/view.schema.json");

/// 基盤の注釈。これと、根の x-annotations に申告したもののほかは、違反にする。
const BASE_ANNOTATIONS: [&str; 6] = [
    "x-prompt",
    "x-view",
    "x-ref",
    "x-derive",
    "x-generates",
    "x-annotations",
];

/// 中を検査しないキー（値の例や定数で、スキーマの構造ではない）。
const OPAQUE: [&str; 4] = ["examples", "const", "default", "enum"];

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

fn parse_json(s: &str) -> Value {
    serde_json::from_str(s).unwrap_or(Value::Null)
}

fn pointer_part(k: &str) -> String {
    k.replace('~', "~0").replace('/', "~1")
}

/// 文の型から {…} の式を取り出す。{{ と }} は文字。
fn expressions(template: &str) -> Vec<String> {
    let chars: Vec<char> = template.chars().collect();
    let (mut out, mut i) = (Vec::new(), 0);
    while i < chars.len() {
        let c = chars[i];
        if (c == '{' || c == '}') && chars.get(i + 1) == Some(&c) {
            i += 2;
            continue;
        }
        if c != '{' {
            i += 1;
            continue;
        }
        let (mut depth, mut quote, mut expr) = (1, None::<char>, String::new());
        i += 1;
        while i < chars.len() {
            let d = chars[i];
            match quote {
                Some(q) if d == q => quote = None,
                Some(_) => {}
                None if "'`\"".contains(d) => quote = Some(d),
                None if d == '{' => depth += 1,
                None if d == '}' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                None => {}
            }
            expr.push(d);
            i += 1;
        }
        out.push(expr.trim().to_owned());
        i += 1;
    }
    out
}

/// x-view が持つ文の型と式（text ・ cases ・ parts ・ label）。式はそのまま {…} で包んで返す。
fn templates(xv: &Value) -> Vec<String> {
    let mut t = Vec::new();
    let case = |c: &Value, t: &mut Vec<String>| {
        if let Some(s) = c.get("text").and_then(Value::as_str) {
            t.push(s.to_owned());
        }
        if let Some(s) = c.get("when").and_then(Value::as_str) {
            t.push(format!("{{{s}}}"));
        }
    };
    if let Some(s) = xv.get("text").and_then(Value::as_str) {
        t.push(s.to_owned());
    }
    for c in xv
        .get("cases")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        case(c, &mut t);
    }
    for (_, p) in xv
        .get("parts")
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
    {
        match p {
            Value::String(s) => t.push(s.clone()),
            Value::Array(a) => a.iter().for_each(|c| case(c, &mut t)),
            _ => {}
        }
    }
    if let Some(s) = xv.get("label").and_then(Value::as_str) {
        t.push(format!("{{{s}}}"));
    }
    t
}

/// パイプの右が関数の呼び出しでない（|name のような古い絞りの書き方）。
fn old_filter(expr: &str) -> bool {
    let mut rest = expr;
    while let Some(p) = rest.find('|') {
        let right = rest[p + 1..].trim_start();
        if let Some(after_or) = right.strip_prefix('|') {
            // || は「または」
            rest = after_or;
            continue;
        }
        let id: String = right
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if !id.is_empty() {
            let after = right[id.len()..].trim_start();
            if !after.starts_with('(') && !after.starts_with('.') && !after.starts_with('[') {
                return true;
            }
        }
        rest = &rest[p + 1..];
    }
    false
}

struct Shapes {
    view: Schema,
    xref: Schema,
    derive: Schema,
}

fn first_error(s: &Schema, v: &Value) -> Option<String> {
    let r = s.validate(v).ok()?;
    r.errors
        .first()
        .map(|e| format!("{} {}", e.property(), e.reason()).trim().to_owned())
        .or_else(|| {
            r.unfilled
                .first()
                .map(|u| format!("{} が無い", u.property()))
        })
}

#[allow(clippy::too_many_arguments)]
fn walk(
    v: &Value,
    at: &str,
    name: &str,
    declared: &BTreeSet<String>,
    shapes: &Shapes,
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
    match v {
        Value::Object(m) => {
            let mut children = Vec::new();
            for (k, c) in m {
                let here = format!("{at}/{}", pointer_part(k));
                if !k.starts_with("x-") {
                    if !OPAQUE.contains(&k.as_str()) {
                        children.push((here, c));
                    }
                    continue;
                }
                let shape = match k.as_str() {
                    "x-view" => Some(&shapes.view),
                    "x-ref" => Some(&shapes.xref),
                    "x-derive" => Some(&shapes.derive),
                    _ => None,
                };
                if let Some(s) = shape {
                    if let Some(e) = first_error(s, c) {
                        push(here.clone(), "注釈の形", e);
                    }
                }
                if k == "x-prompt" {
                    let ok = ["read", "write"].iter().all(|f| {
                        c.get(*f)
                            .and_then(Value::as_str)
                            .is_some_and(|s| !s.is_empty())
                    });
                    if !ok {
                        push(here.clone(), "注釈の形", "read と write が無い".into());
                    }
                }
                if k == "x-view" && c.is_object() {
                    for t in templates(c) {
                        for e in expressions(&t) {
                            if old_filter(&e) {
                                push(here.clone(), "古い絞りの書き方", format!("{{{e}}} ── パイプの右は関数の呼び出しにする（例：name(@)）"));
                            } else if let Err(err) = parse(&e) {
                                push(
                                    here.clone(),
                                    "文の型",
                                    format!("{{{e}}} ── {}", err.lines().next().unwrap_or("")),
                                );
                            }
                        }
                    }
                }
                if !BASE_ANNOTATIONS.contains(&k.as_str()) && !declared.contains(k) {
                    push(
                        here,
                        "申告していない注釈",
                        format!("{k} は基盤の注釈でなく、x-annotations にも無い"),
                    );
                }
            }
            for (here, c) in children {
                walk(c, &here, name, declared, shapes, parse, out);
            }
        }
        Value::Array(a) => {
            for (i, c) in a.iter().enumerate() {
                walk(c, &format!("{at}/{i}"), name, declared, shapes, parse, out);
            }
        }
        _ => {}
    }
}

/// x-ref が指す種類を集める。
fn referenced_kinds(v: &Value, out: &mut BTreeSet<String>) {
    match v {
        Value::Object(m) => {
            for (k, c) in m {
                if k == "x-ref" {
                    match c.get("to") {
                        Some(Value::String(s)) => {
                            out.insert(s.clone());
                        }
                        Some(Value::Array(a)) => {
                            out.extend(a.iter().filter_map(Value::as_str).map(str::to_owned))
                        }
                        _ => {}
                    }
                } else if !OPAQUE.contains(&k.as_str()) {
                    referenced_kinds(c, out);
                }
            }
        }
        Value::Array(a) => a.iter().for_each(|c| referenced_kinds(c, out)),
        _ => {}
    }
}

/// 具体のスキーマの集まりを検査する。parse は、式が JMESPath として読めるかを返す（ポート Query の parse）。
/// 結果は、渡したスキーマの順に、メタスキーマ ・ 注釈 ・ 参照される種類の id の順で並ぶ。
pub fn check_schemas(
    schemas: &[(String, Value)],
    parse: &dyn Fn(&str) -> Result<(), String>,
) -> Vec<SchemaFinding> {
    let meta = Schema::new("meta-schema.json", parse_json(META_SCHEMA), vec![]);
    let sib = vec![
        (
            "annotations.schema.json".to_owned(),
            parse_json(ANNOTATIONS),
        ),
        ("view.schema.json".to_owned(), parse_json(VIEW)),
    ];
    let shape = |r: &str| Schema::new("shape.json", json!({"$ref": r}), sib.clone());
    let shapes = Shapes {
        view: shape("view.schema.json#/$defs/x-view"),
        xref: shape("annotations.schema.json#/$defs/x-ref"),
        derive: shape("annotations.schema.json#/$defs/x-derive"),
    };
    let mut referenced = BTreeSet::new();
    for (_, s) in schemas {
        referenced_kinds(s, &mut referenced);
    }
    referenced.remove("self");
    let mut out = Vec::new();
    for (name, s) in schemas {
        match meta.validate(s) {
            Ok(r) => {
                for u in &r.unfilled {
                    out.push(SchemaFinding {
                        schema: name.clone(),
                        at: u.property().to_owned(),
                        rule: "メタスキーマ".into(),
                        message: format!("{} が無い", u.property()),
                    });
                }
                for e in &r.errors {
                    out.push(SchemaFinding {
                        schema: name.clone(),
                        at: e.property().to_owned(),
                        rule: "メタスキーマ".into(),
                        message: e.reason().to_owned(),
                    });
                }
            }
            Err(e) => out.push(SchemaFinding {
                schema: name.clone(),
                at: String::new(),
                rule: "メタスキーマ".into(),
                message: format!("検査できない（{}）", e.0),
            }),
        }
        let declared: BTreeSet<String> = s
            .get("x-annotations")
            .and_then(Value::as_object)
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default();
        walk(s, "", name, &declared, &shapes, parse, &mut out);
        let kind = s.pointer("/properties/kind/const").and_then(Value::as_str);
        if let Some(k) = kind {
            if s.get("x-generates").is_some()
                && referenced.contains(k)
                && s.pointer("/properties/id").is_none()
            {
                out.push(SchemaFinding {
                    schema: name.clone(),
                    at: "/properties/id".into(),
                    rule: "参照される種類の id".into(),
                    message: format!("{k} はほかのスキーマの x-ref が指すので、id を持つ"),
                });
            }
        }
    }
    out
}
