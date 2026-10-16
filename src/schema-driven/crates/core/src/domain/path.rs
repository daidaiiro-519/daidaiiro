//! ファイルの経路の文字列から、最後のファイル名を取る。区切りは `/` と、Windows の `\` の両方を見る。
//! 経路の区切りを `/` だけで見ると、Windows では区切りの前まで名前に残り、スキーマの URI が壊れる。

/// 経路の最後のファイル名。区切りが無ければ、経路をそのまま返す。
pub fn file_name(path: &str) -> &str {
    path.rsplit(['/', '\\']).next().unwrap_or(path)
}

#[cfg(test)]
mod tests {
    use super::file_name;

    #[test]
    fn the_last_name_is_taken_with_either_separator() {
        let paths = [
            "assets/skill.schema.json",
            "C:\\a\\skills-creator\\assets\\skill.schema.json",
            "C:\\a\\skills-creator/assets\\skill.schema.json",
            "skill.schema.json",
        ];
        let got: Vec<&str> = paths.iter().map(|path| file_name(path)).collect();
        assert_eq!(got, vec!["skill.schema.json"; 4]);
    }
}
