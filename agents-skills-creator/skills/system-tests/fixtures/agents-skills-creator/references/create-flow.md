# 作る流れの手引き

Skill か Agent を作るとき、直すとき、配るときに、この手引きの順で進める。値の組み方そのものは、`scripts/skills create`・`scripts/agents create`・`scripts/skills update`・`scripts/agents update` の `--semantic` が返す手順（各欄の説明・読み方・rule と、打つコマンド）に従う。手順の番号は SKILL.md の手順の番号（手順1〜9）で指す。

## §1 作るか直すか、Skill か Agent かを見分ける（手順1）

| 作る人が言ったこと | すること |
|---|---|
| 作りたい作業 | 作る（§2・§3・§4） |
| 「直して」と、使われて外れた具体（頼んだこと・出たもの・期待したもの） | 直す（§6） |
| 使わなくしたい | 取り下げの影響を示す（§5） |

| 作りたい作業 | 作るもの |
|---|---|
| 1つの Skill で終わる | Skill（`scripts/skills`） |
| 複数の Skill を束ね、どの順に呼び、結果をどうまとめ、どこで人が承認するかが要る | Agent（`scripts/agents`）。束ねたい Skill が skills/ に無ければ、先にそれを Skill として作る |

見分けた結果と理由を、最初の返事で作る人に示す。

## §2 最初の返事の前に、書く手順を受け取る（手順2）

### フォルダの名前

名前は、最初の返事の構造の像の中で、作業の動詞と対象から AI が【推測】で付け、作る人の答えで決める。`--semantic` の前に決める。Skill は `skills/<name>`、Agent は `agents/<name>`。

### 書く手順を受け取る

最初の返事を作る前に、次を打つ。

- Skill：`scripts/skills create skills/<name> --concrete urn:agents-skills-creator:skill:v1 --semantic`
- Agent：`scripts/agents create agents/<name> --concrete urn:agents-skills-creator:agent:v1 --semantic`

返る `flow`（最初の返事で見せる4つ・構造の像・聞く所の見分け方・問いの形）と `sas`（書く手順。`command` は値を書かせる creator のコマンド）に従って、最初の返事を作る。

### 最初の返事で見せる4つ

| 見せるもの | 作り方 |
|---|---|
| 目的の仮説 | 作る人が言ったことから、誰が・どんな場面で・何を得るかを1文にする。【推測】を付け、作る人の承認で決める |
| 完成の像 | 使い手がこう頼むと、こう返る、を具体の文で見せる |
| 構造の像 | 受け持つ範囲・資料と出力の関係（Agent なら呼ぶ Skill と受け渡し）・手順と分かれ道・共通の所と変わる所。箱の名前は、使い手がすることの動詞で書く。フォルダの名前もここに【推測】で出す |
| 推奨付きの問い 0〜2個 | 下の「聞く所の見分け方」を両方満たす所だけ |

### 聞く所の見分け方

次の両方を満たす所だけを聞く。満たさない所は AI が決め、【推測】を付けて像の中に出す。

- 値を変えると、完成の像が目に見えて変わる
- 会話・リポジトリ・既にある Skill のどれからも決まらない

問い1つは、決めること・推奨・ほかの選択肢・完成の像で変わる所の4つで出す。2往復目からは、変わった所だけを「前 → 後」で出し、残りの問いの数を付ける。

### Agent のとき

- 配下の Skill は、`scripts/skills validate skills/<Skill の名前>` の結果のうち、`.skills[0].approval.name`・`.skills[0].approval.description`・`.skills[0].approval.structure.inputs`・`.skills[0].approval.structure.outputs` だけを見て選ぶ（skill.json を直接読まない）。
- `agents/<name>/skills/` に、配下の Skill のフォルダへのシンボリックリンクを、Skill と同じ名前で、相対の経路で張る（例：`agents/<name>/skills/<Skill の名前>` → `../../../skills/<Skill の名前>`）。

## §3 書かせて確かめる（手順3〜7。承認の前）

- 手順3：作る人が目的の仮説を承認し、問いに答えたら、`--semantic` で受け取った手順に従って値を組み、`sas.command` のとおり `scripts/skills create <フォルダ> --concrete <URN> --value <JSON か @<ファイル>>`（Agent なら `scripts/agents create`）で SAS に書かせる。
- 手順4：`scripts/skills write-skill-md <フォルダ>`（Agent なら `scripts/agents write-agent-md`）を打つ。
- 手順5：`scripts/skills validate <フォルダ>`（Agent なら `scripts/agents validate`）を打ち、検出があれば直す。
- 手順6：意味の rule の判定。承認の材料の delegate の a のとおり、別の会話で判定し、`sas validate <経路> --answer <ファイル>` で渡す。そのあと validate を打ち直し、`approvable` が true になるまで承認を求めない。
- 手順7：例を動かす。承認の材料の delegate の b のとおり、別の会話で例を動かし、実際の答えを完成の像と並べて見せる。

### 値をファイルで渡す

`--value` と `--patch` には、`@<ファイル>` でファイルの中身を渡せる（例：`--value @work/value.json`）。控えのファイルは作業場所（打った場所）の中に置き、外（/tmp など）に置かない。

## §4 承認と配布（手順8・9）

- 手順8：承認は人が書く。承認する直前に validate を打ち直し、そのとき承認の材料に出た `approve.command`（判定まで済んだ確かめの操作 ID とハッシュ入り。`approvable` が true のときだけ出る）を作る人に示し、人が自分の端末で打つ。AI のセッションから打つと SAS が断る。
- 手順9：配る。`scripts/skills distribute <フォルダ> --tool <claude-code|codex> --scope <repository|user>`（Agent なら `scripts/agents distribute`）。

## §5 取り下げ（分かれ道 1b）

`scripts/skills withdraw <フォルダ> --reason <理由>`（Agent なら `scripts/agents withdraw`）を打ち、返る影響（配布先の置き場所）と、人が打つコマンドを作る人に示す。外すのは、影響を見た人が自分の端末で打ったときだけ。

## §6 直す（手順2〜9）

使われて外れた具体を受け取ったら、最初の返事の前に `scripts/skills update <フォルダ> --semantic --task <外れた具体>`（Agent なら `scripts/agents update`）を打ち、返る `flow` に従う。

### 直すときの最初の返事（手順2）

| 見せるもの | 作り方 |
|---|---|
| 外れた具体 | 頼んだこと・出たもの・期待したもの |
| 足す例 | 外れた具体から作る examples の1件（from は「使われて外れた具体」）。返事で見せるだけで、まだ書かない |
| 層ごとの前と後 | examples → edge_cases → rules → role → purpose の層ごとに、直す前と後の像 |
| 推奨 | 外れ方から、どの層で直すかを推奨付きで1つ |

作る人が層を選んで承認したら（手順2の完了）、足す例と選んだ層の直しを1つの JSON Patch にし、書くのは承認のあと1回：`sas.command` のとおり `scripts/skills update <フォルダ> --patch <JSON Patch か @<ファイル>> --if-match <ハッシュ> --reason 層：<層>`（手順3）。そのあとは手順4〜9 の順に進める。
