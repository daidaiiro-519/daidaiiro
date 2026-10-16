//! 公開された基盤（schema-driven）の配布物の名前と、SHA-256 の照合。
//! 名前の形は、基盤のワークフロー（.github/workflows/schema-driven.yml）が決めたものに合わせる。

use sha2::{Digest, Sha256};

/// 基盤を公開しているリポジトリ。
pub const REPO: &str = "daidaiiro-519/daidaiiro";
/// 基盤の配布物に証明を付けたワークフロー。照合で署名者を縛る。
pub const WORKFLOW: &str = ".github/workflows/schema-driven.yml";
/// SHA-256 の一覧のファイル名。
pub const SUMS: &str = "SHA256SUMS";
/// 証明の bundle のファイル名。
pub const BUNDLE: &str = "attestation.sigstore.json";
/// 基盤の実行ファイルの名前（拡張子を除く）。
pub const ENGINE_BINARIES: [&str; 2] = ["schema-driven", "schema-driven-mcp"];

/// 基盤の版から、Release の tag を作る。
pub fn tag(version: &str) -> String {
    format!("schema-driven-v{version}")
}

/// 基盤の版と target から、配布物のファイル名を作る。Windows は zip、ほかは tar.gz。
pub fn asset(version: &str, target: &str) -> String {
    let ext = if target.contains("windows") {
        "zip"
    } else {
        "tar.gz"
    };
    format!("schema-driven-{version}-{target}.{ext}")
}

/// target の実行ファイルに付く拡張子。
pub fn exe_suffix(target: &str) -> &'static str {
    if target.contains("windows") {
        ".exe"
    } else {
        ""
    }
}

/// この実行ファイルをビルドした環境に合う、基盤の配布物の target。
pub fn host_target() -> &'static str {
    if cfg!(target_os = "windows") {
        "x86_64-pc-windows-msvc"
    } else if cfg!(target_os = "macos") {
        "universal2-apple-darwin"
    } else {
        "x86_64-unknown-linux-musl"
    }
}

/// SHA256SUMS の中から、配布物の SHA-256 を引く。
pub fn expected_sha256<'a>(sums: &'a str, asset: &str) -> Option<&'a str> {
    sums.lines().find_map(|line| {
        let (sha, name) = line.split_once(char::is_whitespace)?;
        (name.trim_start().trim_start_matches('*') == asset).then_some(sha)
    })
}

/// バイト列の SHA-256 を、小文字の16進で返す。
pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_names_follow_the_base_workflow() {
        let names = [
            asset("0.1.0", "x86_64-unknown-linux-musl"),
            asset("0.1.0", "x86_64-pc-windows-msvc"),
            asset("0.1.0", "universal2-apple-darwin"),
        ];
        assert_eq!(
            names,
            [
                "schema-driven-0.1.0-x86_64-unknown-linux-musl.tar.gz",
                "schema-driven-0.1.0-x86_64-pc-windows-msvc.zip",
                "schema-driven-0.1.0-universal2-apple-darwin.tar.gz",
            ]
        );
        assert_eq!(tag("0.1.0"), "schema-driven-v0.1.0");
    }

    #[test]
    fn the_sha256_is_taken_from_the_line_of_the_asset() {
        let sums = "aaa  schema-driven-0.1.0-a.tar.gz\nbbb  schema-driven-0.1.0-b.zip\n";
        let found = [
            expected_sha256(sums, "schema-driven-0.1.0-b.zip"),
            expected_sha256(sums, "schema-driven-0.1.0-c.zip"),
        ];
        assert_eq!(found, [Some("bbb"), None]);
    }

    #[test]
    fn sha256_matches_the_published_test_vector() {
        // FIPS 180-2 の付録の "abc"
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn only_windows_targets_get_the_exe_suffix() {
        let got: Vec<&str> = ["x86_64-pc-windows-msvc", "x86_64-unknown-linux-musl"]
            .iter()
            .map(|target| exe_suffix(target))
            .collect();
        assert_eq!(got, vec![".exe", ""]);
    }
}
