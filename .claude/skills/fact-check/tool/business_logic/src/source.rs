// SPDX-License-Identifier: MIT
//! 原文を読み、照合し、取得する。
//!
//! **原文を連結しない。** 位置をたどれる形のまま持つ ── 連結すると、どの原文の
//! どこで一致したかが報告できない。
//!
//! **読めなかったものを、黙って除外しない。** 除外すると、0件が「無い」なのか
//! 「読めなかっただけ」なのかを、読み手が区別できない。
//!
//! **行の区切りを1つへ統一してから読む。** 原文には CR 単独で行を区切るものが在る
//! （実測 ── PDF から起こした原文が CR を 11,526 件持っていた）。統一しないと、
//! 報告する行が読み手の見る行と食い違う。

use crate::data_access::{self, files, http};
use crate::find::{self, How};
use std::io;
use std::path::{Path, PathBuf};

/// 1つの原文で読む上限（バイト）。
pub const MAX_BYTES: u64 = 4_000_000;

/// 見ない包み。
const SKIP: [&str; 7] = [
    ".git",
    "node_modules",
    "target",
    "dist",
    "build",
    ".venv",
    "__pycache__",
];

/// 原文1つ。**位置をたどれる形で持つ。**
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Doc {
    /// 呼ぶ側へ見せる名前。
    pub name: String,
    /// 中身。
    pub text: String,
    /// 行の頭の位置（文字で数える）。
    starts: Vec<usize>,
}

impl Doc {
    /// 原文を組む。
    #[must_use]
    pub fn new(name: String, text: String) -> Self {
        let mut starts = vec![0];
        for (i, c) in text.chars().enumerate() {
            if c == '\n' {
                starts.push(i + 1);
            }
        }
        Self { name, text, starts }
    }

    /// その位置が何行目かを返す（1から数える）。
    #[must_use]
    pub fn line_of(&self, offset: usize) -> usize {
        self.starts.partition_point(|s| *s <= offset)
    }

    /// その行の中身を返す。
    #[must_use]
    pub fn line_text(&self, line: usize) -> String {
        let Some(a) = self.starts.get(line.saturating_sub(1)) else {
            return String::new();
        };
        let chars: Vec<char> = self.text.chars().collect();
        let b = self.starts.get(line).copied().unwrap_or(chars.len());
        chars[*a..b.min(chars.len())]
            .iter()
            .collect::<String>()
            .trim_end_matches('\n')
            .to_owned()
    }
}

/// 一致1件。
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Hit {
    /// どの原文か。
    pub doc: String,
    /// 何行目か。
    pub line: usize,
    /// その行の中身。
    pub excerpt: String,
}

/// 1つの語の照合の結果。
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Found {
    /// 照合した語。
    pub needle: String,
    /// 一致した箇所。
    pub hits: Vec<Hit>,
}

/// 照合の全体。**見つかった位置と、読めなかった範囲を必ず一緒に持つ。**
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct Scan {
    /// 語ごとの結果。
    pub results: Vec<Found>,
    /// 読めなかったものと、その理由。
    pub unreadable: Vec<(String, String)>,
    /// 読めた原文の数。
    pub docs_read: usize,
}

impl Scan {
    /// 0件を「無い」と結論してよいか。**読めなかった範囲が在るなら、よくない。**
    #[must_use]
    pub fn can_conclude_absent(&self) -> bool {
        self.unreadable.is_empty()
    }

    /// 一致が1件も無かった語。
    #[must_use]
    pub fn missing(&self) -> Vec<&Found> {
        self.results.iter().filter(|r| r.hits.is_empty()).collect()
    }
}

fn walk(dir: &Path, base: &Path, out: &mut Vec<(PathBuf, String)>) {
    let Ok(paths) = files::list(dir) else {
        return;
    };
    for p in paths {
        let name = p
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        if files::is_dir(&p) {
            if !SKIP.contains(&name.as_str()) {
                walk(&p, base, out);
            }
        } else if !name.ends_with(".meta.json") {
            let rel = p.strip_prefix(base).unwrap_or(&p).display().to_string();
            out.push((p.clone(), rel));
        }
    }
}

/// 行の区切りを LF へ統一する。**CR 単独と CRLF の両方を直す。**
#[must_use]
pub fn normalize(text: &str) -> String {
    if !text.contains('\r') {
        return text.to_owned();
    }
    text.replace("\r\n", "\n").replace('\r', "\n")
}

/// 原文を読む。**読めなかったものは、黙って除外せず返す。**
#[must_use]
pub fn read_docs(path: &Path, max_bytes: u64) -> (Vec<Doc>, Vec<(String, String)>) {
    let mut files = Vec::new();
    if files::is_file(path) {
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        files.push((path.to_path_buf(), name));
    } else if files::is_dir(path) {
        walk(path, path, &mut files);
    } else {
        return (
            Vec::new(),
            vec![(path.display().to_string(), "在らない".to_owned())],
        );
    }
    let mut docs = Vec::new();
    let mut bad = Vec::new();
    for (file, name) in files {
        let Ok(size) = files::size(&file) else {
            bad.push((name, "開けない".to_owned()));
            continue;
        };
        if size > max_bytes {
            bad.push((name, format!("大きすぎる（{size} バイト）")));
            continue;
        }
        match files::read(&file) {
            Ok(raw) => match String::from_utf8(raw) {
                Ok(text) => docs.push(Doc::new(name, normalize(&text))),
                Err(_) => bad.push((name, "文字として読めない".to_owned())),
            },
            Err(e) => bad.push((name, format!("開けない（{e}）"))),
        }
    }
    (docs, bad)
}

/// 原文と照合する。
///
/// **種類は呼ぶ側が渡す** ── 道具の側で「たぶん識別子だろう」と決めない。
/// `near` を渡すと、その語の在る行から `within` 行の内側だけを一致として数える ──
/// **名前が在ることと、その名前がそこで使われることは別である。**
#[must_use]
pub fn scan(
    path: &Path,
    needles: &[String],
    how: How,
    near: Option<&str>,
    within: usize,
    max_bytes: u64,
) -> Scan {
    let (docs, bad) = read_docs(path, max_bytes);
    let mut results = Vec::new();
    for needle in needles {
        let mut hits = Vec::new();
        for doc in &docs {
            let anchors: Option<Vec<usize>> = near.map(|a| {
                find::text(&doc.text, a)
                    .into_iter()
                    .map(|o| doc.line_of(o))
                    .collect()
            });
            for offset in find::find(how, &doc.text, needle) {
                let line = doc.line_of(offset);
                if let Some(anchors) = &anchors {
                    if !anchors.iter().any(|a| line.abs_diff(*a) <= within) {
                        continue;
                    }
                }
                hits.push(Hit {
                    doc: doc.name.clone(),
                    line,
                    excerpt: doc.line_text(line).trim().to_owned(),
                });
            }
        }
        results.push(Found {
            needle: needle.clone(),
            hits,
        });
    }
    Scan {
        results,
        unreadable: bad,
        docs_read: docs.len(),
    }
}

/// 保存する名前を、出どころから作る。
#[must_use]
pub fn slug(url: &str) -> String {
    let body = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);
    let mut out = String::new();
    let mut last_join = false;
    for c in body.chars() {
        if c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-' {
            out.push(c);
            last_join = false;
        } else if !last_join {
            out.push('_');
            last_join = true;
        }
    }
    out.trim_matches('_').chars().take(120).collect()
}

/// 取得したものを原文として受け取るか。
///
/// **記法を剥がしたり変換したりしない** ── 変換した時点で、「原文と1文字ずつ同じ」が
/// 主張できなくなる。
#[must_use]
pub const fn acceptable(code: u32, size: u64) -> bool {
    code == 200 && size > 0
}

/// 取得の記録。
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Fetched {
    /// 保存した場所。
    pub path: PathBuf,
    /// 実際に取得した出どころ。
    pub url: String,
    /// 頼まれた出どころ。
    pub requested: String,
    /// 取得した日時。
    pub fetched_at: String,
    /// 中身の要約。
    pub sha256: String,
    /// バイト数。
    pub bytes: u64,
    /// 行数。
    pub lines: usize,
    /// 種類。
    pub content_type: String,
}

fn sha256_of(data: &[u8]) -> String {
    use sha2::{Digest as _, Sha256};
    Sha256::digest(data)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn now() -> String {
    // **外部コマンドの date を呼ばない** ── Windows に実行ファイルとして無い（ACDR 0029）。
    // 利用者の地域の時刻で書く ── date と同じく、協定世界時にしない
    chrono::Local::now()
        .format("%Y-%m-%dT%H:%M:%S%z")
        .to_string()
}

/// 起動の失敗を、呼ぶ側へ見せる文言にする。**起動できないときは、OS が返した理由をそのまま見せる。**
fn failed(why: &data_access::process::Failed) -> String {
    use data_access::process::Failed;
    match why {
        Failed::Spawn(e) | Failed::Pipe(e) => e.clone(),
        Failed::Timeout => "取得が制限時間を過ぎたので止めた".to_owned(),
        _ => format!("{why:?}"),
    }
}

/// 原文を取得する。**まず `<出どころ>.md` を試し、無ければ本体を取る。**
///
/// 取得は curl で行う ── 利用者の環境のプロキシと証明書の設定をそのまま使うためである。
/// **コマンドは引数で受け取る**（tool.json の external から、サービス層が渡す）。
///
/// # Errors
///
/// 取得できなかったとき、試した先を添えて返す。
pub fn fetch(curl: &str, url: &str, outdir: &Path) -> io::Result<Fetched> {
    files::create_dir_all(outdir)?;
    let plain = url.ends_with(".md") || url.ends_with(".txt") || url.ends_with(".json");
    let candidates: Vec<String> = if plain {
        vec![url.to_owned()]
    } else {
        vec![format!("{url}.md"), url.to_owned()]
    };
    let mut tried = Vec::new();
    for cand in candidates {
        let path = outdir.join(slug(&cand));
        let got = http::download(curl, &cand, &path).map_err(|e| io::Error::other(failed(&e)))?;
        let (code, ctype) = (got.code, got.content_type);
        let size = files::size(&path).unwrap_or(0);
        tried.push(format!("  {code}  {size:>9}  {ctype:32}{cand}"));
        if acceptable(code, size) {
            let body = files::read(&path)?;
            return Ok(Fetched {
                sha256: sha256_of(&body),
                bytes: body.len() as u64,
                lines: body.iter().filter(|b| **b == b'\n').count() + 1,
                path,
                url: cand,
                requested: url.to_owned(),
                fetched_at: now(),
                content_type: ctype,
            });
        }
        let _ = files::remove_file(&path); // 受け取らなかったものを残さない
    }
    Err(io::Error::other(format!(
        "取得できなかった。試したもの:\n{}",
        tried.join("\n")
    )))
}

/// 取得の記録を書く。
///
/// # Errors
///
/// 書けないときに返す。
pub fn write_meta(got: &Fetched) -> io::Result<PathBuf> {
    let name = got
        .path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let meta = got.path.with_file_name(format!("{name}.meta.json"));
    let body = serde_json::json!({
        "url": got.url, "requested": got.requested, "fetched_at": got.fetched_at,
        "sha256": got.sha256, "bytes": got.bytes, "lines": got.lines,
        "content_type": got.content_type,
    });
    let text = serde_json::to_string_pretty(&body)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    files::write(&meta, text.replace("  ", " ") + "\n")?;
    Ok(meta)
}

/// 取得したものを並べる。**印字はしない。**
#[must_use]
pub fn listing(outdir: &Path) -> Vec<(String, usize, String)> {
    let Ok(names) = files::list(outdir) else {
        return Vec::new();
    };
    let mut rows = Vec::new();
    for p in names {
        if !p.to_string_lossy().ends_with(".meta.json") {
            continue;
        }
        let Ok(body) = files::read_to_string(&p) else {
            continue;
        };
        let Ok(meta) = serde_json::from_str::<serde_json::Value>(&body) else {
            continue;
        };
        let at = meta
            .get("fetched_at")
            .and_then(|x| x.as_str())
            .unwrap_or_default();
        rows.push((
            at.chars().take(10).collect(),
            meta.get("lines")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0) as usize,
            meta.get("url")
                .and_then(|x| x.as_str())
                .unwrap_or_default()
                .to_owned(),
        ));
    }
    rows
}

/// 置き場所がフォルダとして在るか。
#[must_use]
pub fn is_place(dir: &Path) -> bool {
    files::is_dir(dir)
}

/// 照合するものを一覧のファイルから読む。**空の行を捨てる。** 読めなければ空を返す。
#[must_use]
pub fn needles_in(from: &str) -> Vec<String> {
    files::read_to_string(from).map_or_else(
        |_| Vec::new(),
        |body| {
            body.lines()
                .map(str::trim)
                .filter(|x| !x.is_empty())
                .map(str::to_owned)
                .collect()
        },
    )
}

/// 原文の横に置いた取得の記録（`<名前>.meta.json`）の中身。**無ければ None を返す。**
#[must_use]
pub fn meta_text(path: &Path) -> Option<String> {
    let name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let meta = path.with_file_name(format!("{name}.meta.json"));
    files::read_to_string(meta).ok()
}
