// SPDX-License-Identifier: MIT
//! SVG の中で使う識別子 ── **中身から決める。**
//!
//! 識別子は文書の中で衝突しなければよく、順番である必要はない。通し番号にすると同じ入力から
//! 違う出力が出て、部品が決定的であるという契約を破る（実測 ── 同じ値で2回描くと違う識別子が
//! 出た）。

/// 識別子に使う要約の桁数。
const DIGITS: usize = 8;

/// 中身から決まる識別子を返す。`parts` は材料を書いたもの（[`crate::py::repr`] の形）。
#[must_use]
pub fn stable(prefix: &str, parts: &[String]) -> String {
    use sha1::{Digest as _, Sha1};
    let body = crate::py::tuple(parts);
    let digest: String = Sha1::digest(body.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    format!("{prefix}{}", &digest[..DIGITS])
}
