//! デザインシステムを、受け取りの型とテンプレートの書き方の契約（template-language.json）に照らす。
//!
//! インスタンスを Concrete（JSON Schema）で検証するのと同じく、テンプレートとトークンも契約で機械に確かめる。
//! テンプレートの書き方は Mustache（ロジックを持たない）。UI ・ UX のロジックはテンプレートの JS が持ち、Semantic Agent Schema は持たない。
//! 描画は、ここで1つも外れが無いデザインシステムでだけ描く。semantic-agent-schema validate でも同じ rule を確かめる（記録しない）。
//! テンプレートに渡す名前と値の入口は、契約の x-names ・ x-data-entry から読む。
//!
//! rule：
//! - json-schema：デザインシステムのファイルが受け取りの型に合う
//! - token-type：どのトークンも種類（$type）が決まり、値が種類の形に合う（Design Tokens Format Module 2025.10）
//! - token-alias：トークンの別名がすべて解け、輪になっていない
//! - template-syntax：どのテンプレートも Mustache として読める
//! - template-loads：部分テンプレートの先が templates の種類にある
//! - template-names：Concrete の根の x-view がそのテンプレートを選ぶとき、テンプレート（部分を含む）が置く名前が、
//!   契約の x-names（value はその Concrete の形、tokens はこのデザインシステムのトークン）のどれかにたどれる
//! - script-entry：値の入口は契約の x-data-entry の要素（<script type="application/json" id="semantic-agent-schema-data">{{{json}}}</script>）だけで、
//!   JS の中に Mustache のタグを書かない。JS として読める
//! - script-reads：Concrete の根の x-view がそのテンプレートを選ぶとき、テンプレート（部分を含む）の JS が入口から読む
//!   欄が、その Concrete の形にたどれる（読む所が実行するまで決まらないものは unverified）

use crate::ports::{DataRead, ScriptReader, TemplateNode, TemplateRenderer};
use crate::ports::{DocumentStore, SchemaValidator};
use crate::use_cases::render::{Refused, index_of, read_concrete, refused};
use rendering_domain::services::template_names::{
    PathCheck, Shape, check_path, partial_names, path_text, pointer_escape, script_blocks,
    walk_names,
};
use rendering_domain::services::tokens::{Token, alias_errors, flatten, type_errors};
use rendering_domain::services::view::view_kind;
use rendering_domain::values::TemplateLanguage;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use validation_domain::values::{By, Counts, RuleResult, Verdict};
use writing_domain::values::ContentHash;

/// デザインシステムの受け取りの型。
pub const DESIGN_SYSTEM: &str = "urn:semantic-agent-schema:published-language:design-system:v1";

/// テンプレートの書き方の契約。
pub const TEMPLATE_LANGUAGE: &str =
    "urn:semantic-agent-schema:published-language:template-language:v1";

/// 読み込んで確かめたデザインシステム。
pub struct DesignSystem {
    /// ファイルの経路
    pub path: String,
    /// 読んだ文字列
    pub text: String,
    /// 読んだ JSON
    pub value: Value,
    /// 種類 → テンプレートの経路（Git の根から）
    pub template_paths: BTreeMap<String, String>,
    /// 種類 → テンプレートの本文
    pub sources: Vec<(String, String)>,
    /// トークンの名前（group.token）→ トークン
    pub tokens: BTreeMap<String, Token>,
    /// テンプレートの書き方の契約のうち、描画が使う所（テンプレートに渡す名前と値の入口）
    pub language: TemplateLanguage,
    /// rule ごとの結果
    pub results: Vec<RuleResult>,
}

impl DesignSystem {
    /// 外れた rule があるか。
    #[must_use]
    pub fn violated(&self) -> bool {
        self.results
            .iter()
            .any(|r| r.verdict() == Verdict::Violated)
    }
}

fn verified(rule: &str, pointer: &str, why: &str) -> RuleResult {
    RuleResult::new(rule, pointer, By::Machine, Verdict::Verified, why)
}

fn violated(rule: &str, pointer: &str, why: &str) -> RuleResult {
    RuleResult::new(rule, pointer, By::Machine, Verdict::Violated, why)
}

fn unverified(rule: &str, pointer: &str, why: &str) -> RuleResult {
    RuleResult::new(rule, pointer, By::Machine, Verdict::Unverified, why)
}

/// 入れて出すテンプレートの書き方の契約から、描画が使う所を読む。
///
/// # Errors
/// 契約が無い ・ x-names か x-data-entry が読めないとき（作り方の誤り）。
pub fn template_language(validator: &dyn SchemaValidator) -> Result<TemplateLanguage, Refused> {
    validator
        .shipped(TEMPLATE_LANGUAGE)
        .as_ref()
        .and_then(TemplateLanguage::read)
        .ok_or_else(|| Refused {
            code: "validator",
            message: format!("{TEMPLATE_LANGUAGE} の x-names か x-data-entry が読めない"),
            ..Refused::default()
        })
}
/// デザインシステムを読み、契約に照らす。読めない ・ JSON でないときだけ断り、外れは結果として返す。
///
/// # Errors
/// ファイルが無い（not_found）・ JSON でない（invalid_input）・ バリデーターが動かない（validator）とき。
pub fn check_design_system(
    path: &str,
    store: &dyn DocumentStore,
    validator: &dyn SchemaValidator,
    renderer: &dyn TemplateRenderer,
    scripts: &dyn ScriptReader,
    concretes: &[(String, Value)],
) -> Result<DesignSystem, Refused> {
    let text = store
        .read(path)
        .map_err(|e| refused("not_found", path, e))?;
    let value: Value = serde_json::from_str(&text).map_err(|e| {
        refused(
            "invalid_input",
            path,
            format!("デザインシステムが JSON として読めない：{e}"),
        )
    })?;
    let language = template_language(validator)?;
    let entry = &language.entry;
    let mut results = Vec::new();

    // 型
    let broken = validator
        .check(DESIGN_SYSTEM, &value, &[])
        .map_err(|e| refused("validator", path, e))?;
    if broken.is_empty() {
        results.push(verified("json-schema", "", "受け取りの型に合う"));
    }
    for (at, why) in &broken {
        results.push(violated("json-schema", at, why));
    }

    // トークン
    let tokens = flatten(value.get("tokens"));
    let types = type_errors(&tokens);
    if types.is_empty() {
        results.push(verified(
            "token-type",
            "/tokens",
            "どのトークンも種類が決まり、値が種類の形（Design Tokens Format Module 2025.10）に合う",
        ));
    }
    for (name, why) in types {
        results.push(violated(
            "token-type",
            &format!("/tokens/{}", name.replace('.', "/")),
            &why,
        ));
    }
    let alias_errors = alias_errors(&tokens);
    if alias_errors.is_empty() {
        results.push(verified(
            "token-alias",
            "/tokens",
            "別名がすべて解け、輪になっていない",
        ));
    }
    for (name, why) in alias_errors {
        results.push(violated(
            "token-alias",
            &format!("/tokens/{}", name.replace('.', "/")),
            &why,
        ));
    }

    // テンプレートを読む
    let templates: BTreeMap<String, String> = value
        .get("templates")
        .and_then(Value::as_object)
        .map(|m| {
            m.iter()
                .map(|(k, v)| (k.clone(), v.as_str().unwrap_or_default().to_string()))
                .collect()
        })
        .unwrap_or_default();
    let base = path.rsplit_once('/').map_or("", |(dir, _)| dir);
    let mut template_paths = BTreeMap::new();
    let mut sources = Vec::new();
    let mut trees: BTreeMap<String, Vec<TemplateNode>> = BTreeMap::new();
    for (kind, rel) in &templates {
        let at = format!("/templates/{}", pointer_escape(kind));
        if rel.starts_with('/') || rel.split('/').any(|p| p == "..") {
            results.push(violated(
                "template-syntax",
                &at,
                &format!("{rel} はデザインシステムのフォルダの中の相対の経路にする"),
            ));
            continue;
        }
        let tpath = if base.is_empty() {
            rel.clone()
        } else {
            format!("{base}/{rel}")
        };
        let source = match store.read(&tpath) {
            Ok(s) => s,
            Err(e) => {
                results.push(violated(
                    "template-syntax",
                    &at,
                    &format!("{tpath} が無い：{e}"),
                ));
                continue;
            }
        };
        match renderer.inspect(&source) {
            Ok(tree) => {
                results.push(verified("template-syntax", &at, "Mustache として読める"));
                trees.insert(kind.clone(), tree);
            }
            Err(e) => results.push(violated("template-syntax", &at, &format!("{tpath}：{e}"))),
        }
        template_paths.insert(kind.clone(), tpath);
        sources.push((kind.clone(), source));
    }

    // 部分テンプレートの先
    for (kind, tree) in &trees {
        let at = format!("/templates/{}", pointer_escape(kind));
        let mut used = BTreeSet::new();
        partial_names(tree, &mut used);
        let bad: Vec<String> = used
            .into_iter()
            .filter(|p| !templates.contains_key(p))
            .collect();
        if bad.is_empty() {
            results.push(verified(
                "template-loads",
                &at,
                "部分テンプレートの先は templates の種類",
            ));
        } else {
            results.push(violated(
                "template-loads",
                &at,
                &format!("templates に無い部分テンプレート：{}", bad.join(" ・ ")),
            ));
        }
    }

    // JS：値の入口と、JS の中のタグ。JS が入口から読む所は種類ごとに集める
    let mut script_reads: BTreeMap<String, Vec<DataRead>> = BTreeMap::new();
    for (kind, source) in &sources {
        if !trees.contains_key(kind) {
            continue;
        }
        let at = format!("/templates/{}", pointer_escape(kind));
        let mut problems = Vec::new();
        let mut reads = Vec::new();
        for s in script_blocks(source, entry) {
            if s.external {
                continue;
            }
            if s.data_entry {
                if s.body.trim() != entry.content {
                    problems.push(format!(
                        "値の入口（id=\"{}\"）の中身は {} だけにする",
                        entry.id, entry.content
                    ));
                }
                continue;
            }
            if s.json_like {
                continue;
            }
            if s.body.contains("{{") {
                problems.push(format!(
                    "JS の中に Mustache のタグがある。値は {} から JSON.parse で読む",
                    entry.id
                ));
                continue;
            }
            match scripts.data_reads(&s.body, &entry.id) {
                Ok(f) => reads.extend(f),
                Err(e) => problems.push(e),
            }
        }
        if problems.is_empty() {
            results.push(verified(
                "script-entry",
                &at,
                &format!(
                    "値は {} から読み、JS にタグが無く、JS として読める",
                    entry.id
                ),
            ));
        } else {
            results.push(violated("script-entry", &at, &problems.join(" ／ ")));
        }
        script_reads.insert(kind.clone(), reads);
    }

    // 置く名前：根の x-view がこのデザインシステムの種類を選ぶ Concrete ごとに
    let token_shape = Shape::of_tokens(value.get("tokens"));
    let mut reached: BTreeSet<String> = BTreeSet::new();
    for (id, doc) in concretes {
        let kind = view_kind(doc);
        let Some(tree) = trees.get(kind) else {
            continue;
        };
        reached.insert(kind.to_string());
        let mut ps = BTreeSet::new();
        partial_names(tree, &mut ps);
        reached.extend(ps);
        // 契約の名前ごとの形：value は Concrete の形、tokens はトークンの木、ほかは文字列
        let root = Shape::Object(
            language
                .names
                .iter()
                .map(|n| {
                    let shape = match n.as_str() {
                        "value" => Shape::of_schema(doc),
                        "tokens" => token_shape.clone(),
                        _ => Shape::Scalar,
                    };
                    (n.clone(), shape)
                })
                .collect(),
        );
        let (mut missing, mut unknown) = (BTreeSet::new(), BTreeSet::new());
        walk_names(
            tree,
            &mut vec![root],
            &trees,
            &mut vec![kind.to_string()],
            &mut missing,
            &mut unknown,
        );
        let at = format!("/templates/{}", pointer_escape(kind));
        if !missing.is_empty() {
            results.push(violated(
                "template-names",
                &at,
                &format!(
                    "{id} で置けない名前：{}（置けるのは {} と、value の下の Concrete の欄）",
                    missing.into_iter().collect::<Vec<_>>().join(" ・ "),
                    language.names.join(" ・ ")
                ),
            ));
        } else if !unknown.is_empty() {
            results.push(unverified(
                "template-names",
                &at,
                &format!(
                    "{id} の形が決まらない所（$ref など）をたどるので確かめられない：{}",
                    unknown.into_iter().collect::<Vec<_>>().join(" ・ ")
                ),
            ));
        } else {
            results.push(verified(
                "template-names",
                &at,
                &format!("{id} の欄と契約の名前だけを置く"),
            ));
        }
        // JS が入口から読む所（部分テンプレートの JS も含む）
        let mut kinds: Vec<String> = vec![kind.to_string()];
        let mut ps2 = BTreeSet::new();
        partial_names(tree, &mut ps2);
        kinds.extend(ps2);
        let value_shape = Shape::of_schema(doc);
        let (mut bad, mut unsure) = (BTreeSet::new(), BTreeSet::new());
        for k in &kinds {
            for r in script_reads.get(k).into_iter().flatten() {
                match r {
                    DataRead::Untraceable(why) => {
                        unsure.insert(why.clone());
                    }
                    DataRead::Path(segs) => match check_path(&value_shape, segs) {
                        PathCheck::Ok => {}
                        PathCheck::Missing => {
                            bad.insert(path_text(segs));
                        }
                        PathCheck::Unknown => {
                            unsure
                                .insert(format!("{}（形が決まらない所をたどる）", path_text(segs)));
                        }
                    },
                }
            }
        }
        if !bad.is_empty() {
            results.push(violated(
                "script-reads",
                &at,
                &format!(
                    "JS が {id} に無い欄を読む：{}",
                    bad.into_iter().collect::<Vec<_>>().join(" ・ ")
                ),
            ));
        } else if !unsure.is_empty() {
            results.push(unverified(
                "script-reads",
                &at,
                &format!(
                    "JS が読む所を確かめられない：{}",
                    unsure.into_iter().collect::<Vec<_>>().join(" ・ ")
                ),
            ));
        } else {
            results.push(verified(
                "script-reads",
                &at,
                &format!("JS は {id} の欄だけを読む"),
            ));
        }
    }
    for kind in trees.keys().filter(|k| !reached.contains(*k)) {
        results.push(unverified(
            "template-names",
            &format!("/templates/{}", pointer_escape(kind)),
            "この種類を選ぶ Concrete もそれを読み込むテンプレートも無いので、置く名前を確かめられない",
        ));
    }

    Ok(DesignSystem {
        path: path.to_string(),
        text,
        value,
        template_paths,
        sources,
        tokens,
        language,
        results,
    })
}

/// デザインシステム1つの検証の結果（記録しない）。
#[derive(Debug, Clone, PartialEq)]
pub struct DesignTarget {
    /// 経路
    pub path: String,
    /// 検証したときの中身のハッシュ
    pub hash: ContentHash,
    /// rule ごとの結果
    pub results: Vec<RuleResult>,
    /// 判定の4つの区分ごとの件数
    pub counts: Counts,
}

/// デザインシステムの検証の結果。描画と同じく、注入された依存を契約に照らして返すだけで、記録しない。
#[derive(Debug, Clone, PartialEq)]
pub struct DesignValidation {
    /// デザインシステムごとの結果（渡した順）
    pub targets: Vec<DesignTarget>,
    /// 全体の件数
    pub counts: Counts,
}

/// UC4 の対象としてのデザインシステム：受け取りの型とテンプレートの書き方の契約に、置き場所の Concrete で照らして返す。記録しない。
///
/// # Errors
/// デザインシステムが無い ・ JSON でない ・ 契約が無いとき。
pub fn check_design_systems(
    paths: &[String],
    schemas: &[String],
    store: &dyn DocumentStore,
    validator: &dyn SchemaValidator,
    renderer: &dyn TemplateRenderer,
    scripts: &dyn ScriptReader,
) -> Result<DesignValidation, Refused> {
    // 置き場所の Concrete（Abstract の版を申告する文書）を、$id の順に1つずつ読む
    let index = index_of(store, schemas)?;
    let mut concretes: Vec<(String, Value)> = Vec::new();
    for id in index.ids() {
        match read_concrete(&index, store, id, None) {
            Ok((_, doc)) => concretes.push((id.to_string(), doc)),
            Err(r) if r.code == "unknown_schema" => {}
            Err(r) => return Err(r),
        }
    }
    let mut total = Counts::default();
    let mut targets = Vec::new();
    for p in paths {
        let d = check_design_system(p, store, validator, renderer, scripts, &concretes)?;
        let results = d.results;
        let counts = Counts::of(&results);
        total.merge(&counts);
        targets.push(DesignTarget {
            path: d.path,
            hash: ContentHash::of(d.text.as_bytes()),
            results,
            counts,
        });
    }
    Ok(DesignValidation {
        targets,
        counts: total,
    })
}
