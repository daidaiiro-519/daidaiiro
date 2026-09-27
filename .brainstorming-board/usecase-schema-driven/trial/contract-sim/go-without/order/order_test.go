package order

import "testing"

// 宣言 AGG-01J7Q4K（出荷指示）のシナリオを検査する。
// ops の対応:
//   作成する   → Order{Status: "準備中"}（post: status == 準備中。実装に生成関数が無いため、事後条件を満たす値を直接組む）
//   明細を足す → Lines へ1件追加（post: lines が1増える）
//   確定する   → Confirm()

func create() *Order { return &Order{Status: "準備中"} }

func addLine(o *Order) { o.Lines = append(o.Lines, "明細") }

// SC-01J7Q4M 明細が0件のとき、確定できない
// then: fails=true, checks=[check.precondition]
func TestSC01J7Q4M_明細が0件のとき確定できない(t *testing.T) {
	o := create()

	err := o.Confirm()

	if err == nil {
		t.Fatalf("確定が成功した。明細0件では失敗するはずである")
	}
	// check.precondition: 事前条件（lines.length > 0）違反で拒否され、状態が変わっていない
	if o.Status != "準備中" {
		t.Errorf("Status = %q, want %q（失敗時に状態が変わってはならない）", o.Status, "準備中")
	}
}

// SC-01J7Q4N 明細が1件あれば、確定できる
// then: fails=false, checks=[check.statusIs]
func TestSC01J7Q4N_明細が1件あれば確定できる(t *testing.T) {
	o := create()
	addLine(o)

	if err := o.Confirm(); err != nil {
		t.Fatalf("Confirm() = %v, want nil", err)
	}
	// check.statusIs: 事後条件 status == 確定済
	if o.Status != "確定済" {
		t.Errorf("Status = %q, want %q", o.Status, "確定済")
	}
}
