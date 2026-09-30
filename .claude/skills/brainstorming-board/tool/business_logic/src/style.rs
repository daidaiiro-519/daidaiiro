// SPDX-License-Identifier: MIT
//! 見た目と動きを、正本のファイルから読む。
//!
//! **見た目の文字列をここに保持しない** ── 正本は `references/board.css` である。この側が
//! 持つのはトークンの組み立てと、連結の順だけである。
//!
//! **動きも、正本のファイルから読む** ── `references/board.js` である。同じ理由で、
//! 150行の script をこちら側の文字列に持たない。

use std::path::Path;

/// 見た目と動き。
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Style {
    /// トークンと、見た目の正本を連結したもの。
    pub css: String,
    /// 動き。
    pub js: String,
}

impl Style {
    /// 正本を読む。**トークンの検査が通らなければ、組まない。**
    ///
    /// # Errors
    ///
    /// 読めないときと、トークンの検査が通らないときに返す。
    pub fn load(references: &Path) -> Result<Self, String> {
        let value = crate::tokens::load(&crate::tokens::path(references))?;
        let bad = crate::tokens::validate(references, &value);
        if !bad.is_empty() {
            return Err(format!(
                "トークンの検査が通っていない:\n  {}",
                bad.join("\n  ")
            ));
        }
        let read = |name: &str| -> Result<String, String> {
            let path = references.join(name);
            std::fs::read_to_string(&path)
                .map_err(|e| format!("{}: 読めない ── {e}", path.display()))
        };
        Ok(Self {
            css: crate::tokens::css(&value, false)? + &read("board.css")?,
            js: read("board.js")?,
        })
    }
}

/// 除外した案の記号を、通過した案の次から振る。
///
/// **記号は候補の識別子である** ── 除外した案も候補だったのに、道具の側は
/// （案, 何が壊れるか）の対しか保持しないので記号が無い。
#[must_use]
pub fn drop_numbering(origin: &[(usize, usize)]) -> String {
    let mut rows: Vec<(usize, usize)> = origin.to_vec();
    rows.sort_unstable();
    rows.iter()
        .map(|(no, at)| format!("#p{no} table:has(span.n.out){{counter-reset:drop {at}}}"))
        .collect()
}
