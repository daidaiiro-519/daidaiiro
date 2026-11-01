//! UC1 作成する ・ UC2 更新する の意味で書く変形：Concrete の rule から書く手順を組み、SAS が起動した書き込み用の子セッションに
//! 欄の値を書かせ、答えを答えの形 ・ Concrete の形に照らしてから、作成 ・ 更新と同じ手続きで書く。
//!
//! - 書く欄は根の欄（x-prompt か、中に rule を持つ欄か、根が求める欄）。人が決める欄（x-decided-by: human）と、
//!   const で決まる欄と、$schema は書かせない。欄の中（入れ子 ・ 配列の要素 ・ $ref の先）の rule も、そのステップに入れる。
//! - インスタンスは Concrete の rule で、Concrete は Abstract の rule で書く（同じ手続き）。
//! - 書き込み用の子セッションは --continue で続けられる（記録の起動の設定と同じ接続の定義でだけ）。
//! - 書く順は rule の reads：読まれる欄を先にする。読み合う欄があれば、手順を出さずに断る。
//! - --ref の文書は SAS が読んで中身を手順に入れる（子セッションはファイルを開けない）。上限を超えたら切り詰めずに断る。
//! - 書くのは SAS で、子セッションではない。答えが形に合わなければ同じ子セッションに2回まで直させる。

use super::{ChangeResult, CreateRequest, UpdateRequest, create, update, violations_message};
use recording_application::ports::{ChildRow, IdentifyCaller, OperationLog};
use serde_json::{Map, Value, json};
use validation_application::ports::{DocumentStore, ReadMeaning, SchemaValidator};
use validation_application::use_cases::meaning::{Child, converse, field_schema};
use validation_application::use_cases::validate::{
    Corpus, Refused, TargetKind, classify, form_violations, log_refused, refused,
};
use validation_domain::services::builtin_checks;
use validation_domain::values::Verdict;
use writing_domain::aggregates::{ConcreteSchema, escape};
use writing_domain::values::ContentHash;

/// 書き込みの答えの形（公開しない。子セッションに渡し、答えを照らす）。
pub const WRITER_ANSWER_SCHEMA: &str = include_str!("../../writer-answer.schema.json");
const WRITER_ANSWER_ID: &str = "urn:semantic-agent-schema:writing:agent-answer-values:v1";
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
    /// 何を書くかの依頼（--brief）
    pub brief: Option<String>,
    /// 読ませる文書の経路（--ref）
    pub refs: Vec<String>,
    /// 意図
    pub intent: Option<String>,
    /// やり直しで同じ操作を2回残さないための ID
    pub request_id: Option<String>,
    /// 続ける書き込み用の子セッションの ID（--continue）。無ければ新しい子セッションを起動する
    pub continue_child: Option<String>,
}

/// 意味で書いた結果：渡した手順と、書き込み用の子セッションのこと。
#[derive(Debug, Clone, PartialEq)]
pub struct SemanticOutcome {
    /// 渡した手順のプロンプト
    pub procedure: Value,
    /// 書いた子セッションの ID（起動できなければ None）
    pub child: Option<String>,
    /// 新しく起動した子セッションの ID
    pub started: Vec<String>,
    /// 形の誤りを直させた回数
    pub corrections: u32,
    /// 時間切れで新しい子セッションにした回数
    pub restarts: u32,
    /// --continue で続けた子セッションか
    pub continued: bool,
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

/// 意味で書く（UC1 ・ UC2 の変形）。
///
/// # Errors
/// 依頼を読めない ・ 型が決まらない ・ 書く欄が無い ・ 読み合う欄がある ・ 資料を読めない ・ 大きすぎる ・
/// 子セッションが処理中 ・ 答えを得られない（記録しない）。書く手前で断られたとき（作成 ・ 更新と同じ）。
pub fn semantic_write(
    request: &SemanticRequest,
    store: &dyn DocumentStore,
    validator: &dyn SchemaValidator,
    who: &dyn IdentifyCaller,
    log: &mut dyn OperationLog,
    reader: &dyn ReadMeaning,
) -> Result<(ChangeResult, SemanticOutcome), Refused> {
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
    let corpus = Corpus::load(store)?;
    let kind = classify(&corpus.concretes, path, &declared)?;
    // インスタンスは Concrete の rule で、Concrete は Abstract の rule で書く
    let doc: Value = match kind {
        TargetKind::Instance => corpus
            .concretes
            .get(&declared)
            .map(|c| c.document().clone())
            .unwrap_or_default(),
        TargetKind::Concrete => validator.shipped(&declared).unwrap_or_default(),
    };
    let doc = &doc;
    let units = order(path, write_units(doc, &corpus.more))?;
    if units.is_empty() {
        return Err(refused(
            "invalid_input",
            path,
            "x-prompt を持ち、子セッションに書かせてよい欄が無い".to_string(),
        ));
    }
    let refs = read_refs(store, &request.refs)?;
    let procedure = build_procedure(
        path,
        &declared,
        doc,
        current.as_ref(),
        current_text.as_deref(),
        request,
        &units,
        &refs,
    );
    let schema: Value = serde_json::from_str(WRITER_ANSWER_SCHEMA).unwrap_or_default();
    let first = prompt_text(&procedure, &schema);
    let base = current
        .clone()
        .unwrap_or_else(|| json!({"$schema": declared}));
    let mut child = continued_child(log, reader, path, request.continue_child.as_deref())?;
    let conversed = converse(reader, &mut child, &first, &schema, &|answer| {
        let candidate = read_values(validator, answer, &units, &base)?;
        let mut broken = form_violations(validator, kind, &declared, &candidate, &corpus.more)?;
        // Concrete は JSON Schema で書けない組み込みの検査にも合わせる
        if kind == TargetKind::Concrete
            && let Some(c) = ConcreteSchema::read(&candidate)
        {
            broken.extend(
                builtin_checks(&c)
                    .into_iter()
                    .filter(|r| r.verdict() == Verdict::Violated)
                    .map(|r| (r.pointer().to_string(), r.reason().to_string())),
            );
        }
        if broken.is_empty() {
            Ok(candidate)
        } else {
            Err(format!(
                "書く元のスキーマの形に合わない：{}",
                violations_message(&broken)
            ))
        }
    })
    .map_err(|code| refused(code, path, "書き込み用の子セッションが処理中".to_string()))?;
    let parent = log.parent_session(&who.identify()).map_err(log_refused)?;
    let launch = child.launch.to_string();
    for id in &conversed.started {
        log.append_child(
            parent.as_deref(),
            &ChildRow {
                id: id.clone(),
                role: "writer".to_string(),
                tool: reader.tool(),
                model: reader.model(),
                launch: launch.clone(),
            },
        )
        .map_err(log_refused)?;
    }
    let outcome = SemanticOutcome {
        procedure,
        child: conversed.child.clone(),
        started: conversed.started.clone(),
        corrections: conversed.corrections,
        restarts: conversed.restarts,
        continued: request.continue_child.is_some(),
    };
    let candidate = conversed
        .outcome
        .map_err(|why| refused("writer_failed", path, why))?;
    let changed = match (&request.concrete, current_text) {
        (Some(c), _) => create(
            &CreateRequest {
                path: path.to_string(),
                concrete: c.clone(),
                value: candidate,
                request_id: request.request_id.clone(),
                intent: request.intent.clone(),
                writer: conversed.child,
            },
            store,
            validator,
            who,
            log,
        )?,
        (None, Some(text)) => {
            let ops: Vec<Value> = units
                .iter()
                .filter_map(|u| {
                    let at = format!("/{}", escape(&u.name));
                    let new = candidate.pointer(&at)?;
                    match base.pointer(&at) {
                        Some(old) if old == new => None,
                        Some(_) => Some(json!({"op": "replace", "path": at, "value": new})),
                        None => Some(json!({"op": "add", "path": at, "value": new})),
                    }
                })
                .collect();
            update(
                &UpdateRequest {
                    path: path.to_string(),
                    patch: Value::Array(ops),
                    // 子セッションが書く間に中身が変わっていたら書かない
                    if_match: Some(ContentHash::of(text.as_bytes()).as_str().to_string()),
                    request_id: request.request_id.clone(),
                    intent: request.intent.clone(),
                    writer: conversed.child,
                },
                store,
                validator,
                who,
                log,
            )?
        }
        (None, None) => return Err(refused("not_found", path, String::new())),
    };
    Ok((changed, outcome))
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
                .pointer("/x-prompt/write")
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
                    r.get("reads")
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
                .pointer("/x-prompt/write")
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

/// --continue のときは記録の書き込み用の子セッションを、同じ接続の定義でだけ続ける。無ければ新しい子セッション。
fn continued_child(
    log: &dyn OperationLog,
    reader: &dyn ReadMeaning,
    path: &str,
    id: Option<&str>,
) -> Result<Child, Refused> {
    let Some(id) = id else {
        return Ok(Child {
            tool_ref: None,
            launch: reader.launch(),
        });
    };
    let rec = log
        .child(id)
        .map_err(log_refused)?
        .ok_or_else(|| refused("not_found", path, format!("子セッション {id} は記録に無い")))?;
    if rec.row.role != "writer" {
        return Err(refused(
            "checker_as_writer",
            path,
            format!(
                "{id} は検証用の子セッション（{}）で、書き込み用の子セッションとして続けられない。判定した子セッションに書かせない",
                rec.row.role
            ),
        ));
    }
    let recorded: Value = serde_json::from_str(&rec.row.launch).unwrap_or_default();
    let now = reader.launch();
    if ["connector", "version", "hash"]
        .iter()
        .any(|k| recorded.get(*k) != now.get(*k))
    {
        return Err(refused(
            "launch_mismatch",
            path,
            format!(
                "子セッション {id} を起動した接続の定義といまの定義の名前 ・ 版 ・ 中身のハッシュが違う。同じ定義を --agent に渡す"
            ),
        ));
    }
    Ok(Child {
        tool_ref: Some(id.to_string()),
        launch: recorded,
    })
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
                    "欄 {} の rule の reads が互いを読んでいて、書く順が決まらない。どちらかを先に置き、もう一方の rule で食い違わないかを確かめる形に Concrete を直す",
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
    current: Option<&Value>,
    current_text: Option<&str>,
    request: &SemanticRequest,
    units: &[Unit],
    refs: &[Value],
) -> Value {
    let mut target = json!({"path": path, "concrete": declared});
    if let Some(t) = current_text {
        target["hash"] = json!(ContentHash::of(t.as_bytes()).as_str());
    }
    let mut about = Map::new();
    for k in ["title", "description"] {
        if let Some(v) = doc.get(k) {
            about.insert(k.to_string(), v.clone());
        }
    }
    if let Some(r) = doc.pointer("/x-prompt/read") {
        about.insert("read".to_string(), r.clone());
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
            if let Some(r) = u.field.pointer("/x-prompt/read") {
                s["read"] = r.clone();
            }
            if let Some(v) = current.and_then(|c| c.pointer(&at)) {
                s["current"] = v.clone();
            }
            if !u.reads.is_empty() {
                s["reads"] = json!(
                    u.reads
                        .iter()
                        .map(|r| json!({"pointer": format!("/{}", escape(r))}))
                        .collect::<Vec<_>>()
                );
            }
            s
        })
        .collect();
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
        "answer": {"schema": "values"},
        "constraints": [
            "答えの JSON だけを返す（ほかの文を付けない）",
            "steps に無い欄は書かない",
            "ファイルを書かない（書くのは SAS）",
        ],
    });
    if let Some(r) = doc
        .pointer("/x-prompt/write")
        .and_then(Value::as_array)
        .filter(|r| !r.is_empty())
    {
        p["rules"] = json!(r);
    }
    if let Some(b) = &request.brief {
        p["brief"] = json!(b);
    }
    if !refs.is_empty() {
        p["refs"] = json!(refs);
    }
    if !not_written.is_empty() {
        p["not_written"] = json!(not_written);
    }
    p
}

/// 子セッションに渡すプロンプトの文。
#[must_use]
pub fn prompt_text(procedure: &Value, answer_schema: &Value) -> String {
    format!(
        "あなたは SAS（Semantic Agent Schema）が起動した書き込み用の子セッションです。\n\
         次の手順のプロンプトの about が書く文書、brief が依頼、refs が資料です。steps を no の順に行い、\
         それぞれの pointer の欄に書く値を、rules の rule（rule ・ why ・ test。bad ・ good があれば例として読む）と、inner_rules の rule（その欄の中の所 pointer に当たる。* は配列の要素か名前を選ばない欄）と、手順の rules（文書全体に当たる rule）を守って決めてください\
         （read があればその読み方で、reads があれば先に書いた欄の値を読んで、current があればいまの値を直す形で）。\n\
         optional のステップは、書かないなら答えから外してかまいません。答えの形に合う JSON だけを返してください。ファイルを書かず、steps に無い欄に触れないでください。\n\n\
         手順のプロンプト：\n{}\n\n答えの形：\n{}\n",
        serde_json::to_string_pretty(procedure).unwrap_or_default(),
        serde_json::to_string_pretty(answer_schema).unwrap_or_default()
    )
}

/// 答えを読み、答えの形に照らし、steps と突き合わせて、書いた姿（base に値を入れたもの）を返す。
fn read_values(
    validator: &dyn SchemaValidator,
    answer: &str,
    units: &[Unit],
    base: &Value,
) -> Result<Value, String> {
    let v: Value =
        serde_json::from_str(answer.trim()).map_err(|e| format!("JSON として読めない（{e}）"))?;
    let schema: Value = serde_json::from_str(WRITER_ANSWER_SCHEMA).unwrap_or_default();
    let broken = validator.check(
        WRITER_ANSWER_ID,
        &v,
        &[(WRITER_ANSWER_ID.to_string(), schema)],
    )?;
    if !broken.is_empty() {
        return Err(broken
            .iter()
            .map(|(at, why)| format!("{at}：{why}"))
            .collect::<Vec<_>>()
            .join(" ／ "));
    }
    let mut out = base.clone();
    let mut seen = Vec::new();
    for item in v["values"].as_array().into_iter().flatten() {
        let no = item["step"]
            .as_u64()
            .and_then(|n| usize::try_from(n).ok())
            .unwrap_or_default();
        let unit = no
            .checked_sub(1)
            .and_then(|i| units.get(i))
            .ok_or_else(|| format!("手順に無い step {no}"))?;
        let at = format!("/{}", escape(&unit.name));
        if item["pointer"] != json!(at) {
            return Err(format!("step {no} の pointer が手順と違う（手順は {at}）"));
        }
        if seen.contains(&no) {
            return Err(format!("step {no} の値が2つある"));
        }
        seen.push(no);
        if let Some(obj) = out.as_object_mut() {
            if item["value"].is_null() && !unit.required {
                obj.remove(&unit.name);
            } else {
                obj.insert(unit.name.clone(), item["value"].clone());
            }
        }
    }
    let missing: Vec<String> = (1..=units.len())
        .filter(|n| !seen.contains(n) && units[n - 1].required)
        .map(|n| n.to_string())
        .collect();
    if !missing.is_empty() {
        return Err(format!("step {} の値が無い", missing.join(" ・ ")));
    }
    Ok(out)
}
