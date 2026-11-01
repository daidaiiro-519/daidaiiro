//! デザインシステムを、受け取りの型とテンプレートの書き方の契約（template-language.json）に照らす。
//!
//! インスタンスを Concrete（JSON Schema）で検証するのと同じく、テンプレートとトークンも契約で機械に確かめる。
//! テンプレートの書き方は Mustache（ロジックを持たない）。UI ・ UX のロジックはテンプレートの JS が持ち、SAS は持たない。
//! 描画は、ここで1つも外れが無いデザインシステムでだけ描く。sas validate でも同じ rule を確かめて記録する。
//!
//! rule：
//! - json-schema：デザインシステムのファイルが受け取りの型に合う
//! - token-type：どのトークンも種類（$type）が決まり、値が種類の形に合う（Design Tokens Format Module 2025.10）
//! - token-alias：トークンの別名がすべて解け、輪になっていない
//! - template-syntax：どのテンプレートも Mustache として読める
//! - template-loads：部分テンプレートの先が templates の種類にある
//! - template-names：Concrete の根の x-view がそのテンプレートを選ぶとき、テンプレート（部分を含む）が置く名前が、
//!   value（その Concrete の形）・ json ・ tokens（このデザインシステムのトークン）・ css のどれかにたどれる
//! - script-entry：値の入口は <script type="application/json" id="sas-data">{{{json}}}</script> だけで、
//!   JS の中に Mustache のタグを書かない。JS として読める
//! - script-reads：Concrete の根の x-view がそのテンプレートを選ぶとき、テンプレート（部分を含む）の JS が入口から読む
//!   欄が、その Concrete の形にたどれる（読む所が実行するまで決まらないものは unverified）

use crate::ports::{DataRead, PathSeg, ScriptReader, TemplateNode, TemplateRenderer};
use crate::use_cases::tokens::{Token, alias_errors, flatten, type_errors};
use recording_application::ports::{IdentifyCaller, OperationLog, RuleRow, ValidateRow};
use recording_application::use_cases::history::record_validations;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use validation_application::ports::{DocumentStore, SchemaValidator};
use validation_application::use_cases::validate::{Corpus, Refused, log_refused, refused};
use validation_domain::values::{By, Counts, RuleResult, Verdict};
use writing_domain::values::ContentHash;

/// デザインシステムの受け取りの型。
pub const DESIGN_SYSTEM: &str = "urn:semantic-agent-schema:published-language:design-system:v1";

/// テンプレートの書き方の契約。
pub const TEMPLATE_LANGUAGE: &str =
    "urn:semantic-agent-schema:published-language:template-language:v1";

/// 読み込んで確かめたデザインシステム。
pub struct DesignSystem {
    /// ファイルの経路
    pub path: String,
    /// 読んだ文字列
    pub text: String,
    /// 読んだ JSON
    pub value: Value,
    /// 種類 → テンプレートの経路（作業ディレクトリから）
    pub template_paths: BTreeMap<String, String>,
    /// 種類 → テンプレートの本文
    pub sources: Vec<(String, String)>,
    /// トークンの名前（group.token）→ トークン
    pub tokens: BTreeMap<String, Token>,
    /// rule ごとの結果と、判定が読んだ所（JSON Pointer。"" は全体）
    pub results: Vec<(RuleResult, String)>,
}

impl DesignSystem {
    /// 外れた rule があるか。
    #[must_use]
    pub fn violated(&self) -> bool {
        self.results
            .iter()
            .any(|(r, _)| r.verdict() == Verdict::Violated)
    }
}

fn verified(rule: &str, pointer: &str, why: &str) -> (RuleResult, String) {
    (
        RuleResult::new(rule, pointer, By::Machine, Verdict::Verified, why),
        pointer.to_string(),
    )
}

fn violated(rule: &str, pointer: &str, why: &str) -> (RuleResult, String) {
    (
        RuleResult::new(rule, pointer, By::Machine, Verdict::Violated, why),
        pointer.to_string(),
    )
}

fn unverified(rule: &str, pointer: &str, why: &str) -> (RuleResult, String) {
    (
        RuleResult::new(rule, pointer, By::Machine, Verdict::Unverified, why),
        pointer.to_string(),
    )
}

fn pointer_escape(s: &str) -> String {
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
fn walk_names(
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

fn partial_names(nodes: &[TemplateNode], out: &mut BTreeSet<String>) {
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
pub const DATA_ENTRY: &str = "sas-data";

/// テンプレートの中の <script> 1つ。
struct ScriptBlock {
    /// 値の入口（type="application/json" かつ id="sas-data"）
    data_entry: bool,
    /// JS でない（type が JSON などの、ほかのデータ）
    json_like: bool,
    /// 外の JS（src を持つ）
    external: bool,
    /// 中身
    body: String,
}

/// テンプレートの本文から <script> を取り出す（属性は type ・ id ・ src だけを見る）。
fn script_blocks(source: &str) -> Vec<ScriptBlock> {
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

enum PathCheck {
    Ok,
    Missing,
    Unknown,
}

/// JS が読む道を、Concrete の形でたどる。並びや文字列に着いたあとの名前（length ・ map など）は JS の組み込みとして見ない。
fn check_path(shape: &Shape, segs: &[PathSeg]) -> PathCheck {
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

fn path_text(segs: &[PathSeg]) -> String {
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

/// デザインシステムを読み、契約に照らす。読めない ・ JSON でないときだけ断り、外れは結果として返す。
///
/// # Errors
/// ファイルが無い（not_found）・ JSON でない（invalid_input）・ バリデーターが動かない（validator）とき。
pub fn check_design_system(
    path: &str,
    store: &dyn DocumentStore,
    validator: &dyn SchemaValidator,
    renderer: &dyn TemplateRenderer,
    scripts: &dyn ScriptReader,
    corpus: &Corpus,
) -> Result<DesignSystem, Refused> {
    let text = store
        .read(path)
        .map_err(|e| refused("not_found", path, e))?;
    let value: Value = serde_json::from_str(&text).map_err(|e| {
        refused(
            "invalid_input",
            path,
            format!("デザインシステムが JSON として読めない：{e}"),
        )
    })?;
    let mut results = Vec::new();

    // 型
    let broken = validator
        .check(DESIGN_SYSTEM, &value, &[])
        .map_err(|e| refused("validator", path, e))?;
    if broken.is_empty() {
        results.push(verified("json-schema", "", "受け取りの型に合う"));
    }
    for (at, why) in &broken {
        results.push(violated("json-schema", at, why));
    }

    // トークン
    let tokens = flatten(value.get("tokens"));
    let types = type_errors(&tokens);
    if types.is_empty() {
        results.push(verified(
            "token-type",
            "/tokens",
            "どのトークンも種類が決まり、値が種類の形（Design Tokens Format Module 2025.10）に合う",
        ));
    }
    for (name, why) in types {
        results.push(violated(
            "token-type",
            &format!("/tokens/{}", name.replace('.', "/")),
            &why,
        ));
    }
    let alias_errors = alias_errors(&tokens);
    if alias_errors.is_empty() {
        results.push(verified(
            "token-alias",
            "/tokens",
            "別名がすべて解け、輪になっていない",
        ));
    }
    for (name, why) in alias_errors {
        results.push(violated(
            "token-alias",
            &format!("/tokens/{}", name.replace('.', "/")),
            &why,
        ));
    }

    // テンプレートを読む
    let templates: BTreeMap<String, String> = value
        .get("templates")
        .and_then(Value::as_object)
        .map(|m| {
            m.iter()
                .map(|(k, v)| (k.clone(), v.as_str().unwrap_or_default().to_string()))
                .collect()
        })
        .unwrap_or_default();
    let base = path.rsplit_once('/').map_or("", |(dir, _)| dir);
    let mut template_paths = BTreeMap::new();
    let mut sources = Vec::new();
    let mut trees: BTreeMap<String, Vec<TemplateNode>> = BTreeMap::new();
    for (kind, rel) in &templates {
        let at = format!("/templates/{}", pointer_escape(kind));
        if rel.starts_with('/') || rel.split('/').any(|p| p == "..") {
            results.push(violated(
                "template-syntax",
                &at,
                &format!("{rel} はデザインシステムのフォルダの中の相対の経路にする"),
            ));
            continue;
        }
        let tpath = if base.is_empty() {
            rel.clone()
        } else {
            format!("{base}/{rel}")
        };
        let source = match store.read(&tpath) {
            Ok(s) => s,
            Err(e) => {
                results.push(violated(
                    "template-syntax",
                    &at,
                    &format!("{tpath} が無い：{e}"),
                ));
                continue;
            }
        };
        match renderer.inspect(&source) {
            Ok(tree) => {
                results.push(verified("template-syntax", &at, "Mustache として読める"));
                trees.insert(kind.clone(), tree);
            }
            Err(e) => results.push(violated("template-syntax", &at, &format!("{tpath}：{e}"))),
        }
        template_paths.insert(kind.clone(), tpath);
        sources.push((kind.clone(), source));
    }

    // 部分テンプレートの先
    for (kind, tree) in &trees {
        let at = format!("/templates/{}", pointer_escape(kind));
        let mut used = BTreeSet::new();
        partial_names(tree, &mut used);
        let bad: Vec<String> = used
            .into_iter()
            .filter(|p| !templates.contains_key(p))
            .collect();
        if bad.is_empty() {
            results.push(verified(
                "template-loads",
                &at,
                "部分テンプレートの先は templates の種類",
            ));
        } else {
            results.push(violated(
                "template-loads",
                &at,
                &format!("templates に無い部分テンプレート：{}", bad.join(" ・ ")),
            ));
        }
    }

    // JS：値の入口と、JS の中のタグ。JS が入口から読む所は種類ごとに集める
    let mut script_reads: BTreeMap<String, Vec<DataRead>> = BTreeMap::new();
    for (kind, source) in &sources {
        if !trees.contains_key(kind) {
            continue;
        }
        let at = format!("/templates/{}", pointer_escape(kind));
        let mut problems = Vec::new();
        let mut reads = Vec::new();
        for s in script_blocks(source) {
            if s.external {
                continue;
            }
            if s.data_entry {
                if s.body.trim() != "{{{json}}}" {
                    problems.push(format!(
                        "値の入口（id=\"{DATA_ENTRY}\"）の中身は {{{{{{json}}}}}} だけにする"
                    ));
                }
                continue;
            }
            if s.json_like {
                continue;
            }
            if s.body.contains("{{") {
                problems.push(
                    "JS の中に Mustache のタグがある。値は sas-data から JSON.parse で読む"
                        .to_string(),
                );
                continue;
            }
            match scripts.data_reads(&s.body, DATA_ENTRY) {
                Ok(f) => reads.extend(f.reads),
                Err(e) => problems.push(e),
            }
        }
        if problems.is_empty() {
            results.push(verified(
                "script-entry",
                &at,
                "値は sas-data から読み、JS にタグが無く、JS として読める",
            ));
        } else {
            results.push(violated("script-entry", &at, &problems.join(" ／ ")));
        }
        script_reads.insert(kind.clone(), reads);
    }

    // 置く名前：根の x-view がこのデザインシステムの種類を選ぶ Concrete ごとに
    let token_shape = Shape::of_tokens(value.get("tokens"));
    let mut reached: BTreeSet<String> = BTreeSet::new();
    for (id, concrete) in &corpus.concretes {
        let doc = concrete.document();
        let kind = doc.get("x-view").and_then(Value::as_str).unwrap_or("page");
        let Some(tree) = trees.get(kind) else {
            continue;
        };
        reached.insert(kind.to_string());
        let mut ps = BTreeSet::new();
        partial_names(tree, &mut ps);
        reached.extend(ps);
        let root = Shape::Object(BTreeMap::from([
            ("value".to_string(), Shape::of_schema(doc)),
            ("json".to_string(), Shape::Scalar),
            ("tokens".to_string(), token_shape.clone()),
            ("css".to_string(), Shape::Scalar),
        ]));
        let (mut missing, mut unknown) = (BTreeSet::new(), BTreeSet::new());
        walk_names(
            tree,
            &mut vec![root],
            &trees,
            &mut vec![kind.to_string()],
            &mut missing,
            &mut unknown,
        );
        let at = format!("/templates/{}", pointer_escape(kind));
        if !missing.is_empty() {
            results.push(violated(
                "template-names",
                &at,
                &format!(
                    "{id} で置けない名前：{}（value は Concrete の欄、ほかは json ・ tokens ・ css）",
                    missing.into_iter().collect::<Vec<_>>().join(" ・ ")
                ),
            ));
        } else if !unknown.is_empty() {
            results.push(unverified(
                "template-names",
                &at,
                &format!(
                    "{id} の形が決まらない所（$ref など）をたどるので確かめられない：{}",
                    unknown.into_iter().collect::<Vec<_>>().join(" ・ ")
                ),
            ));
        } else {
            results.push(verified(
                "template-names",
                &at,
                &format!("{id} の欄と契約の名前だけを置く"),
            ));
        }
        // JS が入口から読む所（部分テンプレートの JS も含む）
        let mut kinds: Vec<String> = vec![kind.to_string()];
        let mut ps2 = BTreeSet::new();
        partial_names(tree, &mut ps2);
        kinds.extend(ps2);
        let value_shape = Shape::of_schema(doc);
        let (mut bad, mut unsure) = (BTreeSet::new(), BTreeSet::new());
        for k in &kinds {
            for r in script_reads.get(k).into_iter().flatten() {
                match r {
                    DataRead::Untraceable(why) => {
                        unsure.insert(why.clone());
                    }
                    DataRead::Path(segs) => match check_path(&value_shape, segs) {
                        PathCheck::Ok => {}
                        PathCheck::Missing => {
                            bad.insert(path_text(segs));
                        }
                        PathCheck::Unknown => {
                            unsure
                                .insert(format!("{}（形が決まらない所をたどる）", path_text(segs)));
                        }
                    },
                }
            }
        }
        if !bad.is_empty() {
            results.push(violated(
                "script-reads",
                &at,
                &format!(
                    "JS が {id} に無い欄を読む：{}",
                    bad.into_iter().collect::<Vec<_>>().join(" ・ ")
                ),
            ));
        } else if !unsure.is_empty() {
            results.push(unverified(
                "script-reads",
                &at,
                &format!(
                    "JS が読む所を確かめられない：{}",
                    unsure.into_iter().collect::<Vec<_>>().join(" ・ ")
                ),
            ));
        } else {
            results.push(verified(
                "script-reads",
                &at,
                &format!("JS は {id} の欄だけを読む"),
            ));
        }
    }
    for kind in trees.keys().filter(|k| !reached.contains(*k)) {
        results.push(unverified(
            "template-names",
            &format!("/templates/{}", pointer_escape(kind)),
            "この種類を選ぶ Concrete もそれを読み込むテンプレートも無いので、置く名前を確かめられない",
        ));
    }

    Ok(DesignSystem {
        path: path.to_string(),
        text,
        value,
        template_paths,
        sources,
        tokens,
        results,
    })
}

/// デザインシステム1つの検証の結果。
#[derive(Debug, Clone, PartialEq)]
pub struct DesignTarget {
    /// 経路
    pub path: String,
    /// 検証したときの中身のハッシュ
    pub hash: ContentHash,
    /// rule ごとの結果
    pub results: Vec<RuleResult>,
    /// 判定の5つの区分ごとの件数
    pub counts: Counts,
    /// 記録した検証の ID
    pub operation: String,
}

/// デザインシステムの検証の結果。
#[derive(Debug, Clone, PartialEq)]
pub struct DesignValidation {
    /// 呼んだ親セッションの ID（人なら None）
    pub parent: Option<String>,
    /// デザインシステムごとの結果（渡した順）
    pub targets: Vec<DesignTarget>,
    /// 全体の件数
    pub counts: Counts,
}

/// UC4 の対象としてのデザインシステム：受け取りの型とテンプレートの書き方の契約に照らし、検証を記録する。
///
/// # Errors
/// デザインシステムが無い ・ JSON でない ・ 契約が無い ・ 記録できないとき。
pub fn validate_design_systems(
    paths: &[String],
    store: &dyn DocumentStore,
    validator: &dyn SchemaValidator,
    renderer: &dyn TemplateRenderer,
    scripts: &dyn ScriptReader,
    who: &dyn IdentifyCaller,
    log: &mut dyn OperationLog,
) -> Result<DesignValidation, Refused> {
    let corpus = Corpus::load(store)?;
    let mut checked = Vec::new();
    for p in paths {
        checked.push(check_design_system(
            p, store, validator, renderer, scripts, &corpus,
        )?);
    }
    let rows: Vec<ValidateRow> = checked
        .iter()
        .map(|d| ValidateRow {
            child_session: None,
            target_path: d.path.clone(),
            hash: ContentHash::of(d.text.as_bytes()).as_str().to_string(),
            results: d
                .results
                .iter()
                .map(|(r, at)| RuleRow {
                    rule: r.rule().to_string(),
                    pointer: r.pointer().to_string(),
                    by: r.by().as_str().to_string(),
                    result: r.verdict().as_str().to_string(),
                    reason: r.reason().to_string(),
                    reads: ContentHash::of_read(at, d.value.pointer(at))
                        .as_str()
                        .to_string(),
                })
                .collect(),
        })
        .collect();
    let recorded = record_validations(who, log, &rows).map_err(log_refused)?;
    let mut total = Counts::default();
    let targets = checked
        .into_iter()
        .zip(recorded.operations)
        .map(|(d, operation)| {
            let results: Vec<RuleResult> = d.results.into_iter().map(|(r, _)| r).collect();
            let counts = Counts::of(&results);
            total.merge(&counts);
            DesignTarget {
                path: d.path,
                hash: ContentHash::of(d.text.as_bytes()),
                results,
                counts,
                operation,
            }
        })
        .collect();
    Ok(DesignValidation {
        parent: recorded.parent,
        targets,
        counts: total,
    })
}
