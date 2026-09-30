// SPDX-License-Identifier: MIT
//! `reply` ・ `review` が読む入力。**読むのはデータアクセス層であり、ここは何を読むかだけを決める。**
//!
//! サービス層は入出力を持たないので、フックの入力 ・ 判定基準 ・ 事例の読み込みを
//! ここに置く（ACDR 0058）。**文言は呼ぶ側が誤用として出す** ── 読めない理由をそのまま返す。

use std::path::Path;

use crate::data_access::{files, stdin};

/// フックの入力の本文を読む。`-` なら標準入力、それ以外はファイルの経路である。
///
/// # Errors
///
/// 読めないときに、誤用として出す文言を返す。
pub fn hook_body(from: &str) -> Result<String, String> {
    if from == "-" {
        return stdin::read_to_string().map_err(|e| format!("標準入力を読めない ── {e}"));
    }
    files::read_to_string(from).map_err(|e| format!("読めない ── {from} ── {e}"))
}

/// この Skill が持つ正本（判定基準）を読む。審査の手順は references の実装（`refs::get`）が読む。
///
/// # Errors
///
/// 読めないときに、誤用として出す文言を返す。
pub fn read_text(path: &Path) -> Result<String, String> {
    files::read_to_string(path).map_err(|e| format!("読めない ── {} ── {e}", path.display()))
}

/// 事例はプロジェクトが持つ ── 廃語の一覧と同じく、上へたどって探す。
///
/// **見つからないことも、読めないことも、失敗にしない** ── 無ければ空の文字列を返し、事例無しで組む。
#[must_use]
pub fn find_examples(start: &Path) -> String {
    let here = files::canonicalize(start).unwrap_or_else(|_| start.to_path_buf());
    std::iter::successors(Some(here.as_path()), |p| p.parent())
        .map(|d| d.join(".doc-writing").join("review-examples.json"))
        .find(|c| files::exists(c))
        .and_then(|p| files::read_to_string(p).ok())
        .unwrap_or_default()
}
