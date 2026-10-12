//! ドメインサービス 具体のスキーマを検査する。
//! 構造の契約は、スキーマの $schema が示す版のメタスキーマ（schemas/meta-schema.json）で、注釈の形（すべての深さ）・ 文の型 ・ 申告していない注釈 ・
//! 参照される種類の id は、この処理で見る。契約はビルドのときに取り込むので、結果はツールの版で決まる。

use crate::domain::contract::{self, Version};
use crate::domain::schema::Schema;
use serde_json::{json, Value};
use std::collections::BTreeSet;

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
    /// 直し方の案内（契約の欄の x-prompt）。案内の無い指摘は None
    pub prompt: Option<Value>,
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

/// パイプの右が関数の呼び出しでない（{name|upper} のように書くと、upper というプロパティを引いて空になる）。
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
fn walk<P: Fn(&str) -> Result<(), String>>(
    value: &Value,
    at: &str,
    name: &str,
    declared: &BTreeSet<String>,
    shapes: &Shapes,
    names: bool,
    parse: &P,
    out: &mut Vec<SchemaFinding>,
) {
    let mut push = |at: String, rule: &str, message: String| {
        out.push(SchemaFinding {
            schema: name.to_owned(),
            at,
            rule: rule.to_owned(),
            message,
            prompt: None,
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
                                push(here.clone(), "パイプの右が関数でない", format!("{{{expr}}} ── パイプの右は関数の呼び出しにする（例：name(@)）"));
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
pub fn check_schemas<P: Fn(&str) -> Result<(), String>>(
    schemas: &[(String, Value)],
    parse: &P,
) -> Vec<SchemaFinding> {
    let meta_v1 = Schema::new(
        "meta-schema.json",
        parse_json(crate::embedded::schema_text(contract::V1)),
        vec![],
    );
    let sib = vec![
        (
            "annotations.schema.json".to_owned(),
            parse_json(crate::embedded::schema_text("schema-driven/annotations@v1")),
        ),
        (
            "view.schema.json".to_owned(),
            parse_json(crate::embedded::schema_text("schema-driven/view@v1")),
        ),
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
                    prompt: None,
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
                    prompt: None,
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
                        prompt: meta.prompt(unfilled.property()),
                    });
                }
                for error in &validation.errors {
                    out.push(SchemaFinding {
                        schema: name.clone(),
                        at: error.property().to_owned(),
                        rule: "メタスキーマ".into(),
                        message: error.reason().to_owned(),
                        prompt: meta.prompt(error.property()),
                    });
                }
            }
            Err(error) => out.push(SchemaFinding {
                schema: name.clone(),
                at: String::new(),
                rule: "メタスキーマ".into(),
                message: format!("検査できない（{}）", error.0),
                prompt: None,
            }),
        }
        let declared: BTreeSet<String> = schema
            .get("x-annotations")
            .and_then(Value::as_object)
            .map(|annotations| annotations.keys().cloned().collect())
            .unwrap_or_default();
        walk(schema, "", name, &declared, &shapes, false, parse, &mut out);
        if schema.get("x-generates").is_some() {
            let id = schema.get("$id").and_then(Value::as_str);
            let named = schema
                .pointer("/properties/$schema/const")
                .and_then(Value::as_str);
            if let (Some(id), Some(named)) = (id, named) {
                if id != named {
                    out.push(SchemaFinding {
                        schema: name.clone(),
                        at: "/properties/$schema/const".into(),
                        rule: "インスタンスが指す名前".into(),
                        message: format!("{named} は、このスキーマの $id {id} と違う"),
                        prompt: None,
                    });
                }
            }
        }
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
                    prompt: None,
                });
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    // 具体のスキーマの検査（schemas/meta-schema.json と、すべての深さの注釈）と、abstract スキーマの版のテスト。
    // 具体のスキーマは `x-abstract` で使う版を示し、v1 では x-prompt を新しい形で書く。
    use crate::domain::schema::Schema;
    use crate::domain::schema_rules::{check_schemas, SchemaFinding};
    use serde_json::{json, Value};

    /// JMESPath として読めない式の表。
    const UNREADABLE: [&str; 1] = ["word["];

    /// ポート Query の parse の代役。core は jmespath を使えないので、読めない式を表で持つ。
    /// 表に無い式は読めるものとして返す（このテストの式は、表の1つを除いて JMESPath として読める）。
    fn parse(expression: &str) -> Result<(), String> {
        match UNREADABLE.contains(&expression) {
            true => Err(format!("読めない：{expression}")),
            false => Ok(()),
        }
    }

    /// base に extra のキーを重ねる。
    fn merged(mut base: Value, extra: Value) -> Value {
        for (key, value) in extra.as_object().unwrap() {
            base[key] = value.clone();
        }
        base
    }

    fn run(schemas: Vec<(&str, Value)>) -> Vec<SchemaFinding> {
        let list: Vec<(String, Value)> = schemas
            .into_iter()
            .map(|(file_name, schema)| (file_name.to_owned(), schema))
            .collect();
        check_schemas(&list, &parse)
    }

    fn rules(findings: &[SchemaFinding]) -> Vec<&str> {
        findings
            .iter()
            .map(|finding| finding.rule.as_str())
            .collect()
    }

    fn field(extra: Value) -> Value {
        merged(
            json!({"type": "string", "description": "説明", "x-prompt": {"read": {"means": "見本の欄である。"},
              "write": [{"id": "one", "rule": "値を1つ書く。", "why": "見本である。", "test": "値があるかを見る。"}]}}),
            extra,
        )
    }

    /// 決まりに従っている具体のスキーマ。
    fn good() -> Value {
        json!({"$schema": "https://json-schema.org/draft/2020-12/schema", "x-abstract": "schema-driven/abstract@v1", "$id": "lexicon/glossary@v1", "title": "用語集", "description": "語の一覧", "x-generates": "declarations/GLO-<番号>.json",
           "x-view": {"label": "header.name"},
           "properties": {
               "$schema": field(json!({"const": "lexicon/glossary@v1"})),
               "kind": field(json!({"const": "glossary"})),
               "id": field(json!({"pattern": "^GLO-[0-9]+$", "x-view": {"hidden": true}})),
               "terms": field(json!({"type": "array", "items": {"type": "object", "properties": {
                   "word": {"type": "string", "x-view": {"text": "{word | quote(@)}"}}}}}))}})
    }

    const V1: &str = "schema-driven/abstract@v1";
    const DIALECT: &str = "https://json-schema.org/draft/2020-12/schema";

    /// スキーマを1つだけ検査する。
    fn run_one(schema: Value) -> Vec<SchemaFinding> {
        run(vec![("glossary.schema.json", schema)])
    }

    /// v1 の x-prompt。
    fn v1_prompt() -> Value {
        json!({
            "read": {"means": "用語の一覧である。", "shown": "見出しの下に表で出る。"},
            "write": [
                {"id": "one-word", "rule": "1行に1語を書く。", "why": "語ごとに参照できるようにする。", "test": "1行に語が1つだけかを見る。"}
            ]
        })
    }

    fn v1_field(extra: Value) -> Value {
        merged(
            json!({"type": "string", "description": "説明", "x-prompt": v1_prompt()}),
            extra,
        )
    }

    /// v1 に従っている具体のスキーマ。
    fn v1_good() -> Value {
        json!({"$schema": DIALECT, "x-abstract": V1, "$id": "lexicon/glossary@v1", "title": "用語集", "description": "語の一覧", "x-prompt": v1_prompt(),
           "x-generates": "declarations/GLO-<番号>.json",
           "properties": {
               "$schema": v1_field(json!({"const": "lexicon/glossary@v1"})),
               "kind": v1_field(json!({"const": "glossary"})),
               "word": v1_field(json!({}))},
           "required": ["kind", "word"]})
    }

    // ── 具体のスキーマの検査

    #[test]
    fn conforming_schema_has_no_findings() {
        assert_eq!(run(vec![("glossary.schema.json", good())]), vec![]);
    }

    #[test]
    fn meta_schema_requires_prompt_per_field_and_kind_for_generating_schemas() {
        let mut schema = good();
        schema["properties"]["terms"]
            .as_object_mut()
            .unwrap()
            .remove("x-prompt");
        schema["properties"].as_object_mut().unwrap().remove("kind");
        let findings = run(vec![("glossary.schema.json", schema)]);
        assert_eq!(
            rules(&findings),
            vec!["メタスキーマ", "メタスキーマ"],
            "{findings:?}"
        );
        assert!(findings
            .iter()
            .any(|finding| finding.at.contains("/properties/kind")));
        assert!(findings
            .iter()
            .any(|finding| finding.at.contains("/properties/terms/x-prompt")));
    }

    #[test]
    fn annotation_shapes_are_checked_at_every_depth() {
        let mut schema = good();
        schema["properties"]["terms"]["items"]["properties"]["word"]["x-ref"] =
            json!({"to": "glossary", "meaning": "語"});
        let findings = run(vec![("glossary.schema.json", schema)]);
        assert_eq!(rules(&findings), vec!["注釈の形"], "{findings:?}");
        assert_eq!(
            findings[0].at,
            "/properties/terms/items/properties/word/x-ref"
        );
    }

    #[test]
    fn templates_must_be_jmespath_and_pipes_must_call_functions() {
        let mut schema = good();
        let word = &mut schema["properties"]["terms"]["items"]["properties"]["word"];
        word["x-view"] = json!({"text": "{word|name} と {word[} と {word | name(@)}"});
        let findings = run(vec![("glossary.schema.json", schema)]);
        assert_eq!(
            rules(&findings),
            vec!["パイプの右が関数でない", "文の型"],
            "{findings:?}"
        );
        assert!(findings[0].message.contains("{word|name}"));
    }

    #[test]
    fn undeclared_annotations_are_findings_and_declared_ones_pass() {
        let mut schema = good();
        schema["properties"]["terms"]["x-test-spec"] = json!({"checks": "x"});
        assert_eq!(
            rules(&run(vec![("glossary.schema.json", schema.clone())])),
            vec!["申告していない注釈"]
        );
        schema["x-annotations"] = json!({"x-test-spec": "test-spec.schema.json"});
        assert_eq!(run(vec![("glossary.schema.json", schema)]), vec![]);
    }

    #[test]
    fn referenced_kinds_must_have_id() {
        let mut glossary = good();
        glossary["properties"].as_object_mut().unwrap().remove("id");
        let use_case = json!({"$schema": "https://json-schema.org/draft/2020-12/schema", "x-abstract": "schema-driven/abstract@v1", "$id": "lexicon/use-case@v1", "title": "ユースケース", "description": "やり取り", "x-generates": "declarations/UC-<番号>.json",
                    "properties": {
                        "$schema": field(json!({"const": "lexicon/use-case@v1"})),
                        "kind": field(json!({"const": "use_case"})),
                        "id": field(json!({})),
                        "terms": field(json!({"x-ref": {"to": "glossary", "item": true}}))}});
        let findings = run(vec![
            ("glossary.schema.json", glossary.clone()),
            ("use_case.schema.json", use_case.clone()),
        ]);
        assert_eq!(
            rules(&findings),
            vec!["参照される種類の id"],
            "{findings:?}"
        );
        assert_eq!(findings[0].schema, "glossary.schema.json");
        // 指されていなければ、id は求めない
        assert_eq!(run(vec![("glossary.schema.json", glossary)]), vec![]);
    }

    #[test]
    fn generating_schemas_must_name_themselves_in_the_instance_schema() {
        let mut missing = good();
        missing["properties"]
            .as_object_mut()
            .unwrap()
            .remove("$schema");
        assert_eq!(
            rules(&run(vec![("glossary.schema.json", missing)])),
            vec!["メタスキーマ"]
        );
        let mut other = good();
        other["properties"]["$schema"]["const"] = json!("lexicon/other@v1");
        let findings = run(vec![("glossary.schema.json", other)]);
        assert_eq!(
            rules(&findings),
            vec!["インスタンスが指す名前"],
            "{findings:?}"
        );
        assert_eq!(findings[0].at, "/properties/$schema/const");
    }

    // ── abstract スキーマの版（v1）

    #[test]
    fn a_v1_schema_validates_instances() {
        let schema = Schema::new("glossary.schema.json", v1_good(), vec![]);
        let validation = schema
            .validate(&json!({"kind": "glossary"}))
            .expect("v1 の $schema で検証器を作れる");
        assert_eq!(validation.errors, vec![]);
        assert_eq!(validation.unfilled.len(), 1);
    }

    #[test]
    fn a_v1_schema_in_the_new_form_has_no_findings() {
        assert_eq!(run_one(v1_good()), vec![]);
    }

    #[test]
    fn a_v1_schema_rejects_string_prompts() {
        let mut schema = v1_good();
        schema["properties"]["word"]["x-prompt"] = json!({"read": "r", "write": "w"});
        assert!(rules(&run_one(schema)).contains(&"注釈の形"));
    }

    #[test]
    fn a_v1_read_needs_means() {
        let mut schema = v1_good();
        schema["properties"]["word"]["x-prompt"]["read"] = json!({"shown": "表で出る。"});
        assert!(rules(&run_one(schema)).contains(&"注釈の形"));
    }

    #[test]
    fn rule_ids_do_not_repeat_within_one_v1_prompt() {
        let mut schema = v1_good();
        let rule = schema["properties"]["word"]["x-prompt"]["write"][0].clone();
        schema["properties"]["word"]["x-prompt"]["write"]
            .as_array_mut()
            .unwrap()
            .push(rule);
        assert!(rules(&run_one(schema)).contains(&"決まりの id の重なり"));
    }

    #[test]
    fn a_sentence_is_at_most_120_characters() {
        let mut schema = v1_good();
        schema["properties"]["word"]["x-prompt"]["write"][0]["why"] = json!("あ".repeat(121));
        assert!(rules(&run_one(schema)).contains(&"注釈の形"));
    }

    #[test]
    fn bad_and_good_come_as_a_pair() {
        let mut schema = v1_good();
        schema["properties"]["word"]["x-prompt"]["write"][0]["bad"] = json!("2語を1行に書く。");
        assert!(rules(&run_one(schema)).contains(&"注釈の形"));
    }

    #[test]
    fn a_schema_without_a_version_is_refused() {
        let unversioned = json!({"title": "用語集", "description": "語の一覧",
        "properties": {"word": {"type": "string", "description": "語", "x-prompt": {"read": "r", "write": "w"}}}});
        assert_eq!(rules(&run_one(unversioned)), vec!["版が無い"]);
    }

    #[test]
    fn an_unknown_version_is_reported() {
        let mut schema = v1_good();
        schema["x-abstract"] = json!("schema-driven/abstract@v9");
        assert!(rules(&run_one(schema)).contains(&"知らない版"));
    }

    #[test]
    fn names_under_defs_are_not_annotations() {
        let mut schema = v1_good();
        schema["$defs"] = json!({"x-ref": {"type": "object", "description": "参照の形",
        "x-prompt": v1_prompt(),
        "properties": {"to": v1_field(json!({}))}}});
        let findings = run_one(schema);
        assert!(
            !rules(&findings).contains(&"注釈の形")
                && !rules(&findings).contains(&"申告していない注釈"),
            "{findings:?}"
        );
    }
}
