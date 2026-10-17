//! 回答を受ける。画面から来た値を確かめ、回答のインスタンス（brainstorming/answer@v1）にする。
//! 差し戻しは「どこが違うか」が無ければ受けない。いまの回のいま答える論点にだけ答えられる。

use crate::domain::board::{round, text, topic};
use serde_json::{json, Value};

/// 画面から来た回答を確かめ、インスタンスにする。existing は、すでにある回答の id の一覧。
pub fn accept(
    board: &Value,
    input: &Value,
    existing: &[String],
    now: &str,
) -> Result<Value, String> {
    let board_id = text(board, "id");
    if text(input, "board") != board_id {
        return Err("回答のボードが違う".to_owned());
    }
    let answered = input.get("round").and_then(Value::as_u64).unwrap_or(0);
    if answered != round(board) {
        return Err(format!(
            "第{answered}回への回答は受けない（いまは第{}回）",
            round(board)
        ));
    }
    let topic_id = text(input, "topic");
    let target = topic(board, topic_id).ok_or(format!("論点 {topic_id} が無い"))?;
    if text(target, "status") != "open" {
        return Err(format!("{topic_id} は いま答える論点でない"));
    }
    let decision = text(input, "decision");
    if !matches!(decision, "approve" | "return" | "hold") {
        return Err(format!("回答の値が違う：{decision}"));
    }
    let field = |key: &str| text(input, key).trim().to_owned();
    if decision == "return" && field("where").is_empty() {
        return Err("差し戻すときは、どこが違うかを書く".to_owned());
    }
    let prefix = format!("{answered}-{topic_id}-");
    let seq = existing.iter().filter(|id| id.starts_with(&prefix)).count() + 1;
    let mut out = json!({
        "$schema": "brainstorming/answer@v1",
        "kind": "answer",
        "id": format!("{prefix}{seq}"),
        "board": board_id,
        "round": answered,
        "topic": topic_id,
        "decision": decision,
    });
    for key in ["where", "reason", "note"] {
        if !field(key).is_empty() {
            out[key] = json!(field(key));
        }
    }
    out["at"] = json!(now);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn board() -> Value {
        json!({"id": "b", "round": 2, "topics": [{"id": "Q1", "status": "open"}, {"id": "Q2", "status": "settled"}]})
    }

    fn input(topic: &str, round: u64, decision: &str, place: &str) -> Value {
        json!({"board": "b", "round": round, "topic": topic, "decision": decision, "where": place, "reason": " 理由 "})
    }

    #[test]
    fn an_approval_becomes_an_answer_with_the_next_sequence_number() {
        let made = accept(
            &board(),
            &input("Q1", 2, "approve", ""),
            &["2-Q1-1".to_owned()],
            "T",
        )
        .unwrap();
        assert_eq!(made["id"], "2-Q1-2");
        assert_eq!(made["reason"], "理由");
        assert_eq!(made.get("where"), None);
        assert_eq!(made["at"], "T");
    }

    #[test]
    fn broken_answers_are_refused_one_reason_each() {
        let cases = [
            (
                input("Q1", 2, "return", "  "),
                "差し戻すときは、どこが違うかを書く",
            ),
            (input("Q2", 2, "approve", ""), "Q2 は いま答える論点でない"),
            (input("Q9", 2, "approve", ""), "論点 Q9 が無い"),
            (
                input("Q1", 1, "approve", ""),
                "第1回への回答は受けない（いまは第2回）",
            ),
            (input("Q1", 2, "maybe", ""), "回答の値が違う：maybe"),
            (
                json!({"board": "x", "round": 2, "topic": "Q1", "decision": "approve"}),
                "回答のボードが違う",
            ),
        ];
        let got: Vec<Result<Value, String>> = cases
            .iter()
            .map(|(i, _)| accept(&board(), i, &[], "T"))
            .collect();
        let want: Vec<Result<Value, String>> =
            cases.iter().map(|(_, w)| Err((*w).to_owned())).collect();
        assert_eq!(got, want);
    }

    #[test]
    fn a_return_with_a_place_is_accepted() {
        let made = accept(&board(), &input("Q1", 2, "return", "答えの1文"), &[], "T").unwrap();
        assert_eq!(
            (made["decision"].as_str(), made["where"].as_str()),
            (Some("return"), Some("答えの1文"))
        );
    }
}
