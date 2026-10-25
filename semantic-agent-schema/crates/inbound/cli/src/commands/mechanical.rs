//! 機械的なコマンドの引数を読む：`semantic-agent-schema <コマンド> <対象> [旗]`

use serde_json::Value;

/// 読んだコマンド
#[derive(Debug, Clone, PartialEq)]
pub enum Mechanical {
    /// Concrete の雛形を作る
    Create {
        /// 置き場所
        path: String,
        /// 持ち主
        owner: String,
        /// 種類
        kind: String,
        /// 作る値（create-request。--value の JSON。無ければ {}）
        value: Value,
    },
    /// Concrete からインスタンスを作る
    CreateInstance {
        /// 置き場所
        path: String,
        /// 従う Concrete の URN
        schema: String,
        /// id（無ければ作るときに決める）
        id: Option<String>,
        /// 作る値（--value の JSON。無ければ {}）
        value: Value,
    },
    /// 意図1つぶんの変更を記録する
    Record {
        /// 変えたファイル
        path: String,
        /// 変えた項目（JSON Pointer）
        pointers: Vec<String>,
        /// 意図
        intent: String,
        /// 目的の参照
        purpose: String,
        /// 書いた者の種類
        writer: String,
        /// 置き換える意図の id
        replaces: Option<String>,
        /// 記録の id（無ければ作るときに決める）
        id: Option<String>,
        /// 手元のセッション（tool:id）
        session: Option<String>,
    },
    /// 前回のコミットからの変更をまとめ、まとめを1件置く（record --summary）
    Summarize {
        /// まとめの id（無ければ作るときに決める）
        id: Option<String>,
    },
    /// 人が意図を判定する
    Judge {
        /// 変更の記録の id
        change_id: String,
        /// 結果
        result: String,
        /// 意味
        meaning: String,
        /// 理由
        reason: String,
    },
    /// 対象の変更の経緯
    History {
        /// 対象のファイル
        path: String,
    },
    /// 1つの場所の値を返す
    Get {
        /// 置き場所
        path: String,
        /// JSON Pointer
        pointer: String,
    },
    /// JSON Patch を適用する
    Patch {
        /// 置き場所
        path: String,
        /// op の並び
        ops: Value,
        /// 読んだときの ETag（楽観ロック）
        if_match: Option<String>,
    },
    /// 検証する
    Validate {
        /// 置き場所
        path: String,
    },
}

/// 読んだ引数：コマンドと、JSON で返すか
#[derive(Debug, Clone, PartialEq)]
pub struct Parsed {
    /// コマンド
    pub command: Mechanical,
    /// --json
    pub json: bool,
}

/// 引数を読む
///
/// # Errors
/// 知らないコマンド ・ 知らない旗 ・ 足りない値のとき（使い方の誤り）
pub fn parse(args: &[String]) -> Result<Parsed, String> {
    let (name, rest) = args.split_first().ok_or(
        "コマンドが無い（create ・ get ・ patch ・ validate ・ record ・ judge ・ history）",
    )?;
    if name == "record" && rest.first().map(String::as_str) == Some("--summary") {
        let mut json = false;
        let mut id = None;
        let mut it = rest[1..].iter();
        while let Some(flag) = it.next() {
            match flag.as_str() {
                "--json" => json = true,
                "--id" => id = Some(it.next().ok_or("--id の値が無い")?.clone()),
                _ => return Err(format!("record --summary は旗「{flag}」を受け取らない")),
            }
        }
        return Ok(Parsed {
            command: Mechanical::Summarize { id },
            json,
        });
    }
    let (path, flags) = rest
        .split_first()
        .ok_or_else(|| format!("{name} の対象が無い"))?;
    let allowed: &[&str] = match name.as_str() {
        "create" => &["--owner", "--kind", "--value", "--schema", "--id"],
        "get" => &["--pointer"],
        "patch" => &["--patch", "--if-match"],
        "validate" => &[],
        "record" => &[
            "--pointer",
            "--intent",
            "--purpose",
            "--writer",
            "--replaces",
            "--id",
            "--session",
        ],
        "judge" => &["--result", "--meaning", "--reason"],
        "history" => &[],
        _ => return Err(format!("知らないコマンド「{name}」")),
    };
    let mut json = false;
    let mut values: Vec<(&str, String)> = Vec::new();
    let mut it = flags.iter();
    while let Some(flag) = it.next() {
        if flag == "--json" {
            json = true;
        } else if let Some(known) = allowed.iter().find(|a| **a == flag.as_str()) {
            let v = it.next().ok_or_else(|| format!("{flag} の値が無い"))?;
            values.push((known, v.clone()));
        } else {
            return Err(format!("{name} は旗「{flag}」を受け取らない"));
        }
    }
    let all = |key: &str| -> Vec<String> {
        values
            .iter()
            .filter(|(k, _)| *k == key)
            .map(|(_, v)| v.clone())
            .collect()
    };
    let take = |key: &str| {
        values
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| v.clone())
    };
    let need = |key: &str| take(key).ok_or_else(|| format!("{name} には {key} が必要"));
    let path = path.clone();
    let command = match name.as_str() {
        "create" => {
            let value = match take("--value") {
                Some(v) => {
                    serde_json::from_str(&v).map_err(|e| format!("--value が JSON でない：{e}"))?
                }
                None => Value::Object(serde_json::Map::new()),
            };
            match take("--schema") {
                Some(schema) => Mechanical::CreateInstance {
                    path,
                    schema,
                    id: take("--id"),
                    value,
                },
                None => Mechanical::Create {
                    path,
                    owner: need("--owner")?,
                    kind: need("--kind")?,
                    value,
                },
            }
        }
        "get" => Mechanical::Get {
            path,
            pointer: take("--pointer").unwrap_or_default(),
        },
        "patch" => {
            let ops = serde_json::from_str(&need("--patch")?)
                .map_err(|e| format!("--patch が JSON でない：{e}"))?;
            Mechanical::Patch {
                path,
                ops,
                if_match: take("--if-match"),
            }
        }
        "record" => Mechanical::Record {
            path,
            pointers: all("--pointer"),
            intent: need("--intent")?,
            purpose: need("--purpose")?,
            writer: take("--writer").unwrap_or_else(|| "ai-agent".to_string()),
            replaces: take("--replaces"),
            id: take("--id"),
            session: take("--session"),
        },
        "judge" => Mechanical::Judge {
            change_id: path,
            result: need("--result")?,
            meaning: need("--meaning")?,
            reason: need("--reason")?,
        },
        "history" => Mechanical::History { path },
        _ => Mechanical::Validate { path },
    };
    Ok(Parsed { command, json })
}

#[cfg(test)]
mod tests {
    use super::{Mechanical, parse};

    fn args(s: &[&str]) -> Vec<String> {
        s.iter().map(|x| x.to_string()).collect()
    }

    /// traces: U1:事後
    #[test]
    fn create_reads_owner_and_kind() {
        let p = parse(&args(&[
            "create", "a.json", "--owner", "o", "--kind", "k", "--json",
        ]))
        .unwrap();
        assert_eq!(
            p.command,
            Mechanical::Create {
                path: "a.json".into(),
                owner: "o".into(),
                kind: "k".into(),
                value: serde_json::json!({})
            }
        );
    }

    /// traces: U5:事後
    #[test]
    fn create_with_schema_makes_an_instance() {
        let p = parse(&args(&[
            "create",
            "a.json",
            "--schema",
            "urn:semantic-agent-schema:a:b:v1",
        ]))
        .unwrap();
        assert!(matches!(p.command, Mechanical::CreateInstance { .. }));
    }

    /// traces: U25:事後
    #[test]
    fn record_summary_reads_the_id() {
        let p = parse(&args(&["record", "--summary", "--id", "X", "--json"])).unwrap();
        assert_eq!(
            (p.command, p.json),
            (
                Mechanical::Summarize {
                    id: Some("X".into())
                },
                true
            )
        );
    }

    /// traces: U1:失敗時
    #[test]
    fn unknown_flag_is_misuse() {
        assert!(parse(&args(&["validate", "a.json", "--fix"])).is_err());
    }
}
