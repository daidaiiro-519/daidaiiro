// Package domain は、中核の業務領域「作成と検証」のドメインモデル。
// 集約 ・ 値オブジェクト ・ ドメインサービスを持ち、JSON Schema で書けない不変条件だけを確かめる。
// ポートを呼ばず、I/O をしない。
package domain

// code は、error の種類の決まった名前。値は error-codes.json のキーと同じ。
type code string

// error の code。
const (
	ArgumentsInvalid   code = "arguments_invalid"
	HumanInCodingAgent code = "human_in_coding_agent"
	SchemaNotFound     code = "schema_not_found"
	SchemaDuplicate    code = "schema_duplicate"
	SchemaViolation    code = "schema_violation"
	InvalidJSON        code = "invalid_json"
	PathOutside        code = "path_outside"
	FileNotFound       code = "file_not_found"
	LogUnavailable     code = "log_unavailable"
	LogLocked          code = "log_locked"
)

// Failure は、error のログイベントになる違反。error を実装し、呼ぶ側は errors.As で取り出して code で分ける。
type Failure struct {
	Code                    code
	Path                    string
	InstanceLocation        string
	AbsoluteKeywordLocation string
	Message                 string
}

// Error は、code と message を返す。
func (f *Failure) Error() string {
	return string(f.Code) + ": " + f.Message
}

// Fail は、code ・ path ・ message を持つ Failure を返す。
func Fail(c code, path, message string) *Failure {
	return &Failure{Code: c, Path: path, Message: message}
}
