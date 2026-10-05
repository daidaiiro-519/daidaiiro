//! パスの文字列の計算。ファイルシステムには触れない（字面だけで . と .. を解く）。

/// パスのディレクトリの部分（最後の / より前）。/ が無ければ空。
pub fn parent(path: &str) -> &str {
    path.rsplit_once('/').map(|(d, _)| d).unwrap_or("")
}

/// . と .. を字面で解く。根より上へ出る .. は残す。
pub fn normalize(path: &str) -> String {
    let absolute = path.starts_with('/');
    let mut out: Vec<&str> = Vec::new();
    for seg in path.split('/') {
        match seg {
            "" | "." => {}
            ".." if out.last().is_some_and(|s| *s != "..") => {
                out.pop();
            }
            ".." if absolute => {}
            s => out.push(s),
        }
    }
    let joined = out.join("/");
    if absolute {
        format!("/{joined}")
    } else {
        joined
    }
}

/// ディレクトリ dir から見た相対パス rel を、dir と同じ基準のパスにする。rel が絶対パスならそのまま。
pub fn join(dir: &str, rel: &str) -> String {
    if rel.starts_with('/') || dir.is_empty() {
        normalize(rel)
    } else {
        normalize(&format!("{dir}/{rel}"))
    }
}

/// ディレクトリ from から target への相対パス。どちらかだけが絶対パスなら target をそのまま返す。
pub fn relative(from: &str, target: &str) -> String {
    if from.starts_with('/') != target.starts_with('/') {
        return target.to_owned();
    }
    let from = normalize(from);
    let target = normalize(target);
    let f: Vec<&str> = from.split('/').filter(|s| !s.is_empty()).collect();
    let t: Vec<&str> = target.split('/').filter(|s| !s.is_empty()).collect();
    let common = f.iter().zip(&t).take_while(|(a, b)| a == b).count();
    if f[common..].contains(&"..") {
        return target;
    }
    let mut parts: Vec<&str> = vec![".."; f.len() - common];
    parts.extend(&t[common..]);
    parts.join("/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_and_join_are_inverse() {
        assert_eq!(
            relative("data", "schema/thing.schema.json"),
            "../schema/thing.schema.json"
        );
        assert_eq!(
            join("data", "../schema/thing.schema.json"),
            "schema/thing.schema.json"
        );
        assert_eq!(relative("", "schema/a.json"), "schema/a.json");
        assert_eq!(relative("a/b", "a/c/d.json"), "../c/d.json");
        assert_eq!(join("", "a.json"), "a.json");
        assert_eq!(normalize("./a/../b"), "b");
        assert_eq!(relative("/x", "y.json"), "y.json");
    }
}
