//! テンプレートが置く名前と、JS が値の入口から読む所を、Concrete の形にたどる決まり（template-language.json の契約）。
//! テンプレートの木と JS の読み方は、ポートのアダプターが読んで渡す（ここは読んだ結果だけを扱う）。

use crate::values::{PathSeg, TemplateNode};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// JSON Pointer の1つの段に入れる名前（~ と / を逃がす）。
#[must_use]
pub fn pointer_escape(s: &str) -> String {
    s.replace('~', "~0").replace('/', "~1")
}

/// テンプレートの名前をたどるための、値の形。
#[derive(Debug, Clone)]
pub enum Shape {
    /// 欄を持つもの
    Object(BTreeMap<String, Shape>),
    /// 並び（中身の形）
    Array(Box<Shape>),
    /// 文字列 ・ 数 ・ 真偽
    Scalar,
    /// 形が決まらない（$ref ・ oneOf など）。この先の名前は確かめられない
    Unknown,
}

impl Shape {
    /// JSON Schema の形から（properties ・ items だけを見る）。
    #[must_use]
    pub fn of_schema(s: &Value) -> Self {
        if let Some(p) = s.get("properties").and_then(Value::as_object) {
            return Self::Object(
                p.iter()
                    .map(|(k, v)| (k.clone(), Self::of_schema(v)))
                    .collect(),
            );
        }
        if let Some(items) = s.get("items") {
            return Self::Array(Box::new(Self::of_schema(items)));
        }
        let kind = s.get("type").and_then(Value::as_str);
        if kind == Some("array") {
            return Self::Array(Box::new(Self::Unknown));
        }
        if kind == Some("object")
            || ["$ref", "oneOf", "anyOf", "allOf"]
                .iter()
                .any(|k| s.get(*k).is_some())
        {
            return Self::Unknown;
        }
        Self::Scalar
    }

    /// 値そのものから（オブジェクトは欄、並びは中身）。
    #[must_use]
    pub fn of_value(v: &Value) -> Self {
        match v {
            Value::Object(m) => Self::Object(
                m.iter()
                    .map(|(k, x)| (k.clone(), Self::of_value(x)))
                    .collect(),
            ),
            Value::Array(a) => {
                Self::Array(Box::new(a.first().map_or(Self::Unknown, Self::of_value)))
            }
            _ => Self::Scalar,
        }
    }

    /// トークンの木から（$ で始まる名前は除く。$value を持つものはトークンで、その値の形）。
    #[must_use]
    pub fn of_tokens(v: Option<&Value>) -> Self {
        let Some(m) = v.and_then(Value::as_object) else {
            return Self::Object(BTreeMap::new());
        };
        Self::Object(
            m.iter()
                .filter(|(k, _)| !k.starts_with('$'))
                .map(|(k, child)| {
                    let shape = if let Some(v) = child.get("$value") {
                        Self::of_value(v)
                    } else {
                        Self::of_tokens(Some(child))
                    };
                    (k.clone(), shape)
                })
                .collect(),
        )
    }
}

/// 名前をたどった結果。
enum Found {
    Shape(Shape),
    Missing,
    Unknown,
}

/// 文脈の積み重ねから名前をたどる（Mustache の引き方と同じ：最初の部分が在る文脈から先をたどる）。
fn lookup(stack: &[Shape], name: &str) -> Found {
    if name == "." {
        return stack
            .last()
            .map_or(Found::Missing, |s| Found::Shape(s.clone()));
    }
    let mut parts = name.split('.');
    let first = parts.next().unwrap_or_default();
    let mut cur = None;
    for frame in stack.iter().rev() {
        match frame {
            Shape::Object(m) => {
                if let Some(s) = m.get(first) {
                    cur = Some(s.clone());
                    break;
                }
            }
            Shape::Unknown => return Found::Unknown,
            _ => {}
        }
    }
    let Some(mut cur) = cur else {
        return Found::Missing;
    };
    for p in parts {
        cur = match cur {
            Shape::Object(m) => match m.get(p) {
                Some(s) => s.clone(),
                None => return Found::Missing,
            },
            Shape::Unknown => return Found::Unknown,
            _ => return Found::Missing,
        };
    }
    Found::Shape(cur)
}

/// テンプレートの木をたどり、たどれない名前（missing）と確かめられない名前（unknown）を集める。
pub fn walk_names(
    nodes: &[TemplateNode],
    stack: &mut Vec<Shape>,
    trees: &BTreeMap<String, Vec<TemplateNode>>,
    visiting: &mut Vec<String>,
    missing: &mut BTreeSet<String>,
    unknown: &mut BTreeSet<String>,
) {
    for n in nodes {
        match n {
            TemplateNode::Var { name, .. } => match lookup(stack, name) {
                Found::Missing => {
                    missing.insert(name.clone());
                }
                Found::Unknown => {
                    unknown.insert(name.clone());
                }
                Found::Shape(_) => {}
            },
            TemplateNode::Section { name, children, .. } => {
                let pushed = match lookup(stack, name) {
                    Found::Missing => {
                        missing.insert(name.clone());
                        Shape::Unknown
                    }
                    Found::Unknown => {
                        unknown.insert(name.clone());
                        Shape::Unknown
                    }
                    Found::Shape(Shape::Array(item)) => *item,
                    Found::Shape(s) => s,
                };
                stack.push(pushed);
                walk_names(children, stack, trees, visiting, missing, unknown);
                stack.pop();
            }
            TemplateNode::Partial { name } => {
                if visiting.contains(name) {
                    continue;
                }
                if let Some(tree) = trees.get(name) {
                    visiting.push(name.clone());
                    walk_names(tree, stack, trees, visiting, missing, unknown);
                    visiting.pop();
                }
            }
        }
    }
}

/// テンプレートの木が読み込む部分テンプレートの名前を、out に集める。
pub fn partial_names(nodes: &[TemplateNode], out: &mut BTreeSet<String>) {
    for n in nodes {
        match n {
            TemplateNode::Partial { name } => {
                out.insert(name.clone());
            }
            TemplateNode::Section { children, .. } => partial_names(children, out),
            TemplateNode::Var { .. } => {}
        }
    }
}

/// JS が値を読む入口の要素の id（template-language.json の x-data-entry）。
pub const DATA_ENTRY: &str = "semantic-agent-schema-data";

/// テンプレートの中の <script> 1つ。
pub struct ScriptBlock {
    /// 値の入口（type="application/json" かつ id="semantic-agent-schema-data"）
    pub data_entry: bool,
    /// JS でない（type が JSON などの、ほかのデータ）
    pub json_like: bool,
    /// 外の JS（src を持つ）
    pub external: bool,
    /// 中身
    pub body: String,
}

/// テンプレートの本文から <script> を取り出す（属性は type ・ id ・ src だけを見る）。
pub fn script_blocks(source: &str) -> Vec<ScriptBlock> {
    let lower = source.to_ascii_lowercase();
    let mut out = Vec::new();
    let mut at = 0;
    while let Some(i) = lower[at..].find("<script") {
        let start = at + i;
        let Some(gt) = lower[start..].find('>') else {
            break;
        };
        let attrs = &source[start + "<script".len()..start + gt];
        let body_start = start + gt + 1;
        let Some(end) = lower[body_start..].find("</script") else {
            break;
        };
        let body = &source[body_start..body_start + end];
        let attr = |name: &str| -> Option<String> {
            let a = attrs.to_ascii_lowercase();
            let p = a.find(&format!("{name}="))?;
            let rest = &attrs[p + name.len() + 1..];
            let q = rest.chars().next()?;
            if q == '"' || q == '\'' {
                rest[1..].find(q).map(|e| rest[1..1 + e].to_string())
            } else {
                Some(
                    rest.split_whitespace()
                        .next()
                        .unwrap_or_default()
                        .to_string(),
                )
            }
        };
        let kind = attr("type").unwrap_or_default().to_ascii_lowercase();
        let json_like = kind.contains("json");
        out.push(ScriptBlock {
            data_entry: json_like && attr("id").as_deref() == Some(DATA_ENTRY),
            json_like,
            external: attr("src").is_some(),
            body: body.to_string(),
        });
        at = body_start + end;
    }
    out
}

/// JS が読む所が、値の形にたどれるか。
pub enum PathCheck {
    /// たどれる
    Ok,
    /// 形に無い
    Missing,
    /// 形が決まらない所をたどる
    Unknown,
}

/// JS が読む道を、Concrete の形でたどる。並びや文字列に着いたあとの名前（length ・ map など）は JS の組み込みとして見ない。
pub fn check_path(shape: &Shape, segs: &[PathSeg]) -> PathCheck {
    let mut cur = shape.clone();
    for seg in segs {
        cur = match (&cur, seg) {
            (Shape::Object(m), PathSeg::Key(k)) => match m.get(k) {
                Some(s) => s.clone(),
                None => return PathCheck::Missing,
            },
            (Shape::Object(_), PathSeg::Index) => return PathCheck::Missing,
            (Shape::Array(item), PathSeg::Index) => (**item).clone(),
            (Shape::Array(item), PathSeg::Key(k)) if k.parse::<usize>().is_ok() => (**item).clone(),
            (Shape::Array(_) | Shape::Scalar, _) => return PathCheck::Ok,
            (Shape::Unknown, _) => return PathCheck::Unknown,
        };
    }
    PathCheck::Ok
}

/// JS が読んだ所を、data.calls[0].id の形の文字列にする。
#[must_use]
pub fn path_text(segs: &[PathSeg]) -> String {
    let mut s = "data".to_string();
    for seg in segs {
        match seg {
            PathSeg::Key(k) => {
                s.push('.');
                s.push_str(k);
            }
            PathSeg::Index => s.push_str("[n]"),
        }
    }
    s
}
