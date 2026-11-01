//! rule を、SAS が起動した検証用の子セッションに判定させる（SF1 と UC4 の意味の部分）。
//!
//! - rule × 当たる所ごとに1ステップの手順のプロンプトを作る。渡すのはいまの中身と rule だけで、前の判定 ・ 書き込み用の子セッションの答え ・
//!   brief は渡さない
//! - 子セッションは新しく起動するか（--continue が無いとき）、記録にある検証用の子セッションを同じ起動の設定で続ける
//! - 答えは step ・ rule ・ pointer で突き合わせる。形が違えば同じ子セッションで2回まで直させ、超えたら該当の rule は error
//! - 時間切れは新しい子セッションで2回までやり直す。起動できなければ該当の rule は error
//! - 意味は検証の中の性質で、業務領域にしない。子セッションを起動するのはアダプター（outbound/agents）

use crate::ports::{ReadMeaning, Reply};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use validation_domain::values::{By, RuleResult, Verdict};

/// 答えの形（検証用の子セッションが返す JSON）。
pub const ANSWER_SCHEMA: &str = include_str!("../agent-answer.schema.json");

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
         次の手順のプロンプトの steps のそれぞれについて、rules の rule（rule ・ why ・ test）に照らして current の値を判定し（read があれば、その読み方で値を読む。reads があれば、その値も読んで判定に使う）、\n\
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

/// 答えを読み、steps と突き合わせる。合えば (step の no → 判定と理由)。合わなければ理由。
///
/// # Errors
/// JSON でない ・ 形が違う ・ ステップの判定が欠けた ・ 重なった ・ rule と所が手順と違うとき。
pub fn read_answer(
    answer: &str,
    steps: &[Step],
) -> Result<BTreeMap<usize, (Verdict, String)>, String> {
    let v: Value =
        serde_json::from_str(answer.trim()).map_err(|e| format!("JSON として読めない（{e}）"))?;
    let results = v
        .get("results")
        .and_then(Value::as_array)
        .ok_or("results の並びが無い")?;
    let mut out = BTreeMap::new();
    for r in results {
        let no = r
            .get("step")
            .and_then(Value::as_u64)
            .and_then(|n| usize::try_from(n).ok())
            .ok_or("step が整数でない")?;
        let step = steps
            .iter()
            .find(|s| s.no == no)
            .ok_or_else(|| format!("手順に無い step {no}"))?;
        let rule = r.get("rule").and_then(Value::as_str).unwrap_or_default();
        let pointer = r.get("pointer").and_then(Value::as_str).unwrap_or_default();
        if rule != step.rule || pointer != step.pointer {
            return Err(format!(
                "step {no} の rule と所が手順と違う（手順は {} {}）",
                step.rule, step.pointer
            ));
        }
        let verdict = match r.get("result").and_then(Value::as_str) {
            Some("verified") => Verdict::Verified,
            Some("violated") => Verdict::Violated,
            Some("unverified") => Verdict::Unverified,
            other => {
                return Err(format!(
                    "step {no} の result が verified ・ violated ・ unverified でない（{other:?}）"
                ));
            }
        };
        let reason = r
            .get("reason")
            .and_then(Value::as_str)
            .filter(|s| !s.trim().is_empty())
            .ok_or_else(|| format!("step {no} の reason が無い"))?;
        if out.insert(no, (verdict, reason.to_string())).is_some() {
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

/// 子セッションに1つの対象の判定を頼む。
///
/// # Errors
/// 子セッションが処理中（busy）のとき。
pub fn judge(
    reader: &dyn ReadMeaning,
    child: &mut Child,
    procedure: &Value,
    steps: &[Step],
) -> Result<Judged, &'static str> {
    let schema: Value = serde_json::from_str(ANSWER_SCHEMA).unwrap_or_default();
    let first = prompt_text(procedure, &schema);
    let mut prompt = first.clone();
    let (mut corrections, mut restarts) = (0, 0);
    let mut started = Vec::new();
    let all = |verdict: Verdict, reason: &str| -> BTreeMap<usize, RuleResult> {
        steps
            .iter()
            .map(|s| {
                (
                    s.no,
                    RuleResult::new(&s.rule, &s.pointer, By::Ai, verdict, reason),
                )
            })
            .collect()
    };
    loop {
        let reply = match &child.tool_ref {
            Some(id) => reader.resume(id, &child.launch, &prompt, &schema),
            None => reader.start(&prompt, &schema),
        };
        match reply {
            Reply::Busy => return Err("busy"),
            Reply::Failed(why) => {
                return Ok(Judged {
                    results: all(
                        Verdict::Error,
                        &format!("エージェントを立ち上げられない：{why}"),
                    ),
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
                    return Ok(Judged {
                        results: all(
                            Verdict::Error,
                            "エージェントが時間内に答えない（新しい子セッションで2回やり直した）",
                        ),
                        child: started.last().cloned().or_else(|| child.tool_ref.clone()),
                        started,
                        corrections,
                        restarts,
                    });
                }
                restarts += 1;
                child.tool_ref = None;
                prompt = first.clone();
            }
            Reply::Answered { tool_ref, answer } => {
                if child.tool_ref.is_none() {
                    started.push(tool_ref.clone());
                }
                child.tool_ref = Some(tool_ref.clone());
                match read_answer(&answer, steps) {
                    Ok(map) => {
                        let results = steps
                            .iter()
                            .filter_map(|s| {
                                map.get(&s.no).map(|(v, why)| {
                                    (s.no, RuleResult::new(&s.rule, &s.pointer, By::Ai, *v, why))
                                })
                            })
                            .collect();
                        return Ok(Judged {
                            results,
                            child: Some(tool_ref),
                            started,
                            corrections,
                            restarts,
                        });
                    }
                    Err(why) => {
                        if corrections >= MAX_CORRECTIONS {
                            return Ok(Judged {
                                results: all(
                                    Verdict::Error,
                                    &format!(
                                        "答えが答えの形に合わない（同じ子セッションで2回直させた）：{why}"
                                    ),
                                ),
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

/// JSON Pointer の所の、Concrete（か Abstract）の欄の定義。
#[must_use]
pub fn field_schema<'a>(schema: &'a Value, pointer: &str) -> Option<&'a Value> {
    let mut cur = schema;
    for part in pointer.split('/').skip(1) {
        let part = part.replace("~1", "/").replace("~0", "~");
        cur = if part.parse::<usize>().is_ok() {
            cur.get("items")?
        } else {
            cur.get("properties")?.get(&part)?
        };
    }
    Some(cur)
}
