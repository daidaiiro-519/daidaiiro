//! 回を進める。前の回の記録と、いまのボードを欄ごとに比べ、変わったところを理由と一緒に書く。
//! 前の回への回答のうち、承認は論点を決着させ、差し戻しの言葉は変わったところの理由になる。
//! 比べる相手は、いちばん新しい回の記録である（最初の回ではない）。

use crate::domain::align::counts;
use crate::domain::board::{list, round, text, topics};
use serde_json::{json, Map, Value};

/// 回を進めた結果。
#[derive(Debug, Clone, PartialEq)]
pub struct Advanced {
    /// 進めたあとのボード
    pub board: Value,
    /// 新しい回の記録（rounds/<回>.json に書く）
    pub record: Value,
    /// 新しい回の番号
    pub round: u64,
    /// 変わったところの数
    pub changes: usize,
}

/// 比べる欄と、その欄の名前（画面の言葉は描画の側が持つ）。
const PARTS: [&str; 7] = [
    "question",
    "answer",
    "details",
    "images",
    "assumptions",
    "checks",
    "grounds",
];

/// 論点ごとに、前の回への回答のうちいちばん新しいもの。
fn latest_answers<'a>(answers: &'a [Value], board: &str, round: u64) -> Map<String, Value> {
    let mut sorted: Vec<&'a Value> = answers
        .iter()
        .filter(|answer| text(answer, "board") == board)
        .filter(|answer| answer.get("round").and_then(Value::as_u64) == Some(round))
        .collect();
    sorted.sort_by(|a, b| {
        text(a, "at")
            .cmp(text(b, "at"))
            .then(text(a, "id").cmp(text(b, "id")))
    });
    let mut out = Map::new();
    for answer in sorted {
        out.insert(text(answer, "topic").to_owned(), answer.clone());
    }
    out
}

/// 前の回で承認された論点か。
fn approved(decided: &Map<String, Value>, topic: &str) -> bool {
    decided
        .get(topic)
        .is_some_and(|answer| text(answer, "decision") == "approve")
}

/// 差し戻しの言葉。どこが違うかと理由を、利用者の言葉のままつなぐ。
fn reason_of(answer: Option<&Value>) -> Option<String> {
    let answer = answer?;
    if text(answer, "decision") != "return" {
        return None;
    }
    let (place, why) = (text(answer, "where"), text(answer, "reason"));
    Some(match (place.is_empty(), why.is_empty()) {
        (false, false) => format!("「{place}」{why}"),
        (false, true) => format!("「{place}」"),
        (true, false) => why.to_owned(),
        (true, true) => return None,
    })
}

fn part_value(topic: &Value, part: &str) -> Value {
    match part {
        "answer" => topic
            .get("answer")
            .and_then(|a| a.get("text"))
            .cloned()
            .unwrap_or(Value::Null),
        "details" => topic
            .get("answer")
            .and_then(|a| a.get("details"))
            .cloned()
            .unwrap_or(json!([])),
        "question" => topic.get(part).cloned().unwrap_or(Value::Null),
        _ => Value::Array(list(topic, part).to_vec()),
    }
}

/// 欄1つの変わり方の1文。変わっていなければ None。
fn summary(part: &str, before: &Value, after: &Value) -> Option<String> {
    if before == after {
        return None;
    }
    let name = match part {
        "question" => return Some("問いを書き直した".to_owned()),
        "answer" => return Some("答えの1文を書き直した".to_owned()),
        "details" => "詳しい決まり",
        "images" => "完成イメージ",
        "assumptions" => "置いた前提",
        "checks" => "確かめてほしい箇所",
        _ => "根拠",
    };
    let empty = Vec::new();
    let a = before.as_array().unwrap_or(&empty);
    let b = after.as_array().unwrap_or(&empty);
    let (added, removed) = counts(a, b);
    Some(match (added, removed) {
        (0, 0) => format!("{name}の並びを変えた"),
        (n, 0) => format!("{name}を{n}件足した"),
        (0, n) => format!("{name}を{n}件消した"),
        (n, m) if n == m => format!("{name}を{n}件書き直した"),
        (n, m) => format!("{name}を{n}件足して{m}件消した"),
    })
}

fn change(topic: &str, part: &str, summary: String, reason: Option<String>) -> Value {
    let mut item = json!({"topic": topic, "part": part, "summary": summary});
    if let Some(reason) = reason {
        item["reason"] = Value::String(reason);
    }
    item
}

/// 前の回の記録 previous（無ければ最初の回）と、前の回への回答 answers から、いまのボード current を次の回へ進める。
pub fn advance(
    current: &Value,
    previous: Option<&Value>,
    answers: &[Value],
    today: &str,
    now: &str,
) -> Result<Advanced, String> {
    let id = text(current, "id").to_owned();
    if id.is_empty() {
        return Err("ボードに id が無い".to_owned());
    }
    let prev_snapshot = previous.and_then(|record| record.get("snapshot"));
    let prev_round = previous
        .and_then(|record| record.get("round"))
        .and_then(Value::as_u64);
    let next = match prev_round {
        Some(n) => n + 1,
        None => round(current).max(1),
    };
    let mut board = current.clone();
    let decided = prev_round
        .map(|n| latest_answers(answers, &id, n))
        .unwrap_or_default();

    // 承認された論点を決着させる
    let mut settled = Vec::new();
    if let Some(items) = board.get_mut("topics").and_then(Value::as_array_mut) {
        for topic in items.iter_mut() {
            let topic_id = text(topic, "id").to_owned();
            if approved(&decided, &topic_id) && text(topic, "status") == "open" {
                topic["status"] = json!("settled");
                settled.push(topic_id);
            }
        }
    }

    // 前の回と欄ごとに比べる
    let mut changes = Vec::new();
    if let Some(before) = prev_snapshot {
        for topic in topics(&board) {
            let topic_id = text(topic, "id");
            let reason = reason_of(decided.get(topic_id));
            let Some(old) = topics(before).iter().find(|t| text(t, "id") == topic_id) else {
                changes.push(change(topic_id, "added", "論点を足した".to_owned(), None));
                continue;
            };
            for part in PARTS {
                if let Some(line) = summary(part, &part_value(old, part), &part_value(topic, part))
                {
                    changes.push(change(topic_id, part, line, reason.clone()));
                }
            }
            if list(old, "rejected") != list(topic, "rejected") {
                let (added, removed) = counts(list(old, "rejected"), list(topic, "rejected"));
                let line = match (added, removed) {
                    (n, 0) => format!("採らなかった案を{n}件足した"),
                    (0, n) => format!("採らなかった案を{n}件消した"),
                    _ => "採らなかった案を書き直した".to_owned(),
                };
                changes.push(change(topic_id, "rejected", line, reason.clone()));
            }
            if text(old, "status") != text(topic, "status") {
                let line = match text(topic, "status") {
                    "settled"
                        if settled.iter().any(|s| s == topic_id)
                            || approved(&decided, topic_id) =>
                    {
                        "承認されて決着した"
                    }
                    "settled" => "決着にした",
                    "open" => "回答待ちにした",
                    _ => "前提待ちにした",
                };
                changes.push(change(topic_id, "status", line.to_owned(), None));
            }
        }
        for old in topics(before) {
            let topic_id = text(old, "id");
            if !topics(&board).iter().any(|t| text(t, "id") == topic_id) {
                changes.push(change(topic_id, "status", "論点を外した".to_owned(), None));
            }
        }
    }

    // 状態 ・ 並び ・ 記録
    let open: Vec<String> = topics(&board)
        .iter()
        .filter(|t| text(t, "status") == "open")
        .map(|t| text(t, "id").to_owned())
        .collect();
    let queue: Vec<Value> = list(&board, "queue")
        .iter()
        .filter(|q| open.iter().any(|o| o == text(q, "topic")))
        .cloned()
        .collect();
    let line = if prev_round.is_some() {
        format!("ボードを更新した（変わったところ {}か所）", changes.len())
    } else {
        "ボードを出した".to_owned()
    };
    let mut log = vec![json!({"date": today, "text": line})];
    log.extend(list(&board, "log").iter().cloned());
    let count = changes.len();
    let object = board.as_object_mut().ok_or("ボードがオブジェクトでない")?;
    object.remove("status");
    object.insert("round".to_owned(), json!(next));
    object.insert("opened".to_owned(), json!(today));
    object.insert("queue".to_owned(), Value::Array(queue));
    object.insert("changes".to_owned(), Value::Array(changes));
    object.insert("log".to_owned(), Value::Array(log));
    let record = json!({
        "$schema": "brainstorming/round@v1",
        "kind": "round",
        "id": next.to_string(),
        "board": id,
        "round": next,
        "at": now,
        "snapshot": board,
    });
    Ok(Advanced {
        board,
        record,
        round: next,
        changes: count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn topic(id: &str, status: &str, answer: &str, details: &[&str]) -> Value {
        json!({"id": id, "name": id, "status": status, "question": "q", "answer": {"text": answer, "details": details}})
    }

    fn board(round: u64, topics: Vec<Value>) -> Value {
        json!({"$schema": "brainstorming/board@v1", "kind": "board", "id": "b", "title": "t",
               "round": round, "queue": [{"topic": "Q1", "why": "w"}], "topics": topics})
    }

    fn record(board: &Value) -> Value {
        json!({"round": round(board), "snapshot": board})
    }

    fn answer(topic: &str, round: u64, decision: &str, place: &str, reason: &str) -> Value {
        json!({"board": "b", "round": round, "topic": topic, "decision": decision, "where": place, "reason": reason, "at": "2026-10-17T00:00:00Z", "id": format!("{round}-{topic}-1")})
    }

    fn parts(done: &Advanced) -> Vec<(String, String)> {
        list(&done.board, "changes")
            .iter()
            .map(|c| (text(c, "topic").to_owned(), text(c, "part").to_owned()))
            .collect()
    }

    #[test]
    fn the_first_round_has_no_changes_and_keeps_its_number() {
        let first = board(1, vec![topic("Q1", "open", "a", &[])]);
        let done = advance(&first, None, &[], "2026-10-17", "2026-10-17T01:00:00Z").unwrap();
        assert_eq!((done.round, done.changes), (1, 0));
        assert_eq!(done.board["opened"], "2026-10-17");
        assert_eq!(done.record["id"], "1");
        assert_eq!(done.board["log"][0]["text"], "ボードを出した");
    }

    #[test]
    fn advancing_without_edits_or_answers_shows_no_changes() {
        let first = board(1, vec![topic("Q1", "open", "a", &["d"])]);
        let done = advance(&first, Some(&record(&first)), &[], "d", "t").unwrap();
        assert_eq!((done.round, done.changes), (2, 0));
    }

    #[test]
    fn only_the_returned_part_appears_with_the_users_words() {
        let first = board(
            1,
            vec![
                topic("Q1", "open", "a", &["d1", "d2"]),
                topic("Q2", "open", "b", &[]),
            ],
        );
        let mut edited = first.clone();
        edited["topics"][0]["answer"]["text"] = json!("a2");
        let answers = [answer("Q1", 1, "return", "答えの1文", "拡張子は外したい")];
        let done = advance(&edited, Some(&record(&first)), &answers, "d", "t").unwrap();
        assert_eq!(parts(&done), vec![("Q1".to_owned(), "answer".to_owned())]);
        assert_eq!(
            done.board["changes"][0]["reason"],
            "「答えの1文」拡張子は外したい"
        );
    }

    #[test]
    fn adding_one_detail_counts_one_and_does_not_mark_the_others() {
        let first = board(1, vec![topic("Q1", "open", "a", &["d1", "d2"])]);
        let mut edited = first.clone();
        edited["topics"][0]["answer"]["details"] = json!(["d0", "d1", "d2"]);
        let done = advance(&edited, Some(&record(&first)), &[], "d", "t").unwrap();
        assert_eq!(
            done.board["changes"][0]["summary"],
            "詳しい決まりを1件足した"
        );
        assert_eq!(done.changes, 1);
    }

    #[test]
    fn approval_settles_the_topic_and_empties_the_board_when_all_are_settled() {
        let first = board(1, vec![topic("Q1", "open", "a", &[])]);
        let answers = [answer("Q1", 1, "approve", "", "")];
        let done = advance(&first, Some(&record(&first)), &answers, "d", "t").unwrap();
        assert_eq!(done.board["topics"][0]["status"], "settled");
        assert_eq!(done.board.get("status"), None);
        assert_eq!(done.board["queue"], json!([]));
        assert_eq!(done.board["changes"][0]["summary"], "承認されて決着した");
    }

    #[test]
    fn a_topic_settled_by_the_writer_after_approval_is_reported_as_approved() {
        let first = board(1, vec![topic("Q1", "open", "a", &[])]);
        let mut edited = first.clone();
        edited["topics"][0]["status"] = json!("settled");
        let answers = [answer("Q1", 1, "approve", "", "")];
        let done = advance(&edited, Some(&record(&first)), &answers, "d", "t").unwrap();
        assert_eq!(done.board["changes"][0]["summary"], "承認されて決着した");
    }

    #[test]
    fn answers_to_other_rounds_and_boards_are_ignored() {
        let first = board(2, vec![topic("Q1", "open", "a", &[])]);
        let prev = json!({"round": 2, "snapshot": first});
        let answers = [
            answer("Q1", 1, "approve", "", ""),
            json!({"board": "other", "round": 2, "topic": "Q1", "decision": "approve", "at": "x", "id": "2-Q1-1"}),
        ];
        let done = advance(&first, Some(&prev), &answers, "d", "t").unwrap();
        assert_eq!(done.board["topics"][0]["status"], "open");
        assert_eq!(done.round, 3);
    }

    #[test]
    fn it_compares_with_the_latest_round_not_the_first() {
        let r1 = board(1, vec![topic("Q1", "open", "a", &[])]);
        let mut r2 = r1.clone();
        r2["topics"][0]["answer"]["text"] = json!("b");
        r2["round"] = json!(2);
        let done = advance(&r2, Some(&record(&r2)), &[], "d", "t").unwrap();
        assert_eq!(done.changes, 0);
    }

    #[test]
    fn added_and_removed_topics_are_listed() {
        let first = board(
            1,
            vec![
                topic("Q1", "open", "a", &[]),
                topic("Q2", "waiting", "b", &[]),
            ],
        );
        let edited = board(
            1,
            vec![
                topic("Q1", "open", "a", &[]),
                topic("Q3", "waiting", "c", &[]),
            ],
        );
        let done = advance(&edited, Some(&record(&first)), &[], "d", "t").unwrap();
        assert_eq!(
            parts(&done),
            vec![
                ("Q3".to_owned(), "added".to_owned()),
                ("Q2".to_owned(), "status".to_owned())
            ]
        );
    }

    #[test]
    fn a_board_without_id_is_refused() {
        assert!(advance(&json!({"topics": []}), None, &[], "d", "t").is_err());
    }
}
