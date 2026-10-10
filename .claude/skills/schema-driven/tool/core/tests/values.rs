//! 値オブジェクトのテスト。

use schema_driven_core::domain::values::{
    Hash, InstancePath, JsonPatch, JsonValue, SchemaPath, Unfilled, ValidationError,
};

// パス：ファイルのパスの文字数は1以上
#[test]
fn vo_1_inv_1_rejects_empty_path() {
    assert!(InstancePath::new("").is_err());
    assert!(InstancePath::new("declarations/UC-1.json").is_ok());
}

// スキーマ：スキーマのファイルのパスの文字数は1以上
#[test]
fn vo_2_inv_1_rejects_empty_schema_path() {
    assert!(SchemaPath::new("").is_err());
    assert!(SchemaPath::new("use_case.schema.json").is_ok());
}

// 検証エラー：理由の文字数は1以上（プロパティは空でもよい：インスタンスの根を指す）
#[test]
fn vo_5_inv_1_rejects_empty_reason() {
    assert!(ValidationError::new("/name", "").is_err());
    assert!(ValidationError::new("", "型が object でない（値：\"x\"）").is_ok());
}

// ハッシュ値：16進の文字列の文字数は64
#[test]
fn vo_6_inv_1_rejects_wrong_length() {
    assert!(Hash::new(&"a".repeat(63)).is_err());
    assert!(Hash::new(&"a".repeat(65)).is_err());
    assert!(Hash::new(&"a".repeat(64)).is_ok());
}

// JSON Patch：操作の並びの文字数は1以上
#[test]
fn vo_7_inv_1_rejects_empty_patch() {
    assert!(JsonPatch::new("").is_err());
}

// JSON Patch：適用する
#[test]
fn vo_7_op_1_ok_1_apply() {
    let patch = JsonPatch::new(r#"[{"op":"add","path":"/a","value":1}]"#).unwrap();
    let after = patch.apply(&JsonValue::new("{}")).unwrap();
    assert_eq!(after.as_str(), r#"{"a":1}"#);
}

// JSON の値：ハッシュ値を求める
#[test]
fn vo_10_op_1_ok_1_hash() {
    let hash = JsonValue::new("{}").hash();
    assert_eq!(
        hash.as_str(),
        "44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a"
    );
}

// 未記入：プロパティの文字数は1以上
#[test]
fn vo_12_inv_1_rejects_empty_property() {
    assert!(Unfilled::new("").is_err());
    assert!(Unfilled::new("/header").is_ok());
}
