//! 値オブジェクトのテスト（テストレベル component）。
//! テストの名前は、宣言から取り出したテスト条件の ID を含む（例：vo_6_inv_1 は VO-6.INV-1）。

use schema_driven_core::domain::values::{
    Hash, InstancePath, JsonPatch, JsonValue, SchemaPath, Unfilled, ValidationError,
};

// VO-1 パス：INV-1 ファイルのパスの文字数は1以上
#[test]
fn vo_1_inv_1_rejects_empty_path() {
    assert!(InstancePath::new("").is_err());
    assert!(InstancePath::new("decls/UC-1.json").is_ok());
}

// VO-2 スキーマ：INV-1 スキーマのファイルのパスの文字数は1以上
#[test]
fn vo_2_inv_1_rejects_empty_schema_path() {
    assert!(SchemaPath::new("").is_err());
    assert!(SchemaPath::new("use_case.schema.json").is_ok());
}

// VO-5 検証エラー：INV-1 理由の文字数は1以上（プロパティは空でもよい：インスタンスの根を指す）
#[test]
fn vo_5_inv_1_rejects_empty_reason() {
    assert!(ValidationError::new("/name", "").is_err());
    assert!(ValidationError::new("", "\"x\" is not of type \"object\"").is_ok());
}

// VO-6 ハッシュ値：INV-1 16進の文字列の文字数は64
#[test]
fn vo_6_inv_1_rejects_wrong_length() {
    assert!(Hash::new(&"a".repeat(63)).is_err());
    assert!(Hash::new(&"a".repeat(65)).is_err());
    assert!(Hash::new(&"a".repeat(64)).is_ok());
}

// VO-7 JSON Patch：INV-1 操作の並びの文字数は1以上
#[test]
fn vo_7_inv_1_rejects_empty_patch() {
    assert!(JsonPatch::new("").is_err());
}

// VO-7 JSON Patch：OP-1 適用する。例 OK-1
#[test]
fn vo_7_op_1_ok_1_apply() {
    let patch = JsonPatch::new(r#"[{"op":"add","path":"/a","value":1}]"#).unwrap();
    let after = patch.apply(&JsonValue::new("{}")).unwrap();
    assert_eq!(after.as_str(), r#"{"a":1}"#);
}

// VO-10 JSON の値：OP-1 ハッシュ値を求める。例 OK-1
#[test]
fn vo_10_op_1_ok_1_hash() {
    let hash = JsonValue::new("{}").hash();
    assert_eq!(
        hash.as_str(),
        "44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a"
    );
}

// VO-12 未記入：INV-1 プロパティの文字数は1以上
#[test]
fn vo_12_inv_1_rejects_empty_property() {
    assert!(Unfilled::new("").is_err());
    assert!(Unfilled::new("/header").is_ok());
}
