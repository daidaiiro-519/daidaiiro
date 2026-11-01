//! rule を、SAS が起動した検証用の子セッションに判定させる（SF1 と UC4 の意味の部分）。
//!
//! - rule × 当たる所ごとに1ステップの手順のプロンプトを作る。渡すのはいまの中身と rule と rule の reads の値だけで、
//!   前の判定と書き込み用の子セッションの答えは渡さない
//! - 子セッションは新しく起動するか（--continue が無いとき）、記録にある検証用の子セッションを同じ起動の設定で続ける
//! - 答えは step ・ rule ・ pointer で突き合わせる。形が違えば同じ子セッションで2回まで直させ、超えたら該当の rule は error
//! - 時間切れは新しい子セッションで2回までやり直す。起動できなければ該当の rule は error
//! - 意味は検証の中の性質で、業務領域にしない。子セッションを起動するのはアダプター（outbound/agents）

use crate::ports::{ReadMeaning, Reply, SchemaValidator};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use validation_domain::values::{By, RuleResult, Verdict};

/// 答えの形（検証用の子セッションが返す JSON）。
pub const ANSWER_SCHEMA: &str = include_str!("../agent-answer.schema.json");

/// 判定の答えの形の $id。
const ANSWER_ID: &str = "urn:semantic-agent-schema:validation:agent-answer-judgments:v1";

/// 形の誤りを同じ子セッションで直させる回数の上限。
pub const MAX_CORRECTIONS: u32 = 2;
/// 時間切れのあと新しい子セッションでやり直す回数の上限。
pub const MAX_RESTARTS: u32 = 2;
/// 同じ子セッションをこの回数より多く使ったら、新しい子セッションを勧める。
pub const MAX_USES: u64 = 10;

/// 検証用の子セッションに頼むときの指定。
pub struct MeaningCall<'a> {
    /// 子セッションを起動する ・ 続ける道具
    pub reader: &'a dyn ReadMeaning,
    /// 続ける検証用の子セッションの ID（--continue）。無ければ新しい子セッションを起動する
    pub continue_child: Option<String>,
}

/// 判定させる1ステップ（rule × 当たる所）。
#[derive(Debug, Clone)]
pub struct Step {
    /// 手順の番号（1から）
    pub no: usize,
    /// rule の id
    pub rule: String,
    /// 当たる所
    pub pointer: String,
    /// rule の定義（x-prompt の write の1行）
    pub definition: Value,
    /// 欄の名前と説明（分かれば）
    pub title: Option<String>,
    /// 欄の説明
    pub description: Option<String>,
    /// 欄の読み方（x-prompt の read：interpret ・ when）
    pub read: Option<Value>,
    /// いまの値
    pub current: Value,
    /// rule の reads が指す所と、その値（判定に一緒に渡す）
    pub reads: Vec<(String, Value)>,
}

/// 対象1つの手順のプロンプト。
#[must_use]
pub fn procedure(path: &str, concrete: &str, hash: &str, steps: &[Step]) -> Value {
    json!({
        "kind": "procedure",
        "operation_kind": "validate",
        "target": {"path": path, "concrete": concrete, "hash": hash},
        "steps": steps.iter().map(|s| {
            let mut v = json!({
                "no": s.no,
                "action": "judge",
                "pointer": s.pointer,
                "rules": [s.definition],
                "current": s.current,
            });
            if let Some(t) = &s.title {
                v["title"] = json!(t);
            }
            if let Some(d) = &s.description {
                v["description"] = json!(d);
            }
            if let Some(r) = &s.read {
                v["read"] = r.clone();
            }
            if !s.reads.is_empty() {
                v["reads"] = s
                    .reads
                    .iter()
                    .map(|(p, val)| json!({"pointer": p, "value": val}))
                    .collect();
            }
            v
        }).collect::<Vec<_>>(),
        "answer": {"schema": "judgments"},
        "constraints": [
            "ファイルを書かない",
            "答えの JSON だけを返す（ほかの文を付けない）",
            "ステップに無い欄に触れない",
        ],
    })
}

/// 子セッションに渡すプロンプトの文。
#[must_use]
pub fn prompt_text(procedure: &Value, answer_schema: &Value) -> String {
    format!(
        "あなたは SAS（Semantic Agent Schema）が起動した検証用の子セッションです。\n\
         次の手順のプロンプトの steps のそれぞれについて、rules の rule（rule ・ why ・ test。bad ・ good があれば、違反する例と満たす例として読む）に照らして current の値を判定し（read があれば、その読み方で値を読む。reads があれば、その値も読んで判定に使う）、\n\
         答えの形に合う JSON だけを返してください。ファイルを書かず、ステップに無い欄に触れないでください。\n\n\
         手順のプロンプト：\n{}\n\n答えの形：\n{}\n",
        serde_json::to_string_pretty(procedure).unwrap_or_default(),
        serde_json::to_string_pretty(answer_schema).unwrap_or_default()
    )
}

/// 形の誤りを直させる文。
#[must_use]
pub fn correction_text(why: &str) -> String {
    format!(
        "前の答えは答えの形に合いませんでした：{why}\n\
         同じ手順の判定を、答えの形に合う JSON だけで返してください（ほかの文を付けない）。\n"
    )
}

/// 答えを読み、形を子セッションに渡したのと同じ答えの形（JSON Schema）に照らしてから、steps と突き合わせる。
/// 合えば (step の no → 判定と理由)。合わなければ理由。
///
/// # Errors
/// JSON でない ・ 答えの形に合わない ・ ステップの判定が欠けた ・ 重なった ・ rule と所が手順と違うとき。
pub fn read_answer(
    validator: &dyn SchemaValidator,
    answer: &str,
    steps: &[Step],
) -> Result<BTreeMap<usize, (Verdict, String)>, String> {
    let v: Value =
        serde_json::from_str(answer.trim()).map_err(|e| format!("JSON として読めない（{e}）"))?;
    let schema: Value = serde_json::from_str(ANSWER_SCHEMA).unwrap_or_default();
    let broken = validator.check(ANSWER_ID, &v, &[(ANSWER_ID.to_string(), schema)])?;
    if !broken.is_empty() {
        return Err(broken
            .iter()
            .map(|(at, why)| format!("{at}：{why}"))
            .collect::<Vec<_>>()
            .join(" ／ "));
    }
    // ここから先は手順との突き合わせ（形は上で確かめた）
    let mut out = BTreeMap::new();
    for r in v["results"].as_array().into_iter().flatten() {
        let no = r["step"]
            .as_u64()
            .and_then(|n| usize::try_from(n).ok())
            .unwrap_or_default();
        let step = steps
            .iter()
            .find(|s| s.no == no)
            .ok_or_else(|| format!("手順に無い step {no}"))?;
        if r["rule"] != json!(step.rule) || r["pointer"] != json!(step.pointer) {
            return Err(format!(
                "step {no} の rule と所が手順と違う（手順は {} {}）",
                step.rule, step.pointer
            ));
        }
        let verdict = match r["result"].as_str() {
            Some("verified") => Verdict::Verified,
            Some("violated") => Verdict::Violated,
            _ => Verdict::Unverified,
        };
        let reason = r["reason"].as_str().unwrap_or_default().to_string();
        if out.insert(no, (verdict, reason)).is_some() {
            return Err(format!("step {no} の判定が2つある"));
        }
    }
    let missing: Vec<String> = steps
        .iter()
        .filter(|s| !out.contains_key(&s.no))
        .map(|s| s.no.to_string())
        .collect();
    if !missing.is_empty() {
        return Err(format!("step {} の判定が無い", missing.join(" ・ ")));
    }
    Ok(out)
}

/// 1つの対象の判定を子セッションに頼んだ結果。
pub struct Judged {
    /// step の no → 判定の結果
    pub results: BTreeMap<usize, RuleResult>,
    /// 判定した子セッションの ID（起動できなければ None）
    pub child: Option<String>,
    /// 起動した子セッション（新しく起動したもの）の ID の並び
    pub started: Vec<String>,
    /// 形の誤りを直させた回数
    pub corrections: u32,
    /// 時間切れで新しい子セッションにした回数
    pub restarts: u32,
}

/// 子セッションの状態（同じ呼び出しの中で、対象をまたいで同じ子セッションを続ける）。
pub struct Child {
    /// いまの子セッションの ID（まだ起動していなければ None）
    pub tool_ref: Option<String>,
    /// 起動の設定
    pub launch: Value,
}

/// 子セッションとのやりとりの結果。
pub struct Conversed<T> {
    /// 読めた答え。読めなければ、なぜ読めなかったか（起動できない ・ 時間切れ ・ 形が合わない）
    pub outcome: Result<T, String>,
    /// 答えた ・ 最後に使った子セッションの ID
    pub child: Option<String>,
    /// 新しく起動した子セッションの ID の並び
    pub started: Vec<String>,
    /// 形の誤りを直させた回数
    pub corrections: u32,
    /// 時間切れで新しい子セッションにした回数
    pub restarts: u32,
}

/// 子セッションに手順を渡し、答えを parse で読む。形が合わなければ同じ子セッションに2回まで直させ、
/// 時間切れなら新しい子セッションで2回までやり直す（判定でも書き込みでも同じ）。
///
/// # Errors
/// 子セッションが処理中（busy）のとき。
pub fn converse<T>(
    reader: &dyn ReadMeaning,
    child: &mut Child,
    first: &str,
    schema: &Value,
    parse: &dyn Fn(&str) -> Result<T, String>,
) -> Result<Conversed<T>, &'static str> {
    let mut prompt = first.to_string();
    let (mut corrections, mut restarts) = (0, 0);
    let mut started: Vec<String> = Vec::new();
    loop {
        let reply = match &child.tool_ref {
            Some(id) => reader.resume(id, &child.launch, &prompt, schema),
            None => reader.start(&prompt, schema),
        };
        match reply {
            Reply::Busy => return Err("busy"),
            Reply::Failed(why) => {
                return Ok(Conversed {
                    outcome: Err(format!("エージェントを立ち上げられない：{why}")),
                    child: child.tool_ref.clone(),
                    started,
                    corrections,
                    restarts,
                });
            }
            Reply::Timeout { tool_ref } => {
                if let Some(id) = tool_ref
                    && child.tool_ref.is_none()
                {
                    started.push(id);
                }
                if restarts >= MAX_RESTARTS {
                    return Ok(Conversed {
                        outcome: Err(
                            "エージェントが時間内に答えない（新しい子セッションで2回やり直した）"
                                .to_string(),
                        ),
                        child: started.last().cloned().or_else(|| child.tool_ref.clone()),
                        started,
                        corrections,
                        restarts,
                    });
                }
                restarts += 1;
                child.tool_ref = None;
                prompt = first.to_string();
            }
            Reply::Answered { tool_ref, answer } => {
                if child.tool_ref.is_none() {
                    started.push(tool_ref.clone());
                }
                child.tool_ref = Some(tool_ref.clone());
                match parse(&answer) {
                    Ok(v) => {
                        return Ok(Conversed {
                            outcome: Ok(v),
                            child: Some(tool_ref),
                            started,
                            corrections,
                            restarts,
                        });
                    }
                    Err(why) => {
                        if corrections >= MAX_CORRECTIONS {
                            return Ok(Conversed {
                                outcome: Err(format!(
                                    "答えが答えの形に合わない（同じ子セッションで2回直させた）：{why}"
                                )),
                                child: Some(tool_ref),
                                started,
                                corrections,
                                restarts,
                            });
                        }
                        corrections += 1;
                        prompt = correction_text(&why);
                    }
                }
            }
        }
    }
}

/// 子セッションに1つの対象の判定を頼む。
///
/// # Errors
/// 子セッションが処理中（busy）のとき。
pub fn judge(
    reader: &dyn ReadMeaning,
    validator: &dyn SchemaValidator,
    child: &mut Child,
    procedure: &Value,
    steps: &[Step],
) -> Result<Judged, &'static str> {
    let schema: Value = serde_json::from_str(ANSWER_SCHEMA).unwrap_or_default();
    let first = prompt_text(procedure, &schema);
    let c = converse(reader, child, &first, &schema, &|a| {
        read_answer(validator, a, steps)
    })?;
    let results = match c.outcome {
        Ok(map) => steps
            .iter()
            .filter_map(|s| {
                map.get(&s.no)
                    .map(|(v, why)| (s.no, RuleResult::new(&s.rule, &s.pointer, By::Ai, *v, why)))
            })
            .collect(),
        Err(why) => steps
            .iter()
            .map(|s| {
                (
                    s.no,
                    RuleResult::new(&s.rule, &s.pointer, By::Ai, Verdict::Error, &why),
                )
            })
            .collect(),
    };
    Ok(Judged {
        results,
        child: c.child,
        started: c.started,
        corrections: c.corrections,
        restarts: c.restarts,
    })
}

/// 対象の中の所に当たる欄の定義（title ・ description ・ x-prompt を引くため）。properties ・ additionalProperties ・ items を
/// たどり、$ref は文書の中（#…）とほかの文書（<$id>#…、others）を引く。$ref と横に並べた欄は、指す先の欄より先に読む。
#[must_use]
pub fn field_schema(schema: &Value, pointer: &str, others: &[(String, Value)]) -> Option<Value> {
    let mut root = schema;
    let mut cur = schema;
    for part in pointer.split('/').skip(1) {
        let part = part.replace("~1", "/").replace("~0", "~");
        let (r, c) = deref(root, cur, others);
        root = r;
        cur = if part.parse::<usize>().is_ok() {
            c.get("items")?
        } else {
            c.get("properties")
                .and_then(|p| p.get(&part))
                .or_else(|| c.get("additionalProperties").filter(|a| a.is_object()))?
        };
    }
    let (_, target) = deref(root, cur, others);
    let mut field = target.clone();
    if let (Some(f), Some(own)) = (field.as_object_mut(), cur.as_object()) {
        for (k, v) in own {
            if k != "$ref" {
                f.insert(k.clone(), v.clone());
            }
        }
    }
    Some(field)
}

/// $ref の先と、その先がある文書。引けなければそのまま。
fn deref<'a>(
    root: &'a Value,
    node: &'a Value,
    others: &'a [(String, Value)],
) -> (&'a Value, &'a Value) {
    let Some(r) = node.get("$ref").and_then(Value::as_str) else {
        return (root, node);
    };
    let (base, fragment) = r.split_once('#').unwrap_or((r, ""));
    let doc = if base.is_empty() {
        Some(root)
    } else {
        others.iter().find(|(id, _)| id == base).map(|(_, d)| d)
    };
    let Some(doc) = doc else {
        return (root, node);
    };
    let target = if fragment.is_empty() {
        Some(doc)
    } else if fragment.starts_with('/') {
        doc.pointer(fragment)
    } else {
        doc.get("$defs").and_then(Value::as_object).and_then(|m| {
            m.values()
                .find(|d| d.get("$anchor").and_then(Value::as_str) == Some(fragment))
        })
    };
    target.map_or((root, node), |t| (doc, t))
}
