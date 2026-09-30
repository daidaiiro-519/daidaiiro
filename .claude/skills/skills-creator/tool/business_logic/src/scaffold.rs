// SPDX-License-Identifier: MIT
//! 道具の契約一式を、Skill のフォルダへ置く。**既に在るものは上書きしない。**

use std::io;
use std::path::{Path, PathBuf};

use crate::data_access::files;

/// 置いたものと、残したもの。
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct Placed {
    /// 置いたファイル。
    pub written: Vec<String>,
    /// 既に在ったので残したファイル。
    pub kept: Vec<String>,
}

/// 置く1件 ── 置く先（Skill のフォルダからの相対）と、中身。
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Item {
    /// 置く先。
    pub to: PathBuf,
    /// 中身。
    pub body: String,
    /// 既に在っても上書きするか。
    pub overwrite: bool,
}

impl Item {
    /// 置く1件を組む。**既に在れば残す。**
    #[must_use]
    pub const fn keep(to: PathBuf, body: String) -> Self {
        Self {
            to,
            body,
            overwrite: false,
        }
    }
}

fn write(item: &Item, root: &Path, placed: &mut Placed) -> io::Result<()> {
    let dst = root.join(&item.to);
    if files::exists(&dst) && !item.overwrite {
        placed.kept.push(dst.display().to_string());
        return Ok(());
    }
    if let Some(parent) = dst.parent() {
        files::create_dir_all(parent)?;
    }
    files::write(&dst, &item.body)?;
    placed.written.push(dst.display().to_string());
    Ok(())
}

/// 受け取った一式を置く。**何を置くかは呼ぶ側が決める** ── この crate は
/// 雛形の並びを認知しない。
///
/// # Errors
///
/// フォルダを作れないとき、またはファイルを書けないときに返す。
pub fn place(root: &Path, items: &[Item]) -> io::Result<Placed> {
    let mut placed = Placed::default();
    for item in items {
        write(item, root, &mut placed)?;
    }
    Ok(placed)
}

/// 雛形を1つ読む。**サービス層は入出力を持たない**ので、ここを通す。
///
/// # Errors
///
/// 読めないときに返す。
pub fn read_template(dir: &Path, name: &str) -> io::Result<String> {
    files::read_to_string(dir.join(name))
}

/// 同じ置き場所に在る Skill の名前を返す。**SKILL.md を持つフォルダだけである。**
#[must_use]
pub fn siblings(root: &Path) -> Vec<String> {
    let Some(parent) = root.parent() else {
        return Vec::new();
    };
    files::list(parent)
        .unwrap_or_default()
        .into_iter()
        .filter(|p| files::is_file(p.join("SKILL.md")))
        .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .collect()
}
