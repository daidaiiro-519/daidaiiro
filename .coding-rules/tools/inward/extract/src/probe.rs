// SPDX-License-Identifier: MIT
//! 探査の脚本を呼び、返った JSON を辺へ直す。
//!
//! **脚本はその言語で書く。** 構文の解析はその言語自身の解析器に任せる ── こちらは
//! 呼んで受け取るだけである。脚本は `probes/` に1言語1つで置く。

use std::io;
use std::path::Path;

use inward_core::Edge;
use serde_json::Value;

use crate::tool::{run, Ran};
use crate::{Escape, Extracted};

/// 脚本が返す JSON を、抽出の結果へ直す。
///
/// # Errors
///
/// JSON として読めないときに返す。
pub fn from_json(text: &str) -> io::Result<Extracted> {
    let parsed: Value = serde_json::from_str(text).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("JSON として読めない ── {e}"),
        )
    })?;
    let list = |key: &str| -> Vec<Value> {
        parsed
            .get(key)
            .and_then(|x| x.as_array())
            .cloned()
            .unwrap_or_default()
    };
    let text_at = |v: &Value, key: &str| -> String {
        v.get(key)
            .and_then(|x| x.as_str())
            .unwrap_or_default()
            .to_owned()
    };
    Ok(Extracted {
        edges: list("edges")
            .iter()
            .map(|e| Edge::new(text_at(e, "from"), text_at(e, "to"), text_at(e, "at")))
            .collect(),
        escapes: list("escapes")
            .iter()
            .map(|e| Escape::new(text_at(e, "in"), text_at(e, "at"), text_at(e, "how")))
            .collect(),
        undecided: list("undecided")
            .iter()
            .filter_map(|x| x.as_str())
            .map(str::to_owned)
            .collect(),
        limits: Vec::new(),
    })
}

/// 脚本を1回呼ぶ。**道具が無ければ、判定できなかったことを返す。**
///
/// # Errors
///
/// 起動が「見つからない」以外の理由で失敗したとき、または返りが JSON でないときに返す。
pub fn ask(program: &str, args: &[&str], cwd: &Path) -> io::Result<Extracted> {
    match run(program, args, cwd)? {
        Ran::Absent => Ok(Extracted {
            undecided: vec![format!("{program} が無いので判定していない")],
            ..Extracted::default()
        }),
        Ran::Output {
            stdout,
            stderr,
            code,
        } => {
            if code != Some(0) {
                return Ok(Extracted {
                    undecided: vec![format!(
                        "{program} が終了コード {} を返した ── {}",
                        code.map_or_else(|| "（信号）".to_owned(), |c| c.to_string()),
                        stderr.trim()
                    )],
                    ..Extracted::default()
                });
            }
            from_json(&stdout)
        }
    }
}
