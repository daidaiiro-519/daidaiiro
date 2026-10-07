//! ボードに固有のツール（ExtraTools）。転写した schema-driven のツールの一覧のあとに足す。
//! 引数の検査は基盤の Toolbox が行ってから call を呼ぶ。

use crate::{inspect, migrate, serve};
use schema_driven_adapters::inbound::tools::{ExtraTools, ToolDef};
use serde_json::{json, Map, Value};
use std::fs;
use std::path::{Component, Path, PathBuf};

pub struct BoardTools {
    /// ブレストボードの Skill のディレクトリ（references/ を持つ）
    skill_root: String,
}

impl BoardTools {
    pub fn new(skill_root: String) -> Self {
        Self { skill_root }
    }

    /// board.json の置き場所から、board.schema.json への相対パス。
    fn schema_from(&self, board_dir: &Path) -> Result<String, String> {
        let schema =
            fs::canonicalize(Path::new(&self.skill_root).join("references/board.schema.json"))
                .map_err(|error| format!("board.schema.json が無い ── {error}"))?;
        let from = fs::canonicalize(board_dir)
            .map_err(|error| format!("{}: 無い ── {error}", board_dir.display()))?;
        Ok(relative(&from, &schema))
    }
}

/// from（ディレクトリ）から to（ファイル）への相対パス。
fn relative(from: &Path, to: &Path) -> String {
    let parts = |path: &Path| -> Vec<String> {
        path.components()
            .filter_map(|component| match component {
                Component::Normal(part) => Some(part.to_string_lossy().into_owned()),
                _ => None,
            })
            .collect()
    };
    let (from_parts, to_parts) = (parts(from), parts(to));
    let common = from_parts
        .iter()
        .zip(&to_parts)
        .take_while(|(left, right)| left == right)
        .count();
    let mut out: Vec<String> =
        std::iter::repeat_n("..".to_owned(), from_parts.len() - common).collect();
    out.extend(to_parts[common..].iter().cloned());
    out.join("/")
}

fn failed(reason: &str, detail: impl Into<String>) -> (i32, Value) {
    (
        1,
        json!({"ok": false, "reason": reason, "detail": detail.into()}),
    )
}

const TOOLS: &[ToolDef] = &[
    ToolDef {
        name: "init",
        description: "ボードの置き場所（<dir>/<name>/）と、board.schema.json に従う board.json の雛形と answers/ を作り、索引（<dir>/README.md）に1行足す。既に在る名前は作り直さない",
        args: &[("name", "ボードの名前（小文字と -）", true), ("title", "ボードが決めることの題", true), ("dir", "ボードを置くディレクトリ。無ければ .brainstorming-board", false)],
    },
    ToolDef {
        name: "inspect",
        description: "ボードを出す前に、機械で見られる決まりを検査する（開いている論点は5件まで ・ いま見る論点 ・ 完成イメージ ・ 宣言されていない依存 ・ 試す相手 ・ 1文の長さ）。形は基盤の check が見る",
        args: &[("path", "board.json のパス", true)],
    },
    ToolDef {
        name: "migrate",
        description: "古い形（論点の18の欄）の board.json を、新しい形へ写す。古いファイルは board.json.v1 として残す",
        args: &[("path", "board.json のパス", true)],
    },
    ToolDef {
        name: "serve",
        description: "ボードのディレクトリを配り、押された回答を answers/<回答の id>.json（answer.schema.json のインスタンス）に書く。止められるまで返らない",
        args: &[("dir", "ボードのディレクトリ（board.json と board.html を持つ）", true), ("port", "待ち受ける番号。無ければ 8731", false)],
    },
];

impl ExtraTools for BoardTools {
    fn tools(&self) -> Vec<ToolDef> {
        TOOLS.to_vec()
    }

    fn call(&self, name: &str, args: &Map<String, Value>) -> (i32, Value) {
        let arg = |key: &str| args.get(key).and_then(Value::as_str).unwrap_or_default();
        match name {
            "init" => self.init(
                arg("name"),
                arg("title"),
                if arg("dir").is_empty() {
                    ".brainstorming-board"
                } else {
                    arg("dir")
                },
            ),
            "inspect" => {
                let text = match fs::read_to_string(arg("path")) {
                    Ok(text) => text,
                    Err(error) => return failed("読めない", format!("{}: {error}", arg("path"))),
                };
                let board: Value = match serde_json::from_str(&text) {
                    Ok(board) => board,
                    Err(error) => return failed("JSON として読めない", error.to_string()),
                };
                let findings = inspect::inspect(&board);
                (
                    if findings.is_empty() { 0 } else { 1 },
                    json!({"ok": findings.is_empty(), "findings": findings}),
                )
            }
            "migrate" => {
                let path = PathBuf::from(arg("path"));
                let schema = match path.parent().map(|dir| self.schema_from(dir)) {
                    Some(Ok(schema)) => schema,
                    Some(Err(error)) => return failed("読めない", error),
                    None => return failed("読めない", "board.json の置き場所が無い"),
                };
                match migrate::migrate(&path, &schema) {
                    Ok(out) => (0, out),
                    Err(error) => failed("写せない", error),
                }
            }
            "serve" => {
                let port = arg("port").parse().unwrap_or(serve::DEFAULT_PORT);
                match serve::run(arg("dir"), port) {
                    Ok(()) => (0, json!({"ok": true})),
                    Err(error) => failed("配れない", error),
                }
            }
            other => failed("知らないツール", other),
        }
    }
}

impl BoardTools {
    fn init(&self, name: &str, title: &str, dir: &str) -> (i32, Value) {
        let valid = !name.is_empty()
            && name.chars().all(|character| {
                character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
            });
        if !valid {
            return failed(
                "名前が使えない",
                format!("{name}（小文字 ・ 数字 ・ - だけ）"),
            );
        }
        let board_dir = Path::new(dir).join(name);
        if board_dir.join("board.json").exists() {
            return failed(
                "既に在る",
                format!("{} ── 作り直すと、書いた論点が消える", board_dir.display()),
            );
        }
        if let Err(error) = fs::create_dir_all(board_dir.join("answers")) {
            return failed("作れない", error.to_string());
        }
        let schema = match self.schema_from(&board_dir) {
            Ok(schema) => schema,
            Err(error) => return failed("読めない", error),
        };
        let board = json!({
            "$schema": schema, "kind": "board", "id": name, "title": title, "round": 1, "queue": [],
            "topics": [{"id": "Q1", "name": "論点の名前", "status": "waiting", "question": "何を決めるか",
                        "answer": {"text": "前提が片付いたら出す"}}]
        });
        if let Err(error) = fs::write(
            board_dir.join("board.json"),
            serde_json::to_string_pretty(&board).unwrap_or_default() + "\n",
        ) {
            return failed("書けない", error.to_string());
        }
        let index = Path::new(dir).join("README.md");
        let row = format!("| `{name}/` | {title} | （未発行） | （未複製） |\n");
        let current = fs::read_to_string(&index).unwrap_or_default();
        let next = if current.ends_with('\n') || current.is_empty() {
            current + &row
        } else {
            current + "\n" + &row
        };
        if let Err(error) = fs::write(&index, next) {
            return failed("索引に書けない", error.to_string());
        }
        (
            0,
            json!({"ok": true, "path": board_dir.join("board.json").display().to_string(),
                   "next": "board.json に論点を書き、inspect と check を通してから render で描画する"}),
        )
    }
}
