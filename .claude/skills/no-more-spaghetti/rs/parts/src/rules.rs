// SPDX-License-Identifier: MIT
//! 規則ファイルを読む。**道具の名前を、この側に書かない。**
//!
//! 実行するものは規則から来る ── 言語ごとの違いは規則が持つので、この Skill は
//! 言語を1つも認知しない。
//!
//! **規則は、適用する成果物（`units`）の名前を持つ。** 成果物は根 ・ 言語 ・ 層の並びを
//! 持ち、規則は成果物ごとに実行1件へ展開する ── 実行する場所は「成果物の根 ＋
//! `check.target`」である。

use std::io;
use std::path::Path;

use serde_json::Value;

/// すべての成果物を指す名前。**依存の向きの規則では、層を持つ成果物だけを指す。**
pub const EVERY_UNIT: &str = "*";

/// 成果物の層1つ。**名前は概念の層、識別子はその言語での名前である。**
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct UnitLayer {
    /// 概念の層の名前。
    pub name: String,
    /// その言語で層を識別する文字列（`where`）。**1つの層が複数を持てる。**
    pub ids: Vec<String>,
    /// 層の性質（`independent` ・ `closed` ・ `composes`）。
    pub marks: Vec<String>,
}

/// 成果物1つ。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct Unit {
    /// 成果物の名前。
    pub name: String,
    /// 根（規則ファイルを実行する場所からの相対）。
    pub root: String,
    /// 言語。**層を持たない成果物では空である。**
    pub language: String,
    /// 層の並び。**内から外の順である。**
    pub layers: Vec<UnitLayer>,
    /// 生成の手順。**inward の直前に、成果物の根で実行する** ── 生成物を指す参照を、
    /// 実在しない参照先として報告しないためである。
    pub generate: Vec<Vec<String>>,
}

/// 依存の向きを測る規則の中身。**言語と層は、成果物から来る。**
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct Inward {
    /// 言語。
    pub language: String,
    /// 層の並び。
    pub layers: Vec<UnitLayer>,
}

/// 実行1件。**契約の全部を持たない** ── 実行に必要な欄だけを読む。
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct Rule {
    /// 何を守るか。**2つ以上の成果物へ展開した規則は、成果物の名前を添える。**
    pub name: String,
    /// 実行するコマンド。**空なら検査できない。** 依存の向きの規則では、表示のための形である。
    pub tool: Vec<String>,
    /// 道具が配列でなかったか。
    pub tool_is_not_a_list: bool,
    /// 実行する場所（規則ファイルを実行する場所からの相対）。
    pub target: String,
    /// 原典の立場。
    pub authority: String,
    /// 出典を持つか。
    pub has_source: bool,
    /// どの成果物の実行か。**成果物を指さない規則では空である。**
    pub unit: String,
    /// 依存の向きを測る規則なら、その中身。
    pub inward: Option<Inward>,
    /// 規則が指したのに実在しない成果物の名前。**空でなければ実行しない。**
    pub missing_unit: String,
    /// 生成の手順なら、その成果物の名前。**失敗すると、その成果物の inward を実行しない。**
    pub generates: String,
}

/// 規則ファイルから成果物の一覧を取り出す。**`units` が無ければ空である。**
#[must_use]
pub fn units(parsed: &Value) -> Vec<Unit> {
    let Some(map) = parsed.get("units").and_then(Value::as_object) else {
        return Vec::new();
    };
    map.iter()
        .map(|(name, raw)| Unit {
            name: name.clone(),
            root: text(raw.get("root")),
            language: text(raw.get("language")),
            layers: raw
                .get("layers")
                .and_then(Value::as_array)
                .map(|list| list.iter().map(layer).collect())
                .unwrap_or_default(),
            generate: raw
                .get("generate")
                .and_then(Value::as_array)
                .map(|steps| {
                    steps
                        .iter()
                        .filter_map(Value::as_array)
                        .map(|step| {
                            step.iter()
                                .filter_map(Value::as_str)
                                .map(str::to_owned)
                                .collect()
                        })
                        .collect()
                })
                .unwrap_or_default(),
        })
        .collect()
}

fn layer(raw: &Value) -> UnitLayer {
    let strings = |key: &str| -> Vec<String> {
        raw.get(key)
            .and_then(Value::as_array)
            .map(|list| {
                list.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default()
    };
    UnitLayer {
        name: text(raw.get("name")),
        ids: strings("where"),
        marks: strings("marks"),
    }
}

/// 規則ファイルから、実行の一覧を取り出す。**1件だけの形も受け取る。**
///
/// # Errors
///
/// 読めないとき、または JSON として解析できないときに返す。
pub fn load(path: &Path) -> io::Result<Vec<Rule>> {
    let body = std::fs::read_to_string(path)?;
    let parsed: Value = serde_json::from_str(&body).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("JSON として読めない ── {e}"),
        )
    })?;
    let items = match parsed.get("rules").and_then(Value::as_array) {
        Some(list) => list.clone(),
        None => vec![parsed.clone()],
    };
    let all = units(&parsed);
    let expanded: Vec<Rule> = items.iter().flat_map(|raw| expand(raw, &all)).collect();
    Ok(with_generation(expanded, &all))
}

/// 成果物ごとに、最初の inward の実行の直前へ、生成の手順を挿入する。**1回の実行で、
/// 成果物ごとに1回だけである** ── 同じ成果物に inward の規則が2件あっても、生成は1回で足りる。
fn with_generation(rules: Vec<Rule>, all: &[Unit]) -> Vec<Rule> {
    let mut done: Vec<String> = Vec::new();
    let mut out = Vec::with_capacity(rules.len());
    for rule in rules {
        if rule.inward.is_some() && !done.contains(&rule.unit) {
            done.push(rule.unit.clone());
            if let Some(unit) = all.iter().find(|u| u.name == rule.unit) {
                let count = unit.generate.len();
                for (i, step) in unit.generate.iter().enumerate() {
                    let name = if count > 1 {
                        format!("生成の手順（{}）{}/{count}", unit.name, i + 1)
                    } else {
                        format!("生成の手順（{}）", unit.name)
                    };
                    out.push(Rule {
                        name,
                        tool: step.clone(),
                        target: rule.target.clone(),
                        unit: unit.name.clone(),
                        generates: unit.name.clone(),
                        has_source: true,
                        ..Rule::default()
                    });
                }
            }
        }
        out.push(rule);
    }
    out
}

fn text(value: Option<&Value>) -> String {
    value.and_then(Value::as_str).unwrap_or_default().to_owned()
}

/// 根と `check.target` をつなぐ。**`.` は書かない** ── 同じ場所が2通りの文字列にならない。
fn join(root: &str, target: &str) -> String {
    let root = root.trim_end_matches('/');
    match (
        root.is_empty() || root == ".",
        target.is_empty() || target == ".",
    ) {
        (true, _) => target.to_owned(),
        (false, true) => root.to_owned(),
        (false, false) => format!("{root}/{target}"),
    }
}

/// 規則1件を、指す成果物ごとの実行へ展開する。
fn expand(raw: &Value, all: &[Unit]) -> Vec<Rule> {
    let base = one(raw);
    let is_inward = raw
        .get("check")
        .and_then(|c| c.get("inward"))
        .and_then(Value::as_bool)
        == Some(true);
    let Some(names) = raw.get("units").and_then(Value::as_array) else {
        return vec![base];
    };
    let mut picked: Vec<Result<&Unit, String>> = Vec::new();
    for name in names.iter().filter_map(Value::as_str) {
        if name == EVERY_UNIT {
            picked.extend(
                all.iter()
                    .filter(|u| !is_inward || !u.layers.is_empty())
                    .map(Ok),
            );
        } else {
            picked.push(
                all.iter()
                    .find(|u| u.name == name)
                    .ok_or_else(|| name.to_owned()),
            );
        }
    }
    let many = picked.len() > 1;
    picked
        .into_iter()
        .map(|got| {
            let mut rule = base.clone();
            match got {
                Ok(unit) => {
                    rule.unit.clone_from(&unit.name);
                    rule.target = join(&unit.root, &base.target);
                    if many {
                        rule.name = format!("{}（{}）", base.name, unit.name);
                    }
                    if is_inward {
                        rule.tool = shown(unit);
                        rule.inward = Some(Inward {
                            language: unit.language.clone(),
                            layers: unit.layers.clone(),
                        });
                    }
                }
                Err(name) => {
                    if many {
                        rule.name = format!("{}（{name}）", base.name);
                    }
                    rule.missing_unit = name;
                }
            }
            rule
        })
        .collect()
}

/// 依存の向きの規則を、実行の前に見せる形。**CLI の `inward` と同じ引数の並びである。**
fn shown(unit: &Unit) -> Vec<String> {
    let mut out = vec!["inward".to_owned(), unit.language.clone(), ".".to_owned()];
    out.extend(unit.layers.iter().map(|l| {
        let mut one = format!("{}={}", l.name, l.ids.join("|"));
        for mark in &l.marks {
            one.push(':');
            one.push_str(mark);
        }
        one
    }));
    out
}

fn one(raw: &Value) -> Rule {
    let check = raw.get("check");
    let tool_value = check.and_then(|c| c.get("tool"));
    let (tool, tool_is_not_a_list) = match tool_value {
        Some(Value::Array(items)) => (
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect(),
            false,
        ),
        Some(_) => (Vec::new(), true),
        None => (Vec::new(), false),
    };
    Rule {
        name: {
            let name = text(raw.get("rule"));
            if name.is_empty() {
                "（名前が無い）".to_owned()
            } else {
                name
            }
        },
        tool,
        tool_is_not_a_list,
        target: text(check.and_then(|c| c.get("target"))),
        authority: text(raw.get("source").and_then(|s| s.get("authority"))),
        has_source: raw.get("source").is_some_and(|s| !s.is_null()),
        ..Rule::default()
    }
}
