// SPDX-License-Identifier: MIT
//! 論点をタブ1枚にまとめ、開いている論点に回答欄を付ける。
//!
//! 先頭のタブは「現在地」── どれが決着し、どれが開いているかの一覧である。決着した論点も
//! 同じ1枚に残す ── 後の論点は、前の決着を前提にしている。
//!
//! **いま見る論点を渡すと、それだけが回答欄を持つ。** 待ちの論点は薄くなり、回答欄を
//! 持たない ── **どれに答えるかを利用者に判定させない。**

use std::collections::BTreeMap;

use serde_json::Value;

use crate::audit::audit;
use crate::cell::{esc, key, table};
use crate::panel::{panel, Labels};
use crate::snapshot::Diff;
use crate::template::Parts;
use crate::topic::{status_label, Topic};

/// 1枚を組むために渡すもの。**呼ぶ側が決める。**
#[derive(Debug, Default)]
pub struct Deck {
    /// ブレストボードの題。
    pub theme: String,
    /// ブレストボードの名前（回答が指す先）。
    pub board: String,
    /// 何回目か。
    pub round: usize,
    /// ブレストボードが何を決めるかの1文。
    pub intro: String,
    /// いま見る論点。`(番号, なぜいま開いたか)`
    pub queue: Vec<(usize, String)>,
    /// 論点に属さない補足。`(見出し, 中身)`
    pub extras: Vec<(String, String)>,
    /// 前の回の基準。
    pub prev: Option<Value>,
    /// 入力の中身から計算した見た目。
    pub style: String,
    /// 節の名前。
    pub labels: Labels,
    /// 動き。**正本のファイルから来る** ── この側は持たない。
    pub js: String,
}

/// 組んだ結果。
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct Made {
    /// 組んだ1枚（`<body>` の中身）。
    pub body: String,
    /// 出す前の検査で見つけたこと。
    pub notes: Vec<String>,
    /// この回で変わった数。
    pub changed: usize,
}

fn fold(parts: &Parts, summary: &str, body: &str) -> Result<String, String> {
    parts.part(
        "fold",
        &[("summary", summary.to_owned()), ("body", body.to_owned())],
    )
}

/// 論点の状態の札。
fn chip(parts: &Parts, t: &Topic, queued: bool, front: bool) -> Result<String, String> {
    if t.settled() {
        return parts.part(
            "status",
            &[("cls", "done".to_owned()), ("label", "決着".to_owned())],
        );
    }
    if !queued {
        let cls = if t.status == "open" { "open" } else { "wait" };
        return parts.part(
            "status",
            &[
                ("cls", cls.to_owned()),
                ("label", status_label(&t.status).unwrap_or("未").to_owned()),
            ],
        );
    }
    if front {
        return parts.part(
            "status",
            &[("cls", "now".to_owned()), ("label", "いま見る".to_owned())],
        );
    }
    parts.part(
        "status",
        &[("cls", "wait".to_owned()), ("label", "待ち".to_owned())],
    )
}

/// 1枚を組む。
///
/// # Errors
///
/// 反証を通過した案が1つしかないときと、型と噛み合わないときに返す。
#[allow(clippy::too_many_lines)]
pub fn build(parts: &Parts, topics: &[Topic], deck: &Deck) -> Result<Made, String> {
    for t in topics {
        if t.kept.len() == 1 {
            return Err(format!(
                "論点{}: 反証を通過した案が1つしかない。論点の立て方を再確認する ── 1つしか残らないなら、それは選択ではない。まだ案を出していない論点は、案を空にして置く",
                t.no
            ));
        }
    }
    let notes = audit(topics, &deck.extras, &deck.queue);

    let front: BTreeMap<usize, String> = deck.queue.iter().cloned().collect();
    let queued = !deck.queue.is_empty();
    let order: BTreeMap<usize, usize> = topics
        .iter()
        .enumerate()
        .map(|(i, t)| (t.no, i + 1))
        .collect();
    let mut diffs: BTreeMap<usize, Diff> = topics
        .iter()
        .map(|t| (t.no, Diff::new(t, deck.prev.as_ref())))
        .collect();

    let mut rows = Vec::new();
    for t in topics {
        let answer = if t.answer.is_empty() {
            parts.part("dim", &[])?
        } else {
            diffs
                .get_mut(&t.no)
                .ok_or("論点が無い")?
                .one(parts, "answer", &t.answer)?
        };
        rows.push(vec![
            key(parts, &t.no.to_string(), "")?,
            esc(&t.question),
            chip(parts, t, queued, front.contains_key(&t.no))?,
            answer,
        ]);
    }

    let mut lead = String::new();
    if !front.is_empty() {
        let mut items = String::new();
        for (no, why) in &deck.queue {
            let Some(at) = order.get(no) else { continue };
            let label = topics
                .iter()
                .find(|t| t.no == *no)
                .map(|t| t.label.clone())
                .unwrap_or_default();
            items.push_str(&parts.part(
                "queue-item",
                &[
                    ("at", at.to_string()),
                    ("no", no.to_string()),
                    ("label", esc(&label)),
                    ("why", why.clone()),
                ],
            )?);
        }
        let waiting = topics
            .iter()
            .filter(|t| !t.settled() && !front.contains_key(&t.no))
            .count();
        let tail = if waiting > 0 {
            parts.part(
                "waiting",
                &[(
                    "body",
                    format!("残り {waiting} 件は、上流が決まるまで動く。回答欄は無い。"),
                )],
            )?
        } else {
            String::new()
        };
        lead = parts.part(
            "queue",
            &[
                ("count", front.len().to_string()),
                ("items", items),
                ("tail", tail),
            ],
        )?;
    }

    let changed: usize = if deck.prev.is_some() {
        topics.iter().map(|t| diffs[&t.no].n).sum()
    } else {
        0
    };
    let mut changes = String::new();
    if deck.prev.is_some() {
        if changed > 0 {
            let mut rows = Vec::new();
            for t in topics {
                let d = &diffs[&t.no];
                if d.n == 0 {
                    continue;
                }
                rows.push(vec![
                    parts.part(
                        "jump",
                        &[
                            ("at", order[&t.no].to_string()),
                            ("no", t.no.to_string()),
                            ("label", esc(&t.label)),
                        ],
                    )?,
                    format!("{} か所", d.n),
                ]);
            }
            let body = parts.part(
                "note-s",
                &[(
                    "body",
                    "本文の中で、".to_owned()
                        + &parts.part(
                            "lead",
                            &[("text", "色の付いた欄が今回の変更である".to_owned())],
                        )?
                        + " ── 押すと前の回の中身が開く。",
                )],
            )? + &table(parts, &["論点".to_owned(), "変わった欄".to_owned()], &rows)?;
            changes = fold(
                parts,
                &format!("この回で変わったところ（{changed} か所）"),
                &body,
            )?;
        } else {
            changes = fold(
                parts,
                "この回で変わったところ（0 か所）",
                &parts.part(
                    "note-s",
                    &[("body", "前の回から、中身は1つも変わっていない。".to_owned())],
                )?,
            )?;
        }
    }

    let mut panels_extra = String::new();
    for (title, body) in &deck.extras {
        panels_extra.push_str(&fold(parts, title, body)?);
    }
    let now = parts.part(
        "front",
        &[
            ("style", deck.style.clone()),
            (
                "intro",
                if deck.intro.is_empty() {
                    String::new()
                } else {
                    parts.part("note", &[("body", deck.intro.clone())])?
                },
            ),
            ("lead", lead),
            (
                "list",
                table(
                    parts,
                    &[
                        "#".to_owned(),
                        "論点".to_owned(),
                        "状態".to_owned(),
                        "いまの答え".to_owned(),
                    ],
                    &rows,
                )?,
            ),
            ("changes", changes),
            ("panels", panels_extra),
        ],
    )?;

    let mut tabs = parts.part("tab-front", &[])?;
    let mut panes = parts.part("pane-front", &[("body", now)])?;
    for (i, t) in topics.iter().enumerate() {
        let at = i + 1;
        let cls = if !queued || t.settled() {
            String::new()
        } else if front.contains_key(&t.no) {
            " class=\"now\"".to_owned()
        } else {
            " class=\"wait\"".to_owned()
        };
        tabs.push_str(&parts.part(
            "tab",
            &[
                ("at", at.to_string()),
                ("no", t.no.to_string()),
                ("cls", cls),
                ("label", esc(&t.label)),
                ("chip", chip(parts, t, queued, front.contains_key(&t.no))?),
            ],
        )?);
        let ask = !queued || front.contains_key(&t.no);
        let body = panel(
            parts,
            t,
            ask,
            diffs.get_mut(&t.no).ok_or("論点が無い")?,
            &deck.labels,
        )?;
        panes.push_str(&parts.part("pane", &[("at", at.to_string()), ("body", body)])?);
    }

    let open_now = topics
        .iter()
        .filter(|t| !t.settled() && t.decided() && (!queued || front.contains_key(&t.no)))
        .count();
    let send = parts.part("send", &[("open", open_now.to_string())])?;

    // この回の変更を、**どの画面からでも開ける引き出し**にする。
    // 現在地の節だけに置くと、他の論点を見ているあいだは何も見えない
    let mut drawer = String::new();
    if deck.prev.is_some() && changed > 0 {
        let mut items = String::new();
        for t in topics {
            let d = &diffs[&t.no];
            if d.items.is_empty() {
                continue;
            }
            items.push_str(&parts.part(
                "drawer-topic",
                &[
                    ("no", t.no.to_string()),
                    ("label", esc(&t.label)),
                    ("count", d.n.to_string()),
                ],
            )?);
            for (cid, where_, excerpt) in &d.items {
                items.push_str(&parts.part(
                    "drawer-item",
                    &[
                        ("at", order[&t.no].to_string()),
                        ("cid", cid.clone()),
                        ("where", where_.clone()),
                        ("excerpt", excerpt.clone()),
                    ],
                )?);
            }
        }
        drawer = parts.part("drawer-toggle", &[("count", changed.to_string())])?
            + &parts.part("drawer", &[("items", items)])?;
    }

    // `<body>` は公開時の器が用意する。ここで書くと入れ子になるので、器は div で持つ
    let body = parts.part(
        "board",
        &[
            ("board", esc(&deck.board)),
            ("round", deck.round.to_string()),
            ("theme", esc(&deck.theme)),
            ("theme_text", esc(&deck.theme)),
            ("tabs", tabs),
            ("panes", panes),
            ("send", send),
            ("drawer", drawer),
            ("script", format!("<script>{}</script>", deck.js)),
        ],
    )?;
    Ok(Made {
        body,
        notes,
        changed,
    })
}

/// 1枚を、そのまま公開できる HTML として組む。
///
/// # Errors
///
/// 型と噛み合わないときに返す。
pub fn page(parts: &Parts, title: &str, css: &str, body: &str) -> Result<String, String> {
    parts.part(
        "page",
        &[
            ("title", esc(title)),
            ("head", parts.part("head", &[])?),
            ("style", css.to_owned()),
            ("body", body.to_owned()),
        ],
    )
}
