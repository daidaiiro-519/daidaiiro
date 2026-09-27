package order

import (
	"os"
	"testing"
)

// traceScenario は、環境変数 SCENARIO_TRACE が設定されていれば、
// そのファイルへ検査するシナリオIDを1行追記する。
func traceScenario(t *testing.T, id string) {
	t.Helper()
	path := os.Getenv("SCENARIO_TRACE")
	if path == "" {
		return
	}
	f, err := os.OpenFile(path, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0o644)
	if err != nil {
		t.Fatalf("SCENARIO_TRACE を開けない: %v", err)
	}
	defer f.Close()
	if _, err := f.WriteString(id + "\n"); err != nil {
		t.Fatalf("SCENARIO_TRACE へ書けない: %v", err)
	}
}
