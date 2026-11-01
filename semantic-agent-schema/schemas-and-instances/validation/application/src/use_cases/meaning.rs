//! rule の意味の判定（UC4 の意味の部分）：判定の手順を組み、打ったセッションが渡した判定を読む。
//!
//! - rule × 当たる所ごとに1ステップの手順を作る。入れるのはいまの中身と rule と rule の reads の値だけで、
//!   前の判定は入れない。rule の全文は手順の rules に1回だけ書き、ステップは rule の id で指す
//! - 判定は書いたときとは別の文脈で行い、判定用の会話はキャッシュの有効時間のうちは続けて使うよう、constraints で求める。
//!   SAS は判定したのが誰かを見分けない
//! - SAS はエージェントを起動しない。手順は sas validate --semantic で打ったセッションに返し、そのセッションが判定して
//!   sas validate --answer で渡す
//! - 渡された判定は、答えの形 ・ 手順のハッシュ ・ step ・ rule ・ pointer で突き合わせる。合わなければ記録せずに理由を返す
//! - 意味は検証の中の性質で、業務領域にしない

use crate::ports::SchemaValidator;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use validation_domain::values::Verdict;

/// 判定の答えの形（打ったセッションが --answer で渡す JSON）。使う側が従う約束なので、公開された言葉に置く。
pub const ANSWER_SCHEMA: &str =
    include_str!("../../../../published-language/v1/judgment-answer.schema.json");

/// 判定の答えの形の $id。
const ANSWER_ID: &str = "urn:semantic-agent-schema:published-language:judgment-answer:v1";

/// 判定する1ステップ（rule × 当たる所）。
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

/// 対象1つの判定の手順。command は判定を渡すときに打つコマンド。
#[must_use]
pub fn procedure(path: &str, concrete: &str, hash: &str, steps: &[Step], command: &str) -> Value {
    let schema: Value = serde_json::from_str(ANSWER_SCHEMA).unwrap_or_default();
    // rule の全文は、ステップに出てきた順に id ごとに1回だけ書く
    let mut rules: Vec<&Value> = Vec::new();
    let mut seen: Vec<&str> = Vec::new();
    for s in steps {
        if !seen.contains(&s.rule.as_str()) {
            seen.push(&s.rule);
            rules.push(&s.definition);
        }
    }
    json!({
        "$schema": "urn:semantic-agent-schema:published-language:procedure:v1",
        "kind": "procedure",
        "operation_kind": "validate",
        "target": {"path": path, "concrete": concrete, "hash": hash},
        "rules": rules,
        "steps": steps.iter().map(|s| {
            let mut v = json!({
                "no": s.no,
                "action": "judge",
                "pointer": s.pointer,
                "rule": s.rule,
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
        "answer": {"format": "judgments", "schema": schema},
        "command": command,
        "constraints": [
            "steps のそれぞれを、ステップの rule（rule の id）で rules から引いた rule（rule ・ why ・ test。bad ・ good があれば違反する例と満たす例）に照らして current の値で判定する（read があればその読み方で、reads があればその値も読む）",
            "判定は answer の形の JSON にし、hash には target.hash を写して、command で渡す",
            "ファイルを書かない",
            "ステップに無い欄に触れない",
            "断られたら、出力の理由を読んで直し、打ち直す（hash が違うと言われたら、手順を取り直す）",
            "判定は、書いたときとは別の文脈（サブエージェントなど）で行う",
            "判定用の会話は、キャッシュの有効時間のうちは同じものを続けて使い、変わった所の手順だけを渡す",
        ],
    })
}

/// 手順に入れるコマンドの1行。空白や記号を含む引数は単一引用符で囲む。<…> は呼ぶ側が埋める所なので、そのまま出す。
#[must_use]
pub fn command_line(args: &[&str]) -> String {
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

/// 渡された答えを JSON として読む。答えが ```json …``` の囲みで包まれていれば、囲みを外して読む
/// （モデルによって、答えの形を渡しても囲みを付ける）。
///
/// # Errors
/// JSON として読めないとき。
pub fn answer_json(answer: &str) -> Result<Value, String> {
    let t = answer.trim();
    let inner = t
        .strip_prefix("```json")
        .or_else(|| t.strip_prefix("```"))
        .and_then(|r| r.trim_end().strip_suffix("```"))
        .unwrap_or(t);
    serde_json::from_str(inner.trim()).map_err(|e| format!("JSON として読めない（{e}）"))
}

/// 答えを答えの形（JSON Schema）に照らす。
///
/// # Errors
/// 答えの形に合わないとき（外れた所と理由）。
pub fn check_answer(validator: &dyn SchemaValidator, answer: &Value) -> Result<(), String> {
    let schema: Value = serde_json::from_str(ANSWER_SCHEMA).unwrap_or_default();
    let broken = validator.check(ANSWER_ID, answer, &[(ANSWER_ID.to_string(), schema)])?;
    if broken.is_empty() {
        Ok(())
    } else {
        Err(broken
            .iter()
            .map(|(at, why)| format!("{at}：{why}"))
            .collect::<Vec<_>>()
            .join(" ／ "))
    }
}

/// 形を確かめた答えを、steps と突き合わせる。合えば (step の no → 判定と理由)。合わなければ理由。
///
/// # Errors
/// ステップの判定が欠けた ・ 重なった ・ rule と所が手順と違うとき。
pub fn match_steps(
    answer: &Value,
    steps: &[Step],
) -> Result<BTreeMap<usize, (Verdict, String)>, String> {
    let mut out = BTreeMap::new();
    for r in answer["results"].as_array().into_iter().flatten() {
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
