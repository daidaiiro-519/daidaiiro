// Package domain は、中核の業務領域「作成と検証」のドメインモデル。
// 集約 ・ 値オブジェクト ・ ドメインサービスを持ち、JSON Schema で書けない不変条件だけを確かめる。
// ポートを呼ばず、I/O をしない。
package domain

import "errors"

// code は、error の種類の決まった名前。値は error-codes.json のキーと同じ。
type code string

// error の code。
const (
	ArgumentsInvalid         code = "arguments_invalid"
	SchemaNotFound           code = "schema_not_found"
	SchemaDuplicate          code = "schema_duplicate"
	SchemaViolation          code = "schema_violation"
	InvalidJSON              code = "invalid_json"
	ConventionViolation      code = "convention_violation"
	DecidedByHuman           code = "decided_by_human"
	PathOutside              code = "path_outside"
	FileExists               code = "file_exists"
	HashMismatch             code = "hash_mismatch"
	PatchNotApplicable       code = "patch_not_applicable"
	RangeRequired            code = "range_required"
	RangeViolation           code = "range_violation"
	ResponseMismatch         code = "response_mismatch"
	OperationNotFound        code = "operation_not_found"
	TemplateMissing          code = "template_missing"
	TemplateInvalid          code = "template_invalid"
	TemplateReferenceMissing code = "template_reference_missing"
	TokenAliasInvalid        code = "token_alias_invalid"
	OutOverlap               code = "out_overlap"
	FileNotFound             code = "file_not_found"
	NotInGit                 code = "not_in_git"
	LogUnavailable           code = "log_unavailable"
	LogLocked                code = "log_locked"
)

// Failure は、error のログイベントになる違反。error を実装し、呼ぶ側は errors.As で取り出して code で分ける。
type Failure struct {
	Code                    code
	Path                    string
	InstanceLocation        string
	AbsoluteKeywordLocation string
	Message                 string
	Line                    int // テンプレートの中の行。テンプレートの違反でなければ 0
}

// Error は、code と message を返す。
func (f *Failure) Error() string {
	return string(f.Code) + ": " + f.Message
}

// Fail は、code ・ path ・ message を持つ Failure を返す。
func Fail(c code, path, message string) *Failure {
	return &Failure{Code: c, Path: path, Message: message}
}

// FailuresOf は、違反ごとに code と path を持つ Failure を作り、errors.Join でまとめて返す。違反が無ければ nil を返す。
func FailuresOf(c code, path string, violations []Violation) error {
	var failures []error
	for _, v := range violations {
		failures = append(failures, &Failure{
			Code: c, Path: path, InstanceLocation: v.InstanceLocation.String(),
			AbsoluteKeywordLocation: v.AbsoluteKeywordLocation, Message: v.Message,
		})
	}
	return errors.Join(failures...)
}
