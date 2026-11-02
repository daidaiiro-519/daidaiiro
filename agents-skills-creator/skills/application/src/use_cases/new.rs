//! UC1 作るの new：Skill を新しく作る手順を返す。creator は AI を起動しないので、打ったセッションがこの手順で作る人に
//! 問1〜問3 を1往復に1問ずつ聞き、別の場面2つに「同じ ・ 違う」を聞き、答えから skill.json の値を組んで SAS に書かせ、
//! build → check → confirm を打つ。何も書かない。
//!
//! 値を組む決まりは、Concrete（published-language/v1/skill.schema.json）の項目ごとの説明 ・ 読み方 ・ rule を写す。
//! 作る人の答えから入れるのは purpose と role だけで、ほかは導く（confirm で推測として見せる）。

use crate::use_cases::confirm::UNKNOWN;
use serde_json::{Value, json};
use skills_domain::services::derivation::derived_from;

/// Skill の項目の Concrete（この文脈の公開された言葉）。
pub const CONCRETE: &str = include_str!("../../../published-language/v1/skill.schema.json");

/// Concrete の $id。
pub const CONCRETE_ID: &str = "urn:agents-skills-creator:skill:v1";

/// 手順の形の $id。
pub const PROCEDURE: &str = "urn:agents-skills-creator:skills:new-procedure:v1";

/// 項目ごとに、答えから入れるか導くかと、何から入れる ・ 導くか。導く項目は domain の1つの表（derivation）から引く。
fn source(field: &str) -> (&'static str, &'static str) {
    match field {
        "purpose" => ("answer", "問1（who ・ when）と問2（gets）の答え"),
        "role" => (
            "answer",
            "問1（does）と問3（does_not。AI が挙げた隣の仕事の候補から選んだもの）の答え",
        ),
        other => (
            "derive",
            derived_from(other).unwrap_or("purpose と role から"),
        ),
    }
}

/// Concrete の項目ごとの、値を組む決まり。
fn compose(concrete: &Value) -> Vec<Value> {
    concrete
        .get("properties")
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
        .filter(|(name, _)| name.as_str() != "$schema")
        .map(|(name, f)| {
            let (src, from) = source(name);
            let mut v = json!({"field": format!("/{name}"), "source": src, "from": from});
            for k in ["title", "description"] {
                if let Some(t) = f.get(k) {
                    v[k] = t.clone();
                }
            }
            for k in ["meaning", "when"] {
                if let Some(t) = f.pointer(&format!("/x-prompt/{k}")) {
                    v[k] = t.clone();
                }
            }
            let rules: Vec<Value> = f
                .pointer("/x-prompt/rules")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .map(|r| json!({"id": r.get("id").cloned().unwrap_or_default(), "rule": r.get("rule").cloned().unwrap_or_default()}))
                .collect();
            v["rules"] = Value::Array(rules);
            v
        })
        .collect()
}

/// Skill を作る手順を返す。place は Skill を置くフォルダの親（無ければ埋める所のまま）。
#[must_use]
pub fn new_procedure(place: Option<&str>) -> Value {
    let concrete: Value = serde_json::from_str(CONCRETE).unwrap_or_default();
    let place = place
        .map(|p| p.trim_end_matches('/').to_string())
        .unwrap_or_else(|| "<Skill の置き場所>".to_string());
    let folder = format!("{place}/<name>");
    json!({
        "$schema": PROCEDURE,
        "kind": "procedure",
        "questions": [
            {"id": "q1",
             "ask": "させたいことを、最近それを手でした場面1つで教えてください（いつ ・ 誰が ・ 何をしたか）。",
             "fills": ["/purpose/who", "/purpose/when", "/role/does"],
             "how": "場面から、使う人（who）・ 使う場面（when）・ 受け持つこと（does）を取る。この場面は examples の最初の例（from「作るときの場面」）にもする"},
            {"id": "q2",
             "ask": "終わったとき、手元に何があれば成功ですか。",
             "fills": ["/purpose/gets"],
             "how": "答えを、使い終えたときに手元にある状態として書く。completion_criteria の根拠（from「問2 の答え」）にもする"},
            {"id": "q3",
             "ask": "似ているけれど、させないことを選んでください。",
             "fills": ["/role/does_not"],
             "how": "AI が、取り違えやすい隣の仕事の候補を3つほど挙げ、作る人が選ぶ。選んだものを does_not に入れ、from を「問3 の答え」にする"}
        ],
        "scenes": {
            "count": 2,
            "ask": "問いのあと、同じ目的のまま別の場面を2つ出し、「同じ」「違う」だけで答えてもらう",
            "same": "「同じ」の場面は examples に足す（from「別の場面で「同じ」」）",
            "different": "「違う」の場面は examples に入れない。理由が言われたら、受け持たないことの候補として確認に出す"
        },
        "unknown": format!("どの問いも「{UNKNOWN}」を選べる。会話を止めず、その所に「{UNKNOWN}」と入れて進める（confirm で推測の一覧の先頭に出る）。目的の who と gets が{UNKNOWN}ままなら、confirm は承認を受け付けない"),
        "compose": compose(&concrete),
        "write": {"command": format!("sas create {folder}/skill.json --concrete {CONCRETE_ID} --value <JSON>")},
        "then": [
            format!("agents-skills-creator build {folder}"),
            format!("agents-skills-creator check {folder}"),
            format!("agents-skills-creator confirm {folder}")
        ],
        "constraints": [
            "1往復に1問だけ聞く",
            "会話に既に出ていることは聞き直さない",
            "name は聞かず、AI が付けて確認に出す",
            "purpose と role のほかの項目は、答えから導き、確認で推測として見せる",
            "skill.json は書かず、write の command で SAS に書かせる",
            "creator は AI を起動しない。意味の判定と例を動かすのは、confirm の delegate のとおり別の会話に任せる",
            "承認するか「〇〇は違う、△△」の一言を受け取るまで、配らない"
        ]
    })
}
