# 作る流れの手引き

Skill か Agent を作るとき、直すとき、配るときに、この手引きの順で進める。値の組み方そのものは、`scripts/skills create`・`scripts/agents create`・`scripts/skills update`・`scripts/agents update` が返す SAS の手順（各欄の説明・読み方・rule）に従う。

## 1. 作るか直すか、Skill か Agent かを見分ける

| 作る人が言ったこと | すること |
|---|---|
| 作りたい作業 | 作る（2〜） |
| 「直して」と、使われて外れた具体（頼んだこと・出たもの・期待したもの） | 直す（6） |
| 使わなくしたい | 取り下げの影響を示す（5） |

| 作りたい作業 | 作るもの |
|---|---|
| 1つの Skill で終わる | Skill（`scripts/skills`） |
| 複数の Skill を束ね、どの順に呼び、結果をどうまとめ、どこで人が承認するかが要る | Agent（`scripts/agents`）。足りない Skill は先に Skill として作る |

見分けた結果と理由を、最初の返事で作る人に示す。

## 2. 最初の返事の前に、書く手順を受け取る

最初の返事を作る前に、次を打つ。

- Skill：`scripts/skills create <Skill のフォルダ> --concrete urn:agents-skills-creator:skill:v1 --semantic`
- Agent：`scripts/agents create <Agent のフォルダ> --concrete urn:agents-skills-creator:agent:v1 --semantic`

名前が決まっていなければ `skills/<name>` のように渡してよい。返る `flow`（最初の返事で見せる4つ・構造の像・聞く所の見分け方・問いの形）と `sas`（SAS の書く手順）に従って、最初の返事を作る。

### 最初の返事で見せる4つ

| 見せるもの | 作り方 |
|---|---|
| 目的の仮説 | 作る人が言ったことから、誰が・どんな場面で・何を得るかを1文にする。【推測】を付け、作る人の承認で決める |
| 完成の像 | 使い手がこう頼むと、こう返る、を具体の文で見せる |
| 構造の像 | 受け持つ範囲・資料と出力の関係（Agent なら呼ぶ Skill と受け渡し）・手順と分かれ道・共通の所と変わる所。箱の名前は、使い手がすることの動詞で書く |
| 推奨付きの問い 0〜2個 | 下の「聞く所の見分け方」を両方満たす所だけ |

### 聞く所の見分け方

次の両方を満たす所だけを聞く。満たさない所は AI が決め、【推測】を付けて像の中に出す。

- 値を変えると、完成の像が目に見えて変わる
- 会話・リポジトリ・既にある Skill のどれからも決まらない

問い1つは、決めること・推奨・ほかの選択肢・完成の像で変わる所の4つで出す。2往復目からは、変わった所だけを「前 → 後」で出し、残りの問いの数を付ける。

### Agent のとき

- 配下の Skill は、`scripts/skills validate <Skill のフォルダ>` の結果の name・description・inputs・outputs だけで選ぶ（skill.json を直接読まない）。
- `agents/<name>/skills/` に、配下の Skill のフォルダへのリンクを、Skill と同じ名前で置く。

## 3. 書かせて確かめる（承認の前）

1. 作る人が目的の仮説を承認し、問いに答えたら、`--semantic` で受け取った手順に従って値を組み、`scripts/skills create <フォルダ> --concrete <URN> --value <JSON>`（Agent なら `scripts/agents create`）で SAS に書かせる。
2. `scripts/skills write-skill-md <フォルダ>`（Agent なら `scripts/agents write-agent-md`）を打つ。
3. `scripts/skills validate <フォルダ>`（Agent なら `scripts/agents validate`）を打ち、検出があれば直す。
4. 意味の rule の判定：承認の材料の delegate の a のとおり、別の会話で判定し、`sas validate <経路> --answer` で渡す。`approvable` が true になるまで承認を求めない。
5. 例を動かす：承認の材料の delegate の b のとおり、別の会話で例を動かし、実際の答えを完成の像と並べて見せる。

## 4. 承認と配布

- 承認は人が書く。validate の承認の材料の `approve.command`（操作 ID とハッシュ入り）を作る人に示し、人が自分の端末で打つ。AI のセッションから打つと SAS が断る。
- 配る：`scripts/skills distribute <フォルダ> --tool <claude-code|codex> --scope <repository|user>`（Agent なら `scripts/agents distribute`）。

## 5. 取り下げ

`scripts/skills withdraw <フォルダ> --reason <理由>`（Agent なら `scripts/agents withdraw`）を打ち、返る影響（配布先の置き場所）と、人が打つコマンドを作る人に示す。外すのは、影響を見た人が自分の端末で打ったときだけ。

## 6. 直す

使われて外れた具体を受け取ったら、最初の返事の前に `scripts/skills update <フォルダ> --semantic --task <外れた具体>`（Agent なら `scripts/agents update`）を打ち、返る `flow` に従う。

### 直すときの最初の返事

| 見せるもの | 作り方 |
|---|---|
| 外れた具体 | 頼んだこと・出たもの・期待したもの |
| 足す例 | 外れた具体から作る examples の1件（from は「使われて外れた具体」）。返事で見せるだけで、まだ書かない |
| 層ごとの前と後 | examples → edge_cases → rules → role → purpose の層ごとに、直す前と後の像 |
| 推奨 | 外れ方から、どの層で直すかを推奨付きで1つ |

作る人が層を選んで承認したら、足す例と選んだ層の直しを1つの JSON Patch にし、書くのは承認のあと1回：`scripts/skills update <フォルダ> --patch <JSON Patch> --if-match <手順の target.hash> --reason 層：<層>`。そのあとは 3 の 2〜5 と 4 の順に進める。
