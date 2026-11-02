//! UC1 作るの承認の材料：確認の1画面の材料（説明 ・ 完成イメージ ・ 振る舞い ・ 構造 ・ 推測の一覧）と、別の会話に任せる手順。
//! validate が Skill ごとに返す。図は描かず、データだけを返す。何も書かない。
//!
//! 作る人が答えるのは目的（purpose）と役割（role）だけなので、それ以外の所はすべて推測とみなし、推測の一覧はそこから
//! 機械で作る（項目を足さない）。件ごとに from を持つ所（受け持たないこと ・ 参照する資料）は、作る人の答え（問1〜問3 ・
//! 資料の問い）を from に持たない件だけを推測とする。
//! 「分からない」と答えた所は一覧の先頭に出し、目的の who と gets が分からないままなら承認を受け付けない。

use crate::use_cases::trimmed;
use serde_json::Value;
use skills_domain::services::derivation::{derived_from, is_answer};

/// 作る人が答えられなかった所に入れる値。
pub const UNKNOWN: &str = "分からない";

/// 推測の一覧の1行。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Guess {
    /// skill.json の中の所（JSON Pointer）
    pub field: String,
    /// 中身（1行）
    pub content: String,
    /// 何から推測したか
    pub from: String,
    /// 作る人が「分からない」と答えた所か
    pub unknown: bool,
}

/// 別の会話に任せる手順。
#[derive(Debug, Clone, PartialEq)]
pub struct Delegate {
    /// a（意味の判定）か b（例を動かす）
    pub id: &'static str,
    /// 何を任せるか
    pub what: String,
    /// どう任せるか
    pub how: String,
    /// (a) で打つコマンド
    pub command: Option<String>,
    /// (b) で動かす例（id ・ given ・ input ・ expected）
    pub examples: Vec<[String; 4]>,
}

/// 確認の1画面の材料。
#[derive(Debug, Clone, PartialEq)]
pub struct Approval {
    /// name
    pub name: String,
    /// description
    pub description: String,
    /// 完成イメージ（例の id と expected）
    pub images: Vec<(String, String)>,
    /// 振る舞い（例の id と behavior の who ・ does ・ then）
    pub behaviors: Vec<(String, [String; 3])>,
    /// 構造（inputs ・ steps ・ edge_cases ・ outputs ・ does ・ does_not。skill.json の値のまま）
    pub structure: Vec<(&'static str, Value)>,
    /// 推測の一覧（分からない所 → 損失の大きい順）
    pub guesses: Vec<Guess>,
    /// 承認できない理由（空なら承認できる）
    pub not_approvable: Vec<String>,
    /// 別の会話に任せる手順
    pub delegate: Vec<Delegate>,
}

/// 推測の一覧に並べる所（損失の大きい順）：受け持たないこと → 最低保証 → rules → 完了条件 → 手順 → 参照する資料 → そのほか。
/// 件に from も例も無いときの「何から推測したか」は、domain の1つの表（derivation）から引く。
const ORDER: [&str; 14] = [
    "role/does_not",
    "minimal_guarantees",
    "rules",
    "completion_criteria",
    "steps",
    "references",
    "name",
    "description",
    "compatibility",
    "preconditions",
    "inputs",
    "outputs",
    "edge_cases",
    "examples",
];

fn s<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(Value::as_str).unwrap_or_default()
}

/// 件の中身を1行にする。
fn content(field: &str, v: &Value) -> String {
    match (field, v) {
        (_, Value::String(t)) => t.clone(),
        ("/role/does_not", _) => s(v, "what").to_string(),
        ("/references", _) => format!("{}（{}）", s(v, "name"), s(v, "path")),
        ("/rules", _) => s(v, "rule").to_string(),
        ("/completion_criteria", _) => s(v, "criterion").to_string(),
        ("/steps", _) => s(v, "do").to_string(),
        ("/inputs" | "/outputs", _) => {
            // 形と置き場所のうち、あるものだけを括弧に入れる
            let more: Vec<&str> = ["form", "where"]
                .into_iter()
                .map(|k| s(v, k))
                .filter(|t| !t.is_empty())
                .collect();
            if more.is_empty() {
                s(v, "name").to_string()
            } else {
                format!("{}（{}）", s(v, "name"), more.join(" ・ "))
            }
        }
        ("/edge_cases", _) => format!("{} {} → {}", s(v, "id"), s(v, "when"), s(v, "then")),
        ("/examples", _) => format!(
            "{} 「{}」 → {}",
            s(v, "id"),
            s(v, "input"),
            s(v, "expected")
        ),
        _ => v.to_string(),
    }
}

/// 件が何から推測されたか：件の from、もとにした例、そのどちらも無ければ所ごとの決まり。
fn guessed_from(v: &Value, rule: &str) -> String {
    if let Some(f) = v.get("from").and_then(Value::as_str) {
        return f.to_string();
    }
    if let Some(ids) = v.get("examples").and_then(Value::as_array) {
        let ids: Vec<&str> = ids.iter().filter_map(Value::as_str).collect();
        if !ids.is_empty() {
            return format!("例 {} から", ids.join(" ・ "));
        }
    }
    rule.to_string()
}

/// 値の中に「分からない」があるか。
fn has_unknown(v: &Value) -> bool {
    match v {
        Value::String(t) => t == UNKNOWN,
        Value::Array(a) => a.iter().any(has_unknown),
        Value::Object(m) => m.values().any(has_unknown),
        _ => false,
    }
}

/// 「分からない」と答えた所（JSON Pointer）。
fn unknowns(v: &Value, at: &str, out: &mut Vec<String>) {
    match v {
        Value::String(t) if t == UNKNOWN => out.push(at.to_string()),
        Value::Array(a) => {
            for (i, x) in a.iter().enumerate() {
                unknowns(x, &format!("{at}/{i}"), out);
            }
        }
        Value::Object(m) => {
            for (k, x) in m {
                unknowns(
                    x,
                    &format!("{at}/{}", k.replace('~', "~0").replace('/', "~1")),
                    out,
                );
            }
        }
        _ => {}
    }
}

/// 推測の一覧を、skill.json から機械で作る。
fn guesses(skill: &Value) -> Vec<Guess> {
    let mut out = Vec::new();
    let mut unknown_at = Vec::new();
    unknowns(skill, "", &mut unknown_at);
    for at in unknown_at {
        out.push(Guess {
            field: at,
            content: UNKNOWN.to_string(),
            from: "作る人が「分からない」と答えた".to_string(),
            unknown: true,
        });
    }
    for name in ORDER {
        let field = format!("/{name}");
        let field = field.as_str();
        let rule = derived_from(name).unwrap_or_default();
        let Some(v) = skill.pointer(field) else {
            continue;
        };
        let rows: Vec<(String, &Value)> = match v {
            Value::Array(a) => a
                .iter()
                .enumerate()
                .map(|(i, x)| (format!("{field}/{i}"), x))
                .collect(),
            other => vec![(field.to_string(), other)],
        };
        for (at, x) in rows {
            if has_unknown(x) {
                continue;
            }
            // 受け持たないこと（role）と参照する資料は、作る人の答え（問1〜問3 ・ 資料の問い）から取った件は推測ではない。
            // ほかの所（完了条件の from「問2 の答え」など）は、答えから導いたものなので推測のまま
            if matches!(field, "/role/does_not" | "/references") && is_answer(s(x, "from")) {
                continue;
            }
            out.push(Guess {
                field: at,
                content: content(field, x),
                from: guessed_from(x, rule),
                unknown: false,
            });
        }
    }
    out
}

fn text(v: &Value, pointer: &str) -> String {
    v.pointer(pointer)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn array(v: &Value, pointer: &str) -> Value {
    v.pointer(pointer)
        .cloned()
        .filter(Value::is_array)
        .unwrap_or_else(|| Value::Array(Vec::new()))
}

/// 確認の1画面の材料を、読んだ skill.json から作る。folder は Skill のフォルダ（別の会話に任せる手順に使う）。
#[must_use]
pub fn approval(folder: &str, skill: &Value) -> Approval {
    let examples: Vec<Value> = skill
        .get("examples")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let images = examples
        .iter()
        .map(|e| (s(e, "id").to_string(), s(e, "expected").to_string()))
        .collect();
    let behaviors = examples
        .iter()
        .filter_map(|e| {
            let b = e.get("behavior")?;
            Some((
                s(e, "id").to_string(),
                [
                    s(b, "who").to_string(),
                    s(b, "does").to_string(),
                    s(b, "then").to_string(),
                ],
            ))
        })
        .collect();
    let structure = vec![
        ("inputs", array(skill, "/inputs")),
        ("steps", array(skill, "/steps")),
        ("edge_cases", array(skill, "/edge_cases")),
        ("outputs", array(skill, "/outputs")),
        ("does", array(skill, "/role/does")),
        ("does_not", array(skill, "/role/does_not")),
        ("references", array(skill, "/references")),
    ];
    let mut not_approvable = Vec::new();
    for (pointer, label) in [
        ("/purpose/who", "purpose.who（誰が）"),
        ("/purpose/gets", "purpose.gets（何を得るか）"),
    ] {
        let t = text(skill, pointer);
        if t.trim().is_empty() || t == UNKNOWN {
            not_approvable.push(format!(
                "{label}が分からない。答えてもらうまで、下書きとして残し、配らない"
            ));
        }
    }
    let skill_json = format!("{}/skill.json", trimmed(folder));
    let delegate = vec![
        Delegate {
            id: "a",
            what: "意味の rule の判定".to_string(),
            how: format!(
                "書いたときとは別の会話で、このコマンドの手順を受け取って判定し、sas validate {skill_json} --answer で渡す"
            ),
            command: Some(format!("sas validate {skill_json} --semantic")),
            examples: Vec::new(),
        },
        Delegate {
            id: "b",
            what: "例を動かす".to_string(),
            how: format!(
                "別の会話に {}/SKILL.md を読ませ、例ごとに given を手で用意して input で頼み、出たものを expected と比べる",
                trimmed(folder)
            ),
            command: None,
            examples: examples
                .iter()
                .map(|e| {
                    [
                        s(e, "id").to_string(),
                        s(e, "given").to_string(),
                        s(e, "input").to_string(),
                        s(e, "expected").to_string(),
                    ]
                })
                .collect(),
        },
    ];
    Approval {
        name: text(skill, "/name"),
        description: text(skill, "/description"),
        images,
        behaviors,
        structure,
        guesses: guesses(skill),
        not_approvable,
        delegate,
    }
}
