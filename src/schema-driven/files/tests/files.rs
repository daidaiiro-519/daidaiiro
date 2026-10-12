//! 取り込んだ一覧を固定する。schemas/ と catalog/ にファイルを足したり消したりしたら、ここも直す。

use schema_driven_files::{CATALOG, SCHEMAS};

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
            ("schema-driven/tools@v1", "tools.schema.json"),
            ("schema-driven/view@v1", "view.schema.json"),
        ]
    );
}

#[test]
fn every_catalog_file_is_taken_in() {
    let taken: Vec<&str> = CATALOG.iter().map(|(file, _)| *file).collect();
    assert_eq!(
        taken,
        vec![
            "catalog.css",
            "catalog.js",
            "design-system.json",
            "parts.html"
        ]
    );
}
