# 作法を提示したとき、AI がテストへシナリオID の出力を組み込むか

道具の本体が保持する作法（`get` の応答の `test_contract`）を、この会話の文脈を保持しない AI に提示し、テストを書かせた記録である（2026-09-27）。

| フォルダ | 渡した応答 | 結果（`schema covered … AGG-01J7Q4K`） |
|---|---|---|
| `go-with/` | `get-with.json`（作法あり） | 網羅 2/2 |
| `py-with/` | `get-with.json`（作法あり） | 網羅 2/2 |
| `ts-with/` | `get-with.json`（作法あり） | 網羅 2/2 |
| `go-without/` | `get-without.json`（作法なし） | 網羅 0/2（記録のファイルが作られない） |

実装（`order.go` ・ `order.py` ・ `order.mjs`）以外のファイルは、すべて AI が書いた。指示の文面に `SCENARIO_TRACE` の語は入れていない。
