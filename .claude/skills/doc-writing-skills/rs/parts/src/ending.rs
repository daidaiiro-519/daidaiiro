// SPDX-License-Identifier: MIT
//! 文末を型へ分ける。
//!
//! **体言止めと常体は、品詞を判定しないと分けられない。** 「扱い」は体言で
//! 「短い」は常体だが、字面は同じ形をしている。**分けられないものを分けたことに
//! せず**、敬体かどうかだけを判定する。

use fancy_regex::Regex;
use std::sync::OnceLock;

/// 文末の型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum Ending {
    /// 敬体。
    Polite,
    /// 敬体ではない。**常体と体言止めを分けない。**
    Plain,
}

impl Ending {
    /// 画面へ出す語。
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Polite => "敬体",
            Self::Plain => "非敬体",
        }
    }
}

fn polite() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"(です|ます|ません|でした|ましょう|でしょう|ください)。?$").expect("組める")
    })
}

/// 常体だと確定できる形。**体言止めと分けるために使う。**
pub fn plain_form() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"(である|だ|た|ない|る|い|う|く|す|つ|ぬ|ぶ|む)。$").expect("組める")
    })
}

/// 文末の型を返す。
#[must_use]
pub fn of(text: &str) -> Ending {
    let t = text.trim().trim_end_matches('。');
    let with = format!("{t}。");
    if polite().is_match(&with).unwrap_or(false) || polite().is_match(t).unwrap_or(false) {
        Ending::Polite
    } else {
        Ending::Plain
    }
}
