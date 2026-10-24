# 基盤の核

ドメインモデル ・ スキーマ ・ ポートの契約 ・ DB ・ 期待する振る舞いを同じ版でそろえ、参照の実装で場面を回して確かめる。

| 置き場所 | 中身 | 作り方 |
|---|---|---|
| model/ | ドメインモデル model.json | `python3 model/apply.py`（v2 に changes.py の変更を、変更前の原文と照合して適用） |
| schemas/ | Abstract の本体とほかのファイル | `python3 schemas/build_abstract.py && python3 schemas/build_members.py` |
| ports.json | 駆動される側のポートの入出力 | 手で書く |
| adapters/sqlite.sql | 手元の DB のアダプター | 手で書く（Abstract のファイル session の形から） |
| examples/ | ACDR の Concrete とインスタンス | `python3 examples/build_examples.py` |
| ref/ | 参照の実装（基盤 base.py と、アダプター adapters.py：Git ・ スナップショット ・ SQLite ・ AI ツールの模擬 ・ 判定の模擬 ・ 設定） | 手で書く |
| behaviors/ | 期待する振る舞いの場面 | `python3 behaviors/build.py` |
| coverage.json | 完成基準 K1 の対応表 | 手で書く |

確かめ方：

```
python3 check/schemas.py                 # 静的：本体とほかのファイルが本体に従う
python3 run.py --tmp <作業の場所>         # 場面を全アダプターで回し、K1 ・ K2 ・ K4 ・ K5 ・ K7 ・ K8 を判定
python3 run.py --tmp <作業の場所> --reverse   # 逆順でも同じ結果か（K6 の一部）
```

不合格が1件でもあれば0以外で終わる。結果は report.json と outputs.json に書く。
