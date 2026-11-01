//! rule の意味の判定（UC4 の意味の部分）：判定の手順を組み、打ったセッションが渡した判定を読む。
//!
//! - 手順の steps はレビューのまとまりで、形は2つだけ：所のまとまり（pointer ・ before ・ after ・ current ・ rules）と、
//!   文書全体のまとまり（changed ・ rules。いつも1つ）。場面によって変わるのは、まとまりの数と中身だけ
//! - 判定済みの対象では、前の判定のあとに記録された変更（changes の field_changes）から変わった所を決め、
//!   その所に当たる rule ・ その所を related_fields で読む rule ・ 文書全体に当たる rule ・ 文が変わった rule だけを入れる
//! - rule の全文は手順の rules に1回だけ書き、まとまりは rule の id で指す。前の判定は入れない
//! - 判定は書いたときとは別の文脈で行い、判定用の会話はキャッシュの有効時間のうちは続けて使うよう、constraints で求める。
//!   SAS は判定したのが誰かを見分けない
//! - SAS はエージェントを起動しない。手順は sas validate --semantic で打ったセッションに返し、そのセッションが判定して
//!   sas validate --answer で渡す
//! - 渡された判定は、答えの形 ・ 手順のハッシュ ・ step（まとまりの番号）・ rule ・ pointer で突き合わせる。合わなければ記録せずに理由を返す
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

/// 手順がどこまでを持つか（手順の review.scope）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewScope {
    /// 前の判定のあとに変わった所だけ
    Changed,
    /// --all-rules：前の判定があっても、すべての当たる所
    AllRules,
    /// 前の判定を使えない（前の判定が無い ・ そのあとの中身が記録で説明できない）ので、すべての当たる所
    First,
}

impl ReviewScope {
    /// 手順に書く名前。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Changed => "changed",
            Self::AllRules => "all-rules",
            Self::First => "first",
        }
    }
}

/// 前の判定のあとに記録された変更の操作1つ。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeOp {
    /// 操作の ID
    pub operation: String,
    /// 打ったセッションの ID（人なら None）
    pub session: Option<String>,
    /// 変えた文書の経路（rule の文の変更なら Concrete の経路）
    pub target: String,
    /// 変更の理由（渡されたときだけ）
    pub reason: Option<String>,
    /// 対象を書いた操作なら、変わった所
    pub pointers: Vec<String>,
    /// Concrete を書いた操作なら、文が変わった rule の id
    pub rules: std::collections::BTreeSet<String>,
}

impl ChangeOp {
    /// 手順に書く形（operation ・ session ・ target ・ reason）。
    #[must_use]
    pub fn to_json(&self) -> Value {
        let mut v =
            json!({"operation": self.operation, "session": self.session, "target": self.target});
        if let Some(r) = &self.reason {
            v["reason"] = json!(r);
        }
        v
    }
}

/// レビューのまとまり1つ。所のまとまり（pointer が空でない）か、文書全体のまとまり（pointer が空）。
#[derive(Debug, Clone)]
pub struct Group {
    /// まとまりの番号（1から。答えの step）
    pub no: usize,
    /// 所（JSON Pointer）。文書全体のまとまりは空
    pub pointer: String,
    /// 前の判定のときの値。その所に前の判定が無い（前の判定を使えない ・ そのあとに足した所）ときだけ None（null）
    pub before: Option<Value>,
    /// 前の判定のあとに、この所に関わった変更の操作（文書全体のまとまりは、変わった所の操作）
    pub changes: Vec<ChangeOp>,
    /// いまの値（文書全体のまとまりは文書全体）
    pub current: Value,
    /// 文書全体のまとまりだけ：前の判定のあとに変わった所の一覧（前の判定を使えないときは空）
    pub changed: Vec<String>,
    /// 欄の名前（分かれば）
    pub title: Option<String>,
    /// 欄の説明
    pub description: Option<String>,
    /// 欄の意味（x-prompt の meaning）
    pub meaning: Option<String>,
    /// 欄をいつ読むか（x-prompt の when）
    pub when: Option<String>,
}

/// 判定する1件（rule × 当たる所）。no はそれを入れたまとまりの番号。
#[derive(Debug, Clone)]
pub struct Step {
    /// まとまりの番号
    pub no: usize,
    /// rule の id
    pub rule: String,
    /// 当たる所
    pub pointer: String,
    /// rule の定義（x-prompt の rules の1件）
    pub definition: Value,
    /// rule の related_fields が指す所と、その値（判定に一緒に渡す）
    pub related_fields: Vec<(String, Value)>,
}

/// 対象1つの判定の手順。command は判定を渡すときに打つコマンド。
#[must_use]
pub fn procedure(
    path: &str,
    concrete: &str,
    hash: &str,
    review: (Option<&str>, ReviewScope),
    groups: &[Group],
    steps: &[Step],
    command: &str,
) -> Value {
    let schema: Value = serde_json::from_str(ANSWER_SCHEMA).unwrap_or_default();
    // rule の全文は、出てきた順に id ごとに1回だけ書く
    let mut rules: Vec<&Value> = Vec::new();
    let mut seen: Vec<&str> = Vec::new();
    for s in steps {
        if !seen.contains(&s.rule.as_str()) {
            seen.push(&s.rule);
            rules.push(&s.definition);
        }
    }
    let rendered: Vec<Value> = groups
        .iter()
        .map(|g| {
            let ids: Vec<&str> = steps
                .iter()
                .filter(|s| s.no == g.no)
                .map(|s| s.rule.as_str())
                .collect();
            if g.pointer.is_empty() {
                return json!({
                    "no": g.no,
                    "action": "review-document",
                    "pointer": "",
                    "changed": g.changed,
                    "changes": g.changes.iter().map(ChangeOp::to_json).collect::<Vec<_>>(),
                    "current": g.current,
                    "rules": ids,
                });
            }
            let mut v = json!({
                "no": g.no,
                "action": "review-place",
                "pointer": g.pointer,
                "before": g.before,
                "current": g.current,
                "changes": g.changes.iter().map(ChangeOp::to_json).collect::<Vec<_>>(),
                "rules": ids,
            });
            if let Some(t) = &g.title {
                v["title"] = json!(t);
            }
            if let Some(d) = &g.description {
                v["description"] = json!(d);
            }
            if let Some(m) = &g.meaning {
                v["meaning"] = json!(m);
            }
            if let Some(w) = &g.when {
                v["when"] = json!(w);
            }
            let mut reads: Vec<Value> = Vec::new();
            for s in steps.iter().filter(|s| s.no == g.no) {
                for (p, val) in &s.related_fields {
                    let r = json!({"pointer": p, "value": val});
                    if !reads.contains(&r) {
                        reads.push(r);
                    }
                }
            }
            if !reads.is_empty() {
                v["related_fields"] = Value::Array(reads);
            }
            v
        })
        .collect();
    json!({
        "$schema": "urn:semantic-agent-schema:published-language:procedure:v1",
        "kind": "procedure",
        "operation_kind": "validate",
        "target": {"path": path, "schema": concrete, "hash": hash},
        "review": {"since": review.0, "scope": review.1.as_str()},
        "rules": rules,
        "steps": rendered,
        "answer": {"format": "judgments", "schema": schema},
        "command": command,
        "constraints": [
            "steps はレビューのまとまり。所のまとまり（review-place）は、その所の rules のそれぞれを、前の判定のときの値（before）といまの値（current）を見比べ、変更の操作（changes）の理由を踏まえて、いまの値で判定する。文書全体のまとまり（review-document）は、変わった所（changed）と変更の操作（changes）を踏まえて、文書全体（current）で rules のそれぞれを判定する",
            "rule は id で手順の rules から引き、rule ・ why ・ check と、違反する例 bad ・ 満たす例 good に照らす。meaning があればその読み方で、when があればその読み手として、related_fields があればその値も読む",
            "判定は answer の形の JSON にし、まとまりの rules の1つごとに1件（step はまとまりの no、pointer はまとまりの pointer）を書き、hash には target.hash を写して、command で渡す",
            "ファイルを書かない",
            "まとまりに無い欄に触れない",
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

/// 形を確かめた答えを、判定する rule × 当たる所と突き合わせる。合えば (rule × 当たる所の位置 → 判定と理由)。合わなければ理由。
///
/// # Errors
/// 判定が欠けた ・ 重なった ・ 手順に無い rule と所 ・ まとまりの番号が違うとき。
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
        let rule = r["rule"].as_str().unwrap_or_default();
        let pointer = r["pointer"].as_str().unwrap_or_default();
        let (i, step) = steps
            .iter()
            .enumerate()
            .find(|(_, s)| s.rule == rule && s.pointer == pointer)
            .ok_or_else(|| format!("rule と所が手順と違う（{rule} {pointer} は手順に無い）"))?;
        if step.no != no {
            return Err(format!(
                "{rule} {pointer} のまとまりの番号が手順と違う（手順は step {}）",
                step.no
            ));
        }
        let verdict = match r["result"].as_str() {
            Some("verified") => Verdict::Verified,
            Some("violated") => Verdict::Violated,
            _ => Verdict::Unverified,
        };
        let reason = r["reason"].as_str().unwrap_or_default().to_string();
        if out.insert(i, (verdict, reason)).is_some() {
            return Err(format!("{rule} {pointer} の判定が2つある"));
        }
    }
    let missing: Vec<String> = steps
        .iter()
        .enumerate()
        .filter(|(i, _)| !out.contains_key(i))
        .map(|(_, s)| format!("step {} の {} {}", s.no, s.rule, s.pointer))
        .collect();
    if !missing.is_empty() {
        return Err(format!("{} の判定が無い", missing.join(" ・ ")));
    }
    Ok(out)
}

/// 2つの所が重なるか：同じか、片方がもう片方の中（文書全体 "" はどの所とも重なる）。
#[must_use]
pub fn related(a: &str, b: &str) -> bool {
    a == b
        || a.is_empty()
        || b.is_empty()
        || a.strip_prefix(b).is_some_and(|r| r.starts_with('/'))
        || b.strip_prefix(a).is_some_and(|r| r.starts_with('/'))
}

/// JSON Pointer の参照トークン（~1 ・ ~0 を戻す）。
fn tokens(pointer: &str) -> Vec<String> {
    pointer
        .split('/')
        .skip(1)
        .map(|t| t.replace("~1", "/").replace("~0", "~"))
        .collect()
}

/// 所の親と最後のトークン。
fn parent_of<'a>(doc: &'a mut Value, pointer: &str) -> Option<(&'a mut Value, String)> {
    let mut t = tokens(pointer);
    let last = t.pop()?;
    let mut here = doc;
    for k in t {
        here = match here {
            Value::Object(m) => m.get_mut(&k)?,
            Value::Array(a) => a.get_mut(k.parse::<usize>().ok()?)?,
            _ => return None,
        };
    }
    Some((here, last))
}

/// 所に値を置く（配列は insert なら差し込み、そうでなければ置き換え）。value が None なら所を除く。
fn put(doc: &mut Value, pointer: &str, value: Option<Value>, insert: bool) {
    let Some((parent, last)) = parent_of(doc, pointer) else {
        return;
    };
    match (parent, value) {
        (Value::Object(m), Some(v)) => {
            m.insert(last, v);
        }
        (Value::Object(m), None) => {
            m.shift_remove(&last);
        }
        (Value::Array(a), v) => {
            let Ok(i) = last.parse::<usize>() else {
                return;
            };
            match v {
                Some(v) if insert && i <= a.len() => a.insert(i, v),
                Some(v) => {
                    if let Some(slot) = a.get_mut(i) {
                        *slot = v;
                    }
                }
                None if i < a.len() => {
                    a.remove(i);
                }
                None => {}
            }
        }
        _ => {}
    }
}

/// 記録した変わった欄1つ（JSON の文字列で持つ値は読んである）。
#[derive(Debug, Clone, PartialEq)]
pub struct FieldEdit {
    /// 所
    pub pointer: String,
    /// add ・ remove ・ replace
    pub change: String,
    /// 前の値（無い ・ 取れなかったときは None）
    pub before: Option<Value>,
    /// 後の値
    pub after: Option<Value>,
}

/// いまの文書から、記録した変更（書いた操作ごとの変わった欄。古い順）を新しいほうから戻して、前の判定のときの文書を組む。
/// 1つの操作の中は、置き換えを戻してから、除いた要素を前から差し込み、足した要素を後ろから除く（配列の位置がずれないように）。
#[must_use]
pub fn revert(now: &Value, writes: &[Vec<FieldEdit>]) -> Value {
    let mut doc = now.clone();
    for edits in writes.iter().rev() {
        for e in edits.iter().filter(|e| e.change == "replace") {
            put(&mut doc, &e.pointer, e.before.clone(), false);
        }
        for e in edits.iter().filter(|e| e.change == "remove") {
            put(&mut doc, &e.pointer, e.before.clone(), true);
        }
        for e in edits.iter().rev().filter(|e| e.change == "add") {
            put(&mut doc, &e.pointer, None, false);
        }
    }
    doc
}

/// 値の中の rule（id と rule を持つオブジェクト）の id を集める。
fn rule_ids(v: &Value, out: &mut std::collections::BTreeSet<String>) {
    match v {
        Value::Object(m) => {
            if let (Some(id), true) = (m.get("id").and_then(Value::as_str), m.contains_key("rule"))
            {
                out.insert(id.to_string());
            }
            for x in m.values() {
                rule_ids(x, out);
            }
        }
        Value::Array(a) => a.iter().for_each(|x| rule_ids(x, out)),
        _ => {}
    }
}

/// Concrete の変わった欄から、文が変わった rule の id を決める：変わった値の中の rule と、変わった所を含む rule（いまの Concrete で）。
#[must_use]
pub fn touched_rules(
    concrete_now: &Value,
    edits: &[FieldEdit],
) -> std::collections::BTreeSet<String> {
    let mut out = std::collections::BTreeSet::new();
    for e in edits {
        for v in [&e.before, &e.after].into_iter().flatten() {
            rule_ids(v, &mut out);
        }
        let t = tokens(&e.pointer);
        for n in 0..t.len() {
            let prefix: String = t[..n]
                .iter()
                .map(|k| format!("/{}", k.replace('~', "~0").replace('/', "~1")))
                .collect();
            if let Some(Value::Object(m)) = concrete_now.pointer(&prefix)
                && let (Some(id), true) =
                    (m.get("id").and_then(Value::as_str), m.contains_key("rule"))
            {
                out.insert(id.to_string());
            }
        }
    }
    out
}
