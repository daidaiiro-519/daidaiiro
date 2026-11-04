package domain

import (
	"errors"
	"slices"
	"strconv"
)

// Creation は、集約 JSON ドキュメントを作成する書く操作。作る JSON ドキュメント ・ 従う Concrete ・ 主アクター ・
// instance location ごとの change を持つ。不変条件：x-decided-by: human の値は、主アクターが human のときだけ default から変わる。
type Creation struct {
	actor    Actor
	document Document
	concrete Concrete
	changes  []Change
}

// CreationInput は、作成を作るために application が読んだもの。
type CreationInput struct {
	Actor    Actor
	Path     Path
	Content  []byte
	Value    any // Content を ParseJSON で読んだ値
	Concrete Concrete
}

// NewCreation は、作成を返す。主アクターが ai で、x-decided-by: human のサブスキーマに当たるインスタンスの値が
// default と違えば（default が無ければ、どの値も）、その instance location ごとの decided_by_human の Failure をまとめて返す。
func NewCreation(in CreationInput) (*Creation, error) {
	if in.Actor.Kind() == AI {
		var failures []error
		decidedByHuman(in.Concrete.structure(), in.Value, nil, func(location Pointer, value, def any, hasDefault bool) {
			if hasDefault && jsonText(value) == jsonText(def) {
				return
			}
			failures = append(failures, &Failure{
				Code: DecidedByHuman, Path: in.Path.String(), InstanceLocation: location.String(),
				Message: "x-decided-by: human の値を、主アクター ai が default から変えた：" + jsonText(value),
			})
		})
		if err := errors.Join(failures...); err != nil {
			return nil, err
		}
	}
	return &Creation{
		actor:    in.Actor,
		document: Document{Path: in.Path, Content: in.Content, Hash: HashOf(in.Content)},
		concrete: in.Concrete,
		changes:  changesOf(nil, false, in.Value),
	}, nil
}

// decidedByHuman は、インスタンスと Concrete を JSON Schema の構造（properties と items）で一緒にたどり、
// x-decided-by: human のサブスキーマに当たるインスタンスの値ごとに found を呼ぶ。配列は要素ごとにたどる。
func decidedByHuman(subschema map[string]any, value any, at []string, found func(location Pointer, value, def any, hasDefault bool)) {
	if subschema["x-decided-by"] == "human" {
		def, hasDefault := subschema["default"]
		found(PointerOf(at...), value, def, hasDefault)
	}
	if properties, ok := subschema["properties"].(map[string]any); ok {
		object, _ := value.(map[string]any) // オブジェクトでなければ、たどる値が無い
		for _, name := range sortedNames(properties) {
			child, isSchema := properties[name].(map[string]any)
			v, present := object[name]
			if isSchema && present {
				decidedByHuman(child, v, append(slices.Clone(at), name), found)
			}
		}
	}
	if items, ok := subschema["items"].(map[string]any); ok {
		array, _ := value.([]any) // 配列でなければ、たどる値が無い
		for i, v := range array {
			decidedByHuman(items, v, append(slices.Clone(at), strconv.Itoa(i)), found)
		}
	}
}

// Actor は、作成した主アクターを返す。
func (c *Creation) Actor() Actor { return c.actor }

// Document は、作る JSON ドキュメントを返す。
func (c *Creation) Document() Document { return c.document }

// Concrete は、作る JSON ドキュメントが従うスキーマを返す。
func (c *Creation) Concrete() Concrete { return c.concrete }

// Changes は、instance location ごとの change を返す。
func (c *Creation) Changes() []Change { return c.changes }
