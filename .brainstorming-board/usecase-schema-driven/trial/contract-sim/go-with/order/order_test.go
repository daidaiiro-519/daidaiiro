package order

import "testing"

// 作成する: status == 準備中
func newOrder() *Order {
	return &Order{Status: "準備中"}
}

// 明細を足す: lines が1増える
func addLine(o *Order, line string) {
	o.Lines = append(o.Lines, line)
}

// SC-01J7Q4M 明細が0件のとき、確定できない
func TestConfirm_NoLines_Fails(t *testing.T) {
	traceScenario(t, "SC-01J7Q4M")

	o := newOrder()
	if err := o.Confirm(); err == nil {
		t.Fatal("明細が0件なのに確定できた")
	}
	if o.Status != "準備中" {
		t.Errorf("status = %q, want 準備中", o.Status)
	}
}

// SC-01J7Q4N 明細が1件あれば、確定できる
func TestConfirm_OneLine_Succeeds(t *testing.T) {
	traceScenario(t, "SC-01J7Q4N")

	o := newOrder()
	addLine(o, "明細1")
	if err := o.Confirm(); err != nil {
		t.Fatalf("確定できない: %v", err)
	}
	if o.Status != "確定済" {
		t.Errorf("status = %q, want 確定済", o.Status)
	}
}
