//! 状態。画面に出すステータスは、保存した事実から毎回導く。手で書く値にしない。
//!
//! 保存する事実は3つだけである。
//! - 論点を開いたこと（論点の status が open）と、決着（settled）。前提が片付いても AI があえて開かないことがあるので、開いたことは保存する
//! - ボードの保留（hold）。利用者がボード全体を止める決定で、ほかの事実からは導けない
//! - AI が動いていること（ロックのファイル）。serve と CLI のどちらから動かしたかを問わない
//!
//! ステータスは、利用者がいまできることを示すためだけに置く。
//!
//! | ステータス | 付くもの | できること |
//! |---|---|---|
//! | 回答待ち | 論点 ・ ボード | 承認か差し戻しを選び、ボードでまとめて送る |
//! | 前提待ち | 論点 | 見る |
//! | 決着 | 論点 ・ ボード | 見る |
//! | 更新中 | ボード | 待つ ・ 止める |
//! | 保留 | ボード | 見る ・ 保留を解く |

use crate::domain::board::{text, topics};
use serde_json::Value;

/// 論点のステータス。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TopicStatus {
    /// 回答待ち
    Answer,
    /// 前提待ち
    Waiting,
    /// 決着
    Done,
}

/// ボードのステータス。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoardStatus {
    /// 回答待ち
    Answer,
    /// 更新中
    Updating,
    /// 保留
    Hold,
    /// 決着
    Done,
}

impl TopicStatus {
    /// 画面とデータで使う名前。
    pub fn name(self) -> &'static str {
        match self {
            Self::Answer => "answer",
            Self::Waiting => "waiting",
            Self::Done => "done",
        }
    }
}

impl BoardStatus {
    /// 画面とデータで使う名前。
    pub fn name(self) -> &'static str {
        match self {
            Self::Answer => "answer",
            Self::Updating => "updating",
            Self::Hold => "hold",
            Self::Done => "done",
        }
    }
}

/// 論点のステータス。決着していれば決着、開いていれば回答待ち、それ以外は前提待ち。
pub fn topic_status(topic: &Value) -> TopicStatus {
    match text(topic, "status") {
        "settled" => TopicStatus::Done,
        "open" => TopicStatus::Answer,
        _ => TopicStatus::Waiting,
    }
}

/// ボードのステータス。AI が動いていれば更新中、保留なら保留、論点がすべて決着なら決着、それ以外は回答待ち。
pub fn board_status(board: &Value, updating: bool) -> BoardStatus {
    if updating {
        return BoardStatus::Updating;
    }
    if board.get("hold").is_some_and(Value::is_object) {
        return BoardStatus::Hold;
    }
    let all = topics(board);
    if !all.is_empty() && all.iter().all(|t| topic_status(t) == TopicStatus::Done) {
        return BoardStatus::Done;
    }
    BoardStatus::Answer
}

/// 回答待ちの論点の数。
pub fn open_count(board: &Value) -> usize {
    topics(board)
        .iter()
        .filter(|t| topic_status(t) == TopicStatus::Answer)
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn board(statuses: &[&str]) -> Value {
        let topics: Vec<Value> = statuses
            .iter()
            .enumerate()
            .map(|(i, s)| json!({"id": format!("Q{i}"), "status": s}))
            .collect();
        json!({"id": "b", "topics": topics})
    }

    #[test]
    fn a_topic_shows_what_can_be_done_with_it() {
        let got: Vec<TopicStatus> = ["open", "waiting", "settled"]
            .iter()
            .map(|s| topic_status(&json!({"status": s})))
            .collect();
        assert_eq!(
            got,
            [TopicStatus::Answer, TopicStatus::Waiting, TopicStatus::Done]
        );
    }

    /// 移る条件の表の、ボードの行。
    #[test]
    fn the_board_status_is_derived_from_the_facts() {
        let mut held = board(&["open"]);
        held["hold"] = json!({"date": "2026-10-17"});
        let cases = [
            // 前提待ちと回答待ちがある
            (board(&["open", "waiting"]), false, BoardStatus::Answer),
            // 送ると AI が動く（回答待ち → 更新中）
            (board(&["open", "waiting"]), true, BoardStatus::Updating),
            // AI に聞いても更新中
            (board(&["settled", "open"]), true, BoardStatus::Updating),
            // 更新が終わり、論点がすべて決着した
            (board(&["settled", "settled"]), false, BoardStatus::Done),
            // 保留にした
            (held.clone(), false, BoardStatus::Hold),
            // 保留でも、AI が動いていれば更新中
            (held, true, BoardStatus::Updating),
            // 論点が無いボードは決着にしない
            (board(&[]), false, BoardStatus::Answer),
        ];
        let got: Vec<BoardStatus> = cases.iter().map(|(b, u, _)| board_status(b, *u)).collect();
        let want: Vec<BoardStatus> = cases.iter().map(|(_, _, w)| *w).collect();
        assert_eq!(got, want);
    }

    #[test]
    fn the_open_count_counts_only_topics_to_answer() {
        assert_eq!(
            open_count(&board(&["open", "open", "waiting", "settled"])),
            2
        );
    }
}
