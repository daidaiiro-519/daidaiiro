// SPDX-License-Identifier: MIT
//! qa-advisor のプレゼンテーション層（CLI）。シェルから呼ぶ唯一の経路である。
//!
//!     qa-advisor <動詞> [対象…] [--json]
//!
//! **道具ごとに実行ファイルを作らない** ── 実行ファイルが増えると、呼ぶ側が形を推測することになる。
//! 依存の向きは `Cargo.toml` が宣言する ── この crate はサービス層だけを参照する。

use std::process::ExitCode;

use qa_service::{catalog, tools, Given, Outcome, Tool};

/// 旗と位置引数を読み、渡された引数を組む。
///
/// **旗は `--名前=値` と `--名前 値` の両方を受ける** ── 書き方を1つに強制すると、
/// 既存の手順が壊れる。**値を伴わない旗は、立っている** ── 空にすると、`--check` が
/// 黙って逆の意味になる。**次が別の旗なら、それは値ではない。**
///
/// **まとめて受ける引数は、繰り返すと足りていく。** 上書きにすると、2つ目以降を黙って
/// 捨てることになる（実測で `--layer` が1件しか残らなかった）。
fn read_args(tool: &Tool, rest: &[String]) -> Result<Given, String> {
    let mut given = Given::default();
    let mut positional: Vec<String> = Vec::new();
    let mut i = 0;
    while i < rest.len() {
        let a = &rest[i];
        if let Some(body) = a.strip_prefix("--") {
            let (key, value) = match body.split_once('=') {
                Some((k, v)) => (k.to_owned(), v.to_owned()),
                None => match rest.get(i + 1) {
                    Some(v) if !v.starts_with("--") => {
                        i += 1;
                        (body.to_owned(), v.clone())
                    }
                    _ => (body.to_owned(), "1".to_owned()),
                },
            };
            let key = key.replace('-', "_");
            // **道具の一覧に無い旗は断る。** 黙って無視すると、打ち間違いが検出されない
            if !tool.args.iter().any(|a| a.name == key) {
                let known: Vec<&str> = tool.args.iter().map(|a| a.name).collect();
                return Err(format!(
                    "その旗は無い: --{key}（{} は {} を受ける）",
                    tool.name,
                    known.join(" ・ ")
                ));
            }
            given.push(&key, value);
        } else {
            positional.push(a.clone());
        }
        i += 1;
    }
    // **位置と旗を混ぜて渡せる。** 旗で渡したぶんを数えずに位置だけで判定すると、
    // 旗を使った呼び方が「引数が足りない」になる
    let mut free = positional.into_iter().peekable();
    for arg in &tool.args {
        if given.has(arg.name) {
            continue;
        }
        if arg.many {
            // **まとめて受ける引数は、残りの位置引数を全部取る**
            for v in free.by_ref() {
                given.push(arg.name, v);
            }
            continue;
        }
        if let Some(v) = free.next() {
            given.push(arg.name, v);
        } else if let Some(d) = arg.default {
            given.push(arg.name, d.to_owned());
        }
    }
    Ok(given)
}

fn usage(all: &[Tool]) {
    println!("道具の一覧");
    for t in all {
        let need: Vec<String> = t
            .args
            .iter()
            .map(|a| {
                if a.many {
                    format!("<{}…>", a.name)
                } else if a.required {
                    format!("<{}>", a.name)
                } else {
                    format!("[{}]", a.name)
                }
            })
            .collect();
        println!("  {} {}\n      {}", t.name, need.join(" "), t.summary);
    }
    println!("\n  どれも --json を付けると、機械が読む形で出る");
}

fn emit(out: &Outcome, as_json: bool, human: fn(&Outcome) -> String) -> ExitCode {
    if as_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&out.to_json()).unwrap_or_default()
        );
    } else {
        println!("{}", human(out));
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    ExitCode::from(out.exit_code() as u8)
}

/// 動詞の前に置いた旗（`--名前 値`）を、引数の末尾へ移す。**旗の位置で失敗させない** ──
/// `--skill_root <場所> get …` の順でも、`get … --skill_root <場所>` と同じに読む。
fn verb_first(argv: Vec<String>) -> Vec<String> {
    let mut lead = Vec::new();
    let mut rest = argv.into_iter().peekable();
    while let Some(flag) = rest.next_if(|a| a.starts_with("--")) {
        lead.push(flag);
        if let Some(value) = rest.next_if(|a| !a.starts_with("--")) {
            lead.push(value);
        }
    }
    let mut out: Vec<String> = rest.collect();
    out.extend(lead);
    out
}

fn main() -> ExitCode {
    let all = tools();
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let as_json = argv.iter().any(|a| a == "--json");
    let argv: Vec<String> = argv.into_iter().filter(|a| a != "--json").collect();
    let argv = verb_first(argv);

    let Some(verb) = argv.first() else {
        // **動詞なしで `--json` を付けたら、道具の一覧を返す。** 検査は実行ファイルを起動するだけで、
        // 言語に依存せずに道具の一覧を読める
        if as_json {
            return emit(&catalog(&all, &Given::default()), true, |_| String::new());
        }
        usage(&all);
        return ExitCode::from(2);
    };
    if matches!(verb.as_str(), "-h" | "--help" | "help") {
        usage(&all);
        return ExitCode::SUCCESS;
    }
    let Some(tool) = all.iter().find(|t| t.name == verb) else {
        eprintln!("その動詞は無い: {verb} ── 一覧は help である");
        return ExitCode::from(2);
    };
    let given = match read_args(tool, &argv[1..]) {
        Ok(given) => given,
        Err(why) => {
            eprintln!("{why}");
            return ExitCode::from(2);
        }
    };
    let need: Vec<&str> = tool
        .args
        .iter()
        .filter(|a| a.required)
        .map(|a| a.name)
        .collect();
    if let Some(missing) = need.iter().find(|n| !given.has(n)) {
        eprintln!("引数が足りない: {} は {missing} を要する", tool.name);
        return ExitCode::from(2);
    }
    let out = (tool.run)(&given);
    emit(&out, as_json, tool.human)
}
