//! ボードを出す前に、機械で見られる決まりだけを検査する。**見つけるが、直さない。**
//! 形（欄の有無 ・ 値の種類 ・ 参照の解決）は基盤の check が見るので、ここでは見ない。
//!
//! | 何を見るか | 何が起きたか |
//! |---|---|
//! | 一度に開く論点が多すぎないか | 8件を同時に出し、どれを確認すればよいか判定できなくなった |
//! | いま見る論点を示しているか | 順番の管理を承認する側へ渡した |
//! | 答えに完成イメージが在るか | 図も実例も無いまま承認を求めた |
//! | 宣言されていない依存が無いか | 答えの本文だけが他の論点を前提にし、根拠の欄に出てこなかった |
//! | 試す相手が在るか | 下流にも外の作業にも使われない答えを、承認へ出そうとした |
//! | 1文が長すぎないか | 1文に主張を詰め込み、読み手が分けて読むことになった |

use serde_json::Value;

/// 一度に開ける論点の数。
const OPEN_MAX: usize = 5;
/// 1文の上限（字数）。
const SENTENCE_MAX: usize = 120;

fn text<'value>(value: &'value Value, key: &str) -> &'value str {
    value.get(key).and_then(Value::as_str).unwrap_or_default()
}

fn list<'value>(value: &'value Value, key: &str) -> &'value [Value] {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
}

/// 文の中で指している論点（「論点N」と「QN」）の番号。
fn referred(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    for marker in ["論点", "Q"] {
        let mut rest = body;
        while let Some(index) = rest.find(marker) {
            let tail = &rest[index + marker.len()..];
            let digits: String = tail.chars().take_while(char::is_ascii_digit).collect();
            if !digits.is_empty() && !out.contains(&digits) {
                out.push(digits);
            }
            rest = tail;
        }
    }
    out
}

/// 1文ずつに分ける（句点で区切る）。「」の中の引用は1文として数えない。
fn sentences(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    let (mut current, mut depth) = (String::new(), 0usize);
    for character in body.chars() {
        match character {
            '「' => depth += 1,
            '」' => depth = depth.saturating_sub(1),
            _ => {}
        }
        current.push(character);
        if character == '。' && depth == 0 {
            out.push(std::mem::take(&mut current));
        }
    }
    if !current.trim().is_empty() {
        out.push(current);
    }
    out
}

/// ボードを検査する。見つかったことを、見つけた順に返す。
pub fn inspect(board: &Value) -> Vec<String> {
    let mut out = Vec::new();
    let topics = list(board, "topics");
    let open: Vec<&Value> = topics
        .iter()
        .filter(|topic| text(topic, "status") == "open")
        .collect();
    if open.len() > OPEN_MAX {
        out.push(format!(
            "開いている論点が{}件ある（{OPEN_MAX}件まで）── どれを確認すればよいか判定できなくなる",
            open.len()
        ));
    }
    if !open.is_empty() && list(board, "queue").is_empty() {
        out.push("開いている論点があるのに、いま見る論点（queue）が無い ── 順番の管理を承認する側へ渡すことになる".to_owned());
    }
    // 下流の論点の根拠に出てくる論点は、下流で使われている
    let used: Vec<String> = topics
        .iter()
        .flat_map(|topic| {
            list(topic, "grounds")
                .iter()
                .flat_map(|ground| referred(text(ground, "source")))
        })
        .collect();
    for topic in topics {
        let id = text(topic, "id");
        let status = text(topic, "status");
        let answer = topic.get("answer").cloned().unwrap_or(Value::Null);
        if status != "waiting" && list(topic, "images").is_empty() {
            out.push(format!("{id}：答えに完成イメージが無い"));
        }
        // 答えの本文が他の論点を前提にしているなら、根拠の出どころに書く
        let mut body = text(&answer, "text").to_owned();
        for detail in list(&answer, "details") {
            body.push_str(detail.as_str().unwrap_or_default());
        }
        let own = id.trim_start_matches('Q');
        let declared: Vec<String> = list(topic, "grounds")
            .iter()
            .flat_map(|ground| referred(text(ground, "source")))
            .collect();
        // 根拠でこの論点を前提にしている論点は下流であり、答えの中で指しても依存ではない（任せる先である）
        let downstream: Vec<String> = topics
            .iter()
            .filter(|other| {
                list(other, "grounds").iter().any(|ground| {
                    referred(text(ground, "source"))
                        .iter()
                        .any(|number| number == own)
                })
            })
            .map(|other| text(other, "id").trim_start_matches('Q').to_owned())
            .collect();
        for number in referred(&body) {
            if number != own && !declared.contains(&number) && !downstream.contains(&number) {
                out.push(format!(
                    "{id}：答えが論点{number}を前提にしているが、根拠の出どころに出てこない"
                ));
            }
        }
        // 開いている論点は、下流の論点か外の作業で試される
        if status == "open" && !used.iter().any(|number| number == own) {
            let outer = topic
                .get("verification")
                .map(|verification| list(verification, "out_of_scope"))
                .unwrap_or_default()
                .iter()
                .any(|item| text(item, "item").contains("外の作業"));
            if !outer {
                out.push(format!(
                    "{id}：下流のどの論点にも使われず、外の作業で試す行き先も書かれていない"
                ));
            }
        }
        let mut cells: Vec<String> = vec![text(&answer, "text").to_owned()];
        cells.extend(
            list(&answer, "details")
                .iter()
                .map(|detail| detail.as_str().unwrap_or_default().to_owned()),
        );
        cells.extend(
            list(topic, "rejected")
                .iter()
                .map(|option| text(option, "reason").to_owned()),
        );
        for cell in cells {
            for sentence in sentences(&cell) {
                let length = sentence.chars().count();
                if length > SENTENCE_MAX {
                    let head: String = sentence.chars().take(20).collect();
                    out.push(format!(
                        "{id}：1文が{length}字ある（{SENTENCE_MAX}字まで）── {head}…"
                    ));
                }
            }
        }
    }
    out
}
