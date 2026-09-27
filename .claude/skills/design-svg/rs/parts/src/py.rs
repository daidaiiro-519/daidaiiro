// SPDX-License-Identifier: MIT
//! 数と値を、移す前と同じ書き方で文字にする。
//!
//! **素のまま SVG へ出す値は、書き方がそのまま出力に現れる。** 整数は `60`、小数は `12.0`
//! と書く ── 書き方を1つでも違えると、同じ宣言から別の SVG が出る。
//!
//! 識別子の材料も、同じ書き方で文字にしてから要約する ── 違えると、同じ材料から別の
//! 識別子が出る。

use serde_json::Value;

/// 小数を、最短で元へ戻る桁で書く。**指数は2桁以上で書く。**
///
/// 小数点より上の桁が 16 以上、または下の桁が 4 を超えて深いときだけ、指数で書く。
#[must_use]
pub fn float(v: f64) -> String {
    if v.is_nan() {
        return "nan".to_owned();
    }
    if v.is_infinite() {
        return if v > 0.0 {
            "inf".to_owned()
        } else {
            "-inf".to_owned()
        };
    }
    if v == 0.0 {
        return if v.is_sign_negative() {
            "-0.0".to_owned()
        } else {
            "0.0".to_owned()
        };
    }
    // 最短で元へ戻る桁を、指数の形で得る（例: 1.2345e2）
    let sci = format!("{v:e}");
    let (mantissa, exp) = sci.split_once('e').unwrap_or((&sci, "0"));
    let exp: i32 = exp.parse().unwrap_or(0);
    let negative = mantissa.starts_with('-');
    let digits: String = mantissa.chars().filter(char::is_ascii_digit).collect();
    let sign = if negative { "-" } else { "" };
    if (-4..16).contains(&exp) {
        // 固定小数点で書く
        let point = exp + 1; // 小数点の前に来る桁の数
        let body = if point <= 0 {
            format!("0.{}{digits}", "0".repeat((-point) as usize))
        } else if (point as usize) >= digits.len() {
            format!("{digits}{}.0", "0".repeat(point as usize - digits.len()))
        } else {
            format!(
                "{}.{}",
                &digits[..point as usize],
                &digits[point as usize..]
            )
        };
        format!("{sign}{body}")
    } else {
        let head = &digits[..1];
        let tail = &digits[1..];
        let mant = if tail.is_empty() {
            head.to_owned()
        } else {
            format!("{head}.{tail}")
        };
        let esign = if exp < 0 { "-" } else { "+" };
        format!("{sign}{mant}e{esign}{:02}", exp.abs())
    }
}

/// 値を、素のまま出すときの書き方で書く。**整数は整数のまま、小数は小数のまま。**
#[must_use]
pub fn num(value: &Value) -> String {
    match value {
        Value::Number(n) => n
            .as_i64()
            .map(|i| i.to_string())
            .or_else(|| n.as_u64().map(|u| u.to_string()))
            .unwrap_or_else(|| float(n.as_f64().unwrap_or(0.0))),
        Value::String(s) => s.clone(),
        Value::Bool(b) => if *b { "True" } else { "False" }.to_owned(),
        Value::Null => "None".to_owned(),
        other => repr(other),
    }
}

/// 文字列を、引用符で囲んだ形で書く。
///
/// **単引用符で囲む。** 中に単引用符が在って二重引用符が無いときだけ、二重引用符で囲む。
#[must_use]
pub fn quote(s: &str) -> String {
    let use_double = s.contains('\'') && !s.contains('"');
    let q = if use_double { '"' } else { '\'' };
    let mut out = String::new();
    out.push(q);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c == q => {
                out.push('\\');
                out.push(c);
            }
            c if (c as u32) < 0x20 || c as u32 == 0x7f => {
                out.push_str(&format!("\\x{:02x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push(q);
    out
}

/// 値を、材料としての書き方で書く。**並びは `[…]`、対応は `{…}`。**
#[must_use]
pub fn repr(value: &Value) -> String {
    match value {
        Value::Null => "None".to_owned(),
        Value::Bool(b) => if *b { "True" } else { "False" }.to_owned(),
        Value::Number(_) => num(value),
        Value::String(s) => quote(s),
        Value::Array(items) => {
            let inner: Vec<String> = items.iter().map(repr).collect();
            format!("[{}]", inner.join(", "))
        }
        Value::Object(map) => {
            let inner: Vec<String> = map
                .iter()
                .map(|(k, v)| format!("{}: {}", quote(k), repr(v)))
                .collect();
            format!("{{{}}}", inner.join(", "))
        }
    }
}

/// 材料の組を書く。**1つだけのときも、組であることを書く。**
#[must_use]
pub fn tuple(parts: &[String]) -> String {
    if parts.len() == 1 {
        return format!("({},)", parts[0]);
    }
    format!("({})", parts.join(", "))
}

/// 小数点以下を決まった桁で書く。**負の0は `-0` のまま書く** ── 移す前と同じである。
#[must_use]
pub fn fixed(v: f64, places: usize) -> String {
    format!("{v:.places$}")
}

/// 小数の並びを足す。**移す前の足し方をそのまま写す。**
///
/// 移す前の足し算は、端数を別に持って最後に戻す（Neumaier の方式）── 順に足すだけだと、
/// 同じ並びから最後の桁が違う和が出る（実測 ── 0.1 を10回足すと、順に足せば
/// 0.9999999999999999、移す前は 1.0）。
///
/// 最初の1つは、そのまま和にする ── 移す前は整数の0から始め、最初の小数を足した時点で
/// 小数の足し方へ切り替える。
#[must_use]
pub fn sum<I: IntoIterator<Item = f64>>(items: I) -> f64 {
    let mut it = items.into_iter();
    let Some(first) = it.next() else {
        return 0.0;
    };
    let mut total = first;
    let mut carry = 0.0_f64;
    for x in it {
        let t = total + x;
        if total.abs() >= x.abs() {
            carry += (total - t) + x;
        } else {
            carry += (x - t) + total;
        }
        total = t;
    }
    // 符号を失わないよう、端数が0でなく有限のときだけ戻す
    if carry != 0.0 && carry.is_finite() {
        total += carry;
    }
    total
}

/// 文字を小数として読む。**前後の空白は許す** ── 移す前と同じ読み方である。
#[must_use]
pub fn parse_float(s: &str) -> Option<f64> {
    let t = s.trim();
    if t.is_empty() {
        return None;
    }
    let cleaned: String = if t.contains('_') {
        // 桁区切りの下線は、数字どうしの間だけ許す
        let chars: Vec<char> = t.chars().collect();
        for (i, c) in chars.iter().enumerate() {
            if *c == '_' {
                let ok = i > 0
                    && i + 1 < chars.len()
                    && chars[i - 1].is_ascii_digit()
                    && chars[i + 1].is_ascii_digit();
                if !ok {
                    return None;
                }
            }
        }
        t.replace('_', "")
    } else {
        t.to_owned()
    };
    match cleaned.to_ascii_lowercase().as_str() {
        "inf" | "+inf" | "infinity" | "+infinity" => return Some(f64::INFINITY),
        "-inf" | "-infinity" => return Some(f64::NEG_INFINITY),
        "nan" | "+nan" | "-nan" => return Some(f64::NAN),
        _ => {}
    }
    cleaned.parse().ok()
}

/// 値の並びを足す。**整数だけなら整数のまま足し、小数が現れたら小数の足し方へ切り替える。**
///
/// 移す前は整数の0から始め、整数は誤差なく足す。最初の小数は素のまま足し、2つ目からは端数を
/// 別に持って足す ── 切り替えの瞬間まで写さないと、最後の桁が違う和が出る。
#[must_use]
pub fn sum_values(items: &[Value]) -> f64 {
    let mut int_total: i64 = 0;
    let mut it = items.iter();
    // 整数の区間
    let mut first_float: Option<f64> = None;
    for v in it.by_ref() {
        match v.as_i64() {
            Some(i) if v.is_i64() || v.is_u64() => int_total += i,
            _ => {
                first_float = Some(int_total as f64 + v.as_f64().unwrap_or(0.0));
                break;
            }
        }
    }
    let Some(start) = first_float else {
        return int_total as f64;
    };
    // 小数の区間 ── 端数を別に持つ
    let mut total = start;
    let mut carry = 0.0_f64;
    for v in it {
        let x = v.as_f64().unwrap_or(0.0);
        let t = total + x;
        if total.abs() >= x.abs() {
            carry += (total - t) + x;
        } else {
            carry += (x - t) + total;
        }
        total = t;
    }
    if carry != 0.0 && carry.is_finite() {
        total += carry;
    }
    total
}

/// 整数か小数か。**足し方と書き方が、どちらかで変わる。**
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Num {
    /// 整数。
    I(i64),
    /// 小数。
    F(f64),
}

impl Num {
    /// 小数として読む。
    #[must_use]
    pub const fn f(self) -> f64 {
        match self {
            Self::I(i) => i as f64,
            Self::F(x) => x,
        }
    }
}

/// 整数と小数の混ざった並びを、移す前の足し方で足す。
#[must_use]
pub fn sum_nums(items: &[Num]) -> Num {
    let mut int_total: i64 = 0;
    let mut it = items.iter();
    let mut start: Option<f64> = None;
    for v in it.by_ref() {
        match v {
            Num::I(i) => int_total += i,
            Num::F(x) => {
                start = Some(int_total as f64 + x);
                break;
            }
        }
    }
    let Some(mut total) = start else {
        return Num::I(int_total);
    };
    let mut carry = 0.0_f64;
    for v in it {
        let x = v.f();
        let t = total + x;
        if total.abs() >= x.abs() {
            carry += (total - t) + x;
        } else {
            carry += (x - t) + total;
        }
        total = t;
    }
    if carry != 0.0 && carry.is_finite() {
        total += carry;
    }
    Num::F(total)
}
