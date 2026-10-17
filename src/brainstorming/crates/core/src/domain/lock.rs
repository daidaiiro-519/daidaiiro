//! ボードのロック。1枚のボードで AI が動くのは、いつも1つだけにする。
//!
//! AI どうしは互いの文脈を持たないので、同時に動くと、片方の直しをもう片方が知らないまま上書きする。
//! ロックは .brainstorming/.serve/locks/<ボード>.json のファイルで持ち、どのプロセスからも見えるようにする。
//!
//! 持ち主は2種類ある。
//! - serve の依頼（送る ・ AI に聞く）。持ち主は依頼の ID で、その AI が呼ぶ brainstorming は環境変数で同じ ID を受けて通る。
//!   serve は動いているあいだ beat を書き直すので、止まった serve のロックは古くなって外れる
//! - ターミナルから呼んだ brainstorming（update ・ advance）。持ち主は cli。advance か unlock で外し、外し忘れは時間で外れる

use serde_json::{json, Value};

/// ターミナルから呼んだ brainstorming の持ち主の名前。
pub const CLI: &str = "cli";
/// serve の依頼のロックが古くなるまでの秒。serve はこれより短い間隔で beat を書き直す。
pub const SERVE_STALE: u64 = 60;
/// ターミナルのロックが古くなるまでの秒（最後の update か advance から）。
pub const CLI_STALE: u64 = 10 * 60;

/// 何のために持っているか。
pub fn what_label(what: &str) -> &'static str {
    match what {
        "send" => "送った回答をもとに更新",
        "ask" => "AI に聞く",
        "hold" => "保留の切り替え",
        _ => "ターミナルの AI が直している",
    }
}

/// ロックの中身を作る。
pub fn make(board: &str, owner: &str, what: &str, since: &str, now: u64) -> Value {
    json!({"board": board, "owner": owner, "what": what, "since": since, "beat": now})
}

/// 古くなったか。
pub fn stale(lock: &Value, now: u64) -> bool {
    let beat = lock.get("beat").and_then(Value::as_u64).unwrap_or(0);
    let limit = if lock.get("owner").and_then(Value::as_str) == Some(CLI) {
        CLI_STALE
    } else {
        SERVE_STALE
    };
    now.saturating_sub(beat) > limit
}

/// owner がこのロックの下で書いてよいか。古いロックと自分のロックなら書いてよい。
pub fn may_write(lock: Option<&Value>, owner: &str, now: u64) -> bool {
    match lock {
        None => true,
        Some(lock) => stale(lock, now) || lock.get("owner").and_then(Value::as_str) == Some(owner),
    }
}

/// 断るときの文。
pub fn refusal(lock: &Value) -> String {
    let what = what_label(lock.get("what").and_then(Value::as_str).unwrap_or(""));
    format!("このボードは更新中（{what}）。終わってから操作する")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_serve_lock_goes_stale_quickly_and_a_cli_lock_slowly() {
        let serve = make("b", "R1", "send", "t", 1000);
        let cli = make("b", CLI, "cli", "t", 1000);
        assert!(!stale(&serve, 1000 + SERVE_STALE));
        assert!(stale(&serve, 1001 + SERVE_STALE));
        assert!(!stale(&cli, 1000 + CLI_STALE));
        assert!(stale(&cli, 1001 + CLI_STALE));
    }

    #[test]
    fn only_the_owner_writes_while_the_lock_is_fresh() {
        let lock = make("b", "R1", "send", "t", 1000);
        assert!(may_write(None, "R2", 1000));
        assert!(may_write(Some(&lock), "R1", 1010));
        assert!(!may_write(Some(&lock), "R2", 1010));
        assert!(!may_write(Some(&lock), CLI, 1010));
        assert!(may_write(Some(&lock), "R2", 2000));
    }

    #[test]
    fn the_refusal_says_what_is_running() {
        let lock = make("b", "R1", "ask", "2026-10-17T03:42:00Z", 0);
        assert_eq!(
            refusal(&lock),
            "このボードは更新中（AI に聞く）。終わってから操作する"
        );
    }
}
