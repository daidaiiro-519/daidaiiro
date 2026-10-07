//! ボードに固有のツール（ExtraTools）：init ・ inspect ・ migrate ・ serve の回答の受け取り。
//! 形の検査 ・ 描画 ・ 承認は、転写した schema-driven のツールが持つ。

use brainstorming_board::serve;
use brainstorming_board::tools::BoardTools;
use schema_driven_adapters::inbound::tools::ExtraTools;
use schema_driven_adapters::outbound::{fs::FileSystem, jmespath::Jmespath};
use schema_driven_core::application::checks::Checks;
use schema_driven_core::ports::inbound::CheckUseCases;
use serde_json::{json, Map, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

const SKILL: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

fn temp(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("bb-tools-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

fn tools() -> BoardTools {
    BoardTools::new(SKILL.to_owned())
}

fn call(name: &str, args: Value) -> (i32, Value) {
    let args: Map<String, Value> = args.as_object().cloned().unwrap();
    tools().call(name, &args)
}

fn check(dir: &Path) -> Value {
    let files = Arc::new(FileSystem);
    let checks = Checks::new(files.clone(), files, Arc::new(Jmespath));
    let checked = checks.check(&dir.to_string_lossy()).unwrap();
    json!({
        "errors": checked.instances.iter().map(|i| i.errors.len()).sum::<usize>(),
        "drifts": checked.findings.iter().filter(|f| f.status.label() != "合格").count()
    })
}

fn fixture() -> Value {
    serde_json::from_str(include_str!("fixtures/board.json")).unwrap()
}

#[test]
fn init_writes_a_board_the_base_check_accepts_and_adds_an_index_row() {
    let root = temp("init");
    fs::write(root.join("README.md"), "| ボード | 題 |\n|---|---|\n").unwrap();
    let (code, out) = call(
        "init",
        json!({"name": "sample-board", "title": "試しのボード", "dir": root.to_string_lossy()}),
    );
    assert_eq!(code, 0, "{out}");
    let board = root.join("sample-board");
    assert!(board.join("answers").is_dir());
    assert_eq!(check(&board), json!({"errors": 0, "drifts": 0}));
    let index = fs::read_to_string(root.join("README.md")).unwrap();
    assert!(
        index.contains("| `sample-board/` | 試しのボード |"),
        "{index}"
    );
    // 既に在る名前は作り直さない（書いた論点が消える）
    let (code, _) = call(
        "init",
        json!({"name": "sample-board", "title": "別", "dir": root.to_string_lossy()}),
    );
    assert_eq!(code, 1);
}

fn inspect(name: &str, board: Value) -> (i32, Vec<String>) {
    let root = temp(&format!("inspect-{name}"));
    fs::write(root.join("board.json"), board.to_string()).unwrap();
    let (code, out) = call(
        "inspect",
        json!({"path": root.join("board.json").to_string_lossy()}),
    );
    let findings = out["findings"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .map(|finding| finding.as_str().unwrap_or_default().to_owned())
        .collect();
    (code, findings)
}

#[test]
fn inspect_passes_a_board_that_follows_the_rules() {
    let (code, findings) = inspect("clean", fixture());
    assert_eq!((code, findings), (0, Vec::<String>::new()));
}

#[test]
fn inspect_finds_an_open_topic_without_an_image_or_a_queue() {
    let mut board = fixture();
    board["queue"] = json!([]);
    board["topics"][4]["images"] = json!([]);
    let (code, findings) = inspect("queue", board);
    assert_eq!(code, 1);
    assert!(
        findings.iter().any(|f| f.contains("いま見る論点")),
        "{findings:?}"
    );
    assert!(
        findings
            .iter()
            .any(|f| f.contains("Q5") && f.contains("完成イメージ")),
        "{findings:?}"
    );
}

#[test]
fn inspect_finds_too_many_open_topics_and_long_sentences() {
    let mut board = fixture();
    let topics = board["topics"].as_array_mut().unwrap();
    for topic in topics.iter_mut() {
        topic["status"] = json!("open");
    }
    let mut extra = topics[0].clone();
    extra["id"] = json!("Q6");
    topics.push(extra);
    topics[0]["answer"]["details"] = json!(["あ".repeat(130)]);
    let (code, findings) = inspect("many", board);
    assert_eq!(code, 1);
    assert!(findings.iter().any(|f| f.contains("6件")), "{findings:?}");
    assert!(findings.iter().any(|f| f.contains("120")), "{findings:?}");
}

#[test]
fn inspect_finds_a_dependency_the_grounds_do_not_declare() {
    let mut board = fixture();
    board["topics"][4]["answer"]["text"] = json!("論点2の形で、デザインシステムに従う。");
    board["topics"][4]["grounds"] =
        json!([{"supports": "s", "basis": "b", "tag": "rule", "source": "利用者"}]);
    let (code, findings) = inspect("dependency", board);
    assert_eq!(code, 1);
    assert!(
        findings
            .iter()
            .any(|f| f.contains("Q5") && f.contains("論点2")),
        "{findings:?}"
    );
}

#[test]
fn inspect_does_not_count_a_downstream_topic_as_a_dependency() {
    // 下流の論点（根拠でこの論点を前提にしている論点）を答えの中で指すのは、依存ではなく任せる先である
    let mut board = fixture();
    board["topics"][0]["answer"]["text"] = json!("直す所は論点2で決める。");
    board["topics"][0]["grounds"] =
        json!([{"supports": "s", "basis": "b", "tag": "rule", "source": "利用者"}]);
    board["topics"][1]["grounds"] =
        json!([{"supports": "開く順", "basis": "b", "tag": "rule", "source": "論点1"}]);
    let (_, findings) = inspect("downstream", board);
    assert!(
        !findings
            .iter()
            .any(|f| f.contains("Q1") && f.contains("論点2")),
        "{findings:?}"
    );
}

#[test]
fn migrate_turns_an_old_board_into_the_new_shape_and_keeps_the_old_file() {
    let root = temp("migrate");
    let board_dir = root.join("old-board");
    fs::create_dir_all(board_dir.join("figures")).unwrap();
    fs::write(
        board_dir.join("figures/shape.svg"),
        "<svg viewBox=\"0 0 10 10\"><rect width=\"10\" height=\"10\"/></svg>",
    )
    .unwrap();
    let old = json!({
        "$schema": "x", "title": "古いボード", "board": "old-board", "round": 2,
        "intro": [{"kind": "para", "text": "何を決めるか"}],
        "queue": [{"no": 1, "why": "開いた"}],
        "topics": [{
            "no": 1, "name": "形", "status": "open", "question": "何を持つか",
            "answer": "<b>5つだけを持つ。</b>",
            "decision": {"letter": "A", "text": [{"kind": "list", "items": [{"text": "答えは1文"}]}]},
            "example": [{"kind": "figure", "name": "shape", "caption": "並び"},
                        {"kind": "html", "text": "{&quot;a&quot;: 1}"},
                        {"kind": "para", "text": "<b>見本</b>の説明"},
                        {"kind": "grid", "cols": ["欄", "値"], "rows": [["a", "1"]]}],
            "passed": [{"name": "A", "body": "b", "cost": "c"}, {"name": "B", "body": "b", "cost": "d"}],
            "dropped": [{"body": "B", "reason": "壊れる"}],
            "grounds": [{"supports": "s", "basis": "b", "tag": "rule", "source": "利用者"}],
            "out_of_scope": [{"item": "外の作業で試す", "treatment": "later"}],
            "findings": ["分かったこと"]
        }]
    });
    fs::write(board_dir.join("board.json"), old.to_string()).unwrap();
    let (code, out) = call(
        "migrate",
        json!({"path": board_dir.join("board.json").to_string_lossy()}),
    );
    assert_eq!(code, 0, "{out}");
    assert!(
        board_dir.join("board.json.v1").exists(),
        "古いファイルを残す"
    );
    let new: Value =
        serde_json::from_str(&fs::read_to_string(board_dir.join("board.json")).unwrap()).unwrap();
    assert_eq!(new["kind"], "board");
    assert_eq!(new["queue"][0]["topic"], "Q1");
    assert_eq!(new["topics"][0]["answer"]["text"], "5つだけを持つ。");
    assert_eq!(new["topics"][0]["images"][0]["kind"], "figure");
    assert!(new["topics"][0]["images"][0]["svg"]
        .as_str()
        .unwrap()
        .starts_with("<svg"));
    assert_eq!(new["topics"][0]["images"][1]["text"], "{\"a\": 1}");
    // 図とコード以外のブロック（段落 ・ 格子など）は、まとめて1枚の UI にして中身を残す
    let ui = new["topics"][0]["images"][2]["html"].as_str().unwrap();
    assert_eq!(new["topics"][0]["images"][2]["kind"], "ui");
    assert!(
        ui.contains("<b>見本</b>の説明") && ui.contains("<th>欄</th>") && ui.contains("<td>1</td>"),
        "{ui}"
    );
    assert_eq!(new["topics"][0]["rejected"][0]["option"], "B");
    assert_eq!(new["topics"][0]["verification"]["passed"][1]["option"], "B");
    assert_eq!(check(&board_dir), json!({"errors": 0, "drifts": 0}));
}

fn answer(topic: &str, verdict: &str) -> Value {
    json!({"kind": "answer", "id": "4-1", "board": "board-on-schema-driven", "round": 4,
           "answers": [{"topic": format!("board-on-schema-driven.{topic}"), "verdict": verdict}]})
}

#[test]
fn serve_writes_each_answer_as_an_instance_and_skips_a_repeated_one() {
    let root = temp("serve");
    fs::write(root.join("board.json"), fixture().to_string()).unwrap();
    let saved = serve::accept(&root, &answer("Q5", "approved")).unwrap();
    assert!(
        saved["saved"]
            .as_str()
            .unwrap()
            .ends_with("answers/4-1.json"),
        "{saved}"
    );
    let written: Value =
        serde_json::from_str(&fs::read_to_string(root.join("answers/4-1.json")).unwrap()).unwrap();
    assert_eq!(written["kind"], "answer");
    assert!(written["$schema"]
        .as_str()
        .unwrap()
        .ends_with("answer.schema.json"));
    let again = serve::accept(&root, &answer("Q5", "approved")).unwrap();
    assert!(
        again["next"].as_str().unwrap().contains("積まず"),
        "{again}"
    );
}

#[test]
fn serve_refuses_a_return_without_a_reason_and_an_answer_to_another_board() {
    let root = temp("serve-bad");
    fs::write(root.join("board.json"), fixture().to_string()).unwrap();
    assert!(serve::accept(&root, &answer("Q5", "returned")).is_err());
    let mut other = answer("Q5", "approved");
    other["board"] = json!("other-board");
    assert!(serve::accept(&root, &other).is_err());
    assert!(
        !root.join("answers").exists()
            || fs::read_dir(root.join("answers")).unwrap().next().is_none()
    );
}
