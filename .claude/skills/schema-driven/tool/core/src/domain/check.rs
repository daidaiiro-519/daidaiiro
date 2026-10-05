//! ドメインサービス 検査する（DS-1）。インスタンスの集まりについて、注釈 x-ref と x-derive を
//! references/annotations.schema.json の仕様どおりに当てる。読むのは注釈と、インスタンスの id と kind だけ。

use crate::domain::schema::Schema;
use crate::domain::values::Hash;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

/// 検査結果の状態（VO-9）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Status {
    /// 合格
    Pass,
    /// ずれ（直すべき食い違い）
    Drift,
    /// 確かめ直し（承認のあとに指す先が変わった、またはまだ承認していない）
    Recheck,
}

impl Status {
    pub fn label(self) -> &'static str {
        match self {
            Status::Pass => "合格",
            Status::Drift => "ずれ",
            Status::Recheck => "確かめ直し",
        }
    }
}

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
pub struct Doc<'a> {
    pub path: String,
    pub value: Value,
    pub hash: Hash,
    pub schema: &'a Schema,
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
}

/// 場所（path）の値を集める。配列は要素すべてに広げる。
pub(crate) fn collect<'a>(value: &'a Value, path: &str) -> Vec<&'a Value> {
    let mut cur: Vec<&Value> = flatten(vec![value]);
    for seg in path.split('/').filter(|s| !s.is_empty()) {
        cur = flatten(cur.into_iter().filter_map(|v| v.get(seg)).collect());
    }
    cur
}

fn flatten(list: Vec<&Value>) -> Vec<&Value> {
    let mut out = Vec::new();
    for v in list {
        match v {
            Value::Array(a) => out.extend(flatten(a.iter().collect())),
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
        collect(base, path).iter().any(|v| allowed.contains(v))
    })
}

fn matches_pattern(pattern: &str, value: &str) -> bool {
    jsonschema::is_valid(&json!({"pattern": pattern}), &json!(value))
}

fn id_of(v: &Value) -> Option<&str> {
    v.get("id").and_then(Value::as_str)
}

fn kinds(xref: &Value) -> Vec<String> {
    match xref.get("to") {
        Some(Value::String(s)) => vec![s.clone()],
        Some(Value::Array(a)) => a
            .iter()
            .filter_map(|v| v.as_str().map(str::to_owned))
            .collect(),
        _ => Vec::new(),
    }
}

fn is_self(xref: &Value) -> bool {
    kinds(xref).iter().any(|k| k == "self")
}

/// 項目の候補：インスタンスの in の場所（無ければ中のどこでも）にある、id を持つオブジェクト。
fn items<'a>(doc: &'a Value, within: Option<&str>) -> Vec<&'a Value> {
    match within {
        Some(path) => collect(doc, path)
            .into_iter()
            .filter(|v| id_of(v).is_some())
            .collect(),
        None => {
            let mut out = Vec::new();
            fn walk<'a>(v: &'a Value, out: &mut Vec<&'a Value>, root: bool) {
                match v {
                    Value::Object(m) => {
                        if !root && m.contains_key("id") {
                            out.push(v);
                        }
                        m.values().for_each(|c| walk(c, out, false));
                    }
                    Value::Array(a) => a.iter().for_each(|c| walk(c, out, false)),
                    _ => {}
                }
            }
            walk(doc, &mut out, true);
            out
        }
    }
}

pub struct Checker<'a> {
    docs: &'a [Doc<'a>],
    links: Vec<Link>,
    derived: Vec<Derived>,
    findings: Vec<Finding>,
    /// 注釈の場所 → その注釈を持つスキーマの名前（inverse の候補を、参照が0件でも数えるため）
    annots: BTreeMap<String, (String, Value)>,
}

impl<'a> Checker<'a> {
    fn label(&self, doc: usize) -> String {
        id_of(&self.docs[doc].value)
            .map(str::to_owned)
            .unwrap_or_else(|| self.docs[doc].path.clone())
    }

    fn target_label(&self, t: &Target) -> String {
        match &t.item {
            Some(i) => format!("{}.{i}", self.label(t.doc)),
            None => self.label(t.doc),
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
        sdoc: &Value,
        ptr: String,
        value: &Value,
        at: String,
        owner: &Value,
        owner_at: &str,
    ) {
        let schema = self.docs[doc].schema;
        let annot_xref = node.get("x-ref").cloned();
        let annot_derive = node.get("x-derive").cloned();
        let (node, sdoc) = schema.resolve(node, sdoc);
        let xref = annot_xref.or_else(|| node.get("x-ref").cloned());
        let derive = annot_derive.or_else(|| node.get("x-derive").cloned());
        if let Some(x) = xref {
            let key = format!("{}#{ptr}", schema.name());
            self.annots
                .entry(key.clone())
                .or_insert_with(|| (schema.name().to_owned(), x.clone()));
            let values: Vec<String> = match value {
                Value::String(s) => vec![s.clone()],
                Value::Array(a) => a
                    .iter()
                    .filter_map(|v| v.as_str().map(str::to_owned))
                    .collect(),
                _ => Vec::new(),
            };
            for v in values {
                if let Some(only) = x.get("only").and_then(Value::as_str) {
                    if !matches_pattern(only, &v) {
                        continue;
                    }
                }
                self.links.push(Link {
                    doc,
                    at: at.clone(),
                    owner: owner.clone(),
                    owner_at: owner_at.to_owned(),
                    annot: key.clone(),
                    xref: x.clone(),
                    value: v,
                    targets: Vec::new(),
                });
            }
        }
        if let (Some(d), Some(_)) = (derive, value.as_object()) {
            self.derive(doc, &d, value, &at);
        }
        if let (Some(props), Some(obj)) = (
            node.get("properties").and_then(Value::as_object),
            value.as_object(),
        ) {
            for (k, sub) in props {
                if let Some(v) = obj.get(k) {
                    self.walk(
                        doc,
                        sub,
                        sdoc,
                        format!("{ptr}/properties/{k}"),
                        v,
                        format!("{at}/{k}"),
                        value,
                        &at,
                    );
                }
            }
        }
        if let (Some(items), Some(arr)) = (node.get("items"), value.as_array()) {
            for (i, v) in arr.iter().enumerate() {
                let here = format!("{at}/{i}");
                self.walk(
                    doc,
                    items,
                    sdoc,
                    format!("{ptr}/items"),
                    v,
                    here.clone(),
                    v,
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
            if when.is_some_and(|w| w.iter().all(|(k, v)| value.get(k) == Some(v))) {
                return rule.get("then").cloned().unwrap_or(Value::Null);
            }
        }
        rules.get("otherwise").cloned().unwrap_or(Value::Null)
    }

    fn derive(&mut self, doc: usize, d: &Value, value: &Value, at: &str) {
        let got = Self::derive_value(d, value);
        let from = format!("{}{at}", self.label(doc));
        if let Some(name) = d.get("declared").and_then(Value::as_str) {
            if let Some(declared) = value.get(name) {
                if declared == &got {
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
        for e in d
            .get("expect")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let root = &self.docs[doc].value;
            if !condition_holds(root, e.get("when").unwrap_or(&Value::Null)) {
                continue;
            }
            let name = e
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("組み合わせ")
                .to_owned();
            let ok = match (
                e.get("in").and_then(Value::as_array),
                e.get("not_in").and_then(Value::as_array),
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
        });
    }

    fn resolve_link(&self, l: &Link) -> Result<Vec<Target>, String> {
        let x = &l.xref;
        let within = x.get("in").and_then(Value::as_str);
        let ks = kinds(x);
        let of_kind = |d: &Doc| {
            d.value
                .get("kind")
                .and_then(Value::as_str)
                .is_some_and(|k| ks.iter().any(|x| x == k))
        };
        let find_items = |doc: usize, id: &str| -> Vec<Target> {
            items(&self.docs[doc].value, within)
                .into_iter()
                .filter(|v| id_of(v) == Some(id))
                .map(|_| Target {
                    doc,
                    item: Some(id.to_owned()),
                })
                .collect()
        };
        let found: Vec<Target> = if is_self(x) {
            find_items(l.doc, &l.value)
        } else if x.get("item") == Some(&Value::Bool(true)) {
            let Some((inst, item)) = l.value.split_once('.') else {
                return Err("形が違う（インスタンスの id.項目の id ではない）".into());
            };
            (0..self.docs.len())
                .filter(|&i| of_kind(&self.docs[i]) && id_of(&self.docs[i].value) == Some(inst))
                .flat_map(|i| find_items(i, item))
                .collect()
        } else if x.get("bare") == Some(&Value::Bool(true)) {
            (0..self.docs.len())
                .filter(|&i| of_kind(&self.docs[i]))
                .flat_map(|i| find_items(i, &l.value))
                .collect()
        } else {
            (0..self.docs.len())
                .filter(|&i| {
                    of_kind(&self.docs[i]) && id_of(&self.docs[i].value) == Some(l.value.as_str())
                })
                .map(|i| Target { doc: i, item: None })
                .collect()
        };
        match found.len() {
            0 => Err("指す先が無い".into()),
            1 => Ok(found),
            _ if x.get("bare") == Some(&Value::Bool(true)) => Err("指す先が1つに決まらない".into()),
            _ => Ok(found[..1].to_vec()),
        }
    }

    fn target_value(&self, t: &Target, within: Option<&str>) -> Value {
        let doc = &self.docs[t.doc].value;
        match &t.item {
            None => doc.clone(),
            Some(id) => items(doc, within)
                .into_iter()
                .find(|v| id_of(v) == Some(id))
                .cloned()
                .unwrap_or(Value::Null),
        }
    }

    fn check_links(&mut self, approved: Option<&Approved>) {
        let mut links = std::mem::take(&mut self.links);
        for l in &mut links {
            let from = format!("{}{}", self.label(l.doc), l.at);
            match self.resolve_link(l) {
                Err(msg) => self.push(Status::Drift, "指す先がある", from, l.value.clone(), msg),
                Ok(targets) => {
                    l.targets = targets.clone();
                    let to = self.target_label(&targets[0]);
                    self.push(Status::Pass, "指す先がある", from.clone(), to.clone(), "");
                    if let Some(acc) = l.xref.get("accept") {
                        let gate = l
                            .xref
                            .get("when")
                            .is_none_or(|w| condition_holds(&l.owner, w));
                        if gate {
                            let within = l.xref.get("in").and_then(Value::as_str);
                            let tv = self.target_value(&targets[0], within);
                            let at = acc.get("at").and_then(Value::as_str).unwrap_or("");
                            let allowed = acc
                                .get("in")
                                .and_then(Value::as_array)
                                .cloned()
                                .unwrap_or_default();
                            let ok = collect(&tv, at).iter().any(|v| allowed.contains(v));
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
                    if let Some(cov) = l.xref.get("covered_by").and_then(Value::as_str) {
                        let ok = collect(&l.owner, cov)
                            .iter()
                            .any(|v| v.as_str() == Some(l.value.as_str()));
                        let status = if ok { Status::Pass } else { Status::Drift };
                        let msg = if ok {
                            String::new()
                        } else {
                            format!("{cov} で扱われていない")
                        };
                        self.push(status, "扱われている", from.clone(), to.clone(), msg);
                    }
                    if let Some(appr) = approved {
                        let tpath = &self.docs[targets[0].doc].path;
                        let (status, msg) = match appr.get(tpath) {
                            Some(h) if h == &self.docs[targets[0].doc].hash => {
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
        for l in links
            .iter()
            .filter(|l| l.xref.get("unique") == Some(&Value::Bool(true)))
        {
            let array_at = l
                .owner_at
                .rsplit_once('/')
                .map(|(a, _)| a.to_owned())
                .unwrap_or_default();
            seen.entry((l.annot.clone(), l.doc, array_at))
                .or_default()
                .push(l);
        }
        for ((_, doc, _), ls) in &seen {
            let mut values = BTreeSet::new();
            for l in ls {
                if !values.insert(l.value.clone()) {
                    let from = format!("{}{}", self.label(*doc), l.at);
                    self.push(
                        Status::Drift,
                        "重ねて指さない",
                        from,
                        l.value.clone(),
                        "同じ値を重ねて指している",
                    );
                }
            }
        }
        self.links = links;
    }

    fn check_inverse(&mut self) {
        let mut groups: BTreeMap<String, (Value, Vec<String>)> = BTreeMap::new();
        for (key, (_, x)) in &self.annots {
            if let Some(inv) = x.get("inverse") {
                let g = inv
                    .get("group")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_owned();
                let e = groups.entry(g).or_insert_with(|| (x.clone(), Vec::new()));
                e.1.push(key.clone());
            }
        }
        for (group, (x, keys)) in groups {
            let inv = x.get("inverse").cloned().unwrap_or_default();
            let within = x.get("in").and_then(Value::as_str);
            let ks = kinds(&x);
            let schema_names: BTreeSet<String> = keys
                .iter()
                .filter_map(|k| self.annots.get(k).map(|a| a.0.clone()))
                .collect();
            let mut candidates: Vec<Target> = Vec::new();
            for (i, d) in self.docs.iter().enumerate() {
                let take = if is_self(&x) {
                    schema_names.contains(d.schema.name())
                } else {
                    d.value
                        .get("kind")
                        .and_then(Value::as_str)
                        .is_some_and(|k| ks.iter().any(|x| x == k))
                };
                if !take {
                    continue;
                }
                let item_level = is_self(&x)
                    || x.get("item").is_some()
                    || x.get("bare").is_some()
                    || within.is_some();
                if item_level {
                    for v in items(&d.value, within) {
                        candidates.push(Target {
                            doc: i,
                            item: id_of(v).map(str::to_owned),
                        });
                    }
                } else {
                    candidates.push(Target { doc: i, item: None });
                }
            }
            for c in candidates {
                let cv = self.target_value(&c, within);
                if let Some(w) = inv.get("where") {
                    if !condition_holds(&cv, w) {
                        continue;
                    }
                }
                if let Some(wd) = inv.get("where_derive") {
                    let via = wd.get("via").and_then(Value::as_str).unwrap_or("");
                    let prop = wd.get("derive").and_then(Value::as_str).unwrap_or("");
                    let allowed = wd
                        .get("in")
                        .and_then(Value::as_array)
                        .cloned()
                        .unwrap_or_default();
                    let hit = collect(&cv, via)
                        .iter()
                        .filter_map(|v| v.as_str())
                        .any(|id| {
                            self.derived.iter().any(|dv| {
                                id_of(&self.docs[dv.doc].value) == Some(id)
                                    && dv.at == format!("/{prop}")
                                    && allowed.contains(&dv.value)
                            })
                        });
                    if !hit {
                        continue;
                    }
                }
                let refs: Vec<&Link> = self
                    .links
                    .iter()
                    .filter(|l| {
                        keys.contains(&l.annot)
                            && l.targets.first() == Some(&c)
                            && (!is_self(&x) || l.doc == c.doc)
                    })
                    .collect();
                let count = refs.len() as u64;
                let decls = refs.iter().map(|l| l.doc).collect::<BTreeSet<_>>().len() as u64;
                let to = self.target_label(&c);
                let mut problems = Vec::new();
                if let Some(min) = inv.get("min").and_then(Value::as_u64) {
                    if count < min {
                        problems.push(format!("指されていない（{count} 件、下限 {min}）"));
                    }
                }
                if let Some(max) = inv.get("max").and_then(Value::as_u64) {
                    if count > max {
                        problems.push(format!("指されすぎ（{count} 件、上限 {max}）"));
                    }
                }
                if let Some(maxd) = inv.get("max_decls").and_then(Value::as_u64) {
                    if decls > maxd {
                        problems.push(format!(
                            "指すインスタンスが多すぎる（{decls} 件、上限 {maxd}）"
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
    let mut c = Checker {
        docs,
        links: Vec::new(),
        derived: Vec::new(),
        findings: Vec::new(),
        annots: BTreeMap::new(),
    };
    for (i, d) in docs.iter().enumerate() {
        let root = d.schema.root();
        let value = d.value.clone();
        c.walk(
            i,
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
    for d in docs {
        register(
            &mut c.annots,
            d.schema,
            d.schema.root(),
            d.schema.root(),
            String::new(),
            0,
        );
    }
    c.check_links(approved);
    c.check_inverse();
    let mut out = c.findings;
    out.sort();
    out.dedup();
    out
}

fn register(
    annots: &mut BTreeMap<String, (String, Value)>,
    schema: &Schema,
    node: &Value,
    sdoc: &Value,
    ptr: String,
    depth: usize,
) {
    if depth > 32 {
        return;
    }
    if let Some(x) = node.get("x-ref") {
        annots
            .entry(format!("{}#{ptr}", schema.name()))
            .or_insert_with(|| (schema.name().to_owned(), x.clone()));
    }
    let (node, sdoc) = schema.resolve(node, sdoc);
    if let Some(x) = node.get("x-ref") {
        annots
            .entry(format!("{}#{ptr}", schema.name()))
            .or_insert_with(|| (schema.name().to_owned(), x.clone()));
    }
    if let Some(props) = node.get("properties").and_then(Value::as_object) {
        for (k, sub) in props {
            register(
                annots,
                schema,
                sub,
                sdoc,
                format!("{ptr}/properties/{k}"),
                depth + 1,
            );
        }
    }
    if let Some(items) = node.get("items") {
        register(
            annots,
            schema,
            items,
            sdoc,
            format!("{ptr}/items"),
            depth + 1,
        );
    }
}
