//! UC3 削除する（機械）：対象を消し、消す前の状態を削除の操作として記録する。
//!
//! ほかのファイルが対象を指していても探さない（参照はただの値で、SAS は参照を見分けない）。
//! Concrete は、従うインスタンスが残っていれば消さずに止める。

use super::{
    Blocked, Refused, classify, field_rows, interleaved, load_corpus, log_refused, record_change,
    refused, target_of,
};
use crate::ports::{ChangeLog, DocumentStore};
use serde_json::{Value, json};
use writing_domain::aggregates::{ConcreteSchema, Instance};
use writing_domain::values::{
    Caller, Change, ChangeRow, ContentHash, Diff, FieldChange, FieldRow, StoredChange, TargetKind,
    untracked_since,
};

/// 削除の依頼。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteRequest {
    /// 消す経路
    pub path: String,
    /// 消す前に読んだ中身のハッシュ。無ければ、いまの中身が記録のどれかの事後かを確かめる
    pub if_match: Option<String>,
    /// やり直しで同じ操作を2回残さないための ID
    pub request_id: Option<String>,
    /// 意図（なぜ消すか）
    pub intent: Option<String>,
}

/// 削除の結果。
#[derive(Debug, Clone, PartialEq)]
pub struct Deletion {
    /// 打ったセッションの ID（人なら None）
    pub session: Option<String>,
    /// 削除の操作の ID
    pub operation: String,
    /// 対象の種類
    pub kind: TargetKind,
    /// 経路
    pub path: String,
    /// 従う Concrete（Concrete なら Abstract の版）の $id
    pub concrete: String,
    /// 消す前のハッシュ
    pub before: ContentHash,
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
    let diff_of = |fields: &[FieldRow]| {
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
    let hash = |s: &Option<String>, op: &str| {
        s.as_deref()
            .and_then(ContentHash::parse)
            .ok_or_else(|| log_refused(format!("操作 {op} のハッシュを読めない")))
    };
    let (kind, concrete) = target_of(store, path, &row.fields);
    Ok(Deletion {
        before: hash(&row.before, &stored.operation)?,
        diff: diff_of(&row.fields),
        session: stored.session,
        operation: stored.operation,
        kind,
        path: row.target_path,
        concrete,
        replayed: true,
    })
}

/// Concrete に従うインスタンスがあるので止めた削除を、従うインスタンスと一緒に返す（記録しない）。
fn has_dependents(path: &str, blocked: Vec<Blocked>) -> Refused {
    let message = blocked
        .iter()
        .map(|b| format!("{}：{}", b.path, b.message))
        .collect::<Vec<_>>()
        .join(" ／ ");
    let mut r = refused(
        "has_dependents",
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

/// UC3 削除する（機械）：前提のハッシュと、Concrete なら従うインスタンスが無いことを確かめてから、対象を消して記録する。
///
/// # Errors
/// 依頼を読めない ・ 対象が無い ・ 従う型が決まらない（何も記録しない）。
/// ハッシュが違う ・ Concrete に従うインスタンスがある（何も変えない）。
/// 消せない ・ 記録できないとき。
pub fn delete(
    request: &DeleteRequest,
    store: &dyn DocumentStore,
    caller: &Caller,
    log: &mut dyn ChangeLog,
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
    let corpus = load_corpus(store)?;
    let kind = classify(&corpus.concretes, path, &declared)?;
    let current = ContentHash::of(text.as_bytes());
    let row = ChangeRow {
        kind: "delete".to_string(),
        source: "sas".to_string(),
        target_path: path.to_string(),
        request_id: request.request_id.clone(),
        before: Some(current.as_str().to_string()),
        after: None,
        intent: request.intent.clone(),
        schema: Some(declared.clone()),
        fields: Vec::new(),
    };

    // 1. 消す前提のハッシュ（更新と同じ決まり）
    let (expected, from) = match &request.if_match {
        Some(hash) => (Some(hash.clone()), "--if-match の"),
        None => (
            untracked_since(&log.writes(path).map_err(log_refused)?, current.as_str()),
            "SAS が最後に書いた事後の",
        ),
    };
    if let Some(expected) = expected.filter(|e| e != current.as_str()) {
        let writes = log.writes(path).map_err(log_refused)?;
        let between = interleaved(&writes, &expected, current.as_str());
        let mut r = refused(
            "conflict",
            path,
            format!(
                "いまの中身のハッシュは {}。{from}ハッシュ {expected} と違うので消さない。間に入った変更（interleaved）を読んでから消す{}",
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
            return Err(has_dependents(path, blocked));
        }
    }

    // 3. 対象を消し、事前の状態を記録する
    store.remove(path).map_err(|e| refused("store", path, e))?;
    let diff = Diff::between(Some(&before), &json!({}));
    let (session, operation) = record_change(
        caller,
        log,
        &ChangeRow {
            fields: field_rows(&diff),
            ..row
        },
    )?;

    Ok(Deletion {
        session,
        operation,
        kind,
        path: path.to_string(),
        concrete: declared,
        before: current,
        diff,
        replayed: false,
    })
}
