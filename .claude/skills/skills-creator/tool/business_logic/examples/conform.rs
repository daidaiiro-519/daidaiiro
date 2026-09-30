// SPDX-License-Identifier: MIT
//! 提供者だけが使う入口 ── 言語の組の references の実装の出力を突き合わせる（ACDR 0070）。
//! **配布する実行ファイルには入れない** ── cargo の example は `cargo install` で組み立てられず、
//! `tool/` は配布物に入らない。利用者の道具の一覧と MCP に、使う場面の無い道具を出さない。
//!
//!     cargo run -q --manifest-path tool/Cargo.toml -p sc_business_logic --example conform -- \
//!         <基準の Skill のフォルダ> <比べる Skill のフォルダ> [<references を持つ Skill の置き場所>]
//!
//! 終了コードは 0 すべて一致 ／ 1 不一致あり ／ 2 誤用である。

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use sc_business_logic::conform;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (Some(base), Some(other)) = (args.first(), args.get(1)) else {
        eprintln!(
            "使い方: conform <基準の Skill> <比べる Skill> [<置き場所>（既定 .claude/skills）]"
        );
        return ExitCode::from(2);
    };
    let corpus = args
        .get(2)
        .map_or_else(|| PathBuf::from(".claude/skills"), PathBuf::from);
    let schema = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../references/profiles/shared/document.schema.json.tmpl");
    match conform::conform(Path::new(base), Path::new(other), &corpus, &schema) {
        Ok(report) => {
            println!(
                "事例 {} 件 ／ 不一致 {} 件",
                report.cases,
                report.mismatches.len()
            );
            for m in &report.mismatches {
                println!("  ×  {m}");
            }
            if report.mismatches.is_empty() {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Err(why) => {
            eprintln!("{why}");
            ExitCode::from(2)
        }
    }
}
