//! UC3 削除する（機械）：対象と、対象を指している参照元の記載を、同じ呼び出しの操作として消す。
//!
//! 参照元はいくつもの集約にまたがるので、1つの集約の不変条件にはせず、この手続きが業務サービスで探して判定する。
//! ファイルをまたぐトランザクションは無いので、先に全部を確かめて止められるものは止め（何も変えない）、
//! そのあと対象を消して delete を記録し、参照元を1件ずつ書いて、削除を原因（cause）とする update を記録する。

use super::{
    Blocked, Corpus, Refused, TargetKind, check_write, classify, entry, field_rows,
    form_violations, log_refused, patch, refused, target_of, to_text, violations_message,
};
use recording_application::ports::{ChangeRow, IdentifyCaller, OperationLog, StoredChange};
use recording_application::use_cases::history::{interleaved, record_change, untracked_since};
use serde_json::{Value, json};
use validation_application::ports::{DocumentStore, SchemaValidator};
use writing_domain::aggregates::{ConcreteSchema, Instance};
use writing_domain::services::{references_to, relationship_targets};
use writing_domain::values::{Change, Diff, FieldChange, Fingerprint};

/// 削除の依頼。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteRequest {
    /// 消す経路
    pub path: String,
    /// 消す前に読んだ中身の指紋。無ければ、いまの中身が記録のどれかの事後かを確かめる
    pub if_match: Option<String>,
    /// やり直しで同じ操作を2回残さないための ID
    pub request_id: Option<String>,
    /// 意図（なぜ消すか）
    pub intent: Option<String>,
}

/// 削除に伴って記載を消した参照元1つ。
#[derive(Debug, Clone, PartialEq)]
pub struct Rewritten {
    /// 書き換えの操作の ID
    pub operation: String,
    /// 経路
    pub path: String,
    /// 従う Concrete の $id
    pub concrete: String,
    /// 事前の指紋
    pub before: Fingerprint,
    /// 事後の指紋
    pub after: Fingerprint,
    /// 消した記載
    pub diff: Diff,
}

/// 削除の結果。
#[derive(Debug, Clone, PartialEq)]
pub struct Deletion {
    /// 呼んだ親セッションの ID（人なら None）
    pub parent: Option<String>,
    /// 削除の操作の ID
    pub operation: String,
    /// 対象の種類
    pub kind: TargetKind,
    /// 経路
    pub path: String,
    /// 従う Concrete（Concrete なら Abstract の版）の $id
    pub concrete: String,
    /// 消す前の指紋
    pub before: Fingerprint,
    /// 消す前の状態（根の欄ごとの remove）
    pub diff: Diff,
    /// 記載を消した参照元（記録した順）
    pub references: Vec<Rewritten>,
    /// 同じ request_id の操作がすでにあり、消さずにそれを返したか
    pub replayed: bool,
}

/// 参照元1つの、記載を消した姿。
struct Plan {
    path: String,
    concrete: String,
    before_text: String,
    before: Value,
    after: Value,
}

fn delete_hint(path: &str) -> String {
    format!("消す：sas delete {path} --intent \"…\"")
}

/// 参照元の記載を消した姿（配列の要素は後ろから外し、1つの値の欄は欄ごと外す）。
fn without(doc: &Value, pointers: &[String]) -> Result<Value, String> {
    let mut sorted: Vec<&String> = pointers.iter().collect();
    // 同じ配列の要素は、後ろの添字から外す（前から外すと後ろの添字がずれる）
    sorted.sort_by(|a, b| {
        let key = |p: &str| -> (String, i64) {
            match p.rsplit_once('/') {
                Some((head, last)) => (head.to_string(), last.parse::<i64>().unwrap_or(-1)),
                None => (p.to_string(), -1),
            }
        };
        let (ha, ia) = key(a);
        let (hb, ib) = key(b);
        ha.cmp(&hb).then(ib.cmp(&ia))
    });
    let ops: Vec<Value> = sorted
        .iter()
        .map(|p| json!({"op": "remove", "path": p}))
        .collect();
    patch::apply(doc, &Value::Array(ops))
}

fn replayed(
    stored: StoredChange,
    path: &str,
    store: &dyn DocumentStore,
    log: &dyn OperationLog,
) -> Result<Deletion, Refused> {
    let row = stored.row;
    if row.kind != "delete" || row.target_path != path {
        return Err(refused(
            "request_reused",
            path,
            format!(
                "request_id は {} の {} に使った。別の依頼には別の request_id を使う",
                row.target_path, row.kind
            ),
        ));
    }
    let parse = |s: &Option<String>| s.as_deref().and_then(|t| serde_json::from_str(t).ok());
    let diff_of = |fields: &[recording_application::ports::FieldRow]| {
        Diff::from_changes(
            fields
                .iter()
                .filter_map(|f| {
                    Some(FieldChange::new(
                        &f.pointer,
                        Change::parse(&f.change)?,
                        parse(&f.before),
                        parse(&f.after),
                    ))
                })
                .collect(),
        )
    };
    let fp = |s: &Option<String>, op: &str| {
        s.as_deref()
            .and_then(Fingerprint::parse)
            .ok_or_else(|| log_refused(format!("操作 {op} の指紋を読めない")))
    };
    let mut references = Vec::new();
    for c in log.caused_by(&stored.operation).map_err(log_refused)? {
        let (_, concrete) = target_of(store, &c.row.target_path, &c.row.fields);
        references.push(Rewritten {
            before: fp(&c.row.before, &c.operation)?,
            after: fp(&c.row.after, &c.operation)?,
            diff: diff_of(&c.row.fields),
            path: c.row.target_path,
            concrete,
            operation: c.operation,
        });
    }
    let (kind, concrete) = target_of(store, path, &row.fields);
    Ok(Deletion {
        before: fp(&row.before, &stored.operation)?,
        diff: diff_of(&row.fields),
        parent: stored.parent,
        operation: stored.operation,
        kind,
        path: row.target_path,
        concrete,
        references,
        replayed: true,
    })
}

/// 止めた削除の理由に、止めた理由になったものを付けて返す（記録しない）。
fn refuse_with(path: &str, code: &'static str, blocked: Vec<Blocked>) -> Refused {
    let message = blocked
        .iter()
        .map(|b| format!("{}：{}", b.path, b.message))
        .collect::<Vec<_>>()
        .join(" ／ ");
    let lead = match code {
        "has_dependents" => "この Concrete に頼っている文書があるので消さない（何も変えていない）",
        _ => "参照元の記載を消せないものがあるので、対象も参照元も変えない",
    };
    let mut r = refused(code, path, format!("{lead}：{message}"));
    r.hint = Some(Box::new(format!(
        "blocked のそれぞれの hint を片付けてから、消し直す：sas delete {} --intent \"…\"",
        r.path.as_deref().unwrap_or_default()
    )));
    r.blocked = Some(Box::new(blocked));
    r
}

/// UC3 削除する（機械）：対象を指している参照元を探し、全部の記載を消せると確かめてから、対象を消して参照元を書き換え、
/// 削除と、削除を原因とする書き換えを同じセッションの操作として記録する。
///
/// # Errors
/// 依頼を読めない ・ 対象が無い ・ 従う型が決まらない（何も記録しない）。
/// 指紋が違う ・ 公開した Concrete ・ 頼っている文書がある ・ 参照元の記載を消せない（何も変えずに記録する）。
/// 消せない ・ 記録できないとき。参照元を書く途中で止まったときは、そこまでの記録と、残った参照元を返す。
pub fn delete(
    request: &DeleteRequest,
    store: &dyn DocumentStore,
    validator: &dyn SchemaValidator,
    who: &dyn IdentifyCaller,
    log: &mut dyn OperationLog,
) -> Result<Deletion, Refused> {
    let path = request.path.as_str();
    if let Some(id) = &request.request_id
        && let Some(stored) = log.find_request(id).map_err(log_refused)?
    {
        return replayed(stored, path, store, log);
    }
    let text = store
        .read(path)
        .map_err(|e| refused("not_found", path, e))?;
    let before: Value = serde_json::from_str(&text)
        .map_err(|e| refused("not_json", path, format!("JSON として読めない：{e}")))?;
    let declared = before
        .get("$schema")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let corpus = Corpus::load(store)?;
    let kind = classify(&corpus.concretes, path, &declared)?;
    let current = Fingerprint::of(text.as_bytes());
    let row = ChangeRow {
        kind: "delete".to_string(),
        source: "sas".to_string(),
        target_path: path.to_string(),
        request_id: request.request_id.clone(),
        before: Some(current.as_str().to_string()),
        after: None,
        intent: request.intent.clone(),
        cause: None,
        fields: Vec::new(),
    };

    // 1. 消す前提の指紋（更新と同じ決まり）
    let (expected, from) = match &request.if_match {
        Some(fp) => (Some(fp.clone()), "--if-match の"),
        None => (
            untracked_since(log, path, current.as_str()).map_err(log_refused)?,
            "SAS が最後に書いた事後の",
        ),
    };
    if let Some(expected) = expected.filter(|e| e != current.as_str()) {
        let between = interleaved(log, path, &expected, current.as_str()).map_err(log_refused)?;
        let mut r = refused(
            "conflict",
            path,
            format!(
                "いまの中身の指紋は {}。{from}指紋 {expected} と違うので消さない。間に入った変更（interleaved）を読んでから消す{}",
                current.as_str(),
                if between.untracked {
                    "。SAS を通らない変更がある"
                } else {
                    ""
                }
            ),
        );
        r.hint = Some(Box::new(if between.untracked {
            format!(
                "外の変更を取り込んでから消す：sas update {path} --adopt --intent \"…\" のあとで sas delete {path} --intent \"…\""
            )
        } else {
            format!(
                "間に入った変更を読み、それでも消すなら打ち直す：sas delete {path} --if-match {} --intent \"…\"",
                current.as_str()
            )
        }));
        r.interleaved = Some(Box::new(between));
        return Err(r);
    }

    // 2. 頼っている文書と、参照元の記載を消した姿
    let mut blocked: Vec<Blocked> = Vec::new();
    let mut plans: Vec<Plan> = Vec::new();
    match kind {
        TargetKind::Concrete => {
            let concrete = ConcreteSchema::read(&before);
            let own = concrete
                .as_ref()
                .and_then(|c| c.id().map(ToString::to_string));
            if let Some(own) = &own {
                for d in &corpus.docs {
                    if d.path == path {
                        continue;
                    }
                    if Instance::read(&d.value).is_some_and(|i| i.concrete() == own) {
                        blocked.push(Blocked {
                            path: d.path.clone(),
                            code: "follows",
                            message: format!("{own} に従うインスタンス"),
                            hint: delete_hint(&d.path),
                        });
                    } else if ConcreteSchema::read(&d.value).is_some() {
                        let points: Vec<String> = relationship_targets(&d.value)
                            .into_iter()
                            .filter(|(_, to)| to == own)
                            .map(|(at, _)| at)
                            .collect();
                        if !points.is_empty() {
                            blocked.push(Blocked {
                                path: d.path.clone(),
                                code: "points_to",
                                message: format!(
                                    "x-relationship で {own} を指している（{}）",
                                    points.join(" ・ ")
                                ),
                                hint: format!(
                                    "指す先を直す：sas update {} --patch <JSON Patch> --intent \"…\"",
                                    d.path
                                ),
                            });
                        }
                    }
                }
            }
            if !blocked.is_empty() {
                return Err(refuse_with(path, "has_dependents", blocked));
            }
        }
        TargetKind::Instance => {
            let id = before.get("id").and_then(Value::as_str).unwrap_or_default();
            for d in corpus
                .docs
                .iter()
                .filter(|d| !id.is_empty() && d.path != path)
            {
                let Some(inst) = Instance::read(&d.value) else {
                    continue;
                };
                let Some(c) = corpus.concretes.get(inst.concrete()) else {
                    continue;
                };
                let pointers = references_to(c.document(), &d.value, &declared, id);
                if pointers.is_empty() {
                    continue;
                }
                let mut block = |code: &'static str, message: String, hint: String| {
                    blocked.push(Blocked {
                        path: d.path.clone(),
                        code,
                        message,
                        hint,
                    });
                };
                let fp = Fingerprint::of(d.text.as_bytes());
                if let Some(last) =
                    untracked_since(log, &d.path, fp.as_str()).map_err(log_refused)?
                {
                    block(
                        "conflict",
                        format!(
                            "最後に SAS で書いた指紋 {last} といまの指紋 {} が違う（SAS を通らない変更がある）",
                            fp.as_str()
                        ),
                        format!(
                            "外の変更を取り込む：sas update {} --adopt --intent \"…\"",
                            d.path
                        ),
                    );
                    continue;
                }
                let after = match without(&d.value, &pointers) {
                    Ok(v) => v,
                    Err(e) => {
                        block(
                            "invalid_value",
                            format!("記載（{}）を外せない：{e}", pointers.join(" ・ ")),
                            format!(
                                "記載を直す：sas update {} --patch <JSON Patch> --intent \"…\"",
                                d.path
                            ),
                        );
                        continue;
                    }
                };
                let broken = form_violations(
                    validator,
                    TargetKind::Instance,
                    inst.concrete(),
                    &after,
                    &corpus.more,
                )
                .map_err(|e| refused("validator", &d.path, e))?;
                if !broken.is_empty() {
                    block(
                        "invalid_value",
                        format!(
                            "記載（{}）を外すと型に従わない：{}",
                            pointers.join(" ・ "),
                            violations_message(&broken)
                        ),
                        format!(
                            "{id} の代わりの値を入れる：sas update {} --patch <JSON Patch> --intent \"…\"",
                            d.path
                        ),
                    );
                    continue;
                }
                if let Err((code, message)) = check_write(
                    TargetKind::Instance,
                    Some(&d.value),
                    &after,
                    Some(c),
                    entry(who),
                ) {
                    let hint = if code == "human_field" {
                        format!("人が --session を付けずに消す：sas delete {path} --intent \"…\"")
                    } else {
                        format!(
                            "記載を直す：sas update {} --patch <JSON Patch> --intent \"…\"",
                            d.path
                        )
                    };
                    block(code, message, hint);
                    continue;
                }
                plans.push(Plan {
                    path: d.path.clone(),
                    concrete: inst.concrete().to_string(),
                    before_text: d.text.clone(),
                    before: d.value.clone(),
                    after,
                });
            }
            if !blocked.is_empty() {
                return Err(refuse_with(path, "references_blocked", blocked));
            }
        }
    }

    // 3. 対象を消し、事前の状態を記録する
    store.remove(path).map_err(|e| refused("store", path, e))?;
    let diff = Diff::between(Some(&before), &json!({}));
    let recorded = record_change(
        who,
        log,
        &ChangeRow {
            fields: field_rows(&diff),
            ..row
        },
    )
    .map_err(log_refused)?;
    let parent = recorded.parent;
    let operation = recorded.operations.into_iter().next().unwrap_or_default();

    // 4. 参照元を1件ずつ書き、削除を原因とする update を記録する
    let mut references = Vec::new();
    for (i, p) in plans.iter().enumerate() {
        let new_text = to_text(&p.after);
        if let Err(e) = store.write(&p.path, &new_text) {
            return Err(Refused {
                code: "references_left",
                path: Some(p.path.as_str().into()),
                message: format!(
                    "{path} は消して記録した（{operation}）が、参照元 {} を書けない：{e}。残った参照元は blocked にある",
                    p.path
                ),
                operation: Some(operation),
                hint: Some(Box::new(
                    "残った参照元の記載を直す（次の検証で relationship-target として見える）"
                        .to_string(),
                )),
                blocked: Some(Box::new(
                    plans[i..]
                        .iter()
                        .map(|q| Blocked {
                            path: q.path.clone(),
                            code: "left",
                            message: format!("{path} を指す記載が残っている"),
                            hint: format!(
                                "記載を直す：sas update {} --patch <JSON Patch> --intent \"…\"",
                                q.path
                            ),
                        })
                        .collect(),
                )),
                ..Refused::default()
            });
        }
        let before_fp = Fingerprint::of(p.before_text.as_bytes());
        let after_fp = Fingerprint::of(new_text.as_bytes());
        let d = Diff::between(Some(&p.before), &p.after);
        let r = record_change(
            who,
            log,
            &ChangeRow {
                kind: "update".to_string(),
                source: "sas".to_string(),
                target_path: p.path.clone(),
                request_id: None,
                before: Some(before_fp.as_str().to_string()),
                after: Some(after_fp.as_str().to_string()),
                intent: Some(format!("{path} を消したので、指している記載を消す")),
                cause: Some(operation.clone()),
                fields: field_rows(&d),
            },
        )
        .map_err(log_refused)?;
        references.push(Rewritten {
            operation: r.operations.into_iter().next().unwrap_or_default(),
            path: p.path.clone(),
            concrete: p.concrete.clone(),
            before: before_fp,
            after: after_fp,
            diff: d,
        });
    }
    Ok(Deletion {
        parent,
        operation,
        kind,
        path: path.to_string(),
        concrete: declared,
        before: current,
        diff,
        references,
        replayed: false,
    })
}

#[cfg(test)]
mod tests {
    use super::without;
    use serde_json::json;

    #[test]
    fn removes_array_items_from_the_back_and_whole_fields() {
        let doc = json!({"calls": ["U1", "U2", "U1"], "parent": "U1", "goal": "g"});
        let out = without(
            &doc,
            &[
                "/calls/0".to_string(),
                "/calls/2".to_string(),
                "/parent".to_string(),
            ],
        );
        assert_eq!(out, Ok(json!({"calls": ["U2"], "goal": "g"})));
    }
}
