//! Mustache の処理系（mustache/spec の必須の部分：comments ・ delimiters ・ interpolation ・ inverted ・ partials ・
//! sections）。任意の部分（lambdas ・ inheritance ・ dynamic-names）は持たない。
//! 仕様の試験（tests/mustache-spec/*.json）をそのまま通す。
//!
//! 1. 字句：区切りの変更を追いながら、文字列とタグに分ける
//! 2. 単独の行：セクション ・ 反転 ・ 閉じ ・ コメント ・ 部分 ・ 区切りの変更のタグだけの行は、行ごと消す
//!    （部分は、その行の前の空白を部分テンプレートの各行に付ける）
//! 3. 木：セクションを入れ子にする
//! 4. 描く：文脈の積み重ねから名前を引く。値は HTML としてエスケープする（& " < >）

use serde_json::Value;
use std::collections::BTreeMap;

/// テンプレートの木の節。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    /// そのままの文字列
    Text(String),
    /// 値を置く。raw は {{{ }}} か {{& }}（エスケープしない）
    Var {
        /// 名前（. か、点でつないだ名前）
        name: String,
        /// エスケープしないか
        raw: bool,
    },
    /// セクション（inverted は {{^ }}）
    Section {
        /// 名前
        name: String,
        /// 反転か
        inverted: bool,
        /// 中身
        children: Vec<Self>,
    },
    /// 部分テンプレート（indent は単独の行のときの前の空白）
    Partial {
        /// 部分テンプレートの名前
        name: String,
        /// 各行に付ける空白
        indent: String,
    },
}

#[derive(Debug, Clone)]
enum Tok {
    Text(String),
    Tag { kind: char, name: String },
}

fn is_standalone_kind(kind: char) -> bool {
    matches!(kind, '#' | '^' | '/' | '!' | '>' | '=')
}

/// 字句に分ける（区切りの変更を含む）。
fn tokenize(src: &str) -> Result<Vec<Tok>, String> {
    let mut open = "{{".to_string();
    let mut close = "}}".to_string();
    let mut out = Vec::new();
    let mut rest = src;
    while !rest.is_empty() {
        let Some(at) = rest.find(&open) else {
            out.push(Tok::Text(rest.to_string()));
            break;
        };
        if at > 0 {
            out.push(Tok::Text(rest[..at].to_string()));
        }
        let after = &rest[at + open.len()..];
        // {{{ name }}}（区切りが既定のときの三重）
        if open == "{{"
            && let Some(inner) = after.strip_prefix('{')
        {
            let end = inner
                .find("}}}")
                .ok_or_else(|| "閉じていない {{{".to_string())?;
            out.push(Tok::Tag {
                kind: '&',
                name: inner[..end].trim().to_string(),
            });
            rest = &inner[end + 3..];
            continue;
        }
        let (kind, body) = match after.chars().next() {
            Some(c @ ('#' | '^' | '/' | '!' | '>' | '&')) => (c, &after[c.len_utf8()..]),
            Some('=') => ('=', &after[1..]),
            Some('{') => ('{', &after[1..]),
            _ => (' ', after),
        };
        let end_mark = match kind {
            '=' => format!("={close}"),
            '{' => format!("}}{close}"),
            _ => close.clone(),
        };
        let end = body
            .find(&end_mark)
            .ok_or_else(|| format!("閉じていないタグ（{open}{kind}…）"))?;
        let content = &body[..end];
        rest = &body[end + end_mark.len()..];
        match kind {
            '=' => {
                let parts: Vec<&str> = content.split_whitespace().collect();
                let [o, c] = parts.as_slice() else {
                    return Err(format!("区切りの変更の書き方の誤り：{content}"));
                };
                open = (*o).to_string();
                close = (*c).to_string();
                out.push(Tok::Tag {
                    kind: '=',
                    name: String::new(),
                });
            }
            '{' => out.push(Tok::Tag {
                kind: '&',
                name: content.trim().to_string(),
            }),
            ' ' => out.push(Tok::Tag {
                kind: ' ',
                name: content.trim().to_string(),
            }),
            k => out.push(Tok::Tag {
                kind: k,
                name: content.trim().to_string(),
            }),
        }
    }
    Ok(out)
}

/// 行の区切りで文字列を割る（改行は行の終わりに付けたまま）。
fn split_lines(toks: Vec<Tok>) -> Vec<Vec<Tok>> {
    let mut lines: Vec<Vec<Tok>> = vec![Vec::new()];
    for t in toks {
        match t {
            Tok::Text(s) => {
                let mut start = 0;
                for (i, ch) in s.char_indices() {
                    if ch == '\n' {
                        if let Some(line) = lines.last_mut() {
                            line.push(Tok::Text(s[start..=i].to_string()));
                        }
                        lines.push(Vec::new());
                        start = i + 1;
                    }
                }
                if start < s.len()
                    && let Some(line) = lines.last_mut()
                {
                    line.push(Tok::Text(s[start..].to_string()));
                }
            }
            tag => {
                if let Some(line) = lines.last_mut() {
                    line.push(tag);
                }
            }
        }
    }
    lines
}

/// 単独の行を消し、部分テンプレートの字下げを決めて、平らな並びに戻す。
fn strip_standalone(lines: Vec<Vec<Tok>>) -> Vec<(Tok, String)> {
    let mut out = Vec::new();
    for line in lines {
        let tags: Vec<&Tok> = line
            .iter()
            .filter(|t| matches!(t, Tok::Tag { .. }))
            .collect();
        let only_space = line.iter().all(|t| match t {
            Tok::Text(s) => s
                .chars()
                .all(|c| c == ' ' || c == '\t' || c == '\r' || c == '\n'),
            Tok::Tag { .. } => true,
        });
        let standalone = only_space
            && tags.len() == 1
            && matches!(tags[0], Tok::Tag { kind, .. } if is_standalone_kind(*kind));
        if standalone {
            let indent: String = match line.first() {
                Some(Tok::Text(s)) => s.chars().take_while(|c| *c == ' ' || *c == '\t').collect(),
                _ => String::new(),
            };
            for t in line {
                if let Tok::Tag { .. } = t {
                    out.push((t, indent.clone()));
                }
            }
        } else {
            for t in line {
                out.push((t, String::new()));
            }
        }
    }
    out
}

/// テンプレートを木にする。
///
/// # Errors
/// 閉じていないタグ ・ セクションの閉じの食い違い ・ 区切りの変更の誤りのとき。
pub fn parse(src: &str) -> Result<Vec<Node>, String> {
    let toks = strip_standalone(split_lines(tokenize(src)?));
    let mut stack: Vec<(String, bool, Vec<Node>)> = vec![(String::new(), false, Vec::new())];
    for (t, indent) in toks {
        let top = &mut stack.last_mut().ok_or("壊れた入れ子")?.2;
        match t {
            Tok::Text(s) => match top.last_mut() {
                Some(Node::Text(prev)) => prev.push_str(&s),
                _ => top.push(Node::Text(s)),
            },
            Tok::Tag { kind, name } => match kind {
                '!' | '=' => {}
                ' ' => top.push(Node::Var { name, raw: false }),
                '&' => top.push(Node::Var { name, raw: true }),
                '>' => top.push(Node::Partial { name, indent }),
                '#' | '^' => stack.push((name, kind == '^', Vec::new())),
                '/' => {
                    let (open, inverted, children) = stack
                        .pop()
                        .filter(|_| !stack.is_empty())
                        .ok_or_else(|| format!("開いていないセクションの閉じ：{name}"))?;
                    if open != name {
                        return Err(format!("セクション {open} を {name} で閉じている"));
                    }
                    stack
                        .last_mut()
                        .ok_or("壊れた入れ子")?
                        .2
                        .push(Node::Section {
                            name,
                            inverted,
                            children,
                        });
                }
                _ => {}
            },
        }
    }
    if stack.len() != 1 {
        return Err(format!(
            "閉じていないセクション：{}",
            stack.last().map(|s| s.0.as_str()).unwrap_or_default()
        ));
    }
    Ok(stack.pop().map(|s| s.2).unwrap_or_default())
}

/// 文脈の積み重ねから名前を引く。点でつないだ名前は、最初の部分が在る文脈から先をたどる。
fn lookup<'a>(stack: &[&'a Value], name: &str) -> Option<&'a Value> {
    if name == "." {
        return stack.last().copied();
    }
    let mut parts = name.split('.');
    let first = parts.next()?;
    let mut cur = stack
        .iter()
        .rev()
        .find_map(|v| v.as_object().and_then(|m| m.get(first)))?;
    for p in parts {
        cur = cur.as_object()?.get(p)?;
    }
    Some(cur)
}

fn falsey(v: Option<&Value>) -> bool {
    match v {
        None | Some(Value::Null | Value::Bool(false)) => true,
        Some(Value::Array(a)) => a.is_empty(),
        _ => false,
    }
}

/// 値を文字にする（文字列はそのまま、数は JSON の数の書き方、真偽は true ・ false、null と無い名前は空）。
#[must_use]
pub fn to_text(v: Option<&Value>) -> String {
    match v {
        None | Some(Value::Null) => String::new(),
        Some(Value::String(s)) => s.clone(),
        Some(other) => other.to_string(),
    }
}

/// HTML のエスケープ（仕様の4文字）。
#[must_use]
pub fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            c => out.push(c),
        }
    }
    out
}

/// 部分テンプレートの各行の先頭に字下げを付ける（最後の改行の後には付けない）。
fn indent_lines(src: &str, indent: &str) -> String {
    if indent.is_empty() {
        return src.to_string();
    }
    let mut out = String::new();
    let mut at_start = true;
    for ch in src.chars() {
        if at_start {
            out.push_str(indent);
            at_start = false;
        }
        out.push(ch);
        if ch == '\n' {
            at_start = true;
        }
    }
    out
}

/// 部分テンプレートを読み込む深さの上限（自分を読み込み続けるテンプレートを止める）。
const MAX_DEPTH: usize = 64;

/// 描く。partials は名前 → 本文。無い名前と無い部分テンプレートは空で描く（仕様）。
///
/// # Errors
/// 構文の誤り ・ 部分テンプレートの読み込みが深すぎるとき。
pub fn render(
    nodes: &[Node],
    data: &Value,
    partials: &BTreeMap<String, String>,
) -> Result<String, String> {
    let mut out = String::new();
    let mut stack = vec![data];
    walk(nodes, &mut stack, partials, &mut out, 0)?;
    Ok(out)
}

fn walk(
    nodes: &[Node],
    stack: &mut Vec<&Value>,
    partials: &BTreeMap<String, String>,
    out: &mut String,
    depth: usize,
) -> Result<(), String> {
    for n in nodes {
        match n {
            Node::Text(s) => out.push_str(s),
            Node::Var { name, raw } => {
                let text = to_text(lookup(stack, name));
                out.push_str(&if *raw { text } else { escape_html(&text) });
            }
            Node::Section {
                name,
                inverted,
                children,
            } => {
                let v = lookup(stack, name);
                if *inverted {
                    if falsey(v) {
                        walk(children, stack, partials, out, depth)?;
                    }
                    continue;
                }
                if falsey(v) {
                    continue;
                }
                match v {
                    Some(Value::Array(items)) => {
                        for item in items {
                            stack.push(item);
                            walk(children, stack, partials, out, depth)?;
                            stack.pop();
                        }
                    }
                    Some(v) => {
                        stack.push(v);
                        walk(children, stack, partials, out, depth)?;
                        stack.pop();
                    }
                    None => {}
                }
            }
            Node::Partial { name, indent } => {
                if depth >= MAX_DEPTH {
                    return Err(format!("部分テンプレート {name} の読み込みが深すぎる"));
                }
                if let Some(src) = partials.get(name) {
                    let tree = parse(&indent_lines(src, indent))?;
                    walk(&tree, stack, partials, out, depth + 1)?;
                }
            }
        }
    }
    Ok(())
}
