// Package cli は、入力側のアダプター。引数を pflag で読んで JSON にし、arguments.schema.json で機械的に検証し、
// 主アクターを決めてユースケースを呼び、結果をログイベント（JSON の1行）で標準出力へ書く。
package cli

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"os"

	"github.com/spf13/pflag"

	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/authoring/application"
	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/authoring/domain"
	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/publishedlanguage"
)

// codingAgentVariables は、coding agent の中で実行していることを示す環境変数の名前。有無だけを見て、値は読まない。
var codingAgentVariables = []string{"CLAUDE_CODE_SESSION_ID", "CODEX_SESSION_ID", "COPILOT_AGENT_SESSION_ID"}

// Authoring は、中核の業務領域「作成と検証」のユースケース。
type Authoring struct {
	Create   application.Create
	Validate application.Validate
}

// Wiring は、入口が作って渡すアダプター。
type Wiring struct {
	// Arguments は、引数の JSON を arguments.schema.json で検証する。
	Arguments application.Validator
	// Open は、working tree と操作の記録を開き、ユースケースを返す。close は操作の記録を閉じる。
	Open func(ctx context.Context, schemas []domain.Path) (uc Authoring, close func() error, err error)
}

// IO は、標準入力 ・ 標準出力 ・ 標準エラー出力。
type IO struct {
	Stdin          io.Reader
	Stdout, Stderr io.Writer
}

// arguments は、引数の JSON。arguments.schema.json の形。
type arguments struct {
	Command  string   `json:"command,omitempty"`
	Paths    []string `json:"paths"`
	Schemas  []string `json:"schemas,omitempty"`
	Human    bool     `json:"human,omitempty"`
	Semantic bool     `json:"semantic,omitempty"`
	SchemaID string   `json:"schemaId,omitempty"`
	Value    *string  `json:"value,omitempty"`
}

// Run は、コマンドを1回実行し、終了コードを返す。error か violated の judgment があれば 1、ほかは 0。
// ログイベントを書けないとき（標準出力へ書けないなど）は、その error を標準エラー出力へ書き、1 を返す。
func Run(ctx context.Context, args []string, std IO, w Wiring) int {
	out, err := newOutput(std.Stdout)
	if err != nil {
		return broken(std.Stderr, err)
	}
	if err := run(ctx, args, std.Stdin, out, w); err != nil {
		return broken(std.Stderr, err)
	}
	return out.exit()
}

func broken(stderr io.Writer, err error) int {
	// ログイベントを書けないので、標準エラー出力へ書く。そこへも書けなければ、伝える先が無い
	fmt.Fprintln(stderr, err)
	return 1
}

// run は、ログイベントを書けないときだけ error を返す。ほかの error は、error のログイベントにする。
func run(ctx context.Context, args []string, stdin io.Reader, out *output, w Wiring) error {
	parsed, err := parse(ctx, args, w.Arguments)
	if err != nil {
		return out.failures(parsed, err)
	}
	actor, err := domain.NewActor(parsed.Human, inCodingAgent())
	if err != nil {
		return out.failures(parsed, err)
	}
	paths, pathErr := pathsOf(parsed.Paths)
	dirs, dirErr := pathsOf(parsed.Schemas)
	if err := errors.Join(pathErr, dirErr); err != nil {
		return out.failures(parsed, err)
	}
	var content []byte
	if parsed.Value != nil {
		if content, err = valueOf(*parsed.Value, stdin); err != nil {
			return out.failures(parsed, err)
		}
	}
	uc, closeRecords, err := w.Open(ctx, dirs)
	if err != nil {
		return out.failures(parsed, err)
	}
	err = dispatch(ctx, parsed, uc, actor, paths, content, out)
	if closeErr := closeRecords(); closeErr != nil {
		return errors.Join(err, out.failures(parsed, closeErr))
	}
	return err
}

// parse は、引数を pflag で読んで JSON にし、arguments.schema.json で検証する（機械的な検証（JSON Schema））。
func parse(ctx context.Context, args []string, validator application.Validator) (arguments, error) {
	var parsed arguments
	if len(args) > 0 {
		parsed.Command = args[0]
		args = args[1:]
	}
	flags := pflag.NewFlagSet(parsed.Command, pflag.ContinueOnError)
	flags.SetOutput(io.Discard)
	flags.SetInterspersed(true)
	schemas := flags.StringArray("schemas", nil, "Concrete を探すディレクトリ（繰り返せる）")
	human := flags.Bool("human", false, "主アクターを human にする")
	semantic := flags.Bool("semantic", false, "procedure を依頼する")
	schemaID := flags.String("schema-id", "", "作る JSON ドキュメントが従う Concrete の $id")
	value := flags.String("value", "", "JSON ドキュメント。- なら標準入力から読む")
	if err := flags.Parse(args); err != nil {
		return parsed, domain.Fail(domain.ArgumentsInvalid, "", err.Error())
	}
	parsed.Paths = append([]string{}, flags.Args()...)
	parsed.Schemas = *schemas
	parsed.Human = *human
	parsed.Semantic = *semantic
	parsed.SchemaID = *schemaID
	if flags.Changed("value") {
		parsed.Value = value
	}
	content, err := json.Marshal(parsed)
	if err != nil {
		return parsed, err
	}
	violations, err := validator.Check(ctx, publishedlanguage.ArgumentsID, content)
	if err != nil {
		return parsed, err
	}
	return parsed, domain.FailuresOf(domain.ArgumentsInvalid, "", violations)
}

// valueOf は、--value の JSON ドキュメントのバイト列を返す。- なら標準入力から読む。
func valueOf(value string, stdin io.Reader) ([]byte, error) {
	if value != "-" {
		return []byte(value), nil
	}
	content, err := io.ReadAll(stdin)
	if err != nil {
		return nil, domain.Fail(domain.InvalidJSON, "", err.Error())
	}
	return content, nil
}

// dispatch は、コマンドのユースケースを呼び、結果をログイベントで書く。
func dispatch(ctx context.Context, args arguments, uc Authoring, actor domain.Actor, paths []domain.Path, content []byte, out *output) error {
	switch {
	case args.Command == "create" && args.Semantic:
		procedure, err := uc.Create.Procedure(ctx, paths[0], args.SchemaID)
		if err != nil {
			return out.failures(args, err)
		}
		return out.procedure(args, procedure)
	case args.Command == "create":
		id, creation, err := uc.Create.Run(ctx, actor, paths[0], content)
		if err != nil {
			return out.failures(args, err)
		}
		return out.change(args, id, creation)
	default:
		for _, result := range uc.Validate.Run(ctx, actor, paths) {
			if result.Err != nil {
				if err := out.failures(args, result.Err); err != nil {
					return err
				}
				continue
			}
			if err := out.validation(args, result.ID, result.Validation); err != nil {
				return err
			}
		}
		return nil
	}
}

func inCodingAgent() bool {
	for _, name := range codingAgentVariables {
		if _, ok := os.LookupEnv(name); ok {
			return true
		}
	}
	return false
}

func pathsOf(values []string) ([]domain.Path, error) {
	var paths []domain.Path
	var failures []error
	for _, value := range values {
		path, err := domain.NewPath(value)
		if err != nil {
			failures = append(failures, err)
			continue
		}
		paths = append(paths, path)
	}
	return paths, errors.Join(failures...)
}
