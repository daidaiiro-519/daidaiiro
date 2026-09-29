// SPDX-License-Identifier: MIT
//! 契約の検査を事例で検証する。
//!
//!     cargo test -p sc_parts

use std::path::{Path, PathBuf};

use sc_parts::check::{self, Templates};

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

/// 雛形が要求する節だけを持つ文書を置く。
fn write_document(root: &Path) {
    let template = std::fs::read_to_string(templates().general).expect("読める");
    let body: String = sc_parts::sections::headings(&template)
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

const GOOD: [(&str, &[&str]); 3] = [
    ("parts", &[]),
    ("declare", &["parts"]),
    ("cli", &["declare"]),
];

#[test]
fn a_skill_with_layers_as_crates_passes() {
    let root = scratch("ok");
    write_document(&root);
    write_layers(&root, &GOOD);
    let found = check::check(&root, &templates()).expect("検査できる");
    assert!(found.is_empty(), "食い違いが出た ── {found:?}");
}

#[test]
fn a_skill_without_tools_needs_no_layers() {
    let root = scratch("advice");
    write_document(&root);
    let found = check::check(&root, &templates()).expect("検査できる");
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
    let found = check::check(&root, &templates()).expect("検査できる");
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
    let found = check::check(&root, &templates()).expect("検査できる");
    assert!(
        found.iter().any(|x| x.contains("crate に分かれていない")),
        "1つの単位の中の module では、内側が外側を参照してもコンパイラが通す ── {found:?}"
    );
}

#[test]
fn an_edge_that_goes_outward_is_reported() {
    let root = scratch("outward");
    write_document(&root);
    // **部品が入口を参照する宣言** ── 内から外である
    write_layers(
        &root,
        &[
            ("parts", &["cli"]),
            ("declare", &["parts"]),
            ("cli", &["declare"]),
        ],
    );
    let found = check::check(&root, &templates()).expect("検査できる");
    assert!(
        found
            .iter()
            .any(|x| x.contains("許可していない辺") && x.contains("parts → entry")),
        "{found:?}"
    );
}

#[test]
fn an_edge_that_skips_inward_is_allowed() {
    let root = scratch("skip");
    write_document(&root);
    // 入口が部品を直に参照する ── 外から内なので許す
    write_layers(
        &root,
        &[
            ("parts", &[]),
            ("declare", &["parts"]),
            ("cli", &["declare", "parts"]),
        ],
    );
    let found = check::check(&root, &templates()).expect("検査できる");
    assert!(found.is_empty(), "外から内は許す ── {found:?}");
}

#[test]
fn a_missing_layer_is_reported() {
    let root = scratch("missing");
    write_document(&root);
    write_layers(&root, &[("parts", &[]), ("cli", &["parts"])]);
    let found = check::check(&root, &templates()).expect("検査できる");
    assert!(
        found
            .iter()
            .any(|x| x.contains("層の crate が無い") && x.contains("declare")),
        "{found:?}"
    );
}

#[test]
fn a_missing_entry_is_reported() {
    let root = scratch("noentry");
    write_document(&root);
    write_layers(&root, &[("parts", &[]), ("declare", &["parts"])]);
    let found = check::check(&root, &templates()).expect("検査できる");
    assert!(
        found.iter().any(|x| x.contains("入口の crate が無い")),
        "{found:?}"
    );
}

#[test]
fn absent_examples_are_reported() {
    let root = scratch("notests");
    write_document(&root);
    write_layers(&root, &GOOD);
    std::fs::remove_dir_all(root.join(check::TOOL).join(check::TESTS)).expect("消せる");
    let found = check::check(&root, &templates()).expect("検査できる");
    assert!(found.iter().any(|x| x.contains("事例が無い")), "{found:?}");
}

#[test]
fn a_missing_section_is_reported() {
    let root = scratch("section");
    write_layers(&root, &GOOD);
    std::fs::write(root.join("SKILL.md"), "## 目的\n\n本文\n").expect("書ける");
    let found = check::check(&root, &templates()).expect("検査できる");
    assert!(found.iter().any(|x| x.contains("節が無い")), "{found:?}");
}

#[test]
fn an_absent_document_is_reported() {
    let root = scratch("nodoc");
    write_layers(&root, &GOOD);
    let found = check::check(&root, &templates()).expect("検査できる");
    assert!(found.iter().any(|x| x.contains("文書が無い")), "{found:?}");
}

/// 部品に1つのファイルを置く。
fn write_part(root: &Path, body: &str) {
    let dir = root.join(check::TOOL).join("parts/src");
    std::fs::create_dir_all(&dir).expect("作れる");
    std::fs::write(dir.join("run.rs"), body).expect("書ける");
}

/// 宣言に、目的に不可欠な外部の道具を、理由と一緒に書く。
fn write_requires(root: &Path, names: &[&str]) {
    let listed: Vec<String> = names
        .iter()
        .map(|n| format!("(\"{n}\", \"この Skill の目的に不可欠である\")"))
        .collect();
    write_external(root, &listed.join(", "));
}

/// 宣言をそのまま書く。
fn write_external(root: &Path, items: &str) {
    let dir = root.join(check::TOOL).join("declare/src");
    std::fs::create_dir_all(&dir).expect("作れる");
    std::fs::write(
        dir.join("lib.rs"),
        format!("pub const EXTERNAL: &[(&str, &str)] = &[{items}];\n"),
    )
    .expect("書ける");
}

#[test]
fn an_external_tool_without_a_reason_is_reported() {
    // **外部の道具は例外である** ── 目的に不可欠だという理由を書かない宣言を、通さない
    let root = scratch("no-reason");
    write_document(&root);
    write_layers(&root, &GOOD);
    write_external(&root, "(\"git\", \"\")");
    write_part(&root, "fn f() { std::process::Command::new(\"git\"); }\n");
    let found = check::check(&root, &templates()).expect("検査できる");
    assert!(
        found
            .iter()
            .any(|x| x.contains("理由") && x.contains("git")),
        "{found:?}"
    );
}

#[test]
fn a_remaining_rs_folder_is_reported() {
    // **道具のソースは tool/ に置く** ── rs/ は言語の名前で、中身の役割を示さない
    let root = scratch("rs-left");
    write_document(&root);
    write_layers(&root, &GOOD);
    std::fs::create_dir_all(root.join("rs")).expect("作れる");
    let found = check::check(&root, &templates()).expect("検査できる");
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
    let found = check::check(&root, &templates()).expect("検査できる");
    assert!(found.iter().any(|x| x.contains("bin/")), "{found:?}");
}

#[test]
fn an_absolute_path_in_the_registration_is_reported() {
    // **登録に開発機の絶対パスを書かない** ── 配布先では存在しない場所を指す
    let root = scratch("abs");
    write_document(&root);
    write_layers(&root, &GOOD);
    std::fs::write(
        root.join("mcp.json"),
        r#"{"mcpServers":{"x":{"command":"/home/me/x/bin/x-mcp","args":[]}}}"#,
    )
    .expect("書ける");
    let found = check::check(&root, &templates()).expect("検査できる");
    assert!(found.iter().any(|x| x.contains("絶対パス")), "{found:?}");
}

#[test]
fn a_command_missing_on_some_os_is_reported() {
    // **OS によって無いコマンドを呼ばない** ── date は Windows に実行ファイルとして無く、
    // timeout は macOS の標準に無い
    let root = scratch("date");
    write_document(&root);
    write_layers(&root, &GOOD);
    write_requires(&root, &["date"]);
    write_part(
        &root,
        "fn today() { std::process::Command::new(\"date\"); }\n",
    );
    let found = check::check(&root, &templates()).expect("検査できる");
    assert!(
        found
            .iter()
            .any(|x| x.contains("OS によって無い") && x.contains("date")),
        "{found:?}"
    );
}

#[test]
fn a_tool_outside_the_declaration_is_reported() {
    let root = scratch("undeclared");
    write_document(&root);
    write_layers(&root, &GOOD);
    write_requires(&root, &[]);
    write_part(&root, "fn f() { std::process::Command::new(\"git\"); }\n");
    let found = check::check(&root, &templates()).expect("検査できる");
    assert!(
        found
            .iter()
            .any(|x| x.contains("宣言に無い外部の道具") && x.contains("git")),
        "{found:?}"
    );
    // 宣言すれば通る
    write_requires(&root, &["git"]);
    let found = check::check(&root, &templates()).expect("検査できる");
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn a_tool_named_at_run_time_needs_its_own_declaration() {
    // **名前が変数で渡される道具は、名前を照合できない** ── 宣言に「利用者が指定する道具」と書く
    let root = scratch("dynamic");
    write_document(&root);
    write_layers(&root, &GOOD);
    write_requires(&root, &[]);
    write_part(&root, "fn f(b: &str) { std::process::Command::new(b); }\n");
    let found = check::check(&root, &templates()).expect("検査できる");
    assert!(
        found.iter().any(|x| x.contains("利用者が指定する道具")),
        "{found:?}"
    );
    write_requires(&root, &[check::USER_CHOSEN]);
    let found = check::check(&root, &templates()).expect("検査できる");
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn a_call_written_inside_a_string_is_not_counted() {
    // **文字列の中の `Command::new(` は、呼び出しではない** ── 呼び出しを探す側の定数が該当する
    let root = scratch("in-string");
    write_document(&root);
    write_layers(&root, &GOOD);
    write_requires(&root, &[]);
    write_part(&root, "const CALL: &str = \"Command::new(\";\n");
    let found = check::check(&root, &templates()).expect("検査できる");
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn examples_are_not_shipped_so_their_tools_are_not_counted() {
    // **cargo の examples は、配布する実行ファイルに入らない** ── 開発用の例が呼ぶ道具を、宣言に求めない
    let root = scratch("examples");
    write_document(&root);
    write_layers(&root, &GOOD);
    write_requires(&root, &[]);
    let dir = root.join(check::TOOL).join("parts/examples");
    std::fs::create_dir_all(&dir).expect("作れる");
    std::fs::write(
        dir.join("bench.rs"),
        "fn f() { std::process::Command::new(\"dot\"); }\n",
    )
    .expect("書ける");
    let found = check::check(&root, &templates()).expect("検査できる");
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn what_scaffold_places_satisfies_check() {
    // **生んだものが、そのまま契約を満たす。** 満たさないと、新しい Skill は必ず
    // 不合格の状態で生まれる（実測 ── 契約を Rust の形へ変えたとき、雛形が Python の
    // ままだったのでそうなった）
    let here = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let tmpl = here.join("references/tool-contract");
    let root = scratch("scaffolded");
    for (from, to) in [
        ("workspace.Cargo.toml.tmpl", "tool/Cargo.toml"),
        ("parts.Cargo.toml.tmpl", "tool/parts/Cargo.toml"),
        ("parts.lib.rs.tmpl", "tool/parts/src/lib.rs"),
        ("parts.tests.rs.tmpl", "tool/parts/tests/example.rs"),
        ("declare.Cargo.toml.tmpl", "tool/declare/Cargo.toml"),
        ("declare.lib.rs.tmpl", "tool/declare/src/lib.rs"),
        ("contract.rs.tmpl", "tool/declare/src/contract.rs"),
        ("cli.Cargo.toml.tmpl", "tool/cli/Cargo.toml"),
        ("mcp.Cargo.toml.tmpl", "tool/mcp/Cargo.toml"),
        ("mcp.json.tmpl", "mcp.json"),
        ("gitignore.tmpl", ".gitignore"),
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
    let found = check::check(&root, &templates()).expect("検査できる");
    assert!(
        found.is_empty(),
        "生んだものが契約を満たしていない ── {found:?}"
    );
}
