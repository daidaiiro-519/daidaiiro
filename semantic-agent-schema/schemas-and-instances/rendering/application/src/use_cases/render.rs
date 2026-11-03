//! UC5 描画する（機械だけ）。
//!
//! SAS が受け取るのは3つだけである：デザインシステム（種類ごとのテンプレートとデザイントークン）、
//! Concrete（SAS の契約で作った JSON Schema）、それに従うインスタンスの JSON。
//! JSON が渡すのは値だけで、どの値をどこにどう置くかはテンプレート（HTML ・ JS）が決め、見た目はトークンの CSS が決める。
//! テンプレートの書き方は Mustache（ロジックを持たない）。SAS は Concrete の根の x-view でテンプレートを選び、値（value）・
//! JS へ渡す値の JSON（json）・ トークン（tokens）・ トークンの CSS（css）を入れて描くだけである。
//! デザインを組み立てる部品を SAS の中に持たない。判定 ・ 記録 ・ ハッシュはページに入れず、記録も読まない。
//! 同じ入力からは同じページになる。
//!
//! 1. デザインシステムを、受け取りの型とテンプレートの書き方の契約に照らす（design.rs）。外れがあれば描かない。
//! 2. 対象を読み、従う Concrete を決める（描くのはインスタンスだけ）。
//! 3. Concrete の根の x-view（無ければ page）のテンプレートがそろうことを確かめる（欠けたら全部を集めて断る）。
//! 4. 描く。すべての対象を描けてから、まとめて書き出す。

use crate::ports::{
    DocumentStore, PageRefusal, PageWriter, SchemaValidator, ScriptReader, TemplateRenderer,
};
use crate::use_cases::design::check_design_system;
use crate::use_cases::tokens::css_of;
use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, BTreeSet};
use validation_domain::values::Verdict;
use writing_domain::aggregates::ConcreteSchema;
use writing_domain::values::{ContentHash, Corpus, TargetKind};

/// 描画 ・ デザインシステムの検証を始められなかった ・ 書き出さなかった理由。描画は記録しない（理由と案内をその場で返す）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Refused {
    /// 決まった名前（not_found ・ not_json ・ no_schema ・ schema_not_listed ・ unknown_schema ・ invalid_input ・ store ・
    /// validator ・ log、描画で断ったもの：invalid_design_system ・ missing_view_kinds ・ template ・ out）
    pub code: &'static str,
    /// 対象の経路（あれば）
    pub path: Option<Box<str>>,
    /// 理由
    pub message: String,
    /// 次にすることの案内（デザインシステムを確かめるコマンド）
    pub hint: Option<Box<String>>,
    /// デザインシステムに無かった x-view の種類（missing_view_kinds）
    pub missing: Option<Box<Vec<String>>>,
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

/// 記録を読み書きできなかったことを、断った理由にする。
#[must_use]
pub fn log_refused(e: String) -> Refused {
    Refused {
        code: "log",
        message: e,
        ..Refused::default()
    }
}

/// リポジトリの文書を読む（Concrete の一覧に使う）。読めない文書は除く。
///
/// # Errors
/// 文書の一覧を読めないとき。
pub(crate) fn load_corpus(store: &dyn DocumentStore) -> Result<Corpus, Refused> {
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

/// 描画の依頼。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderRequest {
    /// 描くインスタンスの経路（1つ以上）
    pub targets: Vec<String>,
    /// デザインシステムのファイルの経路
    pub design_system: String,
    /// 書き出す先
    pub out: String,
}

/// 使ったデザインシステム（出力にだけ返し、ページには入れない）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsedDesignSystem {
    /// 名前
    pub name: String,
    /// 版
    pub version: String,
    /// ファイルの中身のハッシュ
    pub hash: String,
}

/// 描いた1ページ。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page {
    /// 対象の経路
    pub target: String,
    /// 書き出す先からのページの経路
    pub page_path: String,
    /// 使った x-view の種類（並びは名前の順）
    pub views: Vec<String>,
}

/// 描画の結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rendered {
    /// 使ったデザインシステム
    pub design_system: UsedDesignSystem,
    /// 対象ごとのページ
    pub pages: Vec<Page>,
}

/// UC5 描画する。
///
/// # Errors
/// デザインシステムが無い（not_found）・ 型と書き方の契約に合わない（invalid_design_system）・ 対象を読めない（UC4 と同じ code）・
/// 対象がインスタンスでない（invalid_input）・ テンプレートが欠けた（missing_view_kinds）・ テンプレートが無い（not_found）・
/// 描けない（template）・ 書き出すページが入力や対象どうしで重なる（invalid_input）・ 書き出せない（out）とき。
/// どの場合も1ページも書かない。
pub fn render(
    request: &RenderRequest,
    store: &dyn DocumentStore,
    validator: &dyn SchemaValidator,
    renderer: &dyn TemplateRenderer,
    scripts: &dyn ScriptReader,
    writer: &dyn PageWriter,
) -> Result<Rendered, Refused> {
    // 1. デザインシステム：受け取りの型とテンプレートの書き方の契約に照らし、外れがあれば描かない
    let ds_path = &request.design_system;
    let corpus = load_corpus(store)?;
    let ds = check_design_system(ds_path, store, validator, renderer, scripts, &corpus)?;
    if ds.violated() {
        let broken: Vec<String> = ds
            .results
            .iter()
            .filter(|(r, _)| r.verdict() == Verdict::Violated)
            .map(|(r, _)| format!("{}（{}）：{}", r.rule(), r.pointer(), r.reason()))
            .collect();
        return Err(Refused {
            code: "invalid_design_system",
            path: Some(ds_path.as_str().into()),
            message: format!(
                "デザインシステムが契約に合わないので描かない：{}",
                broken.join(" ／ ")
            ),
            hint: Some(Box::new(format!(
                "semantic-agent-schema validate {ds_path}"
            ))),
            ..Refused::default()
        });
    }
    let css = css_of(&ds.tokens);
    let tokens = token_values(ds.value.get("tokens"));
    let templates: BTreeMap<String, String> = ds
        .value
        .get("templates")
        .and_then(Value::as_object)
        .map(|m| {
            m.iter()
                .map(|(k, v)| (k.clone(), v.as_str().unwrap_or_default().to_string()))
                .collect()
        })
        .unwrap_or_default();
    let used = UsedDesignSystem {
        name: text_of(&ds.value, "name"),
        version: text_of(&ds.value, "version"),
        hash: ContentHash::of(ds.text.as_bytes()).as_str().to_string(),
    };
    let template_paths = &ds.template_paths;
    let sources = &ds.sources;

    // 2. 対象と、従う Concrete（同じ対象を2回渡しても1ページ）
    let mut targets: Vec<(String, Value, Value)> = Vec::new();
    for path in &request.targets {
        if targets.iter().any(|(p, _, _)| p == path) {
            continue;
        }
        let text = store
            .read(path)
            .map_err(|e| refused("not_found", path, e))?;
        let value: Value = serde_json::from_str(&text)
            .map_err(|e| refused("not_json", path, format!("JSON として読めない：{e}")))?;
        let declared = value
            .get("$schema")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        if classify(&corpus.concretes, path, &declared)? == TargetKind::Concrete {
            return Err(refused(
                "invalid_input",
                path,
                "描くのは Concrete に従うインスタンスだけ。これは Concrete".to_string(),
            ));
        }
        let concrete = corpus
            .concretes
            .get(&declared)
            .map(|c| c.document().clone())
            .unwrap_or_default();
        targets.push((path.clone(), value, concrete));
    }

    // 3. 使う種類（Concrete の根の x-view。無ければ page）と、欠けた種類
    let kinds: Vec<String> = targets
        .iter()
        .map(|(_, _, concrete)| {
            concrete
                .get("x-view")
                .and_then(Value::as_str)
                .unwrap_or("page")
                .to_string()
        })
        .collect();
    let missing: BTreeSet<String> = kinds
        .iter()
        .filter(|k| !templates.contains_key(*k))
        .cloned()
        .collect();
    if !missing.is_empty() {
        return Err(Refused {
            code: "missing_view_kinds",
            path: Some(ds_path.as_str().into()),
            message: format!(
                "デザインシステムに、Concrete が使う x-view の種類のテンプレートが無い：{}",
                missing.iter().cloned().collect::<Vec<_>>().join(" ・ ")
            ),
            missing: Some(Box::new(missing.into_iter().collect())),
            ..Refused::default()
        });
    }

    // 書き出すページが指してはいけない入力のファイル（実体の経路で比べるのは書き出し先）
    let mut inputs: BTreeSet<String> = std::iter::once(ds_path.clone())
        .chain(template_paths.values().cloned())
        .collect();
    inputs.extend(corpus.docs.iter().map(|d| d.path.clone()));
    inputs.extend(targets.iter().map(|(p, _, _)| p.clone()));

    // 4. 描く。テンプレートに渡すのは値とトークン（値と CSS）だけ
    let mut pages = Vec::new();
    let mut written: Vec<(String, String)> = Vec::new();
    for ((path, value, _), kind) in targets.iter().zip(&kinds) {
        // 値だけを渡す（$schema は従う型の申告で、値ではない）
        let mut values = value.clone();
        if let Some(m) = values.as_object_mut() {
            m.shift_remove("$schema");
        }
        let ctx = json!({
            "value": values,
            "json": script_json(&values),
            "tokens": tokens,
            "css": css,
        });
        let html = renderer.render(sources, kind, &ctx).map_err(|e| {
            refused(
                "template",
                &template_paths[kind],
                format!("{path} を描けない：{e}"),
            )
        })?;
        let page_path = page_path_of(path);
        if written.iter().any(|(p, _)| p == &page_path) {
            return Err(refused(
                "invalid_input",
                path,
                format!("ほかの対象と同じページ {page_path} になる。対象を分けて描く"),
            ));
        }
        written.push((page_path.clone(), html));
        pages.push(Page {
            target: path.clone(),
            page_path,
            views: vec![kind.clone()],
        });
    }
    let inputs: Vec<String> = inputs.into_iter().collect();
    writer
        .write_pages(&request.out, &written, &inputs)
        .map_err(|e| match e {
            PageRefusal::Overlaps { page, input } => refused(
                "invalid_input",
                &page,
                format!(
                    "書き出すページが、描画の入力のファイル {input} と同じ所を指す。--out を別の所にする"
                ),
            ),
            PageRefusal::Failed(message) => refused("out", &request.out, message),
        })?;
    Ok(Rendered {
        design_system: used,
        pages,
    })
}

/// トークンの木から、値だけの木（$ で始まる名前を除き、トークンは $value の値）。テンプレートは tokens.color.accent で引く。
fn token_values(v: Option<&Value>) -> Value {
    let Some(m) = v.and_then(Value::as_object) else {
        return Value::Object(Map::new());
    };
    Value::Object(
        m.iter()
            .filter(|(k, _)| !k.starts_with('$'))
            .map(|(k, child)| {
                let value = child
                    .get("$value")
                    .cloned()
                    .unwrap_or_else(|| token_values(Some(child)));
                (k.clone(), value)
            })
            .collect(),
    )
}

/// JS へ渡す JSON の文字列。<script> に入れても閉じないよう、< > & ' と行の区切り（U+2028 ・ U+2029）を \u の形にする。
fn script_json(v: &Value) -> String {
    let mut out = String::new();
    for c in v.to_string().chars() {
        match c {
            '<' => out.push_str("\\u003c"),
            '>' => out.push_str("\\u003e"),
            '&' => out.push_str("\\u0026"),
            '\'' => out.push_str("\\u0027"),
            '\u{2028}' => out.push_str("\\u2028"),
            '\u{2029}' => out.push_str("\\u2029"),
            c => out.push(c),
        }
    }
    out
}

fn text_of(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

/// 対象の経路から、ページの経路（拡張子を .html に）。
fn page_path_of(target: &str) -> String {
    target
        .strip_suffix(".json")
        .map_or_else(|| format!("{target}.html"), |s| format!("{s}.html"))
}
