//! 試験で共有するもの：作業用のフォルダ、見本の実行ファイル（schema-driven-stub）、SHA-256。
#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

/// 見本の実行ファイルを一度だけ作り、そのパスを返す。
pub fn stub() -> &'static Path {
    static STUB: OnceLock<PathBuf> = OnceLock::new();
    STUB.get_or_init(|| {
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let status = Command::new(env!("CARGO"))
            .args(["build", "-q", "--locked", "-p", "schema-driven-stub"])
            .current_dir(&workspace)
            .status()
            .unwrap();
        assert!(status.success(), "見本の実行ファイルを作れない");
        let name = format!("schema-driven-stub{}", std::env::consts::EXE_SUFFIX);
        let target = std::env::var("CARGO_TARGET_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| workspace.join("target"));
        target.join("debug").join(name)
    })
}

pub fn sha256_of(path: &Path) -> String {
    use std::fmt::Write;
    let bytes = fs::read(path).unwrap();
    let digest = sha256(&bytes);
    digest.iter().fold(String::new(), |mut out, byte| {
        let _ = write!(out, "{byte:02x}");
        out
    })
}

/// テストの側で SHA-256 を求める（エンジンと同じ crate に頼らない）。
fn sha256(bytes: &[u8]) -> [u8; 32] {
    let output = Command::new(if cfg!(windows) {
        "certutil"
    } else {
        "sha256sum"
    })
    .args(if cfg!(windows) {
        vec!["-hashfile".to_owned(), "-".to_owned(), "SHA256".to_owned()]
    } else {
        vec![]
    })
    .stdin(std::process::Stdio::piped())
    .stdout(std::process::Stdio::piped())
    .spawn()
    .and_then(|mut child| {
        use std::io::Write;
        child.stdin.take().unwrap().write_all(bytes)?;
        child.wait_with_output()
    })
    .unwrap();
    let text = String::from_utf8_lossy(&output.stdout);
    let hex: String = text
        .split_whitespace()
        .find(|word| word.len() == 64)
        .unwrap()
        .to_owned();
    let mut out = [0u8; 32];
    for (index, byte) in out.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16).unwrap();
    }
    out
}

/// 試験の作業用のフォルダ。実行ごとに別の名前で作り、試験が終わると消す（失敗で打ち切られたときも消す）。
/// 実行ごとに名前を変えるので、別のセッションが同時に試験を走らせても、同じフォルダを取り合わない。
pub struct Scratch(PathBuf);

impl Scratch {
    pub fn new(name: &str) -> Self {
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("sd-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        Scratch(dir)
    }
}

impl std::ops::Deref for Scratch {
    type Target = Path;
    fn deref(&self) -> &Path {
        &self.0
    }
}

impl AsRef<Path> for Scratch {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
