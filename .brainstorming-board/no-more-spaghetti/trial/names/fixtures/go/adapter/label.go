package adapter
import (
  "example.com/app/core"
  "example.com/app/stray"
)
func Label() string { _ = stray.X(); return core.Name() }
