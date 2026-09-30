// SPDX-License-Identifier: MIT
//! フォルダの木ごとの入出力。**この Skill に固有の入出力である。**
//!
//! `files` は雛形の複製なので、木ごとの操作はここに置く。
//! **判定を置かない** ── どの木を写し、どの木を消すかは、業務ロジック層が決める。

use std::io;
use std::path::Path;

/// フォルダを、中身ごと消す。
///
/// # Errors
///
/// 消せないときに返す。
pub fn remove_dir_all(path: impl AsRef<Path>) -> io::Result<()> {
    std::fs::remove_dir_all(path)
}

/// 木ごと写す。
///
/// # Errors
///
/// 読めないときと、書けないときに返す。
pub fn copy_tree(from: &Path, to: &Path) -> io::Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let at = entry.path();
        let into = to.join(entry.file_name());
        if at.is_dir() {
            copy_tree(&at, &into)?;
        } else {
            std::fs::copy(&at, &into)?;
        }
    }
    Ok(())
}

/// 木の中のファイルを読み取り専用にする（戻すこともできる）。
///
/// # Errors
///
/// 読めないときと、属性を変えられないときに返す。
pub fn set_read_only(at: &Path, on: bool) -> io::Result<()> {
    for entry in std::fs::read_dir(at)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            set_read_only(&path, on)?;
            continue;
        }
        let mut perm = std::fs::metadata(&path)?.permissions();
        perm.set_readonly(on);
        std::fs::set_permissions(&path, perm)?;
    }
    Ok(())
}
