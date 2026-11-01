//! JSON Patch（RFC 6902）を当てる。操作は add ・ remove ・ replace ・ move ・ copy ・ test で、
//! 欄は op ・ path と、op に応じて value か from。どれか1つでも当てられなければ、元の文書を変えずに理由を返す。

use serde_json::Value;

/// patch（操作の並び）を document の写しに当てた結果を返す。
///
/// # Errors
/// patch が操作の並びでない ・ 当てられない操作がある ・ test が外れたとき。理由の先頭に操作の位置（/0 など）を付ける。
pub fn apply(document: &Value, patch: &Value) -> Result<Value, String> {
    let ops = patch.as_array().ok_or("JSON Patch は操作の配列で渡す")?;
    let mut doc = document.clone();
    for (i, op) in ops.iter().enumerate() {
        apply_one(&mut doc, op).map_err(|e| format!("/{i}：{e}"))?;
    }
    Ok(doc)
}

fn field<'a>(op: &'a Value, name: &str) -> Result<&'a Value, String> {
    op.get(name).ok_or_else(|| format!("{name} が無い"))
}

fn pointer(op: &Value, name: &str) -> Result<Vec<String>, String> {
    let text = field(op, name)?
        .as_str()
        .ok_or_else(|| format!("{name} が文字列でない"))?;
    tokens(text)
}

fn tokens(text: &str) -> Result<Vec<String>, String> {
    if text.is_empty() {
        return Ok(Vec::new());
    }
    let rest = text
        .strip_prefix('/')
        .ok_or_else(|| format!("{text} は / で始まる JSON Pointer でない"))?;
    Ok(rest
        .split('/')
        .map(|t| t.replace("~1", "/").replace("~0", "~"))
        .collect())
}

fn apply_one(doc: &mut Value, op: &Value) -> Result<(), String> {
    let name = field(op, "op")?.as_str().ok_or("op が文字列でない")?;
    let path = pointer(op, "path")?;
    match name {
        "add" => add(doc, &path, field(op, "value")?.clone()),
        "remove" => remove(doc, &path).map(drop),
        "replace" => {
            let value = field(op, "value")?.clone();
            *get_mut(doc, &path)? = value;
            Ok(())
        }
        "move" => {
            let from = pointer(op, "from")?;
            if path.len() > from.len() && path.starts_with(&from) {
                return Err("from の下へは move できない".to_string());
            }
            let value = remove(doc, &from)?;
            add(doc, &path, value)
        }
        "copy" => {
            let from = pointer(op, "from")?;
            let value = get_mut(doc, &from)?.clone();
            add(doc, &path, value)
        }
        "test" => {
            let want = field(op, "value")?;
            let got = get_mut(doc, &path)?;
            if got == want {
                Ok(())
            } else {
                Err(format!("test が外れた（いまの値：{got}）"))
            }
        }
        other => Err(format!("知らない op：{other}")),
    }
}

fn index(token: &str, len: usize, allow_end: bool) -> Result<usize, String> {
    if allow_end && token == "-" {
        return Ok(len);
    }
    let ok = !token.is_empty()
        && token.bytes().all(|b| b.is_ascii_digit())
        && (token == "0" || !token.starts_with('0'));
    let i: usize = if ok {
        token
            .parse()
            .map_err(|_| format!("{token} は配列の位置でない"))?
    } else {
        return Err(format!("{token} は配列の位置でない"));
    };
    let max = if allow_end {
        len
    } else {
        len.saturating_sub(1)
    };
    if i > max || (!allow_end && len == 0) {
        return Err(format!("位置 {i} は配列の外"));
    }
    Ok(i)
}

fn get_mut<'a>(doc: &'a mut Value, path: &[String]) -> Result<&'a mut Value, String> {
    let mut at = doc;
    for t in path {
        at = match at {
            Value::Object(m) => m.get_mut(t).ok_or_else(|| format!("{t} が無い"))?,
            Value::Array(a) => {
                let i = index(t, a.len(), false)?;
                &mut a[i]
            }
            _ => return Err(format!("{t} の親セッションがオブジェクトでも配列でもない")),
        };
    }
    Ok(at)
}

fn add(doc: &mut Value, path: &[String], value: Value) -> Result<(), String> {
    let Some((last, parent)) = path.split_last() else {
        *doc = value;
        return Ok(());
    };
    match get_mut(doc, parent)? {
        Value::Object(m) => {
            m.insert(last.clone(), value);
            Ok(())
        }
        Value::Array(a) => {
            let i = index(last, a.len(), true)?;
            a.insert(i, value);
            Ok(())
        }
        _ => Err(format!(
            "{last} の親セッションがオブジェクトでも配列でもない"
        )),
    }
}

fn remove(doc: &mut Value, path: &[String]) -> Result<Value, String> {
    let (last, parent) = path.split_last().ok_or("文書の根は remove できない")?;
    match get_mut(doc, parent)? {
        // 残る欄の並びを変えない（swap_remove にしない）
        Value::Object(m) => m.shift_remove(last).ok_or_else(|| format!("{last} が無い")),
        Value::Array(a) => {
            let i = index(last, a.len(), false)?;
            Ok(a.remove(i))
        }
        _ => Err(format!(
            "{last} の親セッションがオブジェクトでも配列でもない"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::apply;
    use serde_json::json;

    #[test]
    fn rfc6902_operations() {
        let doc = json!({"a": {"b": 1}, "c": [1, 2], "d~/e": 0});
        let out = apply(
            &doc,
            &json!([
                {"op": "add", "path": "/c/-", "value": 3},
                {"op": "add", "path": "/c/0", "value": 0},
                {"op": "replace", "path": "/a/b", "value": 2},
                {"op": "remove", "path": "/d~0~1e"},
                {"op": "copy", "from": "/a", "path": "/f"},
                {"op": "move", "from": "/f/b", "path": "/g"},
                {"op": "test", "path": "/g", "value": 2}
            ]),
        )
        .unwrap();
        assert_eq!(
            out,
            json!({"a": {"b": 2}, "c": [0, 1, 2, 3], "f": {}, "g": 2})
        );
    }

    #[test]
    fn failure_leaves_document_and_names_the_operation() {
        let doc = json!({"a": 1});
        for (patch, at) in [
            (json!([{"op": "test", "path": "/a", "value": 2}]), "/0"),
            (
                json!([{"op": "add", "path": "/b", "value": 1}, {"op": "remove", "path": "/x"}]),
                "/1",
            ),
            (json!([{"op": "replace", "path": "/a"}]), "/0"),
            (json!([{"op": "add", "path": "/a/b/c", "value": 1}]), "/0"),
            (json!([{"op": "move", "from": "/a", "path": "/a/b"}]), "/0"),
            (json!([{"op": "frob", "path": "/a"}]), "/0"),
        ] {
            let e = apply(&doc, &patch).unwrap_err();
            assert!(e.starts_with(at), "{e}");
        }
        assert!(apply(&doc, &json!({"op": "add"})).is_err());
        assert_eq!(doc, json!({"a": 1}));
    }

    #[test]
    fn keys_keep_their_order() {
        let doc: serde_json::Value = serde_json::from_str(r#"{"z": 1, "a": 2, "m": 3}"#).unwrap();
        let out = apply(
            &doc,
            &json!([{"op": "remove", "path": "/z"}, {"op": "replace", "path": "/a", "value": 0}, {"op": "add", "path": "/b", "value": 4}]),
        )
        .unwrap();
        let keys: Vec<&String> = out.as_object().unwrap().keys().collect();
        assert_eq!(keys, ["a", "m", "b"]);
    }

    #[test]
    fn array_positions_are_strict() {
        let doc = json!({"c": [1]});
        assert!(
            apply(
                &doc,
                &json!([{"op": "replace", "path": "/c/01", "value": 0}])
            )
            .is_err()
        );
        assert!(
            apply(
                &doc,
                &json!([{"op": "replace", "path": "/c/1", "value": 0}])
            )
            .is_err()
        );
        assert!(apply(&doc, &json!([{"op": "add", "path": "/c/1", "value": 0}])).is_ok());
        assert!(apply(&doc, &json!([{"op": "remove", "path": "/c/-"}])).is_err());
    }
}
