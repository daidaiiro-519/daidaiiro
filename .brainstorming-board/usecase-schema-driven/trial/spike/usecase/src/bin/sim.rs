//! 完成したときの一日を、実物で流す ── **文章ではなく、道具の出力そのものである。**
//!
//! 画面へ出るものは全部、宣言と変換機と引き算から実際に出ている。
use base::{Bindings, Finding};
use std::collections::BTreeSet;

fn 見出し(n: &str, s: &str) {
    println!("\n@@{n}@@{s}");
}
fn 行(s: &str) {
    println!("{s}");
}
/// 仕様を、根から実際に読む ── **バイナリへ焼き込まない**。
///
/// 焼き込むと、仕様を直したときに、この画面だけが古い中身を出す。
/// **しかも「出た」ので、誰も気づかない。**
fn 仕様を読む(path: &str) -> String {
    let p = usecase::spec_root().join(path);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{} を読めない ── {e}", p.display()))
}

fn 出力(cmd: &str, body: &str) {
    println!("@@cmd@@{cmd}");
    println!("@@out@@{body}");
}

fn 引き算(b: &Bindings, tests: &BTreeSet<String>, placed: &BTreeSet<String>) -> Vec<Finding> {
    let decls = usecase::decls();
    let mut out = base::audit_bindings(&base::node_ids(&decls), b, tests);
    let mut files = Bindings::new();
    for f in usecase::emitted_files() {
        files.bind(&f, &f);
    }
    out.extend(base::audit_bindings(
        &usecase::emitted_files(),
        &files,
        placed,
    ));
    out
}

fn 結果(found: &[Finding]) -> String {
    if found.is_empty() {
        "見つかったこと 0 件 ── 仕様の外に、何も存在しない".to_string()
    } else {
        let mut s = String::new();
        for f in found {
            s.push_str(&format!(
                "{}  ／ {}\n",
                f.line(),
                if f.always_zero() {
                    "常時ゼロ（落とす）"
                } else {
                    "承認の境目でゼロ"
                }
            ));
        }
        s.push_str(&format!("見つかったこと {} 件", found.len()));
        s
    }
}

fn main() {
    let decls = usecase::decls();
    let reg = usecase::wire();
    let agg = decls.iter().find(|d| d.name == "出荷指示").unwrap();

    段1_仕様を書く();
    段2_射影を出す(agg, &reg);
    段34_承認して申告する();
    段5_引き算する();
    段6_壊してみる();
    段7_直す();
}

/// 正本は宣言1つである ── 人はこれを直に読まない。
fn 段1_仕様を書く() {
    見出し("1", "人が仕様を書く ── 正本はこれ1つである");
    行("形式は構造化データ（JSON）。人はこれを直に読まない。");
    行("事業領域で切ったフォルダの根に、事業領域の宣言を必ず置く ── これは契約である。");
    出力(
        "$ ls spec/宅配のサービス/",
        &usecase::spec_files()
            .iter()
            .map(|f| {
                if f.at_root() {
                    format!("{}   ← 根（全体のコンセプト）", f.path)
                } else {
                    f.path.clone()
                }
            })
            .collect::<Vec<_>>()
            .join("\n"),
    );
    出力(
        "$ cat spec/宅配のサービス/business-domain.json",
        &仕様を読む("business-domain.json")
            .lines()
            .take(16)
            .collect::<Vec<_>>()
            .join("\n"),
    );
}

/// 同じ宣言から、形式の違う射影が出る。
fn 段2_射影を出す(agg: &base::Decl, reg: &base::Registry) {
    見出し(
        "2",
        "道具が射影を出す ── 同じ宣言から、形式の違うものが出る",
    );
    行("出す先は 宣言 × 変換機 で決まる。ここでは宣言2件 × 出す先4件 ＝ 8件である。");
    出力(
        "$ schema emit",
        &usecase::emitted_files()
            .into_iter()
            .collect::<Vec<_>>()
            .join("\n"),
    );
    出力(
        "$ schema render --as 承認   # 人が読むのは、これだけ",
        &base::render(agg, reg)
            .unwrap()
            .lines()
            .take(12)
            .collect::<Vec<_>>()
            .join("\n"),
    );
    出力(
        "$ cat .codex/agents/AGG-01J7Q4K.toml   # 実行系が読む形（TOML）",
        base::render_as(agg, reg, "render.codex.agent.toml")
            .unwrap()
            .trim_end(),
    );
}

/// 人が読むのは HTML だけで、AI の申告は MCP の1操作である。
fn 段34_承認して申告する() {
    見出し("3", "人が承認する ── 読むのは HTML だけである");
    行("承認したのは、シナリオ SC-01J7Q4M と SC-01J7Q4N の2本である。");

    見出し("4", "AI が実装し、テストを書き、結んだと申告する");
    行("申告は MCP の1操作で行う。道具はソースを1行も読まない。");
    出力(
        "> bind(node=\"SC-01J7Q4M\", external=\"usecase::tests::明細が0件のとき確定できない\")\n> bind(node=\"SC-01J7Q4N\", external=\"usecase::tests::明細が1件あれば確定できる\")",
        "対応表へ2行入った",
    );
    行("対応表は、道具が持つ1つのファイルである。形式はまだ決めていない ── ここでは JSON で書き出した。");
    出力("$ cat .schema/bindings.json", &{
        let rows: Vec<String> = usecase::bindings()
            .pairs()
            .iter()
            .map(|(k, v)| format!("    {k:?}: [{v:?}]"))
            .collect();
        format!("{{\n  \"bindings\": {{\n{}\n  }}\n}}", rows.join(",\n"))
    });
}

/// CI で走る引き算 ── 仕様の外に在るものだけが出る。
fn 段5_引き算する() {
    見出し("5", "引き算する ── これが CI で走る");
    出力(
        "$ schema check",
        &結果(&引き算(
            &usecase::bindings(),
            &usecase::test_list(),
            &usecase::placed_files(),
        )),
    );
}

/// 4通りの壊れ方が、それぞれ別の行で出ることを確認する。
fn 段6_壊してみる() {
    見出し("6", "壊してみる ── 4通りの壊れ方が、それぞれ別の行で出る");

    行("① シナリオを1本足したが、テストをまだ書いていない");
    let mut decls2 = usecase::test_list();
    出力(
        "$ schema check",
        &結果(&{
            let mut b = Bindings::new();
            b.bind("SC-01J7Q4M", "usecase::tests::明細が0件のとき確定できない");
            引き算(&b, &decls2, &usecase::placed_files())
        }),
    );

    行("② 誰も宣言していないテストが在る");
    decls2.insert("usecase::tests::こっそり足した".into());
    出力(
        "$ schema check",
        &結果(&引き算(
            &usecase::bindings(),
            &decls2,
            &usecase::placed_files(),
        )),
    );

    行("③ テストの名前を変えた ── 対応が切れる");
    let mut t3 = usecase::test_list();
    t3.remove("usecase::tests::明細が1件あれば確定できる");
    t3.insert("usecase::tests::明細が1件あれば確定できること".into());
    出力(
        "$ schema check",
        &結果(&引き算(
            &usecase::bindings(),
            &t3,
            &usecase::placed_files(),
        )),
    );

    行("④ 宣言していない Skill を、手で置いた");
    let mut placed = usecase::placed_files();
    placed.insert(".claude/skills/手で置いた/SKILL.md".into());
    出力(
        "$ schema check",
        &結果(&引き算(
            &usecase::bindings(),
            &usecase::test_list(),
            &placed,
        )),
    );
}

/// 直す手は2つしか無い ── 仕様へ入れるか、成果物を消すか。
fn 段7_直す() {
    見出し("7", "直す ── 仕様を直すか、成果物を消すかのどちらかである");
    行("どの行も「仕様の外に在るもの」を指している。消すか、仕様へ入れるかしかない。");
    出力(
        "$ schema check",
        &結果(&引き算(
            &usecase::bindings(),
            &usecase::test_list(),
            &usecase::placed_files(),
        )),
    );
}
