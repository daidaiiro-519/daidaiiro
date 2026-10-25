//! JSON Patch（test ・ add ・ replace ・ remove）。検査①：適用する前に入力を見て、外れたら状態に触れずに断る

use serde_json::Value;

/// 断った理由（どの op の、どの場所か）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    pointer: String,
    reason: String,
}

impl Refusal {
    /// 断った場所（JSON Pointer）と理由から作る
    pub fn new(pointer: &str, reason: &str) -> Self {
        Refusal {
            pointer: pointer.to_string(),
            reason: reason.to_string(),
        }
    }

    /// 断った場所（JSON Pointer）
    pub fn pointer(&self) -> &str {
        &self.pointer
    }

    /// 理由
    pub fn reason(&self) -> &str {
        &self.reason
    }
}

fn refuse(pointer: &str, reason: &str) -> Refusal {
    Refusal::new(pointer, reason)
}

fn unescape(token: &str) -> String {
    token.replace("~1", "/").replace("~0", "~")
}

/// 親の場所と最後の名前に分ける
fn split(path: &str) -> Option<(String, String)> {
    let i = path.rfind('/')?;
    Some((path[..i].to_string(), unescape(&path[i + 1..])))
}

/// RFC 6902 の test の一致：数は数として比べ、配列と object は中まで比べる
fn same(a: Option<&Value>, b: Option<&Value>) -> bool {
    match (a, b) {
        (Some(Value::Number(x)), Some(Value::Number(y))) => x.as_f64() == y.as_f64(),
        (Some(Value::Array(x)), Some(Value::Array(y))) => {
            x.len() == y.len() && x.iter().zip(y).all(|(p, q)| same(Some(p), Some(q)))
        }
        (Some(Value::Object(x)), Some(Value::Object(y))) => {
            x.len() == y.len() && x.iter().all(|(k, p)| same(Some(p), y.get(k)))
        }
        _ => a == b,
    }
}

/// 文書に JSON Patch を適用した写しを返す。元の文書は変えない。不変条件 I3（$schema の申告を変えさせない）
///
/// # Errors
/// 使えない op（move ・ copy など）・ 根の場所 ・ $id と $schema の書き換え ・ 無い場所 ・ test の不一致のとき
pub fn apply(doc: &Value, ops: &Value) -> Result<Value, Refusal> {
    let ops = ops
        .as_array()
        .ok_or_else(|| refuse("", "JSON Patch は op の配列で書く"))?;
    let mut out = doc.clone();
    for (n, op) in ops.iter().enumerate() {
        let at = format!("/{n}");
        let kind = op
            .get("op")
            .and_then(Value::as_str)
            .ok_or_else(|| refuse(&at, "op が無い"))?;
        let path = op
            .get("path")
            .and_then(Value::as_str)
            .ok_or_else(|| refuse(&at, "path が無い"))?;
        if !["test", "add", "replace", "remove"].contains(&kind) {
            return Err(refuse(
                &at,
                &format!("op「{kind}」は使えない（test ・ add ・ replace ・ remove だけ）"),
            ));
        }
        if path.is_empty() {
            return Err(refuse(&at, "文書の根は書き換えない"));
        }
        if kind != "test" && (path == "/$id" || path == "/$schema") {
            return Err(refuse(&at, "$id と $schema は書き換えない"));
        }
        let value = op.get("value");
        match kind {
            "test" => {
                if !same(out.pointer(path), value) {
                    return Err(refuse(&at, &format!("{path} の値が test と合わない")));
                }
            }
            "replace" => {
                let v = value.ok_or_else(|| refuse(&at, "value が無い"))?.clone();
                let slot = out
                    .pointer_mut(path)
                    .ok_or_else(|| refuse(&at, &format!("{path} が無い")))?;
                *slot = v;
            }
            "remove" => {
                let (parent, last) = split(path).ok_or_else(|| refuse(&at, "path の形が誤り"))?;
                let p = out
                    .pointer_mut(&parent)
                    .ok_or_else(|| refuse(&at, &format!("{parent} が無い")))?;
                let removed = match p {
                    Value::Object(m) => m.remove(&last).is_some(),
                    Value::Array(a) => last
                        .parse::<usize>()
                        .ok()
                        .filter(|i| *i < a.len())
                        .map(|i| a.remove(i))
                        .is_some(),
                    _ => false,
                };
                if !removed {
                    return Err(refuse(&at, &format!("{path} が無い")));
                }
            }
            _ => {
                let v = value.ok_or_else(|| refuse(&at, "value が無い"))?.clone();
                let (parent, last) = split(path).ok_or_else(|| refuse(&at, "path の形が誤り"))?;
                let p = out
                    .pointer_mut(&parent)
                    .ok_or_else(|| refuse(&at, &format!("{parent} が無い")))?;
                match p {
                    Value::Object(m) => {
                        m.insert(last, v);
                    }
                    Value::Array(a) if last == "-" => a.push(v),
                    Value::Array(a) => {
                        let i = last
                            .parse::<usize>()
                            .ok()
                            .filter(|i| *i <= a.len())
                            .ok_or_else(|| refuse(&at, &format!("{path} の位置が配列の外")))?;
                        a.insert(i, v);
                    }
                    _ => return Err(refuse(&at, &format!("{parent} は入れ物でない"))),
                }
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::apply;
    use serde_json::json;

    fn doc() -> serde_json::Value {
        json!({"$id": "urn:semantic-agent-schema:a:b:v1", "why": "", "list": [1, 2]})
    }

    /// traces: U1:事後
    #[test]
    fn replace_changes_the_value() {
        let out = apply(
            &doc(),
            &json!([{"op": "replace", "path": "/why", "value": "理由"}]),
        )
        .unwrap();
        assert_eq!(out["why"], "理由");
    }

    /// traces: U1:事後
    #[test]
    fn add_to_array_end_appends() {
        let out = apply(
            &doc(),
            &json!([{"op": "add", "path": "/list/-", "value": 3}]),
        )
        .unwrap();
        assert_eq!(out["list"], json!([1, 2, 3]));
    }

    /// traces: U1:事後
    #[test]
    fn remove_deletes_the_member() {
        let out = apply(&doc(), &json!([{"op": "remove", "path": "/why"}])).unwrap();
        assert!(out.get("why").is_none());
    }

    /// traces: U1:失敗時
    #[test]
    fn test_mismatch_is_refused() {
        assert!(
            apply(
                &doc(),
                &json!([{"op": "test", "path": "/why", "value": "x"}])
            )
            .is_err()
        );
    }

    /// traces: U1:事後
    #[test]
    fn test_compares_numbers_as_numbers() {
        let d = serde_json::json!({"n": 1});
        assert!(apply(&d, &json!([{"op": "test", "path": "/n", "value": 1.0}])).is_ok());
    }

    /// traces: U1:失敗時
    #[test]
    fn move_is_refused() {
        assert!(
            apply(
                &doc(),
                &json!([{"op": "move", "from": "/why", "path": "/w"}])
            )
            .is_err()
        );
    }

    /// traces: U1:失敗時
    #[test]
    fn root_path_is_refused() {
        assert!(apply(&doc(), &json!([{"op": "replace", "path": "", "value": {}}])).is_err());
    }

    /// traces: U1:失敗時
    #[test]
    fn id_change_is_refused() {
        assert!(
            apply(
                &doc(),
                &json!([{"op": "replace", "path": "/$id", "value": "x"}])
            )
            .is_err()
        );
    }

    /// traces: U1:失敗時
    #[test]
    fn refusal_leaves_the_document_unchanged() {
        let d = doc();
        let _ = apply(
            &d,
            &json!([{"op": "replace", "path": "/why", "value": "a"}, {"op": "move", "from": "/a", "path": "/b"}]),
        );
        assert_eq!(d, doc());
    }
}
