// SPDX-License-Identifier: MIT
//! 値の出どころを検査する ── **生の数値がコードに残っていないかを機械が探す。**
//!
//! 幾何検査は「重なっているか」しか見ない。生の数値で置いた寸法でも、たまたま重ならなければ
//! 通る ── 実測 ── 量系の部品だけで9か所の生の値が見逃されたまま残っていた。
//!
//! コードに現れる数値は3つに分かれる:
//!
//! 1. 設計上の選択（余白 ・ 線幅 ・ 書体の大きさ） → トークンから注入する
//! 2. データから決まる量（文字幅 ・ 箱の大きさ） → 毎回計算する
//! 3. 勘で置いた閾値 → **存在してはいけない**
//!
//! この検査は3を探す。**現れた数値は原則すべて疑わしい**という立場を取る。疑わないのは、構造 ・
//! 数学 ・ 仕様が決めているか、既に名前を持っているものだけである:
//!
//! - 0 ・ 1 ・ 2 ・ 半分（0.5）・ 角度（90 ・ 180 ・ 270 ・ 360）・ 割合（100）
//! - 添字（`xs[3]`）と、個数を検査する比較（`len() >= 3`）
//! - 冪の指数と、冪を含む式の整数の係数（ベジェ曲線の 3 が典型）
//! - 数値誤差の許容値（絶対値が 1e-6 未満）
//! - 名前に束ねた数（`const` ・ `static`）── 名前が付いた時点で、それは宣言である
//! - 対応表（`const` の表）の中身
//!
//! **誤検出する検査は無いより悪い** ── 狼少年になって本物を見逃す。だから除外は規則として書ける
//! 形にする。
//!
//! **読むのは Rust のソースである。** 移す前は Python の構文木を読んでいた ── 規則は同じで、
//! 読む言語だけが変わった。

use std::path::Path;

use crate::data_access::files;

/// 構造上どうしても現れる数。
const STRUCTURAL: [f64; 7] = [0.0, 1.0, 2.0, -1.0, -2.0, 0.5, -0.5];
/// 角度と割合。**座標系そのものが決めている数。**
const GEOMETRIC: [f64; 5] = [90.0, 180.0, 270.0, 360.0, 100.0];
/// 数値誤差の許容値と見なす上限。
const TOLERANCE: f64 = 1e-6;

/// 見ないファイル。**値が書かれている場所そのもの**と、読み書きの道具。
const SKIP: [&str; 4] = ["lint.rs", "theme.rs", "lib.rs", "py.rs"];

/// 見つけた1件。`(ファイル, 行, 関数, その行)`
pub type Finding = (String, usize, String, String);

/// 文字の種類を読み飛ばしながら、数の字句を拾う。
#[derive(Debug, Clone)]
struct Tok {
    text: String,
    line: usize,
    kind: Kind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Num,
    Ident,
    Punct,
    /// 文字列か文字。**中身は捨て、在ったことだけを残す。**
    Lit,
}

/// ソースを字句へ割る。**注釈 ・ 文字列 ・ 文字は捨てる。**
fn lex(src: &str) -> Vec<Tok> {
    let chars: Vec<char> = src.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    let mut line = 1;
    while i < chars.len() {
        let c = chars[i];
        if c == '\n' {
            line += 1;
            i += 1;
            continue;
        }
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        // 注釈
        if c == '/' && chars.get(i + 1) == Some(&'/') {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }
        if c == '/' && chars.get(i + 1) == Some(&'*') {
            i += 2;
            while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') {
                if chars[i] == '\n' {
                    line += 1;
                }
                i += 1;
            }
            i += 2;
            continue;
        }
        // 生の文字列 r"..." ・ r#"..."#
        if c == 'r' && (chars.get(i + 1) == Some(&'"') || chars.get(i + 1) == Some(&'#')) {
            let mut j = i + 1;
            let mut hashes = 0;
            while chars.get(j) == Some(&'#') {
                hashes += 1;
                j += 1;
            }
            if chars.get(j) == Some(&'"') {
                j += 1;
                loop {
                    if j >= chars.len() {
                        break;
                    }
                    if chars[j] == '\n' {
                        line += 1;
                    }
                    if chars[j] == '"' && (0..hashes).all(|h| chars.get(j + 1 + h) == Some(&'#')) {
                        j += 1 + hashes;
                        break;
                    }
                    j += 1;
                }
                out.push(Tok {
                    text: "\"".to_owned(),
                    line,
                    kind: Kind::Lit,
                });
                i = j;
                continue;
            }
        }
        // 文字列
        if c == '"' {
            out.push(Tok {
                text: "\"".to_owned(),
                line,
                kind: Kind::Lit,
            });
            i += 1;
            while i < chars.len() && chars[i] != '"' {
                if chars[i] == '\\' {
                    i += 1;
                }
                if chars.get(i) == Some(&'\n') {
                    line += 1;
                }
                i += 1;
            }
            i += 1;
            continue;
        }
        // 文字 'x' と、生存期間 'a を分ける
        if c == '\'' {
            if chars.get(i + 2) == Some(&'\'') || (chars.get(i + 1) == Some(&'\\')) {
                out.push(Tok {
                    text: "'".to_owned(),
                    line,
                    kind: Kind::Lit,
                });
                i += 1;
                while i < chars.len() && chars[i] != '\'' {
                    if chars[i] == '\\' {
                        i += 1;
                    }
                    i += 1;
                }
                i += 1;
                continue;
            }
            i += 1;
            continue;
        }
        if c.is_ascii_digit() {
            let start = i;
            while i < chars.len()
                && (chars[i].is_ascii_alphanumeric() || chars[i] == '_' || chars[i] == '.')
            {
                // 範囲の `..` と、呼び出しの `.name` は数に含めない
                if chars[i] == '.'
                    && (chars.get(i + 1) == Some(&'.')
                        || chars.get(i + 1).is_some_and(|d| d.is_alphabetic()))
                {
                    break;
                }
                if (chars[i] == 'e' || chars[i] == 'E')
                    && matches!(chars.get(i + 1), Some('-' | '+'))
                {
                    i += 2;
                    continue;
                }
                i += 1;
            }
            out.push(Tok {
                text: chars[start..i].iter().collect(),
                line,
                kind: Kind::Num,
            });
            continue;
        }
        if c.is_alphabetic() || c == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            out.push(Tok {
                text: chars[start..i].iter().collect(),
                line,
                kind: Kind::Ident,
            });
            continue;
        }
        out.push(Tok {
            text: c.to_string(),
            line,
            kind: Kind::Punct,
        });
        i += 1;
    }
    out
}

/// 数の字句を値へ読む。**型の後置きと桁区切りを除く。**
fn value_of(text: &str) -> Option<f64> {
    let body: String = text.chars().filter(|c| *c != '_').collect();
    let body = [
        "f64", "f32", "i64", "i32", "u64", "u32", "usize", "isize", "u8",
    ]
    .iter()
    .find_map(|s| body.strip_suffix(s))
    .map_or(body.as_str(), |b| b)
    .to_owned();
    if let Some(hex) = body.strip_prefix("0x") {
        return i64::from_str_radix(hex, 16).ok().map(|v| v as f64);
    }
    body.parse().ok()
}

fn exempt(v: f64) -> bool {
    STRUCTURAL.contains(&v) || GEOMETRIC.contains(&v) || (v != 0.0 && v.abs() < TOLERANCE)
}

/// 1つのファイルから、生の数値を探す。
#[must_use]
pub fn scan(name: &str, src: &str) -> Vec<Finding> {
    let toks = lex(src);
    let lines: Vec<&str> = src.lines().collect();
    let mut out = Vec::new();
    let mut func = "(直下)".to_owned();
    let mut depth: i64 = 0; // 括弧（丸 ・ 角 ・ 波）の深さ
    let mut skip_until_semi: Option<i64> = None; // 名前に束ねた数の宣言
    let mut i = 0;
    while i < toks.len() {
        let t = &toks[i];
        let prev = |k: usize| if i >= k { Some(&toks[i - k]) } else { None };
        match (t.kind, t.text.as_str()) {
            (Kind::Punct, "#") if toks.get(i + 1).is_some_and(|n| n.text == "[") => {
                let mut attr_depth = 1;
                i += 2;
                while i < toks.len() && attr_depth > 0 {
                    match toks[i].text.as_str() {
                        "[" => attr_depth += 1,
                        "]" => attr_depth -= 1,
                        _ => {}
                    }
                    i += 1;
                }
                continue;
            }
            (Kind::Ident, "const" | "static") if prev(1).is_none_or(|p| p.text != "&") => {
                // `const fn` は宣言ではない
                if toks.get(i + 1).is_some_and(|n| n.text == "fn") {
                    i += 1;
                    continue;
                }
                skip_until_semi = Some(depth);
            }
            (Kind::Ident, "fn") => {
                if let Some(n) = toks.get(i + 1) {
                    func = n.text.clone();
                }
            }
            (Kind::Punct, "{" | "(" | "[") => depth += 1,
            (Kind::Punct, "}" | ")" | "]") => depth -= 1,
            (Kind::Punct, ";") => {
                if skip_until_semi == Some(depth) {
                    skip_until_semi = None;
                }
            }
            (Kind::Num, _) if skip_until_semi.is_none() => {
                let Some(v) = value_of(&t.text) else {
                    i += 1;
                    continue;
                };
                let negative = prev(1).is_some_and(|p| p.text == "-")
                    && prev(2)
                        .is_some_and(|p| p.kind == Kind::Punct && p.text != ")" && p.text != "]");
                let v = if negative { -v } else { v };
                if exempt(v) || context_exempt(&toks, i) {
                    i += 1;
                    continue;
                }
                let text = lines.get(t.line - 1).map_or("", |l| l.trim()).to_owned();
                out.push((name.to_owned(), t.line, func.clone(), text));
            }
            _ => {}
        }
        i += 1;
    }
    out
}

/// その数が、構造 ・ 数学 ・ 仕様の側の数か。
fn context_exempt(toks: &[Tok], i: usize) -> bool {
    let prev = |k: usize| {
        if i >= k {
            toks.get(i - k).map(|t| t.text.as_str())
        } else {
            None
        }
    };
    let next = |k: usize| toks.get(i + k).map(|t| t.text.as_str());
    // 添字 xs[3]
    if prev(1) == Some("[")
        && next(1) == Some("]")
        && prev(2).is_some_and(|p| p != "=" && p != "(" && p != ",")
    {
        return true;
    }
    // 組の欄の番号 a.3
    if prev(1) == Some(".") {
        return true;
    }
    // 対応表 ── 型が値だけの腕 `'C' => 6` ・ `"A" | "B" => 4`
    if prev(1) == Some(">") && prev(2) == Some("=") {
        let mut k = i.saturating_sub(3);
        let mut literal_only = true;
        loop {
            let t = &toks[k];
            if matches!(t.text.as_str(), "," | "{") {
                break;
            }
            if !(t.kind == Kind::Lit || t.text == "|") {
                literal_only = false;
                break;
            }
            if k == 0 {
                break;
            }
            k -= 1;
        }
        if literal_only {
            return true;
        }
    }
    // 冪の指数 .powf(3.0) ・ .powi(3)
    if prev(1) == Some("(") && matches!(prev(2), Some("powf" | "powi")) {
        return true;
    }
    // 個数を検査する比較 ── 同じ文の中に len() が在る
    let stmt = statement(toks, i);
    let has = |w: &str| stmt.iter().any(|t| t.text == w);
    if has("len")
        && stmt
            .iter()
            .any(|t| matches!(t.text.as_str(), "<" | ">" | "="))
    {
        return true;
    }
    // 冪を含む式の整数の係数 ── **整数でない小数はなお疑う**
    if (has("powf") || has("powi")) && value_of(&toks[i].text).is_some_and(|v| v.fract() == 0.0) {
        return true;
    }
    false
}

/// その字句を含む文（`;` ・ `{` ・ `}` で区切った範囲）。
fn statement(toks: &[Tok], i: usize) -> &[Tok] {
    let bound = |t: &Tok| matches!(t.text.as_str(), ";" | "{" | "}");
    let start = (0..i).rev().find(|k| bound(&toks[*k])).map_or(0, |k| k + 1);
    let end = (i..toks.len())
        .find(|k| bound(&toks[*k]))
        .unwrap_or(toks.len());
    &toks[start..end]
}

/// 置き場所の中の各ファイルから、生の数値を探す。**行の昇順。**
///
/// # Errors
///
/// 置き場所が無いときと、読めないときに返す。
pub fn findings(root: &Path) -> Result<Vec<Finding>, String> {
    if !files::exists(root) {
        // **無い場所を検査して「0 箇所」と返さない。** 呼ぶ側は合格と受け取る
        return Err(format!("検査する場所が無い: {}", root.display()));
    }
    // **名前の順に並べて受ける** ── 並びを OS に任せると、同じ置き場所から別の順が出る
    let paths: Vec<_> = files::list(root)
        .map_err(|e| format!("{}: 読めない ── {e}", root.display()))?
        .into_iter()
        .filter(|p| p.extension().is_some_and(|x| x == "rs"))
        .collect();
    let mut out = Vec::new();
    for path in paths {
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        if SKIP.contains(&name.as_str()) {
            continue;
        }
        let src = files::read_to_string(&path).map_err(|e| format!("{name}: 読めない ── {e}"))?;
        out.extend(scan(&name, &src));
    }
    Ok(out)
}
