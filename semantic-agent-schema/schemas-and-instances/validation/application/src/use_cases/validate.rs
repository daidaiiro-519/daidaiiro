//! Concrete とインスタンスの検証（業務領域「書いて検証する」の検証のモジュールの手続き）。
//!
//! - UC4 検証する（機械の rule）：対象を読み、Concrete かインスタンスかを $schema で見分け、形（JSON Schema）・
//!   当たる rule ・ 参照の先を判定し、対象ごとに検証の操作を1件記録して、結果と記録の ID を返す。検証は対象を変えない。
//! - UC4 の範囲「入力が変わった rule だけ」と、SAS を通らない変更があった対象の知らせ。
//! - 書くモジュール（writing-application）が書く前後に使う、対象の見分けと形の確かめ、断った理由の形もここに置く。

use crate::ports::{DocumentStore, SchemaValidator};
use recording_application::ports::{
    Caller, IdentifyCaller, Judgment, OperationLog, RuleRow, ValidateRow,
};
use recording_application::use_cases::history::{Interleaved, record_validations, untracked_since};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use validation_domain::services::{
    check_relationships, check_targets, judge_concrete, judge_others, rules_for_concrete,
    rules_for_instance,
};
use validation_domain::values::{By, Counts, RuleResult, Verdict};
use writing_domain::aggregates::{
    ABSTRACT_VERSIONS, ConcreteSchema, Entry, Instance, extension_names,
};
use writing_domain::values::Fingerprint;

/// 方言（Concrete の入れ子の x- の形を決める）。Concrete は Abstract の本体と、この方言の両方に照らす。
pub const DIALECT: &str = "urn:semantic-agent-schema:published-language:dialect:v1";

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
    /// 判定の5つの区分ごとの件数
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
    /// invalid_input ・ request_reused ・ recorded ・ not_recorded ・ nothing_to_adopt ・ store ・ validator ・ commits ・ log、
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

/// 対象の経路を付けて断る。
#[must_use]
pub fn refused(code: &'static str, path: &str, message: String) -> Refused {
    Refused {
        code,
        path: Some(path.into()),
        message,
        ..Refused::default()
    }
}

/// リポジトリの文書1つ。
#[derive(Clone)]
pub struct Doc {
    /// 作業ディレクトリからの経路
    pub path: String,
    /// 読んだ文字列
    pub text: String,
    /// 読んだ JSON
    pub value: Value,
}

/// リポジトリの文書と、そこから読んだ Concrete ・ インスタンスの一覧。
pub struct Corpus {
    /// 読めた文書
    pub docs: Vec<Doc>,
    /// $id → Concrete
    pub concretes: BTreeMap<String, ConcreteSchema>,
    /// バリデーターに渡す Concrete（$id と文書）
    pub more: Vec<(String, Value)>,
    /// （従う Concrete の $id, インスタンスの id）
    pub instances: BTreeSet<(String, String)>,
}

impl Corpus {
    /// リポジトリの文書を読む（Concrete の一覧と、参照の先のインスタンスに使う）。
    ///
    /// # Errors
    /// 文書の一覧を読めないとき。
    pub fn load(store: &dyn DocumentStore) -> Result<Self, Refused> {
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
///
/// # Errors
/// 版の一覧に無い ・ $schema が無い ・ 知らない型のとき。
pub fn classify(
    concretes: &BTreeMap<String, ConcreteSchema>,
    path: &str,
    declared: &str,
) -> Result<TargetKind, Refused> {
    if ABSTRACT_VERSIONS.contains(&declared) {
        Ok(TargetKind::Concrete)
    } else if declared.starts_with("urn:semantic-agent-schema:published-language:abstract:") {
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
///
/// # Errors
/// バリデーターが動かないとき。
pub fn form_violations(
    validator: &dyn SchemaValidator,
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
        for v in validator.check(schema, value, more)? {
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
    validator: &dyn SchemaValidator,
    who: &dyn IdentifyCaller,
    log: &mut dyn OperationLog,
) -> Result<Validation, Refused> {
    run_validate(scope, false, store, validator, who, log)
}

/// UC4 の範囲「入力が変わった rule だけ」：rule ごとに、前の判定が読んだ所の指紋といまの指紋を比べ、
/// 違う rule（前の判定が無い rule を含む）だけを判定して記録する。変わった rule が1つも無い対象は記録しない。
///
/// # Errors
/// validate と同じ。
pub fn validate_stale(
    scope: &Scope,
    store: &dyn DocumentStore,
    validator: &dyn SchemaValidator,
    who: &dyn IdentifyCaller,
    log: &mut dyn OperationLog,
) -> Result<Validation, Refused> {
    run_validate(scope, true, store, validator, who, log)
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
    validator: &dyn SchemaValidator,
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
    let broken = form_violations(validator, kind, declared, &d.value, more)
        .map_err(|e| refused("validator", &d.path, e))?;
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
            let abstract_doc = validator.shipped(declared).ok_or_else(|| {
                refused(
                    "validator",
                    &d.path,
                    format!("{declared} をバリデーターが持たない"),
                )
            })?;
            let names = extension_names(&abstract_doc);
            let concrete = ConcreteSchema::read(&d.value).ok_or_else(|| {
                refused("validator", &d.path, "Concrete として読めない".to_string())
            })?;
            for rule in rules_for_concrete(&abstract_doc, &concrete) {
                if let Some(rs) = judge_concrete(&concrete, &rule.id, &rule.pointer, &names) {
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
    validator: &dyn SchemaValidator,
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
        let mut results = judge(&d, kind, &declared, validator, &corpus, &known_concretes)?;
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

/// 呼び出し元が人か、ツールか（集約の書く条件の入口）。
#[must_use]
pub fn entry(who: &dyn IdentifyCaller) -> Entry {
    match who.identify() {
        Caller::Human => Entry::Human,
        Caller::Parent { .. } => Entry::Tool,
    }
}

/// 記録を読み書きできなかったことを、断った理由にする。
#[must_use]
pub fn log_refused(e: String) -> Refused {
    Refused {
        code: "log",
        message: e,
        ..Refused::default()
    }
}
