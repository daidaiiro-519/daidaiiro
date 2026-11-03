# 作る流れの手引き

Skill か Agent を作るとき、直すとき、配るときに、この手引きの順で進める。値の組み方そのものは、`skills create`・`agents create`・`skills update` が返す SAS の手順（各欄の説明・読み方・rule）に従う。

## 1. Skill か Agent かを見分ける

| 作りたい作業 | 作るもの |
|---|---|
| 1つの Skill で終わる | Skill（`scripts/skills`） |
| 複数の Skill を束ね、どの順に呼び、結果をどうまとめ、どこで人が承認するかが要る | Agent（`scripts/agents`）。足りない Skill は先に Skill として作る |

見分けた結果と理由を、最初の返事で作る人に示す。

## 2. 最初の返事で見せる4つ

| 見せるもの | 作り方 |
|---|---|
| 目的の仮説 | 作る人が言ったことから、誰が・どんな場面で・何を得るかを1文にする。【推測】を付け、作る人の承認で決める |
| 完成の像 | 使い手がこう頼むと、こう返る、を具体の文で見せる |
| 構造の像 | 受け持つ範囲・資料と出力の関係（Agent なら呼ぶ Skill と受け渡し）・手順と分かれ道・共通の所と変わる所。箱の名前は、使い手がすることの動詞で書く |
| 推奨付きの問い 0〜2個 | 下の「聞く所の見分け方」を両方満たす所だけ |

## 3. 聞く所の見分け方

次の両方を満たす所だけを聞く。満たさない所は AI が決め、【推測】を付けて像の中に出す。

- 値を変えると、完成の像が目に見えて変わる
- 会話・リポジトリ・既にある Skill のどれからも決まらない

問い1つは、決めること・推奨・ほかの選択肢・完成の像で変わる所の4つで出す。2往復目からは、変わった所だけを「前 → 後」で出し、残りの問いの数を付ける。

## 4. 書かせて確かめる

1. `scripts/skills create <フォルダ> --concrete <URN> --value <JSON>`（Agent なら `scripts/agents create`）で SAS に書かせる。
2. `scripts/skills write-skill-md <フォルダ>`（Agent なら `write-agent-md`）を打つ。
3. `scripts/skills validate <フォルダ>`（Agent なら `scripts/agents validate`）を打ち、検出があれば直す。
4. validate の承認の材料の delegate のとおり、別の会話で例を動かし、実際の答えを完成の像と並べて見せる。

## 5. 承認と配布

- 承認は人が書く。人が自分の端末で `sas update <フォルダ>/skill.json --patch '[{"op":"add","path":"/approved","value":"<承認した validate の操作 ID>"}]' --if-match <ハッシュ>` を打つ。AI のセッションからは SAS が断る。
- 配る：`scripts/skills distribute <フォルダ> --tool <claude-code|codex> --scope <repository|user>`。
- 取り下げる：`scripts/skills withdraw <フォルダ> --reason <理由>`。人が打ったときだけ外れる。AI から打つと、影響と人が打つコマンドが返る。

## 6. 直す

使われて外れた具体を受け取ったら、`scripts/skills update <フォルダ> --semantic --task <外れた具体>` の流れに従う。まず examples に足し、直す層を推奨付きで選んでもらい、承認のあとに `--patch … --if-match … --reason 層：<層>` で書かせる。
