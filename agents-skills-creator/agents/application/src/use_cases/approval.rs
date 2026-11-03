//! UC2 Agent を作るの承認の材料：完成の像（例の expected）・ 振る舞い ・ 構造の像（受け持つ範囲 ・ 呼ぶ Skill と受け渡し ・
//! 手順と分かれ道）と、別の会話に任せる手順。Skill の承認の材料と同じ考えで、agent.json から機械で作れる所まで作る。

use crate::use_cases::{strings, trimmed};
use agents_domain::services::agent_md;
use serde_json::{Value, json};

fn s<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(Value::as_str).unwrap_or_default()
}

fn items<'a>(v: &'a Value, key: &str) -> &'a [Value] {
    v.get(key)
        .and_then(Value::as_array)
        .map_or(&[], Vec::as_slice)
}

/// 承認の材料を作る。
#[must_use]
pub fn approval(folder: &str, agent: &Value) -> Value {
    let examples = items(agent, "examples");
    let role = agent.get("role").cloned().unwrap_or_default();
    let steps = items(agent, "steps");
    let calls: Vec<Value> = steps
        .iter()
        .filter(|st| s(st, "kind") == "call")
        .map(|st| {
            let mut v = json!({
                "step": st.get("id").cloned().unwrap_or_default(),
                "skills": strings(st, "calls"),
                "passes": strings(st, "passes"),
            });
            if let Some(c) = st.get("choose") {
                v["choose"] = c.clone();
            }
            v
        })
        .collect();
    let branches: Vec<Value> = items(agent, "edge_cases")
        .iter()
        .map(|e| {
            let digits: String = s(e, "id").chars().take_while(char::is_ascii_digit).collect();
            json!({"id": s(e, "id"), "at_step": digits.parse::<u64>().ok(), "when": s(e, "when"), "then": s(e, "then"), "end": s(e, "end")})
        })
        .collect();
    let agent_json = format!("{}/agent.json", trimmed(folder));
    json!({
        "name": s(agent, "name"),
        "description": s(agent, "description"),
        "images": examples.iter().map(|e| json!({"example": s(e, "id"), "expected": s(e, "expected")})).collect::<Vec<_>>(),
        "behaviors": examples.iter().filter_map(|e| {
            let b = e.get("behavior")?;
            Some(json!({"example": s(e, "id"), "who": s(b, "who"), "does": s(b, "does"), "then": s(b, "then")}))
        }).collect::<Vec<_>>(),
        "structure_image": {
            "scope": {
                "inputs": items(agent, "inputs").iter().map(|i| s(i, "name")).collect::<Vec<_>>(),
                "does": strings(&role, "does"),
                "outputs": items(agent, "outputs").iter().map(|o| s(o, "name")).collect::<Vec<_>>(),
                "outside": items(&role, "does_not").iter().map(|d| s(d, "what")).collect::<Vec<_>>(),
            },
            "calls": calls,
            "steps_and_branches": {
                "steps": steps.iter().map(|st| json!({"id": st.get("id").cloned().unwrap_or_default(), "kind": s(st, "kind"), "do": s(st, "do"), "completion": s(st, "completion")})).collect::<Vec<_>>(),
                "branches": branches,
            },
            "merge": items(agent, "rules").iter().map(|r| s(r, "rule")).collect::<Vec<_>>(),
        },
        "delegate": [
            {"id": "a", "what": "意味の rule の判定", "how": format!("書いたときとは別の会話で、このコマンドの手順を受け取って判定し、答えを、手順の answer.schema に従う JSON（hash に手順の target.hash、results に rule ごとの step ・ rule ・ pointer ・ result ・ reason。example-runs-as-expected だけは b の会話の判定を入れる）として作業場所の中のファイルに書き、semantic-agent-schema validate {agent_json} --answer <ファイル> で渡す。そのあと validate を打ち直す"), "command": format!("semantic-agent-schema validate {agent_json} --semantic")},
            {"id": "b", "what": "例を動かす", "how": format!("別の会話に {}/{} だけを読ませ、例ごとに given を手で用意して input で頼み、実際の答えを expected と比べる。その会話が、意味の rule example-runs-as-expected（pointer /examples）を判定し、reason に例ごとの実際の答えを書く。この判定は a の判定と1つの答えのファイルにまとめて semantic-agent-schema validate {agent_json} --answer <ファイル> で渡す（rule が1つでも欠けた答えは SAS が断る）。そのあと validate を打ち直す", trimmed(folder), agent_md::FILE),
             "examples": examples.iter().map(|e| json!({"id": s(e, "id"), "given": s(e, "given"), "input": s(e, "input"), "expected": s(e, "expected")})).collect::<Vec<_>>()}
        ]
    })
}
