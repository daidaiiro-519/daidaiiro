//! UC6 検索する：呼ぶ側が渡した範囲の Concrete とインスタンスを探す。検索は記録しない。
//!
//! - 範囲は渡された経路の並びで、結果に範囲（経路とハッシュ）を返す。範囲の外は探していない。
//!   インスタンスが従う Concrete は、渡されたスキーマの置き場所に $id であることを確かめる
//! - 構造で絞る：JSON Pointer の所の値が、渡した値と一致する文書（条件が複数なら、すべてに一致する文書）
//! - 意味で探す：範囲の文書ごとに1ステップ（経路とハッシュ）の手順を組み、打ったセッションに返す（Semantic Agent Schema は
//!   エージェントを起動しない）。中身は手順に貼らず、手順を受け取ったセッションが経路から読む
//! - 意味の答え：そのセッションが渡した答えの出どころごとに、範囲の中か ・ ハッシュがいまの中身と同じか ・ 所があるかを確かめ、
//!   通ったときだけ、出どころの所のいまの値と一緒に返す

use crate::ports::{DocumentStore, SchemaValidator};
use serde_json::{Value, json};
use writing_domain::services::{SchemaIndex, concrete_of};
use writing_domain::values::{ContentHash, SchemaPlaces, TargetKind};

const PROCEDURE: &str = "urn:semantic-agent-schema:published-language:procedure:v1";

/// 意味で探す答えの形（打ったセッションが --answer で渡す JSON）。使う側が従う約束なので、公開された言葉に置く。
pub const ANSWER_SCHEMA: &str =
    include_str!("../../../../published-language/v1/search-answer.schema.json");

/// 意味で探す答えの形の $id。
const ANSWER_ID: &str = "urn:semantic-agent-schema:published-language:search-answer:v1";

/// 探す範囲。呼ぶ側が必ず渡し、既定の範囲は持たない。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Range {
    /// 範囲の文書の経路
    pub paths: Vec<String>,
    /// スキーマの置き場所（インスタンスが従う Concrete を $id で引く。返すコマンドにも引き継ぐ）
    pub schemas: SchemaPlaces,
}

impl Range {
    /// 範囲と置き場所をコマンドの引数に戻す（手順の command と案内に入れる）。
    #[must_use]
    pub fn args(&self) -> Vec<String> {
        let mut args = self.paths.clone();
        args.extend(self.schemas.args());
        args
    }
}

/// 範囲の文書1つ。
#[derive(Debug, Clone, PartialEq)]
struct Doc {
    path: String,
    text: String,
    value: Value,
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

/// 検索を始められなかった ・ 答えを返さなかった理由。検索は記録しない（理由と案内をその場で返す）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Refused {
    /// 決まった名前（not_found ・ not_json ・ no_schema ・ schema_not_listed ・ unknown_schema ・ duplicate_schema ・ invalid_input ・
    /// store ・ validator、答えが形に合わない ・ 出どころを確かめられないもの：invalid_answer ・ conflict）
    pub code: &'static str,
    /// 対象の経路（あれば）
    pub path: Option<Box<str>>,
    /// 理由
    pub message: String,
    /// 次にすることの案内（答えの直し方 ・ 手順を取り直すコマンド）
    pub hint: Option<Box<String>>,
    /// 同じ $id を持つファイルの経路（duplicate_schema）
    pub paths: Option<Box<Vec<String>>>,
}

/// 対象の経路を付けて断る。
fn refused(code: &'static str, path: &str, message: String) -> Refused {
    Refused {
        code,
        path: Some(path.into()),
        message,
        ..Refused::default()
    }
}

/// 範囲の文書を読む（経路の順、重なりなし）。範囲の文書は Concrete か、渡された置き場所に Concrete があるインスタンス。
fn load(range: &Range, store: &dyn DocumentStore) -> Result<Vec<Doc>, Refused> {
    let mut index: Option<SchemaIndex> = None;
    let mut docs = Vec::new();
    for p in &range.paths {
        let text = store.read(p).map_err(|e| refused("not_found", p, e))?;
        let value: Value = serde_json::from_str(&text)
            .map_err(|e| refused("not_json", p, format!("JSON として読めない：{e}")))?;
        let declared = value
            .get("$schema")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let kind = TargetKind::classify(&declared)
            .map_err(|u| refused(u.code(), p, u.message().to_string()))?;
        if kind == TargetKind::Instance {
            if index.is_none() {
                let mut i = SchemaIndex::new();
                for f in range.schemas.files() {
                    let text = store.read(f).map_err(|e| refused("store", f, e))?;
                    i.add(f, &text);
                }
                index = Some(i);
            }
            if let Some(i) = &index {
                // 従う Concrete が置き場所に1つだけあり、Concrete であることを確かめる
                let at = i
                    .path_of(&declared, None)
                    .map_err(|u| unresolved(Some(p), &u))?
                    .to_string();
                let text = store.read(&at).map_err(|e| refused("store", &at, e))?;
                let doc: Value = serde_json::from_str(&text)
                    .map_err(|e| refused("store", &at, format!("JSON として読めない：{e}")))?;
                concrete_of(&declared, &at, &doc).map_err(|u| unresolved(Some(p), &u))?;
            }
        }
        docs.push(Doc {
            path: p.clone(),
            text,
            value,
        });
    }
    docs.sort_by(|a, b| a.path.cmp(&b.path));
    docs.dedup_by(|a, b| a.path == b.path);
    Ok(docs)
}

/// 置き場所から引けなかった理由を、断った理由にする。
fn unresolved(path: Option<&str>, u: &writing_domain::services::Unresolved) -> Refused {
    Refused {
        code: u.code(),
        path: path.map(Into::into),
        message: u.message().to_string(),
        paths: (!u.paths().is_empty()).then(|| Box::new(u.paths().to_vec())),
        ..Refused::default()
    }
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

/// 意味で探す手順を組む：範囲の文書ごとに1ステップ（経路とハッシュ。中身は貼らず、手順を受け取ったセッションが読む）。記録しない。
///
/// # Errors
/// 範囲の文書を読めない ・ 範囲に文書が無いとき。
pub fn search_procedure(
    range: &Range,
    question: &str,
    store: &dyn DocumentStore,
) -> Result<Value, Refused> {
    let docs = load(range, store)?;
    if docs.is_empty() {
        return Err(Refused {
            code: "invalid_input",
            message: "範囲に Concrete もインスタンスも無く、探す文書が無い".to_string(),
            ..Refused::default()
        });
    }
    let range_rows = range_of(&docs);
    let steps: Vec<Value> = docs
        .iter()
        .zip(&range_rows)
        .enumerate()
        .map(|(i, (d, (_, hash)))| json!({"no": i + 1, "action": "read", "path": d.path, "hash": hash}))
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
        "command": answer_command(range),
        "constraints": [
            "steps のすべての文書を path から読み、question に、範囲の文書に書いてあることだけで答える",
            "答えは answer の形の JSON にし、答えの根拠になった値ごとに sources へ path ・ pointer ・ hash（steps の path と hash を写す）を書いて、command で渡す",
            "範囲に無い文書を出どころにしない",
            "ファイルを書かない",
            "断られたら、出力の理由を読んで直し、打ち直す（hash が違うと言われたら、手順を取り直す）",
        ],
    }))
}

/// 答えを渡すときに打つコマンド。受け取った範囲と --schemas を引き継ぐ（検索は誰が打ったかを取らない）。
fn answer_command(range: &Range) -> String {
    let range_args = range.args();
    let mut args: Vec<&str> = vec!["semantic-agent-schema", "search"];
    args.extend(range_args.iter().map(String::as_str));
    args.extend(["--answer", "<答えの JSON のファイル | ->"]);
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
            let mut again: Vec<String> =
                vec!["semantic-agent-schema".to_string(), "search".to_string()];
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

/// 手順に入れるコマンドの1行。空白や記号を含む引数は単一引用符で囲む。<…> は呼ぶ側が埋める所なので、そのまま出す。
fn command_line(args: &[&str]) -> String {
    args.iter()
        .map(|a| {
            let plain = !a.is_empty()
                && a.chars()
                    .all(|c| c.is_ascii_alphanumeric() || "-_./:@=+,".contains(c));
            let placeholder = a.starts_with('<') && a.ends_with('>');
            if plain || placeholder {
                (*a).to_string()
            } else {
                format!("'{}'", a.replace('\'', "'\\''"))
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// 渡された答えを JSON として読む。JSON だけを受け取り、囲み（```json）などを外して読まない。
fn answer_json(answer: &str) -> Result<Value, String> {
    serde_json::from_str(answer)
        .map_err(|e| format!("JSON として読めない（{e}）。答えは JSON だけを渡す"))
}
