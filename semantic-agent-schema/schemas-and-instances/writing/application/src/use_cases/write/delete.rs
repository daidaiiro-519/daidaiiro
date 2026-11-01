//! UC3 削除する（機械）：対象を消し、消す前の状態を削除の操作として記録する。
//!
//! ほかのファイルが対象を指していても探さない（参照はただの値で、SAS は参照を見分けない）。
//! Concrete は、従うインスタンスが残っていれば消さずに止める。

use super::{
    Blocked, Corpus, Refused, TargetKind, classify, field_rows, log_refused, refused, target_of,
};
use recording_application::ports::{ChangeRow, IdentifyCaller, OperationLog, StoredChange};
use recording_application::use_cases::history::{interleaved, record_change, untracked_since};
use serde_json::{Value, json};
use validation_application::ports::DocumentStore;
use writing_domain::aggregates::{ConcreteSchema, Instance};
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
    /// 同じ request_id の操作がすでにあり、消さずにそれを返したか
    pub replayed: bool,
}

fn delete_hint(path: &str) -> String {
    format!("消す：sas delete {path} --intent \"…\"")
}

fn replayed(
    stored: StoredChange,
    path: &str,
    store: &dyn DocumentStore,
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
    let (kind, concrete) = target_of(store, path, &row.fields);
    Ok(Deletion {
        before: fp(&row.before, &stored.operation)?,
        diff: diff_of(&row.fields),
        parent: stored.parent,
        operation: stored.operation,
        kind,
        path: row.target_path,
        concrete,
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
    let mut r = refused(
        code,
        path,
        format!("この Concrete に頼っている文書があるので消さない（何も変えていない）：{message}"),
    );
    r.hint = Some(Box::new(format!(
        "blocked のそれぞれの hint を片付けてから、消し直す：sas delete {} --intent \"…\"",
        r.path.as_deref().unwrap_or_default()
    )));
    r.blocked = Some(Box::new(blocked));
    r
}

/// UC3 削除する（機械）：前提の指紋と、Concrete なら従うインスタンスが無いことを確かめてから、対象を消して記録する。
///
/// # Errors
/// 依頼を読めない ・ 対象が無い ・ 従う型が決まらない（何も記録しない）。
/// 指紋が違う ・ 公開した Concrete ・ 頼っている文書がある（何も変えない）。
/// 消せない ・ 記録できないとき。
pub fn delete(
    request: &DeleteRequest,
    store: &dyn DocumentStore,
    who: &dyn IdentifyCaller,
    log: &mut dyn OperationLog,
) -> Result<Deletion, Refused> {
    let path = request.path.as_str();
    if let Some(id) = &request.request_id
        && let Some(stored) = log.find_request(id).map_err(log_refused)?
    {
        return replayed(stored, path, store);
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

    // 2. Concrete に従うインスタンス
    if kind == TargetKind::Concrete
        && let Some(own) =
            ConcreteSchema::read(&before).and_then(|c| c.id().map(ToString::to_string))
    {
        let blocked: Vec<Blocked> = corpus
            .docs
            .iter()
            .filter(|d| d.path != path)
            .filter(|d| Instance::read(&d.value).is_some_and(|i| i.concrete() == own))
            .map(|d| Blocked {
                path: d.path.clone(),
                code: "follows",
                message: format!("{own} に従うインスタンス"),
                hint: delete_hint(&d.path),
            })
            .collect();
        if !blocked.is_empty() {
            return Err(refuse_with(path, "has_dependents", blocked));
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

    Ok(Deletion {
        parent,
        operation,
        kind,
        path: path.to_string(),
        concrete: declared,
        before: current,
        diff,
        replayed: false,
    })
}
