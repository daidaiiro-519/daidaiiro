//! 抽象の基盤。7つの契約だけを持ち、宣言の語（ユースケース ・ 集約 ・ シナリオ ・ ID）を1つも持たない。
use serde_json::{Map, Value};
use std::{fs, path::Path};

pub fn load(path: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(path).expect("読めない")).expect("JSON でない")
}

fn save(path: &Path, doc: &Value) {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).unwrap();
    }
    fs::write(path, serde_json::to_string_pretty(doc).unwrap() + "\n").unwrap();
}

fn skeleton(schema: &Value) -> Value {
    match schema["type"].as_str() {
        Some("object") => {
            let mut m = Map::new();
            if let (Some(req), Some(props)) = (schema["required"].as_array(), schema["properties"].as_object()) {
                for k in req.iter().filter_map(|k| k.as_str()) {
                    m.insert(k.into(), skeleton(&props[k]));
                }
            }
            Value::Object(m)
        }
        Some("array") => Value::Array(vec![]),
        _ => Value::String(String::new()),
    }
}

/// 作成の契約：スキーマから実体を作り、x-generates の場所へ置く。初期値は呼ぶ側が渡す。
pub fn create(root: &Path, schema: &Value, init: &[(&str, Value)]) -> std::path::PathBuf {
    let mut doc = skeleton(schema);
    let mut place = schema["x-generates"].as_str().expect("x-generates が無い").to_string();
    for (k, v) in init {
        doc[*k] = v.clone();
        if let Some(s) = v.as_str() {
            place = place.replace(&format!("{{{k}}}"), s);
        }
    }
    let path = root.join(place);
    save(&path, &doc);
    path
}

/// 取得の契約：JMESPath の式で値を取得する。
pub fn get(doc: &Value, expr: &str) -> Value {
    let e = jmespath::compile(expr).expect("式が誤っている");
    let data = jmespath::Variable::from_json(&doc.to_string()).unwrap();
    serde_json::to_value(&*e.search(data).unwrap()).unwrap()
}

/// 検証の契約：JSON Schema で検証し、違反を返す。
pub fn validate(schema: &Value, doc: &Value) -> Vec<String> {
    let v = jsonschema::validator_for(schema).expect("スキーマが誤っている");
    v.iter_errors(doc).map(|e| format!("{} : {}", e.instance_path(), e)).collect()
}

/// 更新の契約（削除の項目単位も同じ）：JSON Patch を適用し、検証を通過したときだけ書き込む。
pub fn update(path: &Path, schema: &Value, patch: Value) -> Result<(), Vec<String>> {
    let mut doc = load(path);
    let p: json_patch::Patch = serde_json::from_value(patch).expect("JSON Patch でない");
    json_patch::patch(&mut doc, &p).map_err(|e| vec![e.to_string()])?;
    let errs = validate(schema, &doc);
    if !errs.is_empty() {
        return Err(errs);
    }
    save(path, &doc);
    Ok(())
}

/// 削除の契約（実体単位）：ファイルを消す。
pub fn delete(path: &Path) {
    fs::remove_file(path).unwrap();
}

/// 案内の契約：項目ごとの x-prompt を返す。write は未記入の項目だけを返す。
pub fn guide(schema: &Value, doc: &Value, mode: &str) -> Vec<(String, String)> {
    let empty = |v: &Value| v.as_str() == Some("") || v.as_array().is_some_and(|a| a.is_empty());
    schema["properties"].as_object().unwrap().iter()
        .filter(|(k, _)| mode == "read" || doc.get(*k).map_or(true, empty))
        .map(|(k, p)| (format!("/{k}"), p["x-prompt"][mode].as_str().unwrap_or("").to_string()))
        .collect()
}

/// 描画の契約：title と x-view に従って Markdown へ描画する。
pub fn render(schema: &Value, doc: &Value) -> String {
    let props = schema["properties"].as_object().unwrap();
    let mut out = String::new();
    for (k, p) in props {
        let (title, v) = (p["title"].as_str().unwrap_or(k), &doc[k]);
        match p["x-view"].as_str().unwrap_or("text") {
            "hidden" => {}
            "heading" => out += &format!("# {}（{}）\n\n", v.as_str().unwrap_or(""), schema["title"].as_str().unwrap_or("")),
            "list" => {
                out += &format!("## {title}\n\n");
                for x in v.as_array().into_iter().flatten() {
                    out += &format!("- {}\n", x.as_str().unwrap_or(""));
                }
                out += "\n";
            }
            "table" => {
                let cols: Vec<_> = p["items"]["properties"].as_object().unwrap().iter().collect();
                out += &format!("## {title}\n\n| {} |\n|{}\n", cols.iter().map(|(c, cp)| cp["title"].as_str().unwrap_or(c)).collect::<Vec<_>>().join(" | "), "---|".repeat(cols.len()));
                for row in v.as_array().into_iter().flatten() {
                    out += &format!("| {} |\n", cols.iter().map(|(c, _)| row[*c].as_str().unwrap_or("")).collect::<Vec<_>>().join(" | "));
                }
                out += "\n";
            }
            _ => out += &format!("## {title}\n\n{}\n\n", v.as_str().unwrap_or("")),
        }
    }
    out
}
