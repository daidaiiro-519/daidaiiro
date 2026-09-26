// SPDX-License-Identifier: MIT
//! 規則ファイルの形を検査する。**実行の前に、契約を満たすかを確認する。**
//!
//! 規則を立てる条件は2つで、両方を満たすものだけを書く ── 無ければ外すか、コマンドで
//! 検査できるか。**この側が見られるのは後者だけである。**

use std::io;
use std::path::Path;

use serde_json::Value;

/// 契約の置き場所。**呼ぶ側が渡す** ── この crate は Skill の並びを認知しない。
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Contracts {
    /// 規則ファイルの契約。
    pub rules: std::path::PathBuf,
    /// 概念の出典の契約。
    pub concepts: std::path::PathBuf,
    /// 案内の契約。
    pub schema_meta: std::path::PathBuf,
}

impl Contracts {
    /// 契約の置き場所を組む。
    #[must_use]
    pub const fn new(
        rules: std::path::PathBuf,
        concepts: std::path::PathBuf,
        schema_meta: std::path::PathBuf,
    ) -> Self {
        Self {
            rules,
            concepts,
            schema_meta,
        }
    }
}

fn read_json(path: &Path) -> Result<Value, String> {
    let body = std::fs::read_to_string(path).map_err(|e| format!("読めない ── {e}"))?;
    serde_json::from_str(&body).map_err(|e| format!("JSON として読めない ── {e}"))
}

/// 形の検査を1回実施し、食い違いを並べる。**並びは経路の順である。**
fn against(schema: &Value, instance: &Value, head: &str) -> Vec<String> {
    let built = jsonschema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .build(schema);
    let Ok(validator) = built else {
        return vec![format!("{head}: 契約がスキーマとして無効である")];
    };
    let mut found: Vec<String> = validator
        .iter_errors(instance)
        .map(|e| {
            let at = e.instance_path.to_string();
            let at = at.trim_start_matches('/');
            format!("{head}: {at} ── {e}")
        })
        .collect();
    found.sort();
    found
}

/// 規則ファイルを検査する。
///
/// # Errors
///
/// 契約を読めないときに返す。
pub fn check_rules(rules_file: &Path, contracts: &Contracts) -> io::Result<Vec<String>> {
    let instance = match read_json(rules_file) {
        Ok(value) => value,
        Err(why) => return Ok(vec![why]),
    };
    let mut findings = match read_json(&contracts.rules) {
        Ok(schema) => against(&schema, &instance, "形"),
        Err(why) => vec![format!("契約を読めない ── {why}")],
    };
    let items = match instance.get("rules").and_then(Value::as_array) {
        Some(list) => list.clone(),
        None => vec![instance.clone()],
    };
    for (i, raw) in items.iter().enumerate() {
        let name = raw
            .get("rule")
            .and_then(Value::as_str)
            .map_or_else(|| format!("{i}件目"), str::to_owned);
        match raw.get("check").and_then(|c| c.get("tool")) {
            None | Some(Value::Null) => findings.push(format!(
                "{name}: 検証方法に道具が無い ── コマンドで検査できない規則は立てない"
            )),
            Some(Value::Array(items)) if items.is_empty() => findings.push(format!(
                "{name}: 検証方法に道具が無い ── コマンドで検査できない規則は立てない"
            )),
            Some(Value::Array(_)) => {}
            Some(_) => findings.push(format!(
                "{name}: 道具が配列ではない ── シェルを経由すると、展開が実行するシェルに依存する"
            )),
        }
        if !raw.get("source").is_some_and(|s| !s.is_null()) {
            findings.push(format!(
                "{name}: 出典が無い ── 外（原典）か内（記録）かを書く"
            ));
        }
    }
    Ok(findings)
}

/// 層の宣言が、構造として成立するかを見る。
///
/// **値をファイルの場所として検査しない** ── 層を識別する文字列は言語ごとに形が違い、
/// 経路とは限らない。値が正しいかは、**依存の向きの道具が実行できるかで判明する**。
///
/// # Errors
///
/// 規則ファイルを読めないときに返す。
pub fn check_layers(rules_file: &Path) -> io::Result<Vec<String>> {
    let instance = match read_json(rules_file) {
        Ok(value) => value,
        Err(why) => return Ok(vec![why]),
    };
    let Some(layers) = instance.get("layers").and_then(Value::as_object) else {
        return Ok(Vec::new());
    };
    let order = instance
        .get("order")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut findings: Vec<String> = order
        .iter()
        .filter_map(Value::as_str)
        .filter(|name| !layers.contains_key(*name))
        .map(|name| format!("並びの {name} が、層に無い"))
        .collect();
    // **宣言していない層を検出する。** 引数で別に渡すと、宣言と検査がずれても
    // 誰も気づかない（実測 ── 2つの層が宣言から落ちたまま検査されていた）
    let named: Vec<&str> = order.iter().filter_map(Value::as_str).collect();
    findings.extend(
        layers
            .keys()
            .filter(|name| !named.contains(&name.as_str()))
            .map(|name| format!("層の {name} が、並びに無い")),
    );
    Ok(findings)
}

/// 概念の出典が契約を満たすかを見る。
///
/// **原文は同梱しない。** したがってこの側が見られるのは、引用と、取り直すための
/// 4つの値（url ・ 日 ・ sha256 ・ 行）が揃っているかまでである。
///
/// # Errors
///
/// 契約を読めないときに返す。
pub fn check_concepts(path: &Path, contracts: &Contracts) -> io::Result<Vec<String>> {
    let instance = match read_json(path) {
        Ok(value) => value,
        Err(why) => return Ok(vec![why]),
    };
    let mut findings = match read_json(&contracts.concepts) {
        Ok(schema) => against(&schema, &instance, "形"),
        Err(why) => vec![format!("契約を読めない ── {why}")],
    };
    let items = instance
        .get("concepts")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    for (i, raw) in items.iter().enumerate() {
        let name = raw
            .get("concept")
            .and_then(Value::as_str)
            .map_or_else(|| format!("{i}件目"), str::to_owned);
        let source = raw.get("source");
        let quote = source
            .and_then(|s| s.get("quote"))
            .and_then(Value::as_str)
            .unwrap_or_default();
        if quote.is_empty() {
            findings.push(format!(
                "{name}: 引用が無い ── 原文を提示できない概念を、手順の根拠にしない"
            ));
        }
        for field in ["url", "date", "sha256", "line"] {
            let got = source
                .and_then(|s| s.get("fetched"))
                .and_then(|f| f.get(field));
            // **空の文字列も欠けとして数える** ── 雛形のまま出すと、値が入っていない
            let filled = got.is_some_and(|v| !v.is_null() && v.as_str() != Some(""));
            if !filled {
                findings.push(format!(
                    "{name}: 取得に {field} が無い ── 同じ版かを、あとから判定できない"
                ));
            }
        }
    }
    Ok(findings)
}

/// スキーマ自身を実体として検証する。**案内の欠落を検出する。**
///
/// # Errors
///
/// 契約を読めないときに返す。
pub fn check_schema(path: &Path, contracts: &Contracts) -> io::Result<Vec<String>> {
    let instance = match read_json(path) {
        Ok(value) => value,
        Err(why) => return Ok(vec![why]),
    };
    let mut findings = Vec::new();
    if jsonschema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .build(&instance)
        .is_err()
    {
        findings.push("スキーマとして無効である".to_owned());
    }
    match read_json(&contracts.schema_meta) {
        Ok(meta) => findings.extend(against(&meta, &instance, "案内")),
        Err(why) => findings.push(format!("契約を読めない ── {why}")),
    }
    Ok(findings)
}

/// 渡されたファイルの種類。**入口を増やすと、呼ぶ側が形を推測することになる。**
///
/// **種類は中身で決める。** 引数で受け取ると、呼ぶ側が毎回それを決めることになる ──
/// 決め損ねると、別の契約で検査して、在るはずの欄が無いと報告する（実測）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Kind {
    /// 規則ファイル。
    Rules,
    /// 概念の出典。
    Concepts,
    /// スキーマそのもの。
    Schema,
    /// 生成物を持つスキーマが指す先。**そのスキーマ自身で検査する。**
    Generated,
}

impl Kind {
    /// 機械が分岐する値。
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Rules => "rules",
            Self::Concepts => "concepts",
            Self::Schema => "schema",
            Self::Generated => "generated",
        }
    }
}

/// 中身から種類を決める。
///
/// # Errors
///
/// 読めないときに返す。
pub fn kind_of(path: &Path) -> io::Result<Kind> {
    let Ok(instance) = read_json(path) else {
        return Ok(Kind::Rules);
    };
    if instance.get("concepts").is_some() {
        return Ok(Kind::Concepts);
    }
    if instance.get("properties").is_some() && instance.get("rules").is_none() {
        return Ok(Kind::Schema);
    }
    // **生成物を持つスキーマが指す先は、そのスキーマで検査する** ── 規則の契約を
    // 当てると、在るはずの無い欄を要求することになる
    if instance.get("rules").is_none() && instance.get("$schema").is_some() {
        return Ok(Kind::Generated);
    }
    Ok(Kind::Rules)
}

/// 生成物を持つスキーマが指す先を、そのスキーマで検査する。
///
/// # Errors
///
/// 契約を読めないときに返す。
pub fn check_generated(path: &Path) -> io::Result<Vec<String>> {
    let instance = match read_json(path) {
        Ok(value) => value,
        Err(why) => return Ok(vec![why]),
    };
    let Some(reference) = instance.get("$schema").and_then(Value::as_str) else {
        return Ok(vec![
            "$schema が無い ── どの契約で検査するかが決まらない".to_owned()
        ]);
    };
    let base = path.parent().unwrap_or(Path::new("."));
    let schema_path = base.join(reference);
    match read_json(&schema_path) {
        Ok(schema) => Ok(against(&schema, &instance, "形")),
        Err(why) => Ok(vec![format!("契約を読めない ── {why}")]),
    }
}

/// 読めないファイルかを見る。**誤用と検出を、同じ番号で返さないためである。**
#[must_use]
pub fn unreadable(path: &Path) -> Option<String> {
    match std::fs::read_to_string(path) {
        Err(e) => Some(format!("ファイルを読めない ── {e}")),
        Ok(body) => serde_json::from_str::<Value>(&body)
            .err()
            .map(|e| format!("JSON として読めない ── {e}")),
    }
}

/// `$schema` の相対の経路が解決するかを見る ── 移動で壊れる（実測 2026-09-26）。
#[must_use]
pub fn unresolved_schema(path: &Path) -> Option<String> {
    let body = std::fs::read_to_string(path).ok()?;
    let parsed: Value = serde_json::from_str(&body).ok()?;
    let reference = parsed.get("$schema").and_then(Value::as_str)?;
    if reference.starts_with("http") {
        return None;
    }
    let base = path.parent().unwrap_or(Path::new("."));
    if base.join(reference).exists() {
        return None;
    }
    Some(format!("$schema が解決しない ── {reference}"))
}
