//! 結合テストと端から端までのテストの補助。作業用のフォルダ、値の表を回す関数、実行ファイルの起動、試験用の小さなカタログを、ここ1か所に置く。
//! 表を回す関数が assert を持つので、各テストの本体は表を並べるだけにする。
#![allow(dead_code)]

use schema_driven_adapters::compose;
use serde_json::{json, Map, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// 試験の作業用のフォルダ。実行ごとに別の名前で作り、試験が終わると消す（失敗で打ち切られたときも消す）。
/// 実行ごとに名前を変えるので、別のセッションが同時に試験を走らせても、同じフォルダを取り合わない。
///
/// 表の引数と期待値では、`./` で始まる文字列を、このフォルダからの相対パスとして書く。
/// 呼ぶ前に `./` をこのフォルダの絶対パスへ置き換え、結果の中のこのフォルダの絶対パスは `.` へ戻して比べる。
pub struct Scratch(PathBuf);

impl Scratch {
    pub fn new(name: &str) -> Self {
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("sd-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        Scratch(dir)
    }

    /// このフォルダの絶対パス。
    pub fn root(&self) -> &Path {
        &self.0
    }

    /// `./…` をこのフォルダの絶対パスにする。`./` で始まらない文字列は、そのまま返す。
    pub fn path(&self, value: &str) -> String {
        match value.strip_prefix("./") {
            Some(rest) => self.0.join(rest).to_string_lossy().into_owned(),
            None => value.to_owned(),
        }
    }

    fn at(&self, rel: &str) -> PathBuf {
        self.0.join(rel.trim_start_matches("./"))
    }

    /// JSON を整形して書く。親のフォルダが無ければ作る。
    pub fn put(&self, rel: &str, value: &Value) {
        self.put_text(rel, &serde_json::to_string_pretty(value).unwrap());
    }

    /// 文字をそのまま書く。親のフォルダが無ければ作る。
    pub fn put_text(&self, rel: &str, text: &str) {
        let target = self.at(rel);
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::write(target, text).unwrap();
    }

    pub fn mkdir(&self, rel: &str) {
        fs::create_dir_all(self.at(rel)).unwrap();
    }

    pub fn read(&self, rel: &str) -> String {
        fs::read_to_string(self.at(rel)).unwrap()
    }

    pub fn read_json(&self, rel: &str) -> Value {
        serde_json::from_str(&self.read(rel)).unwrap()
    }

    pub fn exists(&self, rel: &str) -> bool {
        self.at(rel).exists()
    }

    pub fn remove(&self, rel: &str) {
        fs::remove_file(self.at(rel)).unwrap();
    }

    pub fn remove_dir(&self, rel: &str) {
        fs::remove_dir_all(self.at(rel)).unwrap();
    }

    pub fn set_readonly(&self, rel: &str, readonly: bool) {
        let path = self.at(rel);
        let mut permissions = fs::metadata(&path).unwrap().permissions();
        permissions.set_readonly(readonly);
        fs::set_permissions(&path, permissions).unwrap();
    }

    /// フォルダの下のファイルの名前と中身を、名前の順に返す（書いたかどうかを比べるため）。
    pub fn snapshot(&self, dirs: &[&str]) -> Vec<(String, Vec<u8>)> {
        let mut out = Vec::new();
        for dir in dirs {
            let mut list: Vec<PathBuf> = fs::read_dir(self.at(dir))
                .unwrap()
                .map(|entry| entry.unwrap().path())
                .collect();
            list.sort();
            for path in list {
                out.push((
                    path.to_string_lossy().into_owned(),
                    fs::read(&path).unwrap(),
                ));
            }
        }
        out
    }

    /// 結果の中のこのフォルダの絶対パスを `.` へ戻す。
    fn relative(&self, value: Value) -> Value {
        let root = serde_json::to_string(&self.0.to_string_lossy()).unwrap();
        let root = &root[1..root.len() - 1];
        serde_json::from_str(&value.to_string().replace(root, ".")).unwrap()
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// 結果の欄の値の確かめ方。
#[derive(Debug, Clone)]
pub enum Is {
    /// 値が等しい
    Eq(Value),
    /// 文字の値が、この文字を含む
    Has(&'static str),
    /// 文字の値が、この文字を含まない
    Lacks(&'static str),
    /// 配列の長さ
    Len(usize),
    /// 欄が無い
    Absent,
}

pub fn eq(value: impl Into<Value>) -> Is {
    Is::Eq(value.into())
}

pub fn has(text: &'static str) -> Is {
    Is::Has(text)
}

pub fn lacks(text: &'static str) -> Is {
    Is::Lacks(text)
}

pub fn len(count: usize) -> Is {
    Is::Len(count)
}

/// JSON の中の pointer の欄を、is で確かめる。what は失敗したときに出す説明。
pub fn check(what: &str, value: &Value, pointer: &str, is: &Is) {
    let found = value.pointer(pointer);
    match is {
        Is::Eq(expected) => assert_eq!(found, Some(expected), "{what} {pointer}：{value}"),
        Is::Has(text) => assert!(
            found
                .and_then(Value::as_str)
                .is_some_and(|got| got.contains(text)),
            "{what} {pointer} に「{text}」が無い：{value}"
        ),
        Is::Lacks(text) => assert!(
            found
                .and_then(Value::as_str)
                .is_some_and(|got| !got.contains(text)),
            "{what} {pointer} に「{text}」がある：{value}"
        ),
        Is::Len(count) => assert_eq!(
            found.and_then(Value::as_array).map(Vec::len),
            Some(*count),
            "{what} {pointer} の長さ：{value}"
        ),
        Is::Absent => assert_eq!(found, None, "{what} {pointer} がある：{value}"),
    }
}

/// 表の1行：ツールの名前 ・ 引数 ・ 期待する終了コード ・ 欄ごとの確かめ方。
pub struct Call<'row> {
    pub tool: &'row str,
    pub args: &'row [(&'row str, &'row str)],
    pub code: i32,
    pub expect: &'row [(&'row str, Is)],
}

/// 表の1行を作る。
pub fn call<'row>(
    tool: &'row str,
    args: &'row [(&'row str, &'row str)],
    code: i32,
    expect: &'row [(&'row str, Is)],
) -> Call<'row> {
    Call {
        tool,
        args,
        code,
        expect,
    }
}

/// 公開 API の振り分け（起動時の結合が組んだツールの一覧の dispatch）を1回呼ぶ。
/// skill は `--skill` に渡す Skill のフォルダ（`./…`）。結果の中の作業用のフォルダのパスは `.` へ戻す。
pub fn dispatch(
    scratch: &Scratch,
    skill: Option<&str>,
    tool: &str,
    args: &[(&str, &str)],
) -> (i32, Value) {
    let skill = skill.map(|dir| scratch.path(dir));
    let map: Map<String, Value> = args
        .iter()
        .map(|(name, value)| ((*name).to_owned(), Value::String(scratch.path(value))))
        .collect();
    let (code, out) = compose::toolbox(skill.as_deref()).dispatch(tool, &map);
    (code, scratch.relative(out))
}

/// 表を上から順に、同じ作業用のフォルダで呼び、行ごとに終了コードと欄を確かめる。
pub fn run(scratch: &Scratch, skill: Option<&str>, rows: &[Call]) {
    for (index, row) in rows.iter().enumerate() {
        let (code, out) = dispatch(scratch, skill, row.tool, row.args);
        let what = format!("{} 行目（{}）", index + 1, row.tool);
        assert_eq!(code, row.code, "{what} の終了コード：{out}");
        for (pointer, is) in row.expect {
            check(&what, &out, pointer, is);
        }
    }
}

/// 値を、欄ごとの確かめ方の表で確かめる。
pub fn expect(what: &str, value: &Value, rows: &[(&str, Is)]) {
    for (pointer, is) in rows {
        check(what, value, pointer, is);
    }
}

/// 文字を、含むべき文字の表で確かめる。
pub fn contains_all(what: &str, text: &str, parts: &[&str]) {
    for part in parts {
        assert!(text.contains(part), "{what} に「{part}」が無い：\n{text}");
    }
}

/// 文字を、含んではいけない文字の表で確かめる。
pub fn contains_none(what: &str, text: &str, parts: &[&str]) {
    for part in parts {
        assert!(!text.contains(part), "{what} に「{part}」がある：\n{text}");
    }
}

/// CLI の実行ファイルを、作業用のフォルダをいまのフォルダにして起動し、終了コードと標準出力の JSON を返す。
/// 引数の `./…` は置き換えない（相対パスを、起動した場所から解決することを確かめるため）。
pub fn cli(scratch: &Scratch, args: &[&str]) -> (i32, Value) {
    let output = Command::new(env!("CARGO_BIN_EXE_schema-driven"))
        .current_dir(scratch.root())
        .args(args)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let out =
        serde_json::from_str(&stdout).unwrap_or_else(|_| panic!("出力が JSON でない：{stdout}"));
    (output.status.code().unwrap_or(-1), out)
}

/// MCP の実行ファイルを、作業用のフォルダをいまのフォルダにして起動し、rmcp のクライアントで1回の接続の中で呼ぶ。
/// 先にツールの一覧を取り、続けて calls（ツールの名前と引数）を順に呼ぶ。返すのは
/// `{"version": serverInfo の版, "tools": [名前…], "calls": [{"isError": …, "body": 結果の文字を JSON として読んだもの}…]}`。
pub fn mcp(scratch: &Scratch, calls: &[(&str, Value)]) -> Value {
    use rmcp::model::CallToolRequestParams;
    use rmcp::transport::TokioChildProcess;
    use rmcp::ServiceExt;

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        let mut command = tokio::process::Command::new(env!("CARGO_BIN_EXE_schema-driven-mcp"));
        command.current_dir(scratch.root());
        let transport = TokioChildProcess::new(command).unwrap();
        let client = ().serve(transport).await.unwrap();
        let version = client
            .peer_info()
            .and_then(|info| info.server_info.clone())
            .map(|server| server.version)
            .unwrap_or_default();
        let tools: Vec<Value> = client
            .list_all_tools()
            .await
            .unwrap()
            .iter()
            .map(|tool| Value::String(tool.name.to_string()))
            .collect();
        let mut results = Vec::new();
        for (name, args) in calls {
            let params = CallToolRequestParams::new(name.to_string())
                .with_arguments(args.as_object().cloned().unwrap_or_default());
            let result = client.call_tool(params).await.unwrap();
            let text = result
                .content
                .first()
                .and_then(|block| block.as_text())
                .map(|text| text.text.clone())
                .unwrap_or_default();
            results.push(serde_json::json!({
                "isError": result.is_error,
                "body": serde_json::from_str::<Value>(&text).unwrap_or(Value::String(text)),
            }));
        }
        client.cancel().await.unwrap();
        serde_json::json!({"version": version, "tools": tools, "calls": results})
    })
}

/// 試験用の小さな部品のカタログ（部品の一覧のスキーマ）。部品は pill（text ・ tone）・ list（items）・ figure（svg）の3つ。
/// 部品の名前は、試験のこのカタログだけが持つ。
pub fn catalog_schema() -> Value {
    json!({
        "$schema": "schema-driven/catalog@v1", "$id": "example/catalog@v1",
        "$defs": {
            "pill": {"type": "object", "description": "短い分類", "required": ["text"],
                     "properties": {"text": {}, "tone": {"type": "object"}},
                     "x-states": {"default": "押せない"}},
            "list": {"type": "object", "description": "並び", "required": ["items"],
                     "properties": {"items": {"type": "array"}},
                     "x-states": {"default": "押せない"}},
            "figure": {"type": "object", "description": "図", "required": ["svg"],
                       "properties": {"svg": {"type": "string", "contentMediaType": "image/svg+xml"}},
                       "x-states": {"default": "押せない"}}
        }
    })
}

/// 試験用の小さなデザインシステム。文字 ink と背景 paper の組を宣言する。
pub fn design_system() -> Value {
    json!({
        "scope": ":root",
        "tokens": {"ink": "#111d1a", "paper": "#f7f8f7", "night": "#0e1614", "mist": "#e6eeeb"},
        "dark": {"ink": "{mist}", "paper": "{night}"},
        "contrast": [{"foreground": "ink", "background": "paper", "use": "text"}]
    })
}

/// 試験用の小さな部品の HTML。ページのテンプレートは head の lead を {{lead}} に置く。
pub const PARTS: &str = r#"<template id="page"><!doctype html><html><head><title>{{title}}</title><style>{{tokens}}{{style}}</style></head><body><h1>{{title}}</h1><p class="lead">{{lead}}</p>{{sections}}<script>{{script}}</script></body></html></template>
<template id="section"><section><h2>{{heading}}</h2>{{body}}</section></template>
<template id="pill"><span class="pill {{tone}}">{{text}}</span></template>
<template id="list"><ul>{{items}}</ul></template>
<template id="figure"><figure>{{svg}}</figure></template>"#;

/// Skill のフォルダ（`./…`）の assets/catalog/ に、試験用の小さなカタログを置く。
pub fn put_catalog(scratch: &Scratch, skill: &str) {
    let dir = format!("{}/assets/catalog", skill.trim_end_matches('/'));
    scratch.put(&format!("{dir}/catalog.schema.json"), &catalog_schema());
    scratch.put_text(&format!("{dir}/parts.html"), PARTS);
    scratch.put_text(
        &format!("{dir}/catalog.css"),
        ".pill{color:var(--ink);background:var(--paper)}",
    );
    scratch.put_text(
        &format!("{dir}/catalog.js"),
        "document.documentElement.dataset.ready = '1';",
    );
    scratch.put(&format!("{dir}/design-system.json"), &design_system());
}
