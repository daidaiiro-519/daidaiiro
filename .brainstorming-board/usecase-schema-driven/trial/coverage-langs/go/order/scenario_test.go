package order

import "os"

// 走ったシナリオを1行書く ── 利用者の側に置く補助（標準ライブラリだけ）
func scenario(id string) {
	if p := os.Getenv("SCENARIO_TRACE"); p != "" {
		f, _ := os.OpenFile(p, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0o644)
		defer f.Close()
		f.WriteString(id + "\n")
	}
}
