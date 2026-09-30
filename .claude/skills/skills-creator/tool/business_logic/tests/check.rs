// SPDX-License-Identifier: MIT
//! 契約の検査を事例で検証する。
//!
//!     cargo test -p sc_business_logic

use std::path::{Path, PathBuf};

use sc_business_logic::behavior::{self, Verdict};
use sc_business_logic::check::{self, State, Templates};

fn templates() -> Templates {
    let skills = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    Templates::new(
        skills.join("skills-creator/references/skill-template.md"),
        skills.join("advisor-creator/references/skill-template-advisor.md"),
    )
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("sc-check-{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("作れる");
    dir
}

/// 2段目（Rust の組）と文書の検出。**実行ファイルを起動しない** ── 層と依存の向きの事例は、
/// 実行ファイルを組まずに確かめる。
fn found_in_source(root: &Path) -> Vec<String> {
    let mut out = check::rust(root);
    out.extend(check::document(root, &templates()));
    out
}

/// 2段すべての検出。
fn found_all(root: &Path) -> Vec<String> {
    check::check(root, &templates()).findings()
}

/// 雛形が要求する節だけを持つ文書を置く。
fn write_document(root: &Path) {
    let template = std::fs::read_to_string(templates().general).expect("読める");
    let body: String = sc_business_logic::sections::headings(&template)
        .iter()
        .filter(|x| !x.contains("{{"))
        .map(|x| format!("## {x}\n\n本文\n\n"))
        .collect();
    std::fs::write(root.join("SKILL.md"), body).expect("書ける");
}

/// 層を crate に分けた、契約を満たす形を置く。**道具のソースは tool/ に、実行ファイルは
/// bin/ に置き、bin/ は git で追跡しない。**
fn write_layers(root: &Path, edges: &[(&str, &[&str])]) {
    std::fs::write(root.join(".gitignore"), "bin/\ntool/target/\n").expect("書ける");
    let rs = root.join(check::TOOL);
    let members: Vec<String> = edges
        .iter()
        .map(|(name, _)| format!("\"{name}\""))
        .collect();
    std::fs::create_dir_all(&rs).expect("作れる");
    std::fs::write(
        rs.join("Cargo.toml"),
        format!("[workspace]\nmembers = [{}]\n", members.join(", ")),
    )
    .expect("書ける");
    for (name, deps) in edges {
        let dir = rs.join(name);
        std::fs::create_dir_all(&dir).expect("作れる");
        let listed: String = deps
            .iter()
            .map(|d| format!("{d} = {{ path = \"../{d}\" }}\n"))
            .collect();
        std::fs::write(
            dir.join("Cargo.toml"),
            format!("[package]\nname = \"{name}\"\n\n[dependencies]\n{listed}"),
        )
        .expect("書ける");
    }
    std::fs::create_dir_all(rs.join(check::TESTS)).expect("作れる");
}

const GOOD: [(&str, &[&str]); 4] = [
    ("data_access", &[]),
    ("business_logic", &["data_access"]),
    ("service", &["business_logic"]),
    ("cli", &["service"]),
];

#[test]
fn a_skill_with_layers_as_crates_passes() {
    let root = scratch("ok");
    write_document(&root);
    write_layers(&root, &GOOD);
    let found = found_in_source(&root);
    assert!(found.is_empty(), "食い違いが出た ── {found:?}");
}

#[test]
fn a_skill_without_tools_needs_no_layers() {
    let root = scratch("advice");
    write_document(&root);
    let found = found_all(&root);
    assert!(
        found.is_empty(),
        "助言だけの Skill に層を要求しない ── {found:?}"
    );
}

#[test]
fn python_that_remains_is_reported() {
    let root = scratch("python");
    write_document(&root);
    write_layers(&root, &GOOD);
    std::fs::create_dir_all(root.join("scripts")).expect("作れる");
    let found = found_all(&root);
    assert!(
        found.iter().any(|x| x.contains("Python が残っている")),
        "{found:?}"
    );
}

#[test]
fn layers_that_are_not_crates_are_reported() {
    let root = scratch("flat");
    write_document(&root);
    std::fs::create_dir_all(root.join("scripts")).expect("作れる");
    let found = found_in_source(&root);
    assert!(
        found.iter().any(|x| x.contains("crate に分かれていない")),
        "1つの単位の中の module では、内側が外側を参照してもコンパイラが通す ── {found:?}"
    );
}

#[test]
fn an_edge_that_goes_outward_is_reported() {
    let root = scratch("outward");
    write_document(&root);
    // **業務ロジック層がプレゼンテーション層を参照する** ── 下から上である
    write_layers(
        &root,
        &[
            ("business_logic", &["cli"]),
            ("service", &["business_logic"]),
            ("cli", &["service"]),
        ],
    );
    let found = found_in_source(&root);
    assert!(
        found.iter().any(|x| x.contains("依存の向きに違反している")
            && x.contains("business_logic → presentation")),
        "{found:?}"
    );
}

#[test]
fn an_edge_that_skips_a_layer_is_reported() {
    let root = scratch("skip");
    write_document(&root);
    // プレゼンテーション層が業務ロジック層を直に参照する ── サービス層を飛ばしている
    write_layers(
        &root,
        &[
            ("business_logic", &[]),
            ("service", &["business_logic"]),
            ("cli", &["service", "business_logic"]),
        ],
    );
    let found = found_in_source(&root);
    assert!(
        found.iter().any(|x| x.contains("依存の向きに違反している")
            && x.contains("presentation → business_logic")),
        "層を飛ばす依存は、道具の一覧を経ない呼び出し方を作る ── {found:?}"
    );
}

#[test]
fn a_retired_layer_name_is_reported() {
    let root = scratch("retired");
    write_document(&root);
    write_layers(&root, &GOOD);
    std::fs::create_dir_all(root.join(check::TOOL).join("parts")).expect("作れる");
    let found = found_in_source(&root);
    assert!(
        found
            .iter()
            .any(|x| x.contains("parts/ が残っている") && x.contains("business_logic")),
        "{found:?}"
    );
}

#[test]
fn io_in_the_business_logic_layer_is_reported() {
    // **入出力はデータアクセス層だけが持つ**（ACDR 0058）── 見つけた行を、そのまま示す
    let root = scratch("io-leak");
    write_document(&root);
    write_layers(&root, &GOOD);
    write_part(
        &root,
        "fn f() -> String {\n    std::fs::read_to_string(\"a\").unwrap()\n}\n",
    );
    let found = found_in_source(&root);
    assert!(
        found.iter().any(|x| x.contains("入出力を直接扱っている")
            && x.contains("run.rs:2")
            && x.contains("std::fs")),
        "{found:?}"
    );
}

#[test]
fn io_through_the_data_access_layer_is_allowed() {
    let root = scratch("io-through");
    write_document(&root);
    write_layers(&root, &GOOD);
    write_part(&root, "fn f() -> bool {\n    files::is_file(\"a\")\n}\n");
    write_access(
        &root,
        "pub fn g() -> bool {\n    std::path::Path::new(\"a\").is_file()\n}\n",
    );
    let found = found_in_source(&root);
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn the_service_layer_is_checked_but_the_contract_is_exempt() {
    // **サービス層も入出力を持たない。** 例外は、起動の設定を読む contract.rs だけである
    let root = scratch("io-service");
    write_document(&root);
    write_layers(&root, &GOOD);
    write_in(
        &root,
        "service/src",
        "fn f() { let _ = std::fs::write(\"a\", \"b\"); }\n",
    );
    let dir = root.join(check::TOOL).join("service/src");
    std::fs::write(
        dir.join("contract.rs"),
        "fn g() { let _ = std::fs::read_to_string(\"tool.json\"); }\n",
    )
    .expect("書ける");
    let found = found_in_source(&root);
    assert!(
        found.iter().any(|x| x.contains("service/src/run.rs")),
        "{found:?}"
    );
    assert!(
        !found.iter().any(|x| x.contains("contract.rs")),
        "{found:?}"
    );
}

#[test]
fn a_service_that_skips_to_data_access_is_reported() {
    let root = scratch("skip-to-data");
    write_document(&root);
    write_layers(
        &root,
        &[
            ("data_access", &[]),
            ("business_logic", &["data_access"]),
            ("service", &["business_logic", "data_access"]),
            ("cli", &["service"]),
        ],
    );
    let found = found_in_source(&root);
    assert!(
        found
            .iter()
            .any(|x| x.contains("依存の向きに違反している") && x.contains("service → data_access")),
        "{found:?}"
    );
}

#[test]
fn a_missing_layer_is_reported() {
    let root = scratch("missing");
    write_document(&root);
    write_layers(
        &root,
        &[("business_logic", &[]), ("cli", &["business_logic"])],
    );
    let found = found_in_source(&root);
    assert!(
        found
            .iter()
            .any(|x| x.contains("層の crate が無い") && x.contains("service")),
        "{found:?}"
    );
}

#[test]
fn a_missing_entry_is_reported() {
    let root = scratch("noentry");
    write_document(&root);
    write_layers(
        &root,
        &[("business_logic", &[]), ("service", &["business_logic"])],
    );
    let found = found_in_source(&root);
    assert!(
        found
            .iter()
            .any(|x| x.contains("プレゼンテーション層の crate が無い")),
        "{found:?}"
    );
}

#[test]
fn absent_examples_are_reported() {
    let root = scratch("notests");
    write_document(&root);
    write_layers(&root, &GOOD);
    std::fs::remove_dir_all(root.join(check::TOOL).join(check::TESTS)).expect("消せる");
    let found = found_in_source(&root);
    assert!(found.iter().any(|x| x.contains("事例が無い")), "{found:?}");
}

#[test]
fn a_missing_section_is_reported() {
    let root = scratch("section");
    write_layers(&root, &GOOD);
    std::fs::write(root.join("SKILL.md"), "## 目的\n\n本文\n").expect("書ける");
    let found = found_in_source(&root);
    assert!(found.iter().any(|x| x.contains("節が無い")), "{found:?}");
}

#[test]
fn an_absent_document_is_reported() {
    let root = scratch("nodoc");
    write_layers(&root, &GOOD);
    let found = found_in_source(&root);
    assert!(found.iter().any(|x| x.contains("文書が無い")), "{found:?}");
}

/// 業務ロジック層に1つのファイルを置く。
fn write_part(root: &Path, body: &str) {
    write_in(root, "business_logic/src", body);
}

/// データアクセス層に1つのファイルを置く。**入出力はここだけが持つ。**
fn write_access(root: &Path, body: &str) {
    write_in(root, "data_access/src", body);
}

fn write_in(root: &Path, dir: &str, body: &str) {
    let dir = root.join(check::TOOL).join(dir);
    std::fs::create_dir_all(&dir).expect("作れる");
    std::fs::write(dir.join("run.rs"), body).expect("書ける");
}

#[test]
fn a_remaining_rs_folder_is_reported() {
    // **道具のソースは tool/ に置く** ── rs/ は言語の名前で、中身の役割を示さない
    let root = scratch("rs-left");
    write_document(&root);
    write_layers(&root, &GOOD);
    std::fs::create_dir_all(root.join("rs")).expect("作れる");
    let found = found_in_source(&root);
    assert!(
        found.iter().any(|x| x.contains("rs/ が残っている")),
        "{found:?}"
    );
}

#[test]
fn bin_that_git_would_track_is_reported() {
    // **実行ファイルは配布物だけが持つ** ── git で追跡すると、OS ごとの実行ファイルが混ざる
    let root = scratch("bin-tracked");
    write_document(&root);
    write_layers(&root, &GOOD);
    std::fs::write(root.join(".gitignore"), "tool/target/\n").expect("書ける");
    let found = found_in_source(&root);
    assert!(found.iter().any(|x| x.contains("bin/")), "{found:?}");
}

#[test]
fn a_call_written_inside_a_string_is_not_counted() {
    // **文字列の中の `Command::new(` は、呼び出しではない** ── 呼び出しを探す側の定数が該当する
    let root = scratch("in-string");
    write_document(&root);
    write_layers(&root, &GOOD);
    write_part(&root, "const CALL: &str = \"Command::new(\";\n");
    let found = found_in_source(&root);
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn examples_are_not_shipped_so_their_tools_are_not_counted() {
    // **cargo の examples は、配布する実行ファイルに入らない** ── 開発用の例が呼ぶ道具を、tool.json に求めない
    let root = scratch("examples");
    write_document(&root);
    write_layers(&root, &GOOD);
    let dir = root.join(check::TOOL).join("business_logic/examples");
    std::fs::create_dir_all(&dir).expect("作れる");
    std::fs::write(
        dir.join("bench.rs"),
        "fn f() { std::process::Command::new(\"dot\"); }\n",
    )
    .expect("書ける");
    let found = found_in_source(&root);
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn what_scaffold_places_satisfies_check() {
    // **生んだものが、そのまま契約を満たす。** 満たさないと、新しい Skill は必ず
    // 不合格の状態で生まれる（実測 ── 契約を Rust の形へ変えたとき、雛形が Python の
    // ままだったのでそうなった）
    let here = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let tmpl = here.join("references/profiles/rust");
    let root = scratch("scaffolded");
    for (from, to) in [
        ("workspace.Cargo.toml.tmpl", "tool/Cargo.toml"),
        ("data_access.Cargo.toml.tmpl", "tool/data_access/Cargo.toml"),
        ("data_access.lib.rs.tmpl", "tool/data_access/src/lib.rs"),
        ("files.rs.tmpl", "tool/data_access/src/files.rs"),
        ("process.rs.tmpl", "tool/data_access/src/process.rs"),
        (
            "business_logic.Cargo.toml.tmpl",
            "tool/business_logic/Cargo.toml",
        ),
        (
            "business_logic.lib.rs.tmpl",
            "tool/business_logic/src/lib.rs",
        ),
        (
            "business_logic.tests.rs.tmpl",
            "tool/business_logic/tests/example.rs",
        ),
        ("service.Cargo.toml.tmpl", "tool/service/Cargo.toml"),
        ("service.lib.rs.tmpl", "tool/service/src/lib.rs"),
        ("contract.rs.tmpl", "tool/service/src/contract.rs"),
        ("cli.Cargo.toml.tmpl", "tool/cli/Cargo.toml"),
        ("mcp.Cargo.toml.tmpl", "tool/mcp/Cargo.toml"),
        ("mcp.json.tmpl", "mcp.json"),
        ("tool.json.tmpl", "tool.json"),
        ("gitignore.tmpl", ".gitignore"),
        ("refs.rs.tmpl", "tool/business_logic/src/refs.rs"),
        ("service.refs.rs.tmpl", "tool/service/src/refs.rs"),
        (
            "document.schema.json.tmpl",
            "references/document.schema.json",
        ),
    ] {
        let body = std::fs::read_to_string(tmpl.join(from))
            .unwrap_or_else(|e| panic!("{from} を読めない ── {e}"))
            .replace("{{Skill名}}", "sample")
            .replace("{{接頭辞}}", "sample");
        let dst = root.join(to);
        std::fs::create_dir_all(dst.parent().expect("親が在る")).expect("作れる");
        std::fs::write(&dst, body).expect("書ける");
    }
    write_document(&root);
    let mut found = found_in_source(&root);
    // **版2 の規則も満たす** ── references の実装は雛形の複製で、references はスキーマに合う
    found.extend(check::rust_refs(
        &root,
        &templates().with_refs(tmpl.join("refs.rs.tmpl")),
    ));
    found.extend(sc_business_logic::refs::validate(&root.join("references")).expect("読める"));
    // **1段目のうち、起動せずに見られる書き方も満たす**（実行ファイルは組んでいないので起動しない）
    found.extend(
        behavior::declaration(&root)
            .into_iter()
            .filter_map(|v| match v {
                Verdict::Fail(t) => Some(t),
                _ => None,
            }),
    );
    assert!(
        found.is_empty(),
        "生んだものが契約を満たしていない ── {found:?}"
    );
}

#[test]
fn a_hardcoded_tool_name_is_reported() {
    // **業務ロジック層は外部の道具の名前を直書きしない** ── tool.json に宣言し、サービス層から注入する
    let root = scratch("hardcoded");
    write_document(&root);
    write_layers(&root, &GOOD);
    write_access(&root, "fn f() { std::process::Command::new(\"git\"); }\n");
    let found = found_in_source(&root);
    assert!(
        found
            .iter()
            .any(|x| x.contains("直書きしている") && x.contains("git")),
        "{found:?}"
    );
}

#[test]
fn a_command_passed_in_is_allowed() {
    // 注入されたコマンドを起動するのは、直書きではない
    let root = scratch("injected");
    write_document(&root);
    write_layers(&root, &GOOD);
    write_access(
        &root,
        "fn f(git: &str) { std::process::Command::new(git); }\n",
    );
    let found = found_in_source(&root);
    assert!(found.is_empty(), "{found:?}");
}

/// 起動のコマンドの登録（tool.json と mcp.json）を置く。
fn write_entries(root: &Path, tool: &str, mcp: &str) {
    std::fs::write(root.join("tool.json"), tool).expect("書ける");
    std::fs::write(root.join("mcp.json"), mcp).expect("書ける");
}

const TOOL_JSON: &str =
    r#"{"cli":{"command":"${CLAUDE_PROJECT_DIR:-.}/bin/fake","args":[]},"external":[]}"#;
const MCP_JSON: &str =
    r#"{"mcpServers":{"fake":{"command":"${CLAUDE_PROJECT_DIR:-.}/bin/fake-mcp","args":[]}}}"#;

#[test]
fn an_external_tool_without_a_reason_or_command_is_reported() {
    let root = scratch("no-reason");
    write_entries(
        &root,
        r#"{"cli":{"command":"bin/x"},"external":[{"name":"git","command":"git","reason":""},{"name":"aws","reason":"音声を合成する"}]}"#,
        MCP_JSON,
    );
    let found: Vec<String> = behavior::declaration(&root)
        .into_iter()
        .filter_map(|v| match v {
            Verdict::Fail(t) => Some(t),
            Verdict::Pass(_) => None,
            _ => None,
        })
        .collect();
    assert!(
        found
            .iter()
            .any(|x| x.contains("理由が無い") && x.contains("git")),
        "{found:?}"
    );
    assert!(
        found
            .iter()
            .any(|x| x.contains("起動するコマンドが無い") && x.contains("aws")),
        "{found:?}"
    );
}

#[test]
fn an_absolute_path_in_the_registration_is_reported() {
    // **登録に開発機の絶対パスを書かない** ── 別の場所では存在しない場所を指す
    let root = scratch("abs");
    write_entries(
        &root,
        TOOL_JSON,
        r#"{"mcpServers":{"x":{"command":"/home/me/x/bin/x-mcp","args":[]}}}"#,
    );
    let found: Vec<String> = behavior::declaration(&root)
        .into_iter()
        .filter_map(|v| match v {
            Verdict::Fail(t) => Some(t),
            _ => None,
        })
        .collect();
    assert!(found.iter().any(|x| x.contains("絶対パス")), "{found:?}");
}

#[test]
fn an_entry_that_is_not_built_is_reported() {
    let root = scratch("not-built");
    write_document(&root);
    write_entries(&root, TOOL_JSON, MCP_JSON);
    let found = found_all(&root);
    assert!(
        found.iter().any(|x| x.contains("組み立ててから")),
        "{found:?}"
    );
}

/// シェルで書いた実行ファイルを置く。**Rust 以外で書いた Skill も、1段目は同じに検査できる**ことを示す。
#[cfg(unix)]
fn write_shell_entries(root: &Path, flag_exit: i32, mcp_args: &str) {
    use std::os::unix::fs::PermissionsExt as _;
    let bin = root.join("bin");
    std::fs::create_dir_all(&bin).expect("作れる");
    let cli = format!(
        r#"#!/bin/sh
case "$1" in
  --json) printf '{{"ok":true,"findings":[],"data":{{"tools":[{{"name":"run","args":[{{"name":"path"}}]}}],"skill_root":"%s"}}}}\n' "$(cd "$(dirname "$0")/.." && pwd)"; exit 0;;
  *) exit {flag_exit};;
esac
"#
    );
    let mcp = format!(
        r#"#!/bin/sh
while IFS= read -r line; do
  case "$line" in
    *'"initialize"'*) printf '{{"jsonrpc":"2.0","id":1,"result":{{"protocolVersion":"2025-06-18","capabilities":{{"tools":{{}}}},"serverInfo":{{"name":"fake","version":"1"}}}}}}\n';;
    *'"tools/list"'*) printf '{{"jsonrpc":"2.0","id":2,"result":{{"tools":[{{"name":"run","inputSchema":{{"type":"object","properties":{{{mcp_args}}}}}}}]}}}}\n';;
  esac
done
"#
    );
    for (name, body) in [("fake", cli), ("fake-mcp", mcp)] {
        let path = bin.join(name);
        std::fs::write(&path, body).expect("書ける");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))
            .expect("変えられる");
    }
    write_entries(root, TOOL_JSON, MCP_JSON);
}

#[cfg(unix)]
#[test]
fn a_skill_written_in_shell_passes_the_first_stage_and_the_second_is_not_run() {
    let root = scratch("shell");
    write_document(&root);
    write_shell_entries(&root, 2, r#""path":{"type":"string"}"#);
    let report = check::check(&root, &templates());
    assert!(report.findings().is_empty(), "{:?}", report.findings());
    assert!(
        report
            .lines
            .iter()
            .any(|l| l.state == State::Skip && l.text.contains("言語の組が無い")),
        "組が無い言語では、2段目を実行しないと出す ── {:?}",
        report.lines
    );
}

#[cfg(unix)]
#[test]
fn mcp_tools_that_differ_from_the_cli_are_reported() {
    // **能力を2か所に書いた実装は、CLI と MCP で食い違う**
    let root = scratch("differ");
    write_document(&root);
    write_shell_entries(&root, 2, r#""target":{"type":"string"}"#);
    let found = found_all(&root);
    assert!(found.iter().any(|x| x.contains("食い違う")), "{found:?}");
}

#[cfg(unix)]
#[test]
fn a_cli_that_accepts_an_unknown_flag_is_reported() {
    let root = scratch("lenient");
    write_document(&root);
    write_shell_entries(&root, 0, r#""path":{"type":"string"}"#);
    let found = found_all(&root);
    assert!(
        found.iter().any(|x| x.contains("旗を断らない")),
        "{found:?}"
    );
}

#[test]
fn a_version_two_skill_without_the_references_implementation_is_reported() {
    // **版2 の規則は、版2 の Skill にだけ当てる** ── 移行していない Skill は、これまでの規則のまま
    let root = scratch("v2-missing");
    write_document(&root);
    write_layers(&root, &GOOD);
    assert!(
        check::rust_refs(&root, &templates()).len() == 2,
        "references の実装が2つとも無い"
    );
    std::fs::write(
        root.join("tool.json"),
        r#"{"contract": 2, "cli": {"command": "bin/x"}, "external": []}"#,
    )
    .expect("書ける");
    let lines = check::source(&root, &templates());
    assert!(
        lines
            .iter()
            .any(|l| l.state == State::Fail && l.text.contains("references の実装が無い")),
        "{lines:?}"
    );
    std::fs::write(
        root.join("tool.json"),
        r#"{"cli": {"command": "bin/x"}, "external": []}"#,
    )
    .expect("書ける");
    let lines = check::source(&root, &templates());
    assert!(
        lines.iter().all(|l| l.state != State::Fail),
        "版1 には当てない ── {lines:?}"
    );
}
