//! UC4 検証する（機械の rule）。対象は Concrete とインスタンスで、1件ずつか全件。
//!
//! 手順：対象を読み、Concrete かインスタンスかを $schema で見分け、形（JSON Schema）・ 当たる rule ・ 参照の先を判定し、
//! 対象ごとに検証の操作を1件記録して、結果と記録の ID を返す。検証は対象を変えない。

use crate::ports::{DocumentStore, SchemaChecker};
use contracts_and_instances_conformance_domain::aggregates::{
    ABSTRACT_VERSIONS, ConcreteSchema, Instance, extension_names,
};
use contracts_and_instances_conformance_domain::services::{
    check_relationships, check_targets, judge_others, rules_for_concrete, rules_for_instance,
};
use contracts_and_instances_conformance_domain::values::{
    By, Counts, Fingerprint, RuleResult, Verdict,
};
use contracts_and_instances_operation_recording_application::ports::{
    IdentifyCaller, OperationLog, RuleRow, ValidateRow,
};
use contracts_and_instances_operation_recording_application::use_cases::history::record_validations;
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
}

/// 検証の結果と、記録の ID。
#[derive(Debug, Clone, PartialEq)]
pub struct Validation {
    /// 呼び出し元のセッションの ID
    pub session: String,
    /// 対象ごとの結果（経路の順）
    pub targets: Vec<TargetResult>,
    /// 全体の件数
    pub counts: Counts,
}

/// 検証を始められなかった理由。何も記録しない。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refused {
    /// 決まった名前（schema_not_listed ・ not_found ・ not_json ・ no_schema ・ unknown_schema ・ checker ・ log）
    pub code: &'static str,
    /// 対象の経路（あれば）
    pub path: Option<String>,
    /// 理由
    pub message: String,
}

fn refused(code: &'static str, path: &str, message: String) -> Refused {
    Refused {
        code,
        path: Some(path.to_string()),
        message,
    }
}

struct Doc {
    path: String,
    text: String,
    value: Value,
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
    // リポジトリの文書を読む（Concrete の一覧と、参照の先のインスタンスに使う）
    let mut corpus: Vec<Doc> = Vec::new();
    for path in store.list().map_err(|e| Refused {
        code: "store",
        path: None,
        message: e,
    })? {
        if let Ok(text) = store.read(&path)
            && let Ok(value) = serde_json::from_str::<Value>(&text)
        {
            corpus.push(Doc { path, text, value });
        }
    }
    let concretes: BTreeMap<String, ConcreteSchema> = corpus
        .iter()
        .filter_map(|d| ConcreteSchema::read(&d.value))
        .filter_map(|c| Some((c.id()?.to_string(), c)))
        .collect();
    let more: Vec<(String, Value)> = concretes
        .iter()
        .map(|(id, c)| (id.clone(), c.document().clone()))
        .collect();
    let instances: BTreeSet<(String, String)> = corpus
        .iter()
        .filter_map(|d| Instance::read(&d.value))
        .filter(|i| concretes.contains_key(i.concrete()))
        .filter_map(|i| Some((i.concrete().to_string(), i.id()?.to_string())))
        .collect();

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
            .into_iter()
            .filter(|d| {
                ConcreteSchema::declares_abstract(&d.value)
                    || d.value
                        .get("$schema")
                        .and_then(Value::as_str)
                        .is_some_and(|s| concretes.contains_key(s))
            })
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
        let kind = if ConcreteSchema::read(&d.value).is_some() {
            TargetKind::Concrete
        } else if ConcreteSchema::declares_abstract(&d.value) {
            return Err(refused(
                "schema_not_listed",
                &d.path,
                format!(
                    "{declared} は SAS が入れて出す Abstract の版（{}）に無い。取りに行かない",
                    ABSTRACT_VERSIONS.join(", ")
                ),
            ));
        } else if declared.is_empty() {
            return Err(refused("no_schema", &d.path, "$schema が無い".to_string()));
        } else if concretes.contains_key(&declared) {
            TargetKind::Instance
        } else {
            return Err(refused(
                "unknown_schema",
                &d.path,
                format!(
                    "{declared} はリポジトリの Concrete にも Abstract の版にも無い。取りに行かない"
                ),
            ));
        };
        classified.push((d, kind, declared));
    }

    // 判定する
    let mut known_concretes: BTreeSet<String> = concretes.keys().cloned().collect();
    known_concretes.extend(ABSTRACT_VERSIONS.iter().map(ToString::to_string));
    let mut judged = Vec::new();
    for (d, kind, declared) in classified {
        let mut results = Vec::new();
        let form_against: Vec<&str> = match kind {
            TargetKind::Concrete => vec![declared.as_str(), DIALECT],
            TargetKind::Instance => vec![declared.as_str()],
        };
        let mut broken: Vec<(String, String)> = Vec::new();
        for schema in form_against {
            for v in checker
                .check(schema, &d.value, &more)
                .map_err(|e| refused("checker", &d.path, e))?
            {
                if !broken.contains(&v) {
                    broken.push(v);
                }
            }
        }
        if broken.is_empty() {
            results.push(RuleResult::new(
                "json-schema",
                "",
                By::Machine,
                Verdict::Verified,
                "形が従う",
            ));
        }
        for (at, why) in broken {
            results.push(RuleResult::new(
                "json-schema",
                &at,
                By::Machine,
                Verdict::Violated,
                &why,
            ));
        }
        match kind {
            TargetKind::Concrete => {
                let abstract_doc = checker.shipped(&declared).ok_or_else(|| {
                    refused("checker", &d.path, format!("{declared} を検査器が持たない"))
                })?;
                let names = extension_names(&abstract_doc);
                let concrete = ConcreteSchema::read(&d.value).ok_or_else(|| {
                    refused("checker", &d.path, "Concrete として読めない".to_string())
                })?;
                for rule in rules_for_concrete(&abstract_doc, &concrete) {
                    if let Some(rs) = concrete.judge(&rule.id, &rule.pointer, &names) {
                        results.extend(rs);
                    } else if rule.id == "target-exists" {
                        let rs = check_targets(concrete.document(), &known_concretes);
                        if rs.is_empty() {
                            results.push(RuleResult::new(
                                &rule.id,
                                &rule.pointer,
                                By::Machine,
                                Verdict::Verified,
                                "x-relationship の to が無い",
                            ));
                        }
                        results.extend(rs);
                    } else {
                        results.push(judge_others(&rule));
                    }
                }
            }
            TargetKind::Instance => {
                let concrete_doc = concretes
                    .get(&declared)
                    .map(ConcreteSchema::document)
                    .cloned()
                    .unwrap_or_default();
                for rule in rules_for_instance(&concrete_doc, &d.value) {
                    results.push(judge_others(&rule));
                }
                results.extend(check_relationships(&concrete_doc, &d.value, &instances));
            }
        }
        let counts = Counts::of(&results);
        judged.push((
            d.path,
            kind,
            declared,
            Fingerprint::of(d.text.as_bytes()),
            results,
            counts,
        ));
    }

    // 対象ごとに検証の操作を記録する（前後の指紋は同じ）
    let rows: Vec<ValidateRow> = judged
        .iter()
        .map(|(path, kind, _, fp, results, _)| ValidateRow {
            target_kind: kind.as_str().to_string(),
            target_path: path.clone(),
            before: fp.as_str().to_string(),
            after: fp.as_str().to_string(),
            results: results
                .iter()
                .map(|r| RuleRow {
                    rule: r.rule().to_string(),
                    pointer: r.pointer().to_string(),
                    by: r.by().as_str().to_string(),
                    result: r.verdict().as_str().to_string(),
                    reason: r.reason().to_string(),
                })
                .collect(),
        })
        .collect();
    let recorded = record_validations(who, log, &rows).map_err(|e| Refused {
        code: "log",
        path: None,
        message: e,
    })?;
    let mut total = Counts::default();
    let targets = judged
        .into_iter()
        .zip(recorded.operations)
        .map(
            |((path, kind, schema, fingerprint, results, counts), operation)| {
                total.merge(&counts);
                TargetResult {
                    path,
                    kind,
                    schema,
                    fingerprint,
                    results,
                    counts,
                    operation,
                }
            },
        )
        .collect();
    Ok(Validation {
        session: recorded.session,
        targets,
        counts: total,
    })
}
