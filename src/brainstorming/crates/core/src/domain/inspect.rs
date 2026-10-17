//! 本文を読む検査。形（スキーマ）は基盤の check が見るので、ここでは形では見られない決まりだけを見る。
//! 1文の長さ ・ 前提にした論点の出どころ ・ 下流の行き先 ・ 完成イメージの有無 ・ いま見る論点の並び。

use crate::domain::board::{list, strings, text, topics};
use crate::domain::text::{too_long, SENTENCE_MAX};
use serde_json::Value;

/// 一度に開いてよい論点の数。
pub const OPEN_MAX: usize = 5;

/// 本文の中の論点の id（Q と数字）。
fn mentioned(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let boundary = i == 0 || !chars[i - 1].is_ascii_alphanumeric();
        if chars[i] == 'Q' && boundary && chars.get(i + 1).is_some_and(char::is_ascii_digit) {
            let mut j = i + 1;
            while chars.get(j).is_some_and(char::is_ascii_digit) {
                j += 1;
            }
            if !chars.get(j).is_some_and(char::is_ascii_alphanumeric) {
                out.push(chars[i..j].iter().collect());
            }
            i = j;
        } else {
            i += 1;
        }
    }
    out
}

/// ボードを検査し、見つけたことを1件ずつ返す。空なら合格。
pub fn inspect(board: &Value) -> Vec<String> {
    let mut out = Vec::new();
    let all = topics(board);
    let ids: Vec<&str> = all.iter().map(|t| text(t, "id")).collect();
    let open: Vec<&Value> = all.iter().filter(|t| text(t, "status") == "open").collect();
    if open.len() > OPEN_MAX {
        out.push(format!(
            "回答待ちの論点が{}件ある（{OPEN_MAX}件まで）",
            open.len()
        ));
    }
    let queue: Vec<&str> = list(board, "queue")
        .iter()
        .map(|q| text(q, "topic"))
        .collect();
    if !open.is_empty() && queue.is_empty() {
        out.push("回答待ちの論点があるのに、queue に いま見る論点が無い".to_owned());
    }
    for id in &queue {
        match all.iter().find(|t| text(t, "id") == *id) {
            Some(t) if text(t, "status") != "open" => out.push(format!(
                "{id}：queue にあるが、回答待ちの論点（open）でない"
            )),
            None => out.push(format!("{id}：queue が指す論点が無い")),
            _ => {}
        }
    }
    if board.get("status").is_some() {
        out.push("ボードの status は書かない（ステータスは論点の状態と保留から導く）".to_owned());
    }
    if open.is_empty() && !all.is_empty() && all.iter().any(|t| text(t, "status") != "settled") {
        out.push("回答待ちの論点が無いのに、決着していない論点がある（前提が片付いた論点を open にする）".to_owned());
    }
    for topic in all {
        let id = text(topic, "id");
        let depends = strings(topic, "depends");
        for dep in &depends {
            if !ids.contains(&dep.as_str()) {
                out.push(format!("{id}：前提にした論点 {dep} が無い"));
            } else if text(topic, "status") == "open"
                && all
                    .iter()
                    .any(|t| text(t, "id") == dep && text(t, "status") != "settled")
            {
                out.push(format!(
                    "{id}：前提にした {dep} が決着していないのに、回答待ちの論点にしている"
                ));
            }
        }
        let downstream: Vec<&str> = all
            .iter()
            .filter(|t| strings(t, "depends").iter().any(|d| d == id))
            .map(|t| text(t, "id"))
            .collect();
        let answer = topic.get("answer").cloned().unwrap_or(Value::Null);
        let mut body = vec![text(&answer, "text").to_owned()];
        body.extend(strings(&answer, "details"));
        for other in body.iter().flat_map(|line| mentioned(line)) {
            if other != id && !depends.contains(&other) && !downstream.contains(&other.as_str()) {
                out.push(format!(
                    "{id}：答えが {other} を前提にしているが、depends に無い"
                ));
            }
        }
        let mut sentences = body.clone();
        sentences.extend(
            list(topic, "rejected")
                .iter()
                .map(|r| text(r, "reason").to_owned()),
        );
        for (length, sentence) in sentences.iter().flat_map(|line| too_long(line)) {
            let head: String = sentence.chars().take(20).collect();
            out.push(format!(
                "{id}：1文が{length}字ある（{SENTENCE_MAX}字まで）── {head}…"
            ));
        }
        if text(topic, "status") == "open" {
            for check in list(topic, "checks") {
                let mut missing: Vec<&str> = Vec::new();
                if text(check, "view").trim().is_empty() {
                    missing.push("私の見解（view）");
                }
                if list(check, "basis").is_empty() {
                    missing.push("根拠（basis）");
                }
                if text(check, "why").trim().is_empty() {
                    missing.push("決めきれない理由（why）");
                }
                if !missing.is_empty() {
                    out.push(format!(
                        "{id}：確かめてほしい箇所「{}」に、{}が無い",
                        text(check, "target"),
                        missing.join("と")
                    ));
                }
            }
        }
        let images = list(topic, "images");
        if images.len() > 1 {
            let unnamed: Vec<String> = images
                .iter()
                .enumerate()
                .filter(|(_, image)| text(image, "name").trim().is_empty())
                .map(|(n, _)| (n + 1).to_string())
                .collect();
            if !unnamed.is_empty() {
                out.push(format!(
                    "{id}：完成イメージが{}枚あるのに、{} 枚目に名前（name）が無い。タブで見分けられない",
                    images.len(),
                    unnamed.join(" ・ ")
                ));
            }
        }
        for (n, image) in images.iter().enumerate() {
            let language = text(image, "language").to_ascii_lowercase();
            if text(image, "kind") == "code"
                && matches!(language.as_str(), "text" | "plain" | "txt" | "")
            {
                out.push(format!(
                    "{id}：完成イメージ {} のコードの言語が「{language}」。ファイルの本文でないなら、図などほかの種類にする",
                    n + 1
                ));
            }
        }
        if text(topic, "status") == "open" {
            if list(topic, "images").is_empty() {
                out.push(format!("{id}：完成イメージが無い"));
            }
            let outer = topic
                .get("verification")
                .map(|v| list(v, "out_of_scope"))
                .unwrap_or(&[])
                .iter()
                .any(|item| text(item, "item").contains("外の作業"));
            if downstream.is_empty() && !outer {
                out.push(format!(
                    "{id}：下流のどの論点にも使われず、外の作業で試す行き先も書かれていない"
                ));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn ok_topic(id: &str, status: &str) -> Value {
        json!({"id": id, "name": id, "status": status, "question": "q", "answer": {"text": "a"},
               "images": [{"kind": "run", "caption": "c", "command": "x", "exit": 0}],
               "verification": {"out_of_scope": [{"item": "外の作業：試す", "treatment": "later"}]}})
    }

    fn board(topics: Vec<Value>, queue: &[&str]) -> Value {
        let queue: Vec<Value> = queue
            .iter()
            .map(|q| json!({"topic": q, "why": "w"}))
            .collect();
        json!({"id": "b", "queue": queue, "topics": topics})
    }

    #[test]
    fn a_well_formed_board_has_no_findings() {
        assert_eq!(
            inspect(&board(vec![ok_topic("Q1", "open")], &["Q1"])),
            Vec::<String>::new()
        );
    }

    #[test]
    fn two_or_more_images_need_names_to_tell_the_tabs_apart() {
        let mut topic = ok_topic("Q1", "settled");
        topic["images"] = json!([{"kind": "figure", "name": "全体"}, {"kind": "figure"}]);
        let found = inspect(&board(vec![topic.clone()], &[]));
        assert_eq!(
            found,
            vec!["Q1：完成イメージが2枚あるのに、2 枚目に名前（name）が無い。タブで見分けられない"]
        );
        topic["images"][1]["name"] = json!("担い手");
        assert!(inspect(&board(vec![topic], &[])).is_empty());
    }

    #[test]
    fn an_open_topic_says_why_it_asks_and_what_the_writer_thinks() {
        let mut topic = ok_topic("Q1", "open");
        topic["checks"] = json!([{"target": "HTTP", "ask": "これでよいか"}]);
        assert_eq!(
            inspect(&board(vec![topic.clone()], &["Q1"])),
            vec!["Q1：確かめてほしい箇所「HTTP」に、私の見解（view）と根拠（basis）と決めきれない理由（why）が無い"]
        );
        topic["checks"][0]["view"] = json!("基盤は待ち受けない");
        topic["checks"][0]["basis"] = json!(["platform-advisor の回答"]);
        assert_eq!(
            inspect(&board(vec![topic.clone()], &["Q1"])),
            vec!["Q1：確かめてほしい箇所「HTTP」に、決めきれない理由（why）が無い"]
        );
        topic["checks"][0]["why"] = json!("別の機械から呼ぶ予定は利用者しか知らない");
        assert!(inspect(&board(vec![topic.clone()], &["Q1"])).is_empty());
        let mut waiting = ok_topic("Q2", "waiting");
        waiting["checks"] = json!([{"target": "古い", "ask": "これでよいか"}]);
        assert!(inspect(&board(vec![topic, waiting], &["Q1"])).is_empty());
    }

    #[test]
    fn each_rule_reports_only_its_own_breach() {
        let mut long = ok_topic("Q1", "open");
        long["answer"]["text"] = json!(format!("{}。", "あ".repeat(130)));
        let mut no_image = ok_topic("Q1", "open");
        no_image["images"] = json!([]);
        let mut no_outer = ok_topic("Q1", "open");
        no_outer["verification"] = json!({});
        let mut hidden_dep = ok_topic("Q2", "open");
        hidden_dep["answer"]["text"] = json!("Q1 の答えを使う");
        let mut missing_dep = ok_topic("Q1", "open");
        missing_dep["depends"] = json!(["Q9"]);
        let head = "あ".repeat(20);
        let cases: [(Value, Vec<String>); 7] = [
            (
                board(vec![long], &["Q1"]),
                vec![format!("Q1：1文が131字ある（120字まで）── {head}…")],
            ),
            (
                board(vec![no_image], &["Q1"]),
                vec!["Q1：完成イメージが無い".to_owned()],
            ),
            (
                board(vec![no_outer], &["Q1"]),
                vec![
                    "Q1：下流のどの論点にも使われず、外の作業で試す行き先も書かれていない"
                        .to_owned(),
                ],
            ),
            (
                board(vec![ok_topic("Q1", "settled"), hidden_dep], &["Q2"]),
                vec!["Q2：答えが Q1 を前提にしているが、depends に無い".to_owned()],
            ),
            (
                board(vec![missing_dep], &["Q1"]),
                vec!["Q1：前提にした論点 Q9 が無い".to_owned()],
            ),
            (
                board(vec![ok_topic("Q1", "open")], &[]),
                vec!["回答待ちの論点があるのに、queue に いま見る論点が無い".to_owned()],
            ),
            (
                board(vec![ok_topic("Q1", "waiting")], &["Q1"]),
                vec![
                    "Q1：queue にあるが、回答待ちの論点（open）でない".to_owned(),
                    "回答待ちの論点が無いのに、決着していない論点がある（前提が片付いた論点を open にする）".to_owned(),
                ],
            ),
        ];
        let got: Vec<Vec<String>> = cases.iter().map(|(b, _)| inspect(b)).collect();
        let want: Vec<Vec<String>> = cases.into_iter().map(|(_, w)| w).collect();
        assert_eq!(got, want);
    }

    #[test]
    fn code_that_is_not_a_file_is_reported() {
        let mut not_code = ok_topic("Q1", "open");
        not_code["images"] =
            json!([{"kind": "code", "caption": "c", "language": "text", "text": "a  b"}]);
        let mut real_code = ok_topic("Q1", "open");
        real_code["images"] =
            json!([{"kind": "code", "caption": "c", "language": "rust", "text": "fn main() {}"}]);
        assert_eq!(
            inspect(&board(vec![not_code], &["Q1"])),
            vec!["Q1：完成イメージ 1 のコードの言語が「text」。ファイルの本文でないなら、図などほかの種類にする"]
        );
        assert_eq!(
            inspect(&board(vec![real_code], &["Q1"])),
            Vec::<String>::new()
        );
    }

    #[test]
    fn a_dependency_that_is_still_open_blocks_opening() {
        let mut q2 = ok_topic("Q2", "open");
        q2["depends"] = json!(["Q1"]);
        let found = inspect(&board(vec![ok_topic("Q1", "open"), q2], &["Q1", "Q2"]));
        assert_eq!(
            found,
            vec!["Q2：前提にした Q1 が決着していないのに、回答待ちの論点にしている"]
        );
    }

    #[test]
    fn too_many_open_topics_are_reported() {
        let many: Vec<Value> = (0..6).map(|n| ok_topic(&format!("Q{n}"), "open")).collect();
        let found = inspect(&board(many, &["Q0"]));
        assert_eq!(found, vec!["回答待ちの論点が6件ある（5件まで）"]);
    }

    #[test]
    fn mentions_are_whole_ids_only() {
        assert_eq!(mentioned("Q1 と Q12、FAQ1 と Q3a"), vec!["Q1", "Q12"]);
    }
}
