package systemtest

// UC-1 注文を確定する の system テスト。駆動する側（CLI）から動かし、決済代行は偽物にする
import (
	"encoding/json"
	"os/exec"
	"strings"
	"testing"
	"time"
)

type out struct {
	OK         bool           `json:"ok"`
	Status     string         `json:"status"`
	Pickup     *string        `json:"pickup"`
	Approval   *string        `json:"approval_no"`
	Reserved   *string        `json:"reserved_slot"`
	SlotFree   map[string]int `json:"slot_free"`
	Progress   int            `json:"progress"`
	Sent       []string       `json:"sent"`
}

func run(t *testing.T, in string) out {
	cmd := exec.Command("python3", "-m", "src.order.app")
	cmd.Dir = ".."
	cmd.Stdin = strings.NewReader(in)
	b, err := cmd.Output()
	if err != nil {
		t.Fatalf("app: %v", err)
	}
	var o out
	if err := json.Unmarshal(b, &o); err != nil {
		t.Fatal(err)
	}
	return o
}

const base = `"lines":[{"product":"P-1","qty":2,"price":500},{"product":"P-2","qty":1,"price":300}]`

func successGuarantees(t *testing.T, o out, slot string) {
	if !o.OK || o.Status != "確定済" || o.Pickup == nil || o.Approval == nil || o.Reserved == nil || *o.Reserved != slot {
		t.Fatalf("成功時保証が成り立たない: %+v", o)
	}
}
func minimalGuarantees(t *testing.T, o out, slot string, free int) {
	if o.Status == "確定済" && o.Approval == nil {
		t.Fatalf("MG-1: 承認なしで確定した")
	}
	if o.Status == "下書き" && (o.Reserved != nil || o.SlotFree[slot] != free) {
		t.Fatalf("MG-2: 調理枠が残った: %+v", o)
	}
	if o.Progress < 1 {
		t.Fatalf("MG-3: 経過が残っていない")
	}
}

func TestMainSuccess(t *testing.T) {
	trace(t, "UC-1.M", "22898128", "system")
	o := run(t, `{`+base+`,"catalog":{"P-1":500,"P-2":300},"slots":[{"id":"12:00","start":"2026-10-03T12:00","free":10}],"payment":"approve"}`)
	successGuarantees(t, o, "12:00")
}

func TestPriceChanged(t *testing.T) {
	trace(t, "UC-1.EXT-1", "a831c121", "system")
	o := run(t, `{`+base+`,"catalog":{"P-1":550,"P-2":300},"slots":[{"id":"12:00","start":"2026-10-03T12:00","free":10}],"payment":"approve"}`)
	successGuarantees(t, o, "12:00")
}

func TestSlotShort(t *testing.T) {
	trace(t, "UC-1.EXT-2", "b125148e", "system")
	o := run(t, `{`+base+`,"catalog":{"P-1":500,"P-2":300},"slots":[{"id":"12:00","start":"2026-10-03T12:00","free":0},{"id":"12:30","start":"2026-10-03T12:30","free":10}],"payment":"approve"}`)
	successGuarantees(t, o, "12:30")
}

func TestPaymentDeclined(t *testing.T) {
	trace(t, "UC-1.EXT-3", "8a16eb95", "system")
	o := run(t, `{`+base+`,"catalog":{"P-1":500,"P-2":300},"slots":[{"id":"12:00","start":"2026-10-03T12:00","free":10}],"payment":"decline"}`)
	if o.OK || o.Status != "下書き" {
		t.Fatalf("成功時保証は成り立ってはいけない: %+v", o)
	}
	minimalGuarantees(t, o, "12:00", 10)
}

func TestConfirmResponseTime(t *testing.T) {
	trace(t, "UC-1.STEP-5.QR-1", "903cbb3d", "system")
	ok := 0
	for i := 0; i < 20; i++ {
		s := time.Now()
		run(t, `{`+base+`,"catalog":{"P-1":500,"P-2":300},"slots":[{"id":"12:00","start":"2026-10-03T12:00","free":10}],"payment":"approve"}`)
		if time.Since(s) <= 2*time.Second {
			ok++
		}
	}
	if ok*100/20 < 90 {
		t.Fatalf("2秒以下の割合が %d%%", ok*100/20)
	}
}


func TestOverLimit(t *testing.T) {
	trace(t, "UC-1.EXT-4", "9acfca23", "system")
	o := run(t, `{"lines":[{"product":"P-1","qty":11,"price":500}],"catalog":{"P-1":500},"slots":[{"id":"12:00","start":"2026-10-03T12:00","free":20}],"payment":"approve"}`)
	if o.OK || o.Status != "下書き" {
		t.Fatalf("成功時保証は成り立ってはいけない: %+v", o)
	}
	minimalGuarantees(t, o, "12:00", 20)
}
