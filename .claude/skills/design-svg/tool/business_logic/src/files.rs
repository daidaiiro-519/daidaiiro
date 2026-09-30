// SPDX-License-Identifier: MIT
//! サービス層が読み書きするファイル ── 宣言の読み込みと、生成物の書き出しである。
//!
//! **サービス層は入出力を直接扱わず、ここを通す**（ACDR 0058）。サービス層はデータアクセス層を
//! 参照できない ── 層を飛ばさないので、業務ロジック層がデータアクセス層の `files` へ渡す。
//! 誤りの文言は呼ぶ側が組む ── ここは `std::io::Error` をそのまま返す。

use std::io;
use std::path::Path;

use crate::data_access::files;

/// 文字列として読む。
///
/// # Errors
///
/// 読めないか、UTF-8 でないときに返す。
pub fn read_to_string(path: impl AsRef<Path>) -> io::Result<String> {
    files::read_to_string(path)
}

/// 書く。**在れば置き換える。**
///
/// # Errors
///
/// 書けないときに返す。
pub fn write(path: impl AsRef<Path>, body: impl AsRef<[u8]>) -> io::Result<()> {
    files::write(path, body)
}
