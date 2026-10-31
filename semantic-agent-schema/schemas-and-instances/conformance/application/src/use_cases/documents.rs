//! Concrete とインスタンスの文書の手続き。
//!
//! - UC4 検証する（機械の rule）：対象を読み、Concrete かインスタンスかを $schema で見分け、形（JSON Schema）・
//!   当たる rule ・ 参照の先を判定し、対象ごとに検証の操作を1件記録して、結果と記録の ID を返す。検証は対象を変えない。
//! - UC1 作成する ・ UC2 更新する（機械）：値か JSON Patch から書く中身を作り、形と集約の書く条件を確かめてから書き、
//!   前後の中身から求めた差分と一緒に操作を記録する。書く手前で断ったものは記録せず、理由をその場で返す。
//! - SAS を通らない変更の取り込み（UC1 の変形 ・ UC2 の --adopt）：ファイルは書かず、いまの中身を形と書く条件に照らして、
//!   source = outside の操作として記録する。更新の取り込みの欄ごとの前の値は、記録のどれかの事後と同じ中身のコミットから取る。
//! - UC4 の範囲「入力が変わった rule だけ」と、SAS を通らない変更があった対象の知らせ。
//! - UC3 削除する（機械）：対象と、対象を指している参照元の記載を、同じ呼び出しの操作として消す（delete.rs）。

mod delete;
mod patch;

pub use delete::{DeleteRequest, Deletion, Rewritten, delete};

use crate::ports::{CommitReader, DocumentStore, SchemaChecker};
use schemas_and_instances_conformance_domain::aggregates::{
    ABSTRACT_VERSIONS, ConcreteSchema, Entry, Instance, extension_names,
};
use schemas_and_instances_conformance_domain::services::{
    check_relationships, check_targets, judge_others, rules_for_concrete, rules_for_instance,
};
use schemas_and_instances_conformance_domain::values::{
    By, Change, Counts, Diff, FieldChange, Fingerprint, RuleResult, Verdict,
};
use schemas_and_instances_operation_recording_application::ports::{
    Caller, ChangeRow, FieldRow, IdentifyCaller, Judgment, OperationLog, RuleRow, StoredChange,
    ValidateRow,
};
use schemas_and_instances_operation_recording_application::use_cases::history::{
    Interleaved, interleaved, record_change, record_validations, untracked_since,
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// 方言（Concrete の入れ子の x- の形を決める）。Concrete は Abstract の本体と、この方言の両方に照らす。
pub const DIALECT: &str = "urn:semantic-agent-schema:base:dialect:v1";

/// 何を検証するか。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scope {
    /// 経路を指定した対象
    Paths(Vec<String>),
    /// リポジトリの Concrete とインスタンスのすべて
    All,
}

/// 対象の種類。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetKind {
    /// Abstract に従う schema
    Concrete,
    /// Concrete に従う JSON
    Instance,
}

impl TargetKind {
    /// 出力と記録に書く名前。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Concrete => "concrete",
            Self::Instance => "instance",
        }
    }
}

/// 1つの対象の検証の結果。
#[derive(Debug, Clone, PartialEq)]
pub struct TargetResult {
    /// 経路
    pub path: String,
    /// 種類
    pub kind: TargetKind,
    /// 従う schema の $id
    pub schema: String,
    /// 検証したときの中身の指紋
    pub fingerprint: Fingerprint,
    /// rule ごとの結果
    pub results: Vec<RuleResult>,
    /// 5つの値ごとの件数
    pub counts: Counts,
    /// 記録した操作の ID
    pub operation: String,
    /// 入力が変わった rule だけを検証したとき、読んだ所が変わっていないので判定し直さなかった rule の id
    pub unchanged_rules: Vec<String>,
}

/// 記録の最後の事後と中身が違う対象（SAS を通らない変更があった）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Untracked {
    /// 経路
    pub path: String,
    /// 対象を最後に書いた操作の ID
    pub last_operation: String,
    /// その操作の事後の指紋
    pub expected: String,
    /// いまの中身の指紋
    pub actual: String,
}

/// 検証の結果と、記録の ID。
#[derive(Debug, Clone, PartialEq)]
pub struct Validation {
    /// 呼んだ親のセッションの ID（人なら None）
    pub parent: Option<String>,
    /// 対象ごとの結果（経路の順）
    pub targets: Vec<TargetResult>,
    /// 全体の件数
    pub counts: Counts,
    /// 入力が変わった rule だけを検証したか
    pub stale_only: bool,
    /// 入力が変わった rule だけを検証したとき、変わった rule が無く記録しなかった対象
    pub unchanged_targets: Vec<String>,
    /// SAS を通らない変更があった対象（いまの中身を検証した）
    pub untracked: Vec<Untracked>,
}

/// 操作を始められなかった ・ 書かなかった理由。断った操作は記録しない（理由 ・ 案内 ・ 間に入った変更をその場で返す）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Refused {
    /// 決まった名前（usage ・ schema_not_listed ・ not_found ・ not_json ・ no_schema ・ unknown_schema ・ exists ・
    /// invalid_input ・ request_reused ・ recorded ・ not_recorded ・ nothing_to_adopt ・ store ・ checker ・ commits ・ log、
    /// 書く手前の判定で断ったもの：invalid_value ・ has_dependents ・ references_blocked ・ conflict ・ patch_failed ・
    /// id_changed ・ human_field ・ derived_mismatch ・ published_changed、書いた途中で止まったもの：references_left）
    pub code: &'static str,
    /// 対象の経路（あれば）
    pub path: Option<Box<str>>,
    /// 理由
    pub message: String,
    /// 書いた途中で止まったとき（references_left）の、記録した削除の ID
    pub operation: Option<String>,
    /// 指紋が違って断ったとき（conflict）の、間に入った変更
    pub interleaved: Option<Box<Interleaved>>,
    /// 次にすることの案内（SAS を通らない変更を取り込むコマンドなど）
    pub hint: Option<Box<String>>,
    /// 削除を止めた理由になったもの（従うインスタンス ・ 指している Concrete ・ 記載を消せない参照元）と、残った参照元
    pub blocked: Option<Box<Vec<Blocked>>>,
}

/// 削除を止めた理由になった文書1つと、それを片付けるコマンドの案内。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Blocked {
    /// 経路
    pub path: String,
    /// 決まった名前（follows ・ points_to ・ conflict ・ invalid_value ・ human_field ・ id_changed ・ derived_mismatch ・ left）
    pub code: &'static str,
    /// 理由
    pub message: String,
    /// 次に打つコマンド
    pub hint: String,
}

fn refused(code: &'static str, path: &str, message: String) -> Refused {
    Refused {
        code,
        path: Some(path.into()),
        message,
        ..Refused::default()
    }
}

#[derive(Clone)]
struct Doc {
    path: String,
    text: String,
    value: Value,
}

/// リポジトリの文書と、そこから読んだ Concrete ・ インスタンスの一覧。
struct Corpus {
    docs: Vec<Doc>,
    /// $id → Concrete
    concretes: BTreeMap<String, ConcreteSchema>,
    /// 検査器に渡す Concrete（$id と文書）
    more: Vec<(String, Value)>,
    /// （従う Concrete の $id, インスタンスの id）
    instances: BTreeSet<(String, String)>,
}

impl Corpus {
    /// リポジトリの文書を読む（Concrete の一覧と、参照の先のインスタンスに使う）。
    fn load(store: &dyn DocumentStore) -> Result<Self, Refused> {
        let mut docs: Vec<Doc> = Vec::new();
        for path in store.list().map_err(|e| Refused {
            code: "store",
            message: e,
            ..Refused::default()
        })? {
            if let Ok(text) = store.read(&path)
                && let Ok(value) = serde_json::from_str::<Value>(&text)
            {
                docs.push(Doc { path, text, value });
            }
        }
        let concretes: BTreeMap<String, ConcreteSchema> = docs
            .iter()
            .filter_map(|d| ConcreteSchema::read(&d.value))
            .filter_map(|c| Some((c.id()?.to_string(), c)))
            .collect();
        let more: Vec<(String, Value)> = concretes
            .iter()
            .map(|(id, c)| (id.clone(), c.document().clone()))
            .collect();
        let instances: BTreeSet<(String, String)> = docs
            .iter()
            .filter_map(|d| Instance::read(&d.value))
            .filter(|i| concretes.contains_key(i.concrete()))
            .filter_map(|i| Some((i.concrete().to_string(), i.id()?.to_string())))
            .collect();
        Ok(Self {
            docs,
            concretes,
            more,
            instances,
        })
    }
}

/// $schema の申告から対象の種類を見分ける。見分けられなければ、取りに行かずに断る理由。
fn classify(
    concretes: &BTreeMap<String, ConcreteSchema>,
    path: &str,
    declared: &str,
) -> Result<TargetKind, Refused> {
    if ABSTRACT_VERSIONS.contains(&declared) {
        Ok(TargetKind::Concrete)
    } else if declared.starts_with("urn:semantic-agent-schema:base:semantic-abstract-schema:") {
        Err(refused(
            "schema_not_listed",
            path,
            format!(
                "{declared} は SAS が入れて出す Abstract の版（{}）に無い。取りに行かない",
                ABSTRACT_VERSIONS.join(", ")
            ),
        ))
    } else if declared.is_empty() {
        Err(refused("no_schema", path, "$schema が無い".to_string()))
    } else if concretes.contains_key(declared) {
        Ok(TargetKind::Instance)
    } else {
        Err(refused(
            "unknown_schema",
            path,
            format!(
                "{declared} はリポジトリの Concrete にも Abstract の版にも無い。取りに行かない"
            ),
        ))
    }
}

/// 形（JSON Schema）の外れ（JSON Pointer と理由）。Concrete は申告した Abstract と方言の両方、インスタンスは従う Concrete に照らす。
fn form_violations(
    checker: &dyn SchemaChecker,
    kind: TargetKind,
    declared: &str,
    value: &Value,
    more: &[(String, Value)],
) -> Result<Vec<(String, String)>, String> {
    let against: Vec<&str> = match kind {
        TargetKind::Concrete => vec![declared, DIALECT],
        TargetKind::Instance => vec![declared],
    };
    let mut broken: Vec<(String, String)> = Vec::new();
    for schema in against {
        for v in checker.check(schema, value, more)? {
            if !broken.contains(&v) {
                broken.push(v);
            }
        }
    }
    Ok(broken)
}

/// UC4 の機械の rule を検証し、対象ごとに検証の操作を記録する。
///
/// # Errors
/// 対象を読めない ・ 申告した Abstract の版が閉じた一覧に無い（取りに行かない）・ 従う schema が分からない ・ 記録できないとき。
pub fn validate(
    scope: &Scope,
    store: &dyn DocumentStore,
    checker: &dyn SchemaChecker,
    who: &dyn IdentifyCaller,
    log: &mut dyn OperationLog,
) -> Result<Validation, Refused> {
    run_validate(scope, false, store, checker, who, log)
}

/// UC4 の範囲「入力が変わった rule だけ」：rule ごとに、前の判定が読んだ所の指紋といまの指紋を比べ、
/// 違う rule（前の判定が無い rule を含む）だけを判定して記録する。変わった rule が1つも無い対象は記録しない。
///
/// # Errors
/// validate と同じ。
pub fn validate_stale(
    scope: &Scope,
    store: &dyn DocumentStore,
    checker: &dyn SchemaChecker,
    who: &dyn IdentifyCaller,
    log: &mut dyn OperationLog,
) -> Result<Validation, Refused> {
    run_validate(scope, true, store, checker, who, log)
}

/// 判定と、その判定が読んだ所（JSON Pointer。"" は文書全体）。
/// 読んだ所は当たる rule の所で、形の検査と参照の先の判定は文書全体を読む。
type Judged = (RuleResult, String);

fn whole(r: RuleResult) -> Judged {
    (r, String::new())
}

/// 1つの対象のすべての rule を判定する。
fn judge(
    d: &Doc,
    kind: TargetKind,
    declared: &str,
    checker: &dyn SchemaChecker,
    corpus: &Corpus,
    known_concretes: &BTreeSet<String>,
) -> Result<Vec<Judged>, Refused> {
    let Corpus {
        concretes,
        more,
        instances,
        ..
    } = corpus;
    let mut results: Vec<Judged> = Vec::new();
    let broken = form_violations(checker, kind, declared, &d.value, more)
        .map_err(|e| refused("checker", &d.path, e))?;
    if broken.is_empty() {
        results.push(whole(RuleResult::new(
            "json-schema",
            "",
            By::Machine,
            Verdict::Verified,
            "形が従う",
        )));
    }
    for (at, why) in broken {
        results.push(whole(RuleResult::new(
            "json-schema",
            &at,
            By::Machine,
            Verdict::Violated,
            &why,
        )));
    }
    match kind {
        TargetKind::Concrete => {
            let abstract_doc = checker.shipped(declared).ok_or_else(|| {
                refused("checker", &d.path, format!("{declared} を検査器が持たない"))
            })?;
            let names = extension_names(&abstract_doc);
            let concrete = ConcreteSchema::read(&d.value).ok_or_else(|| {
                refused("checker", &d.path, "Concrete として読めない".to_string())
            })?;
            for rule in rules_for_concrete(&abstract_doc, &concrete) {
                if let Some(rs) = concrete.judge(&rule.id, &rule.pointer, &names) {
                    results.extend(rs.into_iter().map(|r| (r, rule.pointer.clone())));
                } else if rule.id == "target-exists" {
                    let rs = check_targets(concrete.document(), known_concretes);
                    if rs.is_empty() {
                        results.push(whole(RuleResult::new(
                            &rule.id,
                            &rule.pointer,
                            By::Machine,
                            Verdict::Verified,
                            "x-relationship の to が無い",
                        )));
                    }
                    results.extend(rs.into_iter().map(whole));
                } else {
                    results.push((judge_others(&rule), rule.pointer.clone()));
                }
            }
        }
        TargetKind::Instance => {
            let concrete_doc = concretes
                .get(declared)
                .map(ConcreteSchema::document)
                .cloned()
                .unwrap_or_default();
            for rule in rules_for_instance(&concrete_doc, &d.value) {
                results.push((judge_others(&rule), rule.pointer.clone()));
            }
            results.extend(
                check_relationships(&concrete_doc, &d.value, instances)
                    .into_iter()
                    .map(whole),
            );
        }
    }
    Ok(results)
}

/// 判定が読んだ所の指紋。
fn reads_of(document: &Value, at: &str) -> Fingerprint {
    Fingerprint::of_read(at, document.pointer(at))
}

/// 前の判定のあと、読んだ所が変わった rule の id。前の判定が無い rule も入る。
/// いまの判定と前の判定で、rule ごとに「読んだ所の指紋」の集まりを比べる。
fn changed_rules(document: &Value, now: &[Judged], before: &[Judgment]) -> BTreeSet<String> {
    let mut current: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();
    for (r, at) in now {
        current
            .entry(r.rule())
            .or_default()
            .insert(reads_of(document, at).as_str().to_string());
    }
    let mut previous: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();
    for j in before {
        previous
            .entry(j.rule.rule.as_str())
            .or_default()
            .insert(j.rule.reads.clone());
    }
    current
        .into_iter()
        .filter(|(rule, reads)| previous.get(rule) != Some(reads))
        .map(|(rule, _)| rule.to_string())
        .collect()
}

fn run_validate(
    scope: &Scope,
    stale_only: bool,
    store: &dyn DocumentStore,
    checker: &dyn SchemaChecker,
    who: &dyn IdentifyCaller,
    log: &mut dyn OperationLog,
) -> Result<Validation, Refused> {
    let corpus = Corpus::load(store)?;
    let concretes = &corpus.concretes;

    // 対象を決め、種類を見分ける
    let targets: Vec<Doc> = match scope {
        Scope::Paths(paths) => {
            let mut out = Vec::new();
            for p in paths {
                let text = store.read(p).map_err(|e| refused("not_found", p, e))?;
                let value: Value = serde_json::from_str(&text)
                    .map_err(|e| refused("not_json", p, format!("JSON として読めない：{e}")))?;
                out.push(Doc {
                    path: p.clone(),
                    text,
                    value,
                });
            }
            out
        }
        Scope::All => corpus
            .docs
            .iter()
            .filter(|d| {
                ConcreteSchema::declares_abstract(&d.value)
                    || d.value
                        .get("$schema")
                        .and_then(Value::as_str)
                        .is_some_and(|s| concretes.contains_key(s))
            })
            .cloned()
            .collect(),
    };
    let mut classified = Vec::new();
    for d in targets {
        let declared = d
            .value
            .get("$schema")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let kind = classify(concretes, &d.path, &declared)?;
        classified.push((d, kind, declared));
    }

    // 判定する。入力が変わった rule だけのときは、読んだ所が前の判定と同じ rule を除く
    let mut known_concretes: BTreeSet<String> = concretes.keys().cloned().collect();
    known_concretes.extend(ABSTRACT_VERSIONS.iter().map(ToString::to_string));
    let mut judged = Vec::new();
    let mut untracked = Vec::new();
    let mut unchanged_targets = Vec::new();
    for (d, kind, declared) in classified {
        let fingerprint = Fingerprint::of(d.text.as_bytes());
        // SAS を通らない変更：いまの中身が、記録のどの事後とも違う。知らせて、いまの中身を検証する
        if let Some(expected) =
            untracked_since(log, &d.path, fingerprint.as_str()).map_err(log_refused)?
            && let Some(last) = log.writes(&d.path).map_err(log_refused)?.pop()
        {
            untracked.push(Untracked {
                path: d.path.clone(),
                last_operation: last.operation,
                expected,
                actual: fingerprint.as_str().to_string(),
            });
        }
        let mut results = judge(&d, kind, &declared, checker, &corpus, &known_concretes)?;
        let mut unchanged_rules = Vec::new();
        if stale_only {
            let before = log.last_judgments(&d.path).map_err(log_refused)?;
            let changed = changed_rules(&d.value, &results, &before);
            for (r, _) in &results {
                if !changed.contains(r.rule()) && !unchanged_rules.iter().any(|u| u == r.rule()) {
                    unchanged_rules.push(r.rule().to_string());
                }
            }
            results.retain(|(r, _)| changed.contains(r.rule()));
            if results.is_empty() {
                unchanged_targets.push(d.path);
                continue;
            }
        }
        let rules: Vec<RuleResult> = results.iter().map(|(r, _)| r.clone()).collect();
        let reads: Vec<String> = results
            .iter()
            .map(|(_, at)| reads_of(&d.value, at).as_str().to_string())
            .collect();
        let counts = Counts::of(&rules);
        judged.push((
            d.path,
            kind,
            declared,
            fingerprint,
            rules,
            reads,
            counts,
            unchanged_rules,
        ));
    }

    // 対象ごとに検証を記録する（対象を変えないので指紋は1つ）
    let rows: Vec<ValidateRow> = judged
        .iter()
        .map(|(path, _, _, fp, results, reads, _, _)| ValidateRow {
            target_path: path.clone(),
            hash: fp.as_str().to_string(),
            results: results
                .iter()
                .zip(reads)
                .map(|(r, reads)| RuleRow {
                    rule: r.rule().to_string(),
                    pointer: r.pointer().to_string(),
                    by: r.by().as_str().to_string(),
                    result: r.verdict().as_str().to_string(),
                    reason: r.reason().to_string(),
                    reads: reads.clone(),
                })
                .collect(),
        })
        .collect();
    let recorded = record_validations(who, log, &rows).map_err(log_refused)?;
    let mut total = Counts::default();
    let targets = judged
        .into_iter()
        .zip(recorded.operations)
        .map(
            |(
                (path, kind, schema, fingerprint, results, _, counts, unchanged_rules),
                operation,
            )| {
                total.merge(&counts);
                TargetResult {
                    path,
                    kind,
                    schema,
                    fingerprint,
                    results,
                    counts,
                    operation,
                    unchanged_rules,
                }
            },
        )
        .collect();
    Ok(Validation {
        parent: recorded.parent,
        targets,
        counts: total,
        stale_only,
        unchanged_targets,
        untracked,
    })
}

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
    /// 呼んだ親のセッションの ID（人なら None）
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
    /// 事前の指紋（作成では None）
    pub before: Option<Fingerprint>,
    /// 事後の指紋
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
    /// 直す前に読んだ中身の指紋。無ければ、SAS が最後に書いた事後の指紋と比べる
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

fn entry(who: &dyn IdentifyCaller) -> Entry {
    match who.identify() {
        Caller::Human => Entry::Human,
        Caller::Parent { .. } => Entry::Tool,
    }
}

/// 集約の書く条件。
fn check_write(
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
        TargetKind::Instance => match (Instance::read(after), concrete) {
            (Some(i), Some(c)) => i.check_write(
                before.and_then(Instance::read).as_ref(),
                c.document(),
                entry,
            ),
            _ => return Ok(()),
        },
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

fn log_refused(e: String) -> Refused {
    Refused {
        code: "log",
        message: e,
        ..Refused::default()
    }
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
        .ok_or_else(|| log_refused(format!("操作 {} の事後の指紋を読めない", stored.operation)))?;
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
    checker: &dyn SchemaChecker,
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
        cause: None,
        fields: Vec::new(),
    };
    let broken = form_violations(checker, kind, &request.concrete, &value, &corpus.more)
        .map_err(|e| refused("checker", path, e))?;
    if !broken.is_empty() {
        return Err(refused(
            "invalid_value",
            path,
            format!("型に従わない：{}", violations_message(&broken)),
        ));
    }
    if let Err((code, message)) = check_write(
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

/// UC2 更新する（機械）：いまの中身の指紋を確かめ、JSON Patch を当て、型と集約の書く条件を確かめてから書き、
/// 前後の中身から求めた差分と一緒に更新の操作を記録する。前の判定のうち入力が変わったものと、
/// Concrete の更新で外れたインスタンスを返す。
///
/// # Errors
/// 依頼を読めない ・ 対象が無い ・ 従う型が決まらない（何も記録しない）。
/// 指紋が違う ・ 差分を当てられない ・ 形や書く条件に外れる（書かずに記録する）。書けない ・ 記録できないとき。
pub fn update(
    request: &UpdateRequest,
    store: &dyn DocumentStore,
    checker: &dyn SchemaChecker,
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
        cause: None,
        fields: Vec::new(),
    };
    // 比べる指紋：呼ぶ側が読んだ指紋（--if-match）か、いまの中身が記録のどの事後とも違うときの最後の事後の指紋。
    // SAS が一度も書いていない ・ いまの中身が記録のどれかの事後なら比べる指紋が無く、そのまま書く
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
                "いまの中身の指紋は {}。{from}指紋 {expected} と違うので直さない。間に入った変更（interleaved）を読んでから直す{}",
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
    let broken = form_violations(checker, kind, &declared, &after, &more)
        .map_err(|e| refused("checker", path, e))?;
    if !broken.is_empty() {
        return Err(refused(
            "invalid_value",
            path,
            format!("当てた結果が型に従わない：{}", violations_message(&broken)),
        ));
    }
    if let Err((code, message)) = check_write(
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
        Some(id) => violated_instances(checker, &corpus, path, id, &more)?,
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
    checker: &dyn SchemaChecker,
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
        let broken = checker
            .check(id, &d.value, more)
            .map_err(|e| refused("checker", &d.path, e))?;
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

/// 中身の指紋が記録のどれかの事後と同じコミットを、新しい順に探す。見つかればコミットと中身と、その指紋。
/// DB はワークツリーで1つなので、最後の事後だけでなく、この経路の記録のどの事後でもよい
/// （このワークツリーの履歴にあるのは、ほかのワークツリーが書く前の事後かもしれない）。
fn find_commit(
    committed: &dyn CommitReader,
    path: &str,
    recorded: &[String],
) -> Result<Option<(String, Value, String)>, Refused> {
    let store_err = |e: String| refused("commits", path, e);
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
    checker: &dyn SchemaChecker,
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
        cause: None,
        fields: Vec::new(),
    };
    let found = match &last {
        Some(_) => find_commit(committed, path, &recorded)?,
        None => None,
    };
    // 事前：前の中身のコミットが見つかればその指紋（記録のどれかの事後）、無ければ記録の最後の事後
    let expected: Option<String> = match (&found, &last) {
        (Some((_, _, fp)), _) => Some(fp.clone()),
        (None, l) => l.clone(),
    };
    let broken = form_violations(checker, kind, &declared, &current, &corpus.more)
        .map_err(|e| refused("checker", path, e))?;
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
            violated_instances(checker, &corpus, path, id, &more)?
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
