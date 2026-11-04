//! UC1 作成する ・ UC2 更新する の意味で書く変形：Concrete の rule から書く手順を組み、打ったセッションに返す。
//! Semantic Agent Schema はエージェントを起動しない。手順を受け取ったセッションが値を決め、手順の command（semantic-agent-schema create … --value ／
//! semantic-agent-schema update … --patch）で書く。書くのは、その機械の作成 ・ 更新で、形と書く条件に照らしてから書く。
//!
//! - 書く欄は根の欄（x-prompt か、中に rule を持つ欄か、根が求める欄）。人が決める欄（x-decided-by: human）と、
//!   const で決まる欄と、$schema は書く欄にしない。欄の中（入れ子 ・ 配列の要素 ・ $ref の先）の rule も、そのステップに入れる。
//! - インスタンスは Concrete の rule で、Concrete は Abstract の rule で書く（同じ手続き）。
//! - 書く順は rule の related_fields：読まれる欄を先にする。読み合う欄があれば、手順を出さずに断る。
//! - --ref の文書は Semantic Agent Schema が読んで中身を手順に入れる。上限を超えたら切り詰めずに断る。
//! - 手順を返すだけで、記録しない。

use super::{Refused, carried, range_for, refused, target_of_declared};
use crate::ports::{DocumentStore, SchemaValidator};
use serde_json::{Map, Value, json};
use writing_domain::aggregates::escape;
use writing_domain::services::field_schema;
use writing_domain::values::{Caller, ContentHash, SchemaPlaces, TargetKind};

const PROCEDURE: &str = "urn:semantic-agent-schema:published-language:procedure:v1";
/// --ref の文書1つの上限（バイト）。
pub const REF_LIMIT: usize = 64 * 1024;

/// 意味で書く依頼。
#[derive(Debug, Clone, PartialEq)]
pub struct SemanticRequest {
    /// 書く経路
    pub path: String,
    /// 作成なら従う Concrete の $id。更新なら None（いまの中身の $schema）
    pub concrete: Option<String>,
    /// 何を書くかの依頼（--task）
    pub task: Option<String>,
    /// 読ませる文書の経路（--ref）
    pub refs: Vec<String>,
    /// スキーマの置き場所（従う Concrete と $ref の先を $id で引く。手順の command にも引き継ぐ）
    pub schemas: SchemaPlaces,
    /// Concrete を直すときに、従うインスタンスを確かめる範囲（--range の経路。手順の command に引き継ぐ）
    pub range: Option<Vec<String>>,
    /// 理由（手順の command に入れる）
    pub reason: Option<String>,
    /// やり直しで同じ操作を2回残さないための ID（手順の command に入れる）
    pub request_id: Option<String>,
}

/// 書く欄1つ。
struct Unit {
    name: String,
    field: Value,
    rules: Vec<Value>,
    inner: Vec<Value>,
    reads: Vec<String>,
    required: bool,
}

/// 意味で書く手順を組む（UC1 ・ UC2 の変形）。記録しない。
///
/// # Errors
/// 依頼を読めない ・ 型が決まらない ・ 書く欄が無い ・ 読み合う欄がある ・ 資料を読めない ・ 大きすぎるとき。
pub fn semantic_procedure(
    request: &SemanticRequest,
    store: &dyn DocumentStore,
    validator: &dyn SchemaValidator,
    caller: &Caller,
) -> Result<Value, Refused> {
    let path = request.path.as_str();
    let (declared, current, current_text) = match &request.concrete {
        Some(c) => {
            if store.read(path).is_ok() {
                return Err(refused(
                    "exists",
                    path,
                    format!("{path} はすでに在る。直すときは update を使う"),
                ));
            }
            (c.clone(), None, None)
        }
        None => {
            let text = store
                .read(path)
                .map_err(|e| refused("not_found", path, e))?;
            let value: Value = serde_json::from_str(&text)
                .map_err(|e| refused("not_json", path, format!("JSON として読めない：{e}")))?;
            let declared = value
                .get("$schema")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            (declared, Some(value), Some(text))
        }
    };
    let target = target_of_declared(store, validator, request.schemas.files(), path, &declared)?;
    if request.concrete.is_none() {
        range_for(target.kind, path, request.range.as_deref(), "直す")?;
    }
    // インスタンスは Concrete の rule で、Concrete は Abstract の rule で書く
    let doc: Value = match (target.kind, &target.concrete) {
        (TargetKind::Instance, Some(c)) => c.concrete().document().clone(),
        _ => validator.shipped(&declared).unwrap_or_default(),
    };
    let doc = &doc;
    let units = order(path, write_units(doc, target.more()))?;
    if units.is_empty() {
        return Err(refused(
            "invalid_input",
            path,
            "x-prompt を持ち、意味で書いてよい欄が無い".to_string(),
        ));
    }
    let refs = read_refs(store, &request.refs)?;
    let schema = bundled(doc, validator, target.more());
    Ok(build_procedure(
        path,
        &declared,
        doc,
        schema,
        current.as_ref(),
        current_text.as_deref(),
        request,
        &units,
        &refs,
        caller,
    ))
}

/// 書く欄：根の欄のうち、x-prompt か中の rule を持つか、根が求める欄。人の欄 ・ const の欄 ・ $schema は外す。
fn write_units(doc: &Value, others: &[(String, Value)]) -> Vec<Unit> {
    let Some(props) = doc.get("properties").and_then(Value::as_object) else {
        return Vec::new();
    };
    let required: Vec<&str> = doc
        .get("required")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    props
        .iter()
        .filter_map(|(name, own)| {
            if name == "$schema" {
                return None;
            }
            let field = field_schema(doc, &format!("/{}", escape(name)), others)?;
            if field.get("const").is_some()
                || field.get("x-decided-by").and_then(Value::as_str) == Some("human")
            {
                return None;
            }
            let rules: Vec<Value> = field
                .pointer("/x-prompt/rules")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let mut inner = Vec::new();
            inner_rules(
                doc,
                own,
                others,
                &format!("/{}", escape(name)),
                &mut inner,
                0,
            );
            let is_required = required.contains(&name.as_str());
            if field.get("x-prompt").is_none() && inner.is_empty() && !is_required {
                return None;
            }
            let reads = rules
                .iter()
                .flat_map(|r| {
                    r.get("related_fields")
                        .and_then(Value::as_array)
                        .cloned()
                        .unwrap_or_default()
                })
                .filter_map(|p| {
                    p.as_str()
                        .and_then(|p| p.split('/').nth(1))
                        .map(|s| s.replace("~1", "/").replace("~0", "~"))
                })
                .filter(|n| n != name)
                .collect();
            Some(Unit {
                name: name.clone(),
                field,
                rules,
                inner,
                reads,
                required: is_required,
            })
        })
        .collect()
}

/// 欄の中（properties ・ additionalProperties ・ items ・ allOf と $ref の先）の rule を、所の形
/// （配列の要素と名前を選ばない欄は *）と一緒に集める。
fn inner_rules(
    root: &Value,
    node: &Value,
    others: &[(String, Value)],
    at: &str,
    out: &mut Vec<Value>,
    depth: usize,
) {
    if depth > 8 {
        return;
    }
    let (root, node) = deref(root, node, others);
    for sub in node
        .get("allOf")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        inner_rules(root, sub, others, at, out, depth + 1);
    }
    let mut subs: Vec<(&Value, String)> = Vec::new();
    if let Some(props) = node.get("properties").and_then(Value::as_object) {
        for (name, sub) in props {
            subs.push((sub, format!("{at}/{}", escape(name))));
        }
    }
    for key in ["additionalProperties", "items"] {
        if let Some(sub) = node.get(key).filter(|a| a.is_object()) {
            subs.push((sub, format!("{at}/*")));
        }
    }
    for (sub, here) in subs {
        let (_, target) = deref(root, sub, others);
        let mut rules: Vec<Value> = Vec::new();
        for n in [sub, target] {
            for r in n
                .pointer("/x-prompt/rules")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                if !rules.contains(r) {
                    rules.push(r.clone());
                }
            }
        }
        if !rules.is_empty() && !out.iter().any(|o| o["pointer"] == json!(here)) {
            out.push(json!({"pointer": here, "rules": rules}));
        }
        inner_rules(root, sub, others, &here, out, depth + 1);
    }
}

/// $ref の先と、その先がある文書（文書の中の #… と、ほかの文書の <$id>#…）。引けなければそのまま。
fn deref<'a>(
    root: &'a Value,
    node: &'a Value,
    others: &'a [(String, Value)],
) -> (&'a Value, &'a Value) {
    let Some(r) = node.get("$ref").and_then(Value::as_str) else {
        return (root, node);
    };
    let (base, fragment) = r.split_once('#').unwrap_or((r, ""));
    let doc = if base.is_empty() {
        Some(root)
    } else {
        others.iter().find(|(id, _)| id == base).map(|(_, d)| d)
    };
    let Some(doc) = doc else {
        return (root, node);
    };
    let target = if fragment.is_empty() {
        Some(doc)
    } else if fragment.starts_with('/') {
        doc.pointer(fragment)
    } else {
        doc.get("$defs").and_then(Value::as_object).and_then(|m| {
            m.values()
                .find(|d| d.get("$anchor").and_then(Value::as_str) == Some(fragment))
        })
    };
    target.map_or((root, node), |t| (doc, t))
}

/// 書く元のスキーマの本体に、$ref で指すほかの文書（Concrete か、Semantic Agent Schema が入れて出すスキーマ）を $defs へ入れたもの。
/// 入れた文書は $id を持つので、$ref はそのまま引ける（JSON Schema 2020-12 の束ね方）。手順だけで形の決まりがすべて読めるようにする。
/// 指す先の文書がさらに指す文書も入れる。引けない文書（JSON Schema のメタスキーマなど）は入れない。
fn bundled(doc: &Value, validator: &dyn SchemaValidator, others: &[(String, Value)]) -> Value {
    let own = doc
        .get("$id")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let mut seen = vec![own];
    let mut queue = vec![doc.clone()];
    let mut found: Vec<(String, Value)> = Vec::new();
    while let Some(d) = queue.pop() {
        let mut refs = Vec::new();
        external_refs(&d, &mut refs);
        for base in refs {
            if seen.contains(&base) {
                continue;
            }
            seen.push(base.clone());
            let target = others
                .iter()
                .find(|(id, _)| *id == base)
                .map(|(_, d)| d.clone())
                .or_else(|| validator.shipped(&base));
            if let Some(t) = target {
                queue.push(t.clone());
                found.push((base, t));
            }
        }
    }
    let mut out = doc.clone();
    if found.is_empty() {
        return out;
    }
    if let Some(root) = out.as_object_mut() {
        let defs = root
            .entry("$defs")
            .or_insert_with(|| Value::Object(Map::new()));
        if let Some(defs) = defs.as_object_mut() {
            for (id, d) in found {
                defs.entry(id).or_insert(d);
            }
        }
    }
    out
}

/// 文書の中の $ref のうち、ほかの文書を指すもの（# の前が空でない）の文書の名前を、出てきた順に集める。
fn external_refs(node: &Value, out: &mut Vec<String>) {
    match node {
        Value::Object(m) => {
            for (k, v) in m {
                if k == "$ref"
                    && let Some(r) = v.as_str()
                {
                    let base = r.split_once('#').map_or(r, |(b, _)| b);
                    if !base.is_empty() && !out.iter().any(|o| o == base) {
                        out.push(base.to_string());
                    }
                } else {
                    external_refs(v, out);
                }
            }
        }
        Value::Array(a) => {
            for v in a {
                external_refs(v, out);
            }
        }
        _ => {}
    }
}

/// 読まれる欄を先にする（同じなら Concrete の並び）。読み合う欄があれば断る。
fn order(path: &str, mut units: Vec<Unit>) -> Result<Vec<Unit>, Refused> {
    let names: Vec<String> = units.iter().map(|u| u.name.clone()).collect();
    let mut done: Vec<Unit> = Vec::new();
    while !units.is_empty() {
        let Some(i) = units.iter().position(|u| {
            u.reads
                .iter()
                .all(|r| !names.contains(r) || done.iter().any(|d| &d.name == r))
        }) else {
            let left: Vec<String> = units.iter().map(|u| format!("/{}", u.name)).collect();
            return Err(refused(
                "reads_cycle",
                path,
                format!(
                    "欄 {} の rule の related_fields が互いを読んでいて、書く順が決まらない。どちらかを先に置き、もう一方の rule で食い違わないかを確かめる形に Concrete を直す",
                    left.join(" ・ ")
                ),
            ));
        };
        done.push(units.remove(i));
    }
    Ok(done)
}

/// --ref の文書を読む（経路 ・ ハッシュ ・ 中身）。
fn read_refs(store: &dyn DocumentStore, refs: &[String]) -> Result<Vec<Value>, Refused> {
    refs.iter()
        .map(|r| {
            let text = store.read(r).map_err(|e| refused("not_found", r, e))?;
            if text.len() > REF_LIMIT {
                return Err(refused(
                    "ref_too_large",
                    r,
                    format!(
                        "{r} は {} バイトで、上限 {REF_LIMIT} バイトを超える。切り詰めずに断る",
                        text.len()
                    ),
                ));
            }
            Ok(json!({"path": r, "hash": ContentHash::of(text.as_bytes()).as_str(), "text": text}))
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn build_procedure(
    path: &str,
    declared: &str,
    doc: &Value,
    schema: Value,
    current: Option<&Value>,
    current_text: Option<&str>,
    request: &SemanticRequest,
    units: &[Unit],
    refs: &[Value],
    caller: &Caller,
) -> Value {
    let mut target = json!({"path": path, "schema": declared});
    let hash = current_text.map(|t| ContentHash::of(t.as_bytes()).as_str().to_string());
    if let Some(h) = &hash {
        target["hash"] = json!(h);
    }
    let mut about = Map::new();
    for k in ["title", "description"] {
        if let Some(v) = doc.get(k) {
            about.insert(k.to_string(), v.clone());
        }
    }
    for k in ["meaning", "when"] {
        if let Some(v) = doc.pointer(&format!("/x-prompt/{k}")) {
            about.insert(k.to_string(), v.clone());
        }
    }
    let steps: Vec<Value> = units
        .iter()
        .enumerate()
        .map(|(i, u)| {
            let at = format!("/{}", escape(&u.name));
            let mut s = json!({"no": i + 1, "action": "write", "pointer": at, "rules": u.rules});
            if !u.inner.is_empty() {
                s["inner_rules"] = json!(u.inner);
            }
            if !u.required {
                s["optional"] = json!(true);
            }
            for k in ["title", "description"] {
                if let Some(v) = u.field.get(k) {
                    s[k] = v.clone();
                }
            }
            for k in ["meaning", "when"] {
                if let Some(v) = u.field.pointer(&format!("/x-prompt/{k}")) {
                    s[k] = v.clone();
                }
            }
            if let Some(v) = current.and_then(|c| c.pointer(&at)) {
                s["current"] = v.clone();
            }
            if !u.reads.is_empty() {
                s["related_fields"] = json!(
                    u.reads
                        .iter()
                        .map(|r| json!({"pointer": format!("/{}", escape(r))}))
                        .collect::<Vec<_>>()
                );
            }
            s
        })
        .collect();
    // 作成は文書全体を --value で、更新は書く欄への JSON Patch を --patch で渡す。schema は書いた文書が従うスキーマの本体
    let answer = if current.is_some() {
        json!({"format": "json-patch", "schema": schema})
    } else {
        json!({"format": "document", "schema": schema})
    };
    let not_written: Vec<Value> = doc
        .get("properties")
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
        .filter(|(_, f)| f.get("x-decided-by").and_then(Value::as_str) == Some("human"))
        .map(|(n, _)| {
            json!({"pointer": format!("/{}", escape(n)), "why": "x-decided-by: human（人が決める。作成では default のまま）"})
        })
        .collect();
    let mut p = json!({
        "$schema": PROCEDURE,
        "kind": "procedure",
        "operation_kind": "write",
        "target": target,
        "about": about,
        "steps": steps,
        "answer": answer,
        "command": write_command(path, declared, hash.as_deref(), request, caller),
        "constraints": [
            "steps を no の順に行い、pointer の欄の値を、rules の rule（rule ・ why ・ check と、違反する例 bad ・ 満たす例 good）と、inner_rules の rule（欄の中の所 pointer に当たる。* は配列の要素か名前を選ばない欄）と、手順の rules（文書全体に当たる rule）を守って決める（meaning があればその読み方で、when があればその読み手として、related_fields があれば先に書いた欄の値を読み、current があればいまの値を直す形で）",
            "値は answer.schema の形（JSON Schema）に従わせる。steps に無い形の決まり（求める欄 ・ 型 ・ const など）も、そこから読んで守る",
            "optional のステップは、書かないなら値から外してよい",
            "書くのは command の semantic-agent-schema だけで、ファイルを直接書かない",
            "steps に無い欄は書かない",
            "断られたら、出力の理由を読んで直し、打ち直す",
        ],
    });
    if let Some(r) = doc
        .pointer("/x-prompt/rules")
        .and_then(Value::as_array)
        .filter(|r| !r.is_empty())
    {
        p["rules"] = json!(r);
    }
    if let Some(b) = &request.task {
        p["task"] = json!(b);
    }
    if !refs.is_empty() {
        p["refs"] = json!(refs);
    }
    if !not_written.is_empty() {
        p["not_written"] = json!(not_written);
    }
    p
}

/// 書くときに打つコマンド。作成は文書全体を --value で、更新は JSON Patch を --patch で、手順を組んだときの中身の
/// ハッシュを --if-match で渡す（手順を読んでいる間に中身が変わっていたら書かない）。受け取った --schemas ・ --range と、
/// 誰が打ったか（--session ・ --human）は、書くときにも引き継ぐ。
fn write_command(
    path: &str,
    declared: &str,
    hash: Option<&str>,
    request: &SemanticRequest,
    caller: &Caller,
) -> String {
    let mut args: Vec<&str> = match hash {
        Some(h) => vec!["update", path, "--patch", "<JSON Patch>", "--if-match", h],
        None => vec![
            "create",
            path,
            "--concrete",
            declared,
            "--value",
            "<値の JSON>",
        ],
    };
    if let Some(i) = &request.reason {
        args.extend(["--reason", i.as_str()]);
    }
    if let Some(r) = &request.request_id {
        args.extend(["--request-id", r.as_str()]);
    }
    carried(
        &args,
        Some(&request.schemas),
        request.range.as_deref(),
        caller,
    )
}
