//! Concrete とインスタンスを書く手続き（業務領域「書いて検証する」の書くモジュールの手続き）。
//!
//! - UC1 作成する ・ UC2 更新する（機械）：値か JSON Patch から書く中身を作り、形と集約の書く条件を確かめてから書き、
//!   前後の中身から求めた差分と一緒に操作を記録する。書く手前で断ったものは記録せず、理由をその場で返す。
//! - SAS を通らない変更の取り込み（UC1 の変形 ・ UC2 の --adopt）：ファイルは書かず、いまの中身を形と書く条件に照らして、
//!   source = outside の操作として記録する。更新の取り込みの欄ごとの前の値は、記録のどれかの事後と同じ中身のコミットから取る。
//! - UC3 削除する（機械）：対象を消し、消す前の状態を記録する（delete.rs）。
//! - 対象の見分け ・ 形の確かめ ・ 検証は、検証のモジュール（validation-application）を使う。

mod delete;
mod patch;

pub use delete::{DeleteRequest, Deletion, delete};

use crate::ports::CommitReader;
use recording_application::ports::{
    ChangeRow, FieldRow, IdentifyCaller, OperationLog, StoredChange,
};
use recording_application::use_cases::history::{interleaved, record_change, untracked_since};
use serde_json::Value;
use validation_application::ports::{DocumentStore, SchemaValidator};
use validation_application::use_cases::validate::{
    Blocked, Corpus, Refused, TargetKind, classify, entry, form_violations, log_refused, refused,
};
use validation_domain::values::{By, RuleResult, Verdict};
use writing_domain::aggregates::{ConcreteSchema, Entry, Instance};
use writing_domain::values::{Change, Diff, FieldChange, Fingerprint};

/// 作成か更新か。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Written {
    /// 作った
    Created,
    /// 直した
    Updated,
}

impl Written {
    /// 出力に書く名前。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Updated => "updated",
        }
    }

    const fn kind(self) -> &'static str {
        match self {
            Self::Created => "create",
            Self::Updated => "update",
        }
    }
}

/// この変更で入力が変わった、前の検証の判定。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stale {
    /// rule の id
    pub rule: String,
    /// 判定した所
    pub pointer: String,
    /// 判定を記録した検証の操作の ID
    pub operation: String,
}

/// Concrete の更新で、新しい Concrete の形に従わなくなったインスタンス。
#[derive(Debug, Clone, PartialEq)]
pub struct Violated {
    /// 経路
    pub path: String,
    /// 形の外れ
    pub results: Vec<RuleResult>,
}

/// 作成か更新の結果。
#[derive(Debug, Clone, PartialEq)]
pub struct ChangeResult {
    /// 呼んだ親セッションの ID（人なら None）
    pub parent: Option<String>,
    /// 操作の ID
    pub operation: String,
    /// 作ったか直したか
    pub written: Written,
    /// 対象の種類
    pub kind: TargetKind,
    /// 経路
    pub path: String,
    /// 従う Concrete（Concrete なら Abstract の版）の $id
    pub concrete: String,
    /// 事前のハッシュ（作成では None）
    pub before: Option<Fingerprint>,
    /// 事後のハッシュ
    pub after: Fingerprint,
    /// 前後の中身から求めた差分
    pub diff: Diff,
    /// 入力が変わった前の判定
    pub stale: Vec<Stale>,
    /// 外れたインスタンス（Concrete の更新のとき）
    pub violated: Vec<Violated>,
    /// 同じ request_id の操作がすでにあり、書かずにそれを返したか
    pub replayed: bool,
    /// SAS を通らない変更を取り込んだ操作か（source = outside）
    pub outside: bool,
    /// 更新の取り込みで、欄ごとの前の値を取ったコミット。そのコミットが無ければ None（差分の欄は前の値を持たない）
    pub before_commit: Option<String>,
}

/// 作成の依頼。
#[derive(Debug, Clone, PartialEq)]
pub struct CreateRequest {
    /// 作る経路
    pub path: String,
    /// 従う Concrete の $id（Concrete を作るときは Abstract の版）
    pub concrete: String,
    /// 書く値。$schema が無ければ concrete を入れる
    pub value: Value,
    /// やり直しで同じ操作を2回残さないための ID
    pub request_id: Option<String>,
    /// 意図（なぜ作るか）。記録に残し、間に入った変更として後の書き手に返す
    pub intent: Option<String>,
}

/// 更新の依頼。
#[derive(Debug, Clone, PartialEq)]
pub struct UpdateRequest {
    /// 直す経路
    pub path: String,
    /// JSON Patch（RFC 6902）の操作の並び
    pub patch: Value,
    /// 直す前に読んだ中身のハッシュ。無ければ、SAS が最後に書いた事後のハッシュと比べる
    pub if_match: Option<String>,
    /// やり直しで同じ操作を2回残さないための ID
    pub request_id: Option<String>,
    /// 意図（なぜ直すか）。記録に残し、間に入った変更として後の書き手に返す
    pub intent: Option<String>,
}

/// 書く文書の文字列（1字下げの JSON と、末尾の改行。リポジトリの文書の書き方に合わせる）。
fn to_text(value: &Value) -> String {
    let mut out = String::new();
    pretty(value, 0, &mut out);
    out.push('\n');
    out
}

fn pretty(value: &Value, depth: usize, out: &mut String) {
    let pad = |n: usize| " ".repeat(n);
    match value {
        Value::Object(m) if !m.is_empty() => {
            out.push_str("{\n");
            for (i, (k, v)) in m.iter().enumerate() {
                out.push_str(&pad(depth + 1));
                out.push_str(&Value::String(k.clone()).to_string());
                out.push_str(": ");
                pretty(v, depth + 1, out);
                out.push_str(if i + 1 < m.len() { ",\n" } else { "\n" });
            }
            out.push_str(&pad(depth));
            out.push('}');
        }
        Value::Array(a) if !a.is_empty() => {
            out.push_str("[\n");
            for (i, v) in a.iter().enumerate() {
                out.push_str(&pad(depth + 1));
                pretty(v, depth + 1, out);
                out.push_str(if i + 1 < a.len() { ",\n" } else { "\n" });
            }
            out.push_str(&pad(depth));
            out.push(']');
        }
        other => out.push_str(&other.to_string()),
    }
}

/// 書く手前の判定に要るもの：形を照らすバリデーターと、リポジトリの Concrete。
struct Judge<'a> {
    validator: &'a dyn SchemaValidator,
    more: &'a [(String, Value)],
}

/// 人が決める欄の所と default。前と後の中身のどちらかで、x-decided-by: human が当たった所（入れ子 ・ $ref の先も）。
fn human_fields(
    judge: &Judge<'_>,
    concrete: &str,
    values: &[&Value],
) -> Result<Vec<(String, Option<Value>)>, String> {
    let mut out: Vec<(String, Option<Value>)> = Vec::new();
    for v in values {
        for (at, a) in judge.validator.annotations(concrete, v, judge.more)? {
            if a.get("x-decided-by").and_then(Value::as_str) == Some("human")
                && !out.iter().any(|(p, _)| *p == at)
            {
                out.push((at, a.get("default").cloned()));
            }
        }
    }
    Ok(out)
}

/// 集約の書く条件。
fn check_write(
    judge: &Judge<'_>,
    kind: TargetKind,
    before: Option<&Value>,
    after: &Value,
    concrete: Option<&ConcreteSchema>,
    entry: Entry,
) -> Result<(), (&'static str, String)> {
    let r = match kind {
        TargetKind::Concrete => match ConcreteSchema::read(after) {
            Some(c) => c.check_write(before.and_then(ConcreteSchema::read).as_ref()),
            None => return Ok(()),
        },
        TargetKind::Instance => {
            match (Instance::read(after), concrete.and_then(ConcreteSchema::id)) {
                (Some(i), Some(id)) => {
                    let values: Vec<&Value> =
                        before.into_iter().chain(std::iter::once(after)).collect();
                    let human = human_fields(judge, id, &values).map_err(|e| ("validator", e))?;
                    i.check_write(before.and_then(Instance::read).as_ref(), &human, entry)
                }
                _ => return Ok(()),
            }
        }
    };
    r.map_err(|v| (v.code(), v.message().to_string()))
}

fn field_rows(diff: &Diff) -> Vec<FieldRow> {
    diff.changes()
        .iter()
        .map(|c| FieldRow {
            pointer: c.pointer().to_string(),
            change: c.change().as_str().to_string(),
            before: c.before().map(Value::to_string),
            after: c.after().map(Value::to_string),
            before_unavailable: c.before_unavailable(),
        })
        .collect()
}

/// 記録にある操作の対象の種類と従う型。記録には持たないので、いまのファイルの $schema か、
/// 記録した欄（作成と削除は根の欄をすべて持つ）の $schema から見分ける。
fn target_of(store: &dyn DocumentStore, path: &str, fields: &[FieldRow]) -> (TargetKind, String) {
    let from_file = store
        .read(path)
        .ok()
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        .and_then(|v| {
            v.get("$schema")
                .and_then(Value::as_str)
                .map(ToString::to_string)
        });
    let declared = from_file
        .or_else(|| {
            fields
                .iter()
                .filter(|f| f.pointer == "/$schema")
                .find_map(|f| f.after.as_deref().or(f.before.as_deref()))
                .and_then(|t| serde_json::from_str::<String>(t).ok())
        })
        .unwrap_or_default();
    let kind = if ConcreteSchema::declares_abstract(&serde_json::json!({ "$schema": declared })) {
        TargetKind::Concrete
    } else {
        TargetKind::Instance
    };
    (kind, declared)
}

/// 記録にある同じ依頼の操作を、結果に戻す。source は依頼の種類（sas ・ outside）。
fn replay(
    stored: StoredChange,
    written: Written,
    source: &str,
    path: &str,
    store: &dyn DocumentStore,
) -> Result<ChangeResult, Refused> {
    let row = stored.row;
    if row.kind != written.kind() || row.source != source || row.target_path != path {
        return Err(refused(
            "request_reused",
            path,
            format!(
                "request_id は {} の {} に使った。別の依頼には別の request_id を使う",
                row.target_path, row.kind
            ),
        ));
    }
    let (kind, concrete) = target_of(store, path, &row.fields);
    let parse = |s: &Option<String>| s.as_deref().and_then(|t| serde_json::from_str(t).ok());
    let changes = row
        .fields
        .iter()
        .filter_map(|f| {
            Some(FieldChange::new(
                &f.pointer,
                Change::parse(&f.change)?,
                parse(&f.before),
                parse(&f.after),
            ))
            .map(|c| match (f.before_unavailable, c.after()) {
                (true, Some(a)) => FieldChange::without_before(&f.pointer, a.clone()),
                _ => c,
            })
        })
        .collect();
    let after = row
        .after
        .as_deref()
        .and_then(Fingerprint::parse)
        .ok_or_else(|| {
            log_refused(format!(
                "操作 {} の事後のハッシュを読めない",
                stored.operation
            ))
        })?;
    Ok(ChangeResult {
        parent: stored.parent,
        operation: stored.operation,
        written,
        kind,
        path: row.target_path,
        concrete,
        before: row.before.as_deref().and_then(Fingerprint::parse),
        after,
        diff: Diff::from_changes(changes),
        stale: Vec::new(),
        violated: Vec::new(),
        replayed: true,
        outside: row.source == "outside",
        before_commit: None,
    })
}

fn violations_message(broken: &[(String, String)]) -> String {
    broken
        .iter()
        .map(|(at, why)| format!("{}：{why}", if at.is_empty() { "/" } else { at }))
        .collect::<Vec<_>>()
        .join(" ／ ")
}

/// UC1 作成する（機械）：値を型に照らし、集約の書く条件を確かめてから書き、作成の操作を記録する。
///
/// # Errors
/// 依頼を読めない ・ 従う型が決まらない ・ すでに在る（何も記録しない）。
/// 形に外れる ・ 書く条件に外れる（書かずに記録する）。書けない ・ 記録できないとき。
pub fn create(
    request: &CreateRequest,
    store: &dyn DocumentStore,
    validator: &dyn SchemaValidator,
    who: &dyn IdentifyCaller,
    log: &mut dyn OperationLog,
) -> Result<ChangeResult, Refused> {
    let path = request.path.as_str();
    if let Some(id) = &request.request_id
        && let Some(stored) = log.find_request(id).map_err(log_refused)?
    {
        return replay(stored, Written::Created, "sas", path, store);
    }
    let mut value = request.value.clone();
    let Some(object) = value.as_object_mut() else {
        return Err(refused(
            "invalid_input",
            path,
            "--value は JSON のオブジェクトで渡す".to_string(),
        ));
    };
    match object.get("$schema").and_then(Value::as_str) {
        None => {
            // $schema を先頭に置く（ほかの欄の並びは渡された順のまま）
            let mut first = serde_json::Map::new();
            first.insert(
                "$schema".to_string(),
                Value::String(request.concrete.clone()),
            );
            first.append(object);
            *object = first;
        }
        Some(s) if s == request.concrete => {}
        Some(s) => {
            return Err(refused(
                "invalid_input",
                path,
                format!(
                    "値の $schema（{s}）が --concrete（{}）と違う",
                    request.concrete
                ),
            ));
        }
    }
    if store.read(path).is_ok() {
        return Err(refused(
            "exists",
            path,
            format!("{path} はすでに在る。直すときは update を使う"),
        ));
    }
    let corpus = Corpus::load(store)?;
    let kind = classify(&corpus.concretes, path, &request.concrete)?;
    let row = ChangeRow {
        kind: Written::Created.kind().to_string(),
        source: "sas".to_string(),
        target_path: path.to_string(),
        request_id: request.request_id.clone(),
        before: None,
        after: None,
        intent: request.intent.clone(),
        fields: Vec::new(),
    };
    let broken = form_violations(validator, kind, &request.concrete, &value, &corpus.more)
        .map_err(|e| refused("validator", path, e))?;
    if !broken.is_empty() {
        return Err(refused(
            "invalid_value",
            path,
            format!("型に従わない：{}", violations_message(&broken)),
        ));
    }
    if let Err((code, message)) = check_write(
        &Judge {
            validator,
            more: &corpus.more,
        },
        kind,
        None,
        &value,
        corpus.concretes.get(&request.concrete),
        entry(who),
    ) {
        return Err(refused(code, path, message));
    }
    let text = to_text(&value);
    store
        .write(path, &text)
        .map_err(|e| refused("store", path, e))?;
    let after = Fingerprint::of(text.as_bytes());
    let diff = Diff::between(None, &value);
    let recorded = record_change(
        who,
        log,
        &ChangeRow {
            after: Some(after.as_str().to_string()),
            fields: field_rows(&diff),
            ..row
        },
    )
    .map_err(log_refused)?;
    Ok(ChangeResult {
        parent: recorded.parent,
        operation: recorded.operations.into_iter().next().unwrap_or_default(),
        written: Written::Created,
        kind,
        path: path.to_string(),
        concrete: request.concrete.clone(),
        before: None,
        after,
        diff,
        stale: Vec::new(),
        violated: Vec::new(),
        replayed: false,
        outside: false,
        before_commit: None,
    })
}

/// 2つの JSON Pointer の一方が他方を含むか（根 "" はすべてを含む）。
fn overlaps(a: &str, b: &str) -> bool {
    let within = |inner: &str, outer: &str| {
        inner == outer
            || inner
                .strip_prefix(outer)
                .is_some_and(|r| r.starts_with('/'))
    };
    within(a, b) || within(b, a)
}

/// UC2 更新する（機械）：いまの中身のハッシュを確かめ、JSON Patch を当て、型と集約の書く条件を確かめてから書き、
/// 前後の中身から求めた差分と一緒に更新の操作を記録する。前の判定のうち入力が変わったものと、
/// Concrete の更新で外れたインスタンスを返す。
///
/// # Errors
/// 依頼を読めない ・ 対象が無い ・ 従う型が決まらない（何も記録しない）。
/// ハッシュが違う ・ 差分を当てられない ・ 形や書く条件に外れる（書かずに記録する）。書けない ・ 記録できないとき。
pub fn update(
    request: &UpdateRequest,
    store: &dyn DocumentStore,
    validator: &dyn SchemaValidator,
    who: &dyn IdentifyCaller,
    log: &mut dyn OperationLog,
) -> Result<ChangeResult, Refused> {
    let path = request.path.as_str();
    if let Some(id) = &request.request_id
        && let Some(stored) = log.find_request(id).map_err(log_refused)?
    {
        return replay(stored, Written::Updated, "sas", path, store);
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
        kind: Written::Updated.kind().to_string(),
        source: "sas".to_string(),
        target_path: path.to_string(),
        request_id: request.request_id.clone(),
        before: Some(current.as_str().to_string()),
        after: None,
        intent: request.intent.clone(),
        fields: Vec::new(),
    };
    // 比べるハッシュ：呼ぶ側が読んだハッシュ（--if-match）か、いまの中身が記録のどの事後とも違うときの最後の事後のハッシュ。
    // SAS が一度も書いていない ・ いまの中身が記録のどれかの事後なら比べるハッシュが無く、そのまま書く
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
                "いまの中身のハッシュは {}。{from}ハッシュ {expected} と違うので直さない。間に入った変更（interleaved）を読んでから直す{}",
                current.as_str(),
                if between.untracked {
                    "。SAS を通らない変更がある"
                } else {
                    ""
                }
            ),
        );
        r.hint = Some(Box::new(if between.untracked {
            adopt_hint(path)
        } else {
            reapply_hint(path, current.as_str())
        }));
        r.interleaved = Some(Box::new(between));
        return Err(r);
    }
    let after = match patch::apply(&before, &request.patch) {
        Ok(v) => v,
        Err(e) => {
            return Err(refused(
                "patch_failed",
                path,
                format!("JSON Patch を当てられない（どれも当てていない）：{e}"),
            ));
        }
    };
    // Concrete を直すときは、新しい中身を同じ $id の Concrete として照らす
    let own_id = ConcreteSchema::read(&before).and_then(|c| c.id().map(ToString::to_string));
    let mut more: Vec<(String, Value)> = corpus
        .more
        .iter()
        .filter(|(id, _)| Some(id) != own_id.as_ref())
        .cloned()
        .collect();
    if let Some(id) = &own_id {
        more.push((id.clone(), after.clone()));
    }
    let broken = form_violations(validator, kind, &declared, &after, &more)
        .map_err(|e| refused("validator", path, e))?;
    if !broken.is_empty() {
        return Err(refused(
            "invalid_value",
            path,
            format!("当てた結果が型に従わない：{}", violations_message(&broken)),
        ));
    }
    if let Err((code, message)) = check_write(
        &Judge {
            validator,
            more: &more,
        },
        kind,
        Some(&before),
        &after,
        corpus.concretes.get(&declared),
        entry(who),
    ) {
        return Err(refused(code, path, message));
    }
    let new_text = to_text(&after);
    store
        .write(path, &new_text)
        .map_err(|e| refused("store", path, e))?;
    let after_fp = Fingerprint::of(new_text.as_bytes());
    let diff = Diff::between(Some(&before), &after);
    let recorded = record_change(
        who,
        log,
        &ChangeRow {
            after: Some(after_fp.as_str().to_string()),
            fields: field_rows(&diff),
            ..row
        },
    )
    .map_err(log_refused)?;

    let stale = stale_judgments(log, path, &diff)?;
    let violated = match &own_id {
        Some(id) => violated_instances(validator, &corpus, path, id, &more)?,
        None => Vec::new(),
    };
    Ok(ChangeResult {
        parent: recorded.parent,
        operation: recorded.operations.into_iter().next().unwrap_or_default(),
        written: Written::Updated,
        kind,
        path: path.to_string(),
        concrete: declared,
        before: Some(current),
        after: after_fp,
        diff,
        stale,
        violated,
        replayed: false,
        outside: false,
        before_commit: None,
    })
}

/// 前の判定のうち、読む所が変えた欄と重なるもの（形の判定は書く前に照らし直したので除く）。
fn stale_judgments(log: &dyn OperationLog, path: &str, diff: &Diff) -> Result<Vec<Stale>, Refused> {
    Ok(log
        .last_judgments(path)
        .map_err(log_refused)?
        .into_iter()
        .filter(|j| j.rule.rule != "json-schema")
        .filter(|j| {
            diff.changes()
                .iter()
                .any(|c| overlaps(&j.rule.pointer, c.pointer()))
        })
        .map(|j| Stale {
            rule: j.rule.rule,
            pointer: j.rule.pointer,
            operation: j.operation,
        })
        .collect())
}

/// Concrete を直したら、それに従うインスタンスを新しい形に照らす。
fn violated_instances(
    validator: &dyn SchemaValidator,
    corpus: &Corpus,
    path: &str,
    id: &str,
    more: &[(String, Value)],
) -> Result<Vec<Violated>, Refused> {
    let mut violated = Vec::new();
    for d in &corpus.docs {
        if d.path == path || d.value.get("$schema").and_then(Value::as_str) != Some(id) {
            continue;
        }
        let broken = validator
            .check(id, &d.value, more)
            .map_err(|e| refused("validator", &d.path, e))?;
        if !broken.is_empty() {
            violated.push(Violated {
                path: d.path.clone(),
                results: broken
                    .iter()
                    .map(|(at, why)| {
                        RuleResult::new("json-schema", at, By::Machine, Verdict::Violated, why)
                    })
                    .collect(),
            });
        }
    }
    Ok(violated)
}

/// SAS を通らない変更を取り込むコマンドの案内。
fn adopt_hint(path: &str) -> String {
    format!("意図を付けて取り込む：sas update {path} --adopt --intent \"…\"")
}

/// SAS を通した変更が間に入ったときに、のせ直して打ち直すコマンドの案内。
fn reapply_hint(path: &str, current: &str) -> String {
    format!(
        "最新を読み直し、間に入った変更（interleaved）の上に変更をのせ直して打ち直す：sas update {path} --patch <JSON Patch> --if-match {current} --intent \"…\""
    )
}

/// 取り込みを、作成として行うか更新として行うか。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdoptAs {
    /// SAS の記録に無い既存のファイルを取り込む（UC1 の変形「既存のファイルを取り込む」）
    Create,
    /// SAS が書いたあとに外で直されたファイルを取り込む（UC2 の --adopt）
    Update,
}

/// SAS を通らない変更の取り込みの依頼。ファイルは書かない。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdoptRequest {
    /// 取り込む経路
    pub path: String,
    /// 作成か更新か
    pub as_kind: AdoptAs,
    /// 従う Concrete の $id。渡されたら $schema と同じかを確かめる（無ければ $schema から決める）
    pub concrete: Option<String>,
    /// 意図（なぜ外で直したか）。更新の取り込みでは要る
    pub intent: Option<String>,
    /// やり直しで同じ操作を2回残さないための ID
    pub request_id: Option<String>,
}

/// 前の中身を探すコミットの数の上限。
pub const COMMITS_TO_SEARCH: usize = 50;

/// 中身のハッシュが記録のどれかの事後と同じコミットを、新しい順に探す。見つかればコミットと中身と、そのハッシュ。
/// DB はワークツリーで1つなので、最後の事後だけでなく、この経路の記録のどの事後でもよい
/// （このワークツリーの履歴にあるのは、ほかのワークツリーが書く前の事後かもしれない）。
fn find_commit(
    committed: &dyn CommitReader,
    path: &str,
    recorded: &[String],
) -> Result<Option<(String, Value, String)>, Refused> {
    let store_err = |e: String| refused("versions", path, e);
    for rev in committed
        .commits(path, COMMITS_TO_SEARCH)
        .map_err(store_err)?
    {
        let Some(bytes) = committed.read(&rev, path).map_err(store_err)? else {
            continue;
        };
        let fp = Fingerprint::of(&bytes);
        if recorded.iter().any(|r| r == fp.as_str())
            && let Ok(value) = serde_json::from_slice::<Value>(&bytes)
        {
            return Ok(Some((rev, value, fp.as_str().to_string())));
        }
    }
    Ok(None)
}

/// SAS を通らない変更を取り込む（UC1 の変形 ・ UC2 の --adopt）。ファイルは書かず、いまの中身を
/// 形と集約の書く条件に照らしてから、source = outside の作成か更新として記録する。
/// 更新の取り込みの差分は、記録のどれかの事後と同じ中身のコミット（ふつうは HEAD）との比較で、
/// そのコミットが無ければ前の値の無い差分（before_unavailable）にし、事前は記録の最後の事後にする。
///
/// # Errors
/// 依頼を読めない ・ 対象が無い ・ 従う型が決まらない ・ 作成の取り込みで記録がすでにある ・ 更新の取り込みで記録が無いか
/// 外の変更が無い ・ 形や書く条件に外れる（何も記録しない）。コミットや記録を読めない ・ 記録できないとき。
pub fn adopt(
    request: &AdoptRequest,
    store: &dyn DocumentStore,
    validator: &dyn SchemaValidator,
    committed: &dyn CommitReader,
    who: &dyn IdentifyCaller,
    log: &mut dyn OperationLog,
) -> Result<ChangeResult, Refused> {
    let path = request.path.as_str();
    let written = match request.as_kind {
        AdoptAs::Create => Written::Created,
        AdoptAs::Update => Written::Updated,
    };
    if let Some(id) = &request.request_id
        && let Some(stored) = log.find_request(id).map_err(log_refused)?
    {
        return replay(stored, written, "outside", path, store);
    }
    let text = store
        .read(path)
        .map_err(|e| refused("not_found", path, e))?;
    let current: Value = serde_json::from_str(&text)
        .map_err(|e| refused("not_json", path, format!("JSON として読めない：{e}")))?;
    let declared = current
        .get("$schema")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    if let Some(c) = &request.concrete
        && *c != declared
    {
        return Err(refused(
            "invalid_input",
            path,
            format!("ファイルの $schema（{declared}）が --concrete（{c}）と違う"),
        ));
    }
    let corpus = Corpus::load(store)?;
    let kind = classify(&corpus.concretes, path, &declared)?;
    let fingerprint = Fingerprint::of(text.as_bytes());
    let writes = log.writes(path).map_err(log_refused)?;
    let recorded: Vec<String> = writes.iter().filter_map(|w| w.row.after.clone()).collect();

    // 前の状態：作成の取り込みは記録が無いこと、更新の取り込みはいまの中身が記録のどの事後とも違うこと
    let last = match (request.as_kind, writes.last()) {
        (AdoptAs::Create, Some(w)) => {
            return Err(refused(
                "recorded",
                path,
                format!(
                    "{path} は操作 {} で記録がある。外で直した変更は sas update {path} --adopt --intent \"…\" で取り込む",
                    w.operation
                ),
            ));
        }
        (AdoptAs::Create, None) => None,
        (AdoptAs::Update, None) => {
            return Err(refused(
                "not_recorded",
                path,
                format!(
                    "{path} は SAS の記録に無い。既存のファイルは sas create {path} --adopt で取り込む"
                ),
            ));
        }
        (AdoptAs::Update, Some(w)) => {
            if let Some(same) = writes
                .iter()
                .rev()
                .find(|x| x.row.after.as_deref() == Some(fingerprint.as_str()))
            {
                return Err(refused(
                    "nothing_to_adopt",
                    path,
                    format!(
                        "いまの中身は操作 {} の事後と同じで、SAS を通らない変更が無い",
                        same.operation
                    ),
                ));
            }
            if request.intent.is_none() {
                return Err(refused(
                    "invalid_input",
                    path,
                    "更新の取り込みには、外で直した意図（--intent）が要る".to_string(),
                ));
            }
            Some(w.row.after.clone().unwrap_or_default())
        }
    };
    let row = ChangeRow {
        kind: written.kind().to_string(),
        source: "outside".to_string(),
        target_path: path.to_string(),
        request_id: request.request_id.clone(),
        before: None,
        after: None,
        intent: request.intent.clone(),
        fields: Vec::new(),
    };
    let found = match &last {
        Some(_) => find_commit(committed, path, &recorded)?,
        None => None,
    };
    // 事前：前の中身のコミットが見つかればそのハッシュ（記録のどれかの事後）、無ければ記録の最後の事後
    let expected: Option<String> = match (&found, &last) {
        (Some((_, _, fp)), _) => Some(fp.clone()),
        (None, l) => l.clone(),
    };
    let broken = form_violations(validator, kind, &declared, &current, &corpus.more)
        .map_err(|e| refused("validator", path, e))?;
    if !broken.is_empty() {
        return Err(refused(
            "invalid_value",
            path,
            format!(
                "いまの中身が型に従わないので取り込まない：{}",
                violations_message(&broken)
            ),
        ));
    }
    // 書く条件：前の中身のコミットがあればそれと比べる。無ければ、前の値を知らない作成と同じに見る
    // （ツールからの取り込みで人の欄が default 以外なら、外で人の欄が変わったかを確かめられないので断る）
    let before_value = found.as_ref().map(|(_, v, _)| v);
    if let Err((code, message)) = check_write(
        &Judge {
            validator,
            more: &corpus.more,
        },
        kind,
        before_value,
        &current,
        corpus.concretes.get(&declared),
        entry(who),
    ) {
        let unknown = expected.is_some() && found.is_none();
        let mut r = refused(
            code,
            path,
            if unknown && code == "human_field" {
                format!(
                    "{message}。前の中身のコミットが無く、外で人の欄が変わったかを確かめられない"
                )
            } else {
                message
            },
        );
        if code == "human_field" {
            r.hint = Some(Box::new(format!(
                "人が --session を付けずに取り込む：sas {} {path} --adopt{}",
                written.kind(),
                if expected.is_some() {
                    " --intent \"…\""
                } else {
                    ""
                }
            )));
        }
        return Err(r);
    }
    let diff = match (&expected, &found) {
        (None, _) => Diff::between(None, &current),
        (Some(_), Some((_, before, _))) => Diff::between(Some(before), &current),
        (Some(_), None) => Diff::without_before(&current),
    };
    let recorded = record_change(
        who,
        log,
        &ChangeRow {
            before: expected.clone(),
            after: Some(fingerprint.as_str().to_string()),
            fields: field_rows(&diff),
            ..row
        },
    )
    .map_err(log_refused)?;
    let stale = stale_judgments(log, path, &diff)?;
    let own_id = ConcreteSchema::read(&current).and_then(|c| c.id().map(ToString::to_string));
    let violated = match (&own_id, request.as_kind) {
        (Some(id), AdoptAs::Update) => {
            let mut more: Vec<(String, Value)> = corpus
                .more
                .iter()
                .filter(|(i, _)| i != id)
                .cloned()
                .collect();
            more.push((id.clone(), current.clone()));
            violated_instances(validator, &corpus, path, id, &more)?
        }
        _ => Vec::new(),
    };
    Ok(ChangeResult {
        parent: recorded.parent,
        operation: recorded.operations.into_iter().next().unwrap_or_default(),
        written,
        kind,
        path: path.to_string(),
        concrete: declared,
        before: expected.as_deref().and_then(Fingerprint::parse),
        after: fingerprint,
        diff,
        stale,
        violated,
        replayed: false,
        outside: true,
        before_commit: found.map(|(rev, _, _)| rev),
    })
}
