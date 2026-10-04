# schema-driven の宣言（ボード schema-driven-base の次にすること1）

| ファイル | 中身 |
|---|---|
| `build.py` | 宣言を組む。材料はボード schema-driven-base の論点1〜4の決定 |
| `decls/` | 組んだ宣言（21件） |
| `run.py` | 見本の道具（スキーマ検証 ・ concrete7）を外から当てる |
| `view.py` | 見本の頁の型と部品（render6）で `view/viewer.html` へ描画する。見本のファイルは書き換えない |
