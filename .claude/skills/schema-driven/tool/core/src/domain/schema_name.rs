//! スキーマの名前「<Skill の名前>/<種類>@<版>」。インスタンスの $schema とスキーマの $id に書く。
//! 名前でない値は、インスタンスのファイルからの相対パスとして読む（名前で指す形にする前の写しのため）。

/// 名前の形をしているか。Skill の名前と種類は英小文字 ・ 数字 ・ ハイフン、版は v と数字。
pub fn is_name(value: &str) -> bool {
    let Some((owner, rest)) = value.split_once('/') else {
        return false;
    };
    let Some((kind, version)) = rest.split_once('@') else {
        return false;
    };
    let word = |text: &str| {
        text.chars()
            .next()
            .is_some_and(|first| first.is_ascii_lowercase() || first.is_ascii_digit())
            && text.chars().all(|character| {
                character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
            })
    };
    let number = version.strip_prefix('v').is_some_and(|digits| {
        !digits.is_empty() && digits.chars().all(|digit| digit.is_ascii_digit())
    });
    word(owner) && word(kind) && number
}

/// 名前の頭の Skill の名前。名前の形でなければ None。
pub fn owner(value: &str) -> Option<&str> {
    if is_name(value) {
        value.split_once('/').map(|(owner, _)| owner)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_and_paths_are_told_apart() {
        assert!(is_name("acdr/record@v1"));
        assert!(is_name("schema-driven/abstract@v12"));
        assert!(!is_name("../acdr.schema.json"));
        assert!(!is_name("acdr.schema.json"));
        assert!(!is_name("acdr/record"));
        assert!(!is_name("Acdr/record@v1"));
        assert!(!is_name("acdr/record@1"));
        assert_eq!(owner("acdr/record@v1"), Some("acdr"));
        assert_eq!(owner("record.schema.json"), None);
    }
}
