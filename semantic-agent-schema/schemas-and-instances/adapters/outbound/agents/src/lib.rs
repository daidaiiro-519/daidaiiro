//! ポート ReadMeaning を、エージェントの接続の定義（agent-connector.schema.json）で実装する。
//!
//! - argv と stdin の {prompt.path} ・ {prompt.text} ・ {answer_schema.path} ・ {answer_schema.text} ・ {tool_ref} を埋めて起動する
//!   （シェルを通さない）。1回の走査で埋めるので、埋めた中身に {名前} があっても置き換えない。
//!   {prompt.path} ・ {answer_schema.path} を使う定義のときだけ、作業用のフォルダにファイルを置き、終われば消す
//! - 起動する前に、親セッション ID の環境変数（callers.json の表）をすべて外す
//! - 出力（1つの JSON か、1行1つの JSON）から、答えと子セッションの ID を JSON Pointer で取り出す
//! - 時間を過ぎたら止めて時間切れにする（標準入力への書き込みも時間の中で別のスレッドがする）
//! - 同じ子セッションへ同時に送らない。子セッションごとのファイルに OS の鍵をかけるので、sas が止められても鍵は残らない

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use validation_application::ports::{ReadMeaning, Reply};

/// 接続の定義で動くエージェント。
#[derive(Debug, Clone)]
pub struct Connector {
    def: Value,
    unset_env: Vec<String>,
    work_dir: PathBuf,
}

impl Connector {
    /// 接続の定義（JSON）と、起動の前に外す環境変数の名前と、プロンプトと鍵を置くフォルダから作る。
    #[must_use]
    pub const fn new(def: Value, unset_env: Vec<String>, work_dir: PathBuf) -> Self {
        Self {
            def,
            unset_env,
            work_dir,
        }
    }

    fn text(&self, p: &str) -> Option<&str> {
        self.def.pointer(p).and_then(Value::as_str)
    }

    fn time_limit(&self) -> Duration {
        Duration::from_secs(
            self.def
                .get("time_limit_s")
                .and_then(Value::as_u64)
                .unwrap_or(600),
        )
    }

    fn codes(&self, kind: &str) -> Vec<i64> {
        self.def
            .pointer(&format!("/exit_codes/{kind}"))
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(Value::as_i64).collect())
            .unwrap_or_default()
    }

    /// 内容の指紋で名前を付けて、作業用のフォルダに置く。手放すと消える。
    fn keep(&self, kind: &str, text: &str) -> Result<Kept, String> {
        let dir = self.work_dir.join("prompts");
        std::fs::create_dir_all(&dir).map_err(|e| format!("{} を作れない：{e}", dir.display()))?;
        let hash: String = Sha256::digest(text.as_bytes())
            .iter()
            .take(8)
            .map(|b| format!("{b:02x}"))
            .collect();
        let path = dir.join(format!("{kind}-{hash}.txt"));
        std::fs::write(&path, text).map_err(|e| format!("{} を書けない：{e}", path.display()))?;
        Ok(Kept(path))
    }

    fn run(&self, command: &str, tool_ref: Option<&str>, prompt: &str, schema: &Value) -> Reply {
        let Some(cmd) = self.def.pointer(&format!("/commands/{command}")) else {
            return Reply::Failed(format!("接続の定義に commands.{command} が無い"));
        };
        let schema_text = schema.to_string();
        let uses = |name: &str| cmd.to_string().contains(name);
        let mut kept = Vec::new();
        let mut path_of = |used: bool, kind: &str, text: &str| -> Result<String, String> {
            if !used {
                return Ok(String::new());
            }
            let k = self.keep(kind, text)?;
            let p = k.0.to_string_lossy().into_owned();
            kept.push(k);
            Ok(p)
        };
        let (prompt_path, schema_path) = match (
            path_of(uses("{prompt.path}"), "prompt", prompt),
            path_of(uses("{answer_schema.path}"), "answer-schema", &schema_text),
        ) {
            (Ok(p), Ok(s)) => (p, s),
            (Err(e), _) | (_, Err(e)) => return Reply::Failed(e),
        };
        let vars = [
            ("{prompt.path}", prompt_path.as_str()),
            ("{prompt.text}", prompt),
            ("{answer_schema.path}", schema_path.as_str()),
            ("{answer_schema.text}", schema_text.as_str()),
            ("{tool_ref}", tool_ref.unwrap_or_default()),
        ];
        let fill = |s: &str| fill_once(s, &vars);
        let argv: Vec<String> = cmd
            .get("argv")
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(Value::as_str).map(fill).collect())
            .unwrap_or_default();
        let Some((program, args)) = argv.split_first() else {
            return Reply::Failed("argv が空".to_string());
        };
        let mut c = Command::new(program);
        c.args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        // 子セッションの中で打たれたものが、親セッションの操作に見えないように
        for name in &self.unset_env {
            c.env_remove(name);
        }
        let mut child = match c.spawn() {
            Ok(ch) => ch,
            Err(e) => return Reply::Failed(format!("{program} を起動できない：{e}")),
        };
        let started = Instant::now();
        // 標準入力も別のスレッドで書く（子セッションが読まずにパイプが詰まっても、時間切れで止められるように）
        let input = cmd
            .get("stdin")
            .and_then(Value::as_str)
            .map(fill)
            .unwrap_or_default();
        if let Some(mut stdin) = child.stdin.take() {
            std::thread::spawn(move || {
                let _ = stdin.write_all(input.as_bytes());
            });
        }
        // 出力は別のスレッドで読み続ける（大きな出力でパイプが詰まって子セッションが止まらないように）
        let reader = |pipe: Option<Box<dyn Read + Send>>| {
            std::thread::spawn(move || {
                let mut s = String::new();
                if let Some(mut p) = pipe {
                    let _ = p.read_to_string(&mut s);
                }
                s
            })
        };
        let out_thread = reader(
            child
                .stdout
                .take()
                .map(|p| Box::new(p) as Box<dyn Read + Send>),
        );
        let err_thread = reader(
            child
                .stderr
                .take()
                .map(|p| Box::new(p) as Box<dyn Read + Send>),
        );
        let status = loop {
            match child.try_wait() {
                Ok(Some(s)) => break s,
                Ok(None) if started.elapsed() > self.time_limit() => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Reply::Timeout { tool_ref: None };
                }
                Ok(None) => std::thread::sleep(Duration::from_millis(20)),
                Err(e) => return Reply::Failed(format!("{program} を待てない：{e}")),
            }
        };
        let out = out_thread.join().unwrap_or_default();
        let err = err_thread.join().unwrap_or_default();
        let code = i64::from(status.code().unwrap_or(-1));
        let tool_ref_found = self.extract("tool_ref", &out).map(|v| match v {
            Value::String(s) => s,
            other => other.to_string(),
        });
        if self.codes("timeout").contains(&code) {
            return Reply::Timeout {
                tool_ref: tool_ref_found,
            };
        }
        if self.codes("busy").contains(&code) {
            return Reply::Busy;
        }
        if !status.success() {
            // 記録に入るので、stderr は頭の 500 字だけにする
            let head: String = err.trim().chars().take(500).collect();
            return Reply::Failed(format!("{program} が終了コード {code} で終わった：{head}"));
        }
        let Some(tool_ref) = tool_ref_found else {
            return Reply::Failed(
                "出力に子セッションの ID が無い（接続の定義の tool_ref）".to_string(),
            );
        };
        let answer = match self.extract("answer", &out) {
            Some(Value::String(s)) => s,
            Some(other) => other.to_string(),
            None => String::new(),
        };
        Reply::Answered { tool_ref, answer }
    }

    /// 出力から、接続の定義の answer か tool_ref の所を取り出す。
    fn extract(&self, which: &str, out: &str) -> Option<Value> {
        let pointer = self.text(&format!("/{which}/json_pointer"))?;
        let event = self.text(&format!("/{which}/event_type"));
        if event.is_none()
            && let Ok(v) = serde_json::from_str::<Value>(out.trim())
        {
            return v.pointer(pointer).cloned();
        }
        // 合う行のうち最後のもの
        out.lines()
            .rev()
            .filter_map(|l| serde_json::from_str::<Value>(l.trim()).ok())
            .filter(|v| event.is_none_or(|e| v.get("type").and_then(Value::as_str) == Some(e)))
            .find_map(|v| v.pointer(pointer).cloned())
    }

    /// 子セッションごとの鍵。取れなければ（同じ子セッションが処理中なら）None。
    fn lock(&self, tool_ref: &str) -> Option<Lock> {
        let dir = self.work_dir.join("children");
        std::fs::create_dir_all(&dir).ok()?;
        let safe: String = tool_ref
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        let path = dir.join(format!("{safe}.lock"));
        let file = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(&path)
            .ok()?;
        // OS の鍵。持ち主のプロセスが終われば（止められても）外れる。ファイルは残すが、鍵の有無だけを見る
        file.try_lock().ok().map(|()| Lock(file))
    }
}

/// 子セッションの鍵。手放すと外れる。
struct Lock(#[allow(dead_code)] std::fs::File);

/// 作業用のフォルダに置いたファイル。手放すと消える。
struct Kept(PathBuf);

impl Drop for Kept {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// {名前} を1回の走査で埋める。埋めた中身はもう見ない。
fn fill_once(s: &str, vars: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    'scan: while !rest.is_empty() {
        if rest.starts_with('{') {
            for (name, value) in vars {
                if let Some(after) = rest.strip_prefix(name) {
                    out.push_str(value);
                    rest = after;
                    continue 'scan;
                }
            }
        }
        let ch = rest.chars().next().unwrap_or_default();
        out.push(ch);
        rest = &rest[ch.len_utf8()..];
    }
    out
}

impl ReadMeaning for Connector {
    fn tool(&self) -> String {
        self.text("/name").unwrap_or("unknown").to_string()
    }

    fn model(&self) -> Option<String> {
        self.text("/model").map(ToString::to_string)
    }

    fn launch(&self) -> Value {
        json!({
            "connector": self.text("/name"),
            "version": self.text("/version"),
            "model": self.text("/model"),
        })
    }

    fn cache_ttl_s(&self) -> Option<u64> {
        self.def
            .pointer("/capabilities/cache_ttl_s")
            .and_then(Value::as_u64)
    }

    fn start(&self, prompt: &str, answer_schema: &Value) -> Reply {
        self.run("start", None, prompt, answer_schema)
    }

    fn resume(
        &self,
        tool_ref: &str,
        _launch: &Value,
        prompt: &str,
        answer_schema: &Value,
    ) -> Reply {
        if self
            .def
            .pointer("/capabilities/resume")
            .and_then(Value::as_bool)
            != Some(true)
        {
            return Reply::Failed(format!(
                "{} は子セッションを続けられない（capabilities.resume）",
                self.tool()
            ));
        }
        let Some(_lock) = self.lock(tool_ref) else {
            return Reply::Busy;
        };
        self.run("continue", Some(tool_ref), prompt, answer_schema)
    }
}
