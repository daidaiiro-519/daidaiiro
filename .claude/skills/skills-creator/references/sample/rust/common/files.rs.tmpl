// SPDX-License-Identifier: MIT
//! ファイルの入出力。**どの Skill も同じファイルを複製して使う** ── `contract.rs` と同じ扱いである。
//!
//! データアクセス層が持つ。**業務ロジック層は `std::fs` を直接呼ばず、ここを通す**（ACDR 0058）
//! ── 入出力を1つの層に集めると、業務ロジック層は読んだ値を受けて判定するだけになる。
//! 関数は `std::fs` と同じ名前と引数にそろえる ── 移すときに呼び出しの形を変えずに済む。
//! **判定を置かない** ── 何を読むか ・ 読んだものをどう扱うかは、業務ロジック層が決める。

use std::io;
use std::path::{Path, PathBuf};

/// 文字列として読む。
///
/// # Errors
///
/// 読めないか、UTF-8 でないときに返す。
pub fn read_to_string(path: impl AsRef<Path>) -> io::Result<String> {
    std::fs::read_to_string(path)
}

/// バイト列として読む。
///
/// # Errors
///
/// 読めないときに返す。
pub fn read(path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
    std::fs::read(path)
}

/// 書く。**在れば置き換える。**
///
/// # Errors
///
/// 書けないときに返す。
pub fn write(path: impl AsRef<Path>, body: impl AsRef<[u8]>) -> io::Result<()> {
    std::fs::write(path, body)
}

/// フォルダを、途中の階層も含めて作る。
///
/// # Errors
///
/// 作れないときに返す。
pub fn create_dir_all(path: impl AsRef<Path>) -> io::Result<()> {
    std::fs::create_dir_all(path)
}

/// ファイルを消す。
///
/// # Errors
///
/// 消せないときに返す。
pub fn remove_file(path: impl AsRef<Path>) -> io::Result<()> {
    std::fs::remove_file(path)
}

/// フォルダの中身の経路を、**名前の順に並べて**返す ── 並びを OS に任せると、
/// 同じ入力から別の結果が出る。
///
/// # Errors
///
/// フォルダを読めないときに返す。
pub fn list(dir: impl AsRef<Path>) -> io::Result<Vec<PathBuf>> {
    let mut out: Vec<PathBuf> = std::fs::read_dir(dir)?
        .flatten()
        .map(|e| e.path())
        .collect();
    out.sort();
    Ok(out)
}

/// ファイルとして在るか。
#[must_use]
pub fn is_file(path: impl AsRef<Path>) -> bool {
    path.as_ref().is_file()
}

/// フォルダとして在るか。
#[must_use]
pub fn is_dir(path: impl AsRef<Path>) -> bool {
    path.as_ref().is_dir()
}

/// 何かが在るか。
#[must_use]
pub fn exists(path: impl AsRef<Path>) -> bool {
    path.as_ref().exists()
}

/// バイト数。
///
/// # Errors
///
/// 読めないときに返す。
pub fn size(path: impl AsRef<Path>) -> io::Result<u64> {
    std::fs::metadata(path).map(|m| m.len())
}

/// 絶対の経路へ解く。
///
/// # Errors
///
/// 経路が無いときに返す。
pub fn canonicalize(path: impl AsRef<Path>) -> io::Result<PathBuf> {
    std::fs::canonicalize(path)
}

/// 一時の置き場所。
#[must_use]
pub fn temp_dir() -> PathBuf {
    std::env::temp_dir()
}
