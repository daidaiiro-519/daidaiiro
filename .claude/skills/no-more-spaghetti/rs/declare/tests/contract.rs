// SPDX-License-Identifier: MIT
//! 依存の向きの契約を、**すべての言語へ同じ形で**課す。
//!
//! どの義務を課すかは、言語の表の契約の欄（`bridge` ・ `dynamic` ・ `branches`）が決める。
//! **表に言語を1つ足すと、この試験がその言語にも課される** ── 試験データが無ければ失敗する。
//!
//!     cargo test -p nms_declare --test contract

use std::path::{Path, PathBuf};

use nms_declare::{tools, Given, Outcome};
use nms_parts::inward::syntax::{table, Branches, Dynamic};

/// 試験データ1つ。**試験データは、どの層にも属さない `rs/fixtures/` に置く** ──
/// 違反を意図して含むので、層の中へ置くと、この Skill 自身の検査がそれを検出する。
///
/// 試験データ1つ。層は内から外の並びで、core（内）・ gen（中身の無い層）・ adapter（外）である。
struct Case {
    language: &'static str,
    core: &'static str,
    gen: &'static str,
    adapter: &'static str,
    /// 条件で分かれる読み込みの、2つの行き先。
    fast: &'static str,
    slow: &'static str,
    /// 読めない設定を作る ── （ファイル, 中身）。
    broken: (&'static str, &'static str),
}

const CASES: [Case; 10] = [
    Case {
        language: "python",
        core: "src.app.core",
        gen: "src.app.gen",
        adapter: "src.app.adapter",
        fast: "src.app.core.fast",
        slow: "src.app.core.slow",
        broken: ("pyproject.toml", "[tool"),
    },
    Case {
        language: "rust",
        core: "core",
        gen: "gen",
        adapter: "adapter",
        fast: "core.src.fast",
        slow: "core.src.slow",
        broken: ("core/Cargo.toml", "[package"),
    },
    Case {
        language: "typescript",
        core: "src/core",
        gen: "src/gen",
        adapter: "src/adapter",
        fast: "src/core/fast",
        slow: "src/core/slow",
        broken: ("tsconfig.json", "{ \"compilerOptions\": "),
    },
    Case {
        language: "java",
        core: "com.acme.core",
        gen: "com.acme.gen",
        adapter: "com.acme.adapter",
        fast: "",
        slow: "",
        broken: ("", ""),
    },
    Case {
        language: "kotlin",
        core: "com.acme.core",
        gen: "com.acme.gen",
        adapter: "com.acme.adapter",
        fast: "",
        slow: "",
        broken: ("", ""),
    },
    Case {
        language: "csharp",
        core: "Acme.Core",
        gen: "Acme.Gen",
        adapter: "Acme.Adapter",
        fast: "Acme.Core.Fast",
        slow: "Acme.Core.Slow",
        broken: ("", ""),
    },
    Case {
        language: "php",
        core: "App\\Core",
        gen: "App\\Gen",
        adapter: "App\\Adapter",
        fast: "",
        slow: "",
        broken: ("", ""),
    },
    Case {
        language: "go",
        core: "core",
        gen: "gen",
        adapter: "adapter",
        fast: "core/fast",
        slow: "core/slow",
        broken: ("go.mod", "go 1.22\n"),
    },
    Case {
        language: "ruby",
        core: "lib/app/core",
        gen: "lib/app/gen",
        adapter: "lib/app/adapter",
        fast: "lib/app/core/fast",
        slow: "lib/app/core/slow",
        broken: ("app.gemspec", ""),
    },
    Case {
        language: "cpp",
        core: "include/app/core",
        gen: "include/app/gen",
        adapter: "src/adapter",
        fast: "include/app/core/fast",
        slow: "include/app/core/slow",
        broken: (
            "CMakeLists.txt",
            "include_directories(${UNKNOWN_DIR}/inc)\n",
        ),
    },
];

fn fixture(language: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../fixtures/contract")
        .join(language)
}

fn inward(language: &str, root: &Path, layers: &[(&str, &str)]) -> Outcome {
    let tool = tools()
        .into_iter()
        .find(|t| t.name == "inward")
        .expect("在る");
    let mut given = Given::default();
    given.push("language", language.to_owned());
    given.push("root", root.display().to_string());
    for (name, id) in layers {
        given.push("layer", format!("{name}={id}"));
    }
    (tool.run)(&given)
}

fn has(out: &Outcome, needle: &str) -> bool {
    out.findings.iter().any(|f| f.contains(needle))
}

fn copy(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("作れる");
    for e in std::fs::read_dir(from).expect("読める").flatten() {
        let p = e.path();
        let q = to.join(e.file_name());
        if p.is_dir() {
            copy(&p, &q);
        } else {
            std::fs::copy(&p, &q).expect("写せる");
        }
    }
}

#[test]
fn every_language_in_the_table_has_a_case() {
    // **表に言語を足したら、試験データも足す** ── 足さなければ、契約を課されないまま通る
    for s in table() {
        assert!(
            CASES.iter().any(|c| c.language == s.language),
            "{} に契約の試験データが無い",
            s.language
        );
    }
}

#[test]
fn every_language_meets_the_contract() {
    let mut broken_all = Vec::new();
    for s in table() {
        let c = CASES
            .iter()
            .find(|c| c.language == s.language)
            .expect("試験データが在る");
        let root = fixture(c.language);
        let lang = c.language;
        let order = [("core", c.core), ("gen", c.gen), ("adapter", c.adapter)];
        let out = inward(lang, &root, &order);
        let mut broken = Vec::new();
        // 義務1：正しい並びでは、向きの違反が無い
        if has(&out, "内側が外側") {
            broken.push("正しい並びで違反が出た".to_owned());
        }
        // 義務2：作業領域の中を指すのに、どの層にも属さない参照を報告する
        if !has(&out, "どの層にも属さない") {
            broken.push("層に属さない参照（stray）を報告しない".to_owned());
        }
        // 義務3：層を指すのに、その層にモジュールが無い参照を報告する
        if !has(&out, "参照先が実在しない") {
            broken.push("実在しない参照先（gen）を報告しない".to_owned());
        }
        // 義務4：名前を実行時に決める読み込みを、静的に追跡できない読み込みとして報告する
        if matches!(s.dynamic, Dynamic::Calls { .. }) && !has(&out, "実行時に決まる") {
            broken.push("名前を実行時に決める読み込みを報告しない".to_owned());
        }
        // 義務5：条件で分かれる読み込みは、すべての分岐を依存にする
        if matches!(s.branches, Branches::AllBranches(_)) {
            let edges = out.data["edge_list"]
                .as_array()
                .cloned()
                .unwrap_or_default();
            for want in [c.fast, c.slow] {
                if !edges
                    .iter()
                    .any(|e| e[1].as_str().is_some_and(|t| t.starts_with(want)))
                {
                    broken.push(format!("条件の分岐の行き先 {want} を依存にしていない"));
                }
            }
        }
        // 義務6：逆の並びでは、向きの違反を出す
        let reversed = [("adapter", c.adapter), ("gen", c.gen), ("core", c.core)];
        if !has(&inward(lang, &root, &reversed), "内側が外側") {
            broken.push("逆の並びで違反が出ない".to_owned());
        }
        // 義務7：名前空間をつなぐ設定を読めなければ、報告する
        if s.bridge.is_some() {
            let tmp = std::env::temp_dir().join(format!("nms-contract-{lang}"));
            let _ = std::fs::remove_dir_all(&tmp);
            copy(&root, &tmp);
            let (file, body) = c.broken;
            let target = tmp.join(file);
            if body.is_empty() {
                // 中身で壊せない設定（文字列として読むもの）は、読む権限を外す
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o000))
                    .expect("変えられる");
            } else {
                std::fs::write(&target, body).expect("書ける");
            }
            if !has(&inward(lang, &tmp, &order), "設定を読めない") {
                broken.push(format!("読めない設定（{file}）を報告しない"));
            }
        }
        if !broken.is_empty() {
            broken_all.push(format!("{lang}: {}", broken.join(" ／ ")));
        }
    }
    assert!(
        broken_all.is_empty(),
        "契約を満たさない言語がある:\n{}",
        broken_all.join("\n")
    );
}

#[test]
fn a_runtime_load_outside_every_layer_is_not_reported() {
    // 層に属さないファイルは、向きの規則を課されない ── 参照元が層に属すときだけ報告する
    // （実測 ── 違反を意図して含む試験データが、この Skill 自身の検査で検出された）
    let tmp = std::env::temp_dir().join("nms-contract-unlayered-escape");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(tmp.join("src/core")).expect("作れる");
    std::fs::create_dir_all(tmp.join("other")).expect("作れる");
    std::fs::write(tmp.join("src/core/a.ts"), "export const a = 1;\n").expect("書ける");
    std::fs::write(
        tmp.join("other/x.ts"),
        "const n = 'a';\nexport const m = import(n);\n",
    )
    .expect("書ける");
    let out = inward("typescript", &tmp, &[("core", "src/core")]);
    assert!(!has(&out, "実行時に決まる"), "{:?}", out.findings);
    // 同じ読み込みを層の中へ置けば、報告する
    std::fs::write(
        tmp.join("src/core/b.ts"),
        "const n = 'a';\nexport const m = import(n);\n",
    )
    .expect("書ける");
    let out = inward("typescript", &tmp, &[("core", "src/core")]);
    assert!(has(&out, "実行時に決まる"), "{:?}", out.findings);
}
