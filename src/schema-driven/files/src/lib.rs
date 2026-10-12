//! 実行ファイルに取り込んだ、基盤のスキーマと部品のカタログ。
//! どれを取り込むかは build.rs が schemas/ と catalog/ から決める。使う側は名前かファイル名で引く。

/// 取り込んだスキーマ1つ。名前はスキーマ自身の `$id` である。
pub struct Schema {
    pub name: &'static str,
    pub file: &'static str,
    pub text: &'static str,
}

include!(concat!(env!("OUT_DIR"), "/files.rs"));

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

/// ファイル名で部品のカタログのファイルを引く。取り込んでいないファイルならビルドの誤りなので止める。
pub fn catalog(file: &str) -> &'static str {
    CATALOG
        .iter()
        .find(|(name, _)| *name == file)
        .map(|(_, text)| *text)
        .unwrap_or_else(|| panic!("catalog/{file} を取り込んでいない"))
}
