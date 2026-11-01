//! Concrete とインスタンスの検証（業務領域「書いて検証する」の検証のモジュールの手続き）。
//!
//! - UC4 検証する：対象を読み、Concrete かインスタンスかを $schema で見分け、形（JSON Schema）・ 組み込みの検査 ・
//!   当たる rule を判定し、対象ごとに検証の操作を1件記録して、結果と記録の ID を返す。検証は対象を変えない。
//! - UC4 の範囲「入力が変わった rule だけ」と、SAS を通らない変更があった対象の知らせ。
//! - UC4 の意味の部分：判定の手順を打ったセッションに返し（記録しない）、そのセッションが渡した判定を機械の結果と合わせて記録する。

use crate::ports::{DocumentStore, SchemaValidator, ValidationLog};
use crate::use_cases::meaning::{
    Step, answer_json, check_answer, command_line, match_steps, procedure,
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use validation_domain::services::{
    Judged, concrete_rules, instance_rules, reads_hash, rule_reads, whole,
};
use validation_domain::values::{By, Counts, Judgment, RuleResult, RuleRow, ValidateRow, Verdict};
use writing_domain::aggregates::ConcreteSchema;
use writing_domain::services::field_schema;
use writing_domain::values::{Caller, ContentHash, Corpus, Doc, TargetKind, untracked_since};

/// 何を検証するか。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scope {
    /// 経路を指定した対象
    Paths(Vec<String>),
    /// リポジトリの Concrete とインスタンスのすべて
    All,
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
    /// 検証したときの中身のハッシュ
    pub hash: ContentHash,
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
    /// その操作の事後のハッシュ
    pub expected: String,
    /// いまの中身のハッシュ
    pub actual: String,
}

/// 検証の結果と、記録の ID。
#[derive(Debug, Clone, PartialEq)]
pub struct Validation {
    /// 打ったセッションの ID（人なら None）
    pub session: Option<String>,
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
    /// 決まった名前（usage ・ schema_not_listed ・ not_found ・ not_json ・ no_schema ・ unknown_schema ・ store ・ validator ・ log、
    /// 渡された判定が手順に合わないもの：invalid_answer、手順のハッシュがいまの中身と違うもの：conflict）
    pub code: &'static str,
    /// 対象の経路（あれば）
    pub path: Option<Box<str>>,
    /// 理由
    pub message: String,
    /// 次にすることの案内（判定の直し方 ・ 手順を取り直すコマンド）
    pub hint: Option<Box<String>>,
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

/// リポジトリの文書を読む（Concrete の一覧に使う）。読めない文書は除く。
///
/// # Errors
/// 文書の一覧を読めないとき。
fn load_corpus(store: &dyn DocumentStore) -> Result<Corpus, Refused> {
    let paths = store.list().map_err(|e| Refused {
        code: "store",
        message: e,
        ..Refused::default()
    })?;
    Ok(Corpus::read(
        paths
            .into_iter()
            .filter_map(|p| store.read(&p).ok().map(|t| (p, t))),
    ))
}

/// $schema の申告から対象の種類を見分ける。見分けられなければ、取りに行かずに断る理由。
fn classify(
    concretes: &BTreeMap<String, ConcreteSchema>,
    path: &str,
    declared: &str,
) -> Result<TargetKind, Refused> {
    TargetKind::classify(concretes, declared)
        .map_err(|u| refused(u.code(), path, u.message().to_string()))
}

/// 形（JSON Schema）の外れ（JSON Pointer と理由）。Concrete は申告した Abstract と方言の両方、インスタンスは従う Concrete に照らす。
///
/// # Errors
/// バリデーターが動かないとき。
fn form_violations(
    validator: &dyn SchemaValidator,
    kind: TargetKind,
    declared: &str,
    value: &Value,
    more: &[(String, Value)],
) -> Result<Vec<(String, String)>, String> {
    let mut broken: Vec<(String, String)> = Vec::new();
    for schema in kind.form_schemas(declared) {
        for v in validator.check(schema, value, more)? {
            if !broken.contains(&v) {
                broken.push(v);
            }
        }
    }
    Ok(broken)
}

/// UC4 検証する：形 ・ 組み込みの検査 ・ 当たる rule を判定し（rule は判定を渡されなければ unsupported）、対象ごとに検証の操作を記録する。
///
/// # Errors
/// 対象を読めない ・ 申告した Abstract の版が閉じた一覧に無い（取りに行かない）・ 従う schema が分からない ・ 記録できないとき。
pub fn validate(
    scope: &Scope,
    store: &dyn DocumentStore,
    validator: &dyn SchemaValidator,
    caller: &Caller,
    log: &mut dyn ValidationLog,
) -> Result<Validation, Refused> {
    run_validate(scope, false, store, validator, caller, log, None)
}

/// UC4 の範囲「入力が変わった rule だけ」：rule ごとに、前の判定が読んだ所のハッシュといまのハッシュを比べ、
/// 違う rule（前の判定が無い rule を含む）だけを判定して記録する。変わった rule が1つも無い対象は記録しない。
///
/// # Errors
/// validate と同じ。
pub fn validate_stale(
    scope: &Scope,
    store: &dyn DocumentStore,
    validator: &dyn SchemaValidator,
    caller: &Caller,
    log: &mut dyn ValidationLog,
) -> Result<Validation, Refused> {
    run_validate(scope, true, store, validator, caller, log, None)
}

/// UC4 の意味の部分の1つ目：機械の検証をしたうえで、意味の rule（ai）の判定の手順を対象ごとに返す。記録しない。
/// 判定する rule が無い対象の手順は無い（どの対象にも無ければ空）。
/// 入力が変わった rule だけ（stale_only）のときは、前に判定できなかった rule も手順に入れる。
///
/// # Errors
/// validate と同じ。
pub fn semantic_procedures(
    scope: &Scope,
    stale_only: bool,
    store: &dyn DocumentStore,
    validator: &dyn SchemaValidator,
    caller: &Caller,
    log: &dyn ValidationLog,
) -> Result<Vec<Value>, Refused> {
    let (corpus, current, _) = current_for(scope, stale_only, true, store, validator, log)?;
    Ok(current
        .iter()
        .filter_map(|c| {
            let (steps, _) = steps_of(c, &corpus, validator);
            if steps.is_empty() {
                return None;
            }
            let command = answer_command(&c.doc.path, stale_only, caller);
            Some(procedure(
                &c.doc.path,
                &c.declared,
                c.hash.as_str(),
                &steps,
                &command,
            ))
        })
        .collect())
}

/// UC4 の意味の部分の2つ目：打ったセッションが渡した判定（answer）を、形 ・ 手順のハッシュ ・ step ・ rule ・ pointer で
/// 確かめ、機械の結果と合わせて記録する。合わなければ記録せずに断る。
///
/// # Errors
/// validate と同じ。ほかに、答えが形に合わない ・ 手順と突き合わない（invalid_answer）・ 手順のハッシュがいまの中身と違う（conflict）とき。
pub fn validate_with_answer(
    path: &str,
    stale_only: bool,
    answer: &str,
    store: &dyn DocumentStore,
    validator: &dyn SchemaValidator,
    caller: &Caller,
    log: &mut dyn ValidationLog,
) -> Result<Validation, Refused> {
    run_validate(
        &Scope::Paths(vec![path.to_string()]),
        stale_only,
        store,
        validator,
        caller,
        log,
        Some(answer),
    )
}

/// 判定を渡すときに打つコマンド。--session で名乗ったセッションは、渡すときも名乗る。
fn answer_command(path: &str, stale_only: bool, caller: &Caller) -> String {
    let mut args = vec!["sas", "validate", path];
    if stale_only {
        args.push("--stale");
    }
    args.extend(["--answer", "<判定の JSON のファイル | ->"]);
    if let Some(id) = caller.flagged() {
        args.extend(["--session", id]);
    }
    command_line(&args)
}

/// 手順を取り直すコマンド。
fn procedure_command(path: &str, stale_only: bool) -> String {
    let mut args = vec!["sas", "validate", path];
    if stale_only {
        args.push("--stale");
    }
    args.push("--semantic");
    command_line(&args)
}

/// 対象1つの、意味の rule（ai）を判定するステップと、そのステップが当たる results の位置。
fn steps_of(
    c: &Current,
    corpus: &Corpus,
    validator: &dyn SchemaValidator,
) -> (Vec<Step>, Vec<usize>) {
    let schema_doc = match c.kind {
        TargetKind::Concrete => validator.shipped(&c.declared).unwrap_or_default(),
        TargetKind::Instance => corpus
            .concretes
            .get(&c.declared)
            .map(|k| k.document().clone())
            .unwrap_or_default(),
    };
    let mut steps = Vec::new();
    let mut at = Vec::new();
    for (i, ((r, _), def)) in c.results.iter().zip(&c.definitions).enumerate() {
        if r.by() != By::Ai {
            continue;
        }
        let field = field_schema(&schema_doc, r.pointer(), &corpus.more);
        let field = field.as_ref();
        steps.push(Step {
            no: steps.len() + 1,
            rule: r.rule().to_string(),
            pointer: r.pointer().to_string(),
            definition: def.clone().unwrap_or_default(),
            title: field
                .and_then(|f| f.get("title"))
                .and_then(Value::as_str)
                .map(ToString::to_string),
            description: field
                .and_then(|f| f.get("description"))
                .and_then(Value::as_str)
                .map(ToString::to_string),
            read: field
                .and_then(|f| f.pointer("/x-prompt/read"))
                .filter(|r| r.as_object().is_some_and(|m| !m.is_empty()))
                .cloned(),
            current: c
                .doc
                .value
                .pointer(r.pointer())
                .cloned()
                .unwrap_or(Value::Null),
            reads: rule_reads(def.as_ref())
                .into_iter()
                .map(|p| {
                    let v = c.doc.value.pointer(&p).cloned().unwrap_or(Value::Null);
                    (p, v)
                })
                .collect(),
        });
        at.push(i);
    }
    (steps, at)
}

/// 渡された判定で、対象の意味の rule の結果を置き換える。
fn apply_answer(
    current: &mut [Current],
    corpus: &Corpus,
    validator: &dyn SchemaValidator,
    answer: &str,
    stale_only: bool,
) -> Result<(), Refused> {
    let [c] = current else {
        return Err(Refused {
            code: "usage",
            message: "--answer は対象を1つだけ渡してください".to_string(),
            ..Refused::default()
        });
    };
    let path = c.doc.path.clone();
    let invalid = |why: String| {
        let mut r = refused(
            "invalid_answer",
            &path,
            format!("判定の答えを記録しない：{why}"),
        );
        r.hint = Some(Box::new(
            "答えを手順の answer の形と steps に合わせて直し、打ち直す".to_string(),
        ));
        r
    };
    let v = answer_json(answer).map_err(invalid)?;
    check_answer(validator, &v).map_err(|e| invalid(format!("答えの形に合わない：{e}")))?;
    let hash = v["hash"].as_str().unwrap_or_default();
    if hash != c.hash.as_str() {
        let mut r = refused(
            "conflict",
            &path,
            format!(
                "判定した中身のハッシュ（{hash}）が、いまの中身（{}）と違う。判定を記録に入れない",
                c.hash.as_str()
            ),
        );
        r.hint = Some(Box::new(format!(
            "いまの中身で手順を取り直して判定し直す：{}",
            procedure_command(&path, stale_only)
        )));
        return Err(r);
    }
    let (steps, at) = steps_of(c, corpus, validator);
    let judged = match_steps(&v, &steps).map_err(invalid)?;
    for (step, i) in steps.iter().zip(at) {
        if let Some((verdict, reason)) = judged.get(&step.no) {
            c.results[i].0 = RuleResult::new(&step.rule, &step.pointer, By::Ai, *verdict, reason);
        }
    }
    Ok(())
}

/// Abstract の語彙（入れ子に書ける x- の名前と、その値の形の対応を持つ）。
const VOCABULARY: &str = "urn:semantic-agent-schema:published-language:vocabulary:v1";

/// 1つの対象のすべての rule を判定する。
fn judge(
    d: &Doc,
    kind: TargetKind,
    declared: &str,
    validator: &dyn SchemaValidator,
    corpus: &Corpus,
) -> Result<Vec<Judged>, Refused> {
    let Corpus {
        concretes, more, ..
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
            let concrete = ConcreteSchema::read(&d.value).ok_or_else(|| {
                refused("validator", &d.path, "Concrete として読めない".to_string())
            })?;
            let vocabulary = validator.shipped(VOCABULARY);
            results.extend(concrete_rules(
                &abstract_doc,
                vocabulary.as_ref(),
                &concrete,
            ));
        }
        TargetKind::Instance => {
            let concrete_doc = concretes
                .get(declared)
                .map(ConcreteSchema::document)
                .cloned()
                .unwrap_or_default();
            results.extend(instance_rules(&concrete_doc, &d.value, more));
        }
    }
    Ok(results)
}

/// いま判定した対象1つ（記録する前）。
pub struct Current {
    /// 対象の文書
    pub doc: Doc,
    /// 種類
    pub kind: TargetKind,
    /// 従う schema の $id
    pub declared: String,
    /// いまの中身のハッシュ
    pub hash: ContentHash,
    /// rule ごとの結果と、その判定が読んだ所のハッシュ
    pub results: Vec<(RuleResult, ContentHash)>,
    /// rule ごとの定義（x-prompt の write の1行。results と同じ並び。機械だけの検査は None）
    pub definitions: Vec<Option<Value>>,
}

/// 対象を決めて種類を見分け、いまの中身ですべての rule を判定する。記録しない。
///
/// # Errors
/// 対象を読めない ・ 申告した Abstract の版が閉じた一覧に無い ・ 従う schema が分からない ・ バリデーターが動かないとき。
pub fn judge_current(
    scope: &Scope,
    store: &dyn DocumentStore,
    validator: &dyn SchemaValidator,
) -> Result<(Corpus, Vec<Current>), Refused> {
    let corpus = load_corpus(store)?;
    let concretes = &corpus.concretes;
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
    let mut current = Vec::new();
    for (doc, kind, declared) in classified {
        let judged = judge(&doc, kind, &declared, validator, &corpus)?;
        let definitions = judged.iter().map(|(_, _, d)| d.clone()).collect();
        let results = judged
            .into_iter()
            .map(|(r, at, definition)| {
                // 当たる所の値 ・ rule の reads が指す所の値 ・ rule の文（どれかが変われば、前の判定は効かない）
                let reads = reads_hash(&doc.value, &at, definition.as_ref());
                (r, reads)
            })
            .collect();
        current.push(Current {
            hash: ContentHash::of(doc.text.as_bytes()),
            doc,
            kind,
            declared,
            results,
            definitions,
        });
    }
    Ok((corpus, current))
}

/// 前の判定のあと、読んだ所が変わった rule の id。前の判定が無い rule も入る。意味の判定の手順を出す ・ 判定を受け取るとき
/// （retry_unjudged）は、前に判定できなかった（unsupported ・ error）rule も入る。
/// いまの判定と前の判定で、rule ごとに「読んだ所のハッシュ」の集まりを比べる。
fn changed_rules(
    now: &[(RuleResult, ContentHash)],
    before: &[Judgment],
    retry_unjudged: bool,
) -> BTreeSet<String> {
    let mut current: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();
    for (r, reads) in now {
        current
            .entry(r.rule())
            .or_default()
            .insert(reads.as_str().to_string());
    }
    let mut previous: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();
    for j in before
        .iter()
        .filter(|j| !retry_unjudged || (j.rule.result != "unsupported" && j.rule.result != "error"))
    {
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

/// 経路 → 入力が変わった rule だけを検証したときに除いた rule の id。
type Unchanged = BTreeMap<String, Vec<String>>;

/// いまの中身で判定し、入力が変わった rule だけ（stale_only）のときは読んだ所が前の判定と同じ rule を除く。
/// retry_unjudged なら、前に判定できなかった（unsupported ・ error）rule も残す。経路 → 除いた rule の id も返す。
fn current_for(
    scope: &Scope,
    stale_only: bool,
    retry_unjudged: bool,
    store: &dyn DocumentStore,
    validator: &dyn SchemaValidator,
    log: &dyn ValidationLog,
) -> Result<(Corpus, Vec<Current>, Unchanged), Refused> {
    let (corpus, mut current) = judge_current(scope, store, validator)?;
    let mut unchanged = Unchanged::new();
    if stale_only {
        for c in &mut current {
            let before = log.last_judgments(&c.doc.path).map_err(log_refused)?;
            let changed = changed_rules(&c.results, &before, retry_unjudged);
            let rules = unchanged.entry(c.doc.path.clone()).or_default();
            for (r, _) in &c.results {
                if !changed.contains(r.rule()) && !rules.iter().any(|u| u == r.rule()) {
                    rules.push(r.rule().to_string());
                }
            }
            let keep: Vec<bool> = c
                .results
                .iter()
                .map(|(r, _)| changed.contains(r.rule()))
                .collect();
            let mut k = keep.iter();
            c.results.retain(|_| *k.next().unwrap_or(&false));
            let mut k = keep.iter();
            c.definitions.retain(|_| *k.next().unwrap_or(&false));
        }
    }
    Ok((corpus, current, unchanged))
}

fn run_validate(
    scope: &Scope,
    stale_only: bool,
    store: &dyn DocumentStore,
    validator: &dyn SchemaValidator,
    caller: &Caller,
    log: &mut dyn ValidationLog,
    answer: Option<&str>,
) -> Result<Validation, Refused> {
    let (corpus, mut current, mut unchanged) =
        current_for(scope, stale_only, answer.is_some(), store, validator, log)?;
    if let Some(answer) = answer {
        apply_answer(&mut current, &corpus, validator, answer, stale_only)?;
    }
    // 入力が変わった rule だけのときは、読んだ所が前の判定と同じ rule を除く
    let mut judged = Vec::new();
    let mut untracked = Vec::new();
    let mut unchanged_targets = Vec::new();
    for Current {
        doc: d,
        kind,
        declared,
        hash,
        results,
        ..
    } in current
    {
        // SAS を通らない変更：いまの中身が、記録のどの事後とも違う。知らせて、いまの中身を検証する
        let mut writes = log.writes(&d.path).map_err(log_refused)?;
        if let Some(expected) = untracked_since(&writes, hash.as_str())
            && let Some(last) = writes.pop()
        {
            untracked.push(Untracked {
                path: d.path.clone(),
                last_operation: last.operation,
                expected,
                actual: hash.as_str().to_string(),
            });
        }
        let unchanged_rules = unchanged.remove(&d.path).unwrap_or_default();
        if stale_only && results.is_empty() {
            unchanged_targets.push(d.path);
            continue;
        }
        let rules: Vec<RuleResult> = results.iter().map(|(r, _)| r.clone()).collect();
        let reads: Vec<String> = results
            .iter()
            .map(|(_, reads)| reads.as_str().to_string())
            .collect();
        let counts = Counts::of(&rules);
        judged.push((
            d.path,
            kind,
            declared,
            hash,
            rules,
            reads,
            counts,
            unchanged_rules,
        ));
    }

    // 対象ごとに検証を記録する（対象を変えないのでハッシュは1つ）
    let rows: Vec<ValidateRow> = judged
        .iter()
        .map(
            |(path, _, declared, hash, results, reads, _, _)| ValidateRow {
                target_path: path.clone(),
                hash: hash.as_str().to_string(),
                schema: Some(declared.clone()),
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
            },
        )
        .collect();
    // 打ったセッションの行は、入口が先に足している
    let session = caller.session_id();
    let mut operations = Vec::new();
    for row in &rows {
        operations.push(log.append_validate(session, row).map_err(log_refused)?);
    }
    let mut total = Counts::default();
    let targets = judged
        .into_iter()
        .zip(operations)
        .map(
            |((path, kind, schema, hash, results, _, counts, unchanged_rules), operation)| {
                total.merge(&counts);
                TargetResult {
                    path,
                    kind,
                    schema,
                    hash,
                    results,
                    counts,
                    operation,
                    unchanged_rules,
                }
            },
        )
        .collect();
    Ok(Validation {
        session: session.map(ToString::to_string),
        targets,
        counts: total,
        stale_only,
        unchanged_targets,
        untracked,
    })
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
