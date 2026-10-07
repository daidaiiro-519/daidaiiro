//! ボードの Design。
//! 基盤の render が、board.schema.json のインスタンスを、ボードのページテンプレートと Design で描画する。

use brainstorming_board::design::BoardDesign;
use schema_driven_adapters::inbound::tools::Toolbox;
use schema_driven_adapters::outbound::document_design::DocumentDesign;
use schema_driven_adapters::outbound::{fs::FileSystem, jmespath::Jmespath};
use schema_driven_core::application::checks::Checks;
use schema_driven_core::application::instances::Instances;
use schema_driven_core::application::renders::Renders;
use serde_json::{json, Map, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

const REFERENCES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../references");

/// 一時ディレクトリに、ボードのインスタンスとスキーマを置く。
fn setup(name: &str, board: Value) -> PathBuf {
    let root = std::env::temp_dir().join(format!("bb-design-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("board")).unwrap();
    fs::copy(
        format!("{REFERENCES}/board.schema.json"),
        root.join("board/board.schema.json"),
    )
    .unwrap();
    fs::write(root.join("board/board.json"), board.to_string()).unwrap();
    root
}

fn fixture() -> Value {
    serde_json::from_str(include_str!("fixtures/board.json")).unwrap()
}

fn render(root: &Path) -> (i32, Value) {
    let (files, query) = (Arc::new(FileSystem), Arc::new(Jmespath));
    let (instances, checks) = (
        Instances::new(files.clone(), files.clone(), query.clone()),
        Checks::new(files.clone(), files.clone(), query.clone()),
    );
    let renders = Renders::new(
        files.clone(),
        files,
        query,
        Arc::new(DocumentDesign),
        Some(Arc::new(BoardDesign)),
        None,
    );
    let args: Map<String, Value> = json!({
        "dir": root.join("board").to_string_lossy(),
        "pages": format!("{REFERENCES}/pages"),
        "out": root.join("out").to_string_lossy()
    })
    .as_object()
    .cloned()
    .unwrap();
    Toolbox::base()
        .with_render(Arc::new(renders))
        .dispatch("render", &args, &instances, &checks)
}

fn page(name: &str, board: Value) -> String {
    let root = setup(name, board);
    let (code, out) = render(&root);
    assert_eq!(code, 0, "{out}");
    fs::read_to_string(root.join("out/board.html")).unwrap()
}

#[test]
fn each_topic_shows_question_answer_images_grounds_and_rejected_options_only() {
    let html = page("parts", fixture());
    for expected in [
        "<span class=\"tn\">Q1</span>論点の形",
        "ブレストボードの論点は、何を持つか",
        "論点は、問い ・ 答え ・ 完成イメージ ・ 根拠 ・ 採らなかった案の5つだけを持つ。",
        "この答えの完成イメージ ── 図 ・ コード",
        "根拠（",
        "採らなかった案（",
    ] {
        assert!(html.contains(expected), "{expected} が無い");
    }
    // 書き手の検証の記録は描画しない
    assert!(!html.contains("ここは承認者に見せない書き手の記録"));
}

#[test]
fn only_open_topics_get_an_answer_form() {
    let html = page("form", fixture());
    assert_eq!(
        html.matches("class=\"form\"").count(),
        1,
        "開いているのは Q5 だけ"
    );
    assert!(html.contains("data-topic=\"Q5\""));
    assert!(html.contains("<span class=\"st now\">いま見る</span>"));
}

#[test]
fn images_switch_between_figure_code_and_ui() {
    let html = page("images", fixture());
    assert!(html.contains("この答えの完成イメージ ── 図 ・ コード ・ UI"));
    // UI はボードと見た目が混ざらないよう、分けた枠（iframe）の中に描画し、HTML は属性の中でエスケープする
    assert!(
        html.contains("<iframe class=\"ix-ui\" title=\"描画した UI\" srcdoc=\"&lt;!-- ui --&gt;")
    );
    // 図は基盤が検査した SVG をそのまま埋め込む
    assert!(html.contains("<figure class=\"fig-top\"><svg"));
}

#[test]
fn a_svg_with_a_script_is_refused() {
    let mut board = fixture();
    board["topics"][0]["images"][0]["svg"] = json!("<svg><script>alert(1)</script></svg>");
    let root = setup("script", board);
    let (code, out) = render(&root);
    assert_eq!(code, 1, "{out}");
    assert!(!root.join("out").exists());
}

#[test]
fn the_board_follows_its_design_system() {
    let html = page("tokens", fixture());
    // 色は基盤がトークンから作った変数で出る。押した状態はどのボタンも key
    assert!(html.contains(":root{"));
    assert!(html.contains(":root[data-theme=\"dark\"]{"));
    assert!(!html.contains(".vb.ret[aria-pressed=\"true\"]{"));
    assert!(html.contains(".vb[aria-pressed=\"true\"]{background:var(--key)"));
}
