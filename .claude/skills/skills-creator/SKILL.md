---
name: skills-creator
description: Agent Skills の公開の仕様に従う Skill を作るときと、作った Skill がどの AI ツールでも呼ばれて手順どおりに動くかを確かめるときに使う。新しい Skill の雛形と基盤のエンジンを置く scaffold、エンジンを入れ替える engine、項目ごとに結果を返す check を持つ。Skill を新しく作る ・ 直す ・ 確かめるよう依頼されたときに使う。
license: MIT
---

# Agent Skills の形の Skill を作り、検査する Skill：skills-creator

## 目的

どの AI ツールに置いても、正しいときに呼ばれ、手順どおりに動く Skill を作る。そのために、新しい Skill の雛形と、Skill が必ず持つものを決めた型（`skills-creator/skill@v1`）に合うかの検査を持つ。

型が決めるのは、必ず書く項目と並びだけである。各項目の中身は、Skill を作る人が書く。作った Skill をどう配るかは、作った人が決める。

## 役割

- 新しい Skill のフォルダに、SKILL.md の雛形 ・ 入口 ・ 基盤のエンジン ・ concrete スキーマの雛形を置く（scaffold）
- Skill のフォルダの基盤のエンジンを、版を指して入れ替える（engine）
- Skill のフォルダを、項目ごとに結果で検査する（check）
- 基盤のエンジンは、公開された版を取り、SHA-256 と証明を照合してから置く。照合できなければ止める
- SKILL.md の型の判定は、読み取った結果を基盤のエンジンに型のスキーマで検査させて得る。この Skill は判定を書き直さない

## 入力の想定

| 受け取る情報 | 解釈 ・ 既定値 |
|---|---|
| Skill のフォルダ | scaffold は、まだ無いフォルダを受け取る。フォルダの名前が Skill の name になる |
| 基盤のエンジンの版（`--engine` ・ `--version`） | scaffold で省けば、skills-creator 自身のエンジンと同じ版を入れる |
| target（`--target`） | 明示されなければ、動かしている環境の OS に合うものを入れる |

## 実行手順

### Step 1: 雛形を置く

```
scripts/bin/skills-creator scaffold <Skill のフォルダ>
```

置くのは次の6つで、作業用のフォルダでそろえてから移す。途中で失敗したら何も残らない。

| 置くもの | 中身 |
|---|---|
| `SKILL.md` | 先頭の欄と6つの節の雛形 |
| `scripts/<Skill名>` | 入口（Python）。基盤の操作をエンジンへそのまま渡し、Skill の能力は中で行う |
| `scripts/<Skill名>.cmd` | Windows で入口を起動する数行 |
| `scripts/bin/schema-driven` ・ `schema-driven-mcp` | 照合した基盤のエンジン |
| `assets/<Skill名>.schema.json` | concrete スキーマの雛形（基盤の create で作る） |

基盤のエンジンを取って照合するには、gh（attestation が使える版）が必要である。

### Step 2: 記入欄を書き換え、入口に能力を足す

雛形は、記入する所を二重の波括弧で示す。全部を書き換える。

- 先頭の欄には、Agent Skills の仕様の欄（name ・ description ・ license ・ compatibility ・ metadata ・ allowed-tools）だけを書く
- 本文には、目的 ・ 役割 ・ 入力の想定 ・ 実行手順 ・ 出力形式 ・ ガードレールの6つの節を、この順に書く。ほかの節は足してよい
- 手順は、入口 `scripts/<Skill名>` だけを呼ぶ。言語に依存するコマンドを SKILL.md に書かない
- ファイルは、Skill のフォルダからの相対パスで指す
- スキーマの各欄の書き方は、`scripts/<Skill名> prompt` が x-prompt で返す
- Skill の能力は、入口の中に足す。入口を別の言語で書き直すときは、`scripts/<Skill名>` と `scripts/<Skill名>.cmd` を差し替える

### Step 3: 検査する

```
scripts/bin/skills-creator check <Skill のフォルダ>
```

項目ごとに、id と結果が返る。

| 項目 | 何を見るか |
|---|---|
| `skill.frontmatter` | SKILL.md の先頭の欄が在り、YAML として読める |
| `skill.type` | SKILL.md が型（[型のスキーマ](assets/skill.schema.json)）に合う |
| `skill.placeholders` | 記入欄が残っていない |
| `schemas.present` | concrete スキーマが `assets/` に1件以上ある |
| `schemas.check` | 基盤の check-schemas が所見0件で通る |
| `engine.runs` | `scripts/bin/schema-driven` が起動して版を答える |
| `engine.abstract` | スキーマの x-abstract の版を、エンジンが扱える |
| `entry.delegate` | 入口が基盤の操作をエンジンに渡して、同じ結果を返す |
| `legacy.absent` | 古い形のファイル（tool.json ・ tools.json）が無い |

### Step 4: 不合格と判定不能を0件にする

不合格の項目は、中身に書かれた理由を直す。判定不能は、確かめる前提が欠けていることを示すので、中身に書かれた前提（エンジンが在るかなど）を直す。

### Step 5: 基盤のエンジンを入れ替える

```
scripts/bin/skills-creator engine <Skill のフォルダ> --version <基盤の版>
```

別の OS で使うときは `--target` で、その OS の配布物を入れる。

## 出力形式

どの操作も JSON を返す。終了コードは、正常なら 0、不合格 ・ 判定不能 ・ 照合で止まったなら 1、使い方の誤りなら 2 である。

| check の結果 | 意味 |
|---|---|
| 合格（`pass`） | 確かめて、合っていた |
| 不合格（`fail`） | 確かめて、合っていなかった。中身に理由がある |
| 判定不能（`unknown`） | 確かめられなかった。中身に、欠けている前提がある |
| 対象外（`skip`） | その Skill では見ない（入口を持たない Skill の `entry.delegate` など）。中身に理由がある |

どの項目も、確かめるまでは判定不能である。合格は、不合格と判定不能が0件のときだけである。引数を付けずに呼ぶと、操作の一覧を返す。

## ガードレール

- **型の判定を、この Skill の中で書き直さない**。判定は型のスキーマが持ち、基盤のエンジンが行う。2か所に置くと、片方だけ直されたときに結果が食い違う
- **確かめていない項目を合格にしない**。どの項目も既定は判定不能である。既定を合格にすると、誤りがあっても合格が出る
- **基盤のエンジンを、照合しないまま置かない**。gh が無いか古いときは止める。照合を飛ばす経路は持たない
- **既に在るフォルダに雛形を置かない**。書いた中身が消える
- **本文に、特定の AI ツールの置き場所 ・ ツール専用の変数 ・ 絶対パスを書かない**。ほかの AI ツールでは解決しない
- **公開した型の版を書き換えない**。中身を変えるときは、版を上げた新しい名前（`@v2`）で足す
