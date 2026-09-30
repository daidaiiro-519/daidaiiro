// SPDX-License-Identifier: MIT
//! 図の中の文字が、重なっていないか ・ 枠からはみ出していないかを検査する。
//!
//! **目で見つける前に、機械で検出する。** 実測 ── 矢印が箱を貫き、注記が2つ重なって別の文に
//! なっている図を出してしまった。
//!
//! 見るのは5つ ── 文字の重なり ・ 枠からのはみ出し ・ 線が箱を貫くこと ・ 空白で字下げ ・
//! 箱が文字を覆うこと。
//!
//! **絶対座標で描いた図だけが対象である** ── 群を平行移動して組む図は、この検査では全部の
//! 文字が重なって見える（実測で129件の偽陽性）。その図は**組ませた相手の検査**に掛ける。
//!
//! **受け取るのは SVG のファイルである** ── 以前はモジュールの名前を受け取って読み込んで
//! いたが、ブレストボードは SVG のファイルを持つので、読み込む相手がもう居ない。

use crate::data_access::files;

use std::path::Path;

/// 文字1つの置き場所。`(縦, 左, 右, 中身)`
type Item = (f64, f64, f64, String);

/// 印の中の値を1つ読む。
fn attr(tag: &str, name: &str) -> Option<String> {
    let key = format!("{name}=\"");
    let at = tag.find(&key)? + key.len();
    let end = tag[at..].find('"')? + at;
    Some(tag[at..end].to_owned())
}

fn number(tag: &str, name: &str) -> Option<f64> {
    attr(tag, name)?.parse().ok()
}

/// 印を落とす。
fn strip(body: &str) -> String {
    let mut out = String::new();
    let mut inside = false;
    for c in body.chars() {
        match c {
            '<' => inside = true,
            '>' => inside = false,
            _ if !inside => out.push(c),
            _ => {}
        }
    }
    out
}

/// 開きの印と、その中身を並べる。
fn elements(svg: &str, name: &str) -> Vec<(String, String)> {
    let head = format!("<{name}");
    let shut = format!("</{name}>");
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(at) = svg[from..].find(&head).map(|x| x + from) {
        let Some(open_end) = svg[at..].find('>').map(|x| at + x) else {
            break;
        };
        let tag = svg[at..=open_end].to_owned();
        let body = svg[open_end + 1..]
            .find(&shut)
            .map(|x| svg[open_end + 1..open_end + 1 + x].to_owned())
            .unwrap_or_default();
        out.push((tag, body));
        from = open_end + 1;
    }
    out
}

/// 文字の置き場所を集める。
///
/// 全角は1文字ぶん、英数字と記号は約0.55文字ぶんで数える。
fn items(svg: &str) -> Vec<Item> {
    let mut out = Vec::new();
    for (tag, body) in elements(svg, "text") {
        let (Some(x), Some(y)) = (number(&tag, "x"), number(&tag, "y")) else {
            continue;
        };
        // **行をまたぐ文字は測らない** ── 幅の見積もりは「1行ぶんの字数 × 字寸」なので、
        // 中に改行を持つものへ当てると、実際の3倍の幅を主張して重なりを捏造する
        if body.contains('\n') {
            continue;
        }
        let text = strip(&body);
        let size = number(&tag, "font-size").unwrap_or(10.0);
        let width: f64 = size
            * text
                .chars()
                .map(|c| if c as u32 > 0x2E80 { 1.0 } else { 0.55 })
                .sum::<f64>();
        let left = if tag.contains("middle") {
            x - width / 2.0
        } else if tag.contains("text-anchor=\"end\"") {
            x - width
        } else {
            x
        };
        out.push((y, left, left + width, text));
    }
    out.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    out
}

/// 箱の置き場所。`(左, 上, 幅, 高さ)`
fn rects(svg: &str) -> Vec<(f64, f64, f64, f64)> {
    elements(svg, "rect")
        .iter()
        .filter_map(|(tag, _)| {
            Some((
                number(tag, "x")?,
                number(tag, "y")?,
                number(tag, "width")?,
                number(tag, "height")?,
            ))
        })
        .collect()
}

/// 画布の大きさ。
fn viewbox(svg: &str) -> Option<(f64, f64)> {
    let key = "viewBox=\"0 0 ";
    let at = svg.find(key)? + key.len();
    let end = svg[at..].find('"')? + at;
    let mut parts = svg[at..end].split_whitespace();
    Some((parts.next()?.parse().ok()?, parts.next()?.parse().ok()?))
}

/// 図1枚の検査の結果。
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct Found {
    /// 図の名前。
    pub name: String,
    /// 文字の数。
    pub items: usize,
    /// 見つけたこと。
    pub lines: Vec<String>,
    /// 種類ごとの数。`(重なり, はみ出し, 貫通, 空白の字下げ, 覆い)`
    pub counts: (usize, usize, usize, usize, usize),
}

impl Found {
    /// 直すところの数。
    #[must_use]
    pub const fn total(&self) -> usize {
        let (a, b, c, d, e) = self.counts;
        a + b + c + d + e
    }

    /// 人が読む1行。
    #[must_use]
    pub fn headline(&self) -> String {
        let (a, b, c, d, e) = self.counts;
        format!(
            "{}: 文字 {} 件 ／ 重なり {a} 件 ／ はみ出し {b} 件 ／ 貫通 {c} 件 ／ 空白の字下げ {d} 件 ／ 覆い {e} 件",
            self.name, self.items
        )
    }
}

/// 行頭が空白で字下げされているか。**SVG は行頭の空白を除去するので、階層が潰れる。**
fn indented(text: &str) -> bool {
    let plain = strip(text);
    let blanks = plain
        .chars()
        .take_while(|c| c.is_whitespace() || *c == '\u{3000}' || *c == '\u{a0}')
        .count();
    blanks >= 2
        && plain
            .chars()
            .nth(blanks)
            .is_some_and(|c| !c.is_whitespace())
}

/// 図1枚を検査する。
#[must_use]
pub fn check(name: &str, svg: &str) -> Found {
    let items = items(svg);
    let mut found = Found {
        name: name.to_owned(),
        items: items.len(),
        ..Found::default()
    };
    // 重なり
    let mut overlap = Vec::new();
    for (i, (y, a, b, t)) in items.iter().enumerate() {
        for (y2, a2, b2, t2) in items.iter().skip(i + 1) {
            if (y - y2).abs() < 10.0 && a < &(b2 - 2.0) && a2 < &(b - 2.0) {
                overlap.push(format!(
                    "重なり y={y}: 「{}」 × 「{}」",
                    t.chars().take(26).collect::<String>(),
                    t2.chars().take(26).collect::<String>()
                ));
            }
        }
    }
    // はみ出し
    let mut over = Vec::new();
    if let Some((w, h)) = viewbox(svg) {
        for (y, _, b, t) in &items {
            if *b > w + 2.0 || *y > h {
                over.push(format!(
                    "はみ出し: 「{}」",
                    t.chars().take(40).collect::<String>()
                ));
            }
        }
    }
    // 線が箱を貫いていないか（縦横の線だけを対象にする）
    let boxes = rects(svg);
    let mut pierced = Vec::new();
    for (tag, _) in elements(svg, "line") {
        let (Some(x1), Some(y1), Some(x2), Some(y2)) = (
            number(&tag, "x1"),
            number(&tag, "y1"),
            number(&tag, "x2"),
            number(&tag, "y2"),
        ) else {
            continue;
        };
        for (rx, ry, rw, rh) in &boxes {
            if (x1 - x2).abs() < f64::EPSILON
                && *rx < x1
                && x1 < rx + rw
                && y1.min(y2) < *ry
                && ry + rh < y1.max(y2)
            {
                pierced.push(format!(
                    "縦線 x={x1} が箱（y {ry}〜{}）を貫いている",
                    ry + rh
                ));
            }
            if (y1 - y2).abs() < f64::EPSILON
                && *ry < y1
                && y1 < ry + rh
                && x1.min(x2) < *rx
                && rx + rw < x1.max(x2)
            {
                pierced.push(format!(
                    "横線 y={y1} が箱（x {rx}〜{}）を貫いている",
                    rx + rw
                ));
            }
        }
    }
    // 空白で字下げを作っていないか
    let blanks: Vec<String> = elements(svg, "text")
        .iter()
        .filter(|(_, body)| !body.contains('\n') && indented(body))
        .map(|(_, body)| {
            format!(
                "空白で字下げしている: 「{}」 ── x の値で表す",
                strip(body).chars().take(30).collect::<String>()
            )
        })
        .collect();
    // 箱の枠線が、文字を横切っていないか
    let mut covered = Vec::new();
    for (rx, ry, rw, rh) in &boxes {
        for ex in [*rx, rx + rw] {
            for (ty, a0, b0, t) in &items {
                if a0 + 2.0 < ex && ex < b0 - 2.0 && *ry < ty - 3.0 && ty + 3.0 < ry + rh {
                    covered.push(format!(
                        "箱の縦の枠（x={ex}）が「{}」を横切っている",
                        t.chars().take(22).collect::<String>()
                    ));
                }
            }
        }
        for ey in [*ry, ry + rh] {
            for (ty, a0, b0, t) in &items {
                if *rx < *a0 && *b0 < rx + rw && ty - 8.0 < ey && ey < ty + 3.0 {
                    covered.push(format!(
                        "箱の横の枠（y={ey}）が「{}」を横切っている",
                        t.chars().take(22).collect::<String>()
                    ));
                }
            }
        }
    }
    found.counts = (
        overlap.len(),
        over.len(),
        pierced.len(),
        blanks.len(),
        covered.len(),
    );
    found.lines.extend(covered.into_iter().take(6));
    found.lines.extend(blanks.into_iter().take(6));
    found.lines.extend(pierced);
    found.lines.extend(overlap);
    found.lines.extend(over);
    found
}

/// 渡された図を1枚ずつ検査する。
///
/// # Errors
///
/// 読めないときに返す。
pub fn count(paths: &[String]) -> Result<Vec<Found>, String> {
    let mut out = Vec::new();
    for path in paths {
        let at = Path::new(path);
        let svg = files::read_to_string(at).map_err(|e| format!("{path}: 読めない ── {e}"))?;
        let name = at
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        out.push(check(&name, &svg));
    }
    Ok(out)
}
