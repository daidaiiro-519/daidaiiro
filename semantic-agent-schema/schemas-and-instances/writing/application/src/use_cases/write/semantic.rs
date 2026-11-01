//! UC1 作成する ・ UC2 更新する の意味で書く変形：Concrete の rule から書く手順を組み、SAS が起動した書き込み用の子セッションに
//! 欄の値を書かせ、答えを答えの形 ・ Concrete の形に照らしてから、作成 ・ 更新と同じ手続きで書く。
//!
//! - 書く欄は x-prompt を持つ根の欄。人が決める欄（x-decided-by: human）と、const で決まる欄は書かせない。
//! - 書く順は rule の reads：読まれる欄を先にする。読み合う欄があれば、手順を出さずに断る。
//! - --ref の文書は SAS が読んで中身を手順に入れる（子セッションはファイルを開けない）。上限を超えたら切り詰めずに断る。
//! - 書くのは SAS で、子セッションではない。答えが形に合わなければ同じ子セッションに2回まで直させる。

use super::{ChangeResult, CreateRequest, UpdateRequest, create, update, violations_message};
use recording_application::ports::{ChildRow, IdentifyCaller, OperationLog};
use serde_json::{Map, Value, json};
use validation_application::ports::{DocumentStore, ReadMeaning, SchemaValidator};
use validation_application::use_cases::meaning::{Child, converse, field_schema};
use validation_application::use_cases::validate::{
    Corpus, Refused, TargetKind, form_violations, log_refused, refused,
};
use writing_domain::aggregates::escape;
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
}

/// 書く欄1つ。
struct Unit {
    name: String,
    field: Value,
    rules: Vec<Value>,
    reads: Vec<String>,
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
    let Some(concrete) = corpus.concretes.get(&declared) else {
        return Err(refused(
            "unknown_schema",
            path,
            format!(
                "{declared} はリポジトリの Concrete に無い（意味で書けるのはインスタンスだけ）"
            ),
        ));
    };
    let doc = concrete.document();
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
    let mut child = Child {
        tool_ref: None,
        launch: reader.launch(),
    };
    let conversed = converse(reader, &mut child, &first, &schema, &|answer| {
        let candidate = read_values(validator, answer, &units, &base)?;
        let broken = form_violations(
            validator,
            TargetKind::Instance,
            &declared,
            &candidate,
            &corpus.more,
        )?;
        if broken.is_empty() {
            Ok(candidate)
        } else {
            Err(format!(
                "Concrete の形に合わない：{}",
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

/// 書く欄：x-prompt を持つ根の欄。人が決める欄と const で決まる欄は外す。
fn write_units(doc: &Value, others: &[(String, Value)]) -> Vec<Unit> {
    let Some(props) = doc.get("properties").and_then(Value::as_object) else {
        return Vec::new();
    };
    props
        .keys()
        .filter_map(|name| {
            let field = field_schema(doc, &format!("/{}", escape(name)), others)?;
            let prompt = field.get("x-prompt")?;
            if field.get("const").is_some()
                || field.get("x-decided-by").and_then(Value::as_str) == Some("human")
            {
                return None;
            }
            let rules: Vec<Value> = prompt
                .get("write")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
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
                reads,
            })
        })
        .collect()
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
         それぞれの pointer の欄に書く値を、rules の rule（rule ・ why ・ test。bad ・ good があれば例として読む）を守って決めてください\
         （read があればその読み方で、reads があれば先に書いた欄の値を読んで、current があればいまの値を直す形で）。\n\
         答えの形に合う JSON だけを返してください。ファイルを書かず、steps に無い欄に触れないでください。\n\n\
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
            obj.insert(unit.name.clone(), item["value"].clone());
        }
    }
    let missing: Vec<String> = (1..=units.len())
        .filter(|n| !seen.contains(n))
        .map(|n| n.to_string())
        .collect();
    if !missing.is_empty() {
        return Err(format!("step {} の値が無い", missing.join(" ・ ")));
    }
    Ok(out)
}
