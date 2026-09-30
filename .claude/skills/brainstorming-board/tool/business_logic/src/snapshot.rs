// SPDX-License-Identifier: MIT
//! この回で何が変わったかを、ブレストボードが自分で示す。
//!
//! **毎回どこを直したかを、書き手が文章で言うのは仕組みではない。** 前の回の生成物を
//! 記録しておき、ブレストボードが自分で印を付ける。
//!
//! **印を付ける欄と、数える欄は一致させる。** 片方にしか無い欄は、変わっても画面に出ない
//! （実測 ── 完成イメージと案の変更が1つも出なかった）。

use std::collections::BTreeMap;

use serde_json::{json, Value};

use crate::cell::plain;
use crate::seq::opcodes;
use crate::topic::Topic;

/// 比べる欄。**この並びが正本である。**
pub const FIELDS: [&str; 13] = [
    "note",
    "pick",
    "answer",
    "figures",
    "example",
    "kept",
    "dropped",
    "path",
    "found",
    "costs",
    "weaknesses",
    "tables",
    "grounds",
];

/// 足した欄の、変更前として出す語。
pub const ADDED: &str = "（この回で足した）";

/// 印がどの節に在るかを、読み手の言葉で持つ。
const WHERE: [(&str, &str); 8] = [
    ("note", "前書き"),
    ("pick", "答え"),
    ("path", "道筋"),
    ("found", "分かったこと"),
    ("costs", "要求事項"),
    ("weaknesses", "扱わない範囲"),
    ("tables", "表"),
    ("grounds", "根拠"),
];

fn where_of(field: &str) -> &str {
    WHERE
        .iter()
        .find(|(k, _)| *k == field)
        .map_or(field, |(_, v)| *v)
}

fn sha1_head(body: &str) -> String {
    use sha1::{Digest as _, Sha1};
    Sha1::digest(body.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>()
        .chars()
        .take(12)
        .collect()
}

/// 1つの論点の姿。**1行を欄の並びとして持つ** ── 行の中のどの欄が変わったかまで見る。
type Rows = Vec<Vec<String>>;

/// 論点の姿を記録する。**次の回で、この記録と比べる。**
#[must_use]
pub fn snapshot(topics: &[Topic]) -> Value {
    let mut out = serde_json::Map::new();
    for t in topics {
        let one = |x: &str| vec![vec![x.to_owned()]];
        let mut rows: BTreeMap<&str, Rows> = BTreeMap::new();
        rows.insert("note", one(&t.note));
        rows.insert(
            "pick",
            one(t.pick.as_ref().map_or("", |(_, body)| body.as_str())),
        );
        rows.insert("answer", one(&t.answer));
        // 図は中身が長いので、要約（sha1 の頭）で比べる ── 描き直しも変更である
        rows.insert(
            "figures",
            t.figures
                .iter()
                .map(|(svg, cap)| vec![cap.clone(), sha1_head(svg)])
                .collect(),
        );
        rows.insert("example", one(&t.example));
        rows.insert(
            "kept",
            t.kept
                .iter()
                .map(|o| vec![o.name.clone(), o.gist.clone(), o.cost.clone()])
                .collect(),
        );
        rows.insert(
            "dropped",
            t.dropped
                .iter()
                .map(|(a, b)| vec![a.clone(), b.clone()])
                .collect(),
        );
        rows.insert("path", t.path.iter().map(|x| vec![x.clone()]).collect());
        rows.insert("found", t.found.iter().map(|x| vec![x.clone()]).collect());
        rows.insert("costs", t.costs.iter().map(|x| vec![x.clone()]).collect());
        rows.insert(
            "weaknesses",
            t.weaknesses
                .iter()
                .map(|(item, how)| {
                    if how.is_empty() {
                        vec![item.clone()]
                    } else {
                        vec![item.clone(), how.clone()]
                    }
                })
                .collect(),
        );
        rows.insert(
            "tables",
            t.tables
                .iter()
                .flat_map(|tb| {
                    tb.rows.iter().map(move |(head, values)| {
                        let mut row = vec![tb.caption.clone(), head.clone()];
                        row.extend(values.iter().cloned());
                        row
                    })
                })
                .collect(),
        );
        rows.insert(
            "grounds",
            t.grounds
                .iter()
                .map(|(a, b, c, d)| vec![a.clone(), b.clone(), c.clone(), d.clone()])
                .collect(),
        );
        // **並びは宣言の順である** ── 並べ替えると、前の回との突き合わせが崩れる
        let mut one_topic = serde_json::Map::new();
        for field in FIELDS {
            one_topic.insert(
                field.to_owned(),
                json!(rows.remove(field).unwrap_or_default()),
            );
        }
        out.insert(t.no.to_string(), Value::Object(one_topic));
    }
    Value::Object(out)
}

/// 前の回との違い。**欄ごとに比べる。**
///
/// 並びを突き合わせてから欄を比べる ── 位置だけで比べると、1行足しただけで以降が全部
/// 「変わった」と出る（実測 ── 実際に出た）。
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct Diff {
    /// 前の回の記録が在るか。
    pub on: bool,
    /// 変わった欄の数。
    pub n: usize,
    /// 論点の番号。
    pub no: usize,
    /// 一覧へ出す行。`(印の番号, どの節か, 抜き書き)`
    pub items: Vec<(String, String, String)>,
    /// 前の回の、対応する行。
    pair: BTreeMap<(String, usize), Option<Vec<String>>>,
    /// 行ごとに1件だけ一覧へ出す。
    rows: BTreeMap<(String, usize), String>,
    /// 振った印の数。
    seen: usize,
    /// いまの姿。
    now: BTreeMap<String, Rows>,
}

fn rows_of(value: &Value, field: &str) -> Rows {
    value
        .get(field)
        .and_then(|x| x.as_array())
        .map(|rows| {
            rows.iter()
                .map(|row| {
                    row.as_array()
                        .map(|cells| {
                            cells
                                .iter()
                                .map(|c| {
                                    c.as_str().map_or_else(
                                        || c.to_string(),
                                        std::borrow::ToOwned::to_owned,
                                    )
                                })
                                .collect()
                        })
                        .unwrap_or_default()
                })
                .collect()
        })
        .unwrap_or_default()
}

/// 行を1つの鍵へ畳む。**単位の区切りは、本文に現れない字を使う。**
fn key_of(row: &[String]) -> String {
    row.join("\u{241f}")
}

/// 一覧に出す、その行の見出し。**行を見分けられる欄を選ぶ。**
fn label_of(now: &BTreeMap<String, Rows>, field: &str, i: usize, fallback: &str) -> String {
    let Some(row) = now.get(field).and_then(|rows| rows.get(i)) else {
        return fallback.to_owned();
    };
    if field == "tables" && row.len() >= 2 {
        return format!("{}／{}", row[0], row[1]);
    }
    row.first().cloned().unwrap_or_else(|| fallback.to_owned())
}

impl Diff {
    /// 前の回と比べる。**記録が無ければ、印を1つも付けない。**
    #[must_use]
    pub fn new(t: &Topic, prev: Option<&Value>) -> Self {
        let was = prev.and_then(|p| p.get(t.no.to_string()));
        let Some(was) = was else {
            return Self {
                no: t.no,
                ..Self::default()
            };
        };
        let taken = snapshot(std::slice::from_ref(t));
        let now_value = taken.get(t.no.to_string()).cloned().unwrap_or(Value::Null);
        let mut now: BTreeMap<String, Rows> = BTreeMap::new();
        let mut pair: BTreeMap<(String, usize), Option<Vec<String>>> = BTreeMap::new();
        let mut n = 0;
        for field in FIELDS {
            let old = rows_of(was, field);
            let new = rows_of(&now_value, field);
            let old_keys: Vec<String> = old.iter().map(|r| key_of(r)).collect();
            let new_keys: Vec<String> = new.iter().map(|r| key_of(r)).collect();
            let old_ref: Vec<&str> = old_keys.iter().map(String::as_str).collect();
            let new_ref: Vec<&str> = new_keys.iter().map(String::as_str).collect();
            for (kind, i1, i2, j1, j2) in opcodes(&old_ref, &new_ref) {
                for k in j1..j2 {
                    let at = i1 + (k - j1);
                    let taken = match kind {
                        crate::seq::Op::Equal => old.get(at).cloned(),
                        crate::seq::Op::Replace if at < i2 => old.get(at).cloned(),
                        _ => None,
                    };
                    pair.insert((field.to_owned(), k), taken);
                }
            }
            for (j, row) in new.iter().enumerate() {
                let old_row = pair.get(&(field.to_owned(), j)).cloned().flatten();
                for (c, value) in row.iter().enumerate() {
                    if value.is_empty() {
                        continue;
                    }
                    match &old_row {
                        None => n += 1,
                        Some(old_row) if c >= old_row.len() || old_row[c] != *value => n += 1,
                        Some(_) => {}
                    }
                }
            }
            now.insert(field.to_owned(), new);
        }
        Self {
            on: true,
            n,
            no: t.no,
            items: Vec::new(),
            pair,
            rows: BTreeMap::new(),
            seen: 0,
            now,
        }
    }

    /// 変わっていれば印にする。**変わっていなければ、何も足さない。**
    ///
    /// 印には番号を振る ── **引き出しから、その印まで直に跳ぶため**である。
    ///
    /// # Errors
    ///
    /// 型と噛み合わないときに返す。
    pub fn mark(
        &mut self,
        parts: &crate::template::Parts,
        field: &str,
        i: usize,
        j: usize,
        text: &str,
    ) -> Result<String, String> {
        if !self.on || text.is_empty() {
            return Ok(text.to_owned());
        }
        let row = self.pair.get(&(field.to_owned(), i)).cloned().flatten();
        let added = row.as_ref().is_none_or(|r| j >= r.len());
        if !added && row.as_ref().is_some_and(|r| r[j] == text) {
            return Ok(text.to_owned());
        }
        let cid = format!("c{}-{}", self.no, self.seen);
        self.seen += 1;
        // **一覧は行ごとに1件にする。** 欄ごとに出すと、「持つ」だけの行が並ぶ
        if let std::collections::btree_map::Entry::Vacant(seat) =
            self.rows.entry((field.to_owned(), i))
        {
            seat.insert(cid.clone());
            self.items.push((
                cid.clone(),
                where_of(field).to_owned(),
                plain(&label_of(&self.now, field, i, text), 46),
            ));
        }
        let before = if added {
            ADDED.to_owned()
        } else {
            row.map(|r| r[j].clone()).unwrap_or_default()
        };
        let why = if added {
            "この回で足した"
        } else {
            "この回で変わった"
        };
        crate::cell::mark(parts, text, &before, why, false, Some(&cid))
    }

    /// 1つしか無い欄（前書き ・ 答え）に当てる。
    ///
    /// # Errors
    ///
    /// 型と噛み合わないときに返す。
    pub fn one(
        &mut self,
        parts: &crate::template::Parts,
        field: &str,
        text: &str,
    ) -> Result<String, String> {
        self.mark(parts, field, 0, 0, text)
    }
}
