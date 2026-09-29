// SPDX-License-Identifier: MIT
//! 変更した文書が複数あるとき、タブ1枚にまとめて提示する。
//!
//! **タブの中身は、拡張子で決まる。**
//!
//! | 拡張子 | どう置くか |
//! |---|---|
//! | `.md` | 描画する |
//! | コード | 差分として置く。変更前を取得できなければ全文を置く |
//! | その他 | そのままの見た目で置く。iframe に流し込み、届かなければ Shadow DOM へ切り替える |
//!
//! **どの入れ方でも、印の付け方と開閉は同じである。** 印は `mark.chg` で、押すと変更前と
//! 理由が開く。
//!
//! **HTML の形は、ここが持たない** ── `references/acdr.template.html` が持つ。

use std::path::Path;
use std::process::{Command, Stdio};

use regex::Regex;
use serde_json::Value;

use crate::code;
use crate::markdown::{self, esc};
use crate::style::Style;
use crate::template::Parts;

/// 組み立てに要るもの。**呼ぶ側が渡す** ── この側で置き場所を推測しない。
#[derive(Debug)]
pub struct Shop<'a> {
    /// 型。
    pub parts: &'a Parts,
    /// 見た目と動き。
    pub style: &'a Style,
    /// 変更前を取得する git のコマンド。**名前を直書きしない** ── 宣言の層が tool.json から
    /// 読んで渡す。利用者は tool.json を書き換えるだけで、別の git を使える。
    pub git: &'a str,
    /// 記録のスキーマ。**上部の節の見出しと見せ方は、ここから取る。**
    pub schema: &'a Value,
}

/// 組んだ結果。
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct Made {
    /// 組んだ1枚。
    pub page: String,
    /// 付いた印の数。
    pub total: usize,
    /// 報告へ出す行。**黙って除外しない。**
    pub notes: Vec<String>,
    /// 理由が付いていないまとまり。`(面の鍵, 欠けた数, まとまりの数)`
    pub nowhy: Vec<(String, usize, usize)>,
    /// 面の数のうち、そのままの見た目で置いたもの。
    pub embedded: usize,
}

fn text_of(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(|x| x.as_str())
        .unwrap_or_default()
        .to_owned()
}

fn array_of<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value
        .get(key)
        .and_then(|x| x.as_array())
        .map_or(&[], |x| x.as_slice())
}

/// `<head>` の `<style>` と `<body>` の中身だけを取り出す。
///
/// `<html>` と `<body>` を除去するのは、入れ子にすると親の文書構造が壊れるためである。
/// **`<style>` は除去しない** ── 除去すると、見た目がそのままでなくなる。
#[must_use]
pub fn strip_document(src: &str) -> String {
    let style = Regex::new(r"(?si)<style\b[^>]*>.*?</style>").expect("式である");
    let styles: Vec<&str> = style.find_iter(src).map(|m| m.as_str()).collect();
    let body_tag = Regex::new(r"(?si)<body\b[^>]*>(.*?)</body>").expect("式である");
    let body = if let Some(m) = body_tag.captures(src) {
        m[1].to_owned()
    } else {
        // `<body>` を持たない断片は、`<head>` の中身を外して本文とみなす
        let head = Regex::new(r"(?si)<head\b[^>]*>.*?</head>").expect("式である");
        let shell = Regex::new(r"(?i)</?(?:html|body)\b[^>]*>").expect("式である");
        let mut got = shell
            .replace_all(&head.replace_all(src, ""), "")
            .into_owned();
        for one in &styles {
            got = got.replace(one, "");
        }
        got
    };
    styles.concat() + &body
}

/// Shadow DOM に入れるとき、`:root` と `body` を `:host` へ寄せる。
///
/// **Shadow の中に `:root` は無い。** 寄せないと、そこで定めたカスタムプロパティが
/// 1つも適用されず、色が全部失われる。
#[must_use]
pub fn scope_for_shadow(chunk: &str) -> String {
    let root = Regex::new(r"(?P<a>[^\w-]|^):root(?P<b>[^\w-]|$)").expect("式である");
    let body = Regex::new(r"(?m)^(?P<a>\s*)body(?P<b>\s*[,{])").expect("式である");
    let once = root.replace_all(chunk, "${a}:host${b}").into_owned();
    // 重なった一致は1度で置き換わらない ── 止まるまで当てる
    let mut got = once;
    loop {
        let next = root.replace_all(&got, "${a}:host${b}").into_owned();
        if next == got {
            break;
        }
        got = next;
    }
    body.replace_all(&got, "${a}:host${b}").into_owned()
}

/// 実際に印が付いたものだけを返す。
///
/// **印が付かなかったものを一覧に載せると、押しても運べない。** タブの数字と一覧の
/// 件数も食い違う。**除外したものは、必ず報告する。**
fn landed(body: &str, marks: &[Value]) -> (Vec<Value>, Vec<String>) {
    let mut keep = Vec::new();
    let mut notes = Vec::new();
    for change in marks {
        let key = format!("data-b=\"{}\"", esc(&text_of(change, "before")));
        if body.contains(&key) {
            keep.push(change.clone());
        } else {
            notes.push(format!(
                "一覧から外した（印が付かず）: {}",
                head(&text_of(change, "find"), 50)
            ));
        }
    }
    (keep, notes)
}

fn head(body: &str, count: usize) -> String {
    body.chars().take(count).collect()
}

/// 面の頭に置く、変更の一覧。
///
/// **HTML の面では、印が内側の隠れたタブに入ることがある** ── 実際に4件とも隠れた。
/// 一覧が無ければ、読み手はそこへ辿り着けない。
fn index(parts: &Parts, marks: &[Value]) -> Result<String, String> {
    if marks.is_empty() {
        return Ok(String::new());
    }
    let mut items = String::new();
    for (i, change) in marks.iter().enumerate() {
        items.push_str(&parts.part(
            "index-item",
            &[
                ("find", esc(&head(&text_of(change, "find"), 40))),
                ("at", i.to_string()),
                ("why", esc(&text_of(change, "why"))),
            ],
        )?);
    }
    parts.part(
        "index",
        &[("count", marks.len().to_string()), ("items", items)],
    )
}

/// 変更前の中身を取得する。**記録へ複製しない** ── git から取る。
///
/// 記録が `before` を渡していればそれを使う。無ければ `rev`（既定は `HEAD`）の版から取る。
/// **欄の名前は契約（スキーマ）どおりである** ── 以前は日本語の名前（変更前 ・ 基準）で読んで
/// いて、渡した変更前が使われなかった。git は、この Skill の目的に不可欠な外部の道具である
/// （tool.json の external）。**コマンドは引数で受け取る。** 取得できなければ `None` を返し、
/// 呼ぶ側が全文を置く。
#[must_use]
pub fn before_of(doc: &Value, git: &str) -> Option<String> {
    if let Some(given) = doc.get("before").and_then(|x| x.as_str()) {
        return Some(given.to_owned()); // 直に渡された場合はそれを使う
    }
    let rev = doc.get("rev").and_then(|x| x.as_str()).unwrap_or("HEAD");
    let path = std::fs::canonicalize(text_of(doc, "file")).ok()?;
    let root = path.ancestors().skip(1).find(|d| d.join(".git").exists())?;
    let rel = path.strip_prefix(root).ok()?;
    // 版の中の経路は、OS を問わず / で区切る
    let rel = rel
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/");
    let done = Command::new(git)
        .arg("-C")
        .arg(root)
        .arg("show")
        .arg(format!("{rev}:{rel}"))
        .stdin(Stdio::null())
        .output()
        .ok()?;
    if !done.status.success() {
        return None;
    }
    String::from_utf8(done.stdout).ok()
}

/// 面1つを組む。
fn pane(shop: &Shop, doc: &Value, made: &mut Made) -> Result<(String, usize), String> {
    let parts = shop.parts;
    let file = text_of(doc, "file");
    let path = Path::new(&file);
    let src = std::fs::read_to_string(path).map_err(|e| format!("{file}: 読めない ── {e}"))?;
    let marks = array_of(doc, "marks");
    let ext = code::ext_of(path);
    let key = text_of(doc, "key");
    let built = if code::is_code(&ext) {
        let (body, lane) = match before_of(doc, shop.git) {
            None => (
                code::render_code(parts, &src, &ext, marks)?,
                parts.part(
                    "lane-code",
                    &[("note", "変更前を取得できないので、全文を置く".to_owned())],
                )?,
            ),
            Some(base) => {
                let diff = code::render_diff(parts, &base, &src, &ext, marks)?;
                if diff.hunks == 0 {
                    (
                        code::render_code(parts, &src, &ext, marks)?,
                        parts.part(
                            "lane-code",
                            &[("note", "差分が無いので、全文を置く".to_owned())],
                        )?,
                    )
                } else {
                    let miss = diff.hunks - diff.explained;
                    if miss > 0 {
                        made.nowhy.push((key.clone(), miss, diff.hunks));
                    }
                    let why = if miss > 0 {
                        parts.part("lane-nowhy", &[("count", miss.to_string())])?
                    } else {
                        " ／ 全件に理由が付いている".to_owned()
                    };
                    (
                        diff.body,
                        parts.part(
                            "lane-diff",
                            &[("count", diff.hunks.to_string()), ("why", why)],
                        )?,
                    )
                }
            }
        };
        let (kept, notes) = landed(&body, marks);
        made.notes.extend(notes);
        parts.part(
            "pane-code",
            &[
                ("key", key),
                ("lane", lane),
                ("index", index(parts, &kept)?),
                ("body", body),
            ],
        )?
    } else if ext == ".md" {
        let drawn = markdown::render(parts, &src)?;
        let marked = markdown::mark(parts, &drawn, marks)?;
        made.notes.extend(marked.dropped.clone());
        made.notes
            .push(format!("{}/{} 件に印を付けた", marked.kept, marked.asked));
        let (kept, notes) = landed(&marked.body, marks);
        made.notes.extend(notes);
        parts.part(
            "pane-md",
            &[
                ("key", key),
                ("index", index(parts, &kept)?),
                ("body", marked.body),
            ],
        )?
    } else {
        let marked = markdown::mark(parts, &strip_document(&src), marks)?;
        made.notes.extend(marked.dropped.clone());
        made.notes
            .push(format!("{}/{} 件に印を付けた", marked.kept, marked.asked));
        let (kept, notes) = landed(&marked.body, marks);
        made.notes.extend(notes);
        let style = Regex::new(r"(?si)<style\b[^>]*>(.*?)</style>").expect("式である");
        let inner: String = style
            .captures_iter(&marked.body)
            .map(|c| c[1].to_owned())
            .collect();
        made.embedded += 1;
        parts.part(
            "pane-html",
            &[
                ("key", key),
                ("tab", esc(&text_of(doc, "tab"))),
                ("index", index(parts, &kept)?),
                ("shadowcss", esc(&scope_for_shadow(&inner))),
                ("body", marked.body),
            ],
        )?
    };
    let count = built.matches("mark class=\"chg\"").count();
    Ok((built, count))
}

/// 1枚へ組む。
///
/// 見出しと、足す CSS は、呼ぶ側が差し替えられる ── **組んだあとの文字列を置換して
/// 差し込むと、置換の当て先が変わったときに黙って外れる。**
///
/// # Errors
///
/// 対象の文書を読めないときと、型と噛み合わないときに返す。
pub fn build(shop: &Shop, spec: &Value, heading: &str, extra_css: &str) -> Result<Made, String> {
    let parts = shop.parts;
    let mut made = Made::default();
    let mut tabs = String::new();
    let mut panes = String::new();
    for doc in array_of(spec, "docs") {
        let (built, count) = pane(shop, doc, &mut made)?;
        made.total += count;
        panes.push_str(&built);
        tabs.push_str(&parts.part(
            "tab",
            &[
                ("key", text_of(doc, "key")),
                ("label", esc(&text_of(doc, "tab"))),
                ("count", count.to_string()),
            ],
        )?);
    }
    let title = esc(&text_of(spec, "title"));
    let heading = if heading.is_empty() {
        parts.part(
            "md-heading",
            &[("level", "1".to_owned()), ("body", title.clone())],
        )?
    } else {
        heading.to_owned()
    };
    let style = &shop.style;
    made.page = parts.part(
        "page",
        &[
            ("title", title),
            ("heading", heading),
            (
                "style",
                format!("{}{}{}{extra_css}", style.tokens, style.base, style.code),
            ),
            ("markcss", style.mark.clone()),
            ("embed", format!("{}{}", style.tokens_embed, style.mark)),
            ("lede", text_of(spec, "lede")),
            ("intro", text_of(spec, "intro")),
            ("total", made.total.to_string()),
            ("tabs", tabs),
            ("panes", panes),
            ("js", style.js.clone()),
        ],
    )?;
    Ok(made)
}

/// 作ったあと、開閉が成立する形かを検査する。
///
/// 見るのは8つ。**どれも、以前に実際にやらかしたものである。**
#[must_use]
pub fn check(out: &str, spec: &Value, made: &Made) -> Vec<(String, bool)> {
    let mut ok = Vec::new();
    ok.push(("<script> が在る".to_owned(), out.contains("<script>")));
    ok.push((
        ".pop[hidden] が在る".to_owned(),
        out.contains(".pop[hidden]{display:none!important}"),
    ));
    let count = |pat: &str| -> usize {
        Regex::new(pat)
            .map(|r| r.find_iter(out).count())
            .unwrap_or(0)
    };
    let nb = count(r#"<mark class="chg"[^>]*data-b=""#);
    let nw = count(r#"<mark class="chg"[^>]*data-w=""#);
    ok.push((
        format!("印ごとに data-b と data-w（{nb}／{nw}）"),
        nb == nw && nb == made.total,
    ));
    ok.push((
        "表の中に <div> が無い".to_owned(),
        !out.contains("<tbody><div") && !out.contains("<tr><div"),
    ));
    let nt = count(r#"<button class="tab" role="tab""#);
    let np = count(r#"<section class="pane (?:md|html|code-pane)""#);
    let docs = array_of(spec, "docs").len();
    ok.push((
        format!("タブとパネルの数が合う（{nt}／{np}）"),
        nt == np && nt == docs,
    ));
    // 面の中身は対象の文書である ── 波括弧が釣り合う保証は無い。数えるのは枠だけにする
    let frame = out.split("<script>").next().unwrap_or_default();
    let pane = Regex::new(r#"(?s)<section class="pane.*?</section>"#).expect("式である");
    let body = pane.replace_all(frame, "");
    let (open, shut) = (body.matches('{').count(), body.matches('}').count());
    ok.push((
        format!("波括弧が閉じている（{open}／{shut}）"),
        open == shut,
    ));
    if made.nowhy.is_empty() {
        ok.push(("差分のまとまりに、全件理由が付いている".to_owned(), true));
    } else {
        for (key, miss, all) in &made.nowhy {
            ok.push((
                format!("{key}: まとまり {all} 件のうち {miss} 件に理由が付いていない"),
                false,
            ));
        }
    }
    ok.push((
        format!(
            "HTML の面に雛形が在る（{}／{}）",
            out.matches("<template").count(),
            made.embedded
        ),
        out.matches("<template").count() == made.embedded,
    ));
    ok
}
