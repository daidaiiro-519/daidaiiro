// SPDX-License-Identifier: MIT
//! 通信の入出力。**fact-check に固有の module である**（`files` ・ `process` は雛形の複製）。
//!
//! 取得は curl で行う ── 利用者の環境のプロキシと証明書の設定をそのまま使うためである。
//! **コマンドの名前を持たない** ── 名前は tool.json の external からサービス層が読み、引数で渡る。
//! **判定を置かない** ── 取得したものを原文として受け取るかは、業務ロジック層が決める。

use std::path::Path;
use std::time::Duration;

use crate::process::{self, Failed};

/// curl へ渡す制限時間（秒）。
const CURL_SECONDS: &str = "60";

/// curl が止まらなかったときに、こちらから止めるまでの時間。**curl の制限時間より長く取る**
/// ── curl が自分で止まる限り、こちらの制限は働かない。
const LIMIT: Duration = Duration::from_secs(90);

/// 1回の取得の応答。
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Got {
    /// HTTP の状態コード。読み取れなければ 0 である。
    pub code: u32,
    /// 種類（Content-Type）。
    pub content_type: String,
}

/// 出どころを取得し、`dest` へ書く。**書いたものを受け取るかは決めない。**
///
/// # Errors
///
/// curl を起動できないか、制限時間を過ぎたときに返す。
pub fn download(curl: &str, url: &str, dest: &Path) -> Result<Got, Failed> {
    let args = vec![
        "-sSL".to_owned(),
        "-m".to_owned(),
        CURL_SECONDS.to_owned(),
        "-A".to_owned(),
        "Mozilla/5.0".to_owned(),
        url.to_owned(),
        "-o".to_owned(),
        dest.to_string_lossy().into_owned(),
        "-w".to_owned(),
        "%{http_code} %{content_type}".to_owned(),
    ];
    let ran = process::run(curl, &args, ".", LIMIT)?;
    let mut parts = ran.stdout.splitn(2, ' ');
    let code: u32 = parts.next().unwrap_or("0").trim().parse().unwrap_or(0);
    let content_type = parts.next().unwrap_or("").trim().to_owned();
    Ok(Got { code, content_type })
}
