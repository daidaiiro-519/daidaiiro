// SPDX-License-Identifier: MIT
//! 規則に書いた道具を実行し、終了コードで判定する。
//!
//! **出力を解析しない。** 検出は道具の仕事で、意味の解釈は読み手が実施する ──
//! 解析すると、道具ごとに違う書式へ依存する。
//!
//! **出力はファイルへ流す。** まとめて受け取ると、道具が出した量がそのままこの側の
//! 記憶に載る（実測 2026-09-24、100MB を出す道具で最大常駐 306MB）。

use std::fs::File;
use std::io::{self, Read as _, Seek as _, SeekFrom, Write as _};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::inward::judge::Layer;
use crate::label::Verdict;
use crate::rules::{Inward, Rule};

/// 1件あたりの制限時間（秒）。
pub const DEFAULT_TIMEOUT: u64 = 120;
/// 出力から読む先頭の量（バイト）。
pub const OUTPUT_HEAD: usize = 8_000;
/// 出力から読む末尾の量（バイト）。
///
/// **落としてよいのは真ん中だけである。** 読み手は人だけではない ── MCP から呼ぶ側は
/// この出力で次の手を決める。翻訳器は先頭に、試験の実行器は末尾に要るものを置く。
/// **全文は保存し、続きは `output` が読む。**
pub const OUTPUT_TAIL: usize = 8_000;

/// 1件の結果。
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Outcome {
    /// 規則の名前。
    pub name: String,
    /// 実行したコマンド。
    pub tool: Vec<String>,
    /// 実行した場所。
    pub target: String,
    /// 終了コード。**実行していなければ無い。**
    pub code: Option<i32>,
    /// 判定。
    pub verdict: Verdict,
    /// 実行しなかった理由。
    pub reason: String,
    /// 読んだ出力。
    pub output: String,
    /// 出力の全体の大きさ。
    pub output_size: u64,
    /// 全文を保持したか。
    pub saved: bool,
}

impl Outcome {
    fn skipped(rule: &Rule, reason: String) -> Self {
        Self {
            name: rule.name.clone(),
            tool: rule.tool.clone(),
            target: rule.target.clone(),
            code: None,
            verdict: Verdict::Skip,
            reason,
            output: String::new(),
            output_size: 0,
            saved: false,
        }
    }
}

/// この成果物の保存先。**成果物ごとに分ける** ── 別の成果物の実行を消さない。
#[must_use]
pub fn runs_dir(root: &Path) -> PathBuf {
    let key = short_key(&root.canonicalize().unwrap_or_else(|_| root.to_path_buf()));
    std::env::temp_dir()
        .join("no-more-spaghetti-runs")
        .join(key)
}

/// 経路から、短い固定長の名前を作る。**衝突しても別の成果物の出力を返さない** ──
/// 保持するのは直近の1回だけで、識別子が合わなければ断る。
fn short_key(path: &Path) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in path.display().to_string().bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100_0000_01b3);
    }
    format!("{hash:016x}")
}

/// 一時ファイルに振る番号。**同時に走る実行が、同じ名前を使わないようにする**
/// ── 時刻だけでは区別できない（実測 ── 並列の事例3件が互いの出力を上書きした）。
static NEXT: AtomicU64 = AtomicU64::new(0);

/// 実行ごとの識別子。**直近の1回だけを保持する**ので、時刻から作る。
#[must_use]
pub fn new_run_id() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{:08x}{:06x}", now.as_secs(), now.subsec_micros())
}

/// 出力を流す先。**同じ名前を2つの実行が使わない** ── 処理の識別子と、増える番号を
/// 混ぜる。
fn sink_path(index: usize) -> PathBuf {
    let serial = NEXT.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "no-more-spaghetti-{}-{}-{serial}-{index}.log",
        std::process::id(),
        new_run_id()
    ))
}

/// 子を待つ。**制限時間を超えたら終わらせる。**
fn wait_for(child: &mut std::process::Child, limit: Duration) -> io::Result<Option<i32>> {
    let started = Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(Some(status.code().unwrap_or(-1)));
        }
        if started.elapsed() >= limit {
            child.kill()?;
            let _ = child.wait()?;
            return Ok(None);
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// 出力から、先頭と末尾を読む。**中略したことを本文へ書く。**
fn read_ends(file: &mut File, size: u64) -> io::Result<String> {
    let cap = (OUTPUT_HEAD + OUTPUT_TAIL) as u64;
    file.seek(SeekFrom::Start(0))?;
    if size <= cap {
        let mut body = Vec::new();
        file.read_to_end(&mut body)?;
        return Ok(String::from_utf8_lossy(&body).trim().to_owned());
    }
    let mut head = vec![0_u8; OUTPUT_HEAD];
    file.read_exact(&mut head)?;
    file.seek(SeekFrom::Start(size - OUTPUT_TAIL as u64))?;
    let mut tail = vec![0_u8; OUTPUT_TAIL];
    file.read_exact(&mut tail)?;
    Ok(format!(
        "{}\n\n── 中略（全 {size} バイトのうち {cap} バイトを表示）\n\
         ── 続きは output で読む（offset={OUTPUT_HEAD}）\n\n{}",
        String::from_utf8_lossy(&head).trim_end(),
        String::from_utf8_lossy(&tail).trim_start()
    ))
}

/// 1件を実行する。**シェルを経由しない** ── 配列のまま渡す。
///
/// **`check.target` が在れば、そこで実行する。** 実在しなければ「実行しない」で、
/// 合格に寄せない。**成果物の場所の外を指す対象は実行しない** ── 解決してから、
/// 成果物の場所の中かを確認する。
///
/// # Errors
///
/// 出力を保持する先へ書けないときに返す。
pub fn run_one(
    rule: &Rule,
    root: &Path,
    timeout: u64,
    save_to: Option<&Path>,
    index: usize,
) -> io::Result<Outcome> {
    if !rule.missing_unit.is_empty() {
        return Ok(Outcome::skipped(
            rule,
            format!("成果物が無い ── {}", rule.missing_unit),
        ));
    }
    if rule.inward.is_none() {
        if rule.tool_is_not_a_list {
            return Ok(Outcome::skipped(rule, "道具が配列ではない".to_owned()));
        }
        if rule.tool.is_empty() {
            return Ok(Outcome::skipped(rule, "検証方法に道具が無い".to_owned()));
        }
    }
    let base = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let Ok(cwd) = base.join(&rule.target).canonicalize() else {
        return Ok(Outcome::skipped(
            rule,
            format!("対象が実在しない ── {}", base.join(&rule.target).display()),
        ));
    };
    if !cwd.starts_with(&base) {
        return Ok(Outcome::skipped(
            rule,
            format!("対象が成果物の場所の外を指す ── {}", rule.target),
        ));
    }
    if !cwd.is_dir() {
        return Ok(Outcome::skipped(
            rule,
            format!("対象が実在しない ── {}", cwd.display()),
        ));
    }

    if let Some(inward) = &rule.inward {
        return Ok(run_inward(rule, inward, &cwd));
    }

    let sink_path = sink_path(index);
    let sink = File::create(&sink_path)?;
    let err = sink.try_clone()?;
    let spawned = Command::new(&rule.tool[0])
        .args(&rule.tool[1..])
        .current_dir(&cwd)
        .stdin(Stdio::null())
        .stdout(sink)
        .stderr(err)
        .spawn();
    let mut child = match spawned {
        Ok(child) => child,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            let _ = std::fs::remove_file(&sink_path);
            return Ok(Outcome::skipped(rule, "道具が見つからない".to_owned()));
        }
        Err(e) => {
            let _ = std::fs::remove_file(&sink_path);
            return Err(e);
        }
    };
    let code = wait_for(&mut child, Duration::from_secs(timeout))?;
    let Some(code) = code else {
        let _ = std::fs::remove_file(&sink_path);
        return Ok(Outcome::skipped(rule, format!("{timeout}秒で終わらない")));
    };

    let mut file = File::open(&sink_path)?;
    let size = file.metadata()?.len();
    let output = read_ends(&mut file, size)?;
    let mut saved = false;
    if size > (OUTPUT_HEAD + OUTPUT_TAIL) as u64 {
        if let Some(dir) = save_to {
            std::fs::create_dir_all(dir)?;
            std::fs::copy(&sink_path, dir.join(format!("{index}.log")))?;
            saved = true;
        }
    }
    drop(file);
    let _ = std::fs::remove_file(&sink_path);
    Ok(Outcome {
        name: rule.name.clone(),
        tool: rule.tool.clone(),
        target: rule.target.clone(),
        code: Some(code),
        verdict: if code == 0 {
            Verdict::Pass
        } else {
            Verdict::Fail
        },
        reason: String::new(),
        output,
        output_size: size,
        saved,
    })
}

/// 依存の向きを、この Skill の中で測る。**別の process を立てない** ── 立てると、
/// MCP から呼んだときに、実行する binary が CLI ではない。
///
/// 判定は他の規則と同じで、食い違いが0件なら合格である。**測れなかったときは
/// 「実行しない」にし、合格に寄せない。**
fn run_inward(rule: &Rule, inward: &Inward, cwd: &Path) -> Outcome {
    let mut layers = Vec::new();
    for l in &inward.layers {
        let mut layer = Layer::of(l.name.clone(), l.ids.clone());
        for mark in &l.marks {
            match layer.marked(mark) {
                Ok(next) => layer = next,
                Err(why) => return Outcome::skipped(rule, why),
            }
        }
        layers.push(layer);
    }
    let got = match crate::inward::measure(&inward.language, cwd, layers) {
        Ok(got) => got,
        Err(why) => return Outcome::skipped(rule, why),
    };
    let mut lines = if got.findings.is_empty() {
        vec![format!(
            "向きは内向きである ── 依存 {} 件を確認した",
            got.edges.len()
        )]
    } else {
        let mut v = vec![format!(
            "食い違い　{} 件 ／ 確認した依存 {} 件",
            got.findings.len(),
            got.edges.len()
        )];
        v.extend(got.findings.iter().map(|x| format!("  ・{x}")));
        v
    };
    // **測り方の限界は毎回出す。** 黙らせると、取りこぼしの範囲が読めない
    lines.extend(got.limits.iter().map(|x| format!("  （測り方）{x}")));
    let output = lines.join("\n");
    let pass = got.findings.is_empty();
    Outcome {
        name: rule.name.clone(),
        tool: rule.tool.clone(),
        target: rule.target.clone(),
        code: Some(i32::from(!pass)),
        verdict: if pass { Verdict::Pass } else { Verdict::Fail },
        reason: String::new(),
        output_size: output.len() as u64,
        output,
        saved: false,
    }
}

/// 全件の結果。
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct Report {
    /// 1件ずつの結果。
    pub rules: Vec<Outcome>,
    /// 実行の識別子。
    pub run: String,
    /// 検出したもの。
    pub findings: Vec<String>,
}

impl Report {
    /// 判定ごとの件数を数える。
    #[must_use]
    pub fn count(&self, verdict: Verdict) -> usize {
        self.rules.iter().filter(|x| x.verdict == verdict).count()
    }
}

/// 規則を全件実行する。**早期終了しない** ── 途中で止めると、実行しなかった規則が
/// 合格と区別できない。
///
/// # Errors
///
/// 規則ファイルを読めないとき、または出力を保持できないときに返す。
pub fn check(root: &Path, rules_file: &Path, timeout: u64) -> io::Result<Report> {
    let rules = crate::rules::load(rules_file)?;
    // **保持するのは直近の1回だけである。** 溜め続けると置き場所が膨らむ
    let run = new_run_id();
    let dir = runs_dir(root);
    let _ = std::fs::remove_dir_all(&dir);
    let save_to = dir.join(&run);
    let mut out: Vec<Outcome> = Vec::new();
    // **生成の手順が失敗した成果物は、inward を実行しない** ── 生成物が無いまま測ると、
    // 「参照先が実在しない」が実際の違反と区別できない
    let mut failed: Vec<(String, String)> = Vec::new();
    for (i, rule) in rules.iter().enumerate() {
        if let Some((_, step)) = failed.iter().find(|(unit, _)| *unit == rule.unit) {
            if rule.inward.is_some() || !rule.generates.is_empty() {
                out.push(Outcome::skipped(
                    rule,
                    format!("生成の手順が失敗した ── {step}"),
                ));
                continue;
            }
        }
        let got = run_one(rule, root, timeout, Some(&save_to), i)?;
        if !rule.generates.is_empty() && got.verdict != Verdict::Pass {
            failed.push((rule.generates.clone(), rule.name.clone()));
        }
        out.push(got);
    }
    let mut findings: Vec<String> = out
        .iter()
        .filter(|x| x.verdict != Verdict::Pass)
        .map(|x| {
            if x.reason.is_empty() {
                format!("{} ── {}", x.name, x.verdict.label())
            } else {
                format!("{} ── {}（{}）", x.name, x.verdict.label(), x.reason)
            }
        })
        .collect();
    if rules.is_empty() {
        // **1件も検査していない状態を、合格と同じ姿で返さない。**
        findings.push("規則が0件である ── 1件も検査していない".to_owned());
    }
    if out.iter().any(|x| x.saved) {
        std::fs::create_dir_all(&save_to)?;
        let names: Vec<&str> = out.iter().map(|x| x.name.as_str()).collect();
        let body = serde_json::json!({ "run": run, "rules": names });
        let mut index = File::create(save_to.join("index.json"))?;
        index.write_all(serde_json::to_string(&body).unwrap_or_default().as_bytes())?;
    }
    Ok(Report {
        rules: out,
        run,
        findings,
    })
}

/// 保持した出力の続き。
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Slice {
    /// 読んだ中身。
    pub output: String,
    /// 読み始めた位置。
    pub offset: u64,
    /// 全体の大きさ。
    pub output_size: u64,
    /// 次に読む位置。**終わりなら無い。**
    pub next_offset: Option<u64>,
}

/// 保持した出力を読む。**道具は実行しない。**
///
/// # Errors
///
/// その実行を保持していないとき、またはその名前がその実行に無いときに返す。
pub fn read_output(
    root: &Path,
    run: &str,
    name: &str,
    offset: u64,
    length: usize,
) -> io::Result<Slice> {
    let saved = runs_dir(root).join(run);
    let index = saved.join("index.json");
    let Ok(body) = std::fs::read_to_string(&index) else {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("その実行の出力は保持していない ── run={run}。check を実行し直す"),
        ));
    };
    let parsed: serde_json::Value = serde_json::from_str(&body)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    let names: Vec<&str> = parsed
        .get("rules")
        .and_then(serde_json::Value::as_array)
        .map(|list| list.iter().filter_map(serde_json::Value::as_str).collect())
        .unwrap_or_default();
    let Some(at) = names.iter().position(|x| *x == name) else {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("その名前は、この実行に無い ── {name}"),
        ));
    };
    let path = saved.join(format!("{at}.log"));
    let mut file = File::open(&path).map_err(|_| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("その規則の出力は保持していない ── {name}"),
        )
    })?;
    let size = file.metadata()?.len();
    file.seek(SeekFrom::Start(offset))?;
    let mut buf = vec![0_u8; length.max(1)];
    let read = file.read(&mut buf)?;
    buf.truncate(read);
    let next = offset + read as u64;
    Ok(Slice {
        output: String::from_utf8_lossy(&buf).into_owned(),
        offset,
        output_size: size,
        next_offset: if next < size { Some(next) } else { None },
    })
}
