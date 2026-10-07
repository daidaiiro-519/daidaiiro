//! パスの文字列の計算。ファイルシステムには触れない（字面だけで . と .. を解く）。

/// パスのディレクトリの部分（最後の / より前）。/ が無ければ空。
pub fn parent(path: &str) -> &str {
    path.rsplit_once('/').map(|(dir, _)| dir).unwrap_or("")
}

/// . と .. を字面で解く。根より上へ出る .. は残す。
pub fn normalize(path: &str) -> String {
    let absolute = path.starts_with('/');
    let mut out: Vec<&str> = Vec::new();
    for segment in path.split('/') {
        match segment {
            "" | "." => {}
            ".." if out.last().is_some_and(|part| *part != "..") => {
                out.pop();
            }
            ".." if absolute => {}
            part => out.push(part),
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
    let from_parts: Vec<&str> = from.split('/').filter(|part| !part.is_empty()).collect();
    let target_parts: Vec<&str> = target.split('/').filter(|part| !part.is_empty()).collect();
    let common = from_parts
        .iter()
        .zip(&target_parts)
        .take_while(|(from_part, target_part)| from_part == target_part)
        .count();
    if from_parts[common..].contains(&"..") {
        return target;
    }
    let mut parts: Vec<&str> = vec![".."; from_parts.len() - common];
    parts.extend(&target_parts[common..]);
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
