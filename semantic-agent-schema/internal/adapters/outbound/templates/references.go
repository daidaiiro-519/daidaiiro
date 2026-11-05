package templates

import (
	"regexp"
	"strconv"
	"strings"
	"text/template/parse"

	tparse "github.com/tdewolff/parse/v2"
	"github.com/tdewolff/parse/v2/js"

	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/authoring/domain"
)

// References は、テンプレートを html/template と同じ構文木（text/template/parse）で読み、.Value と .Tokens の欄を集める。
// {{range}} ・ {{with}} の中の . と、変数に入れた欄もたどる。さらに、テンプレートの <script> の中の JS を読み、
// {{.Value…}} ・ {{.Tokens…}} を初期値にした変数から始まるメンバーの参照（for…of の要素を含む）を集める。
// 追える形は、変数の初期値 ・ for…of ・ 名前か数のリテラルで引くメンバーだけ。変数を作り直す ・ 計算した名前で引く ・
// 関数に渡す などは追わない。テンプレートか JS の構文が誤っていれば template_invalid を返す。
func (t *Templates) References(template domain.Template) ([]domain.Reference, error) {
	tree := parse.New(template.Path.String())
	tree.Mode = parse.SkipFuncCheck
	if _, err := tree.Parse(template.Text, "", "", map[string]*parse.Tree{}); err != nil {
		return nil, &domain.Failure{Code: domain.TemplateInvalid, Path: template.Path.String(), Line: lineOf(err.Error()), Message: err.Error()}
	}
	w := &walker{tree: tree, placeholders: map[string]ref{}}
	top := ref{top: true}
	w.list(tree.Root, scope{dot: top, vars: map[string]ref{"$": top}})
	scripts, err := w.scripts(template)
	if err != nil {
		return nil, err
	}
	return append(w.references, scripts...), nil
}

// ref は、構文木の中の値が指す欄。top はテンプレートに渡すデータそのもの（.Value と .Tokens を持つ）。
// known が false なら、追えない値。
type ref struct {
	top      bool
	known    bool
	root     string
	segments []domain.Segment
}

// field は、この値の欄 name を指す ref を返す。追えない値からは追えない値のままにする。
func (r ref) field(name string) ref {
	if r.top {
		return ref{known: true, root: name}
	}
	if !r.known {
		return r
	}
	return ref{known: true, root: r.root, segments: append(append([]domain.Segment(nil), r.segments...), domain.Segment{Name: name})}
}

// element は、この値が配列や map のとき、その要素を指す ref を返す。
func (r ref) element() ref {
	if r.top || !r.known {
		return ref{}
	}
	return ref{known: true, root: r.root, segments: append(append([]domain.Segment(nil), r.segments...), domain.Segment{Element: true})}
}

type scope struct {
	dot  ref
	vars map[string]ref
}

func (s scope) with(dot ref) scope { return scope{dot: dot, vars: s.vars} }

// bind は、変数 name が r を指す scope を返す。元の scope の変数は変えない。
func (s scope) bind(name string, r ref) scope {
	vars := map[string]ref{}
	for k, v := range s.vars {
		vars[k] = v
	}
	vars[name] = r
	return scope{dot: s.dot, vars: vars}
}

// walker は、テンプレートの構文木をたどり、欄を集め、<script> を読むための平らな文字列を作る。
// 平らな文字列では、1つの欄を出す {{…}} を JS の識別子 __sasRefN__ に、ほかの {{…}} を __sasExpr__ に置き換える。
type walker struct {
	tree         *parse.Tree
	references   []domain.Reference
	flat         strings.Builder
	placeholders map[string]ref
}

// list は、ノードの並びを順にたどる。途中で作った変数は、並びの後ろのノードでも使える。
func (w *walker) list(list *parse.ListNode, s scope) scope {
	if list == nil {
		return s
	}
	for _, n := range list.Nodes {
		s = w.node(n, s)
	}
	return s
}

// node は、ノード1つをたどり、{{$x := …}} で変数を作れば、その変数を持つ scope を返す。
func (w *walker) node(n parse.Node, s scope) scope {
	switch n := n.(type) {
	case *parse.TextNode:
		w.flat.Write(n.Text)
	case *parse.ActionNode:
		w.collect(n.Pipe, s)
		r := w.resolve(n.Pipe, s)
		if len(n.Pipe.Decl) > 0 {
			return s.bind(n.Pipe.Decl[0].Ident[0], r)
		}
		if r.known {
			name := "__sasRef" + strconv.Itoa(len(w.placeholders)) + "__"
			w.placeholders[name] = r
			w.flat.WriteString(name)
		} else {
			w.flat.WriteString("__sasExpr__")
		}
	case *parse.IfNode:
		w.collect(n.Pipe, s)
		w.list(n.List, s)
		w.list(n.ElseList, s)
	case *parse.WithNode:
		w.collect(n.Pipe, s)
		w.list(n.List, s.with(w.resolve(n.Pipe, s)))
		w.list(n.ElseList, s)
	case *parse.RangeNode:
		w.collect(n.Pipe, s)
		element := w.resolve(n.Pipe, s).element()
		inner := s.with(element)
		switch len(n.Pipe.Decl) {
		case 1:
			inner = inner.bind(n.Pipe.Decl[0].Ident[0], element)
		case 2:
			inner = inner.bind(n.Pipe.Decl[0].Ident[0], ref{}).bind(n.Pipe.Decl[1].Ident[0], element)
		}
		w.list(n.List, inner)
		w.list(n.ElseList, s)
	}
	return s
}

// resolve は、1つの値だけの pipeline が指す欄を返す。関数を呼ぶ pipeline は追わない。
func (w *walker) resolve(pipe *parse.PipeNode, s scope) ref {
	if pipe == nil || len(pipe.Cmds) != 1 || len(pipe.Cmds[0].Args) != 1 {
		return ref{}
	}
	return w.value(pipe.Cmds[0].Args[0], s)
}

// value は、{{.a.b}} や {{$x.a}} のような値が指す欄を返す。欄を追えない値は、追えない ref を返す。
func (w *walker) value(n parse.Node, s scope) ref {
	switch n := n.(type) {
	case *parse.DotNode:
		return s.dot
	case *parse.FieldNode:
		r := s.dot
		for _, name := range n.Ident {
			r = r.field(name)
		}
		return r
	case *parse.VariableNode:
		r, ok := s.vars[n.Ident[0]]
		if !ok {
			return ref{}
		}
		for _, name := range n.Ident[1:] {
			r = r.field(name)
		}
		return r
	case *parse.ChainNode:
		r := w.value(n.Node, s)
		for _, name := range n.Field {
			r = r.field(name)
		}
		return r
	case *parse.PipeNode:
		return w.resolve(n, s)
	}
	return ref{}
}

// collect は、pipeline の中で欄を読む引数（関数の引数を含む）をすべて集める。
func (w *walker) collect(pipe *parse.PipeNode, s scope) {
	if pipe == nil {
		return
	}
	for _, cmd := range pipe.Cmds {
		for _, arg := range cmd.Args {
			if p, ok := arg.(*parse.PipeNode); ok {
				w.collect(p, s)
			}
			if r := w.value(arg, s); r.known {
				w.record(r, w.line(arg))
			}
		}
	}
}

// record は、読んだ欄とその行を references に足す。
func (w *walker) record(r ref, line int) {
	w.references = append(w.references, domain.Reference{Root: r.root, Segments: r.segments, Line: line})
}

// line は、ノードのあるテンプレートの行を返す。
func (w *walker) line(n parse.Node) int {
	location, _ := w.tree.ErrorContext(n)
	return lineOf(location)
}

var (
	lineNumber = regexp.MustCompile(`:(\d+)(?::\d+)?\b`)
	scriptTag  = regexp.MustCompile(`(?is)<script\b([^>]*)>(.*?)</script>`)
	scriptType = regexp.MustCompile(`(?i)\btype\s*=\s*["']?([^"'\s>]+)`)
)

// lineOf は、構文木の位置か誤りの文（name:行:桁）から行を返す。読めなければ 0。
func lineOf(text string) int {
	m := lineNumber.FindStringSubmatch(text)
	if m == nil {
		return 0
	}
	line, err := strconv.Atoi(m[1])
	if err != nil {
		return 0
	}
	return line
}

// scripts は、平らな文字列の <script> ごとに JS を読み、欄を集める。type が JS でない <script>（JSON など）は読まない。
func (w *walker) scripts(template domain.Template) ([]domain.Reference, error) {
	var references []domain.Reference
	for _, m := range scriptTag.FindAllStringSubmatch(w.flat.String(), -1) {
		if t := scriptType.FindStringSubmatch(m[1]); t != nil {
			kind := strings.ToLower(t[1])
			if kind != "module" && !strings.Contains(kind, "javascript") {
				continue
			}
		}
		ast, err := js.Parse(tparse.NewInputString(m[2]), js.Options{})
		if err != nil {
			return nil, &domain.Failure{
				Code: domain.TemplateInvalid, Path: template.Path.String(), Line: textLine(template.Text, firstLine(m[2])),
				Message: "<script> の JS：" + err.Error(),
			}
		}
		v := &visitor{placeholders: w.placeholders, bindings: map[string]ref{}}
		js.Walk(v, ast)
		for _, found := range v.found {
			references = append(references, domain.Reference{
				Root: found.ref.root, Segments: found.ref.segments, Line: textLine(template.Text, found.text),
			})
		}
	}
	return references, nil
}

func firstLine(text string) string {
	line, _, _ := strings.Cut(strings.TrimSpace(text), "\n")
	return line
}

// textLine は、テンプレートの中で text が最初に出る行を返す。見つからなければ 0。
// JS の構文木は位置を持たないので、読んだ式の文字列で行を探す。
func textLine(template, text string) int {
	i := strings.Index(template, text)
	if i < 0 || text == "" {
		return 0
	}
	return strings.Count(template[:i], "\n") + 1
}

// visitor は、JS の構文木をたどり、欄に結び付いた変数と、そこから始まるメンバーの参照を集める。
type visitor struct {
	placeholders map[string]ref
	bindings     map[string]ref
	found        []jsReference
}

type jsReference struct {
	ref  ref
	text string // 行を探すための、ソースの中の式の文字列
}

func (v *visitor) Enter(n js.INode) js.IVisitor {
	switch n := n.(type) {
	case *js.VarDecl:
		for _, element := range n.List {
			name, ok := element.Binding.(*js.Var)
			if !ok || element.Default == nil {
				continue
			}
			if r, _, ok := v.resolve(element.Default); ok {
				v.bindings[string(name.Data)] = r
			}
		}
	case *js.ForOfStmt:
		declaration, ok := n.Init.(*js.VarDecl)
		if !ok || len(declaration.List) != 1 {
			break
		}
		name, ok := declaration.List[0].Binding.(*js.Var)
		if !ok {
			break
		}
		if r, _, ok := v.resolve(n.Value); ok {
			v.bindings[string(name.Data)] = r.element()
		}
	case *js.DotExpr, *js.IndexExpr:
		if r, text, ok := v.resolve(n.(js.IExpr)); ok {
			v.found = append(v.found, jsReference{ref: r, text: text})
		}
	}
	return v
}

func (v *visitor) Exit(js.INode) {}

// resolve は、式が指す欄と、ソースの中の式の文字列を返す。追えない式なら false。
func (v *visitor) resolve(e js.IExpr) (ref, string, bool) {
	switch e := e.(type) {
	case *js.Var:
		name := string(e.Data)
		if r, ok := v.placeholders[name]; ok {
			return r, name, true
		}
		r, ok := v.bindings[name]
		return r, name, ok
	case *js.DotExpr:
		r, text, ok := v.resolve(e.X)
		if !ok {
			return ref{}, "", false
		}
		var name string
		switch y := e.Y.(type) {
		case js.LiteralExpr:
			name = string(y.Data)
		case *js.LiteralExpr:
			name = string(y.Data)
		case *js.Var:
			name = string(y.Data)
		default:
			return ref{}, "", false
		}
		return r.field(name), text + "." + name, true
	case *js.IndexExpr:
		r, text, ok := v.resolve(e.X)
		var data string
		switch y := e.Y.(type) {
		case js.LiteralExpr:
			data = string(y.Data)
		case *js.LiteralExpr:
			data = string(y.Data)
		default:
			return ref{}, "", false
		}
		if !ok {
			return ref{}, "", false
		}
		if _, err := strconv.Atoi(data); err == nil {
			return r.element(), text + "[" + data + "]", true
		}
		if unquoted, err := strconv.Unquote(strings.ReplaceAll(data, "'", `"`)); err == nil {
			return r.field(unquoted), text + "[" + data + "]", true
		}
	}
	return ref{}, "", false
}
