// SPDX-License-Identifier: MIT
//! 組み立ての型 ── HTML の形は、コードではなく型が持つ。
//!
//! **形をコードの中の文字列に散らすと、枚ごとに違う形が出る。** ここが読むのは
//! `references/board.template.html` 1枚で、差し込む場所（`{{名前}}`）も、
//! 部品の名前も、そのファイルが決める。
//!
//! **差し込む場所の過不足を、その場で誤りにする** ── 埋め忘れも、余分な値も、
//! 出てから気づく形にしない。

use crate::data_access::files;

use std::collections::BTreeMap;
use std::path::Path;

const OPEN: &str = "<template data-part=\"";
const CLOSE: &str = "</template>";

/// 型が持つ部品。
#[derive(Debug, Clone, Default)]
pub struct Parts {
    parts: BTreeMap<String, String>,
}

/// 開きの印を1つ読み、部品の名前と、中身の始まる位置を返す。
fn open_at(body: &str, from: usize) -> Option<(usize, String, usize)> {
    let at = body[from..].find(OPEN).map(|x| from + x)?;
    let head = at + OPEN.len();
    let end = body[head..].find('"').map(|x| head + x)?;
    let name = &body[head..end];
    if !name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        || name.is_empty()
    {
        return None;
    }
    let shut = body[end..].find('>').map(|x| end + x)?;
    Some((at, name.to_owned(), shut + 1))
}

impl Parts {
    /// 型を読む。**1か所からしか読まない。**
    ///
    /// # Errors
    ///
    /// 読めないとき、部品が1つも無いとき、閉じていないとき、名前が重複するときに返す。
    pub fn load(path: &Path) -> Result<Self, String> {
        let body = files::read_to_string(path)
            .map_err(|e| format!("{}: 読めない ── {e}", path.display()))?;
        let mut found = Vec::new();
        let mut from = 0;
        while let Some((at, name, head)) = open_at(&body, from) {
            found.push((at, name, head));
            from = head;
        }
        if found.is_empty() {
            return Err(format!("{} に部品が1つも無い", path.display()));
        }
        let mut parts: BTreeMap<String, String> = BTreeMap::new();
        for (i, (_, name, head)) in found.iter().enumerate() {
            let stop = found.get(i + 1).map_or(body.len(), |(at, _, _)| *at);
            let chunk = &body[*head..stop];
            // **入れ子の <template> を、部品の切れ目と取り違えない** ──
            // 面の型は中に <template> を持つ。最後の閉じだけが切れ目である
            let Some(cut) = chunk.rfind(CLOSE) else {
                return Err(format!("部品 {name} が閉じていない"));
            };
            if parts.contains_key(name) {
                return Err(format!("部品の名前が重複している: {name}"));
            }
            parts.insert(name.clone(), chunk[..cut].to_owned());
        }
        Ok(Self { parts })
    }

    /// 型が持つ部品の名前。
    #[must_use]
    pub fn names(&self) -> Vec<&str> {
        self.parts.keys().map(String::as_str).collect()
    }

    /// 部品1つを組む。**差し込む場所と、渡した値が、過不足なく一致する。**
    ///
    /// # Errors
    ///
    /// 型に無い部品のとき、差し込む値が足りないとき、型に無い値を渡したときに返す。
    pub fn part(&self, name: &str, slots: &[(&str, String)]) -> Result<String, String> {
        let Some(frag) = self.parts.get(name) else {
            return Err(format!(
                "型に無い部品: {name}。使えるのは {:?} である",
                self.names()
            ));
        };
        let mut need = find_slots(frag);
        need.sort_unstable();
        need.dedup();
        let mut got: Vec<&str> = slots.iter().map(|(k, _)| *k).collect();
        got.sort_unstable();
        got.dedup();
        let short: Vec<&str> = need.iter().copied().filter(|k| !got.contains(k)).collect();
        if !short.is_empty() {
            return Err(format!("部品 {name}: 差し込む値が足りない {short:?}"));
        }
        let extra: Vec<&str> = got.iter().copied().filter(|k| !need.contains(k)).collect();
        if !extra.is_empty() {
            return Err(format!("部品 {name}: 型に無い値を渡した {extra:?}"));
        }
        Ok(fill(frag, slots))
    }
}

/// 差し込む場所の名前を集める。
fn find_slots(frag: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(at) = frag[from..].find("{{").map(|x| from + x) {
        let head = at + 2;
        let Some(end) = frag[head..].find("}}").map(|x| head + x) else {
            break;
        };
        let name = &frag[head..end];
        if !name.is_empty()
            && name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        {
            out.push(name);
        }
        from = end + 2;
    }
    out
}

/// 差し込む。**渡した値の中の `{{…}}` は、差し込む場所として読まない。**
fn fill(frag: &str, slots: &[(&str, String)]) -> String {
    let mut out = String::with_capacity(frag.len());
    let mut from = 0;
    while let Some(at) = frag[from..].find("{{").map(|x| from + x) {
        let head = at + 2;
        let Some(end) = frag[head..].find("}}").map(|x| head + x) else {
            break;
        };
        let name = &frag[head..end];
        match slots.iter().find(|(k, _)| *k == name) {
            Some((_, value)) => {
                out.push_str(&frag[from..at]);
                out.push_str(value);
            }
            None => out.push_str(&frag[from..end + 2]),
        }
        from = end + 2;
    }
    out.push_str(&frag[from..]);
    out
}
