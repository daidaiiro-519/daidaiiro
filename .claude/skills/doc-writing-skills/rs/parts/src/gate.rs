// SPDX-License-Identifier: MIT
//! 判定を当てる機構と、語の一覧の読み込み。
//!
//! **判定の一覧が正本である。** どの判定を、どの単位へ当てるかはここが持つ ──
//! 呼ぶ側が並べ直すと、当てる範囲が呼び方ごとに変わる。

use std::io;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::checks::{self, Pair, Words};
use crate::finding::Finding;
use crate::unit::{self, Kind, Unit};

/// 検査を外す印。**文書の頭に在るときだけ効く。**
pub const EXEMPT_MARK: &str = "<!-- doc-writing-skills: exempt -->";

/// 当てる単位の範囲。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Scope {
    /// 文書の全体へ1回当てる。
    Whole,
    /// 描画される単位へ1つずつ当てる。
    Rendered,
    /// すべての単位へ1つずつ当てる。
    Every,
    /// 散文と見出しと表の欄へ当てる ── **引用は外す。**
    Written,
}

impl Scope {
    /// 画面へ出す語。
    #[must_use]
    pub fn label(self) -> String {
        match self {
            Self::Whole => "文書全体".to_owned(),
            Self::Rendered => "引用 ・ 本文 ・ 箇条書き ・ 表のセル ・ 見出し".to_owned(),
            Self::Every => "コード ・ 引用 ・ 本文 ・ 箇条書き ・ 表のセル ・ 見出し".to_owned(),
            Self::Written => "本文 ・ 箇条書き ・ 表のセル ・ 見出し".to_owned(),
        }
    }

    fn takes(self, kind: Kind) -> bool {
        match self {
            Self::Whole => false,
            Self::Rendered => kind.rendered(),
            Self::Every => true,
            Self::Written => matches!(kind, Kind::Body | Kind::Heading | Kind::Item | Kind::Cell),
        }
    }
}

/// 判定1つの宣言。
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Check {
    /// 判定の名前。
    pub name: &'static str,
    /// 拠って立つもの。
    pub basis: &'static str,
    /// 当てる範囲。
    pub scope: Scope,
    /// 何を見るか。
    pub note: &'static str,
}

/// 当てている判定の一覧。**ここが正本である。**
#[must_use]
pub fn all() -> Vec<Check> {
    vec![
        Check {
            name: "見出しの階層が飛んでいる",
            basis: "概念4",
            scope: Scope::Whole,
            note: "見出し2の次に見出し4が来ると、間に何が在るはずだったかが分からない",
        },
        Check {
            name: "文体が混ざっている",
            basis: "概念6",
            scope: Scope::Whole,
            note: "敬体と常体を、同じ文書の中で混ぜない。体言止めは数えない",
        },
        Check {
            name: "並んだ項目の語尾が統一されていない",
            basis: "概念6",
            scope: Scope::Whole,
            note: "同じ立場のものは、同じ形で書く。敬体かどうかだけを判定する",
        },
        Check {
            name: "文字で図や表を描いている",
            basis: "概念2",
            scope: Scope::Every,
            note: "箱の角と、横罫の連なりを検出する。二倍ダッシュと置き場所の一覧は図ではない",
        },
        Check {
            name: "同じ意味の語が2つある",
            basis: "概念7",
            scope: Scope::Whole,
            note: "対を渡さなければ何も出ない",
        },
        Check {
            name: "述部が和語である",
            basis: "概念8",
            scope: Scope::Written,
            note: "公用文 Ⅲ－４ ウ を必須へ上げたリポジトリの決定（.acdr/0003）。\
                   引用は検査しない ── 原文を書き換えてはならない",
        },
        Check {
            name: "廃語を使用している",
            basis: "概念7",
            scope: Scope::Written,
            note: "一覧は .doc-writing/retired-words.json が持つ。渡されなければ何も出ない",
        },
        Check {
            name: "強調が描画されない",
            basis: "媒体の決め",
            scope: Scope::Rendered,
            note: "概念からは導けない。CommonMark の記法に拠る",
        },
    ]
}

/// JSON は、値の文字列だけを見る。
///
/// **構文を本文として読まない** ── 名前も括弧も書き手の文ではない。生のまま検査すると、
/// 引用が値の中に入って引用と判定されず、文体の判定が誤って検出する（実測）。
fn prose_of_json(raw: &str) -> String {
    fn walk(v: &Value, out: &mut Vec<String>) {
        match v {
            Value::String(s) => out.push(s.clone()),
            Value::Array(items) => items.iter().for_each(|x| walk(x, out)),
            Value::Object(map) => {
                for (k, v) in map {
                    if !k.starts_with('$') {
                        walk(v, out);
                    }
                }
            }
            _ => {}
        }
    }
    let Ok(parsed) = serde_json::from_str::<Value>(raw) else {
        return raw.to_owned();
    };
    let mut out = Vec::new();
    walk(&parsed, &mut out);
    out.join("\n\n")
}

fn per_unit(check: &Check, u: &Unit, words: &Words) -> Vec<String> {
    match check.name {
        "文字で図や表を描いている" => checks::drawn_figure(u),
        "述部が和語である" => checks::wago_predicate(u, words),
        "廃語を使用している" => checks::retired_word(u, words),
        "強調が描画されない" => checks::broken_emphasis(u),
        _ => Vec::new(),
    }
}

fn whole(check: &Check, units: &[Unit], words: &Words) -> Vec<Finding> {
    match check.name {
        "見出しの階層が飛んでいる" => checks::heading_skip(units),
        "文体が混ざっている" => checks::mixed_style(units),
        "並んだ項目の語尾が統一されていない" => checks::unparallel_items(units),
        "同じ意味の語が2つある" => checks::synonym(units, words),
        _ => Vec::new(),
    }
}

/// 1つの文書へ全部の判定を当てる。
///
/// # Errors
///
/// 読めないときに返す。
pub fn inspect(path: &Path, words: &Words) -> io::Result<Vec<Finding>> {
    let mut raw = std::fs::read_to_string(path)?;
    if path.extension().is_some_and(|x| x == "json") {
        raw = prose_of_json(&raw);
    }
    // **外す印は、文書の頭に在るときだけ効く** ── 途中に書いて全体を外せないようにする
    if raw
        .chars()
        .take(400)
        .collect::<String>()
        .contains(EXEMPT_MARK)
    {
        return Ok(Vec::new());
    }
    let units = unit::split(&raw);
    let mut out = Vec::new();
    for check in all() {
        if check.scope == Scope::Whole {
            out.extend(whole(&check, &units, words));
            continue;
        }
        for u in units.iter().filter(|u| check.scope.takes(u.kind)) {
            out.extend(
                per_unit(&check, u, words)
                    .into_iter()
                    .map(|ex| Finding::new(check.name, check.basis, u.line, ex)),
            );
        }
    }
    // **同じ検出を2回出さない** ── 読み手が同じ場所を2回開くことになる
    let mut seen = std::collections::BTreeSet::new();
    out.retain(|f| seen.insert((f.check.clone(), f.line, f.excerpt.clone())));
    Ok(out)
}

/// 和語の一覧を、契約から読む。**この側に語を書かない。**
///
/// # Errors
///
/// 読めないとき、または形が違うときに返す。
pub fn load_predicates(path: &Path) -> io::Result<Vec<(String, String)>> {
    let body = std::fs::read_to_string(path)?;
    let parsed: Value = serde_json::from_str(&body)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    let items = parsed
        .get("predicates")
        .and_then(Value::as_array)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "predicates が無い"))?;
    Ok(items
        .iter()
        .filter_map(|x| {
            Some((
                x.get("pattern")?.as_str()?.to_owned(),
                x.get("use_instead")?.as_str()?.to_owned(),
            ))
        })
        .collect())
}

/// 対象のファイルから上へたどり、廃語の一覧を探す。
///
/// **見つからないことを、失敗として扱わない。** 一覧を持たないプロジェクトでも、この
/// 道具はそのまま動く ── 持ち出した先で必ず止まる作りにしない。
#[must_use]
pub fn find_retired(start: &Path) -> Option<PathBuf> {
    let here = start.canonicalize().unwrap_or_else(|_| start.to_path_buf());
    std::iter::successors(Some(here.as_path()), |p| p.parent())
        .map(|d| d.join(".doc-writing").join("retired-words.json"))
        .find(|c| c.exists())
}

/// 廃語の一覧を読む。**形が違えば返す** ── 黙って空にしない。
///
/// # Errors
///
/// 読めないとき、または形が違うときに返す。
pub fn load_retired(path: &Path) -> io::Result<Vec<Pair>> {
    let body = std::fs::read_to_string(path)?;
    let parsed: Value = serde_json::from_str(&body)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    let items = parsed
        .get("retired")
        .and_then(Value::as_array)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "retired が無い"))?;
    Ok(items
        .iter()
        .filter_map(|x| {
            Some(Pair::new(
                x.get("word")?.as_str()?.to_owned(),
                x.get("use_instead")?.as_str()?.to_owned(),
                x.get("source")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned(),
            ))
        })
        .collect())
}
