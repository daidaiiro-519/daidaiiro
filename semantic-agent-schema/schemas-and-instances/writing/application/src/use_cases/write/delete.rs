//! UC3 削除する（機械）：対象を消し、消す前の状態を削除の操作として記録する。
//!
//! ほかのファイルが対象を指していても探さない（参照はただの値で、Semantic Agent Schema は参照を見分けない）。
//! Concrete は、渡された範囲（--range）と記録に従うインスタンスが残っていれば消さずに止め、見た文書を結果に返す。

use super::{
    Blocked, Checked, Refused, carried, classify, field_rows, interleaved, log_refused, range_docs,
    range_for, record_change, refused,
};
use crate::ports::{ChangeLog, DocumentStore};
use serde_json::{Value, json};
use writing_domain::aggregates::ConcreteSchema;
use writing_domain::values::{Caller, ChangeRow, ContentHash, Diff, SchemaPlaces, TargetKind};

/// 削除の依頼。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteRequest {
    /// 消す経路
    pub path: String,
    /// 消す前に読んだ中身のハッシュ（--if-match。必ず渡す）。いまの中身のハッシュと違えば消さない
    pub if_match: String,
    /// 理由（なぜ消すか。必ず渡す。記録の事前の状態と一緒に残す）
    pub reason: String,
    /// スキーマの置き場所（外の変更があるときの取り込みの案内に引き継ぐ）
    pub schemas: SchemaPlaces,
    /// Concrete を消すときに、従うインスタンスが無いことを確かめる範囲（--range の経路）
    pub range: Option<Vec<String>>,
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
    /// 従う schema の $id（インスタンスなら Concrete、Concrete なら Abstract の版）
    pub schema: String,
    /// 消す前のハッシュ
    pub before: ContentHash,
    /// 消す前の状態（根の欄ごとの remove）
    pub diff: Diff,
    /// Concrete を消すときに、従うインスタンスかを見た文書（渡された範囲と記録から）
    pub checked: Vec<Checked>,
}

fn delete_hint(path: &str, hash: &str, schemas: &SchemaPlaces, caller: &Caller) -> String {
    format!(
        "消す：{}",
        carried(
            &["delete", path, "--if-match", hash, "--reason", "<理由>"],
            Some(schemas),
            None,
            caller
        )
    )
}

/// Concrete に従うインスタンスがあるので止めた削除を、従うインスタンスと一緒に返す（記録しない）。
fn has_dependents(
    path: &str,
    hash: &str,
    blocked: Vec<Blocked>,
    schemas: &SchemaPlaces,
    range: &[String],
    caller: &Caller,
) -> Refused {
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
        "blocked のそれぞれの hint を片付けてから、消し直す：{}",
        carried(
            &["delete", path, "--if-match", hash, "--reason", "<理由>"],
            Some(schemas),
            Some(range),
            caller
        )
    )));
    r.blocked = Some(Box::new(blocked));
    r
}

/// UC3 削除する（機械）：前提のハッシュと、Concrete なら渡された範囲と記録に従うインスタンスが無いことを確かめてから、
/// 対象を消して記録する。
///
/// # Errors
/// 依頼を読めない ・ 対象が無い ・ 従う型が決まらない（何も記録しない）。
/// Concrete で範囲が無い（usage）・ ハッシュが違う ・ Concrete に従うインスタンスがある（何も変えない）。
/// 消せない ・ 記録できないとき。
pub fn delete(
    request: &DeleteRequest,
    store: &dyn DocumentStore,
    caller: &Caller,
    log: &mut dyn ChangeLog,
) -> Result<Deletion, Refused> {
    let path = request.path.as_str();
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
    let kind = classify(path, &declared)?;
    let range = range_for(kind, path, request.range.as_deref(), "消す")?;
    let current = ContentHash::of(text.as_bytes());
    let row = ChangeRow {
        kind: "delete".to_string(),
        source: super::SOURCE.to_string(),
        target_path: path.to_string(),
        before: Some(current.as_str().to_string()),
        after: None,
        reason: Some(request.reason.clone()),
        schema: Some(declared.clone()),
        fields: Vec::new(),
    };

    // 1. 消す前提のハッシュ（呼ぶ側が読んだハッシュ。更新と同じ決まり）
    let expected = request.if_match.clone();
    if expected != current.as_str() {
        let writes = log.writes(path).map_err(log_refused)?;
        let between = interleaved(&writes, &expected, current.as_str());
        let mut r = refused(
            "conflict",
            path,
            format!(
                "いまの中身のハッシュは {}。--if-match のハッシュ {expected} と違うので消さない。間に入った変更（interleaved）を読んでから消す{}",
                current.as_str(),
                if between.unrecorded {
                    "。Semantic Agent Schema を通らない変更がある"
                } else {
                    ""
                }
            ),
        );
        let range = request.range.as_deref();
        r.hint = Some(Box::new(if between.unrecorded {
            format!(
                "外の変更を取り込んでから消す：{} のあとで {}",
                carried(
                    &["update", path, "--adopt", "--reason", "<理由>"],
                    Some(&request.schemas),
                    range,
                    caller
                ),
                carried(
                    &[
                        "delete",
                        path,
                        "--if-match",
                        current.as_str(),
                        "--reason",
                        "<理由>"
                    ],
                    Some(&request.schemas),
                    range,
                    caller
                )
            )
        } else {
            format!(
                "間に入った変更を読み、それでも消すなら打ち直す：{}",
                carried(
                    &[
                        "delete",
                        path,
                        "--if-match",
                        current.as_str(),
                        "--reason",
                        "<理由>"
                    ],
                    Some(&request.schemas),
                    range,
                    caller
                )
            )
        }));
        r.interleaved = Some(Box::new(between));
        return Err(r);
    }

    // 2. Concrete に従うインスタンス（渡された範囲と記録の文書）
    let mut checked: Vec<Checked> = Vec::new();
    if kind == TargetKind::Concrete
        && let Some(own) =
            ConcreteSchema::read(&before).and_then(|c| c.id().map(ToString::to_string))
    {
        checked = range_docs(store, log, path, &own, range)?
            .into_iter()
            .map(|(c, _)| c)
            .collect();
        let blocked: Vec<Blocked> = checked
            .iter()
            .filter(|c| c.follows)
            .map(|c| Blocked {
                path: c.path.clone(),
                code: "follows",
                message: format!("{own} に従うインスタンス"),
                hint: delete_hint(&c.path, c.hash.as_str(), &request.schemas, caller),
            })
            .collect();
        if !blocked.is_empty() {
            return Err(has_dependents(
                path,
                current.as_str(),
                blocked,
                &request.schemas,
                range,
                caller,
            ));
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
        schema: declared,
        before: current,
        diff,
        checked,
    })
}
