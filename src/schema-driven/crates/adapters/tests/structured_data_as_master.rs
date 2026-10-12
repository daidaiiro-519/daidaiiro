//! 構造化データを正本として持つ（要約レベル）。
//! 基盤の共通ツールだけで、作成→ x-prompt を受け取る→ 更新→ 描画→ 承認を通す。
//! Skill report が、報告の concrete スキーマ ・ ページテンプレート ・ 部品のカタログ（試験用の小さなもの）を持つ。

mod support;

use serde_json::{json, Value};
use support::{call, contains_all, eq, len, put_catalog, run, Scratch};

const SKILL: Option<&str> = Some("./report");
const SCHEMA: &str = "report/report@v1";
const SCHEMA_FILE: &str = "./report/assets/report.schema.json";
const INSTANCE: &str = "./declarations/D-1.json";

/// 未記入の題と出来事を埋める JSON Patch。
const FILL: &str = r#"[
    {"op": "add", "path": "/kind", "value": "report"},
    {"op": "add", "path": "/title", "value": "注文 API の応答遅延の調査報告"},
    {"op": "add", "path": "/events", "value": ["応答が最大 8.2 秒まで遅れた"]}
]"#;

fn field(means: &str, value: Value) -> Value {
    let mut out = json!({"description": means,
        "x-prompt": {"read": {"means": means},
                     "write": [{"id": "one", "rule": "値を書く。", "why": "報告の材料になる。", "test": "値があるかを見る。"}]}});
    out.as_object_mut()
        .unwrap()
        .extend(value.as_object().unwrap().clone());
    out
}

/// Skill report：報告のスキーマ ・ ページテンプレート ・ 部品のカタログを置く。
fn setup(name: &str) -> Scratch {
    let scratch = Scratch::new(name);
    scratch.mkdir("declarations");
    put_catalog(&scratch, "report");
    scratch.put(
        "report/assets/report.schema.json",
        &json!({"$schema": "https://json-schema.org/draft/2020-12/schema", "x-abstract": "schema-driven/abstract@v1",
            "$id": SCHEMA, "title": "報告", "description": "調べたことの報告", "x-generates": "declarations/<番号>.json",
            "x-view": {"label": "title"}, "type": "object", "required": ["$schema", "kind", "title", "events"],
            "properties": {
                "$schema": field("従うスキーマの名前である。", json!({"const": SCHEMA})),
                "kind": field("報告であることを表す。", json!({"const": "report"})),
                "title": field("報告の題である。", json!({"type": "string"})),
                "events": field("起きたことの並びである。", json!({"type": "array", "items": {"type": "string"}}))
            }}),
    );
    scratch.put(
        "report/pages/report.json",
        &json!({"kind": "report", "sections": [{"heading": "起きたこと", "body": [
            {"component": "list", "items": [{"component": "pill", "each": "this.events", "text": "@"}]}]}]}),
    );
    scratch
}

#[test]
fn uc_0_sg_1_the_owner_approves_a_master_written_from_prompts() {
    let scratch = setup("uc0-success");
    run(
        &scratch,
        SKILL,
        &[
            // 作成する：必須のプロパティは未記入として返る
            call(
                "create",
                &[("schema", SCHEMA_FILE), ("path", INSTANCE)],
                0,
                &[
                    ("/unfilled", len(3)),
                    ("/unfilled/0/property", eq("/events")),
                    ("/unfilled/1/property", eq("/kind")),
                    ("/unfilled/2/property", eq("/title")),
                ],
            ),
            // 未記入のプロパティの x-prompt を受け取る
            call(
                "prompt",
                &[("schema", SCHEMA_FILE), ("property", "/events")],
                0,
                &[("/prompt/read/means", eq("起きたことの並びである。"))],
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
                &[
                    ("dir", "./declarations"),
                    ("pages", "./report/pages"),
                    ("out", "./out"),
                ],
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
        &[
            "<h1>注文 API の応答遅延の調査報告</h1>",
            r#"<span class="pill ">応答が最大 8.2 秒まで遅れた</span>"#,
        ],
    );
    assert!(scratch.exists("declarations/approval.json"));
}

#[test]
fn uc_0_mg_1_a_master_with_validation_errors_is_neither_rendered_nor_approved() {
    let scratch = setup("uc0-minimal");
    run(
        &scratch,
        SKILL,
        &[
            call(
                "create",
                &[("schema", SCHEMA_FILE), ("path", INSTANCE)],
                0,
                &[],
            ),
            call("update", &[("path", INSTANCE), ("patch", FILL)], 0, &[]),
        ],
    );
    // 更新の道を通らずにファイルを書き換え、形の違う値を入れる
    let mut instance = scratch.read_json(INSTANCE);
    instance["title"] = json!(1);
    scratch.put(INSTANCE, &instance);
    run(
        &scratch,
        SKILL,
        &[
            call(
                "render",
                &[
                    ("dir", "./declarations"),
                    ("pages", "./report/pages"),
                    ("out", "./out"),
                ],
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
