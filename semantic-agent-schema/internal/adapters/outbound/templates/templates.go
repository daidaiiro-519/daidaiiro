// Package templates は、アダプター。ポート Renderer を標準の html/template で満たす。テンプレートに関数は足さず
// （Funcs を呼ばない）、渡すのは .Value と .Tokens だけ。無い名前を引けば missingkey=error で失敗する。
// 全部を描いてから、一時ファイルに書いて置き換える。
package templates

import (
	"bytes"
	"context"
	"errors"
	"html/template"
	"os"
	"path/filepath"
	"strings"

	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/authoring/domain"
)

// Templates は、working tree の中へ HTML を書き出す。
type Templates struct {
	top string
}

// New は、working tree の top-level directory を基にする Templates を返す。
func New(top string) *Templates {
	return &Templates{top: top}
}

// Render は、すべての Page を描いてから書き出す。描けないテンプレートがあれば template_invalid を返し、何も書き出さない。
// 書き出す先がシンボリックリンクで working tree の外なら path_outside を返す。
func (t *Templates) Render(_ context.Context, pages []domain.Page) error {
	rendered := make([][]byte, len(pages))
	var failures []error
	for i, p := range pages {
		parsed, err := template.New(p.Out.String()).Option("missingkey=error").Parse(p.Template)
		if err != nil {
			failures = append(failures, domain.Fail(domain.TemplateInvalid, p.Out.String(), err.Error()))
			continue
		}
		var b bytes.Buffer
		if err := parsed.Execute(&b, map[string]any{"Value": p.Value, "Tokens": p.Tokens}); err != nil {
			failures = append(failures, domain.Fail(domain.TemplateInvalid, p.Out.String(), err.Error()))
			continue
		}
		rendered[i] = b.Bytes()
	}
	if err := errors.Join(failures...); err != nil {
		return err
	}
	root, err := filepath.EvalSymlinks(t.top)
	if err != nil {
		return domain.Fail(domain.FileNotFound, "", err.Error())
	}
	for _, p := range pages {
		if err := inside(root, p.Out); err != nil {
			return err
		}
	}
	for i, p := range pages {
		if err := write(filepath.Join(root, filepath.FromSlash(p.Out.String())), rendered[i]); err != nil {
			return domain.Fail(domain.FileNotFound, p.Out.String(), err.Error())
		}
	}
	return nil
}

// inside は、書き出す先のある祖先のディレクトリを、シンボリックリンクをたどって working tree の中かを確かめる。
func inside(root string, out domain.Path) error {
	dir := filepath.Dir(filepath.Join(root, filepath.FromSlash(out.String())))
	for {
		resolved, err := filepath.EvalSymlinks(dir)
		if err == nil {
			if resolved != root && !strings.HasPrefix(resolved, root+string(filepath.Separator)) {
				return domain.Fail(domain.PathOutside, out.String(), resolved)
			}
			return nil
		}
		parent := filepath.Dir(dir)
		if parent == dir {
			return nil
		}
		dir = parent
	}
}

// write は、同じディレクトリに一時ファイルを書いてから置き換える。無いディレクトリは作る。
func write(file string, content []byte) error {
	if err := os.MkdirAll(filepath.Dir(file), 0o755); err != nil {
		return err
	}
	temp, err := os.CreateTemp(filepath.Dir(file), ".semantic-agent-schema-*")
	if err != nil {
		return err
	}
	_, writeErr := temp.Write(content)
	if err := errors.Join(writeErr, temp.Close()); err != nil {
		return errors.Join(err, os.Remove(temp.Name()))
	}
	if err := os.Rename(temp.Name(), file); err != nil {
		return errors.Join(err, os.Remove(temp.Name()))
	}
	return nil
}
