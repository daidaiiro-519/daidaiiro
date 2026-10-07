//! ドメインサービス 検査する（DS-1）。インスタンスの集まりについて、注釈 x-ref と x-derive を
//! references/annotations.schema.json の仕様どおりに適用する。読むのは注釈と、インスタンスの id と kind だけ。

use crate::domain::schema::Schema;
use crate::domain::values::{Derived as DerivedValue, Hash};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

pub use crate::domain::values::Status;

/// 検査結果1件（VO-9）。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub status: Status,
    pub check: String,
    pub from: String,
    pub to: String,
    pub message: String,
}

/// 検査する対象のインスタンス1件。
pub struct Doc<'schema> {
    pub path: String,
    pub value: Value,
    pub hash: Hash,
    pub schema: &'schema Schema,
}

/// 承認記録の中身（パスごとのハッシュ値）。
pub type Approved = BTreeMap<String, Hash>;

/// 指す先の項目（インスタンス、またはその中の項目）。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Target {
    doc: usize,
    item: Option<String>,
}

struct Link {
    doc: usize,
    at: String,
    owner: Value,
    owner_at: String,
    annot: String,
    xref: Value,
    value: String,
    targets: Vec<Target>,
}

struct Derived {
    doc: usize,
    at: String,
    value: Value,
    title: Value,
    declared: Value,
    questions: Vec<Value>,
}

/// 参照1件と、その指す先（描画の文脈の links ・ referrers の元）。
#[derive(Debug, Clone, PartialEq)]
pub struct GraphLink {
    /// 参照を書いたインスタンスの番号
    pub doc: usize,
    /// 参照を書いた場所（インスタンスの根から）
    pub at: String,
    /// 書いた値
    pub value: String,
    /// 指す先のインスタンスの番号と、項目の id。決まらなければ None
    pub target: Option<(usize, Option<String>)>,
    /// 指す先の名前（「インスタンスの id」か「インスタンスの id.項目の id」）
    pub label: Option<String>,
}

/// x-derive を持つオブジェクト1つの導出値（描画の文脈の derived の元）。
#[derive(Debug, Clone, PartialEq)]
pub struct GraphDerived {
    pub doc: usize,
    pub at: String,
    pub value: Value,
    pub title: Value,
    pub declared: Value,
    pub questions: Vec<Value>,
}

/// インスタンスの集まりの参照と導出値。検査と描画が同じ解決を使う。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Graph {
    pub links: Vec<GraphLink>,
    pub derived: Vec<GraphDerived>,
}

/// 場所（path）の値を集める。配列は要素すべてに広げる。
pub(crate) fn collect<'value>(value: &'value Value, path: &str) -> Vec<&'value Value> {
    let mut current: Vec<&Value> = flatten(vec![value]);
    for segment in path.split('/').filter(|segment| !segment.is_empty()) {
        current = flatten(
            current
                .into_iter()
                .filter_map(|item| item.get(segment))
                .collect(),
        );
    }
    current
}

fn flatten(list: Vec<&Value>) -> Vec<&Value> {
    let mut out = Vec::new();
    for value in list {
        match value {
            Value::Array(elements) => out.extend(flatten(elements.iter().collect())),
            other => out.push(other),
        }
    }
    out
}

/// field_condition：すべての場所の値が、それぞれの一覧のどれかと同じなら当たる。
fn condition_holds(base: &Value, cond: &Value) -> bool {
    let Some(map) = cond.as_object() else {
        return true;
    };
    map.iter().all(|(path, allowed)| {
        let allowed = allowed.as_array().cloned().unwrap_or_default();
        collect(base, path)
            .iter()
            .any(|value| allowed.contains(value))
    })
}

fn matches_pattern(pattern: &str, value: &str) -> bool {
    jsonschema::is_valid(&json!({"pattern": pattern}), &json!(value))
}

fn id_of(value: &Value) -> Option<&str> {
    value.get("id").and_then(Value::as_str)
}

fn kinds(xref: &Value) -> Vec<String> {
    match xref.get("to") {
        Some(Value::String(kind)) => vec![kind.clone()],
        Some(Value::Array(kinds)) => kinds
            .iter()
            .filter_map(|kind| kind.as_str().map(str::to_owned))
            .collect(),
        _ => Vec::new(),
    }
}

fn is_self(xref: &Value) -> bool {
    kinds(xref).iter().any(|kind| kind == "self")
}

/// 項目の候補：インスタンスの in の場所（無ければ中のどこでも）にある、id を持つオブジェクト。
fn items<'value>(doc: &'value Value, within: Option<&str>) -> Vec<&'value Value> {
    match within {
        Some(path) => collect(doc, path)
            .into_iter()
            .filter(|item| id_of(item).is_some())
            .collect(),
        None => {
            let mut out = Vec::new();
            fn walk<'value>(value: &'value Value, out: &mut Vec<&'value Value>, root: bool) {
                match value {
                    Value::Object(object) => {
                        if !root && object.contains_key("id") {
                            out.push(value);
                        }
                        object.values().for_each(|child| walk(child, out, false));
                    }
                    Value::Array(elements) => {
                        elements.iter().for_each(|child| walk(child, out, false))
                    }
                    _ => {}
                }
            }
            walk(doc, &mut out, true);
            out
        }
    }
}

pub struct Checker<'docs> {
    docs: &'docs [Doc<'docs>],
    links: Vec<Link>,
    derived: Vec<Derived>,
    findings: Vec<Finding>,
    /// 注釈の場所 → その注釈を持つスキーマの名前（inverse の候補を、参照が0件でも数えるため）
    annots: BTreeMap<String, (String, Value)>,
}

impl Checker<'_> {
    fn label(&self, doc: usize) -> String {
        id_of(&self.docs[doc].value)
            .map(str::to_owned)
            .unwrap_or_else(|| self.docs[doc].path.clone())
    }

    fn target_label(&self, target: &Target) -> String {
        match &target.item {
            Some(item) => format!("{}.{item}", self.label(target.doc)),
            None => self.label(target.doc),
        }
    }

    fn push(
        &mut self,
        status: Status,
        check: &str,
        from: String,
        to: String,
        message: impl Into<String>,
    ) {
        self.findings.push(Finding {
            status,
            check: check.to_owned(),
            from,
            to,
            message: message.into(),
        });
    }

    /// スキーマとインスタンスを一緒にたどり、x-ref の参照と x-derive の導出値を集める。
    #[allow(clippy::too_many_arguments)]
    fn walk(
        &mut self,
        doc: usize,
        node: &Value,
        schema_document: &Value,
        pointer: String,
        value: &Value,
        at: String,
        owner: &Value,
        owner_at: &str,
    ) {
        let schema = self.docs[doc].schema;
        let annot_xref = node.get("x-ref").cloned();
        let annot_derive = node.get("x-derive").cloned();
        let (node, schema_document) = schema.resolve(node, schema_document);
        let xref = annot_xref.or_else(|| node.get("x-ref").cloned());
        let derive = annot_derive.or_else(|| node.get("x-derive").cloned());
        if let Some(xref) = xref {
            let key = format!("{}#{pointer}", schema.name());
            self.annots
                .entry(key.clone())
                .or_insert_with(|| (schema.name().to_owned(), xref.clone()));
            let written: Vec<String> = match value {
                Value::String(text) => vec![text.clone()],
                Value::Array(elements) => elements
                    .iter()
                    .filter_map(|element| element.as_str().map(str::to_owned))
                    .collect(),
                _ => Vec::new(),
            };
            for written_value in written {
                if let Some(only) = xref.get("only").and_then(Value::as_str) {
                    if !matches_pattern(only, &written_value) {
                        continue;
                    }
                }
                self.links.push(Link {
                    doc,
                    at: at.clone(),
                    owner: owner.clone(),
                    owner_at: owner_at.to_owned(),
                    annot: key.clone(),
                    xref: xref.clone(),
                    value: written_value,
                    targets: Vec::new(),
                });
            }
        }
        if let (Some(derive), Some(_)) = (derive, value.as_object()) {
            self.derive(doc, &derive, value, &at, node, schema_document);
        }
        if let (Some(properties), Some(object)) = (
            node.get("properties").and_then(Value::as_object),
            value.as_object(),
        ) {
            for (key, sub) in properties {
                if let Some(property_value) = object.get(key) {
                    self.walk(
                        doc,
                        sub,
                        schema_document,
                        format!("{pointer}/properties/{key}"),
                        property_value,
                        format!("{at}/{key}"),
                        value,
                        &at,
                    );
                }
            }
        }
        if let (Some(items), Some(array)) = (node.get("items"), value.as_array()) {
            for (position, element) in array.iter().enumerate() {
                let here = format!("{at}/{position}");
                self.walk(
                    doc,
                    items,
                    schema_document,
                    format!("{pointer}/items"),
                    element,
                    here.clone(),
                    element,
                    &here,
                );
            }
        }
    }

    fn derive_value(rules: &Value, value: &Value) -> Value {
        for rule in rules
            .get("rules")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let when = rule.get("when").and_then(Value::as_object);
            if when.is_some_and(|when| {
                when.iter()
                    .all(|(key, expected)| value.get(key) == Some(expected))
            }) {
                return rule.get("then").cloned().unwrap_or(Value::Null);
            }
        }
        rules.get("otherwise").cloned().unwrap_or(Value::Null)
    }

    #[allow(clippy::too_many_arguments)]
    fn derive(
        &mut self,
        doc: usize,
        derive: &Value,
        value: &Value,
        at: &str,
        node: &Value,
        schema_document: &Value,
    ) {
        let got = Self::derive_value(derive, value);
        let schema = self.docs[doc].schema;
        // 問い：決まりの when に出てくるプロパティを、出てきた順に1回ずつ
        let mut keys: Vec<String> = Vec::new();
        for rule in derive
            .get("rules")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            for key in rule
                .get("when")
                .and_then(Value::as_object)
                .into_iter()
                .flatten()
                .map(|(key, _)| key)
            {
                if !keys.contains(key) {
                    keys.push(key.clone());
                }
            }
        }
        let questions = keys
            .iter()
            .map(|key| {
                let title = node
                    .get("properties")
                    .and_then(|properties| properties.get(key))
                    .and_then(|sub| {
                        sub.get("title")
                            .or_else(|| schema.resolve(sub, schema_document).0.get("title"))
                            .cloned()
                    })
                    .unwrap_or_else(|| Value::String(key.clone()));
                json!({"key": key, "title": title, "answer": value.get(key).cloned().unwrap_or(Value::Null)})
            })
            .collect();
        let declared_value = derive
            .get("declared")
            .and_then(Value::as_str)
            .and_then(|name| value.get(name))
            .cloned()
            .unwrap_or(Value::Null);
        let from = format!("{}{at}", self.label(doc));
        if let Some(name) = derive.get("declared").and_then(Value::as_str) {
            if let Some(declared) = value.get(name) {
                if DerivedValue::new(got.clone(), declared.clone()).compare() == Status::Pass {
                    self.push(
                        Status::Pass,
                        "導出値と宣言した値",
                        from.clone(),
                        name.to_owned(),
                        format!("{got}"),
                    );
                } else {
                    self.push(
                        Status::Drift,
                        "導出値と宣言した値",
                        from.clone(),
                        name.to_owned(),
                        format!("導出値は {got}、宣言した値は {declared}"),
                    );
                }
            }
        }
        for expect in derive
            .get("expect")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let root = &self.docs[doc].value;
            if !condition_holds(root, expect.get("when").unwrap_or(&Value::Null)) {
                continue;
            }
            let name = expect
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("組み合わせ")
                .to_owned();
            let ok = match (
                expect.get("in").and_then(Value::as_array),
                expect.get("not_in").and_then(Value::as_array),
            ) {
                (Some(list), _) => list.contains(&got),
                (_, Some(list)) => !list.contains(&got),
                _ => true,
            };
            let status = if ok { Status::Pass } else { Status::Drift };
            self.push(
                status,
                &name,
                from.clone(),
                String::new(),
                format!("導出値は {got}"),
            );
        }
        self.derived.push(Derived {
            doc,
            at: at.to_owned(),
            value: got,
            title: derive.get("title").cloned().unwrap_or(Value::Null),
            declared: declared_value,
            questions,
        });
    }

    fn resolve_link(&self, link: &Link) -> Result<Vec<Target>, String> {
        let xref = &link.xref;
        let within = xref.get("in").and_then(Value::as_str);
        let target_kinds = kinds(xref);
        let of_kind = |candidate: &Doc| {
            candidate
                .value
                .get("kind")
                .and_then(Value::as_str)
                .is_some_and(|kind| target_kinds.iter().any(|target_kind| target_kind == kind))
        };
        let find_items = |doc: usize, id: &str| -> Vec<Target> {
            items(&self.docs[doc].value, within)
                .into_iter()
                .filter(|item| id_of(item) == Some(id))
                .map(|_| Target {
                    doc,
                    item: Some(id.to_owned()),
                })
                .collect()
        };
        let found: Vec<Target> = if is_self(xref) {
            find_items(link.doc, &link.value)
        } else if xref.get("item") == Some(&Value::Bool(true)) {
            let Some((instance_id, item)) = link.value.split_once('.') else {
                return Err("形が違う（インスタンスの id.項目の id ではない）".into());
            };
            (0..self.docs.len())
                .filter(|&doc_index| {
                    of_kind(&self.docs[doc_index])
                        && id_of(&self.docs[doc_index].value) == Some(instance_id)
                })
                .flat_map(|doc_index| find_items(doc_index, item))
                .collect()
        } else if xref.get("bare") == Some(&Value::Bool(true)) {
            (0..self.docs.len())
                .filter(|&doc_index| of_kind(&self.docs[doc_index]))
                .flat_map(|doc_index| find_items(doc_index, &link.value))
                .collect()
        } else {
            (0..self.docs.len())
                .filter(|&doc_index| {
                    of_kind(&self.docs[doc_index])
                        && id_of(&self.docs[doc_index].value) == Some(link.value.as_str())
                })
                .map(|doc_index| Target {
                    doc: doc_index,
                    item: None,
                })
                .collect()
        };
        match found.len() {
            0 => Err("指す先が無い".into()),
            1 => Ok(found),
            _ if xref.get("bare") == Some(&Value::Bool(true)) => {
                Err("指す先が1つに決まらない".into())
            }
            _ => Ok(found[..1].to_vec()),
        }
    }

    fn target_value(&self, target: &Target, within: Option<&str>) -> Value {
        let doc = &self.docs[target.doc].value;
        match &target.item {
            None => doc.clone(),
            Some(id) => items(doc, within)
                .into_iter()
                .find(|item| id_of(item) == Some(id))
                .cloned()
                .unwrap_or(Value::Null),
        }
    }

    fn check_links(&mut self, approved: Option<&Approved>) {
        let mut links = std::mem::take(&mut self.links);
        for link in &mut links {
            let from = format!("{}{}", self.label(link.doc), link.at);
            match self.resolve_link(link) {
                Err(msg) => self.push(Status::Drift, "指す先がある", from, link.value.clone(), msg),
                Ok(targets) => {
                    link.targets = targets.clone();
                    let to = self.target_label(&targets[0]);
                    self.push(Status::Pass, "指す先がある", from.clone(), to.clone(), "");
                    if let Some(accept) = link.xref.get("accept") {
                        let gate = link
                            .xref
                            .get("when")
                            .is_none_or(|when| condition_holds(&link.owner, when));
                        if gate {
                            let within = link.xref.get("in").and_then(Value::as_str);
                            let target_value = self.target_value(&targets[0], within);
                            let at = accept.get("at").and_then(Value::as_str).unwrap_or("");
                            let allowed = accept
                                .get("in")
                                .and_then(Value::as_array)
                                .cloned()
                                .unwrap_or_default();
                            let ok = collect(&target_value, at)
                                .iter()
                                .any(|value| allowed.contains(value));
                            let status = if ok { Status::Pass } else { Status::Drift };
                            let msg = if ok {
                                ""
                            } else {
                                "指す先の種類が違う"
                            };
                            self.push(
                                status,
                                "指す先が受け付ける値",
                                from.clone(),
                                to.clone(),
                                msg,
                            );
                        }
                    }
                    if let Some(covered_by) = link.xref.get("covered_by").and_then(Value::as_str) {
                        let ok = collect(&link.owner, covered_by)
                            .iter()
                            .any(|value| value.as_str() == Some(link.value.as_str()));
                        let status = if ok { Status::Pass } else { Status::Drift };
                        let msg = if ok {
                            String::new()
                        } else {
                            format!("{covered_by} で扱われていない")
                        };
                        self.push(status, "扱われている", from.clone(), to.clone(), msg);
                    }
                    if let Some(approved) = approved {
                        let target_path = &self.docs[targets[0].doc].path;
                        let (status, msg) = match approved.get(target_path) {
                            Some(hash) if hash == &self.docs[targets[0].doc].hash => {
                                (Status::Pass, "承認した時点から変わっていない")
                            }
                            Some(_) => (Status::Recheck, "承認のあとで指す先が変わった"),
                            None => (Status::Recheck, "まだ承認していない"),
                        };
                        self.push(status, "承認のあとの変化", from, to, msg);
                    }
                }
            }
        }
        // unique：同じ配列に並ぶ項目のあいだで、同じ値を2回指さない
        let mut seen: BTreeMap<(String, usize, String), Vec<&Link>> = BTreeMap::new();
        for link in links
            .iter()
            .filter(|link| link.xref.get("unique") == Some(&Value::Bool(true)))
        {
            let array_at = link
                .owner_at
                .rsplit_once('/')
                .map(|(array_at, _)| array_at.to_owned())
                .unwrap_or_default();
            seen.entry((link.annot.clone(), link.doc, array_at))
                .or_default()
                .push(link);
        }
        for ((_, doc, _), same_array_links) in &seen {
            let mut values = BTreeSet::new();
            for link in same_array_links {
                if !values.insert(link.value.clone()) {
                    let from = format!("{}{}", self.label(*doc), link.at);
                    self.push(
                        Status::Drift,
                        "重ねて指さない",
                        from,
                        link.value.clone(),
                        "同じ値を重ねて指している",
                    );
                }
            }
        }
        self.links = links;
    }

    fn check_inverse(&mut self) {
        let mut groups: BTreeMap<String, (Value, Vec<String>)> = BTreeMap::new();
        for (key, (_, xref)) in &self.annots {
            if let Some(inverse) = xref.get("inverse") {
                let group = inverse
                    .get("group")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_owned();
                let entry = groups
                    .entry(group)
                    .or_insert_with(|| (xref.clone(), Vec::new()));
                entry.1.push(key.clone());
            }
        }
        for (group, (xref, keys)) in groups {
            let inverse = xref.get("inverse").cloned().unwrap_or_default();
            let within = xref.get("in").and_then(Value::as_str);
            let target_kinds = kinds(&xref);
            let schema_names: BTreeSet<String> = keys
                .iter()
                .filter_map(|key| self.annots.get(key).map(|annot| annot.0.clone()))
                .collect();
            let mut candidates: Vec<Target> = Vec::new();
            for (doc_index, doc) in self.docs.iter().enumerate() {
                let take = if is_self(&xref) {
                    schema_names.contains(doc.schema.name())
                } else {
                    doc.value
                        .get("kind")
                        .and_then(Value::as_str)
                        .is_some_and(|kind| {
                            target_kinds.iter().any(|target_kind| target_kind == kind)
                        })
                };
                if !take {
                    continue;
                }
                let item_level = is_self(&xref)
                    || xref.get("item").is_some()
                    || xref.get("bare").is_some()
                    || within.is_some();
                if item_level {
                    for item in items(&doc.value, within) {
                        candidates.push(Target {
                            doc: doc_index,
                            item: id_of(item).map(str::to_owned),
                        });
                    }
                } else {
                    candidates.push(Target {
                        doc: doc_index,
                        item: None,
                    });
                }
            }
            for candidate in candidates {
                let candidate_value = self.target_value(&candidate, within);
                if let Some(condition) = inverse.get("where") {
                    if !condition_holds(&candidate_value, condition) {
                        continue;
                    }
                }
                if let Some(where_derive) = inverse.get("where_derive") {
                    let via = where_derive
                        .get("via")
                        .and_then(Value::as_str)
                        .unwrap_or("");
                    let property = where_derive
                        .get("derive")
                        .and_then(Value::as_str)
                        .unwrap_or("");
                    let allowed = where_derive
                        .get("in")
                        .and_then(Value::as_array)
                        .cloned()
                        .unwrap_or_default();
                    let hit = collect(&candidate_value, via)
                        .iter()
                        .filter_map(|value| value.as_str())
                        .any(|id| {
                            self.derived.iter().any(|derived| {
                                id_of(&self.docs[derived.doc].value) == Some(id)
                                    && derived.at == format!("/{property}")
                                    && allowed.contains(&derived.value)
                            })
                        });
                    if !hit {
                        continue;
                    }
                }
                let refs: Vec<&Link> = self
                    .links
                    .iter()
                    .filter(|link| {
                        keys.contains(&link.annot)
                            && link.targets.first() == Some(&candidate)
                            && (!is_self(&xref) || link.doc == candidate.doc)
                    })
                    .collect();
                let count = refs.len() as u64;
                let decls = refs
                    .iter()
                    .map(|link| link.doc)
                    .collect::<BTreeSet<_>>()
                    .len() as u64;
                let to = self.target_label(&candidate);
                let mut problems = Vec::new();
                if let Some(min) = inverse.get("min").and_then(Value::as_u64) {
                    if count < min {
                        problems.push(format!("指されていない（{count} 件、下限 {min}）"));
                    }
                }
                if let Some(max) = inverse.get("max").and_then(Value::as_u64) {
                    if count > max {
                        problems.push(format!("指されすぎ（{count} 件、上限 {max}）"));
                    }
                }
                if let Some(max_decls) = inverse.get("max_decls").and_then(Value::as_u64) {
                    if decls > max_decls {
                        problems.push(format!(
                            "指すインスタンスが多すぎる（{decls} 件、上限 {max_decls}）"
                        ));
                    }
                }
                if problems.is_empty() {
                    self.push(
                        Status::Pass,
                        &group,
                        String::new(),
                        to,
                        format!("{count} 件"),
                    );
                } else {
                    self.push(
                        Status::Drift,
                        &group,
                        String::new(),
                        to,
                        problems.join("、"),
                    );
                }
            }
        }
    }
}

/// インスタンスの集まりを検査する。approved があれば、承認のあとの変化も出す。
pub fn check(docs: &[Doc], approved: Option<&Approved>) -> Vec<Finding> {
    let mut checker = collect_all(docs);
    checker.check_links(approved);
    checker.check_inverse();
    let mut out = checker.findings;
    out.sort();
    out.dedup();
    out
}

/// インスタンスの集まりの参照（指す先を解決したもの）と導出値を返す。検査と同じ解決を使う。
pub fn graph(docs: &[Doc]) -> Graph {
    let mut checker = collect_all(docs);
    checker.check_links(None);
    let links = checker
        .links
        .iter()
        .map(|link| {
            let target = link.targets.first().cloned();
            GraphLink {
                doc: link.doc,
                at: link.at.clone(),
                value: link.value.clone(),
                label: target.as_ref().map(|target| checker.target_label(target)),
                target: target.map(|target| (target.doc, target.item)),
            }
        })
        .collect();
    let derived = checker
        .derived
        .iter()
        .map(|derived| GraphDerived {
            doc: derived.doc,
            at: derived.at.clone(),
            value: derived.value.clone(),
            title: derived.title.clone(),
            declared: derived.declared.clone(),
            questions: derived.questions.clone(),
        })
        .collect();
    Graph { links, derived }
}

/// スキーマとインスタンスを一緒にたどり、参照と導出値を集める（指す先はまだ解決しない）。
fn collect_all<'docs>(docs: &'docs [Doc<'docs>]) -> Checker<'docs> {
    let mut checker = Checker {
        docs,
        links: Vec::new(),
        derived: Vec::new(),
        findings: Vec::new(),
        annots: BTreeMap::new(),
    };
    for (doc_index, doc) in docs.iter().enumerate() {
        let root = doc.schema.root();
        let value = doc.value.clone();
        checker.walk(
            doc_index,
            root,
            root,
            String::new(),
            &value,
            String::new(),
            &value,
            "",
        );
    }
    // 参照が1件も無いスキーマの注釈も、inverse の候補を数えるために登録する
    for doc in docs {
        register(
            &mut checker.annots,
            doc.schema,
            doc.schema.root(),
            doc.schema.root(),
            String::new(),
            0,
        );
    }
    checker
}

fn register(
    annots: &mut BTreeMap<String, (String, Value)>,
    schema: &Schema,
    node: &Value,
    schema_document: &Value,
    pointer: String,
    depth: usize,
) {
    if depth > 32 {
        return;
    }
    if let Some(xref) = node.get("x-ref") {
        annots
            .entry(format!("{}#{pointer}", schema.name()))
            .or_insert_with(|| (schema.name().to_owned(), xref.clone()));
    }
    let (node, schema_document) = schema.resolve(node, schema_document);
    if let Some(xref) = node.get("x-ref") {
        annots
            .entry(format!("{}#{pointer}", schema.name()))
            .or_insert_with(|| (schema.name().to_owned(), xref.clone()));
    }
    if let Some(properties) = node.get("properties").and_then(Value::as_object) {
        for (key, sub) in properties {
            register(
                annots,
                schema,
                sub,
                schema_document,
                format!("{pointer}/properties/{key}"),
                depth + 1,
            );
        }
    }
    if let Some(items) = node.get("items") {
        let items_pointer = format!("{pointer}/items");
        register(
            annots,
            schema,
            items,
            schema_document,
            items_pointer,
            depth + 1,
        );
    }
}
