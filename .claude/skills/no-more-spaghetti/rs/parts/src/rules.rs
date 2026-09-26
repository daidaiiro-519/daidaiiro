// SPDX-License-Identifier: MIT
//! 規則ファイルを読む。**道具の名前を、この側に書かない。**
//!
//! 実行するものは規則から来る ── 言語ごとの違いは規則が持つので、この Skill は
//! 言語を1つも認知しない。

use std::io;
use std::path::Path;

use serde_json::Value;

/// 規則1件。**契約の全部を持たない** ── 実行に要る欄だけを読む。
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct Rule {
    /// 何を守るか。
    pub name: String,
    /// 実行するコマンド。**空なら検査できない。**
    pub tool: Vec<String>,
    /// 道具が配列でなかったか。
    pub tool_is_not_a_list: bool,
    /// 実行する場所（成果物の場所からの相対）。
    pub target: String,
    /// 原典の立場。
    pub authority: String,
    /// 出典を持つか。
    pub has_source: bool,
}

/// 規則ファイルから一覧を取り出す。**1件だけの形も受け取る。**
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
        None => vec![parsed],
    };
    Ok(items.iter().map(one).collect())
}

fn text(value: Option<&Value>) -> String {
    value.and_then(Value::as_str).unwrap_or_default().to_owned()
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
    }
}
