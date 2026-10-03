// SPDX-License-Identifier: MIT
//! ソースコードを、言語ごとのコードブロックとして組む。
//!
//! **印は行の単位で付ける。** 語の単位で付けると、色付けが挿入した要素と交差して
//! どちらかが壊れる ── コードの変更は行の単位で読むものなので、行で足りる。
//!
//! **色付けは近似である。** 構文解析を実施せず、行ごとに式を適用する ── 文字列の中の
//! 予約語のような事例は取り違える。読解の補助であり、判定の根拠にしない。
//!
//! **HTML の形は、ここが持たない** ── `references/acdr.template.html` が持つ。

use std::collections::HashMap;

use regex::Regex;
use serde_json::Value;

use crate::markdown::esc;
use crate::template::Parts;

/// 拡張子と言語の対応。**ここに無い拡張子は、コードとして扱わない。**
const LANGS: [(&str, &str); 24] = [
    (".py", "python"),
    (".pyi", "python"),
    (".js", "js"),
    (".mjs", "js"),
    (".cjs", "js"),
    (".ts", "js"),
    (".tsx", "js"),
    (".jsx", "js"),
    (".json", "json"),
    (".go", "go"),
    (".cs", "csharp"),
    (".rs", "rust"),
    (".rb", "ruby"),
    (".sh", "shell"),
    (".bash", "shell"),
    (".zsh", "shell"),
    (".ps1", "shell"),
    (".sql", "sql"),
    (".yaml", "yaml"),
    (".yml", "yaml"),
    (".toml", "toml"),
    (".css", "css"),
    (".svg", "xml"),
    (".xml", "xml"),
];

/// 予約語。
const KEYWORDS: [(&str, &str); 13] = [
    (
        "python",
        "False None True and as assert async await break class continue def del elif \
      else except finally for from global if import in is lambda nonlocal not or \
      pass raise return try while with yield match case",
    ),
    (
        "js",
        "as async await break case catch class const continue debugger default delete do \
      else export extends false finally for from function if import in instanceof let \
      new null of return static super switch this throw true try typeof var void while \
      with yield interface type enum implements readonly",
    ),
    (
        "go",
        "break case chan const continue default defer else fallthrough for func go goto if \
      import interface map package range return select struct switch type var nil true false",
    ),
    (
        "csharp",
        "abstract as async await base bool break case catch class const continue default \
      delegate do else enum false finally for foreach get if in init interface internal is \
      new null out override params private protected public readonly record ref return \
      sealed set static string struct switch this throw true try using var void while yield",
    ),
    (
        "rust",
        "as async await break const continue crate dyn else enum extern false fn for if \
      impl in let loop match mod move mut pub ref return self static struct super \
      trait true type unsafe use where while",
    ),
    (
        "ruby",
        "def end class module if elsif else unless while until for in do then begin rescue \
      ensure yield return self nil true false and or not require",
    ),
    (
        "shell",
        "if then elif else fi for while until do done case esac function return local \
      export readonly set unset echo exit source",
    ),
    (
        "sql",
        "select from where group by having order limit offset insert into values update \
      set delete create table drop alter index join left right inner outer on as and \
      or not null distinct union all",
    ),
    ("json", "true false null"),
    ("yaml", "true false null yes no"),
    ("toml", "true false"),
    ("css", ""),
    ("xml", ""),
];

/// 行の中を色付けする規則。**順に適用し、先に一致したものが優先する。**
const LINE: [(&str, &[(&str, &str)]); 13] = [
    (
        "python",
        &[
            (r"#.*$", "c"),
            (
                r"(?s)(?:'''|\x22\x22\x22).*?(?:'''|\x22\x22\x22)|'[^']*'|\x22[^\x22]*\x22",
                "s",
            ),
        ],
    ),
    (
        "js",
        &[
            (r"//.*$", "c"),
            (r"(?s)/\*.*?\*/", "c"),
            (r"'[^']*'|\x22[^\x22]*\x22|`[^`]*`", "s"),
        ],
    ),
    (
        "go",
        &[(r"//.*$", "c"), (r"'[^']*'|\x22[^\x22]*\x22|`[^`]*`", "s")],
    ),
    (
        "csharp",
        &[
            (r"//.*$", "c"),
            (r"(?s)/\*.*?\*/", "c"),
            (r"'[^']*'|\x22[^\x22]*\x22", "s"),
        ],
    ),
    (
        "rust",
        &[(r"//.*$", "c"), (r"'[^']*'|\x22[^\x22]*\x22", "s")],
    ),
    (
        "ruby",
        &[(r"#.*$", "c"), (r"'[^']*'|\x22[^\x22]*\x22", "s")],
    ),
    (
        "shell",
        &[(r"#.*$", "c"), (r"'[^']*'|\x22[^\x22]*\x22", "s")],
    ),
    ("sql", &[(r"--.*$", "c"), (r"'[^']*'", "s")]),
    ("json", &[(r"\x22[^\x22]*\x22", "s")]),
    (
        "yaml",
        &[(r"#.*$", "c"), (r"'[^']*'|\x22[^\x22]*\x22", "s")],
    ),
    (
        "toml",
        &[(r"#.*$", "c"), (r"'[^']*'|\x22[^\x22]*\x22", "s")],
    ),
    (
        "css",
        &[(r"(?s)/\*.*?\*/", "c"), (r"'[^']*'|\x22[^\x22]*\x22", "s")],
    ),
    (
        "xml",
        &[(r"(?s)&lt;!--.*?--&gt;", "c"), (r"\x22[^\x22]*\x22", "s")],
    ),
];

const NUMBER: &str = r"\b\d[\d_]*(?:\.\d+)?\b";

/// その拡張子をコードとして扱うか。
#[must_use]
pub fn is_code(ext: &str) -> bool {
    let lower = ext.to_lowercase();
    LANGS.iter().any(|(k, _)| *k == lower)
}

/// 経路から、言語を決める拡張子を取る。**雛形（`.tmpl`）は、その前の拡張子で決める** ──
/// `install.sh.tmpl` は shell である。前の拡張子が無い雛形は `.tmpl` のまま返す。
#[must_use]
pub fn ext_of(path: &std::path::Path) -> String {
    let dot = |p: &std::path::Path| {
        p.extension()
            .map(|x| format!(".{}", x.to_string_lossy().to_lowercase()))
            .unwrap_or_default()
    };
    let ext = dot(path);
    if ext == ".tmpl" {
        if let Some(stem) = path.file_stem() {
            let inner = dot(std::path::Path::new(stem));
            if !inner.is_empty() {
                return inner;
            }
        }
    }
    ext
}

/// 拡張子から言語を引く。
#[must_use]
pub fn lang_of(ext: &str) -> Option<&'static str> {
    let lower = ext.to_lowercase();
    LANGS
        .iter()
        .find(|(k, _)| *k == lower)
        .map(|(_, lang)| *lang)
}

/// 言語ごとの式を、1度だけ組む。**行ごとに組み直すと、行数だけ組み立てが走る。**
#[derive(Debug)]
pub struct Painter {
    lang: &'static str,
    rules: Vec<(Regex, &'static str)>,
    words: Option<Regex>,
    number: Regex,
}

impl Painter {
    /// 言語の色付けを組む。
    ///
    /// # Errors
    ///
    /// 式として組めないときに返す。
    pub fn new(lang: &'static str) -> Result<Self, String> {
        let rules = LINE
            .iter()
            .find(|(k, _)| *k == lang)
            .map(|(_, rules)| *rules)
            .unwrap_or(&[]);
        let built: Result<Vec<_>, String> = rules
            .iter()
            .map(|(pat, kind)| {
                Regex::new(pat)
                    .map(|r| (r, *kind))
                    .map_err(|e| format!("{lang}: 式として組めない ── {e}"))
            })
            .collect();
        let words = KEYWORDS
            .iter()
            .find(|(k, _)| *k == lang)
            .map(|(_, words)| *words)
            .unwrap_or("");
        let listed: Vec<String> = words.split_whitespace().map(regex::escape).collect();
        let words = if listed.is_empty() {
            None
        } else {
            Some(
                Regex::new(&format!(r"\b(?:{})\b", listed.join("|")))
                    .map_err(|e| format!("{lang}: 予約語の式が組めない ── {e}"))?,
            )
        };
        Ok(Self {
            lang,
            rules: built?,
            words,
            number: Regex::new(NUMBER).map_err(|e| format!("数の式が組めない ── {e}"))?,
        })
    }

    /// 1行を色付けする。**すでに逃がした文字列に対して適用する。**
    ///
    /// # Errors
    ///
    /// 型と噛み合わないときに返す。
    pub fn paint(&self, parts: &Parts, line: &str) -> Result<String, String> {
        let mut spans: Vec<(usize, usize, &str)> = Vec::new();
        let claim = |spans: &mut Vec<(usize, usize, &str)>, a: usize, b: usize, kind| {
            if !spans.iter().any(|(x, y, _)| *x <= a && a < *y) {
                spans.push((a, b, kind));
            }
        };
        for (rule, kind) in &self.rules {
            for m in rule.find_iter(line) {
                claim(&mut spans, m.start(), m.end(), kind);
            }
        }
        if let Some(words) = &self.words {
            for m in words.find_iter(line) {
                claim(&mut spans, m.start(), m.end(), "k");
            }
        }
        for m in self.number.find_iter(line) {
            claim(&mut spans, m.start(), m.end(), "n");
        }
        spans.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(b.2)));
        let mut out = String::new();
        let mut at = 0;
        for (a, b, kind) in spans {
            if a < at {
                continue;
            }
            out.push_str(&line[at..a]);
            out.push_str(&parts.part(
                "token",
                &[
                    ("kind", (*kind).to_owned()),
                    ("body", line[a..b].to_owned()),
                ],
            )?);
            at = b;
        }
        out.push_str(&line[at..]);
        Ok(out)
    }

    /// 言語の名前。
    #[must_use]
    pub const fn lang(&self) -> &'static str {
        self.lang
    }
}

fn text_of(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(|x| x.as_str())
        .unwrap_or_default()
        .to_owned()
}

/// 1行を、色付けした本文へ組む。**空の行は、崩れない空白にする。**
fn painted(painter: &Painter, parts: &Parts, raw: &str) -> Result<String, String> {
    let got = painter.paint(parts, &esc(raw))?;
    Ok(if got.is_empty() {
        "&nbsp;".to_owned()
    } else {
        got
    })
}

/// コードを、行番号を備えたコードブロックへ組む。**印は行の単位で付く。**
///
/// # Errors
///
/// 言語を引けないときと、型と噛み合わないときに返す。
pub fn render_code(parts: &Parts, src: &str, ext: &str, marks: &[Value]) -> Result<String, String> {
    let Some(lang) = lang_of(ext) else {
        return Err(format!("コードとして扱わない拡張子: {ext}"));
    };
    let painter = Painter::new(lang)?;
    let lines: Vec<&str> = src.split('\n').collect();
    // 印を行へ割り当てる。**同じ行に2件は付けない** ── 入れ子になるためである
    let mut at: HashMap<usize, &Value> = HashMap::new();
    for change in marks {
        let find = text_of(change, "find");
        if find.is_empty() {
            continue;
        }
        for (i, raw) in lines.iter().enumerate() {
            if raw.contains(&find) && !at.contains_key(&i) {
                at.insert(i, change);
                break;
            }
        }
    }
    let mut rows = String::new();
    for (i, raw) in lines.iter().enumerate() {
        let mut body = painted(&painter, parts, raw)?;
        if let Some(change) = at.get(&i) {
            body = parts.part(
                "mark",
                &[
                    ("before", esc(&text_of(change, "before"))),
                    ("why", esc(&text_of(change, "why"))),
                    ("body", body),
                ],
            )?;
        }
        rows.push_str(&parts.part("code-row", &[("no", (i + 1).to_string()), ("body", body)])?);
    }
    parts.part("code-block", &[("lang", lang.to_owned()), ("rows", rows)])
}

/// 差分の1行。`mark` は空白（変化なし）・ `-`（旧）・ `+`（新）である。
type Row = (Option<usize>, Option<usize>, char, String);

/// 統合差分のまとまりを組む。
///
/// **全行を先に組み、変化した箇所の周りだけを切り出す** ── 途中で切ると文脈が落ちる。
#[must_use]
pub fn hunks(old: &[&str], new: &[&str], ctx: usize) -> Vec<Vec<Row>> {
    let mut rows: Vec<Row> = Vec::new();
    for (kind, a1, a2, b1, b2) in crate::code::opcodes(old, new) {
        match kind {
            Op::Equal => {
                for k in 0..(a2 - a1) {
                    rows.push((
                        Some(a1 + k + 1),
                        Some(b1 + k + 1),
                        ' ',
                        old[a1 + k].to_owned(),
                    ));
                }
            }
            Op::Replace | Op::Delete | Op::Insert => {
                if matches!(kind, Op::Replace | Op::Delete) {
                    for (k, line) in old.iter().enumerate().take(a2).skip(a1) {
                        rows.push((Some(k + 1), None, '-', (*line).to_owned()));
                    }
                }
                if matches!(kind, Op::Replace | Op::Insert) {
                    for (k, line) in new.iter().enumerate().take(b2).skip(b1) {
                        rows.push((None, Some(k + 1), '+', (*line).to_owned()));
                    }
                }
            }
        }
    }
    let moved: Vec<usize> = rows
        .iter()
        .enumerate()
        .filter(|(_, r)| r.2 != ' ')
        .map(|(i, _)| i)
        .collect();
    if moved.is_empty() {
        return Vec::new();
    }
    let mut groups: Vec<Vec<usize>> = vec![vec![moved[0]]];
    for i in &moved[1..] {
        let last = *groups.last().and_then(|g| g.last()).unwrap_or(&0);
        if i - last <= ctx * 2 {
            groups.last_mut().expect("在る").push(*i);
        } else {
            groups.push(vec![*i]);
        }
    }
    groups
        .iter()
        .map(|g| {
            let a = g[0].saturating_sub(ctx);
            let b = (g[g.len() - 1] + ctx + 1).min(rows.len());
            rows[a..b].to_vec()
        })
        .collect()
}

/// 差分の種類。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    /// 一致している。
    Equal,
    /// 置き換わった。
    Replace,
    /// 消えた。
    Delete,
    /// 増えた。
    Insert,
}

/// 一致する塊を探す。**Ratcliff と Obershelp の方式である。**
fn longest_match(
    a: &[&str],
    b: &[&str],
    b2j: &HashMap<&str, Vec<usize>>,
    alo: usize,
    ahi: usize,
    blo: usize,
    bhi: usize,
) -> (usize, usize, usize) {
    let (mut besti, mut bestj, mut bestsize) = (alo, blo, 0);
    let mut j2len: HashMap<usize, usize> = HashMap::new();
    for (i, line) in a.iter().enumerate().take(ahi).skip(alo) {
        let mut next: HashMap<usize, usize> = HashMap::new();
        if let Some(places) = b2j.get(*line) {
            for j in places {
                if *j < blo {
                    continue;
                }
                if *j >= bhi {
                    break;
                }
                let k = j.checked_sub(1).and_then(|p| j2len.get(&p)).unwrap_or(&0) + 1;
                next.insert(*j, k);
                if k > bestsize {
                    (besti, bestj, bestsize) = (i + 1 - k, j + 1 - k, k);
                }
            }
        }
        j2len = next;
    }
    while besti > alo && bestj > blo && a[besti - 1] == b[bestj - 1] {
        besti -= 1;
        bestj -= 1;
        bestsize += 1;
    }
    while besti + bestsize < ahi
        && bestj + bestsize < bhi
        && a[besti + bestsize] == b[bestj + bestsize]
    {
        bestsize += 1;
    }
    (besti, bestj, bestsize)
}

/// 一致する塊を、前から順に並べる。
fn matching_blocks(a: &[&str], b: &[&str]) -> Vec<(usize, usize, usize)> {
    let mut b2j: HashMap<&str, Vec<usize>> = HashMap::new();
    for (j, line) in b.iter().enumerate() {
        b2j.entry(line).or_default().push(j);
    }
    let mut queue = vec![(0, a.len(), 0, b.len())];
    let mut found = Vec::new();
    while let Some((alo, ahi, blo, bhi)) = queue.pop() {
        let (i, j, k) = longest_match(a, b, &b2j, alo, ahi, blo, bhi);
        if k == 0 {
            continue;
        }
        found.push((i, j, k));
        if alo < i && blo < j {
            queue.push((alo, i, blo, j));
        }
        if i + k < ahi && j + k < bhi {
            queue.push((i + k, ahi, j + k, bhi));
        }
    }
    found.sort_unstable();
    // 隣り合う塊をつなぐ
    let mut joined: Vec<(usize, usize, usize)> = Vec::new();
    let (mut i1, mut j1, mut k1) = (0, 0, 0);
    for (i2, j2, k2) in found {
        if i1 + k1 == i2 && j1 + k1 == j2 {
            k1 += k2;
        } else {
            if k1 > 0 {
                joined.push((i1, j1, k1));
            }
            (i1, j1, k1) = (i2, j2, k2);
        }
    }
    if k1 > 0 {
        joined.push((i1, j1, k1));
    }
    joined.push((a.len(), b.len(), 0));
    joined
}

/// 差分の手順を並べる。
#[must_use]
pub fn opcodes(a: &[&str], b: &[&str]) -> Vec<(Op, usize, usize, usize, usize)> {
    let mut out = Vec::new();
    let (mut i, mut j) = (0, 0);
    for (ai, bj, size) in matching_blocks(a, b) {
        let kind = if i < ai && j < bj {
            Some(Op::Replace)
        } else if i < ai {
            Some(Op::Delete)
        } else if j < bj {
            Some(Op::Insert)
        } else {
            None
        };
        if let Some(kind) = kind {
            out.push((kind, i, ai, j, bj));
        }
        i = ai + size;
        j = bj + size;
        if size > 0 {
            out.push((Op::Equal, ai, i, bj, j));
        }
    }
    out
}

/// 差分を組んだ結果。
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Diff {
    /// 組んだ本文。
    pub body: String,
    /// まとまりの数。
    pub hunks: usize,
    /// 理由が付いたまとまりの数。
    pub explained: usize,
}

/// Git の差分の形へ組み、まとまりごとに理由を添える。
///
/// **Git は差分を出すが、なぜ変えたかを出さない** ── そこを埋めるのがこの道具である。
///
/// # Errors
///
/// 言語を引けないときと、型と噛み合わないときに返す。
pub fn render_diff(
    parts: &Parts,
    old_src: &str,
    new_src: &str,
    ext: &str,
    marks: &[Value],
) -> Result<Diff, String> {
    let Some(lang) = lang_of(ext) else {
        return Err(format!("コードとして扱わない拡張子: {ext}"));
    };
    let painter = Painter::new(lang)?;
    let old: Vec<&str> = old_src.split('\n').collect();
    let new: Vec<&str> = new_src.split('\n').collect();
    let all = hunks(&old, &new, 3);

    // 理由を、まとまりへ割り当てる。**変化した行に一致するものを先に見る**
    let mut used: Vec<usize> = Vec::new();
    let mut at: HashMap<usize, &Value> = HashMap::new();
    for change in marks {
        let find = text_of(change, "find");
        if find.is_empty() {
            continue;
        }
        let added = all.iter().enumerate().find(|(i, hunk)| {
            !used.contains(i)
                && hunk
                    .iter()
                    .any(|(_, _, m, line)| *m == '+' && line.contains(&find))
        });
        let picked = added.or_else(|| {
            all.iter().enumerate().find(|(i, hunk)| {
                !used.contains(i) && hunk.iter().any(|(_, _, _, line)| line.contains(&find))
            })
        });
        if let Some((i, _)) = picked {
            at.insert(i, change);
            used.push(i);
        }
    }

    let mut rows = String::new();
    for (i, hunk) in all.iter().enumerate() {
        let a = hunk.iter().find_map(|(x, _, _, _)| *x);
        let b = hunk.iter().find_map(|(_, y, _, _)| *y);
        let change = at.get(&i);
        let head = parts.part(
            "hunk-head",
            &[
                ("old", a.map_or_else(|| "-".to_owned(), |x| x.to_string())),
                ("new", b.map_or_else(|| "-".to_owned(), |y| y.to_string())),
                (
                    "note",
                    if change.is_some() {
                        String::new()
                    } else {
                        parts.part("hunk-nowhy", &[])?
                    },
                ),
            ],
        )?;
        rows.push_str(&head);
        // **印を置く行を先に決める** ── 探す文字列を含む足した行、無ければ含む行、無ければ最初の消した行。
        // 削除だけのまとまりにも印を付ける（付けないと、理由は在るのに一覧から外れる）
        let target = change.and_then(|c| {
            let find = text_of(c, "find");
            hunk.iter()
                .position(|(_, _, mk, line)| *mk == '+' && line.contains(&find))
                .or_else(|| hunk.iter().position(|(_, _, _, line)| line.contains(&find)))
                .or_else(|| hunk.iter().position(|(_, _, mk, _)| *mk == '-'))
        });
        for (k, (x, y, mk, line)) in hunk.iter().enumerate() {
            let mut body = painted(&painter, parts, line)?;
            if let Some(change) = change {
                if Some(k) == target {
                    let before = change
                        .get("before")
                        .and_then(|x| x.as_str())
                        .filter(|x| !x.is_empty())
                        .unwrap_or("── 上の - の行が変更前である");
                    body = parts.part(
                        "mark",
                        &[
                            ("before", esc(before)),
                            ("why", esc(&text_of(change, "why"))),
                            ("body", body),
                        ],
                    )?;
                }
            }
            let cls = match mk {
                '-' => "d",
                '+' => "a",
                _ => "",
            };
            rows.push_str(&parts.part(
                "diff-row",
                &[
                    ("cls", cls.to_owned()),
                    ("old", x.map_or_else(String::new, |v| v.to_string())),
                    ("new", y.map_or_else(String::new, |v| v.to_string())),
                    ("mk", esc(&mk.to_string())),
                    ("body", body),
                ],
            )?);
        }
    }
    Ok(Diff {
        body: parts.part("diff-block", &[("lang", lang.to_owned()), ("rows", rows)])?,
        hunks: all.len(),
        explained: at.len(),
    })
}
