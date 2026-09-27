// SPDX-License-Identifier: MIT
//! 論点1つぶんの面。
//!
//! 開いているものは答えと裏づけと回答欄、まだのものは問いだけを持つ。
//!
//! **節は4つだけにする。** 同じことを3か所に書かない ── 論証（残った理由）・ 前提 ・
//! 残る危険 ・ 経過。**この順は、読み手が判定に使う順である。**

use crate::cell::{cell, esc, key, mark, pairs, table};
use crate::snapshot::Diff;
use crate::template::Parts;
use crate::topic::{kind_of, Topic, LETTERS};

/// よく使う差し戻しの理由。**値そのものは限定しない** ── 差し込みの見本である。
pub const REASONS: [&str; 3] = ["もっと単純に", "前提が違う", "別の道も見たい"];

/// 道筋の節の名前。
///
/// **差し戻しが在るものと無いものを、同じ名前で呼ばない** ── 論証の連鎖に「履歴」と
/// 付けると、在りもしない差し戻しを探させることになる。
pub const HISTORY: &str = "この答えの履歴（{n}件） ── 差し戻しで何が失効し、何へ変更したか";
/// 差し戻しの無い道筋の節の名前。
pub const PROGRESS: &str = "この答えに至る経過（{n}手） ── 何を問い、そこで何が判明したか";

fn fold(parts: &Parts, summary: &str, body: &str) -> Result<String, String> {
    parts.part(
        "fold",
        &[("summary", summary.to_owned()), ("body", body.to_owned())],
    )
}

/// 面に出す節の名前。**呼ぶ側が差し替えられる。**
#[derive(Debug, Clone)]
pub struct Labels {
    /// 差し戻しが在るときの道筋の名前。
    pub history: String,
    /// 差し戻しが無いときの道筋の名前。
    pub progress: String,
}

impl Default for Labels {
    fn default() -> Self {
        Self {
            history: HISTORY.to_owned(),
            progress: PROGRESS.to_owned(),
        }
    }
}

/// 論点1つぶんを組む。
///
/// # Errors
///
/// 出どころの種類が決められた値でないときと、根拠に支える先か出どころが無いときと、
/// 型と噛み合わないときに返す。
#[allow(clippy::too_many_lines)]
pub fn panel(
    parts: &Parts,
    t: &Topic,
    ask: bool,
    diff: &mut Diff,
    labels: &Labels,
) -> Result<String, String> {
    let qid = format!("Q{}", t.no);
    let mut out = vec![parts.part(
        "topic-head",
        &[("qid", qid.clone()), ("question", esc(&t.question))],
    )?];
    if !t.note.is_empty() {
        let body = diff.one(parts, "note", &t.note)?;
        out.push(parts.part("note", &[("body", body)])?);
    }

    if let Some((letter, concl)) = &t.pick {
        let body = diff.one(parts, "pick", concl)?;
        out.push(parts.part("answer", &[("letter", esc(letter)), ("body", body)])?);
    } else if !t.decision.is_empty() {
        let mut rows = Vec::new();
        for (head, body) in &t.decision {
            rows.push(vec![
                parts.part(
                    "status",
                    &[("cls", "done".to_owned()), ("label", esc(head))],
                )?,
                body.clone(),
            ]);
        }
        out.push(table(parts, &[String::new(), String::new()], &rows)?);
    }

    let mut folds: Vec<String> = Vec::new();

    // 完成イメージ ── **見出しは道具が作る。** 手で書かせるとブレストボードごとに違う形になる。
    // **畳まない** ── 畳むと、読み手は答えを文章だけで受け取ることになる
    let mut image = String::new();
    if !t.figures.is_empty() {
        // 図は縦方向へ並べる。横に並べると、縦横比の違う図が幅に合わせて縮み、
        // 文字が読めなくなる（実測 ── 742×100 の図が 380px で潰れた）
        let mut inner = String::new();
        for (i, (svg, cap)) in t.figures.iter().enumerate() {
            let caption = diff.mark(parts, "figures", i, 0, cap)?;
            inner.push_str(&parts.part("figure", &[("svg", svg.clone()), ("caption", caption)])?);
        }
        image.push_str(&parts.part("figures", &[("figures", inner)])?);
    }
    if !t.example.is_empty() {
        let body = diff.one(parts, "example", &t.example)?;
        image.push_str(&parts.part("example", &[("body", body)])?);
    }
    if !image.is_empty() {
        let mut what = Vec::new();
        if !t.figures.is_empty() {
            what.push(if t.figures.len() > 1 {
                format!("図{}枚", t.figures.len())
            } else {
                "図".to_owned()
            });
        }
        if !t.example.is_empty() {
            what.push("実例".to_owned());
        }
        folds.push(parts.part(
            "fold-open",
            &[
                (
                    "summary",
                    format!("この答えの完成イメージ ── {}", what.join("と、")),
                ),
                ("body", image),
            ],
        )?);
    }

    for (part, claim, kind, src) in &t.grounds {
        if kind_of(kind).is_none() {
            return Err(format!(
                "論点{}: 出どころの種類が「{kind}」。使えるのは {} である",
                t.no,
                crate::topic::KINDS
                    .iter()
                    .map(|(k, _, _)| *k)
                    .collect::<Vec<_>>()
                    .join("／")
            ));
        }
        if src.trim().is_empty() || part.trim().is_empty() {
            return Err(format!(
                "論点{}: 根拠に、支える先か出どころが無い ── 「{}…」",
                t.no,
                claim.chars().take(20).collect::<String>()
            ));
        }
    }

    let mut argue = String::new();
    if !t.kept.is_empty() {
        let mut rows = Vec::new();
        for (i, o) in t.kept.iter().enumerate() {
            // **印を二重にしない** ── 書き手が付けた印が在る欄は、そちらを残す
            let name = if o.marked() {
                let lead = parts.part("lead", &[("text", o.name.clone())])?;
                mark(parts, &lead, &o.before, &o.why, false, None)?
            } else {
                let text = diff.mark(parts, "kept", i, 0, &o.name)?;
                parts.part("lead", &[("text", text)])?
            };
            let gist = diff.mark(parts, "kept", i, 1, &o.gist)?;
            let cost = diff.mark(parts, "kept", i, 2, &o.cost)?;
            rows.push(vec![
                key(parts, LETTERS.get(i).copied().unwrap_or("?"), "")?,
                name,
                cell(parts, &gist)?,
                parts.part("cost", &[("body", cell(parts, &cost)?)])?,
            ]);
        }
        argue.push_str(&parts.part(
            "note-s",
            &[(
                "body",
                format!(
                    "反証を通過した案 {}件。このうち1つを残し、他は代償が重いか、前提を壊す。",
                    t.kept.len()
                ),
            )],
        )?);
        argue.push_str(&table(
            parts,
            &[
                String::new(),
                "案".to_owned(),
                "中身".to_owned(),
                "代償".to_owned(),
            ],
            &rows,
        )?);
    }
    if !t.dropped.is_empty() {
        let mut rows = Vec::new();
        for (di, (x, w)) in t.dropped.iter().enumerate() {
            let text = diff.mark(parts, "dropped", di, 0, x)?;
            let lead = parts.part("lead", &[("text", text)])?;
            let why = diff.mark(parts, "dropped", di, 1, w)?;
            rows.push(vec![
                key(parts, "×", "out")?,
                mark(parts, &lead, "この案は残っていた", w, true, None)?,
                cell(parts, &why)?,
            ]);
        }
        argue.push_str(&parts.part(
            "note-s",
            &[(
                "body",
                format!("除外した案 {}件 ── 何が壊れるか。", t.dropped.len()),
            )],
        )?);
        argue.push_str(&table(
            parts,
            &[
                String::new(),
                "除外した案".to_owned(),
                "何が壊れるか".to_owned(),
            ],
            &rows,
        )?);
    }
    let mut ti = 0;
    for tb in &t.tables {
        let mut rows = Vec::new();
        for (head, values) in &tb.rows {
            let first = if tb.plain {
                let text = diff.mark(parts, "tables", ti, 1, head)?;
                parts.part("lead", &[("text", text)])?
            } else {
                key(parts, head, "")?
            };
            let mut row = vec![first];
            for (j, value) in values.iter().enumerate() {
                let text = diff.mark(parts, "tables", ti, 2 + j, value)?;
                row.push(cell(parts, &text)?);
            }
            rows.push(row);
            ti += 1;
        }
        if !tb.lead.is_empty() {
            argue.push_str(&parts.part("note-s", &[("body", tb.lead.clone())])?);
        }
        let caption = parts.part("lead", &[("text", esc(&tb.caption))])?;
        argue.push_str(&parts.part("note-s", &[("body", caption)])?);
        let mut head = vec![String::new()];
        head.extend(tb.columns.iter().cloned());
        argue.push_str(&table(parts, &head, &rows)?);
    }
    if !argue.is_empty() {
        folds.push(fold(
            parts,
            &format!(
                "この答えが残った理由 ── 通過 {}件 ／ 除外 {}件",
                t.kept.len(),
                t.dropped.len()
            ),
            &argue,
        )?);
    }

    if !t.grounds.is_empty() {
        let mut rows = Vec::new();
        for (gi, (p, c, k, src)) in t.grounds.iter().enumerate() {
            let (cls, label) = kind_of(k).unwrap_or(("", ""));
            let supports = diff.mark(parts, "grounds", gi, 0, p)?;
            let basis = diff.mark(parts, "grounds", gi, 1, c)?;
            let source = diff.mark(parts, "grounds", gi, 3, src)?;
            rows.push(vec![
                parts.part("part", &[("body", supports)])?,
                basis,
                parts.part(
                    "kind",
                    &[("cls", cls.to_owned()), ("label", label.to_owned())],
                )? + &parts.part("small", &[("body", source)])?,
            ]);
        }
        folds.push(fold(
            parts,
            &format!(
                "この答えの前提（{}件） ── 何に依拠しているか",
                t.grounds.len()
            ),
            &table(
                parts,
                &[
                    "結論のどこを支えるか".to_owned(),
                    "もとにしたこと".to_owned(),
                    "その出どころ".to_owned(),
                ],
                &rows,
            )?,
        )?);
    }

    if !t.weaknesses.is_empty() {
        let mut rows = Vec::new();
        let mut naked = 0;
        for (wi, (item, how)) in t.weaknesses.iter().enumerate() {
            let left = diff.mark(parts, "weaknesses", wi, 0, item)?;
            if how.is_empty() {
                naked += 1;
                rows.push(vec![
                    cell(parts, &left)?,
                    parts.part("cost", &[("body", "扱いが未記載である".to_owned())])?,
                ]);
            } else {
                let right = diff.mark(parts, "weaknesses", wi, 1, how)?;
                rows.push(vec![cell(parts, &left)?, cell(parts, &right)?]);
            }
        }
        let lead_text = parts.part(
            "lead",
            &[("text", "いずれも、この答えを覆さない。".to_owned())],
        )? + "覆しうるものは反証で除外済みであり、ここには残存しない ── "
            + &parts.part(
                "lead",
                &[("text", "承認を保留する理由にはならない。".to_owned())],
            )?;
        let kinds = "扱いは3種である ── ".to_owned()
            + &parts.part("lead", &[("text", "対象外".to_owned())])?
            + "（この答えでは解決しない。解決する手段が別に要る）／ "
            + &parts.part("lead", &[("text", "後続で決定".to_owned())])?
            + "（この答えの内側で、どこで決めるかが定まっている）／ "
            + &parts.part("lead", &[("text", "解消済".to_owned())])?
            + "（既に解決した）。";
        let mut lead = parts.part("note-s", &[("body", lead_text)])?
            + &parts.part("note-s", &[("body", kinds)])?;
        if naked > 0 {
            let said = parts.part("lead", &[("text", format!("{naked}件に扱いが無い。"))])?;
            lead.push_str(&parts.part("note-s", &[("body", said)])?);
        }
        folds.push(fold(
            parts,
            &format!("この答えが扱わない範囲（{}件）", t.weaknesses.len()),
            &(lead + &table(parts, &["事項".to_owned(), "扱い".to_owned()], &rows)?),
        )?);
    }

    // **欠陥は、適用範囲外と分離する。** 同じ節に混ぜると、制約が欠陥に見える
    if !t.defects.is_empty() {
        let mut rows = Vec::new();
        for (a, b) in &t.defects {
            rows.push(vec![cell(parts, a)?, cell(parts, b)?]);
        }
        let lead = parts.part(
            "lead",
            &[(
                "text",
                "この答えの中で、まだ修正していない誤りである。".to_owned(),
            )],
        )? + "適用範囲外とは別に記載する ── 混在させると、制約が誤りに見える。";
        folds.push(fold(
            parts,
            &format!("未修正の誤り（{}件）", t.defects.len()),
            &(parts.part("note-s", &[("body", lead)])?
                + &table(parts, &["誤り".to_owned(), "現状".to_owned()], &rows)?),
        )?);
    }

    // **1つの欄に1つのことだけを入れる。** 道筋 ・ 分かったこと ・ 要求する事項 ・ 面は
    // 別のことなので、器も名前も別になる。**名前はここで確定させる**
    if !t.path.is_empty() {
        let mut body = String::new();
        for (i, x) in t.path.iter().enumerate() {
            let text = diff.mark(parts, "path", i, 0, x)?;
            body.push_str(&parts.part("path-item", &[("body", text)])?);
        }
        let returned = body.contains("g-returned");
        let name = if returned {
            &labels.history
        } else {
            &labels.progress
        }
        .replace("{n}", &t.path.len().to_string());
        let inner = parts.part("note-s", &[("body", "古い順".to_owned())])?
            + &parts.part("path", &[("items", body)])?;
        folds.push(fold(parts, &name, &inner)?);
    }
    if !t.found.is_empty() {
        let mut items = Vec::new();
        for (i, x) in t.found.iter().enumerate() {
            items.push(diff.mark(parts, "found", i, 0, x)?);
        }
        folds.push(fold(
            parts,
            &format!("反証で分かったこと（{}件）", t.found.len()),
            &pairs(parts, &items, "何が分かったか", "だから何が決まったか")?,
        )?);
    }
    if !t.costs.is_empty() {
        let mut items = Vec::new();
        for (i, x) in t.costs.iter().enumerate() {
            items.push(diff.mark(parts, "costs", i, 0, x)?);
        }
        folds.push(fold(
            parts,
            &format!("この答えが要求する事項（{}件）", t.costs.len()),
            &pairs(parts, &items, "要求する事項", "理由")?,
        )?);
    }
    for (title, body) in &t.extras {
        folds.push(fold(parts, &esc(title), body)?);
    }

    if !folds.is_empty() {
        out.push(parts.part("folds", &[("body", folds.concat())])?);
    }

    // 決着した論点は回答欄を持たない。**決まったことを、もう一度聞かない**
    if ask && !t.settled() && t.decided() {
        let mut reasons = String::new();
        for r in REASONS {
            reasons.push_str(&parts.part("reason-button", &[("text", r.to_owned())])?);
        }
        out.push(parts.part(
            "form",
            &[
                ("qid", qid),
                ("title", esc(&t.question)),
                ("reasons", reasons),
            ],
        )?);
    }
    parts.part("topic", &[("body", out.concat())])
}
