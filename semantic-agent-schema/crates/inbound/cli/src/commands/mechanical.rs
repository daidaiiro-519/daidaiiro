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
    let (name, rest) = args
        .split_first()
        .ok_or("コマンドが無い（create ・ get ・ patch ・ validate）")?;
    let (path, flags) = rest
        .split_first()
        .ok_or_else(|| format!("{name} の対象が無い"))?;
    let allowed: &[&str] = match name.as_str() {
        "create" => &["--owner", "--kind"],
        "get" => &["--pointer"],
        "patch" => &["--patch"],
        "validate" => &[],
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
    let take = |key: &str| {
        values
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| v.clone())
    };
    let need = |key: &str| take(key).ok_or_else(|| format!("{name} には {key} が必要"));
    let path = path.clone();
    let command = match name.as_str() {
        "create" => Mechanical::Create {
            path,
            owner: need("--owner")?,
            kind: need("--kind")?,
        },
        "get" => Mechanical::Get {
            path,
            pointer: take("--pointer").unwrap_or_default(),
        },
        "patch" => {
            let ops = serde_json::from_str(&need("--patch")?)
                .map_err(|e| format!("--patch が JSON でない：{e}"))?;
            Mechanical::Patch { path, ops }
        }
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
                kind: "k".into()
            }
        );
    }

    /// traces: U1:失敗時
    #[test]
    fn unknown_flag_is_misuse() {
        assert!(parse(&args(&["validate", "a.json", "--fix"])).is_err());
    }
}
