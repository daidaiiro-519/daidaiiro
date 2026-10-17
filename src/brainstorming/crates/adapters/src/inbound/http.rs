//! serve の受け口。127.0.0.1 だけで待ち受け、描いたページを配り、回答の送信と AI のチャットを受ける。
//!
//! 守ること：
//! - 起動ごとに作る合言葉を URL で1回だけ受け、HttpOnly ・ SameSite=Strict の cookie へ移す。合言葉の無い要求は断る
//! - Host が 127.0.0.1:<番号> か localhost:<番号> でなければ断る（DNS リバインディング）
//! - 書き換える要求は POST だけで受け、Origin が同じでなければ断る
//! - AI の CLI はボード1枚につき同時に1つ（ボードのロックのファイルで、ほかの serve と CLI にも効かせる）

use brainstorming_core::application::chat::{self, Kind, Limits, Turn};
use brainstorming_core::application::{hold, status, Env, Skill};
use brainstorming_core::domain::agent::Agent;
use brainstorming_core::domain::board::{text, Layout};
use brainstorming_core::ports::{Agents, Clock, Files, Programs};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

/// 受ける本文の上限（バイト）。
const BODY_MAX: usize = 1 << 20;
/// 受けるヘッダーの上限（バイト）。
const HEAD_MAX: usize = 64 << 10;

/// 読んだ要求。
#[derive(Debug, Default, PartialEq)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub query: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

fn decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => out.push(b' '),
            b'%' if i + 2 < bytes.len() => {
                match text
                    .get(i + 1..i + 3)
                    .and_then(|hex| u8::from_str_radix(hex, 16).ok())
                    .ok_or(())
                {
                    Ok(byte) => {
                        out.push(byte);
                        i += 2;
                    }
                    Err(_) => out.push(b'%'),
                }
            }
            other => out.push(other),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// 要求を読む。形が違えば None。
pub fn read_request(stream: &mut impl Read) -> Option<Request> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).ok()?;
    let mut parts = line.split_whitespace();
    let method = parts.next()?.to_owned();
    let target = parts.next()?.to_owned();
    let mut headers = HashMap::new();
    let mut size = line.len();
    loop {
        let mut header = String::new();
        let n = reader.read_line(&mut header).ok()?;
        size += n;
        if n == 0 || size > HEAD_MAX {
            return None;
        }
        let header = header.trim_end();
        if header.is_empty() {
            break;
        }
        let (name, value) = header.split_once(':')?;
        headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_owned());
    }
    let length: usize = headers
        .get("content-length")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    if length > BODY_MAX {
        return None;
    }
    let mut body = vec![0u8; length];
    reader.read_exact(&mut body).ok()?;
    let (path, query) = target.split_once('?').unwrap_or((&target, ""));
    let query = query
        .split('&')
        .filter_map(|pair| pair.split_once('='))
        .map(|(k, v)| (decode(k), decode(v)))
        .collect();
    Some(Request {
        method,
        path: decode(path),
        query,
        headers,
        body,
    })
}

/// 応答。
#[derive(Debug, PartialEq)]
pub struct Response {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Response {
    fn json(status: u16, value: &Value) -> Self {
        Self {
            status,
            headers: vec![(
                "Content-Type".to_owned(),
                "application/json; charset=utf-8".to_owned(),
            )],
            body: value.to_string().into_bytes(),
        }
    }
    fn refuse(status: u16, reason: &str) -> Self {
        Self::json(status, &json!({"ok": false, "reason": reason}))
    }
    fn to_bytes(&self) -> Vec<u8> {
        let reason = match self.status {
            200 => "OK",
            303 => "See Other",
            400 => "Bad Request",
            403 => "Forbidden",
            404 => "Not Found",
            409 => "Conflict",
            _ => "Error",
        };
        let mut head = format!("HTTP/1.1 {} {reason}\r\n", self.status);
        for (name, value) in &self.headers {
            head.push_str(&format!("{name}: {value}\r\n"));
        }
        head.push_str(&format!(
            "Content-Length: {}\r\nConnection: close\r\n\r\n",
            self.body.len()
        ));
        let mut out = head.into_bytes();
        out.extend_from_slice(&self.body);
        out
    }
}

/// どの応答にも付けるヘッダー。
fn safe_headers() -> String {
    "X-Content-Type-Options: nosniff\r\nX-Frame-Options: DENY\r\nReferrer-Policy: no-referrer\r\nCache-Control: no-store\r\n".to_owned()
}

/// 要求を受けてよいか（Host ・ 合言葉 ・ Origin）。受けてよくなければ断る応答。
pub fn guard(request: &Request, port: u16, token: &str) -> Result<(), Response> {
    let hosts = [format!("127.0.0.1:{port}"), format!("localhost:{port}")];
    let host = request
        .headers
        .get("host")
        .map(String::as_str)
        .unwrap_or("");
    if !hosts.iter().any(|h| h == host) {
        return Err(Response::refuse(403, "Host が違う"));
    }
    let cookie = format!("bs{port}={token}");
    let has_cookie = request
        .headers
        .get("cookie")
        .is_some_and(|c| c.split(';').any(|part| part.trim() == cookie));
    if !has_cookie {
        return Err(Response::refuse(
            403,
            "serve が出した URL から開いてください",
        ));
    }
    if request.method == "POST" {
        let origin = request
            .headers
            .get("origin")
            .map(String::as_str)
            .unwrap_or("");
        if !hosts.iter().any(|h| origin == format!("http://{h}")) {
            return Err(Response::refuse(403, "Origin が違う"));
        }
        let kind = request
            .headers
            .get("content-type")
            .map(String::as_str)
            .unwrap_or("");
        if !kind.starts_with("application/json") {
            return Err(Response::refuse(400, "本文は JSON で送る"));
        }
    }
    Ok(())
}

/// 合言葉を受けたら cookie へ移して、合言葉の無い URL へ送る。
pub fn welcome(request: &Request, port: u16, token: &str) -> Option<Response> {
    let given = request.query.get("k")?;
    if given != token {
        return Some(Response::refuse(403, "合言葉が違う"));
    }
    let to = if request.path == "/" {
        "/dashboard.html".to_owned()
    } else {
        request.path.clone()
    };
    Some(Response {
        status: 303,
        headers: vec![
            ("Location".to_owned(), to),
            (
                "Set-Cookie".to_owned(),
                format!("bs{port}={token}; HttpOnly; SameSite=Strict; Path=/"),
            ),
        ],
        body: vec![],
    })
}

fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("json") => "application/json; charset=utf-8",
        Some("png") => "image/png",
        _ => "application/octet-stream",
    }
}

/// 配るファイルの場所。site の外を指すものは None。
pub fn static_path(site: &Path, path: &str) -> Option<PathBuf> {
    let rest = Path::new(path.trim_start_matches('/'));
    if rest
        .components()
        .any(|c| !matches!(c, Component::Normal(_)))
    {
        return None;
    }
    Some(site.join(rest))
}

/// serve の状態。
pub struct Server<F, P, C, A> {
    pub env: Env<F, P, C>,
    pub agents: A,
    pub skill: Skill,
    pub root: PathBuf,
    pub port: u16,
    pub token: String,
    pub limits: Limits,
    running: Mutex<HashMap<String, Arc<AtomicU8>>>,
}

impl<F, P, C, A> Server<F, P, C, A>
where
    F: Files + Send + Sync + 'static,
    P: Programs + Send + Sync + 'static,
    C: Clock + Send + Sync + 'static,
    A: Agents + Send + Sync + 'static,
{
    pub fn new(
        env: Env<F, P, C>,
        agents: A,
        skill: Skill,
        root: PathBuf,
        port: u16,
        token: String,
    ) -> Self {
        Self {
            env,
            agents,
            skill,
            root,
            port,
            token,
            limits: Limits::default(),
            running: Mutex::new(HashMap::new()),
        }
    }

    fn body(request: &Request) -> Result<Value, Response> {
        serde_json::from_slice(&request.body)
            .map_err(|_| Response::refuse(400, "本文を JSON として読めない"))
    }

    /// 流さない要求に答える。
    pub fn answer(&self, request: &Request) -> Response {
        if let Some(response) = welcome(request, self.port, &self.token) {
            return response;
        }
        if let Err(response) = guard(request, self.port, &self.token) {
            return response;
        }
        let ok = |data: Value| Response::json(200, &json!({"ok": true, "data": data}));
        let fail = |reason: String| Response::json(400, &json!({"ok": false, "reason": reason}));
        match (request.method.as_str(), request.path.as_str()) {
            ("GET", "/api/agents") => {
                let found: serde_json::Map<String, Value> = Agent::ALL
                    .iter()
                    .map(|a| (a.name().to_owned(), json!(self.agents.found(a.program()))))
                    .collect();
                ok(json!({"agents": found}))
            }
            ("GET", "/api/chat") => {
                let get = |k: &str| request.query.get(k).map(String::as_str).unwrap_or("");
                match chat::history(&self.env, &self.root, get("board"), get("agent")) {
                    Ok(data) => ok(data),
                    Err(reason) => fail(reason),
                }
            }
            ("GET", "/api/status") => ok(json!({"boards": status::boards(&self.env, &self.root)})),
            ("POST", "/api/hold") => match Self::body(request) {
                Ok(input) => match hold::set(
                    &self.env,
                    &self.skill,
                    &self.root,
                    text(&input, "board"),
                    input.get("hold").and_then(Value::as_bool).unwrap_or(false),
                    text(&input, "why"),
                ) {
                    Ok(data) => ok(data),
                    Err(reason) => fail(reason),
                },
                Err(response) => response,
            },
            ("POST", "/api/chat/stop") => match Self::body(request) {
                Ok(input) => {
                    let board = text(&input, "board");
                    let flag = self
                        .running
                        .lock()
                        .ok()
                        .and_then(|map| map.get(board).cloned());
                    flag.iter()
                        .for_each(|f| f.store(chat::STOPPED_BY_USER, Ordering::SeqCst));
                    ok(json!({"stopping": flag.is_some()}))
                }
                Err(response) => response,
            },
            ("POST", "/api/restore") => match Self::body(request) {
                Ok(input) => match chat::restore(
                    &self.env,
                    &self.skill,
                    &self.root,
                    text(&input, "board"),
                    text(&input, "run"),
                ) {
                    Ok(data) => ok(data),
                    Err(reason) => fail(reason),
                },
                Err(response) => response,
            },
            ("GET", "/") => Response {
                status: 303,
                headers: vec![("Location".to_owned(), "/dashboard.html".to_owned())],
                body: vec![],
            },
            ("GET", path) => match static_path(&Layout::new(&self.root).site(), path) {
                Some(file) => match self.env.files.read_bytes(&file) {
                    Ok(bytes) => Response {
                        status: 200,
                        headers: vec![("Content-Type".to_owned(), content_type(&file).to_owned())],
                        body: bytes,
                    },
                    Err(_) => Response::refuse(404, "無い"),
                },
                None => Response::refuse(404, "無い"),
            },
            _ => Response::refuse(404, "無い"),
        }
    }

    /// チャットと回答の送信の要求に、出来事を1行ずつ送って答える。send なら回答を書いてから AI がボードを更新する。
    fn chat(&self, request: &Request, stream: &mut TcpStream, send: bool) {
        let input = match Self::body(request) {
            Ok(input) => input,
            Err(response) => return write(stream, &response),
        };
        let turn = Turn {
            board: text(&input, "board").to_owned(),
            topic: text(&input, "topic").to_owned(),
            agent: text(&input, "agent").to_owned(),
            text: text(&input, "text").to_owned(),
            kind: if send {
                Kind::Send {
                    round: input.get("round").and_then(Value::as_u64).unwrap_or(0),
                    answers: input
                        .get("answers")
                        .and_then(Value::as_array)
                        .cloned()
                        .unwrap_or_default(),
                }
            } else {
                Kind::Ask
            },
        };
        let cancel = Arc::new(AtomicU8::new(chat::RUNNING));
        {
            let Ok(mut map) = self.running.lock() else {
                return;
            };
            if map.contains_key(&turn.board) {
                return write(
                    stream,
                    &Response::refuse(409, "このボードでは、AI がまだ作業している"),
                );
            }
            map.insert(turn.board.clone(), Arc::clone(&cancel));
        }
        let head = format!("HTTP/1.1 200 OK\r\nContent-Type: application/x-ndjson; charset=utf-8\r\n{}Connection: close\r\n\r\n", safe_headers());
        let mut alive = stream.write_all(head.as_bytes()).is_ok();
        let mut sink = |event: Value| {
            let line = format!("{event}\n");
            alive = alive
                && stream
                    .write_all(line.as_bytes())
                    .and_then(|_| stream.flush())
                    .is_ok();
            if !alive {
                let _ = cancel.compare_exchange(
                    chat::RUNNING,
                    chat::DISCONNECTED,
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                );
            }
        };
        if let Err(reason) = chat::turn(
            &self.env,
            &self.agents,
            &self.skill,
            &self.root,
            &turn,
            &self.limits,
            &cancel,
            &mut sink,
        ) {
            sink(json!({"type": "error", "text": reason}));
            sink(json!({"type": "done", "ok": false}));
        }
        if let Ok(mut map) = self.running.lock() {
            map.remove(&turn.board);
        }
    }

    /// 1つのつながりを扱う。
    pub fn handle(&self, mut stream: TcpStream) {
        let Some(request) = read_request(&mut stream) else {
            return write(&mut stream, &Response::refuse(400, "要求を読めない"));
        };
        if request.method == "POST" && (request.path == "/api/chat" || request.path == "/api/send")
        {
            if let Err(response) = guard(&request, self.port, &self.token) {
                return write(&mut stream, &response);
            }
            return self.chat(&request, &mut stream, request.path == "/api/send");
        }
        let response = self.answer(&request);
        write(&mut stream, &response);
    }
}

fn write(stream: &mut TcpStream, response: &Response) {
    let mut bytes = response.to_bytes();
    let at = bytes
        .windows(2)
        .position(|w| w == b"\r\n")
        .map(|p| p + 2)
        .unwrap_or(0);
    let extra = safe_headers().into_bytes();
    bytes.splice(at..at, extra);
    let _ = stream.write_all(&bytes);
}

/// 待ち受けを始める。番号 0 なら空いている番号を使う。返すのは、使った番号と待ち受け。
pub fn bind(port: u16) -> Result<(u16, TcpListener), String> {
    let listener = TcpListener::bind(("127.0.0.1", port))
        .map_err(|error| format!("127.0.0.1:{port} で待ち受けられない：{error}"))?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    Ok((port, listener))
}

/// つながりごとにスレッドを立てて扱う。戻らない。
pub fn serve<F, P, C, A>(server: Arc<Server<F, P, C, A>>, listener: TcpListener)
where
    F: Files + Send + Sync + 'static,
    P: Programs + Send + Sync + 'static,
    C: Clock + Send + Sync + 'static,
    A: Agents + Send + Sync + 'static,
{
    for stream in listener.incoming().flatten() {
        let server = Arc::clone(&server);
        thread::spawn(move || server.handle(stream));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(method: &str, path: &str, headers: &[(&str, &str)]) -> Request {
        Request {
            method: method.to_owned(),
            path: path.to_owned(),
            headers: headers
                .iter()
                .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
                .collect(),
            ..Request::default()
        }
    }

    #[test]
    fn requests_are_read_with_query_and_body() {
        let raw = "POST /api/send?board=a%2Db&x=1 HTTP/1.1\r\nHost: 127.0.0.1:9\r\nContent-Length: 2\r\n\r\n{}";
        let got = read_request(&mut raw.as_bytes()).unwrap();
        assert_eq!(
            (got.method.as_str(), got.path.as_str()),
            ("POST", "/api/send")
        );
        assert_eq!(got.query.get("board").map(String::as_str), Some("a-b"));
        assert_eq!(
            got.headers.get("host").map(String::as_str),
            Some("127.0.0.1:9")
        );
        assert_eq!(got.body, b"{}");
    }

    #[test]
    fn the_guard_refuses_wrong_host_missing_cookie_and_cross_origin_posts() {
        let cookie = ("cookie", "a=b; bs9=T");
        let cases = [
            (
                request("GET", "/x", &[("host", "evil.example:9"), cookie]),
                Some(403),
            ),
            (request("GET", "/x", &[("host", "127.0.0.1:9")]), Some(403)),
            (
                request("GET", "/x", &[("host", "localhost:9"), cookie]),
                None,
            ),
            (
                request(
                    "POST",
                    "/api/send",
                    &[
                        ("host", "127.0.0.1:9"),
                        cookie,
                        ("origin", "http://evil.example"),
                        ("content-type", "application/json"),
                    ],
                ),
                Some(403),
            ),
            (
                request(
                    "POST",
                    "/api/send",
                    &[
                        ("host", "127.0.0.1:9"),
                        cookie,
                        ("origin", "http://127.0.0.1:9"),
                        ("content-type", "text/plain"),
                    ],
                ),
                Some(400),
            ),
            (
                request(
                    "POST",
                    "/api/send",
                    &[
                        ("host", "127.0.0.1:9"),
                        cookie,
                        ("origin", "http://127.0.0.1:9"),
                        ("content-type", "application/json"),
                    ],
                ),
                None,
            ),
        ];
        let got: Vec<Option<u16>> = cases
            .iter()
            .map(|(r, _)| guard(r, 9, "T").err().map(|e| e.status))
            .collect();
        let want: Vec<Option<u16>> = cases.iter().map(|(_, w)| *w).collect();
        assert_eq!(got, want);
    }

    #[test]
    fn the_token_moves_into_a_cookie_once() {
        let mut first = request("GET", "/", &[]);
        first.query.insert("k".to_owned(), "T".to_owned());
        let response = welcome(&first, 9, "T").unwrap();
        assert_eq!(response.status, 303);
        assert!(response.headers.contains(&(
            "Set-Cookie".to_owned(),
            "bs9=T; HttpOnly; SameSite=Strict; Path=/".to_owned()
        )));
        first.query.insert("k".to_owned(), "X".to_owned());
        assert_eq!(welcome(&first, 9, "T").map(|r| r.status), Some(403));
        assert_eq!(welcome(&request("GET", "/", &[]), 9, "T"), None);
    }

    #[test]
    fn static_paths_stay_inside_the_site() {
        let site = Path::new("/r/site");
        assert_eq!(
            static_path(site, "/a.html"),
            Some(PathBuf::from("/r/site/a.html"))
        );
        assert_eq!(static_path(site, "/../b.json"), None);
        assert_eq!(static_path(site, "/x/../../b"), None);
    }
}
