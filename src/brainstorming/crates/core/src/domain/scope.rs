//! AI が作業した範囲の検査。作業の前と後のファイルの指紋を比べ、書き換えてよい範囲の外で変わったものを返す。

use std::collections::BTreeMap;

/// 書き換えてよい範囲か。範囲はファイルそのものか、フォルダ（その下のすべて）で書く。
fn allowed(path: &str, ranges: &[String]) -> bool {
    ranges.iter().any(|range| {
        let range = range.trim_end_matches('/');
        path == range || path.starts_with(&format!("{range}/"))
    })
}

/// before と after（場所 → 指紋）を比べ、ranges の外で、増えた ・ 消えた ・ 変わったファイルを場所の順に返す。
pub fn outside(
    before: &BTreeMap<String, String>,
    after: &BTreeMap<String, String>,
    ranges: &[String],
) -> Vec<String> {
    let mut paths: Vec<&String> = before.keys().chain(after.keys()).collect();
    paths.sort();
    paths.dedup();
    paths
        .into_iter()
        .filter(|path| before.get(*path) != after.get(*path))
        .filter(|path| !allowed(path, ranges))
        .cloned()
        .collect()
}

/// `git status --porcelain=v1 -z` の出力から、場所の一覧を読む（名前を変えたものは新しい名前）。
pub fn porcelain(output: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut entries = output.split('\0').filter(|entry| !entry.is_empty());
    while let Some(entry) = entries.next() {
        if entry.len() < 4 {
            continue;
        }
        let (code, path) = entry.split_at(3);
        out.push(path.to_owned());
        if code.starts_with('R') || code.starts_with('C') {
            entries.next();
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(items: &[(&str, &str)]) -> BTreeMap<String, String> {
        items
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect()
    }

    #[test]
    fn changes_inside_the_ranges_are_allowed_and_the_rest_are_reported() {
        let before = map(&[
            ("a.txt", "1"),
            (".brainstorming/b.json", "1"),
            ("keep.txt", "1"),
        ]);
        let after = map(&[
            ("a.txt", "2"),
            (".brainstorming/b.json", "2"),
            (".brainstorming/b/rounds/2.json", "1"),
            ("new.txt", "1"),
            ("keep.txt", "1"),
        ]);
        let ranges = [
            ".brainstorming/b.json".to_owned(),
            ".brainstorming/b/".to_owned(),
        ];
        assert_eq!(outside(&before, &after, &ranges), vec!["a.txt", "new.txt"]);
    }

    #[test]
    fn a_similar_name_is_not_inside_a_folder_range() {
        let after = map(&[
            (".brainstorming/b2/x.json", "1"),
            (".brainstorming/b.json.bak", "1"),
        ]);
        let ranges = [
            ".brainstorming/b".to_owned(),
            ".brainstorming/b.json".to_owned(),
        ];
        assert_eq!(
            outside(&BTreeMap::new(), &after, &ranges),
            vec![".brainstorming/b.json.bak", ".brainstorming/b2/x.json"]
        );
    }

    #[test]
    fn porcelain_reads_paths_and_takes_the_new_name_of_a_rename() {
        let output = " M a.txt\0?? dir/b.txt\0R  new.txt\0old.txt\0";
        assert_eq!(porcelain(output), vec!["a.txt", "dir/b.txt", "new.txt"]);
    }
}
