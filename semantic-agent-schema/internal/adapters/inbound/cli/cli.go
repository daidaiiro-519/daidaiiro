// Package cli は、入力側のアダプター。引数を pflag で読んで JSON にし、arguments.schema.json で機械的に検証し、
// ユースケースを呼び、結果をログイベント（JSON の1行）で標準出力へ書く。
package cli

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"strconv"
	"strings"

	"github.com/spf13/pflag"

	authoring "github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/authoring/application"
	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/authoring/domain"
	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/publishedlanguage"
	record "github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/record/application"
	search "github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/search/application"
)

// Authoring は、中核の業務領域「作成と検証」のユースケース。
type Authoring struct {
	Create   authoring.Create
	Update   authoring.Update
	Validate authoring.Validate
	Judge    authoring.Judge
}

// Wiring は、入口が作って渡すアダプター。
type Wiring struct {
	// Arguments は、引数の JSON を arguments.schema.json で検証する。
	Arguments authoring.Validator
	// Open は、working tree と操作の記録を開き、中核の業務領域のユースケースを返す。close は操作の記録を閉じる。
	Open func(ctx context.Context, schemas []domain.Path) (uc Authoring, close func() error, err error)
	// OpenLog は、操作の記録を読むだけのために開き、ユースケース「操作の記録の参照」を返す。
	OpenLog func(ctx context.Context) (uc record.Log, close func() error, err error)
	// OpenSearch は、working tree を引き、ユースケース「検索」を返す。
	OpenSearch func(ctx context.Context) (uc search.Search, err error)
}

// IO は、標準入力 ・ 標準出力 ・ 標準エラー出力。
type IO struct {
	Stdin          io.Reader
	Stdout, Stderr io.Writer
}

// arguments は、引数の JSON。arguments.schema.json の形。Semantic は、semantic <ユースケース>（意味的な操作）のとき true。
type arguments struct {
	Semantic  bool     `json:"semantic,omitempty"`
	Command   string   `json:"command,omitempty"`
	Paths     []string `json:"paths"`
	Schemas   []string `json:"schemas,omitempty"`
	Schema    string   `json:"schema,omitempty"`
	JSON      *string  `json:"json,omitempty"`
	Patch     *string  `json:"patch,omitempty"`
	Range     []string `json:"range,omitempty"`
	All       bool     `json:"all,omitempty"`
	Judgments *string  `json:"judgments,omitempty"`
	Limit     *int     `json:"limit,omitempty"`
	Operation string   `json:"operation,omitempty"`
	Where     []where  `json:"where,omitempty"`
	Question  string   `json:"question,omitempty"`
	Matches   *string  `json:"matches,omitempty"`
}

// where は、--where <JSON Pointer>=<JSON> の1つ。値は JSON として読んだもの。
type where struct {
	Pointer string `json:"pointer"`
	Value   any    `json:"value"`
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

// run は、ログイベントを書けないときだけ error を返す。ほかの error は、rejected のログイベントにする。
func run(ctx context.Context, args []string, stdin io.Reader, out *output, w Wiring) error {
	parsed, err := parse(ctx, args, w.Arguments)
	if err != nil {
		return out.failures(parsed, err)
	}
	paths, pathErr := pathsOf(parsed.Paths)
	dirs, dirErr := pathsOf(parsed.Schemas)
	ranges, rangeErr := pathsOf(parsed.Range)
	if err := errors.Join(pathErr, dirErr, rangeErr); err != nil {
		return out.failures(parsed, err)
	}
	var content []byte
	for _, v := range []*string{parsed.JSON, parsed.Patch, parsed.Judgments, parsed.Matches} {
		if v == nil {
			continue
		}
		if content, err = valueOf(*v, stdin); err != nil {
			return out.failures(parsed, err)
		}
	}
	if parsed.Command == "log" {
		return list(ctx, parsed, paths, w, out)
	}
	if parsed.Command == "search" {
		return find(ctx, parsed, paths, content, w, out)
	}
	uc, closeRecords, err := w.Open(ctx, dirs)
	if err != nil {
		return out.failures(parsed, err)
	}
	err = dispatch(ctx, parsed, uc, paths, ranges, content, out)
	if closeErr := closeRecords(); closeErr != nil {
		return errors.Join(err, out.failures(parsed, closeErr))
	}
	return err
}

// parse は、引数を pflag で読んで JSON にし、arguments.schema.json で検証する（機械的な検証（JSON Schema））。
func parse(ctx context.Context, args []string, validator authoring.Validator) (arguments, error) {
	var parsed arguments
	if len(args) > 0 && args[0] == "semantic" {
		parsed.Semantic = true
		args = args[1:]
	}
	if len(args) > 0 {
		parsed.Command = args[0]
		args = args[1:]
	}
	flags := pflag.NewFlagSet(parsed.Command, pflag.ContinueOnError)
	flags.SetOutput(io.Discard)
	flags.SetInterspersed(true)
	schemas := flags.StringArray("schemas", nil, "Concrete を探すディレクトリ（繰り返せる）")
	schema := flags.String("schema", "", "作る JSON ドキュメントが従う Concrete の $id")
	document := flags.String("json", "", "AI が作った JSON ドキュメント。- なら標準入力から読む")
	patch := flags.String("patch", "", "AI が作った JSON Patch。- なら標準入力から読む")
	ranges := flags.StringArray("range", nil, "Concrete を更新するときの、範囲のインスタンスの path（繰り返せる）")
	all := flags.Bool("all", false, "変更点だけでなく、すべての rule とすべての instance location を判定する")
	judgments := flags.String("judgments", "", "AI が判定した judgments。- なら標準入力から読む")
	limit := flags.Int("limit", 0, "返す操作の件数の上限")
	operation := flags.String("operation", "", "1件を指す <command>:<id>")
	wheres := flags.StringArray("where", nil, "<JSON Pointer>=<JSON>。その instance location の値が JSON と同じもの（繰り返せる。すべてを満たすもの）")
	question := flags.String("question", "", "検索の問い")
	matches := flags.String("matches", "", "AI が問いで探した matches。- なら標準入力から読む")
	if err := flags.Parse(args); err != nil {
		return parsed, domain.Fail(domain.ArgumentsInvalid, "", err.Error())
	}
	parsed.Paths = append([]string{}, flags.Args()...)
	parsed.Schemas = *schemas
	parsed.Schema = *schema
	if flags.Changed("json") {
		parsed.JSON = document
	}
	if flags.Changed("patch") {
		parsed.Patch = patch
	}
	parsed.Range = *ranges
	parsed.All = *all
	if flags.Changed("judgments") {
		parsed.Judgments = judgments
	}
	if flags.Changed("limit") {
		parsed.Limit = limit
	}
	parsed.Operation = *operation
	parsed.Question = *question
	if flags.Changed("matches") {
		parsed.Matches = matches
	}
	for _, text := range *wheres {
		// 最初の = で JSON Pointer と値に分ける。値は JSON として読み、読めなければ invalid_json
		pointer, value, _ := strings.Cut(text, "=")
		parsedValue, err := domain.ParseJSON([]byte(value))
		if err != nil {
			return parsed, err
		}
		parsed.Where = append(parsed.Where, where{Pointer: pointer, Value: parsedValue})
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

// valueOf は、--json の JSON ドキュメント ・ --patch の JSON Patch ・ --judgments の judgments のバイト列を返す。- なら標準入力から読む。
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

// dispatch は、コマンドのユースケースを呼び、結果をログイベントで書く。semantic <ユースケース> は prompt を返し、
// semantic の無い <ユースケース> は、AI が作ったものを受け取って機械的に検証する。
func dispatch(ctx context.Context, args arguments, uc Authoring, paths, ranges []domain.Path, content []byte, out *output) error {
	switch {
	case args.Command == "update" && args.Semantic:
		prompt, err := uc.Update.Prompt(ctx, paths[0])
		if err != nil {
			return out.failures(args, err)
		}
		return out.requested(args, prompt, nil)
	case args.Command == "update":
		id, write, err := uc.Update.Run(ctx, authoring.UpdateInput{
			Path: paths[0], Patch: content, Range: ranges,
		})
		if err != nil {
			return out.failures(args, err)
		}
		return out.written(args, id, write)
	case args.Command == "create" && args.Semantic:
		prompt, err := uc.Create.Prompt(ctx, paths[0], args.Schema)
		if err != nil {
			return out.failures(args, err)
		}
		return out.requested(args, prompt, nil)
	case args.Command == "create":
		id, creation, err := uc.Create.Run(ctx, paths[0], content)
		if err != nil {
			return out.failures(args, err)
		}
		return out.written(args, id, creation)
	case args.Command == "validate" && args.Semantic:
		prompt, counted, err := uc.Judge.Prompt(ctx, paths[0], args.All)
		if err != nil {
			return out.failures(args, err)
		}
		return out.requested(args, prompt, counted)
	case args.Command == "validate" && args.Judgments != nil:
		id, validation, err := uc.Judge.Run(ctx, paths[0], content, args.All)
		if err != nil {
			return out.failures(args, err)
		}
		return out.validation(args, id, validation)
	default:
		for _, result := range uc.Validate.Run(ctx, paths) {
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

// list は、ユースケース「操作の記録の参照」を呼び、操作を1件ずつ listed のログイベントで書く。
func list(ctx context.Context, args arguments, paths []domain.Path, w Wiring, out *output) error {
	uc, closeRecords, err := w.OpenLog(ctx)
	if err != nil {
		return out.failures(args, err)
	}
	err = listOperations(ctx, args, paths, uc, out)
	if closeErr := closeRecords(); closeErr != nil {
		return errors.Join(err, out.failures(args, closeErr))
	}
	return err
}

func listOperations(ctx context.Context, args arguments, paths []domain.Path, uc record.Log, out *output) error {
	if args.Operation != "" {
		command, id, _ := strings.Cut(args.Operation, ":") // 形は arguments.schema.json の pattern が決めている
		number, err := strconv.ParseInt(id, 10, 64)
		if err != nil {
			return out.failures(args, domain.Fail(domain.ArgumentsInvalid, "", err.Error()))
		}
		operation, err := uc.One(ctx, command, number)
		if err != nil {
			return out.failures(args, err)
		}
		if operation == nil {
			return out.failures(args, domain.Fail(domain.OperationNotFound, "", args.Operation+" は操作の記録に無い"))
		}
		return out.listed(*operation)
	}
	count := 0
	if args.Limit != nil {
		count = *args.Limit
	}
	operations, err := uc.List(ctx, paths, count)
	if err != nil {
		return out.failures(args, err)
	}
	for _, operation := range operations {
		if err := out.listed(operation); err != nil {
			return err
		}
	}
	return nil
}

// find は、ユースケース「検索」を呼び、search_requested か found のログイベントを書く。
func find(ctx context.Context, args arguments, paths []domain.Path, content []byte, w Wiring, out *output) error {
	uc, err := w.OpenSearch(ctx)
	if err != nil {
		return out.failures(args, err)
	}
	if args.Semantic {
		documents, err := uc.Prompt(ctx, paths)
		if err != nil {
			return out.failures(args, err)
		}
		return out.searchRequested(args, documents)
	}
	var matches []search.Match
	if args.Matches != nil {
		matches, err = uc.Matches(ctx, paths, content)
	} else {
		matches, err = uc.Where(ctx, paths, conditionsOf(args.Where))
	}
	if err != nil {
		return out.failures(args, err)
	}
	return out.found(paths, matches)
}

// conditionsOf は、引数の where を検索の条件にする。JSON Pointer の形は arguments.schema.json が確かめている。
func conditionsOf(wheres []where) []domain.Condition {
	conditions := make([]domain.Condition, 0, len(wheres))
	for _, w := range wheres {
		pointer, err := domain.NewPointer(w.Pointer)
		if err != nil {
			continue // arguments.schema.json の pattern を通った形なので、ここには来ない
		}
		conditions = append(conditions, domain.Condition{InstanceLocation: pointer, Value: w.Value})
	}
	return conditions
}
