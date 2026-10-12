//! 構造化データを正本として持つ（要約レベル）。
//! 基盤の共通ツールだけで、作成→ x-prompt を受け取る→ 更新→ 描画→ 承認を通す。
//! ページテンプレートを持たないので、基盤の文書の契約（schemas/document.schema.json）で書く。
//! スキーマは、エンジンが持つ文書のスキーマを名前で引く。

mod support;

use serde_json::json;
use support::{call, contains_all, eq, len, run, Scratch};

const SCHEMA: &str = "schema-driven/document@v1";
const INSTANCE: &str = "./declarations/D-1.json";

/// 未記入の題と節を埋める JSON Patch。
const FILL: &str = r#"[
    {"op": "add", "path": "/kind", "value": "document"},
    {"op": "add", "path": "/title", "value": "注文 API の応答遅延の調査報告"},
    {"op": "add", "path": "/sections", "value": [
        {"heading": "起きたこと", "blocks": [{"type": "paragraph", "text": "応答が最大 **8.2 秒**まで遅れた。"}]}
    ]}
]"#;

#[test]
fn uc_0_sg_1_the_owner_approves_a_master_written_from_prompts() {
    let scratch = Scratch::new("uc0-success");
    scratch.mkdir("declarations");
    run(
        &scratch,
        None,
        &[
            // 作成する：必須のプロパティは未記入として返る
            call(
                "create",
                &[("schema", SCHEMA), ("path", INSTANCE)],
                0,
                &[
                    ("/unfilled", len(3)),
                    ("/unfilled/0/property", eq("/kind")),
                    ("/unfilled/1/property", eq("/sections")),
                    ("/unfilled/2/property", eq("/title")),
                ],
            ),
            // 未記入のプロパティの x-prompt を受け取る
            call(
                "prompt",
                &[("schema", SCHEMA), ("property", "/sections")],
                0,
                &[("/prompt/read/means", eq("文書の節の並びである。"))],
            ),
            // 更新する：書いたものをその場で確かめ、未記入が無くなる
            call(
                "update",
                &[("path", INSTANCE), ("patch", FILL)],
                0,
                &[("/errors", eq(json!([]))), ("/unfilled", eq(json!([])))],
            ),
            // 描画して、データの持ち主が読む
            call(
                "render",
                &[("dir", "./declarations"), ("out", "./out")],
                0,
                &[("/pages/0/path", eq("./out/D-1.html"))],
            ),
            // 承認する：承認記録が空でない
            call(
                "approve",
                &[("dir", "./declarations")],
                0,
                &[("/instances", len(1)), ("/instances/0/path", eq(INSTANCE))],
            ),
        ],
    );
    contains_all(
        "D-1.html",
        &scratch.read("out/D-1.html"),
        &["注文 API の応答遅延の調査報告", "<strong>8.2 秒</strong>"],
    );
    assert!(scratch.exists("declarations/approval.json"));
}

#[test]
fn uc_0_mg_1_a_master_with_validation_errors_is_neither_rendered_nor_approved() {
    let scratch = Scratch::new("uc0-minimal");
    scratch.mkdir("declarations");
    run(
        &scratch,
        None,
        &[
            call("create", &[("schema", SCHEMA), ("path", INSTANCE)], 0, &[]),
            call("update", &[("path", INSTANCE), ("patch", FILL)], 0, &[]),
        ],
    );
    // 更新の道を通らずにファイルを書き換え、形の違う値を入れる
    let mut instance = scratch.read_json(INSTANCE);
    instance["title"] = json!(1);
    scratch.put(INSTANCE, &instance);
    run(
        &scratch,
        None,
        &[
            call(
                "render",
                &[("dir", "./declarations"), ("out", "./out")],
                1,
                &[("/ok", eq(false))],
            ),
            // 承認した正本は検証を通過している（MG-1）：検証エラーがあれば承認記録を書かない
            call(
                "approve",
                &[("dir", "./declarations")],
                1,
                &[("/ok", eq(false))],
            ),
        ],
    );
    assert!(!scratch.exists("out/D-1.html"));
    assert!(!scratch.exists("declarations/approval.json"));
}
