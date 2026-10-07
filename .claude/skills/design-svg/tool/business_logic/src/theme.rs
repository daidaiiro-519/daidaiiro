// SPDX-License-Identifier: MIT
//! デザイントークン ── CSS の `:root{--var}` に相当する。
//!
//! **正本は `references/theme.json` である。** ここは読むだけで、値を1つも持たない ──
//! 2か所に書くと、片方だけ直したときに誰も気づけない。
//!
//! 組み立ての時点で読み込む ── 実行のたびに置き場所を探すと、どこから呼んだかで結果が
//! 変わる。

use std::sync::OnceLock;

use serde_json::{Map, Value};

/// 正本。**組み立ての時点で埋め込む。**
const SOURCE: &str = include_str!("../../../references/theme.json");

/// 役割の接頭辞。
pub const ROLE_PREFIX: &str = "role.";
/// 何も上書きしない役割。**常に有効。**
pub const PLAIN: &str = "plain";

/// 正本を読んだもの。
#[derive(Debug)]
pub struct Source {
    /// 既定のテーマ。
    pub tokens: Map<String, Value>,
    /// 値に妥当な範囲があるトークン。`(最小, 最大)`
    pub ranges: Vec<(String, f64, f64)>,
    /// 色の濃さの呼び名から、色のトークンへ。
    pub tones: Map<String, Value>,
    /// 図の表記法 ── class の一覧 ・ 段階 ・ 禁止した組み合わせ ・ ダークモードの値 ・ 矢じり ・ 検査のしきい値。
    /// **読むのは `classes` だけである** ── ここは欄を取り出すだけで、意味を持たない。
    pub notation: Map<String, Value>,
}

fn load() -> Source {
    let whole: Value =
        serde_json::from_str(SOURCE).expect("theme.json は正本であり、読めなければ組めない");
    let tokens = whole
        .get("tokens")
        .and_then(|x| x.as_object())
        .cloned()
        .unwrap_or_default();
    let ranges = whole
        .get("ranges")
        .and_then(|x| x.as_object())
        .map(|m| {
            m.iter()
                .filter_map(|(k, v)| {
                    let pair = v.as_array()?;
                    Some((k.clone(), pair.first()?.as_f64()?, pair.get(1)?.as_f64()?))
                })
                .collect()
        })
        .unwrap_or_default();
    let tones = whole
        .get("tones")
        .and_then(|x| x.as_object())
        .cloned()
        .unwrap_or_default();
    let notation = ["scale", "classes", "forbidden", "dark", "marker", "checks"]
        .iter()
        .filter_map(|k| whole.get(*k).map(|v| ((*k).to_owned(), v.clone())))
        .collect();
    Source {
        tokens,
        ranges,
        tones,
        notation,
    }
}

/// 正本。**1度だけ読む。**
pub fn source() -> &'static Source {
    static ONCE: OnceLock<Source> = OnceLock::new();
    ONCE.get_or_init(load)
}

/// 既定のテーマ。
#[must_use]
pub fn default_theme() -> &'static Map<String, Value> {
    &source().tokens
}

/// 色の濃さの呼び名から、色のトークンを引く。
#[must_use]
pub fn tone(name: &str) -> Option<&'static str> {
    source().tones.get(name).and_then(|x| x.as_str())
}

/// 数を返すキーから、数として取り出す。
///
/// # Errors
///
/// そのキーがテーマに無いときと、値が数でないときに返す。
pub fn num(theme: &Map<String, Value>, key: &str) -> Result<f64, String> {
    let v = theme
        .get(key)
        .ok_or_else(|| format!("トークン '{key}' がテーマに無い"))?;
    match v {
        Value::Number(n) => n
            .as_f64()
            .ok_or_else(|| format!("トークン '{key}' は数のはずだが読めない")),
        other => Err(format!(
            "トークン '{key}' は数のはずだが {} が入っている: {}",
            type_name(other),
            crate::py::repr(other)
        )),
    }
}

/// 値の型の名前。**移す前と同じ語で言う** ── 報告を照らすときに、別の語だと探せない。
#[must_use]
pub fn type_name(v: &Value) -> &'static str {
    match v {
        Value::Null => "NoneType",
        Value::Bool(_) => "bool",
        Value::Number(n) if n.is_f64() => "float",
        Value::Number(_) => "int",
        Value::String(_) => "str",
        Value::Array(_) => "list",
        Value::Object(_) => "dict",
    }
}

/// 解決済みの見た目。**キーごとに型が決まっているので、取り出すときに型を選ぶ。**
///
/// キーで参照する以外の使い方（反復 ・ 複製 ・ 展開）は持たせない ── 解決済みの見た目は
/// 「参照して使うもの」であって、組み替えるものではない。
#[derive(Debug, Clone)]
pub struct Style {
    /// 値。
    pub values: Map<String, Value>,
}

impl Style {
    /// 数を返すキーから取り出す。
    ///
    /// # Errors
    ///
    /// そのキーが無いときと、値が数でないときに返す。
    pub fn num(&self, key: &str) -> Result<f64, String> {
        let v = self
            .values
            .get(key)
            .ok_or_else(|| format!("トークン '{key}' がテーマに無い"))?;
        match v {
            Value::Number(n) => n
                .as_f64()
                .ok_or_else(|| format!("トークン '{key}' を数として読めない")),
            other => Err(format!(
                "トークン '{key}' は数のはずだが {}: {}",
                type_name(other),
                crate::py::repr(other)
            )),
        }
    }

    /// 数を、素のまま出すときの書き方で返す。**整数は整数のまま。**
    ///
    /// # Errors
    ///
    /// そのキーが無いときと、値が数でないときに返す。
    pub fn raw(&self, key: &str) -> Result<String, String> {
        let v = self
            .values
            .get(key)
            .ok_or_else(|| format!("トークン '{key}' がテーマに無い"))?;
        Ok(crate::py::num(v))
    }

    /// 文字列を返すキーから取り出す。
    ///
    /// # Errors
    ///
    /// そのキーが無いときと、値が文字列でないときに返す。
    pub fn text(&self, key: &str) -> Result<String, String> {
        match self.values.get(key) {
            None => Err(format!("トークン '{key}' がテーマに無い")),
            Some(Value::String(s)) => Ok(s.clone()),
            Some(other) => Err(format!(
                "トークン '{key}' は文字列のはずだが {}: {}",
                type_name(other),
                crate::py::repr(other)
            )),
        }
    }

    /// 文字列を返すキーから取り出す。**無ければ既定値を返す。**
    ///
    /// # Errors
    ///
    /// 値が文字列でないときに返す。
    pub fn text_or(&self, key: &str, default: &str) -> Result<String, String> {
        if self.values.contains_key(key) {
            self.text(key)
        } else {
            Ok(default.to_owned())
        }
    }

    /// 在れば文字列、無ければ `None`。**無いことが既定のキーに使う。**
    #[must_use]
    pub fn opt_text(&self, key: &str) -> Option<String> {
        self.values
            .get(key)
            .and_then(|v| v.as_str())
            .map(str::to_owned)
    }

    /// 文字列の並びを返すキーから取り出す（系列の色など）。
    ///
    /// # Errors
    ///
    /// そのキーが無いときと、値が並びでないときに返す。
    pub fn tones(&self, key: &str) -> Result<Vec<String>, String> {
        match self.values.get(key) {
            None => Err(format!("トークン '{key}' がテーマに無い")),
            Some(Value::Array(items)) => Ok(items
                .iter()
                .map(|x| x.as_str().map_or_else(|| crate::py::num(x), str::to_owned))
                .collect()),
            Some(other) => Err(format!(
                "トークン '{key}' は並びのはずだが {}: {}",
                type_name(other),
                crate::py::repr(other)
            )),
        }
    }
}
