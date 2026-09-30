// SPDX-License-Identifier: MIT
//! ブレストボードの置き場所を作る。
//!
//! 作るのは器だけである。**論点は書かない** ── 書くのは対話の側で、この道具は置き場所と、
//! 入力の雛形と、索引の行だけを用意する。
//!
//! **ブレストボードの下に実装を置かない。** 正本は JSON であり、組み立てるのは Skill 側である。
//!
//! **既に在る名前は作り直さない。** 上書きすると、書いた論点が消える。
//!
//! **雛形の本文は `references/` が持つ** ── この側に置くと、雛形を直す作業が実装を直す
//! 作業になる。

use crate::data_access::files;

use std::path::{Path, PathBuf};

/// 索引の名前。
const INDEX: &str = "README.md";

fn read(references: &Path, name: &str) -> Result<String, String> {
    let path = references.join(name);
    files::read_to_string(&path).map_err(|e| format!("{}: 読めない ── {e}", path.display()))
}

/// 置き場所 ・ 雛形 ・ 索引の行を作り、報告の行を返す。**印字はしない。**
///
/// # Errors
///
/// 既に在るときと、雛形を読めないときと、書けないときに返す。
pub fn create(
    references: &Path,
    name: &str,
    dir: &str,
    title: &str,
) -> Result<Vec<String>, String> {
    let root = PathBuf::from(dir);
    let board = root.join(name);
    if files::exists(&board) {
        return Err(format!(
            "既に在る: {} ── 作り直さない。上書きすると、書いた論点が消える",
            board.display()
        ));
    }
    let title = if title.is_empty() { name } else { title };
    let make = |at: &Path| -> Result<(), String> {
        files::create_dir_all(at).map_err(|e| format!("{}: 作れない ── {e}", at.display()))
    };
    let put = |at: PathBuf, body: &str| -> Result<(), String> {
        files::write(&at, body).map_err(|e| format!("{}: 書けない ── {e}", at.display()))
    };
    make(&board.join("figures"))?;
    // 完成イメージの置き場所を、見本ごと作る ── 図は design-svg に組ませ、ここへ置く
    put(
        board.join("figures").join("example.svg"),
        &read(references, "figure.example.svg")?,
    )?;
    for sub in ["rounds", "sources", "answers"] {
        make(&board.join(sub))?;
    }
    put(board.join("answers").join(".read"), "")?;
    put(
        board.join("sources").join(INDEX),
        &read(references, "sources.example.md")?,
    )?;
    put(
        board.join("board.json"),
        &read(references, "board.example.json")?
            .replace("{title}", title)
            .replace("{name}", name),
    )?;

    let index = root.join(INDEX);
    if !files::exists(&index) {
        put(index.clone(), &read(references, "index.example.md")?)?;
    }
    let line = format!("| `{name}/` | {title} | （未発行） | （未複製） |\n");
    let now = files::read_to_string(&index)
        .map_err(|e| format!("{}: 読めない ── {e}", index.display()))?;
    if !now.contains(&line) {
        put(index.clone(), &(now + &line))?;
    }

    let mut lines = vec![format!("作った: {}", board.display())];
    let mut placed = Vec::new();
    walk(&board, &mut placed);
    placed.sort();
    for p in placed {
        lines.push(format!(
            "  {}",
            p.strip_prefix(&root).unwrap_or(&p).display()
        ));
    }
    lines.push(format!("索引へ1行足した: {}", index.display()));
    lines.push(String::new());
    lines.push(
        "次にすること ── board.json へ論点を書き、brainstorming-board render <この置き場所> で組む。"
            .to_owned(),
    );
    lines.push("図は design-svg に組ませ、返った SVG を figures/ へ置く。".to_owned());
    Ok(lines)
}

/// 置いたものを並べる。
fn walk(at: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = files::list(at) else {
        return;
    };
    for p in entries {
        out.push(p.clone());
        if files::is_dir(&p) {
            walk(&p, out);
        }
    }
}
