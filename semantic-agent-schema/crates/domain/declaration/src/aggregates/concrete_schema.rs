//! Semantic Concrete Schema。欄は pub(in crate::aggregates::concrete_schema)
pub mod entities;
pub mod factories;

use crate::aggregates::abstract_schema::X_KEYWORDS;
use crate::aggregates::entities::delegate::Delegate;
use crate::values::breach::Breach;
use crate::values::by::By;
use crate::values::patch::{self, Refusal};
use crate::values::term::Term;
use crate::values::urn::Urn;
use entities::rule::Rule;
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};

/// Semantic Concrete Schema（セマンティックコンクリートスキーマ）：Abstract の版を1つ申告し、データの形と書き方の rule を持つ契約
#[derive(Debug, Clone, PartialEq)]
pub struct ConcreteSchema {
    pub(in crate::aggregates::concrete_schema) id: Urn,
    pub(in crate::aggregates::concrete_schema) abstract_id: Urn,
    pub(in crate::aggregates::concrete_schema) document: Value,
}

fn escape(token: &str) -> String {
    token.replace('~', "~0").replace('/', "~1")
}

/// 欄（入れ子を含む）を1つずつ渡す。たどる先は Abstract の field と同じ
fn walk_fields(field: &Value, at: &str, visit: &mut dyn FnMut(&str, &Map<String, Value>)) {
    let Some(map) = field.as_object() else {
        return;
    };
    visit(at, map);
    for key in ["properties", "$defs"] {
        if let Some(children) = map.get(key).and_then(Value::as_object) {
            for (name, child) in children {
                walk_fields(
                    child,
                    &format!("{at}/{}/{}", escape(key), escape(name)),
                    visit,
                );
            }
        }
    }
    for key in ["items", "additionalProperties"] {
        if let Some(child) = map.get(key) {
            walk_fields(child, &format!("{at}/{key}"), visit);
        }
    }
    if let Some(list) = map.get("prefixItems").and_then(Value::as_array) {
        for (n, child) in list.iter().enumerate() {
            walk_fields(child, &format!("{at}/prefixItems/{n}"), visit);
        }
    }
}

impl ConcreteSchema {
    /// 文書から読む。$id と $schema が URN であることだけを見る（形の検証は Validator が行う）。不変条件 I3
    ///
    /// # Errors
    /// $id か $schema が無いか、URN でないとき
    pub fn from_document(document: Value) -> Result<Self, String> {
        let id = document
            .get("$id")
            .and_then(Value::as_str)
            .ok_or("$id が無い")?;
        let abstract_id = document
            .get("$schema")
            .and_then(Value::as_str)
            .ok_or("$schema が無い")?;
        Ok(ConcreteSchema {
            id: Urn::parse(id)?,
            abstract_id: Urn::parse(abstract_id)?,
            document,
        })
    }

    /// 名前（$id）
    pub fn id(&self) -> &Urn {
        &self.id
    }

    /// 申告した Abstract の版（$schema）
    pub fn abstract_id(&self) -> &Urn {
        &self.abstract_id
    }

    /// 文書
    pub fn document(&self) -> &Value {
        &self.document
    }

    /// JSON Patch を適用した Concrete を返す。検査①：外れたら断り、自分は変えない。不変条件 I3
    ///
    /// # Errors
    /// 断ったとき（理由は [`Refusal`]）
    pub fn patched(&self, ops: &Value) -> Result<ConcreteSchema, Refusal> {
        let document = patch::apply(&self.document, ops)?;
        Ok(ConcreteSchema {
            id: self.id.clone(),
            abstract_id: self.abstract_id.clone(),
            document,
        })
    }

    /// 任せる先（x-delegates）
    pub fn delegates(&self) -> Vec<Delegate> {
        delegates_of(&self.document)
    }

    /// rule の一覧（根と欄の x-prompt.write）
    pub fn rules(&self) -> Vec<Rule> {
        let mut out = Vec::new();
        walk_fields(&self.document, "", &mut |at, map| {
            let written = map
                .get("x-prompt")
                .and_then(|p| p.get("write"))
                .and_then(Value::as_array);
            for (n, r) in written.into_iter().flatten().enumerate() {
                let text = |k: &str| r.get(k).and_then(Value::as_str).map(str::to_string);
                out.push(Rule {
                    id: text("id").unwrap_or_default(),
                    pointer: format!("{at}/x-prompt/write/{n}"),
                    by: text("by").as_deref().and_then(By::parse),
                    agent: text("agent"),
                    reads: r
                        .get("reads")
                        .and_then(Value::as_array)
                        .into_iter()
                        .flatten()
                        .filter_map(|x| x.as_str().map(str::to_string))
                        .collect(),
                });
            }
        });
        out
    }

    /// 欄が $ref で指す glossary の term（場所と term）
    pub fn terms(&self) -> Vec<(String, Term)> {
        let mut out = Vec::new();
        walk_fields(&self.document, "", &mut |at, map| {
            if let Some(term) = map
                .get("$ref")
                .and_then(Value::as_str)
                .and_then(Term::from_ref)
            {
                out.push((format!("{at}/$ref"), term));
            }
        });
        out
    }

    /// その x- のキーワードを、根か欄のどこかで使っているか
    pub fn uses(&self, keyword: &str) -> bool {
        let mut found = self.document.get(keyword).is_some();
        walk_fields(&self.document, "", &mut |_, map| {
            found |= map.contains_key(keyword)
        });
        found
    }

    /// x-relationship が指す Concrete の URN（場所と URN）
    pub fn relationship_targets(&self) -> Vec<(String, String)> {
        let mut out = Vec::new();
        walk_fields(&self.document, "", &mut |at, map| {
            if let Some(to) = map
                .get("x-relationship")
                .and_then(|r| r.get("to"))
                .and_then(Value::as_str)
            {
                out.push((format!("{at}/x-relationship/to"), to.to_string()));
            }
        });
        out
    }

    /// 形（JSON Schema）では見られない、Abstract の機械の rule への違反。rule の id 付きで返す。
    /// body_delegates は Abstract の本体の x-delegates の名前（agent はこのスキーマか本体の任せる先を指せる）。不変条件 I4 ・ I6
    pub fn breaches(&self, body_delegates: &[String]) -> Vec<Breach> {
        let mut out = Vec::new();
        let doc = &self.document;
        if self.id.kind() == "glossary" {
            out.push(Breach::new("owner-kind-version", "/$id", "種類 glossary は glossary の Abstract のファイルに取ってある（Concrete では使わない）".to_string()));
        }
        // rule：unique-id ・ agent-exists ・ by-matches-kind
        let own = self.delegates();
        let mut first: BTreeMap<String, String> = BTreeMap::new();
        for rule in self.rules() {
            if let Some(earlier) = first.get(rule.id()) {
                out.push(Breach::new(
                    "unique-id",
                    rule.pointer(),
                    format!("rule の id「{}」が {earlier} と重なる", rule.id()),
                ));
            } else {
                first.insert(rule.id().to_string(), rule.pointer().to_string());
            }
            let Some(agent) = rule.agent() else { continue };
            match own.iter().find(|d| d.name() == agent) {
                Some(d) => {
                    let ok = match rule.by() {
                        Some(By::Ai) => d.kind() == "judging" && !d.has_argv(),
                        Some(By::Machine) => d.kind() == "judging" && d.has_argv(),
                        _ => true,
                    };
                    if !ok {
                        out.push(Breach::new("by-matches-kind", rule.pointer(), format!("by と任せる先「{agent}」の種類 ・ argv が合わない（ai は argv の無い judging、machine は argv のある judging）")));
                    }
                }
                None if body_delegates.iter().any(|b| b == agent) => {}
                None => out.push(Breach::new(
                    "agent-exists",
                    rule.pointer(),
                    format!("任せる先「{agent}」が、このスキーマにも本体の x-delegates にも無い"),
                )),
            }
        }
        // x-delegates：own-only ・ answer-inline
        if let Some(ds) = doc.get("x-delegates").and_then(Value::as_object) {
            for (name, d) in ds {
                let at = format!("/x-delegates/{}", escape(name));
                if contains_text(d, "urn:semantic-agent-schema:") {
                    out.push(Breach::new(
                        "own-only",
                        &at,
                        format!("任せる先「{name}」にほかの Concrete の URN がある"),
                    ));
                }
                if d.get("answer").is_some_and(|a| contains_key(a, "$ref")) {
                    out.push(Breach::new(
                        "answer-inline",
                        &format!("{at}/answer"),
                        "答えの形に $ref がある（answer の中に書き切る）".to_string(),
                    ));
                }
            }
        }
        // x-extensions：no-abstract-names ・ prompt-like-field、と定義の無い x-（abstract-keywords-closed）
        let declared: BTreeSet<String> = doc
            .get("x-extensions")
            .and_then(Value::as_object)
            .into_iter()
            .flatten()
            .map(|(k, _)| k.clone())
            .collect();
        for key in &declared {
            let at = format!("/x-extensions/{}", escape(key));
            if X_KEYWORDS.contains(&key.as_str()) {
                out.push(Breach::new(
                    "no-abstract-names",
                    &at,
                    format!("{key} は Abstract の x- と同じ名前（Concrete の拡張では定義しない）"),
                ));
            }
            let means = doc
                .pointer(&format!("{at}/prompt/read/means"))
                .and_then(Value::as_str);
            if means.is_none_or(str::is_empty) {
                out.push(Breach::new(
                    "prompt-like-field",
                    &format!("{at}/prompt"),
                    "prompt が欄の x-prompt と同じ形でない（read.means が無い）".to_string(),
                ));
            }
        }
        walk_fields(doc, "", &mut |at, map| {
            for key in map.keys().filter(|k| k.starts_with("x-")) {
                if !X_KEYWORDS.contains(&key.as_str()) && !declared.contains(key) {
                    out.push(Breach::new(
                        "abstract-keywords-closed",
                        &format!("{at}/{}", escape(key)),
                        format!("{key} は Abstract にも x-extensions にも無い"),
                    ));
                }
            }
        });
        // $ref：no-ref-into-extensions
        for (at, r) in refs(doc, "") {
            if r.contains("/x-extensions") {
                out.push(Breach::new(
                    "no-ref-into-extensions",
                    &at,
                    "$ref が x-extensions の中を指している".to_string(),
                ));
            }
        }
        // 根の欄の名前
        let fields: BTreeSet<String> = doc
            .get("properties")
            .and_then(Value::as_object)
            .into_iter()
            .flatten()
            .map(|(k, _)| k.clone())
            .collect();
        // x-instance-path：level1-template ・ only-for-instances ・ instance-path-and-kind ・ instance-schema-name
        if let Some(path) = doc.get("x-instance-path").and_then(Value::as_str) {
            for name in placeholders(path) {
                if !is_level1(&name) || !fields.contains(&name) {
                    out.push(Breach::new(
                        "level1-template",
                        "/x-instance-path",
                        format!("{{{name}}} は根の properties にある欄の名前（レベル1）でない"),
                    ));
                }
            }
            if doc.get("properties").is_none() {
                out.push(Breach::new(
                    "only-for-instances",
                    "/x-instance-path",
                    "インスタンスを作らない（properties の無い）スキーマに x-instance-path がある"
                        .to_string(),
                ));
            }
            let has_const = |n: &str| doc.pointer(&format!("/properties/{n}/const")).is_some();
            if !has_const("kind") || !fields.contains("id") {
                out.push(Breach::new(
                    "instance-path-and-kind",
                    "/properties",
                    "x-instance-path を書いたら、根の properties に kind（const）と id を置く"
                        .to_string(),
                ));
            }
            if doc
                .pointer("/properties/$schema/const")
                .and_then(Value::as_str)
                != Some(self.id.to_string().as_str())
            {
                out.push(Breach::new(
                    "instance-schema-name",
                    "/properties/$schema",
                    "根の properties の $schema の const が、このスキーマの $id でない".to_string(),
                ));
            }
        }
        // x-instance-label：label-fields-exist
        if let Some(label) = doc.get("x-instance-label").and_then(Value::as_str) {
            for name in placeholders(label) {
                if !fields.contains(&name) {
                    out.push(Breach::new(
                        "label-fields-exist",
                        "/x-instance-label",
                        format!("{{{name}}} は根の properties に無い"),
                    ));
                }
            }
        }
        // 欄：prompt-per-unit
        for name in &fields {
            if name == "$schema"
                || name == "properties"
                || name == "$defs"
                || name.starts_with("x-")
            {
                continue;
            }
            let f = &doc["properties"][name];
            if f.is_object() && (f.get("description").is_none() || f.get("x-prompt").is_none()) {
                out.push(Breach::new(
                    "prompt-per-unit",
                    &format!("/properties/{}", escape(name)),
                    format!("欄「{name}」に description と x-prompt が無い"),
                ));
            }
        }
        // x-derived-from：declared-exists ・ explicit-default
        walk_objects(doc, "", &mut |at, map| {
            let Some(props) = map.get("properties").and_then(Value::as_object) else {
                return;
            };
            for (name, f) in props {
                let Some(d) = f.get("x-derived-from") else {
                    continue;
                };
                let here = format!("{at}/properties/{}/x-derived-from", escape(name));
                if let Some(declared) = d.get("declared").and_then(Value::as_str)
                    && !props.contains_key(declared)
                {
                    out.push(Breach::new(
                        "declared-exists",
                        &format!("{here}/declared"),
                        format!("「{declared}」が同じオブジェクトの欄に無い"),
                    ));
                }
                let then = d
                    .get("rules")
                    .and_then(Value::as_array)
                    .and_then(|r| r.first())
                    .and_then(|r| r.get("then"));
                if let (Some(t), Some(o)) = (then, d.get("otherwise"))
                    && json_type(t) != json_type(o)
                {
                    out.push(Breach::new(
                        "explicit-default",
                        &format!("{here}/otherwise"),
                        format!(
                            "otherwise の型（{}）が決まりの結果の型（{}）と違う",
                            json_type(o),
                            json_type(t)
                        ),
                    ));
                }
            }
        });
        out
    }
}

fn delegates_of(document: &Value) -> Vec<Delegate> {
    document
        .get("x-delegates")
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
        .map(|(name, d)| Delegate {
            name: name.clone(),
            kind: d
                .pointer("/run/kind")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            has_argv: d.pointer("/run/argv").is_some(),
        })
        .collect()
}

/// 文書の中のオブジェクトを、場所と一緒に1つずつ渡す
fn walk_objects(v: &Value, at: &str, visit: &mut dyn FnMut(&str, &Map<String, Value>)) {
    match v {
        Value::Object(map) => {
            visit(at, map);
            for (k, child) in map {
                walk_objects(child, &format!("{at}/{}", escape(k)), visit);
            }
        }
        Value::Array(list) => {
            for (n, child) in list.iter().enumerate() {
                walk_objects(child, &format!("{at}/{n}"), visit);
            }
        }
        _ => {}
    }
}

/// $ref の値を、場所と一緒に集める
fn refs(v: &Value, at: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    walk_objects(v, at, &mut |here, map| {
        if let Some(r) = map.get("$ref").and_then(Value::as_str) {
            out.push((format!("{here}/$ref"), r.to_string()));
        }
    });
    out
}

fn contains_text(v: &Value, needle: &str) -> bool {
    match v {
        Value::String(s) => s.contains(needle),
        Value::Array(l) => l.iter().any(|x| contains_text(x, needle)),
        Value::Object(m) => m.values().any(|x| contains_text(x, needle)),
        _ => false,
    }
}

fn contains_key(v: &Value, key: &str) -> bool {
    match v {
        Value::Array(l) => l.iter().any(|x| contains_key(x, key)),
        Value::Object(m) => m.contains_key(key) || m.values().any(|x| contains_key(x, key)),
        _ => false,
    }
}

/// {…} の中の名前を並べる
fn placeholders(template: &str) -> Vec<String> {
    template
        .split('{')
        .skip(1)
        .filter_map(|part| part.split_once('}').map(|(name, _)| name.to_string()))
        .collect()
}

/// RFC 6570 のレベル1：演算子も修飾子も無い、名前だけの式
fn is_level1(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn json_type(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

#[cfg(test)]
mod tests {
    use super::ConcreteSchema;
    use serde_json::{Value, json};

    fn concrete(extra: Value) -> ConcreteSchema {
        let mut doc = json!({
            "$schema": "urn:semantic-agent-schema:base:semantic-abstract-schema:v1",
            "$id": "urn:semantic-agent-schema:t:record:v1",
            "x-delegates": {"judge": {"role": "読む", "run": {"kind": "judging"}}, "lint": {"role": "検査", "run": {"kind": "judging", "argv": ["lint"]}}},
            "x-extensions": {"x-owner": {"schema": true, "description": "責任者", "prompt": {"read": {"means": "責任者"}}}}
        });
        if let (Some(d), Some(e)) = (doc.as_object_mut(), extra.as_object()) {
            d.extend(e.clone());
        }
        ConcreteSchema::from_document(doc).unwrap()
    }

    fn field(extra: Value) -> Value {
        let mut f = json!({"description": "欄", "x-prompt": {"read": {"means": "意味"}}});
        if let (Some(d), Some(e)) = (f.as_object_mut(), extra.as_object()) {
            d.extend(e.clone());
        }
        f
    }

    fn with_rule(r: Value) -> Value {
        json!({"properties": {"a": field(json!({"x-prompt": {"read": {"means": "m"}, "write": [r]}}))}})
    }

    fn rule(id: &str, by: &str, agent: Option<&str>) -> Value {
        let mut r = json!({"id": id, "rule": "r", "why": "w", "test": "t", "by": by});
        if let Some(a) = agent {
            r["agent"] = json!(a);
        }
        r
    }

    fn broken(c: &ConcreteSchema) -> Vec<String> {
        c.breaches(&["judge".to_string()])
            .iter()
            .map(|b| b.rule().to_string())
            .collect()
    }

    /// traces: I3
    #[test]
    fn schema_that_is_not_a_urn_is_refused() {
        assert!(ConcreteSchema::from_document(json!({"$schema": "https://example.com/s", "$id": "urn:semantic-agent-schema:t:r:v1"})).is_err());
    }

    /// traces: I4
    #[test]
    fn rules_are_found_in_nested_fields() {
        let c = concrete(
            json!({"properties": {"a": field(json!({"type": "object", "properties": {"b": {"x-prompt": {"read": {"means": "m"}, "write": [rule("b-rule", "human", None)]}}}}))}}),
        );
        assert_eq!(
            c.rules()[0].pointer(),
            "/properties/a/properties/b/x-prompt/write/0"
        );
    }

    /// traces: I4
    #[test]
    fn duplicate_rule_id_breaks_unique_id() {
        let c = concrete(
            json!({"properties": {"a": field(json!({"x-prompt": {"read": {"means": "m"}, "write": [rule("same", "human", None)]}})), "b": field(json!({"x-prompt": {"read": {"means": "m"}, "write": [rule("same", "human", None)]}}))}}),
        );
        assert_eq!(broken(&c), vec!["unique-id"]);
    }

    /// traces: I4
    #[test]
    fn agent_missing_everywhere_breaks_agent_exists() {
        assert_eq!(
            broken(&concrete(with_rule(rule("r", "ai", Some("nobody"))))),
            vec!["agent-exists"]
        );
    }

    /// traces: I4
    #[test]
    fn agent_of_the_abstract_body_is_allowed() {
        let mut c = concrete(with_rule(rule("r", "ai", Some("judge"))));
        c = c
            .patched(&json!([{"op": "remove", "path": "/x-delegates/judge"}]))
            .unwrap();
        assert!(broken(&c).is_empty());
    }

    /// traces: I4
    #[test]
    fn ai_rule_pointing_to_a_cli_breaks_by_matches_kind() {
        assert_eq!(
            broken(&concrete(with_rule(rule("r", "ai", Some("lint"))))),
            vec!["by-matches-kind"]
        );
    }

    /// traces: I4
    #[test]
    fn machine_rule_pointing_to_a_cli_passes() {
        assert!(broken(&concrete(with_rule(rule("r", "machine", Some("lint"))))).is_empty());
    }

    /// traces: I6
    #[test]
    fn x_keyword_without_definition_breaks_abstract_keywords_closed() {
        assert_eq!(
            broken(&concrete(
                json!({"properties": {"a": field(json!({"x-color": "red"}))}})
            )),
            vec!["abstract-keywords-closed"]
        );
    }

    /// traces: I6
    #[test]
    fn x_keyword_declared_in_extensions_passes() {
        assert!(
            broken(&concrete(
                json!({"properties": {"a": field(json!({"x-owner": "me"}))}})
            ))
            .is_empty()
        );
    }

    /// traces: I6
    #[test]
    fn extension_named_like_an_abstract_keyword_breaks_no_abstract_names() {
        let c = concrete(json!({})).patched(&json!([{"op": "add", "path": "/x-extensions/x-prompt", "value": {"schema": true, "description": "d", "prompt": {"read": {"means": "m"}}}}])).unwrap();
        assert_eq!(broken(&c), vec!["no-abstract-names"]);
    }

    /// traces: I4
    #[test]
    fn field_without_prompt_breaks_prompt_per_unit() {
        assert_eq!(
            broken(&concrete(json!({"properties": {"a": {"type": "string"}}}))),
            vec!["prompt-per-unit"]
        );
    }

    /// traces: I4
    #[test]
    fn instance_path_with_unknown_field_breaks_level1_template() {
        let c = concrete(
            json!({"x-instance-path": "r/{no}.json", "properties": {"$schema": {"const": "urn:semantic-agent-schema:t:record:v1"}, "kind": field(json!({"const": "record"})), "id": field(json!({}))}}),
        );
        assert_eq!(broken(&c), vec!["level1-template"]);
    }

    /// traces: I4
    #[test]
    fn instance_path_needs_kind_id_and_schema_name() {
        let c = concrete(
            json!({"x-instance-path": "r/{id}.json", "properties": {"id": field(json!({}))}}),
        );
        assert_eq!(
            broken(&c),
            vec!["instance-path-and-kind", "instance-schema-name"]
        );
    }

    /// traces: I4
    #[test]
    fn label_with_unknown_field_breaks_label_fields_exist() {
        assert_eq!(
            broken(&concrete(
                json!({"x-instance-label": "{title}", "properties": {}})
            )),
            vec!["label-fields-exist"]
        );
    }

    /// traces: I4
    #[test]
    fn delegate_with_another_concrete_breaks_own_only() {
        let c = concrete(
            json!({"x-delegates": {"other": {"role": "urn:semantic-agent-schema:x:y:v1 に任せる", "run": {"kind": "judging"}}}}),
        );
        assert_eq!(broken(&c), vec!["own-only"]);
    }

    /// traces: I4
    #[test]
    fn ref_into_extensions_breaks_no_ref_into_extensions() {
        assert_eq!(
            broken(&concrete(
                json!({"properties": {"a": field(json!({"$ref": "#/x-extensions/x-owner/schema"}))}})
            )),
            vec!["no-ref-into-extensions"]
        );
    }

    /// traces: I4
    #[test]
    fn concrete_of_kind_glossary_breaks_owner_kind_version() {
        let c = ConcreteSchema::from_document(json!({"$schema": "urn:semantic-agent-schema:base:semantic-abstract-schema:v1", "$id": "urn:semantic-agent-schema:t:glossary:v1"})).unwrap();
        assert_eq!(broken(&c), vec!["owner-kind-version"]);
    }

    /// traces: I4
    #[test]
    fn field_ref_to_glossary_is_a_term() {
        let c = concrete(
            json!({"properties": {"price": field(json!({"$ref": "urn:semantic-agent-schema:t:glossary:v1#Price"}))}}),
        );
        assert_eq!(c.terms()[0].0, "/properties/price/$ref");
    }

    /// traces: I3
    #[test]
    fn refused_patch_leaves_the_concrete_unchanged() {
        let c = concrete(json!({}));
        let before = c.clone();
        assert!(
            c.patched(&json!([{"op": "replace", "path": "/$schema", "value": "x"}]))
                .is_err()
        );
        assert_eq!(c, before);
    }
}
