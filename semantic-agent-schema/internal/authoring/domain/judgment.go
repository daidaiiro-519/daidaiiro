package domain

// verdict は、1つの rule と1つの instance location の組の結果の値。
type verdict string

// verdict の値。
const (
	Verified   verdict = "verified"
	Violated   verdict = "violated"
	Unverified verdict = "unverified"
	NotJudged  verdict = "not-judged"
)

// judge は、judgment を出した者。
type judge string

// judgment を出した者。
const (
	Machine judge = "machine"
	ByAI    judge = "ai"
)

// Judgment は、値オブジェクト judgment。1つの rule と1つの instance location の組の結果を表す。
type Judgment interface {
	Rule() Rule
	InstanceLocation() Pointer
	Verdict() verdict
	By() judge
	Reason() string
	judgment()
}

type result struct {
	rule     Rule
	location Pointer
	verdict  verdict
	by       judge
	reason   string
}

func (r result) Rule() Rule                { return r.rule }
func (r result) InstanceLocation() Pointer { return r.location }
func (r result) Verdict() verdict          { return r.verdict }
func (r result) By() judge                 { return r.by }
func (r result) Reason() string            { return r.reason }
func (result) judgment()                   {}

// RecordedJudgment は、操作の記録に残った文字列から judgment を返す。
// rule の id が空、instance location が JSON Pointer でない、verdict か by が閉じた一覧に無いときは error を返す。
func RecordedJudgment(ruleID, location, verdictValue, byValue, reason string) (Judgment, error) {
	r, err := NewRule(ruleID, nil)
	if err != nil {
		return nil, err
	}
	p, err := NewPointer(location)
	if err != nil {
		return nil, err
	}
	v := verdict(verdictValue)
	switch v {
	case Verified, Violated, Unverified, NotJudged:
	default:
		return nil, Fail(LogUnavailable, "", verdictValue+" は verdict の値でない")
	}
	b := judge(byValue)
	switch b {
	case Machine, ByAI:
	default:
		return nil, Fail(LogUnavailable, "", byValue+" は judgment を出した者の値でない")
	}
	return result{rule: r, location: p, verdict: v, by: b, reason: reason}, nil
}

// Counts は、verdict ごとの judgment の件数。満たしたとするのは verified だけで、unverified と not-judged は数えない。
type Counts struct {
	Verified, Violated, Unverified, NotJudged int
}

func countOf(judgments []Judgment) Counts {
	var c Counts
	for _, j := range judgments {
		switch j.Verdict() {
		case Verified:
			c.Verified++
		case Violated:
			c.Violated++
		case Unverified:
			c.Unverified++
		case NotJudged:
			c.NotJudged++
		}
	}
	return c
}
