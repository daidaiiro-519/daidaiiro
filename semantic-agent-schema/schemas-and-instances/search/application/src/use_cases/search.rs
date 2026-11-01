//! UC6 検索する：呼ぶ側が渡した範囲の Concrete とインスタンスを探す。検索は記録しない。
//!
//! - 範囲は経路 ・ Concrete（従うインスタンスのすべて）・ すべて（--all）のどれかで、結果に範囲（経路とハッシュ）を返す。
//!   範囲の外は探していない
//! - 構造で絞る：JSON Pointer の所の値が、渡した値と一致する文書（条件が複数なら、すべてに一致する文書）
//! - 意味で探す：範囲の文書ごとに1ステップの手順を組み、打ったセッションに返す（SAS はエージェントを起動しない）。
//!   範囲の中身の合計が上限を超えたら、切り詰めずに断る
//! - 意味の答え：そのセッションが渡した答えの出どころごとに、範囲の中か ・ ハッシュがいまの中身と同じか ・ 所があるかを確かめ、
//!   通ったときだけ、出どころの所のいまの値と一緒に返す

use recording_application::ports::{Caller, IdentifyCaller, Via};
use serde_json::{Value, json};
use validation_application::ports::{DocumentStore, SchemaValidator};
use validation_application::use_cases::meaning::{answer_json, command_line};
use validation_application::use_cases::validate::{Corpus, Doc, Refused, classify, refused};
use writing_domain::aggregates::ConcreteSchema;
use writing_domain::values::ContentHash;

const PROCEDURE: &str = "urn:semantic-agent-schema:published-language:procedure:v1";

/// 意味で探す答えの形（打ったセッションが --answer で渡す JSON）。
pub const ANSWER_SCHEMA: &str = include_str!("../search-answer.schema.json");

/// 意味で探す答えの形の $id。
const ANSWER_ID: &str = "urn:semantic-agent-schema:search:agent-answer-search:v1";

/// 意味で探すときの、範囲の中身の合計の上限（バイト）。書くときの --ref の上限（REF_LIMIT）と同じ。
pub const RANGE_LIMIT: usize = 64 * 1024;

/// 探す範囲。呼ぶ側が必ず渡す。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Range {
    /// 経路を指定した文書
    Paths(Vec<String>),
    /// その $id の Concrete に従うインスタンスのすべて
    Concrete(String),
    /// リポジトリの Concrete とインスタンスのすべて
    All,
}

impl Range {
    /// 範囲をコマンドの引数に戻す（手順の command と案内に入れる）。
    #[must_use]
    pub fn args(&self) -> Vec<String> {
        match self {
            Self::Paths(paths) => paths.clone(),
            Self::Concrete(id) => vec!["--concrete".to_string(), id.clone()],
            Self::All => vec!["--all".to_string()],
        }
    }
}

/// 見つけた値1つ（構造で絞った一致か、意味の答えの出どころ）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Match {
    /// 文書の経路
    pub path: String,
    /// 文書の中の所（JSON Pointer）
    pub pointer: String,
    /// その所のいまの値
    pub value: Value,
}

/// 検索の結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Found {
    /// 探した範囲（経路とハッシュ。経路の順）
    pub range: Vec<(String, String)>,
    /// 意味の答えの文（構造で絞ったときは None）
    pub answer: Option<String>,
    /// 構造で絞ったときは一致、意味の答えでは出どころ
    pub found: Vec<Match>,
}

/// 範囲の文書を読む（経路の順、重なりなし）。範囲の文書は Concrete かインスタンス。
fn load(range: &Range, store: &dyn DocumentStore) -> Result<Vec<Doc>, Refused> {
    let corpus = Corpus::load(store)?;
    let mut docs: Vec<Doc> = match range {
        Range::Paths(paths) => {
            let mut out = Vec::new();
            for p in paths {
                let text = store.read(p).map_err(|e| refused("not_found", p, e))?;
                let value: Value = serde_json::from_str(&text)
                    .map_err(|e| refused("not_json", p, format!("JSON として読めない：{e}")))?;
                let declared = value
                    .get("$schema")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                classify(&corpus.concretes, p, declared)?;
                out.push(Doc {
                    path: p.clone(),
                    text,
                    value,
                });
            }
            out
        }
        Range::Concrete(id) => {
            if !corpus.concretes.contains_key(id) {
                return Err(Refused {
                    code: "unknown_schema",
                    message: format!("{id} はリポジトリの Concrete に無い"),
                    ..Refused::default()
                });
            }
            corpus
                .docs
                .iter()
                .filter(|d| d.value.get("$schema").and_then(Value::as_str) == Some(id))
                .cloned()
                .collect()
        }
        Range::All => corpus
            .docs
            .iter()
            .filter(|d| {
                ConcreteSchema::declares_abstract(&d.value)
                    || d.value
                        .get("$schema")
                        .and_then(Value::as_str)
                        .is_some_and(|s| corpus.concretes.contains_key(s))
            })
            .cloned()
            .collect(),
    };
    docs.sort_by(|a, b| a.path.cmp(&b.path));
    docs.dedup_by(|a, b| a.path == b.path);
    Ok(docs)
}

fn range_of(docs: &[Doc]) -> Vec<(String, String)> {
    docs.iter()
        .map(|d| {
            (
                d.path.clone(),
                ContentHash::of(d.text.as_bytes()).as_str().to_string(),
            )
        })
        .collect()
}

/// 構造で絞る：範囲の文書のうち、条件（JSON Pointer と値）のすべてに一致する文書の、条件の所と値を返す。
///
/// # Errors
/// 範囲の文書を読めない ・ 従う型が決まらない ・ Concrete が無いとき。
pub fn search_where(
    range: &Range,
    conditions: &[(String, Value)],
    store: &dyn DocumentStore,
) -> Result<Found, Refused> {
    let docs = load(range, store)?;
    let mut found = Vec::new();
    for d in &docs {
        if conditions
            .iter()
            .all(|(at, want)| d.value.pointer(at) == Some(want))
        {
            found.extend(conditions.iter().map(|(at, want)| Match {
                path: d.path.clone(),
                pointer: at.clone(),
                value: want.clone(),
            }));
        }
    }
    Ok(Found {
        range: range_of(&docs),
        answer: None,
        found,
    })
}

/// 意味で探す手順を組む：範囲の文書ごとに1ステップ（経路 ・ ハッシュ ・ 中身）。記録しない。
///
/// # Errors
/// 範囲の文書を読めない ・ 範囲に文書が無い ・ 範囲の中身の合計が上限を超えるとき。
pub fn search_procedure(
    range: &Range,
    question: &str,
    store: &dyn DocumentStore,
    who: &dyn IdentifyCaller,
) -> Result<Value, Refused> {
    let docs = load(range, store)?;
    if docs.is_empty() {
        return Err(Refused {
            code: "invalid_input",
            message: "範囲に Concrete もインスタンスも無く、探す文書が無い".to_string(),
            ..Refused::default()
        });
    }
    let total: usize = docs.iter().map(|d| d.text.len()).sum();
    if total > RANGE_LIMIT {
        return Err(Refused {
            code: "range_too_large",
            message: format!(
                "範囲の中身は合わせて {total} バイトで、上限 {RANGE_LIMIT} バイトを超える。切り詰めずに断る。範囲を絞って打ち直す"
            ),
            ..Refused::default()
        });
    }
    let range_rows = range_of(&docs);
    let steps: Vec<Value> = docs
        .iter()
        .zip(&range_rows)
        .enumerate()
        .map(|(i, (d, (_, hash)))| {
            json!({"no": i + 1, "action": "read", "path": d.path, "hash": hash, "current": d.value})
        })
        .collect();
    let schema: Value = serde_json::from_str(ANSWER_SCHEMA).unwrap_or_default();
    Ok(json!({
        "$schema": PROCEDURE,
        "kind": "procedure",
        "operation_kind": "search",
        "question": question,
        "range": range_rows.iter().map(|(p, h)| json!({"path": p, "hash": h})).collect::<Vec<_>>(),
        "steps": steps,
        "answer": {"format": "search", "schema": schema},
        "command": answer_command(range, who),
        "constraints": [
            "steps のすべての文書の current を読み、question に、範囲の文書に書いてあることだけで答える",
            "答えは answer の形の JSON にし、答えの根拠になった値ごとに sources へ path ・ pointer ・ hash（steps の path と hash を写す）を書いて、command で渡す",
            "範囲に無い文書を出どころにしない",
            "ファイルを書かない",
            "断られたら、出力の理由を読んで直し、打ち直す（hash が違うと言われたら、手順を取り直す）",
        ],
    }))
}

/// 答えを渡すときに打つコマンド。--session で名乗ったセッションは、渡すときも名乗る。
fn answer_command(range: &Range, who: &dyn IdentifyCaller) -> String {
    let range_args = range.args();
    let mut args: Vec<&str> = vec!["sas", "search"];
    args.extend(range_args.iter().map(String::as_str));
    args.extend(["--answer", "<答えの JSON のファイル | ->"]);
    let caller = who.identify();
    if let Caller::Session {
        id, via: Via::Flag, ..
    } = &caller
    {
        args.extend(["--session", id.as_str()]);
    }
    command_line(&args)
}

/// 意味の答えを確かめる：答えの形 → 出どころごとに、範囲の中か ・ ハッシュがいまの中身と同じか ・ 所があるか。
/// 通れば、出どころの所のいまの値と一緒に返す。記録しない。
///
/// # Errors
/// 範囲の文書を読めないとき。答えが形に合わない ・ 出どころが範囲の外か所が無い（invalid_answer）・
/// 出どころのハッシュがいまの中身と違う（conflict）とき。
pub fn search_with_answer(
    range: &Range,
    answer: &str,
    store: &dyn DocumentStore,
    validator: &dyn SchemaValidator,
) -> Result<Found, Refused> {
    let docs = load(range, store)?;
    let range_rows = range_of(&docs);
    let invalid = |path: Option<&str>, why: String| {
        let mut r = Refused {
            code: "invalid_answer",
            path: path.map(Into::into),
            message: format!("答えを返さない：{why}"),
            ..Refused::default()
        };
        r.hint = Some(Box::new(
            "答えを手順の answer の形に合わせ、出どころを範囲の steps の文書と、その中にある所に直して打ち直す"
                .to_string(),
        ));
        r
    };
    let v = answer_json(answer).map_err(|e| invalid(None, e))?;
    let schema: Value = serde_json::from_str(ANSWER_SCHEMA).unwrap_or_default();
    let broken = validator
        .check(ANSWER_ID, &v, &[(ANSWER_ID.to_string(), schema)])
        .map_err(|e| Refused {
            code: "validator",
            message: e,
            ..Refused::default()
        })?;
    if !broken.is_empty() {
        return Err(invalid(
            None,
            format!(
                "答えの形に合わない：{}",
                broken
                    .iter()
                    .map(|(at, why)| format!("{at}：{why}"))
                    .collect::<Vec<_>>()
                    .join(" ／ ")
            ),
        ));
    }
    let mut found = Vec::new();
    for s in v["sources"].as_array().into_iter().flatten() {
        let path = s["path"].as_str().unwrap_or_default();
        let pointer = s["pointer"].as_str().unwrap_or_default();
        let hash = s["hash"].as_str().unwrap_or_default();
        let Some((d, (_, now))) = docs.iter().zip(&range_rows).find(|(d, _)| d.path == path) else {
            return Err(invalid(
                Some(path),
                format!("出どころ {path} は探した範囲の外にある。範囲の外は探していない"),
            ));
        };
        if hash != now {
            let mut r = refused(
                "conflict",
                path,
                format!(
                    "出どころ {path} の読んだ中身のハッシュ（{hash}）が、いまの中身（{now}）と違う。答えを返さない"
                ),
            );
            let mut again: Vec<String> = vec!["sas".to_string(), "search".to_string()];
            again.extend(range.args());
            again.extend([
                "--semantic".to_string(),
                "--question".to_string(),
                "<問い>".to_string(),
            ]);
            r.hint = Some(Box::new(format!(
                "いまの中身で手順を取り直して探し直す：{}",
                command_line(&again.iter().map(String::as_str).collect::<Vec<_>>())
            )));
            return Err(r);
        }
        let Some(value) = d.value.pointer(pointer) else {
            return Err(invalid(
                Some(path),
                format!("出どころ {path} に所 {pointer} が無い"),
            ));
        };
        found.push(Match {
            path: path.to_string(),
            pointer: pointer.to_string(),
            value: value.clone(),
        });
    }
    Ok(Found {
        range: range_rows,
        answer: Some(v["answer"].as_str().unwrap_or_default().to_string()),
        found,
    })
}
