//! 基盤のエラー型は、Display と std::error::Error を実装する（Rust API Guidelines の C-GOOD-ERR）。
//! Display の文は短く、句点で終えない。具体は、基盤のエラーを `?` で自分のエラーへ渡せる。

use schema_driven_core::application::instances::UseCaseError;
use schema_driven_core::application::view::ViewError;
use schema_driven_core::domain::approval::ApprovalReject;
use schema_driven_core::domain::instance::Reject;
use schema_driven_core::domain::schema::SchemaError;
use schema_driven_core::domain::values::PatchError;
use schema_driven_core::ports::outbound::{QueryError, ReadError, WriteError};
use std::error::Error;

/// `?` で Box<dyn Error> へ渡す。渡せなければコンパイルが止まる。
fn pass_on<E: Error + 'static>(error: E) -> Result<(), Box<dyn Error>> {
    Err(error)?
}

fn shown<E: Error + 'static>(error: E) -> String {
    pass_on(error).unwrap_err().to_string()
}

#[test]
fn port_errors_describe_themselves() {
    assert_eq!(
        shown(ReadError("a.json が無い".into())),
        "読めない：a.json が無い"
    );
    assert_eq!(shown(WriteError::Conflict), "ほかの更新と競合した");
    assert_eq!(
        shown(WriteError::Unwritable("権限が無い".into())),
        "書けない：権限が無い"
    );
    assert_eq!(
        shown(QueryError("[ が閉じていない".into())),
        "JMESPath 式を読めない：[ が閉じていない"
    );
}

#[test]
fn use_case_errors_put_the_reason_first_and_omit_an_empty_detail() {
    let with_detail = UseCaseError {
        reason: "読めない".into(),
        detail: "a.json".into(),
        errors: vec![],
    };
    assert_eq!(shown(with_detail), "読めない：a.json");
    let without_detail = UseCaseError {
        reason: "ほかの更新と競合した".into(),
        detail: String::new(),
        errors: vec![],
    };
    assert_eq!(shown(without_detail), "ほかの更新と競合した");
}

#[test]
fn internal_errors_follow_the_same_form() {
    assert_eq!(
        shown(SchemaError("$ref の先が無い".into())),
        "スキーマが壊れている：$ref の先が無い"
    );
    assert_eq!(
        shown(PatchError("path が無い".into())),
        "JSON Patch を適用できない：path が無い"
    );
    assert_eq!(
        shown(ViewError("{ が閉じていない".into())),
        "x-view を文にできない：{ が閉じていない"
    );
    assert_eq!(shown(Reject::AlreadyExists), "パスにインスタンスが既にある");
    assert_eq!(shown(Reject::Invalid(vec![])), "検証を通過しない");
    assert_eq!(shown(ApprovalReject::Drift), "参照と導出値のずれがある");
}

#[test]
fn messages_do_not_end_with_punctuation() {
    let all = [
        shown(ReadError("x".into())),
        shown(WriteError::Conflict),
        shown(QueryError("x".into())),
        shown(Reject::Conflict),
        shown(ApprovalReject::Empty),
    ];
    for message in all {
        assert!(
            !message.ends_with('。') && !message.ends_with('.'),
            "{message}"
        );
    }
}
