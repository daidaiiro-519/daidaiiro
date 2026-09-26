//! 射影の契約 ── **基盤が持つ。** 雛形の記法は3つで止める。
//!
//! `{{経路}}` 穴 ／ `{{#each 経路}}…{{/each}}` 繰り返し ／ `{{#if 経路}}…{{/if}}` 条件。
//! **値の加工を持たない** ── 持たせると記法が言語に育つ。
use serde_json::Value;
use std::collections::BTreeSet;

/// 雛形が読む経路を、全部並べる ── **「何を読むか」の申告が、雛形から導ける。**
pub fn holes(tpl: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut rest = tpl;
    while let Some(i) = rest.find("{{") {
        let r = &rest[i + 2..];
        let Some(j) = r.find("}}") else { break };
        let k = r[..j].trim();
        let k = k
            .strip_prefix("#each ")
            .or_else(|| k.strip_prefix("#if "))
            .unwrap_or(k);
        if !k.starts_with('/') && !k.is_empty() {
            out.insert(k.to_string());
        }
        rest = &r[j + 2..];
    }
    out
}

fn 解く<'a>(ctx: &'a Value, path: &str) -> Option<&'a Value> {
    if path == "." {
        return Some(ctx);
    }
    path.split('.').try_fold(ctx, |v, k| v.get(k))
}

/// 数える ── 経路の末尾が `.len` なら、その並びの件数を返す。
fn 数える(ctx: &Value, path: &str) -> Option<usize> {
    let base = path.strip_suffix(".len")?;
    let v = 解く(ctx, base)?;
    v.as_array()
        .map(|a| a.len())
        .or_else(|| v.as_object().map(|o| o.len()))
}

fn 文字に(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        _ => v.to_string(),
    }
}

fn 穴を埋める(seg: &str, ctx: &Value) -> Result<String, String> {
    let mut out = String::new();
    let mut rest = seg;
    while let Some(i) = rest.find("{{") {
        out.push_str(&rest[..i]);
        let r = &rest[i + 2..];
        let j = r.find("}}").ok_or("閉じが無い")?;
        let p = r[..j].trim();
        if let Some(n) = 数える(ctx, p) {
            out.push_str(&n.to_string());
        } else {
            let v = 解く(ctx, p).ok_or(format!("経路が解けない: {p}"))?;
            out.push_str(&文字に(v));
        }
        rest = &r[j + 2..];
    }
    out.push_str(rest);
    Ok(out)
}

/// いちばん外側の `{{else}}` の位置 ── 入れ子の `#if` の中のものは数えない。
fn 外側の分岐(inner: &str) -> Option<usize> {
    let (mut 深さ, mut i) = (0usize, 0usize);
    while i < inner.len() {
        if inner[i..].starts_with("{{#if ") {
            深さ += 1;
            i += "{{#if ".len();
        } else if inner[i..].starts_with("{{/if}}") {
            深さ -= 1;
            i += "{{/if}}".len();
        } else if 深さ == 0 && inner[i..].starts_with("{{else}}") {
            return Some(i);
        } else {
            i += inner[i..].chars().next().map_or(1, char::len_utf8);
        }
    }
    None
}

/// 雛形と宣言から、射影を出す。**concrete はコードを1行も書かない。**
pub fn render(tpl: &str, ctx: &Value) -> Result<String, String> {
    let mut out = String::new();
    let mut rest = tpl;
    loop {
        let Some(i) = rest.find("{{#") else {
            out.push_str(&穴を埋める(rest, ctx)?);
            return Ok(out);
        };
        out.push_str(&穴を埋める(&rest[..i], ctx)?);
        let r = &rest[i + 3..];
        let j = r.find("}}").ok_or("閉じが無い")?;
        let (語, 残り) = r[..j].trim().split_once(' ').ok_or("経路が無い")?;
        // 区切りは、経路のあとに引用符で書く ── {{#each ops ", "}}
        let (経路, 区切り) = match 残り.split_once(' ') {
            Some((p, s)) => (p, s.trim().trim_matches('"').to_string()),
            None => (残り, String::new()),
        };
        let 開き = format!("{{{{#{語} ");
        let 閉じ = format!("{{{{/{語}}}}}");
        let 中 = &r[j + 2..];
        // **同じ語の入れ子を数える** ── 数えないと、内側の閉じで切れる
        let k = {
            let (mut 深さ, mut i, mut 見つけた) = (1usize, 0usize, None);
            while i < 中.len() {
                if 中[i..].starts_with(&開き) {
                    深さ += 1;
                    i += 開き.len();
                } else if 中[i..].starts_with(&閉じ) {
                    深さ -= 1;
                    if 深さ == 0 {
                        見つけた = Some(i);
                        break;
                    }
                    i += 閉じ.len();
                } else {
                    i += 中[i..].chars().next().map_or(1, char::len_utf8);
                }
            }
            見つけた.ok_or(format!("{閉じ} が無い"))?
        };
        let inner = &中[..k];
        match 語 {
            "each" => {
                let v = 解く(ctx, 経路).ok_or(format!("経路が解けない: {経路}"))?;
                let xs = v.as_array().ok_or(format!("並びではない: {経路}"))?;
                for (i, x) in xs.iter().enumerate() {
                    if i > 0 {
                        out.push_str(&区切り);
                    }
                    out.push_str(&render(inner, x)?);
                }
            }
            "if" => {
                // 分岐 ── {{#if 経路}}…{{else}}…{{/if}}
                let (表, 裏) = match 外側の分岐(inner) {
                    Some(k) => (&inner[..k], &inner[k + "{{else}}".len()..]),
                    None => (inner, ""),
                };
                let 真 =
                    解く(ctx, 経路).is_some_and(|v| v != &Value::Null && v != &Value::Bool(false));
                out.push_str(&render(if 真 { 表 } else { 裏 }, ctx)?);
            }
            _ => return Err(format!("知らない記法: {語}")),
        }
        rest = &中[k + 閉じ.len()..];
    }
}
