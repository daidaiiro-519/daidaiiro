package order

import "testing"

func TestConfirm(t *testing.T) {
	t.Run("明細が0件なら確定できない", func(t *testing.T) {
		scenario("SC-01J7Q4M")
		o := &Order{Status: "準備中"}
		if o.Confirm() == nil {
			t.Fatal("確定できてしまった")
		}
	})
	t.Run("明細が1件あれば確定できる", func(t *testing.T) {
		scenario("SC-01J7Q4N")
		o := &Order{Status: "準備中", Lines: []string{"りんご"}}
		if err := o.Confirm(); err != nil {
			t.Fatal(err)
		}
	})
}
