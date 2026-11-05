// Package files は、アダプター。ポート Documents と Schemas を、working tree の中のファイルで満たす。
package files

import (
	"context"
	"encoding/json"
	"errors"
	"io/fs"
	"os"
	"path/filepath"
	"strings"

	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/authoring/domain"
)

// Documents は、working tree の中の path のファイルを読む。
type Documents struct {
	top string
}

// NewDocuments は、working tree の top-level directory を基にする Documents を返す。
func NewDocuments(top string) *Documents {
	return &Documents{top: top}
}

// Read は、path のファイルのバイト列とハッシュを返す。無ければ file_not_found、
// シンボリックリンクの先が working tree の外なら path_outside の Failure を返す。
func (d *Documents) Read(_ context.Context, path domain.Path) (domain.Document, error) {
	resolved, err := resolve(d.top, path)
	if err != nil {
		return domain.Document{}, err
	}
	content, err := os.ReadFile(resolved)
	if err != nil {
		return domain.Document{}, domain.Fail(domain.FileNotFound, path.String(), err.Error())
	}
	return domain.Document{Path: path, Content: content, Hash: domain.HashOf(content)}, nil
}

// Write は、path に content を書く。不完全なファイルを残さないように、同じディレクトリに一時ファイルを書いてから置き換える。
// 無いディレクトリは作る。シンボリックリンクの先が working tree の外なら path_outside の Failure を返す。
func (d *Documents) Write(_ context.Context, path domain.Path, content []byte) error {
	root, err := filepath.EvalSymlinks(d.top)
	if err != nil {
		return domain.Fail(domain.FileNotFound, path.String(), err.Error())
	}
	dir := filepath.Dir(filepath.Join(root, filepath.FromSlash(path.String())))
	if err := inside(root, existing(dir), path); err != nil {
		return err
	}
	if err := os.MkdirAll(dir, 0o755); err != nil {
		return domain.Fail(domain.FileNotFound, path.String(), err.Error())
	}
	file, err := os.CreateTemp(dir, ".semantic-agent-schema-*")
	if err != nil {
		return domain.Fail(domain.FileNotFound, path.String(), err.Error())
	}
	_, writeErr := file.Write(content)
	if err := errors.Join(writeErr, file.Close()); err != nil {
		return errors.Join(domain.Fail(domain.FileNotFound, path.String(), err.Error()), os.Remove(file.Name()))
	}
	if err := os.Rename(file.Name(), filepath.Join(dir, filepath.Base(path.String()))); err != nil {
		return errors.Join(domain.Fail(domain.FileNotFound, path.String(), err.Error()), os.Remove(file.Name()))
	}
	return nil
}

// existing は、dir から上へたどって最初にある祖先のディレクトリを、シンボリックリンクをたどった先で返す。
func existing(dir string) string {
	for {
		if resolved, err := filepath.EvalSymlinks(dir); err == nil {
			return resolved
		}
		parent := filepath.Dir(dir)
		if parent == dir {
			return dir
		}
		dir = parent
	}
}

// inside は、集約 JSON ドキュメントの不変条件として、resolved が working tree の中になければ path_outside の Failure を返す。
func inside(root, resolved string, path domain.Path) error {
	if resolved != root && !strings.HasPrefix(resolved, root+string(filepath.Separator)) {
		return domain.Fail(domain.PathOutside, path.String(), resolved)
	}
	return nil
}

// resolve は、シンボリックリンクをたどった先の絶対パスを返す。集約 JSON ドキュメントの不変条件として、
// 先が working tree の外なら path_outside の Failure を返す。
func resolve(top string, path domain.Path) (string, error) {
	root, err := filepath.EvalSymlinks(top)
	if err != nil {
		return "", domain.Fail(domain.FileNotFound, path.String(), err.Error())
	}
	resolved, err := filepath.EvalSymlinks(filepath.Join(root, filepath.FromSlash(path.String())))
	if err != nil {
		return "", domain.Fail(domain.FileNotFound, path.String(), err.Error())
	}
	if err := inside(root, resolved, path); err != nil {
		return "", err
	}
	return resolved, nil
}

// Schemas は、schemas（AI が渡したディレクトリの中の *.json）から $id で Concrete を引く。
type Schemas struct {
	top     string
	dirs    []domain.Path
	builtin map[string][]byte
	index   map[string][]entry
	err     error
	indexed bool
}

type entry struct {
	path    domain.Path
	content []byte
}

// NewSchemas は、Schemas を返す。builtin は、公開された言葉の Abstract のように、schemas の外から渡す $id とバイト列。
func NewSchemas(top string, dirs []domain.Path, builtin map[string][]byte) *Schemas {
	return &Schemas{top: top, dirs: dirs, builtin: builtin}
}

// Lookup は、$id の Concrete を返す。無ければ schema_not_found、2つ以上あれば schema_duplicate の Failure を返す。
func (s *Schemas) Lookup(ctx context.Context, id string) (domain.Concrete, error) {
	if content, ok := s.builtin[id]; ok {
		return domain.NewConcrete(nil, content)
	}
	found, err := s.find(ctx, id)
	if err != nil {
		return nil, err
	}
	return domain.NewConcrete(found.path, found.content)
}

// Content は、$id の Concrete のバイト列を返す。JSON Schema の実装が $ref の先を引くときに使う。
func (s *Schemas) Content(ctx context.Context, id string) ([]byte, error) {
	if content, ok := s.builtin[id]; ok {
		return content, nil
	}
	found, err := s.find(ctx, id)
	if err != nil {
		return nil, err
	}
	return found.content, nil
}

// find は、$id の Concrete を引く。schemas の読み込みは最初の 1 回だけ行い、無ければ schema_not_found、2 つ以上あれば schema_duplicate を返す。
func (s *Schemas) find(_ context.Context, id string) (entry, error) {
	if !s.indexed {
		s.index, s.err = s.read()
		s.indexed = true
	}
	if s.err != nil {
		return entry{}, s.err
	}
	found := s.index[id]
	switch len(found) {
	case 0:
		return entry{}, domain.Fail(domain.SchemaNotFound, "", id)
	case 1:
		return found[0], nil
	default:
		return entry{}, domain.Fail(domain.SchemaDuplicate, "", id)
	}
}

// read は、schemas のディレクトリの中の *.json を読み、$id ごとにまとめる。JSON として読めないファイルは Concrete でないので飛ばす。
func (s *Schemas) read() (map[string][]entry, error) {
	index := map[string][]entry{}
	for _, dir := range s.dirs {
		resolved, err := resolve(s.top, dir)
		if err != nil {
			return nil, err
		}
		err = filepath.WalkDir(resolved, func(file string, d fs.DirEntry, err error) error {
			if err != nil {
				return err
			}
			if d.IsDir() || filepath.Ext(file) != ".json" {
				return nil
			}
			content, err := os.ReadFile(file)
			if err != nil {
				return err
			}
			var head struct {
				ID string `json:"$id"`
			}
			if json.Unmarshal(content, &head) != nil || head.ID == "" {
				return nil
			}
			rel, err := filepath.Rel(resolved, file)
			if err != nil {
				return err
			}
			path, err := domain.NewPath(dir.String() + "/" + filepath.ToSlash(rel))
			if err != nil {
				return err
			}
			index[head.ID] = append(index[head.ID], entry{path: path, content: content})
			return nil
		})
		if err != nil {
			return nil, domain.Fail(domain.FileNotFound, dir.String(), err.Error())
		}
	}
	return index, nil
}
