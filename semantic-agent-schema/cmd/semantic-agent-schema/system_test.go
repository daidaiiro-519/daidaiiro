package main_test

import (
	"bytes"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"regexp"
	"strconv"
	"strings"
	"testing"

	"github.com/google/go-cmp/cmp"
	"github.com/santhosh-tekuri/jsonschema/v6"
)

// scenario は、システムテストの場面。ユースケースの主成功シナリオか拡張1つを1場面にし、期待値はページの行から書く。
type scenario struct {
	Title   string                     `json:"title"`
	Git     *bool                      `json:"git"`
	Files   map[string]json.RawMessage `json:"files"`
	Raw     map[string]string          `json:"raw"`
	Outside map[string]json.RawMessage `json:"outside"`
	Links   map[string]string          `json:"links"`
	Steps   []step                     `json:"steps"`
}

type step struct {
	Remove []string          `json:"remove"`
	Args   []string          `json:"args"`
	Env    map[string]string `json:"env"`
	Stdin  string            `json:"stdin"`
	Exit   int               `json:"exit"`
	// Files は、ステップの後のファイルの内容。null はファイルが無いこと
	Files map[string]*string `json:"files"`
	// Events は、出力の行ごとの、JSON Pointer と値の組。行の数も比べる。message の文は比べない
	Events []map[string]json.RawMessage `json:"events"`
}

var binary string

func TestMain(m *testing.M) {
	dir, err := os.MkdirTemp("", "semantic-agent-schema-system")
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(2)
	}
	binary = filepath.Join(dir, "semantic-agent-schema")
	build := exec.Command("go", "build", "-o", binary, ".")
	build.Stderr = os.Stderr
	code := 2
	if err := build.Run(); err != nil {
		fmt.Fprintln(os.Stderr, err)
	} else {
		code = m.Run()
	}
	if err := os.RemoveAll(dir); err != nil {
		fmt.Fprintln(os.Stderr, err)
	}
	os.Exit(code)
}

func TestScenarios(t *testing.T) {
	t.Parallel()
	events := eventSchema(t)
	git, err := exec.LookPath("git")
	if err != nil {
		t.Fatalf("LookPath(git) = %v", err)
	}
	files, err := filepath.Glob("testdata/*/*.json")
	if err != nil || len(files) == 0 {
		t.Fatalf("Glob(testdata/*/*.json) = %v, %v", files, err)
	}
	for _, file := range files {
		if filepath.Base(filepath.Dir(file)) == "fixtures" {
			continue
		}
		content, err := os.ReadFile(file)
		if err != nil {
			t.Fatalf("ReadFile(%s) = %v", file, err)
		}
		var s scenario
		if err := json.Unmarshal(content, &s); err != nil {
			t.Fatalf("json.Unmarshal(%s) = %v", file, err)
		}
		t.Run(s.Title, func(t *testing.T) {
			t.Parallel()
			run(t, s, git, events)
		})
	}
}

func run(t *testing.T, s scenario, git string, events *jsonschema.Schema) {
	t.Helper()
	base := t.TempDir()
	repo := filepath.Join(base, "repo")
	write(t, repo, s.Files)
	write(t, base, s.Outside)
	for name, text := range s.Raw {
		writeFile(t, filepath.Join(repo, name), []byte(text))
	}
	for link, target := range s.Links {
		if err := os.Symlink(target, filepath.Join(repo, link)); err != nil {
			t.Fatalf("Symlink(%s, %s) = %v", target, link, err)
		}
	}
	// 子プロセスの環境変数は空から作る
	env := []string{"PATH=" + filepath.Dir(git), "HOME=" + base, "GIT_CONFIG_NOSYSTEM=1"}
	if s.Git == nil || *s.Git {
		cmd := exec.Command(git, "init", "-q", repo)
		cmd.Env = env
		if out, err := cmd.CombinedOutput(); err != nil {
			t.Fatalf("git init = %v: %s", err, out)
		}
	} else {
		// 一時ディレクトリより上の階層のリポジトリを見ないようにする
		env = append(env, "GIT_CEILING_DIRECTORIES="+base)
	}
	for _, st := range s.Steps {
		for _, name := range st.Remove {
			if err := os.Remove(filepath.Join(repo, name)); err != nil {
				t.Fatalf("Remove(%s) = %v", name, err)
			}
		}
		lines, exit := execute(t, repo, env, st)
		label := "semantic-agent-schema " + strings.Join(st.Args, " ")
		if exit != st.Exit {
			t.Errorf("%s exit = %d, want %d\n%s", label, exit, st.Exit, strings.Join(lines, "\n"))
		}
		if len(lines) != len(st.Events) {
			t.Errorf("%s = %d events, want %d\n%s", label, len(lines), len(st.Events), strings.Join(lines, "\n"))
		}
		for j := range min(len(lines), len(st.Events)) {
			compare(t, label, repo, lines[j], st.Events[j], events)
		}
		for name, want := range st.Files {
			content, err := os.ReadFile(filepath.Join(repo, name))
			switch {
			case want == nil && !errors.Is(err, os.ErrNotExist):
				t.Errorf("%s: ReadFile(%s) = %q, %v, want no file", label, name, content, err)
			case want != nil && (err != nil || string(content) != *want):
				t.Errorf("%s: ReadFile(%s) = %q, %v, want %q", label, name, content, err, *want)
			}
		}
	}
}

func execute(t *testing.T, repo string, env []string, st step) ([]string, int) {
	t.Helper()
	// 引数の {sha256 <path>} は、実行する前のそのファイルのハッシュにする
	args := make([]string, len(st.Args))
	for i, arg := range st.Args {
		args[i] = sha256Of.ReplaceAllStringFunc(arg, func(m string) string {
			return hashOf(t, repo, sha256Of.FindStringSubmatch(m)[1])
		})
	}
	cmd := exec.Command(binary, args...)
	cmd.Dir = repo
	cmd.Stdin = strings.NewReader(st.Stdin)
	cmd.Env = append([]string(nil), env...)
	for k, v := range st.Env {
		cmd.Env = append(cmd.Env, k+"="+v)
	}
	var stdout, stderr bytes.Buffer
	cmd.Stdout, cmd.Stderr = &stdout, &stderr
	err := cmd.Run()
	exit := 0
	var exitErr *exec.ExitError
	if errors.As(err, &exitErr) {
		exit = exitErr.ExitCode()
	} else if err != nil {
		t.Fatalf("Run(%v) = %v", st.Args, err)
	}
	if stderr.Len() > 0 {
		t.Errorf("semantic-agent-schema %v stderr = %q, want empty", st.Args, stderr.String())
	}
	if stdout.Len() == 0 {
		return nil, exit
	}
	return strings.Split(strings.TrimRight(stdout.String(), "\n"), "\n"), exit
}

// compare は、出力の1行を event.schema.json で検証し、JSON Pointer と値の組を比べる。
func compare(t *testing.T, label, repo, line string, want map[string]json.RawMessage, events *jsonschema.Schema) {
	t.Helper()
	value, err := jsonschema.UnmarshalJSON(strings.NewReader(line))
	if err != nil {
		t.Errorf("%s = %s, want JSON: %v", label, line, err)
		return
	}
	if err := events.Validate(value); err != nil {
		t.Errorf("%s = %s, want event.schema.json: %v", label, line, err)
	}
	for pointer, raw := range want {
		got, ok := lookup(value, pointer)
		if !ok {
			t.Errorf("%s %s = (none), want %s\n%s", label, pointer, raw, line)
			continue
		}
		if prefix, ok := matcher(t, raw, "$prefix"); ok {
			if text, _ := got.(string); !strings.HasPrefix(text, prefix) { // 文字列でなければ空と比べて不一致になる
				t.Errorf("%s %s = %v, want prefix %q", label, pointer, got, prefix)
			}
			continue
		}
		if diff := cmp.Diff(expected(t, repo, raw), got); diff != "" {
			t.Errorf("%s %s mismatch (-want +got):\n%s", label, pointer, diff)
		}
	}
}

// matcher は、期待値が {"<key>": "<文字列>"} の形なら、その文字列を返す。
func matcher(t *testing.T, raw json.RawMessage, key string) (string, bool) {
	t.Helper()
	var object map[string]json.RawMessage
	if json.Unmarshal(raw, &object) != nil || len(object) != 1 {
		return "", false
	}
	inner, ok := object[key]
	if !ok {
		return "", false
	}
	var text string
	if err := json.Unmarshal(inner, &text); err != nil {
		t.Fatalf("json.Unmarshal(%s) = %v", inner, err)
	}
	return text, true
}

var sha256Of = regexp.MustCompile(`\{sha256 ([^}]+)\}`)

// expected は、期待値を読む。{"$sha256": "<path>"} と、{"$format": "…{sha256 <path>}…"} の中のハッシュは、
// 試験の中でそのファイルのバイト列から求める。
func expected(t *testing.T, repo string, raw json.RawMessage) any {
	t.Helper()
	if path, ok := matcher(t, raw, "$sha256"); ok {
		return hashOf(t, repo, path)
	}
	if format, ok := matcher(t, raw, "$format"); ok {
		return sha256Of.ReplaceAllStringFunc(format, func(m string) string {
			return hashOf(t, repo, sha256Of.FindStringSubmatch(m)[1])
		})
	}
	value, err := jsonschema.UnmarshalJSON(bytes.NewReader(raw))
	if err != nil {
		t.Fatalf("UnmarshalJSON(%s) = %v", raw, err)
	}
	return value
}

func hashOf(t *testing.T, repo, path string) string {
	t.Helper()
	content, err := os.ReadFile(filepath.Join(repo, path))
	if err != nil {
		t.Fatalf("ReadFile(%s) = %v", path, err)
	}
	sum := sha256.Sum256(content)
	return "sha256:" + hex.EncodeToString(sum[:])
}

func lookup(value any, pointer string) (any, bool) {
	if pointer == "" {
		return value, true
	}
	for _, token := range strings.Split(pointer[1:], "/") {
		token = strings.ReplaceAll(strings.ReplaceAll(token, "~1", "/"), "~0", "~")
		switch v := value.(type) {
		case map[string]any:
			child, ok := v[token]
			if !ok {
				return nil, false
			}
			value = child
		case []any:
			i, err := strconv.Atoi(token)
			if err != nil || i < 0 || i >= len(v) {
				return nil, false
			}
			value = v[i]
		default:
			return nil, false
		}
	}
	return value, true
}

func write(t *testing.T, dir string, files map[string]json.RawMessage) {
	t.Helper()
	for name, raw := range files {
		var content []byte
		if fixture, ok := matcher(t, raw, "$fixture"); ok {
			var err error
			if content, err = os.ReadFile(filepath.Join("testdata", "fixtures", fixture)); err != nil {
				t.Fatalf("ReadFile(%s) = %v", fixture, err)
			}
		} else {
			var indented bytes.Buffer
			if err := json.Indent(&indented, raw, "", "  "); err != nil {
				t.Fatalf("json.Indent(%s) = %v", name, err)
			}
			content = indented.Bytes()
		}
		writeFile(t, filepath.Join(dir, name), content)
	}
}

func writeFile(t *testing.T, path string, content []byte) {
	t.Helper()
	if err := os.MkdirAll(filepath.Dir(path), 0o755); err != nil {
		t.Fatalf("MkdirAll(%s) = %v", path, err)
	}
	if err := os.WriteFile(path, content, 0o644); err != nil {
		t.Fatalf("WriteFile(%s) = %v", path, err)
	}
}

// eventSchema は、event.schema.json と、それが参照する procedure.schema.json ・ vocabulary.schema.json を読んで返す。
func eventSchema(t *testing.T) *jsonschema.Schema {
	t.Helper()
	compiler := jsonschema.NewCompiler()
	for _, name := range []string{"event.schema.json", "procedure.schema.json", "vocabulary.schema.json"} {
		content, err := os.ReadFile(filepath.Join("../../internal/publishedlanguage/v1", name))
		if err != nil {
			t.Fatalf("ReadFile(%s) = %v", name, err)
		}
		value, err := jsonschema.UnmarshalJSON(bytes.NewReader(content))
		if err != nil {
			t.Fatalf("UnmarshalJSON(%s) = %v", name, err)
		}
		object, _ := value.(map[string]any) // 公開された言葉のスキーマは、どれも $id を持つオブジェクト
		id, _ := object["$id"].(string)
		if err := compiler.AddResource(id, value); err != nil {
			t.Fatalf("AddResource(%s) = %v", name, err)
		}
	}
	schema, err := compiler.Compile("urn:semantic-agent-schema:event:v1")
	if err != nil {
		t.Fatalf("Compile(event.schema.json) = %v", err)
	}
	return schema
}
