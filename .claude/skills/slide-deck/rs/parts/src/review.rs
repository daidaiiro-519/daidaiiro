// SPDX-License-Identifier: MIT
//! 原稿を持つデッキの照合の出力を組む ── 比較ページ ・ 確認記録 ・ 台本 ・ 観点ごとの結果。
//!
//! **4つとも、同じ1つの入力（`references/review.schema.json`）から組む。** 入力を
//! 出力ごとに分けると、比較ページと記録の中身が食い違う。
//!
//! **HTML の形を、ここへ書かない。** 形は `references/review.template.html` が持つ。
//! ここが持つのは、どの値をどの部品へ差し込むかと、原稿の差分の計算だけである。
//!
//! **同じ入力からは、同じ出力が出る。** 日付も乱数も読まない。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::deck::esc;
use crate::template::Parts;
use crate::validate;

/// 型の置き場所。
#[must_use]
pub fn template_path(references: &Path) -> PathBuf {
    references.join("review.template.html")
}

/// 入力の契約の置き場所。
#[must_use]
pub fn schema_path(references: &Path) -> PathBuf {
    references.join("review.schema.json")
}

/// 観点の群。**並び順は `review.md` §2 と一致させる。**
pub const GROUPS: [(&str, &str); 7] = [
    ("consistency", "スライドと原稿の整合"),
    ("story", "論とストーリー"),
    ("japanese", "日本語"),
    ("heading", "見出しと導入文"),
    ("figure", "図の表現"),
    ("layout", "表示の不良と読みやすさ"),
    ("video", "動画の文体と構成"),
];

fn group_name(key: &str) -> &str {
    GROUPS
        .iter()
        .find(|(k, _)| *k == key)
        .map_or(key, |(_, n)| n)
}

fn status_name(key: &str) -> &'static str {
    match key {
        "fixed" => "修正",
        "ok" => "問題なし",
        "pending" => "保留",
        _ => "未確認",
    }
}

fn text_of(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

fn array_of<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value
        .get(key)
        .and_then(Value::as_array)
        .map_or(&[][..], Vec::as_slice)
}

/// 原稿を文へ割る。**句点を文の側に残す** ── 残さないと、差分の表示で句点が消える。
#[must_use]
pub fn sentences(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for c in text.chars() {
        cur.push(c);
        if matches!(c, '。' | '？' | '！') {
            if !cur.trim().is_empty() {
                out.push(cur.trim().to_owned());
            }
            cur.clear();
        }
    }
    if !cur.trim().is_empty() {
        out.push(cur.trim().to_owned());
    }
    out
}

/// 文の差分の1行。
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Line {
    /// 両方に在る文。
    Same(String),
    /// 変更前にだけ在る文。
    Del(String),
    /// 変更後にだけ在る文。
    Ins(String),
}

/// 文の単位で差分を取る。最長共通部分列で、共通の文を残す。
#[must_use]
pub fn diff(before: &[String], after: &[String]) -> Vec<Line> {
    let (n, m) = (before.len(), after.len());
    let mut len = vec![vec![0usize; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            len[i][j] = if before[i] == after[j] {
                len[i + 1][j + 1] + 1
            } else {
                len[i + 1][j].max(len[i][j + 1])
            };
        }
    }
    let (mut i, mut j) = (0, 0);
    let mut out = Vec::new();
    while i < n && j < m {
        if before[i] == after[j] {
            out.push(Line::Same(before[i].clone()));
            i += 1;
            j += 1;
        } else if len[i + 1][j] >= len[i][j + 1] {
            out.push(Line::Del(before[i].clone()));
            i += 1;
        } else {
            out.push(Line::Ins(after[j].clone()));
            j += 1;
        }
    }
    out.extend(before[i..].iter().cloned().map(Line::Del));
    out.extend(after[j..].iter().cloned().map(Line::Ins));
    out
}

/// 画像を出力の隣へ写し、出力から見た相対のパスを返す。
///
/// **写した先の名前は、区切りと枚の識別子から決める** ── 入力の名前を使うと、
/// 変更前と変更後で同じ名前の画像が上書きし合う。
fn place_image(
    base: &Path,
    out_dir: &Path,
    side: &str,
    section: &str,
    slide: &Value,
) -> Result<(String, Vec<u8>), String> {
    let src = base.join(text_of(slide, "image"));
    let bytes =
        std::fs::read(&src).map_err(|e| format!("{}: 画像を読めない ── {e}", src.display()))?;
    let ext = src
        .extension()
        .and_then(|x| x.to_str())
        .unwrap_or("png")
        .to_owned();
    let rel = format!("img/{side}/{section}-{}.{ext}", text_of(slide, "id"));
    let dest = out_dir.join(&rel);
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("{}: 作れない ── {e}", parent.display()))?;
    }
    std::fs::write(&dest, &bytes).map_err(|e| format!("{}: 書けない ── {e}", dest.display()))?;
    Ok((rel, bytes))
}

/// 1枚の比較の中身。
struct Item {
    section: String,
    id: String,
    number: String,
    heading: String,
    heading_before: Option<String>,
    lede: String,
    lede_before: Option<String>,
    after_img: String,
    before_img: Option<String>,
    narration: String,
    narration_before: Option<String>,
    /// 変更前での並び（表紙を含めて0から）。
    before_pos: Option<usize>,
    pos: usize,
    same: bool,
}

/// 区切りごとの比較の中身 ── 区切りの識別子 ・ 名前 ・ 枚の並び。
type Sections = Vec<(String, String, Vec<Item>)>;

/// 入力を読み、検査し、画像を写して、枚ごとの比較の中身を組む。
fn items(references: &Path, input: &Path, out_dir: &Path) -> Result<(Value, Sections), String> {
    let body = std::fs::read_to_string(input)
        .map_err(|e| format!("{}: 読めない ── {e}", input.display()))?;
    let review: Value = serde_json::from_str(&body)
        .map_err(|e| format!("{}: JSON として読めない ── {e}", input.display()))?;
    let bad = validate::shape(&schema_path(references), &review);
    if !bad.is_empty() {
        return Err(format!(
            "入力の検査が通っていない ── 何も書き出さない:\n  {}",
            bad.iter()
                .map(|e| format!("× {e}"))
                .collect::<Vec<_>>()
                .join("\n  ")
        ));
    }
    let base = input.parent().unwrap_or(Path::new("."));
    let empty = Value::Null;
    let before = review.get("before").unwrap_or(&empty);
    let after = review.get("after").unwrap_or(&empty);
    let mut out = Vec::new();
    for sec in array_of(after, "sections") {
        let key = text_of(sec, "key");
        let old = array_of(before, "sections")
            .iter()
            .find(|s| text_of(s, "key") == key);
        let old_slides = old.map_or(&[][..], |s| array_of(s, "slides"));
        let mut list = Vec::new();
        for (pos, s) in array_of(sec, "slides").iter().enumerate() {
            let id = text_of(s, "id");
            let (a_rel, a_bytes) = place_image(base, out_dir, "after", &key, s)?;
            let found = old_slides
                .iter()
                .enumerate()
                .find(|(_, b)| text_of(b, "id") == id);
            let (before_img, before_pos, b_bytes, hb, nb, lb) = match found {
                Some((bpos, b)) => {
                    let (rel, bytes) = place_image(base, out_dir, "before", &key, b)?;
                    (
                        Some(rel),
                        Some(bpos),
                        Some(bytes),
                        Some(text_of(b, "heading")),
                        Some(text_of(b, "narration")),
                        Some(text_of(b, "lede")),
                    )
                }
                None => (None, None, None, None, None, None),
            };
            let lede = text_of(s, "lede");
            let heading = text_of(s, "heading");
            let narration = text_of(s, "narration");
            let same = b_bytes.as_deref() == Some(a_bytes.as_slice())
                && hb.as_deref() == Some(heading.as_str())
                && nb.as_deref() == Some(narration.as_str())
                && lb.as_deref() == Some(lede.as_str());
            list.push(Item {
                section: key.clone(),
                number: format!("{}枚目", pos + 1),
                id,
                heading,
                heading_before: hb,
                lede,
                lede_before: lb,
                after_img: a_rel,
                before_img,
                narration,
                narration_before: nb,
                before_pos,
                pos,
                same,
            });
        }
        out.push((key, text_of(sec, "name"), list));
    }
    Ok((review, out))
}

/// 枚ごとの確認の結果を、区切りと枚で引ける形へ組む。
fn log_of(review: &Value) -> BTreeMap<(String, String), Vec<&Value>> {
    let mut map: BTreeMap<(String, String), Vec<&Value>> = BTreeMap::new();
    for f in array_of(review, "log") {
        map.entry((text_of(f, "section"), text_of(f, "slide")))
            .or_default()
            .push(f);
    }
    map
}

fn log_table(parts: &Parts, rows: &[&Value]) -> Result<String, String> {
    if rows.is_empty() {
        return Ok(String::new());
    }
    let mut sorted: Vec<&Value> = rows.to_vec();
    sorted.sort_by_key(|f| {
        let g = text_of(f, "group");
        GROUPS
            .iter()
            .position(|(k, _)| *k == g)
            .unwrap_or(GROUPS.len())
    });
    let mut body = String::new();
    for f in sorted {
        let status = text_of(f, "status");
        body.push_str(&parts.part(
            "log-row",
            &[
                ("group", esc(group_name(&text_of(f, "group")))),
                ("status", esc(&status)),
                ("status_name", status_name(&status).to_owned()),
                ("found", esc(&text_of(f, "found"))),
                ("fixed", esc(&text_of(f, "fixed"))),
            ],
        )?);
    }
    parts.part("log", &[("rows", body)])
}

fn narration_block(parts: &Parts, label: &str, lines: &[(&str, String)]) -> Result<String, String> {
    // **原稿の無い枚（表紙など）には、原稿の欄を出さない** ── 「なし」を並べると、
    // 原稿を書き忘れた枚と区別できない
    if lines.is_empty() {
        return Ok(String::new());
    }
    let mut body = String::new();
    for (kind, text) in lines {
        body.push_str(&parts.part(
            "sentence",
            &[("kind", (*kind).to_owned()), ("text", esc(text))],
        )?);
    }
    parts.part("narration", &[("label", esc(label)), ("sentences", body)])
}

fn image(parts: &Parts, src: &str, alt: &str) -> Result<String, String> {
    parts.part("image", &[("src", esc(src)), ("alt", esc(alt))])
}

fn summary(parts: &Parts, review: &Value, counts: String) -> Result<String, String> {
    let mut lines = String::new();
    for l in array_of(review, "summary") {
        lines.push_str(&parts.part("line", &[("text", esc(l.as_str().unwrap_or_default()))])?);
    }
    parts.part("summary", &[("lines", lines), ("counts", esc(&counts))])
}

/// 比較ページ（`expand` なら確認記録）を組む。
fn compare(
    parts: &Parts,
    review: &Value,
    sections: &[(String, String, Vec<Item>)],
    expand: bool,
) -> Result<String, String> {
    let log = log_of(review);
    let mut nav = String::new();
    let mut body = String::new();
    let (mut total, mut changed) = (0, 0);
    for (key, name, list) in sections {
        let count = list.iter().filter(|x| !x.same).count();
        total += list.len();
        changed += count;
        nav.push_str(&parts.part(
            "nav-link",
            &[
                ("anchor", format!("s-{key}")),
                ("name", esc(name)),
                ("count", format!("{}枚 ／ 変更{count}", list.len())),
            ],
        )?);
        let mut items = String::new();
        for x in list {
            let mut badges = String::new();
            let (kind, text) = match (x.before_pos, x.same) {
                (None, _) => ("added", "追加"),
                (_, true) => ("same", "変更なし"),
                _ => ("changed", "変更"),
            };
            badges.push_str(&parts.part(
                "badge",
                &[("kind", kind.to_owned()), ("text", text.to_owned())],
            )?);
            if let Some(b) = x.before_pos {
                if b != x.pos {
                    badges.push_str(&parts.part(
                        "badge",
                        &[
                            ("kind", "moved".to_owned()),
                            ("text", format!("並べ替え（前は{}枚目）", b + 1)),
                        ],
                    )?);
                }
            }
            let rows = log
                .get(&(x.section.clone(), x.id.clone()))
                .cloned()
                .unwrap_or_default();
            let log_html = log_table(parts, &rows)?;
            let after_img = image(parts, &x.after_img, &format!("変更後　{}", x.heading))?;
            if x.same {
                items.push_str(
                    &parts.part(
                        "pair-same",
                        &[
                            ("anchor", format!("p-{}-{}", x.section, x.id)),
                            ("number", esc(&x.number)),
                            ("badges", badges),
                            ("heading", esc(&x.heading)),
                            (
                                "open",
                                if expand {
                                    " open".to_owned()
                                } else {
                                    String::new()
                                },
                            ),
                            (
                                "after",
                                after_img
                                    + &narration_block(
                                        parts,
                                        "原稿",
                                        &sentences(&x.narration)
                                            .into_iter()
                                            .map(|s| ("", s))
                                            .collect::<Vec<_>>(),
                                    )?,
                            ),
                            ("log", log_html),
                        ],
                    )?,
                );
                continue;
            }
            let (before_col, after_col) =
                if let (Some(bimg), Some(nb)) = (&x.before_img, &x.narration_before) {
                    let lines = diff(&sentences(nb), &sentences(&x.narration));
                    let left: Vec<(&str, String)> = lines
                        .iter()
                        .filter_map(|l| match l {
                            Line::Same(s) => Some(("", s.clone())),
                            Line::Del(s) => Some(("del", s.clone())),
                            Line::Ins(_) => None,
                        })
                        .collect();
                    let right: Vec<(&str, String)> = lines
                        .iter()
                        .filter_map(|l| match l {
                            Line::Same(s) => Some(("", s.clone())),
                            Line::Ins(s) => Some(("ins", s.clone())),
                            Line::Del(_) => None,
                        })
                        .collect();
                    let hb = x.heading_before.clone().unwrap_or_default();
                    let (mut hd_b, mut hd_a) = if hb == x.heading {
                        (String::new(), String::new())
                    } else {
                        (
                            parts.part(
                                "heading-diff",
                                &[
                                    ("kind", "del".to_owned()),
                                    ("label", "見出し".to_owned()),
                                    ("text", esc(&hb)),
                                ],
                            )?,
                            parts.part(
                                "heading-diff",
                                &[
                                    ("kind", "ins".to_owned()),
                                    ("label", "見出し".to_owned()),
                                    ("text", esc(&x.heading)),
                                ],
                            )?,
                        )
                    };
                    let lb = x.lede_before.clone().unwrap_or_default();
                    if lb != x.lede {
                        hd_b.push_str(&parts.part(
                            "heading-diff",
                            &[
                                ("kind", "del".to_owned()),
                                ("label", "導入文".to_owned()),
                                ("text", esc(&lb)),
                            ],
                        )?);
                        hd_a.push_str(&parts.part(
                            "heading-diff",
                            &[
                                ("kind", "ins".to_owned()),
                                ("label", "導入文".to_owned()),
                                ("text", esc(&x.lede)),
                            ],
                        )?);
                    }
                    (
                        image(parts, bimg, &format!("変更前　{hb}"))?
                            + &hd_b
                            + &narration_block(parts, "原稿（変更前）", &left)?,
                        after_img + &hd_a + &narration_block(parts, "原稿（変更後）", &right)?,
                    )
                } else {
                    (
                        parts.part(
                            "no-before",
                            &[(
                                "text",
                                "この枚は新しく追加したものである。変更前には無い。".to_owned(),
                            )],
                        )?,
                        after_img
                            + &narration_block(
                                parts,
                                "原稿（変更後）",
                                &sentences(&x.narration)
                                    .into_iter()
                                    .map(|s| ("ins", s))
                                    .collect::<Vec<_>>(),
                            )?,
                    )
                };
            items.push_str(&parts.part(
                "pair",
                &[
                    ("anchor", format!("p-{}-{}", x.section, x.id)),
                    ("number", esc(&x.number)),
                    ("badges", badges),
                    ("heading", esc(&x.heading)),
                    ("before", before_col),
                    ("after", after_col),
                    ("log", log_html),
                ],
            )?);
        }
        body.push_str(&parts.part(
            "section",
            &[
                ("anchor", format!("s-{key}")),
                ("name", esc(name)),
                ("items", items),
            ],
        )?);
    }
    parts.part(
        "page",
        &[
            (
                "kind",
                if expand {
                    "確認記録"
                } else {
                    "比較ページ"
                }
                .to_owned(),
            ),
            ("title", esc(&text_of(review, "title"))),
            (
                "summary",
                summary(parts, review, format!("全{total}枚 ／ 変更{changed}枚"))?,
            ),
            ("nav", parts.part("nav", &[("links", nav)])?),
            ("body", body),
        ],
    )
}

/// 台本を組む。変更後の画像と原稿の全文を、枚ごとに並べる。
fn script(
    parts: &Parts,
    review: &Value,
    sections: &[(String, String, Vec<Item>)],
) -> Result<String, String> {
    let mut nav = String::new();
    let mut body = String::new();
    let mut total = 0;
    for (key, name, list) in sections {
        total += list.len();
        nav.push_str(&parts.part(
            "nav-link",
            &[
                ("anchor", format!("s-{key}")),
                ("name", esc(name)),
                ("count", format!("{}枚", list.len())),
            ],
        )?);
        let mut items = String::new();
        for x in list {
            let lines: Vec<(&str, String)> = sentences(&x.narration)
                .into_iter()
                .map(|s| ("", s))
                .collect();
            items.push_str(&parts.part(
                "script-slide",
                &[
                    ("anchor", format!("p-{}-{}", x.section, x.id)),
                    ("number", esc(&x.number)),
                    ("heading", esc(&x.heading)),
                    ("image", image(parts, &x.after_img, &x.heading)?),
                    ("narration", narration_block(parts, "原稿", &lines)?),
                ],
            )?);
        }
        body.push_str(&parts.part(
            "section",
            &[
                ("anchor", format!("s-{key}")),
                ("name", esc(name)),
                ("items", items),
            ],
        )?);
    }
    parts.part(
        "page",
        &[
            ("kind", "台本".to_owned()),
            ("title", esc(&text_of(review, "title"))),
            ("summary", summary(parts, review, format!("全{total}枚"))?),
            ("nav", parts.part("nav", &[("links", nav)])?),
            ("body", body),
        ],
    )
}

/// 観点ごとの結果を組む。枚ごとに、7つの群の結果を1行へ並べ、下に明細を置く。
fn results(
    parts: &Parts,
    review: &Value,
    sections: &[(String, String, Vec<Item>)],
) -> Result<String, String> {
    let log = log_of(review);
    let mut heads = String::new();
    for (_, n) in GROUPS {
        heads.push_str(&parts.part("results-head", &[("name", n.to_owned())])?);
    }
    let mut nav = String::new();
    let mut body = String::new();
    let (mut fixed, mut pending, mut unchecked) = (0, 0, 0);
    for (key, name, list) in sections {
        nav.push_str(&parts.part(
            "nav-link",
            &[
                ("anchor", format!("s-{key}")),
                ("name", esc(name)),
                ("count", format!("{}枚", list.len())),
            ],
        )?);
        let mut rows = String::new();
        let mut details = String::new();
        for x in list {
            let got = log
                .get(&(x.section.clone(), x.id.clone()))
                .cloned()
                .unwrap_or_default();
            let mut cells = String::new();
            for (g, _) in GROUPS {
                let status = got
                    .iter()
                    .filter(|f| text_of(f, "group") == g)
                    .map(|f| text_of(f, "status"))
                    .max_by_key(|s| match s.as_str() {
                        "pending" => 3,
                        "fixed" => 2,
                        "ok" => 1,
                        _ => 0,
                    })
                    .unwrap_or_default();
                match status.as_str() {
                    "fixed" => fixed += 1,
                    "pending" => pending += 1,
                    "" => unchecked += 1,
                    _ => {}
                }
                cells.push_str(&parts.part(
                    "results-cell",
                    &[
                        ("status", esc(&status)),
                        ("status_name", status_name(&status).to_owned()),
                    ],
                )?);
            }
            rows.push_str(&parts.part(
                "results-row",
                &[
                    ("number", esc(&x.number)),
                    ("heading", esc(&x.heading)),
                    ("cells", cells),
                ],
            )?);
            let table = log_table(parts, &got)?;
            if !table.is_empty() {
                details.push_str(&parts.part(
                    "script-slide",
                    &[
                        ("anchor", format!("p-{}-{}", x.section, x.id)),
                        ("number", esc(&x.number)),
                        ("heading", esc(&x.heading)),
                        ("image", image(parts, &x.after_img, &x.heading)?),
                        ("narration", table),
                    ],
                )?);
            }
        }
        let table = parts.part("results-table", &[("heads", heads.clone()), ("rows", rows)])?;
        body.push_str(&parts.part(
            "section",
            &[
                ("anchor", format!("s-{key}")),
                ("name", esc(name)),
                ("items", table + &details),
            ],
        )?);
    }
    parts.part(
        "page",
        &[
            ("kind", "観点ごとの結果".to_owned()),
            ("title", esc(&text_of(review, "title"))),
            (
                "summary",
                summary(
                    parts,
                    review,
                    format!(
                        "修正 {fixed} ／ 保留 {pending} ／ 未確認 {unchecked}（枚 × 観点の群）"
                    ),
                )?,
            ),
            ("nav", parts.part("nav", &[("links", nav)])?),
            ("body", body),
        ],
    )
}

fn write(path: &Path, body: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("{}: 作れない ── {e}", parent.display()))?;
        }
    }
    std::fs::write(path, body).map_err(|e| format!("{}: 書けない ── {e}", path.display()))
}

/// 比較ページを1枚書き出す。画像は出力の隣の `img/` へ写す。
///
/// # Errors
///
/// 入力の検査が通らないとき、画像を読めないとき、書けないときに返す。
pub fn build_compare(references: &Path, input: &Path, out: &Path) -> Result<usize, String> {
    let out_dir = out.parent().unwrap_or(Path::new("."));
    let parts = Parts::load(&template_path(references))?;
    let (review, sections) = items(references, input, out_dir)?;
    let html = compare(&parts, &review, &sections, false)?;
    write(out, &html)?;
    Ok(sections.iter().map(|(_, _, l)| l.len()).sum())
}

/// 3つの成果物を HTML で書き出し、書き出したパスを返す。
///
/// # Errors
///
/// 入力の検査が通らないとき、画像を読めないとき、書けないときに返す。
pub fn build_exports(
    references: &Path,
    input: &Path,
    out_dir: &Path,
) -> Result<Vec<PathBuf>, String> {
    let parts = Parts::load(&template_path(references))?;
    let (review, sections) = items(references, input, out_dir)?;
    let mut done = Vec::new();
    for (name, html) in [
        ("script.html", script(&parts, &review, &sections)?),
        ("record.html", compare(&parts, &review, &sections, true)?),
        ("results.html", results(&parts, &review, &sections)?),
    ] {
        let path = out_dir.join(name);
        write(&path, &html)?;
        done.push(path);
    }
    Ok(done)
}

/// HTML を描画して PDF を書き出す。**ブラウザの場所は、呼ぶ側が渡す** ── どこに
/// 在るかを、この側で推測しない。
///
/// # Errors
///
/// ブラウザを起動できないとき、PDF が書き出されなかったときに返す。
pub fn print_pdf(browser: &Path, html: &Path, pdf: &Path) -> Result<(), String> {
    let abs =
        std::fs::canonicalize(html).map_err(|e| format!("{}: 読めない ── {e}", html.display()))?;
    let status = std::process::Command::new(browser)
        .args([
            "--headless",
            "--no-sandbox",
            "--disable-gpu",
            "--no-pdf-header-footer",
            &format!("--print-to-pdf={}", pdf.display()),
            &format!("file://{}", abs.display()),
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_err(|e| format!("{}: 起動できない ── {e}", browser.display()))?;
    if !status.success() || !pdf.exists() {
        return Err(format!("{}: PDF が書き出されなかった", pdf.display()));
    }
    Ok(())
}
