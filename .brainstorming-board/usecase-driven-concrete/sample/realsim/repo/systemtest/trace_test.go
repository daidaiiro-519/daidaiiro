package systemtest

// 記録の契約（concrete contract）の補助の関数。標準ライブラリだけで書き、テストの間で共有する
import (
	"encoding/json"
	"os"
	"testing"
)

func trace(t *testing.T, condition, hash, level string) {
	path := os.Getenv("CONCRETE_TRACE")
	if path == "" {
		return
	}
	f, err := os.OpenFile(path, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0o644)
	if err != nil {
		t.Fatal(err)
	}
	defer f.Close()
	b, _ := json.Marshal(map[string]string{"condition": condition, "hash": hash, "level": level, "test": t.Name()})
	f.Write(append(b, '\n'))
}
