// SPDX-License-Identifier: MIT
//! 置き場所の作成を、事例で検証する。**雛形の本文を references の JSON から取り出し、そのまま置く。**
//!
//!     cargo test -p bb_business_logic

use std::path::PathBuf;

use bb_business_logic::init;
use serde_json::Value;

fn references() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../references")
}

fn text_of(id: &str) -> String {
    let body = std::fs::read_to_string(references().join("init-files.json")).expect("読める");
    let data: Value = serde_json::from_str(&body).expect("JSON である");
    data["items"]
        .as_array()
        .expect("items が在る")
        .iter()
        .find(|x| x["id"] == id)
        .and_then(|x| x["text"].as_str())
        .expect("本文が在る")
        .to_owned()
}

#[test]
fn init_places_the_texts_held_in_references() {
    let parent = std::env::temp_dir().join(format!("bb-init-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&parent);
    let dir = parent.display().to_string();
    init::create(&references(), "tameshi", &dir, "試しの題").expect("作れる");

    let line = "| `tameshi/` | 試しの題 | （未発行） | （未複製） |\n";
    let index = std::fs::read_to_string(parent.join("README.md")).expect("索引が在る");
    assert_eq!(
        index,
        text_of("index") + line,
        "索引は本文に1行を足した姿である"
    );

    let sources = std::fs::read_to_string(parent.join("tameshi/sources/README.md")).expect("在る");
    assert_eq!(
        sources,
        text_of("sources"),
        "取り直し方の本文をそのまま置く"
    );

    let board = std::fs::read_to_string(parent.join("tameshi/board.json")).expect("在る");
    let d: Value = serde_json::from_str(&board).expect("JSON である");
    assert_eq!(
        d["$schema"], "../../.claude/skills/brainstorming-board/references/board.schema.json",
        "置いた board.json は、置き場所からスキーマを指す"
    );
    assert_eq!(d["title"], "試しの題");
    assert_eq!(d["board"], "tameshi");

    // 2回目は作り直さない ── 上書きすると、書いた論点が消える
    assert!(init::create(&references(), "tameshi", &dir, "").is_err());
    let _ = std::fs::remove_dir_all(&parent);
}
