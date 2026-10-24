//! 小さな workspace を一時ディレクトリに作り、実行ファイルを起動して確かめる。
//! 規則は examples/semantic-agent-schema.json（最初の利用者の規則）を使う。

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const BIN: &str = env!("CARGO_BIN_EXE_architecture-lint");
const ROOT: &str = env!("CARGO_MANIFEST_DIR");

/// crate の名前と、Cargo.toml の [dependencies] ・ [dev-dependencies] に書く行
struct Krate<'a> {
    name: &'a str,
    deps: &'a [&'a str],
    dev: &'a [&'a str],
}

fn workspace(tag: &str, krates: &[Krate]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "architecture-lint-test-{tag}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    let members: Vec<String> = krates.iter().map(|k| format!("\"{}\"", k.name)).collect();
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("Cargo.toml"),
        format!(
            "[workspace]\nmembers = [{}]\nresolver = \"2\"\n",
            members.join(", ")
        ),
    )
    .unwrap();
    for k in krates {
        let d = dir.join(k.name);
        fs::create_dir_all(d.join("src")).unwrap();
        fs::write(d.join("src/lib.rs"), "").unwrap();
        let line = |dep: &&str| {
            if dep.starts_with("semantic-agent-schema-") {
                format!("{dep} = {{ path = \"../{dep}\" }}")
            } else {
                format!("{dep} = \"1\"")
            }
        };
        let deps: Vec<String> = k.deps.iter().map(line).collect();
        let dev: Vec<String> = k.dev.iter().map(line).collect();
        fs::write(
            d.join("Cargo.toml"),
            format!(
                "[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\n{}\n\n[dev-dependencies]\n{}\n",
                k.name,
                deps.join("\n"),
                dev.join("\n")
            ),
        )
        .unwrap();
    }
    dir
}

fn rules() -> PathBuf {
    Path::new(ROOT).join("examples/semantic-agent-schema.json")
}

fn run(args: &[&str]) -> Output {
    Command::new(BIN).args(args).output().unwrap()
}

fn good() -> Vec<Krate<'static>> {
    vec![
        Krate {
            name: "semantic-agent-schema-declaration",
            deps: &[],
            dev: &[],
        },
        Krate {
            name: "semantic-agent-schema-change",
            deps: &["semantic-agent-schema-declaration"],
            dev: &[],
        },
        Krate {
            name: "semantic-agent-schema-application",
            deps: &[
                "semantic-agent-schema-change",
                "semantic-agent-schema-declaration",
            ],
            dev: &[],
        },
        Krate {
            name: "semantic-agent-schema-fs",
            deps: &["semantic-agent-schema-application"],
            dev: &[],
        },
        Krate {
            name: "semantic-agent-schema-sqlite",
            deps: &["semantic-agent-schema-application", "rusqlite"],
            dev: &["tempfile"],
        },
    ]
}

fn schema(name: &str) -> jsonschema::Validator {
    let s: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(Path::new(ROOT).join(name)).unwrap()).unwrap();
    jsonschema::validator_for(&s).unwrap()
}

#[test]
fn 規則どおりの_workspace_は終了コード_0() {
    let ws = workspace("good", &good());
    let out = run(&[
        "check",
        "--rules",
        rules().to_str().unwrap(),
        "--cargo",
        ws.to_str().unwrap(),
    ]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
}

#[test]
fn 許していない辺は終了コード_1() {
    let mut ks = good();
    // change → fs（domain から outbound）・ sqlite が tempfile を本体で使う ・ どこにも入らない serde
    ks[1] = Krate {
        name: "semantic-agent-schema-change",
        deps: &[
            "semantic-agent-schema-declaration",
            "semantic-agent-schema-fs",
            "serde",
        ],
        dev: &[],
    };
    ks[4] = Krate {
        name: "semantic-agent-schema-sqlite",
        deps: &["semantic-agent-schema-application", "rusqlite", "tempfile"],
        dev: &[],
    };
    let ws = workspace("bad", &ks);
    let out = run(&[
        "check",
        "--rules",
        rules().to_str().unwrap(),
        "--cargo",
        ws.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(out.status.code(), Some(1));
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(
        schema("result.schema.json").is_valid(&v),
        "結果は result.schema.json に合う"
    );
    let got: Vec<(String, String, String)> = v["violations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| {
            (
                x["reason"].as_str().unwrap().into(),
                x["from"].as_str().unwrap().into(),
                x["to"].as_str().unwrap().into(),
            )
        })
        .collect();
    let want = |r: &str, f: &str, t: &str| (r.to_string(), f.to_string(), t.to_string());
    assert_eq!(
        got,
        vec![
            want(
                "not-in-allowed",
                "semantic-agent-schema-change",
                "semantic-agent-schema-fs"
            ),
            want("not-in-allowed", "semantic-agent-schema-sqlite", "tempfile"),
            want("uncovered", "semantic-agent-schema-change", "serde"),
        ]
    );
}

#[test]
fn graph_の出力を_check_の_graph_に渡すと同じ結果() {
    let ws = workspace("roundtrip", &good());
    let g = run(&["graph", "--cargo", ws.to_str().unwrap()]);
    assert_eq!(g.status.code(), Some(0));
    let gv: serde_json::Value = serde_json::from_slice(&g.stdout).unwrap();
    assert!(
        schema("graph.schema.json").is_valid(&gv),
        "グラフは graph.schema.json に合う"
    );
    let mut child = Command::new(BIN)
        .args([
            "check",
            "--rules",
            rules().to_str().unwrap(),
            "--graph",
            "-",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(&g.stdout).unwrap();
    assert_eq!(child.wait_with_output().unwrap().status.code(), Some(0));
}

#[test]
fn 規則の例は_rules_schema_に合う() {
    let v: serde_json::Value = serde_json::from_str(&fs::read_to_string(rules()).unwrap()).unwrap();
    assert!(schema("rules.schema.json").is_valid(&v));
}

#[test]
fn 読めない規則と使い方の誤りは終了コード_2() {
    let ws = workspace("err", &good());
    let bad =
        std::env::temp_dir().join(format!("architecture-lint-bad-{}.json", std::process::id()));
    fs::write(
        &bad,
        r#"{"version":1,"components":[],"allow":[{"from":"x","to":["y"]}]}"#,
    )
    .unwrap();
    let out = run(&[
        "check",
        "--rules",
        bad.to_str().unwrap(),
        "--cargo",
        ws.to_str().unwrap(),
    ]);
    assert_eq!(
        out.status.code(),
        Some(2),
        "知らない component を指す allow"
    );
    fs::write(&bad, r#"{"version":2,"components":[],"allow":[]}"#).unwrap();
    assert_eq!(
        run(&[
            "check",
            "--rules",
            bad.to_str().unwrap(),
            "--cargo",
            ws.to_str().unwrap()
        ])
        .status
        .code(),
        Some(2),
        "読めない版"
    );
    assert_eq!(
        run(&[
            "check",
            "--rules",
            rules().to_str().unwrap(),
            "--cargo",
            "/nonexistent"
        ])
        .status
        .code(),
        Some(2),
        "cargo metadata の失敗"
    );
    assert_eq!(
        run(&["check", "--unknown"]).status.code(),
        Some(2),
        "知らない旗"
    );
}

#[test]
fn help_と_version() {
    assert_eq!(run(&["--help"]).status.code(), Some(0));
    assert!(String::from_utf8_lossy(&run(&["--version"]).stdout).starts_with("architecture-lint "));
}
