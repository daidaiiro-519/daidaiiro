package domain

// actorKind は、主アクターの種類。
type actorKind string

// 主アクターの種類。
const (
	AI    actorKind = "ai"
	Human actorKind = "human"
)

// Actor は、値オブジェクト 主アクター。引数と環境から決め、推し量らない。
type Actor interface {
	Kind() actorKind
	actor()
}

type primaryActor struct{ kind actorKind }

func (a primaryActor) Kind() actorKind { return a.kind }
func (primaryActor) actor()            {}

// NewActor は、--human の有無と、coding agent の環境変数の有無から主アクターを返す。
// 集約 JSON ドキュメントの不変条件として、coding agent の中では human として実行できず、
// human_in_coding_agent の Failure を返す。
func NewActor(human, inCodingAgent bool) (Actor, error) {
	if !human {
		return primaryActor{AI}, nil
	}
	if inCodingAgent {
		return nil, Fail(HumanInCodingAgent, "", "coding agent の環境変数があるところで --human が渡された")
	}
	return primaryActor{Human}, nil
}
