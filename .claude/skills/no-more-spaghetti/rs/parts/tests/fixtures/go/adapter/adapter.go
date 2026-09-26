package adapter

import "core"

// Label は内側の層を呼ぶ。
func Label() string { return core.Name() }
