// SPDX-License-Identifier: MIT
//! 見た目を、正本のファイルから読む。
//!
//! **見た目の文字列をここに保持しない** ── 正本は `references/acdr.css` である。
//! この側が持つのはトークンの組み立てと、塊への分割だけである。
//!
//! | 名前 | 何を着せるか |
//! |---|---|
//! | `tokens` | トークンの表。本体の先頭に置く |
//! | `tokens_embed` | 同じ表を、`:host` でも解決する形で。iframe と Shadow の中へ流し込む |
//! | `base` | 頁 ・ タブ ・ 面 |
//! | `code` | コードと差分 |
//! | `mark` | 印と、押すと開くラベル。**中へも流し込む** |
//! | `section` | 節（決定 ・ なぜ ・ 形の変化 …） |
//!
//! 塊の境目は、正本の中の `/* == 名前 == */` が示す ── **この側が境目を決めない**。
//! 決めると、正本を割り直したときに気づけない。
//!
//! **動きも、正本のファイルから読む** ── `references/acdr.js` である。同じ理由で、
//! 200行の script をこちら側の文字列に持たない。

use std::path::Path;

use crate::data_access;

/// 正本が持つ塊。**4つでなければ止まる。**
const BLOCKS: [&str; 4] = ["BASE", "CODE", "MARK", "SECTION"];

/// 見た目と動き。
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Style {
    /// トークンの表。
    pub tokens: String,
    /// 同じ表を、`:host` でも解決する形で。
    pub tokens_embed: String,
    /// 頁 ・ タブ ・ 面。
    pub base: String,
    /// コードと差分。
    pub code: String,
    /// 印と、押すと開くラベル。
    pub mark: String,
    /// 節。
    pub section: String,
    /// 動き。
    pub js: String,
}

/// 正本の中の塊を切り出す。
fn split(body: &str) -> Vec<(String, String)> {
    const HEAD: &str = "/* == ";
    const TAIL: &str = " == */";
    let mut at = Vec::new();
    let mut from = 0;
    while let Some(start) = body[from..].find(HEAD).map(|x| from + x) {
        let head = start + HEAD.len();
        let Some(end) = body[head..].find(TAIL).map(|x| head + x) else {
            break;
        };
        let name = &body[head..end];
        if name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') && !name.is_empty() {
            at.push((start, name.to_owned(), end + TAIL.len()));
        }
        from = end + TAIL.len();
    }
    let mut out = Vec::new();
    for (i, (_, name, head)) in at.iter().enumerate() {
        let stop = at.get(i + 1).map_or(body.len(), |(s, _, _)| *s);
        out.push((name.clone(), body[*head..stop].to_owned()));
    }
    out
}

impl Style {
    /// 正本を読む。
    ///
    /// # Errors
    ///
    /// 読めないときと、塊が宣言と一致しないときに返す。
    pub fn load(references: &Path) -> Result<Self, String> {
        let tokens = crate::tokens::load(&crate::tokens::path(references))?;
        let read = |name: &str| -> Result<String, String> {
            let path = references.join(name);
            data_access::files::read_to_string(&path)
                .map_err(|e| format!("{}: 読めない ── {e}", path.display()))
        };
        let css = read("acdr.css")?;
        let blocks = split(&css);
        let mut names: Vec<&str> = blocks.iter().map(|(k, _)| k.as_str()).collect();
        names.sort_unstable();
        let mut want = BLOCKS;
        want.sort_unstable();
        if names != want {
            return Err(format!(
                "acdr.css の塊が宣言と一致しない: {names:?} ── 要るのは {want:?} である"
            ));
        }
        let pick = |name: &str| -> String {
            blocks
                .iter()
                .find(|(k, _)| k == name)
                .map(|(_, v)| v.clone())
                .unwrap_or_default()
        };
        Ok(Self {
            tokens: crate::tokens::css(&tokens, false)?,
            tokens_embed: crate::tokens::css(&tokens, true)?,
            base: pick("BASE"),
            code: pick("CODE"),
            mark: pick("MARK"),
            section: pick("SECTION"),
            js: read("acdr.js")?,
        })
    }
}
