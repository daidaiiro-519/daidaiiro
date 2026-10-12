//! パスの文字列の計算。ファイルシステムには触れない。

/// パスのディレクトリの部分（最後の / より前）。/ が無ければ空。
pub fn parent(path: &str) -> &str {
    path.rsplit_once('/').map(|(dir, _)| dir).unwrap_or("")
}
