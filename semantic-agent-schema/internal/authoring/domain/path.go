package domain

import "strings"

// Path は、値オブジェクト path。working tree の top-level directory からの相対のパスを表す。
type Path interface {
	String() string
	path()
}

type relativePath struct{ value string }

func (p relativePath) String() string { return p.value }
func (relativePath) path()            {}

// NewPath は、path を返す。集約 JSON ドキュメントの不変条件として、working tree の外を指す形
// （絶対パス ・ 空 ・ . と .. の要素 ・ 空の要素）なら path_outside の Failure を返す。
// シンボリックリンクの先は、ファイルシステムを見るアダプターが確かめる。
func NewPath(value string) (Path, error) {
	if value == "" || strings.HasPrefix(value, "/") {
		return nil, outside(value)
	}
	for _, segment := range strings.Split(value, "/") {
		if segment == "" || segment == "." || segment == ".." {
			return nil, outside(value)
		}
	}
	return relativePath{value}, nil
}

func outside(value string) *Failure {
	return Fail(PathOutside, value, value+" は working tree の外を指す")
}
