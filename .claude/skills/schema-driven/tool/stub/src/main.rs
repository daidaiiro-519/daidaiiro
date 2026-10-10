//! 固有のツールの見本。1つ目の引数で振る舞いを選び、境界の違反を1つずつ起こす。

use serde_json::{json, Value};
use std::io::Read;

fn main() {
    let mut input = String::new();
    let _ = std::io::stdin().read_to_string(&mut input);
    let request: Value = serde_json::from_str(&input).unwrap_or(Value::Null);
    match std::env::args().nth(1).as_deref() {
        // 受け取った入力と、見えた環境変数をそのまま返す
        Some("echo") => println!(
            "{}",
            json!({"ok": true, "request": request,
                   "allowed": std::env::var("SD_ALLOWED").ok(),
                   "secret": std::env::var("SD_SECRET").ok()})
        ),
        Some("fail") => {
            println!("{}", json!({"ok": false, "reason": "見本の失敗"}));
            std::process::exit(1);
        }
        Some("slow") => {
            std::thread::sleep(std::time::Duration::from_secs(5));
            println!("{}", json!({"ok": true}));
        }
        Some("not-json") => println!("JSON ではない出力"),
        Some("no-ok") => println!("{}", json!({"result": 1})),
        Some("big") => println!(
            "{}",
            json!({"ok": true, "data": "x".repeat(2 * 1024 * 1024)})
        ),
        Some("exit-7") => {
            println!("{}", json!({"ok": false}));
            std::process::exit(7);
        }
        _ => {
            println!("{}", json!({"ok": false, "reason": "知らない振る舞い"}));
            std::process::exit(2);
        }
    }
}
