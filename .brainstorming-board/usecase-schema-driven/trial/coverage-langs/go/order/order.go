// Package order は、受注の確定を持つ。
package order

import "errors"

// Order は出荷指示である。
type Order struct {
	Status string
	Lines  []string
}

// Confirm は確定する。明細が0件なら確定できない。
func (o *Order) Confirm() error {
	if len(o.Lines) == 0 {
		return errors.New("明細が0件")
	}
	o.Status = "確定済"
	return nil
}
