//! Concrete とインスタンスの検証（業務領域「書いて検証する」の検証のモジュールの手続き）。
//!
//! - UC4 検証する：対象を読み、Concrete かインスタンスかを $schema で見分け、形（JSON Schema）・ 組み込みの検査 ・
//!   当たる rule を判定し、対象ごとに検証の操作を1件記録して、結果と記録の ID を返す。検証は対象を変えない。
//! - UC4 の範囲「入力が変わった rule だけ」と、SAS を通らない変更があった対象の知らせ。
//! - UC4 の意味の部分：判定の手順を打ったセッションに返し（記録しない）、そのセッションが渡した判定を機械の結果と合わせて記録する。

use crate::ports::{DocumentStore, SchemaValidator, ValidationLog};
use crate::use_cases::meaning::{
    ChangeOp, FieldEdit, Group, ReviewScope, Step, answer_json, check_answer, command_line,
    match_steps, procedure, related, revert, touched_rules,
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use validation_domain::services::{
    Judged, concrete_rules, instance_rules, reads_hash, related_fields, whole,
};
use validation_domain::values::{By, Counts, Judgment, RuleResult, RuleRow, ValidateRow, Verdict};
use writing_domain::aggregates::ConcreteSchema;
use writing_domain::services::{
    Resolved, SchemaIndex, Unresolved, concrete_of, field_schema, refs,
};
use writing_domain::values::{
    Caller, ContentHash, FieldRow, TargetKind, WriteRecord, unrecorded_since,
};

/// 何を検証するか。範囲は渡された経路だけで、既定の範囲は持たない。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scope {
    /// 検証する対象の経路
    pub paths: Vec<String>,
    /// スキーマの置き場所のファイル（インスタンスが従う Concrete と $ref の先を $id で引く）
    pub schemas: Vec<String>,
}

/// 渡された経路の文書1つ。
#[derive(Debug, Clone, PartialEq)]
pub struct Doc {
    /// 経路
    pub path: String,
    /// 読んだ文字列
    pub text: String,
    /// 読んだ JSON
    pub value: Value,
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
pub struct Unrecorded {
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
    pub unrecorded: Vec<Unrecorded>,
}

/// 操作を始められなかった ・ 書かなかった理由。断った操作は記録しない（理由 ・ 案内 ・ 間に入った変更をその場で返す）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Refused {
    /// 決まった名前（usage ・ schema_not_listed ・ not_found ・ not_json ・ no_schema ・ unknown_schema ・ duplicate_schema ・ store ・
    /// validator ・ log、
    /// 渡された判定が手順に合わないもの：invalid_answer、手順のハッシュがいまの中身と違うもの：conflict）
    pub code: &'static str,
    /// 対象の経路（あれば）
    pub path: Option<Box<str>>,
    /// 理由
    pub message: String,
    /// 次にすることの案内（判定の直し方 ・ 手順を取り直すコマンド）
    pub hint: Option<Box<String>>,
    /// 同じ $id を持つファイルの経路（duplicate_schema）
    pub paths: Option<Box<Vec<String>>>,
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

/// 置き場所から引けなかった理由を、断った理由にする。
fn unresolved(path: Option<&str>, u: Unresolved) -> Refused {
    Refused {
        code: u.code(),
        path: path.map(Into::into),
        message: u.message().to_string(),
        paths: (!u.paths().is_empty()).then(|| Box::new(u.paths().to_vec())),
        ..Refused::default()
    }
}

/// 渡されたスキーマの置き場所のファイルを1件ずつ読み、$id と経路だけを索引に残す。
///
/// # Errors
/// ファイルを読めないとき。
fn index_of(store: &dyn DocumentStore, schemas: &[String]) -> Result<SchemaIndex, Refused> {
    let mut index = SchemaIndex::new();
    for p in schemas {
        let text = store.read(p).map_err(|e| refused("store", p, e))?;
        index.add(p, &text);
    }
    Ok(index)
}

/// 索引から $id の文書を1つ読む。from は $ref で指した文書の $id。
///
/// # Errors
/// 置き場所に無い ・ 2つ以上ある ・ 読めない ・ JSON でないとき。
fn read_one(
    index: &SchemaIndex,
    store: &dyn DocumentStore,
    id: &str,
    from: Option<&str>,
    target: &str,
) -> Result<(String, Value), Refused> {
    let path = index
        .path_of(id, from)
        .map_err(|u| unresolved(Some(target), u))?
        .to_string();
    let text = store.read(&path).map_err(|e| refused("store", &path, e))?;
    let value = serde_json::from_str(&text)
        .map_err(|e| refused("store", &path, format!("JSON として読めない：{e}")))?;
    Ok((path, value))
}

/// インスタンスが従う Concrete（$id が declared）と、その $ref の先（同じ置き場所から引く。入れて出すスキーマは引かない）。
///
/// # Errors
/// 置き場所に無い ・ 2つ以上ある ・ Concrete でない ・ 読めないとき。
fn resolve(
    index: &SchemaIndex,
    store: &dyn DocumentStore,
    validator: &dyn SchemaValidator,
    target: &str,
    declared: &str,
) -> Result<Resolved, Refused> {
    let (at, value) = read_one(index, store, declared, None, target)?;
    let concrete = concrete_of(declared, &at, &value).map_err(|u| unresolved(Some(target), u))?;
    let mut seen: BTreeSet<String> = BTreeSet::from([declared.to_string()]);
    let mut queue: Vec<(String, String)> = refs(&value)
        .into_iter()
        .map(|r| (r, declared.to_string()))
        .collect();
    let mut more = vec![(declared.to_string(), value)];
    while let Some((next, from)) = queue.pop() {
        if validator.shipped(&next).is_some() || !seen.insert(next.clone()) {
            continue;
        }
        let (_, doc) = read_one(index, store, &next, Some(&from), target)?;
        queue.extend(refs(&doc).into_iter().map(|r| (r, next.clone())));
        more.push((next, doc));
    }
    Ok(Resolved::new(at, concrete, more))
}

/// $schema の申告から対象の種類を見分ける。見分けられなければ、取りに行かずに断る理由。
fn classify(path: &str, declared: &str) -> Result<TargetKind, Refused> {
    TargetKind::classify(declared).map_err(|u| refused(u.code(), path, u.message().to_string()))
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

/// UC4 検証する：形 ・ 組み込みの検査 ・ 当たる rule を判定し（rule は判定を渡されなければ not-judged）、対象ごとに検証の操作を記録する。
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
    run_validate(scope, Pick::All, store, validator, caller, log, None)
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
    run_validate(
        scope,
        Pick::ChangedRules,
        store,
        validator,
        caller,
        log,
        None,
    )
}

/// 意味の判定の既定の選び方（rule × 当たる所の単位で、変わった所だけ）で、判定を渡さずに記録する。
/// 判定の手順に入る所が無かった --semantic が使う。手順に入らない所は前の判定を引き継ぎ、検証の結果全体を記録する。
/// 前の判定から何も変わらず、消えた所も無い対象は記録しない。
///
/// # Errors
/// validate と同じ。
pub fn validate_changed(
    scope: &Scope,
    store: &dyn DocumentStore,
    validator: &dyn SchemaValidator,
    caller: &Caller,
    log: &mut dyn ValidationLog,
) -> Result<Validation, Refused> {
    run_validate(
        scope,
        Pick::ChangedPlaces,
        store,
        validator,
        caller,
        log,
        None,
    )
}

/// どの判定を、前の判定と比べて選ぶか。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pick {
    /// すべてを判定する（前の判定と比べない）
    All,
    /// --stale（機械の検証）：rule ごとに、読んだ所のハッシュの集まりが前の判定と違う rule だけを判定して記録する
    ChangedRules,
    /// 意味の判定の既定：前の判定のあとに記録された変更と重なる所 ・ 読む所 ・ 文が変わった rule ・ 前の判定に無い所だけを
    /// 手順に入れ、ほかの所は前の判定を引き継いで、検証の結果全体を記録する
    ChangedPlaces,
    /// 意味の判定の --all-rules：すべてを判定する。レビューのまとまりには前の判定のときの値と、そのあとの変更を入れる
    AllRules,
}

/// 意味の判定（--semantic ・ --answer）で、どの rule を手順に入れるか。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selection {
    /// --all-rules：前の判定にかかわらず、すべての rule を入れる。無ければ、前の判定から読んだ所が変わった rule と、
    /// 前に判定できなかった rule だけ（前の判定が無い対象はすべて）
    pub all_rules: bool,
    /// --stale を渡したか（渡すコマンドにも付ける）。選び方は all_rules で決まる
    pub stale: bool,
}

impl Selection {
    /// 前の判定と比べてどれを手順に入れるか。
    const fn pick(self) -> Pick {
        if self.all_rules {
            Pick::AllRules
        } else {
            Pick::ChangedPlaces
        }
    }
}

/// UC4 の意味の部分の1つ目：機械の検証をしたうえで、意味の rule（ai）の判定の手順を対象ごとに返す。記録しない。
/// 判定する rule が無い対象の手順は無い（どの対象にも無ければ空）。
/// 既定では、rule × 当たる所ごとに、前の判定から読んだ所（reads_hash）が変わった所 ・ 前に判定できなかった所 ・
/// 前の判定に無い所だけを手順に入れる（前の判定が無い対象はすべて）。--all-rules なら、すべての所を入れる。
///
/// # Errors
/// validate と同じ。
pub fn semantic_procedures(
    scope: &Scope,
    selection: Selection,
    store: &dyn DocumentStore,
    validator: &dyn SchemaValidator,
    caller: &Caller,
    log: &dyn ValidationLog,
) -> Result<Vec<Value>, Refused> {
    let (current, _) = current_for(scope, selection.pick(), store, validator, log)?;
    Ok(current
        .iter()
        .filter_map(|c| {
            let (groups, steps, _) = steps_of(c, validator);
            if steps.is_empty() {
                return None;
            }
            let command = answer_command(&c.doc.path, selection, caller);
            Some(procedure(
                &c.doc.path,
                &c.declared,
                c.hash.as_str(),
                (c.since.as_deref(), c.review),
                &groups,
                &steps,
                &command,
            ))
        })
        .collect())
}

/// UC4 の意味の部分の2つ目：打ったセッションが渡した判定（answer）を、形 ・ 手順のハッシュ ・ step ・ rule ・ pointer で
/// 確かめ、機械の結果と合わせて記録する。合わなければ記録せずに断る。
/// 手順を組んだときと同じ選び方（selection）で所を選ぶ。既定では手順に入らなかった所の前の判定を引き継ぎ、
/// 検証の結果全体を記録する。
///
/// # Errors
/// validate と同じ。ほかに、答えが形に合わない ・ 手順と突き合わない（invalid_answer）・ 手順のハッシュがいまの中身と違う（conflict）とき。
pub fn validate_with_answer(
    scope: &Scope,
    selection: Selection,
    answer: &str,
    store: &dyn DocumentStore,
    validator: &dyn SchemaValidator,
    caller: &Caller,
    log: &mut dyn ValidationLog,
) -> Result<Validation, Refused> {
    run_validate(
        scope,
        selection.pick(),
        store,
        validator,
        caller,
        log,
        Some((answer, selection)),
    )
}

/// 判定を渡すときに打つコマンド。--session で名乗ったセッションは、渡すときも名乗る。
fn answer_command(path: &str, selection: Selection, caller: &Caller) -> String {
    let mut args = vec!["semantic-agent-schema", "validate", path];
    args.extend(selection_args(selection));
    args.extend(["--answer", "<判定の JSON のファイル | ->"]);
    if let Some(id) = caller.flagged() {
        args.extend(["--session", id]);
    }
    command_line(&args)
}

/// 手順を取り直すコマンド。
fn procedure_command(path: &str, selection: Selection) -> String {
    let mut args = vec!["semantic-agent-schema", "validate", path];
    args.extend(selection_args(selection));
    args.push("--semantic");
    command_line(&args)
}

/// 選び方を、続けて打つコマンドの引数にする。
fn selection_args(selection: Selection) -> Vec<&'static str> {
    let mut args = Vec::new();
    if selection.stale {
        args.push("--stale");
    }
    if selection.all_rules {
        args.push("--all-rules");
    }
    args
}

/// 対象1つのレビューのまとまりと、判定する rule × 当たる所（意味の rule のうち、前の判定を引き継がないもの）と、
/// それが当たる results の位置。所のまとまりは当たる所ごとに1つ（results に出てきた順）、文書全体のまとまりはいつも1つで最後。
/// 判定するものが無ければ、どれも空。
fn steps_of(c: &Current, validator: &dyn SchemaValidator) -> (Vec<Group>, Vec<Step>, Vec<usize>) {
    let schema_doc = match (&c.kind, &c.concrete) {
        (TargetKind::Instance, Some(k)) => k.concrete().document().clone(),
        _ => validator.shipped(&c.declared).unwrap_or_default(),
    };
    let chosen: Vec<usize> = c
        .results
        .iter()
        .enumerate()
        .filter(|(i, (r, _))| r.by() == By::Ai && !c.kept[*i])
        .map(|(i, _)| i)
        .collect();
    if chosen.is_empty() {
        return (Vec::new(), Vec::new(), Vec::new());
    }
    let mut groups: Vec<Group> = Vec::new();
    for &i in &chosen {
        let at = c.results[i].0.pointer();
        if at.is_empty() || groups.iter().any(|g| g.pointer == at) {
            continue;
        }
        let now = c.doc.value.pointer(at).cloned().unwrap_or(Value::Null);
        // 前の判定のときの値。前の判定を使えない ・ そのあとに足した所は無い（null）。値が変わっていなければ current と同じ
        let before = c.before_doc.as_ref().and_then(|b| b.pointer(at)).cloned();
        // この所の値を変えた操作 ・ この所の rule が読む所を変えた操作 ・ この所の rule の文を変えた Concrete の操作
        let here: Vec<(&str, Vec<String>)> = chosen
            .iter()
            .filter(|&&k| c.results[k].0.pointer() == at)
            .map(|&k| {
                (
                    c.results[k].0.rule(),
                    related_fields(c.definitions[k].as_ref()),
                )
            })
            .collect();
        let changes: Vec<ChangeOp> = c
            .ops
            .iter()
            .filter(|o| {
                o.pointers.iter().any(|p| {
                    related(at, p)
                        || here
                            .iter()
                            .any(|(_, reads)| reads.iter().any(|q| related(q, p)))
                }) || here.iter().any(|(rule, _)| o.rules.contains(*rule))
            })
            .cloned()
            .collect();
        let field = field_schema(&schema_doc, at, c.more());
        let field = field.as_ref();
        groups.push(Group {
            no: groups.len() + 1,
            pointer: at.to_string(),
            before,
            changes,
            current: now,
            changed: Vec::new(),
            title: field
                .and_then(|f| f.get("title"))
                .and_then(Value::as_str)
                .map(ToString::to_string),
            description: field
                .and_then(|f| f.get("description"))
                .and_then(Value::as_str)
                .map(ToString::to_string),
            meaning: field
                .and_then(|f| f.pointer("/x-prompt/meaning"))
                .and_then(Value::as_str)
                .map(ToString::to_string),
            when: field
                .and_then(|f| f.pointer("/x-prompt/when"))
                .and_then(Value::as_str)
                .map(ToString::to_string),
        });
    }
    groups.push(Group {
        no: groups.len() + 1,
        pointer: String::new(),
        before: None,
        changes: c
            .ops
            .iter()
            .filter(|o| !o.pointers.is_empty())
            .cloned()
            .collect(),
        current: c.doc.value.clone(),
        changed: c.changed.clone(),
        title: None,
        description: None,
        meaning: None,
        when: None,
    });
    let mut steps = Vec::new();
    for &i in &chosen {
        let r = &c.results[i].0;
        let def = c.definitions[i].as_ref();
        let no = groups
            .iter()
            .find(|g| g.pointer == r.pointer())
            .map_or(groups.len(), |g| g.no);
        steps.push(Step {
            no,
            rule: r.rule().to_string(),
            pointer: r.pointer().to_string(),
            definition: def.cloned().unwrap_or_default(),
            related_fields: related_fields(def)
                .into_iter()
                .map(|p| {
                    let v = c.doc.value.pointer(&p).cloned().unwrap_or(Value::Null);
                    (p, v)
                })
                .collect(),
        });
    }
    (groups, steps, chosen)
}

/// 渡された判定で、対象の意味の rule の結果を置き換える。
fn apply_answer(
    current: &mut [Current],
    validator: &dyn SchemaValidator,
    answer: &str,
    selection: Selection,
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
            procedure_command(&path, selection)
        )));
        return Err(r);
    }
    let (_, steps, at) = steps_of(c, validator);
    let judged = match_steps(&v, &steps).map_err(invalid)?;
    for (n, (step, i)) in steps.iter().zip(at).enumerate() {
        if let Some((verdict, reason)) = judged.get(&n) {
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
    concrete: Option<&Resolved>,
) -> Result<Vec<Judged>, Refused> {
    let more = concrete.map_or(&[][..], Resolved::more);
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
            let concrete_doc = concrete
                .map(|c| c.concrete().document().clone())
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
    /// インスタンスが従う Concrete（渡された置き場所から引いたもの。Concrete の対象は None）
    pub concrete: Option<Resolved>,
    /// いまの中身のハッシュ
    pub hash: ContentHash,
    /// rule ごとの結果と、その判定が読んだ所のハッシュ
    pub results: Vec<(RuleResult, ContentHash)>,
    /// rule ごとの定義（x-prompt の rules の1件。results と同じ並び。機械だけの検査は None）
    pub definitions: Vec<Option<Value>>,
    /// results と同じ並びで、前の判定から変わっていない所か（意味の判定の既定で、手順に入れず前の判定を引き継ぐ）
    pub kept: Vec<bool>,
    /// 前の判定から何も変わっていない対象か（記録しない）
    pub same: bool,
    /// 意味の判定で、前の判定のときの文書（記録した変更を戻して組む）。前の判定を使えないときは None
    pub before_doc: Option<Value>,
    /// 意味の判定で、前の判定のあとに記録された変更の所（重ならず、所の順）
    pub changed: Vec<String>,
    /// 意味の判定で、前の判定の操作の ID（無ければ None）
    pub since: Option<String>,
    /// 意味の判定で、手順がどこまでを持つか
    pub review: ReviewScope,
    /// 意味の判定で、前の判定のあとに記録された変更の操作（対象を書いた操作と、文が変わった rule の Concrete の操作。古い順）
    pub ops: Vec<ChangeOp>,
}

impl Current {
    /// 形の検査と欄の定義で引ける文書（インスタンスなら Concrete と $ref の先）。
    fn more(&self) -> &[(String, Value)] {
        self.concrete.as_ref().map_or(&[], Resolved::more)
    }
}

/// 渡された経路の対象を読んで種類を見分け、いまの中身ですべての rule を判定する。記録しない。
///
/// # Errors
/// 対象を読めない ・ 申告した Abstract の版が閉じた一覧に無い ・ 従う schema が分からない ・ バリデーターが動かないとき。
pub fn judge_current(
    scope: &Scope,
    store: &dyn DocumentStore,
    validator: &dyn SchemaValidator,
) -> Result<Vec<Current>, Refused> {
    let mut index: Option<SchemaIndex> = None;
    let mut classified = Vec::new();
    for p in &scope.paths {
        let text = store.read(p).map_err(|e| refused("not_found", p, e))?;
        let value: Value = serde_json::from_str(&text)
            .map_err(|e| refused("not_json", p, format!("JSON として読めない：{e}")))?;
        let d = Doc {
            path: p.clone(),
            text,
            value,
        };
        let declared = d
            .value
            .get("$schema")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let kind = classify(&d.path, &declared)?;
        let concrete = match kind {
            TargetKind::Concrete => None,
            TargetKind::Instance => {
                if index.is_none() {
                    index = Some(index_of(store, &scope.schemas)?);
                }
                match &index {
                    Some(i) => Some(resolve(i, store, validator, &d.path, &declared)?),
                    None => None,
                }
            }
        };
        classified.push((d, kind, declared, concrete));
    }
    let mut current = Vec::new();
    for (doc, kind, declared, concrete) in classified {
        let judged = judge(&doc, kind, &declared, validator, concrete.as_ref())?;
        let definitions = judged.iter().map(|(_, _, d)| d.clone()).collect();
        let results = judged
            .into_iter()
            .map(|(r, at, definition)| {
                // 当たる所の値 ・ rule の related_fields が指す所の値 ・ rule の文（どれかが変われば、前の判定は効かない）
                let reads = reads_hash(&doc.value, &at, definition.as_ref());
                (r, reads)
            })
            .collect();
        let results: Vec<(RuleResult, ContentHash)> = results;
        current.push(Current {
            hash: ContentHash::of(doc.text.as_bytes()),
            kept: vec![false; results.len()],
            same: false,
            before_doc: None,
            changed: Vec::new(),
            since: None,
            review: ReviewScope::First,
            ops: Vec::new(),
            doc,
            kind,
            declared,
            concrete,
            results,
            definitions,
        });
    }
    Ok(current)
}

/// 前の判定のあと、読んだ所が変わった rule の id（--stale）。前の判定が無い rule も入る。
/// いまの判定と前の判定で、rule ごとに「読んだ所のハッシュ」の集まりを比べる。
fn changed_rules(now: &[(RuleResult, ContentHash)], before: &[Judgment]) -> BTreeSet<String> {
    let mut current: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();
    for (r, reads) in now {
        current
            .entry(r.rule())
            .or_default()
            .insert(reads.as_str().to_string());
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

/// 経路 → 入力が変わった rule だけを検証したときに、判定し直さなかった rule の id。
type Unchanged = BTreeMap<String, Vec<String>>;

/// 記録した判定の区分の名前を、区分に戻す。知らない名前は unverified（verified に数えない）。
fn verdict_of(name: &str) -> Verdict {
    Verdict::parse(name).unwrap_or(Verdict::Unverified)
}

/// 操作の ID（o-0001）の番号。
fn op_no(id: &str) -> u64 {
    id.strip_prefix("o-")
        .and_then(|n| n.parse().ok())
        .unwrap_or_default()
}

/// 記録した変わった欄を、値を読んだ形にする。
fn edits(fields: &[FieldRow]) -> Vec<FieldEdit> {
    let parse = |s: &Option<String>| s.as_deref().and_then(|t| serde_json::from_str(t).ok());
    fields
        .iter()
        .map(|f| FieldEdit {
            pointer: f.pointer.clone(),
            change: f.change.clone(),
            before: parse(&f.before),
            after: parse(&f.after),
        })
        .collect()
}

/// 対象の前の判定と、そのあとに記録された変更（意味の判定の手順のレビューのまとまりに入れる）。
/// 前の判定は、by が ai で判定できた（verified ・ violated ・ unverified）行を持つ最も新しい検証。
/// 前の判定が無い対象と、前の判定のあとの中身が記録で説明できない対象（取り込んでいない変更がある）は first で、
/// 前の判定を使わない。concrete はインスタンスが従う Concrete のファイル（文が変わった rule を引く）と、その書いた操作。
fn review_of(
    c: &mut Current,
    before: &[Judgment],
    writes: &[WriteRecord],
    concrete: Option<(&Doc, &[WriteRecord])>,
) {
    c.review = ReviewScope::First;
    let judged = |j: &&Judgment| {
        j.rule.by == "ai" && Verdict::parse(&j.rule.result).is_some_and(Verdict::judged)
    };
    let Some(base) = before
        .iter()
        .filter(judged)
        .max_by_key(|j| op_no(&j.operation))
    else {
        return;
    };
    let base_no = op_no(&base.operation);
    let after_base: Vec<&WriteRecord> = writes
        .iter()
        .filter(|w| op_no(&w.operation) > base_no)
        .collect();
    let explained = after_base
        .last()
        .map_or(Some(base.hash.as_str()), |w| w.row.after.as_deref());
    if explained != Some(c.hash.as_str()) {
        return;
    }
    let edits_after: Vec<Vec<FieldEdit>> =
        after_base.iter().map(|w| edits(&w.row.fields)).collect();
    let mut changed: Vec<String> = edits_after
        .iter()
        .flatten()
        .map(|e| e.pointer.clone())
        .collect();
    changed.sort();
    changed.dedup();
    let mut ops: Vec<ChangeOp> = after_base
        .iter()
        .zip(&edits_after)
        .map(|(w, e)| ChangeOp {
            operation: w.operation.clone(),
            session: w.session.clone(),
            target: w.row.target_path.clone(),
            reason: w.row.reason.clone(),
            pointers: e.iter().map(|e| e.pointer.clone()).collect(),
            rules: BTreeSet::new(),
        })
        .collect();
    if let Some((doc, concrete_writes)) = concrete {
        for w in concrete_writes
            .iter()
            .filter(|w| op_no(&w.operation) > base_no)
        {
            let rules = touched_rules(&doc.value, &edits(&w.row.fields));
            if !rules.is_empty() {
                ops.push(ChangeOp {
                    operation: w.operation.clone(),
                    session: w.session.clone(),
                    target: w.row.target_path.clone(),
                    reason: w.row.reason.clone(),
                    pointers: Vec::new(),
                    rules,
                });
            }
        }
    }
    ops.sort_by_key(|o| op_no(&o.operation));
    c.review = ReviewScope::Changed;
    c.since = Some(base.operation.clone());
    c.before_doc = Some(revert(&c.doc.value, &edits_after));
    c.changed = changed;
    c.ops = ops;
}

/// 意味の判定の既定の選び方：前の判定のあとに記録された変更から、判定するもの（引き継がないもの）を選ぶ。選ぶのは、意味の rule × 当たる所のうち
/// (1) 当たる所が変わった所と重なる（同じ ・ その中 ・ その外側。文書全体に当たる rule は、どこかが変われば入る）
/// (2) rule の related_fields が変わった所と重なる (3) rule の文が前の判定のあとに変わった (4) 前の判定に無い ・ 前に判定できなかった。
/// ほかは前の判定を引き継ぐ。前の判定を使えない対象（first）はすべてを選ぶ。
fn select_changed(c: &mut Current, before: &[Judgment]) {
    if c.review != ReviewScope::Changed {
        return;
    }
    let rules_changed: BTreeSet<String> =
        c.ops.iter().flat_map(|o| o.rules.iter().cloned()).collect();
    let mut previous: BTreeMap<(&str, &str), &RuleRow> = BTreeMap::new();
    for j in before {
        previous
            .entry((j.rule.rule.as_str(), j.rule.pointer.as_str()))
            .or_insert(&j.rule);
    }
    for (i, ((r, _), def)) in c.results.iter_mut().zip(&c.definitions).enumerate() {
        if r.by() != By::Ai {
            continue;
        }
        let Some(row) = previous.get(&(r.rule(), r.pointer())) else {
            continue;
        };
        if !Verdict::parse(&row.result).is_some_and(Verdict::judged) {
            continue;
        }
        let hit = c.changed.iter().any(|p| {
            related(r.pointer(), p) || related_fields(def.as_ref()).iter().any(|q| related(q, p))
        });
        if hit || rules_changed.contains(r.rule()) {
            continue;
        }
        c.kept[i] = true;
        *r = RuleResult::new(
            &row.rule,
            &row.pointer,
            By::Ai,
            verdict_of(&row.result),
            &row.reason,
        );
    }
    // 機械の検査は毎回いまの中身で判定する。何も変わらず、消えた所も無ければ、記録しない
    let places_now: BTreeSet<(&str, &str)> = c
        .results
        .iter()
        .map(|(r, _)| (r.rule(), r.pointer()))
        .collect();
    let places_before: BTreeSet<(&str, &str)> = previous.keys().copied().collect();
    let ai_kept = c
        .results
        .iter()
        .zip(&c.kept)
        .all(|((r, _), k)| r.by() != By::Ai || *k);
    c.same = c.ops.is_empty() && ai_kept && places_now == places_before;
}

/// いまの中身で判定し、前の判定と比べて選ぶ。経路 → 判定し直さなかった rule の id も返す。
/// - ChangedRules（--stale）：読んだ所が前の判定と同じ rule を除く
/// - ChangedPlaces（意味の判定の既定）：何も除かず、変わっていない所に印を付けて、意味の rule の所は前の判定を引き継ぐ
fn current_for(
    scope: &Scope,
    pick: Pick,
    store: &dyn DocumentStore,
    validator: &dyn SchemaValidator,
    log: &dyn ValidationLog,
) -> Result<(Vec<Current>, Unchanged), Refused> {
    let mut current = judge_current(scope, store, validator)?;
    let mut unchanged = Unchanged::new();
    for c in &mut current {
        match pick {
            Pick::All => {}
            Pick::ChangedRules => {
                let before = log.last_judgments(&c.doc.path).map_err(log_refused)?;
                let changed = changed_rules(&c.results, &before);
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
                c.kept = vec![false; c.results.len()];
                c.same = c.results.is_empty();
            }
            Pick::ChangedPlaces | Pick::AllRules => {
                let before = log.last_judgments(&c.doc.path).map_err(log_refused)?;
                let writes = log.writes(&c.doc.path).map_err(log_refused)?;
                // インスタンスでは、従う Concrete のファイルに前の判定のあと記録された変更から、文が変わった rule を引く
                let concrete = c.concrete.as_ref().map(|k| Doc {
                    path: k.path().to_string(),
                    text: String::new(),
                    value: k.concrete().document().clone(),
                });
                let concrete_writes = match &concrete {
                    Some(d) => log.writes(&d.path).map_err(log_refused)?,
                    None => Vec::new(),
                };
                review_of(
                    c,
                    &before,
                    &writes,
                    concrete.as_ref().map(|d| (d, concrete_writes.as_slice())),
                );
                if pick == Pick::AllRules {
                    if c.review == ReviewScope::Changed {
                        c.review = ReviewScope::AllRules;
                    }
                    continue;
                }
                select_changed(c, &before);
                // どの所も前の判定を引き継いだ rule
                let rules = unchanged.entry(c.doc.path.clone()).or_default();
                for (r, _) in &c.results {
                    let all_kept = c
                        .results
                        .iter()
                        .zip(&c.kept)
                        .filter(|((o, _), _)| o.rule() == r.rule())
                        .all(|(_, k)| *k);
                    if all_kept && !rules.iter().any(|u| u == r.rule()) {
                        rules.push(r.rule().to_string());
                    }
                }
            }
        }
    }
    Ok((current, unchanged))
}

fn run_validate(
    scope: &Scope,
    pick: Pick,
    store: &dyn DocumentStore,
    validator: &dyn SchemaValidator,
    caller: &Caller,
    log: &mut dyn ValidationLog,
    answer: Option<(&str, Selection)>,
) -> Result<Validation, Refused> {
    let stale_only = matches!(pick, Pick::ChangedRules | Pick::ChangedPlaces);
    let (mut current, mut unchanged) = current_for(scope, pick, store, validator, log)?;
    if let Some((answer, selection)) = answer {
        apply_answer(&mut current, validator, answer, selection)?;
    }
    // 入力が変わった rule だけのときは、読んだ所が前の判定と同じ rule を除く
    let mut judged = Vec::new();
    let mut unrecorded = Vec::new();
    let mut unchanged_targets = Vec::new();
    for Current {
        doc: d,
        kind,
        declared,
        hash,
        results,
        same,
        ..
    } in current
    {
        // 外の変更（Semantic Agent Schema を通らない変更）：いまの中身が、このワークツリーで記録した最後の事後と違う（まだ書いていなければ、記録のどの中身とも違う）。
        // 知らせて、いまの中身を検証する
        let writes = log.writes(&d.path).map_err(log_refused)?;
        if let Some(last) = unrecorded_since(&writes, hash.as_str())
            && let Some(expected) = last.row.after.clone()
        {
            unrecorded.push(Unrecorded {
                path: d.path.clone(),
                last_operation: last.operation.clone(),
                expected,
                actual: hash.as_str().to_string(),
            });
        }
        let unchanged_rules = unchanged.remove(&d.path).unwrap_or_default();
        if same {
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
        unrecorded,
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
