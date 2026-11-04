package domain

// Validation は、集約「検証」。1つの JSON ドキュメントの、1つの内容について、機械的な検証の judgment と、
// 前回の検証から引き継いだ AI の judgment を持つ。ID は操作の記録に追記したときに付き、作った後は変えない。
type Validation struct {
	document  Document
	concrete  Concrete
	judgments []Judgment
	judged    bool
}

// ValidationInput は、検証を作るために application がポートから読んだもの。
type ValidationInput struct {
	Document   Document
	Value      any // Document.Content を ParseJSON で読んだ値
	Concrete   Concrete
	Violations []Violation // JSON Schema の違反
	Convention []Violation // 作法の検査の違反
	Judged     []Judgment  // AI が判定した judgments。意味的な検証でなければ nil
	Previous   *Previous   // 前回の検証。無ければ nil。引き継ぐ AI の judgment の出どころ
}

// NewValidation は、機械的な検証の違反を violated の judgment にし、AI の response の judgment と、前回の検証から引き継いだ
// AI の judgment と合わせた検証を返す。response で判定した組は変更点に当たるので、引き継ぐ judgment と重ならない。
func NewValidation(in ValidationInput) *Validation {
	var judgments []Judgment
	for _, v := range in.Violations {
		judgments = append(judgments, machine(promptRule{id: v.AbsoluteKeywordLocation}, v))
	}
	for _, v := range in.Convention {
		judgments = append(judgments, machine(promptRule{id: ConventionRule}, v))
	}
	judgments = append(judgments, in.Judged...)
	judgments = append(judgments, Inherited(in.Previous, in.Value, in.Concrete)...)
	return &Validation{document: in.Document, concrete: in.Concrete, judgments: judgments, judged: in.Judged != nil}
}

func machine(r Rule, v Violation) Judgment {
	return result{rule: r, location: v.InstanceLocation, verdict: Violated, by: Machine, reason: v.Message}
}

// Document は、検証した JSON ドキュメントを返す。
func (v *Validation) Document() Document { return v.document }

// Concrete は、検証に使ったスキーマを返す。
func (v *Validation) Concrete() Concrete { return v.concrete }

// Judgments は、機械的な検証の judgment と、引き継いだ AI の judgment を返す。
func (v *Validation) Judgments() []Judgment { return v.judgments }

// Judged は、AI の判定を受け取った意味的な検証かを返す。意味的な検証は、次の変更点の基準になる。
func (v *Validation) Judged() bool { return v.judged }

// Counts は、verdict ごとの件数を返す。満たしたとするのは verified だけ。
func (v *Validation) Counts() Counts { return countOf(v.judgments) }
