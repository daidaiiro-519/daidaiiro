package adapter
import (
  "plugin"
  "example.com/app/core"
  "example.com/app/stray"
  "example.com/app/gen"
)
func Use(p string) { _, _ = plugin.Open(p); _ = core.Name(); _ = stray.X; _ = gen.Made }
