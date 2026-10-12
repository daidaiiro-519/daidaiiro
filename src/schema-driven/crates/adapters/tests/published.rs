//! 公開した版のスキーマが書き換えられていないことを、公開の記録（published.json）の指紋で確かめる。
//! check-schemas が、記録した版の中身の変化と消失を指摘し、まだ記録していない名前を指紋つきで返す。
//! 指紋の期待値は、見本のスキーマから前もって計算した値である。

mod support;

use serde_json::{json, Value};
use support::{call, eq, has, len, run, Scratch};

const DIALECT: &str = "https://json-schema.org/draft/2020-12/schema";
/// schema("lexicon/term@v1", 3) の指紋。
const TERM_V1: &str = "cf768b718424af4f8a23957013cdb9fe3e7616bef7ee9b4a0f63802ac1301d4d";
/// schema("lexicon/term@v2", 4) の指紋。
const TERM_V2: &str = "07ff57776878f9271bd0edc98f9217406743c52b1d8ecc31b5ceab5953d77855";
/// meta() の指紋。
const META: &str = "8188d1eb41ddd2e2aa8137d59409bdb659d03d4271570ff3bea92fc55a12c7e8";

fn schema(id: &str, max: u64) -> Value {
    json!({"$schema": DIALECT, "x-abstract": "schema-driven/abstract@v1", "$id": id,
           "title": "語の決まり", "description": "見本",
           "$defs": {"short": {"type": "string", "maxLength": max}}})
}

/// .schema.json で終わらない名前を持つスキーマ。
fn meta(kind: &str) -> Value {
    json!({"$schema": DIALECT, "$id": "lexicon/abstract@v1", "type": kind})
}

/// 公開の記録に、名前と指紋を書く。
fn publish(scratch: &Scratch, name: &str, sha256: &str) {
    scratch.put(
        "published.json",
        &json!({"$schema": "schema-driven/published@v1", "kind": "published",
                "schemas": [{"name": name, "sha256": sha256}]}),
    );
}

/// term.schema.json に lexicon/term@v1 を置く。
fn term(name: &str) -> Scratch {
    let scratch = Scratch::new(&format!("published-{name}"));
    scratch.put("term.schema.json", &schema("lexicon/term@v1", 3));
    scratch
}

#[test]
fn a_new_name_is_listed_with_its_fingerprint() {
    let dir = term("new");
    run(
        &dir,
        None,
        &[call(
            "check-schemas",
            &[("dir", "./")],
            0,
            &[
                ("/findings", eq(json!([]))),
                (
                    "/unpublished",
                    eq(json!([{"name": "lexicon/term@v1", "sha256": TERM_V1}])),
                ),
            ],
        )],
    );
}

#[test]
fn a_recorded_version_that_did_not_change_passes() {
    let dir = term("same");
    publish(&dir, "lexicon/term@v1", TERM_V1);
    run(
        &dir,
        None,
        &[call(
            "check-schemas",
            &[("dir", "./")],
            0,
            &[
                ("/findings", eq(json!([]))),
                ("/unpublished", eq(json!([]))),
            ],
        )],
    );
}

#[test]
fn key_order_and_spacing_do_not_change_the_fingerprint() {
    let dir = term("order");
    publish(&dir, "lexicon/term@v1", TERM_V1);
    // 同じ中身を、キーの順を変えて字下げして書き直す
    dir.put_text(
        "term.schema.json",
        r#"{
      "$defs": {"short": {"maxLength": 3, "type": "string"}},
      "description": "見本", "title": "語の決まり",
      "$id": "lexicon/term@v1", "x-abstract": "schema-driven/abstract@v1",
      "$schema": "https://json-schema.org/draft/2020-12/schema"
    }"#,
    );
    run(
        &dir,
        None,
        &[call(
            "check-schemas",
            &[("dir", "./")],
            0,
            &[("/findings", eq(json!([])))],
        )],
    );
}

#[test]
fn a_rewritten_published_version_is_a_finding() {
    let dir = term("rewritten");
    publish(&dir, "lexicon/term@v1", TERM_V1);
    dir.put("term.schema.json", &schema("lexicon/term@v1", 4));
    run(
        &dir,
        None,
        &[call(
            "check-schemas",
            &[("dir", "./")],
            0,
            &[
                ("/findings", len(1)),
                ("/findings/0/rule", eq("公開した版が書き換えられている")),
                ("/findings/0/message", has("@v2")),
            ],
        )],
    );
}

#[test]
fn a_removed_published_version_is_a_finding() {
    let dir = term("removed");
    publish(&dir, "lexicon/term@v1", TERM_V1);
    dir.remove("term.schema.json");
    run(
        &dir,
        None,
        &[call(
            "check-schemas",
            &[("dir", "./")],
            0,
            &[
                ("/findings", len(1)),
                ("/findings/0/rule", eq("公開した版が無い")),
            ],
        )],
    );
}

#[test]
fn a_new_version_beside_the_old_one_is_only_unpublished() {
    let dir = term("v2");
    publish(&dir, "lexicon/term@v1", TERM_V1);
    dir.put("term.v2.schema.json", &schema("lexicon/term@v2", 4));
    run(
        &dir,
        None,
        &[call(
            "check-schemas",
            &[("dir", "./")],
            0,
            &[
                ("/findings", eq(json!([]))),
                (
                    "/unpublished",
                    eq(json!([{"name": "lexicon/term@v2", "sha256": TERM_V2}])),
                ),
            ],
        )],
    );
}

#[test]
fn a_named_schema_whose_file_is_not_dot_schema_json_is_also_fingerprinted() {
    let dir = Scratch::new("published-meta");
    dir.put("meta-schema.json", &meta("object"));
    run(
        &dir,
        None,
        &[call(
            "check-schemas",
            &[("dir", "./")],
            0,
            &[(
                "/unpublished",
                eq(json!([{"name": "lexicon/abstract@v1", "sha256": META}])),
            )],
        )],
    );
    publish(&dir, "lexicon/abstract@v1", META);
    dir.put("meta-schema.json", &meta("array"));
    run(
        &dir,
        None,
        &[call(
            "check-schemas",
            &[("dir", "./")],
            0,
            &[
                ("/findings", len(1)),
                ("/findings/0/rule", eq("公開した版が書き換えられている")),
            ],
        )],
    );
}
