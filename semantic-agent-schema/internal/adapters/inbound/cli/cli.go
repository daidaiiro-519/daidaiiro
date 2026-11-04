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

// Wiring は、入口が作って渡すアダプター。
type Wiring struct {
	// Arguments は、引数の JSON を arguments.schema.json で検証する。
	Arguments application.Validator
	// Validate は、working tree と操作の記録を開き、ユースケース「検証」を返す。close は操作の記録を閉じる。
	Validate func(ctx context.Context, schemas []domain.Path) (uc application.Validate, close func() error, err error)
}

// arguments は、引数の JSON。arguments.schema.json の形。
type arguments struct {
	Command string   `json:"command,omitempty"`
	Paths   []string `json:"paths"`
	Schemas []string `json:"schemas,omitempty"`
	Human   bool     `json:"human,omitempty"`
}

// Run は、コマンドを1回実行し、終了コードを返す。error か violated の judgment があれば 1、ほかは 0。
// ログイベントを書けないとき（標準出力へ書けないなど）は、その error を stderr へ書き、1 を返す。
func Run(ctx context.Context, args []string, stdout, stderr io.Writer, w Wiring) int {
	out, err := newOutput(stdout)
	if err != nil {
		return broken(stderr, err)
	}
	if err := run(ctx, args, out, w); err != nil {
		return broken(stderr, err)
	}
	return out.exit()
}

func broken(stderr io.Writer, err error) int {
	// ログイベントを書けないので、stderr へ書く。stderr へも書けなければ、伝える先が無い
	fmt.Fprintln(stderr, err)
	return 1
}

// run は、ログイベントを書けないときだけ error を返す。ほかの error は、error のログイベントにする。
func run(ctx context.Context, args []string, out *output, w Wiring) error {
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
	if err := flags.Parse(args); err != nil {
		return out.failures(parsed, domain.Fail(domain.ArgumentsInvalid, "", err.Error()))
	}
	parsed.Paths = append([]string{}, flags.Args()...)
	parsed.Schemas = *schemas
	parsed.Human = *human

	// 機械的な検証（JSON Schema）：引数を JSON にして、arguments.schema.json で検証する
	content, err := json.Marshal(parsed)
	if err != nil {
		return err
	}
	violations, err := w.Arguments.Check(ctx, publishedlanguage.ArgumentsID, content)
	if err != nil {
		return out.failures(parsed, err)
	}
	if len(violations) > 0 {
		var failures []error
		for _, v := range violations {
			failures = append(failures, &domain.Failure{
				Code: domain.ArgumentsInvalid, InstanceLocation: v.InstanceLocation.String(),
				AbsoluteKeywordLocation: v.AbsoluteKeywordLocation, Message: v.Message,
			})
		}
		return out.failures(parsed, errors.Join(failures...))
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

	uc, closeRecords, err := w.Validate(ctx, dirs)
	if err != nil {
		return out.failures(parsed, err)
	}
	for _, result := range uc.Run(ctx, actor, paths) {
		if result.Err != nil {
			if err := out.failures(parsed, result.Err); err != nil {
				return errors.Join(err, closeRecords())
			}
			continue
		}
		if err := out.validation(parsed, result.ID, result.Validation); err != nil {
			return errors.Join(err, closeRecords())
		}
	}
	if err := closeRecords(); err != nil {
		return out.failures(parsed, err)
	}
	return nil
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
