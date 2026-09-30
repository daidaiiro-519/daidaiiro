// SPDX-License-Identifier: MIT
//! サービス層の道具の一覧 ── **SKILL.md が案内する呼び方が、道具の一覧の上で成立すること。**

use fc_service::{tools, Given};

fn verify_args(pairs: &[(&str, &str)]) -> fc_service::Outcome {
    let all = tools();
    let tool = all
        .iter()
        .find(|t| t.name == "verify")
        .expect("verify が在る");
    // **入口と同じく、必須の引数が欠けていれば断る** ── 宣言が必須とする引数を、ここで確かめる
    let mut given = Given::default();
    for (k, v) in pairs {
        given.push(k, (*v).to_owned());
    }
    for a in &tool.args {
        assert!(
            !a.required || given.has(a.name),
            "{} を必須と宣言している",
            a.name
        );
    }
    (tool.run)(&given)
}

fn file(name: &str, body: &str) -> String {
    let dir = std::env::temp_dir().join(format!("fc_declare_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("作れる");
    let p = dir.join(name);
    std::fs::write(&p, body).expect("書ける");
    p.display().to_string()
}

#[test]
fn needles_can_come_from_a_list_alone() {
    // `verify <原文> --as quote --from quotes.txt` ── 照合するものを位置引数で渡さない呼び方
    let src = file("src.md", "these are the words here\n");
    let list = file("quotes.txt", "these are the words\n");
    let out = verify_args(&[("path", &src), ("as", "quote"), ("from", &list)]);
    assert!(out.ok && out.findings.is_empty(), "{:?}", out.findings);
}

#[test]
fn nothing_to_check_is_a_misuse() {
    let src = file("src2.md", "text\n");
    let out = verify_args(&[("path", &src), ("as", "quote")]);
    assert!(!out.ok);
}
