//! 実行ファイルに取り込んだ、基盤のスキーマ。
//! どれを取り込むかは build.rs が schemas/ から決める。使う側は名前（`$id`）で引く。

/// 取り込んだスキーマ1つ。名前はスキーマ自身の `$id` である。
pub struct Schema {
    pub name: &'static str,
    pub file: &'static str,
    pub text: &'static str,
}

include!(concat!(env!("OUT_DIR"), "/schemas.rs"));

/// 名前（`$id`）でスキーマを引く。
pub fn schema(name: &str) -> Option<&'static Schema> {
    SCHEMAS.iter().find(|schema| schema.name == name)
}

/// 名前（`$id`）でスキーマの中身を引く。取り込んでいない名前ならビルドの誤りなので止める。
pub fn schema_text(name: &str) -> &'static str {
    schema(name)
        .unwrap_or_else(|| panic!("{name} を取り込んでいない"))
        .text
}

#[cfg(test)]
mod tests {
    use super::SCHEMAS;

    /// 取り込んだ一覧を固定する。schemas/ にファイルを足したり消したりしたら、ここも直す。
    #[test]
    fn every_schema_is_taken_in_under_its_own_name() {
        let taken: Vec<(&str, &str)> = SCHEMAS
            .iter()
            .map(|schema| (schema.name, schema.file))
            .collect();
        assert_eq!(
            taken,
            vec![
                ("schema-driven/annotations@v1", "annotations.schema.json"),
                ("schema-driven/catalog@v1", "catalog.schema.json"),
                (
                    "schema-driven/design-system@v1",
                    "design-system.schema.json"
                ),
                ("schema-driven/document@v1", "document.schema.json"),
                ("schema-driven/abstract@v1", "meta-schema.json"),
                ("schema-driven/page@v1", "page.schema.json"),
                ("schema-driven/published@v1", "published.schema.json"),
                ("schema-driven/view@v1", "view.schema.json"),
            ]
        );
    }
}
