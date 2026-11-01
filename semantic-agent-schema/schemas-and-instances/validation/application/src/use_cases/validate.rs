//! Concrete とインスタンスの検証（業務領域「書いて検証する」の検証のモジュールの手続き）。
//!
//! - UC4 検証する：対象を読み、Concrete かインスタンスかを $schema で見分け、形（JSON Schema）・ 組み込みの検査 ・
//!   当たる rule を判定し、対象ごとに検証の操作を1件記録して、結果と記録の ID を返す。検証は対象を変えない。
//! - UC4 の範囲「入力が変わった rule だけ」と、SAS を通らない変更があった対象の知らせ。
//! - UC4 の意味の部分：判定の手順を打ったセッションに返し（記録しない）、そのセッションが渡した判定を機械の結果と合わせて記録する。
//! - 書くモジュール（writing-application）が書く前後に使う、対象の見分けと形の確かめ、断った理由の形もここに置く。

use crate::ports::{DocumentStore, SchemaValidator};
use crate::use_cases::meaning::{
    Step, answer_json, check_answer, command_line, field_schema, match_steps, procedure,
};
use recording_application::ports::{
    Caller, IdentifyCaller, Judgment, OperationLog, RuleRow, ValidateRow, Via,
};
use recording_application::use_cases::history::{Interleaved, record_validations, untracked_since};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use validation_domain::services::{
    builtin_checks, rules_for_concrete, rules_for_instance, unjudged,
};
use validation_domain::values::{By, Counts, RuleResult, Verdict};
use writing_domain::aggregates::{ABSTRACT_VERSIONS, ConcreteSchema, Entry};
use writing_domain::values::ContentHash;

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
    /// 決まった名前（usage ・ schema_not_listed ・ not_found ・ not_json ・ no_schema ・ unknown_schema ・ exists ・
    /// invalid_input ・ request_reused ・ recorded ・ not_recorded ・ nothing_to_adopt ・ store ・ validator ・ versions ・ log、
    /// 描画で断ったもの：invalid_design_system ・ missing_view_kinds ・ template ・ out、
    /// 書く手前の判定で断ったもの：invalid_value ・ conflict ・ patch_failed ・ human_field ・ published_changed、
    /// 止めた削除：has_dependents、意味で書く手順を組めないもの：reads_cycle ・ ref_too_large、
    /// 渡された判定が手順に合わないもの：invalid_answer）
    pub code: &'static str,
    /// 対象の経路（あれば）
    pub path: Option<Box<str>>,
    /// 理由
    pub message: String,
    /// ハッシュが違って断ったとき（conflict）の、間に入った変更
    pub interleaved: Option<Box<Interleaved>>,
    /// 次にすることの案内（SAS を通らない変更を取り込むコマンドなど）
    pub hint: Option<Box<String>>,
    /// 削除を止めた理由になったもの（Concrete に従うインスタンス）
    pub blocked: Option<Box<Vec<Blocked>>>,
    /// 描画で、デザインシステムに無かった x-view の種類（missing_view_kinds）
    pub missing: Option<Box<Vec<String>>>,
}

/// 削除を止めた理由になった文書1つと、それを片付けるコマンドの案内。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Blocked {
    /// 経路
    pub path: String,
    /// 決まった名前（follows）
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
}

impl Corpus {
    /// リポジトリの文書を読む（Concrete の一覧に使う）。
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
        Ok(Self {
            docs,
            concretes,
            more,
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

/// UC4 検証する：形 ・ 組み込みの検査 ・ 当たる rule を判定し（rule は判定を渡されなければ unsupported）、対象ごとに検証の操作を記録する。
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
    run_validate(scope, false, store, validator, who, log, None)
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
    who: &dyn IdentifyCaller,
    log: &mut dyn OperationLog,
) -> Result<Validation, Refused> {
    run_validate(scope, true, store, validator, who, log, None)
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
    who: &dyn IdentifyCaller,
    log: &dyn OperationLog,
) -> Result<Vec<Value>, Refused> {
    let (corpus, current, _) = current_for(scope, stale_only, true, store, validator, log)?;
    Ok(current
        .iter()
        .filter_map(|c| {
            let (steps, _) = steps_of(c, &corpus, validator);
            if steps.is_empty() {
                return None;
            }
            let command = answer_command(&c.doc.path, stale_only, who);
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
    who: &dyn IdentifyCaller,
    log: &mut dyn OperationLog,
) -> Result<Validation, Refused> {
    run_validate(
        &Scope::Paths(vec![path.to_string()]),
        stale_only,
        store,
        validator,
        who,
        log,
        Some(answer),
    )
}

/// 判定を渡すときに打つコマンド。--session で名乗ったセッションは、渡すときも名乗る。
fn answer_command(path: &str, stale_only: bool, who: &dyn IdentifyCaller) -> String {
    let mut args = vec!["sas", "validate", path];
    if stale_only {
        args.push("--stale");
    }
    args.extend(["--answer", "<判定の JSON のファイル | ->"]);
    let caller = who.identify();
    if let Caller::Session {
        id, via: Via::Flag, ..
    } = &caller
    {
        args.extend(["--session", id.as_str()]);
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

/// いまの中身で、対象の rule ごとの読んだ所のハッシュ（rule の id ・ 当たる所 ・ ハッシュ）。
/// 更新のあと、前の判定のうち判定し直しが要るものを決めるのに使う（--stale と同じ基準）。
///
/// # Errors
/// 対象を読めない ・ 型が決まらないとき。
pub fn reads_now(
    path: &str,
    store: &dyn DocumentStore,
    validator: &dyn SchemaValidator,
) -> Result<Vec<(String, String, String)>, Refused> {
    let (_, current) = judge_current(&Scope::Paths(vec![path.to_string()]), store, validator)?;
    Ok(current
        .into_iter()
        .flat_map(|c| {
            c.results.into_iter().map(|(r, h)| {
                (
                    r.rule().to_string(),
                    r.pointer().to_string(),
                    h.as_str().to_string(),
                )
            })
        })
        .collect())
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

/// 判定と、その判定が読んだ所（JSON Pointer。"" は文書全体）と、判定に使った rule の定義（x-prompt の write の1行）。
/// 読んだ所は当たる rule の所（と rule の reads）で、形の検査と組み込みの検査は文書全体を読む。
type Judged = (RuleResult, String, Option<Value>);

fn whole(r: RuleResult) -> Judged {
    (r, String::new(), None)
}

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
            // JSON Schema で書けない Concrete の決まり2つは、SAS が組み込みで確かめる
            results.extend(builtin_checks(&concrete).into_iter().map(whole));
            for rule in rules_for_concrete(&abstract_doc, vocabulary.as_ref(), &concrete) {
                results.push((
                    unjudged(&rule),
                    rule.pointer.clone(),
                    Some(rule.definition.clone()),
                ));
            }
        }
        TargetKind::Instance => {
            let concrete_doc = concretes
                .get(declared)
                .map(ConcreteSchema::document)
                .cloned()
                .unwrap_or_default();
            for rule in rules_for_instance(&concrete_doc, &d.value, more) {
                results.push((
                    unjudged(&rule),
                    rule.pointer.clone(),
                    Some(rule.definition.clone()),
                ));
            }
        }
    }
    Ok(results)
}

/// 判定が読んだ所のハッシュ。
#[must_use]
pub fn reads_of(document: &Value, at: &str) -> ContentHash {
    ContentHash::of_read(at, document.pointer(at))
}

/// rule の定義の reads（判定のときに一緒に読む所。このインスタンスの中の JSON Pointer）。
fn rule_reads(definition: Option<&Value>) -> Vec<String> {
    definition
        .and_then(|d| d.get("reads"))
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default()
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

/// 対象を決めて種類を見分け、いまの中身ですべての rule を判定する。記録しない（描画も使う）。
///
/// # Errors
/// 対象を読めない ・ 申告した Abstract の版が閉じた一覧に無い ・ 従う schema が分からない ・ バリデーターが動かないとき。
pub fn judge_current(
    scope: &Scope,
    store: &dyn DocumentStore,
    validator: &dyn SchemaValidator,
) -> Result<(Corpus, Vec<Current>), Refused> {
    let corpus = Corpus::load(store)?;
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
                // 当たる所の値に、rule の reads が指す所の値を足す（読む欄が変われば、前の判定は効かない）
                let mut reads = reads_of(&doc.value, &at);
                for p in rule_reads(definition.as_ref()) {
                    let more = reads_of(&doc.value, &p);
                    reads = ContentHash::of(
                        format!("{}\u{0}{}", reads.as_str(), more.as_str()).as_bytes(),
                    );
                }
                // rule の文も読んだものに入れる（Concrete の rule を書き換えたら、前の判定は効かない）
                let reads = match definition {
                    Some(def) => {
                        ContentHash::of(format!("{}\u{0}{def}", reads.as_str()).as_bytes())
                    }
                    None => reads,
                };
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
    log: &dyn OperationLog,
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
    who: &dyn IdentifyCaller,
    log: &mut dyn OperationLog,
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
        if let Some(expected) = untracked_since(log, &d.path, hash.as_str()).map_err(log_refused)?
            && let Some(last) = log.writes(&d.path).map_err(log_refused)?.pop()
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
        .map(|(path, _, _, hash, results, reads, _, _)| ValidateRow {
            target_path: path.clone(),
            hash: hash.as_str().to_string(),
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
        session: recorded.session,
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
        Caller::Session { .. } => Entry::Tool,
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
