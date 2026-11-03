# 作る流れの手引き

Skill か Agent を作るとき、直すとき、配るときに、この手引きの順で進める。使うコマンドは、semantic-agent-schema と同じく PATH に置く `agents-skills-creator` で、サブコマンド `skills`（Skill）・ `agents`（Agent）・ `approve`（人が打つ承認）を持つ。`--concrete` は省ける（Skill は urn:agents-skills-creator:skill:v1、Agent は urn:agents-skills-creator:agent:v1）。

「承認」は、`agents-skills-creator approve` で approved を書くことだけを指す。目的の仮説は、作る人が「進めてよい」と答えて決まる。

Semantic Agent Schema の記録は <git-common-dir>/sas/sas.db に書かれ、作業場所の Git の中に残る（git status には出ない）。最低保証の「作るフォルダと配布先の外は変わらない」は、この記録を除く。

控えのファイル（値・JSON Patch・判定の答えなど）は、どのコマンドの出力も作業場所の中に置き、終わったら消す（作業場所は打った場所。外の /tmp などに置かない）。値の組み方そのものは、`agents-skills-creator skills create`・`agents-skills-creator agents create`・`agents-skills-creator skills update`・`agents-skills-creator agents update` の `--semantic` が返す手順（各欄の説明・読み方・rule と、打つコマンド）に従う。手順の番号は SKILL.md の手順の番号（手順1〜9）で指す。

## §1 作るか直すか、Skill か Agent かを見分ける（手順1）

| 作る人が言ったこと | すること |
|---|---|
| 作りたい作業 | 作る（§2・§3・§4） |
| 「直して」と、使われて外れた具体（頼んだこと・出たもの・期待したもの） | 直す（§6） |
| 使わなくしたい | 取り下げの影響を示す（§5） |

| 作りたい作業 | 作るもの |
|---|---|
| 1つの Skill で終わる | Skill（`agents-skills-creator skills`） |
| 複数の Skill を束ね、どの順に呼び、結果をどうまとめ、どこで人が承認するかが要る | Agent（`agents-skills-creator agents`）。束ねたい Skill が skills/ に無ければ、先にそれを Skill として作る |

見分けた結果と理由を、最初の返事で作る人に示す。

## §2 最初の返事の前に、書く手順を受け取る（手順2）

### フォルダの名前

名前は、最初の返事の前に、作業の動詞と対象から AI が【推測】で付けて `--semantic` を打ち、最初の返事の構造の像で見せる。作る人が名前を変えたら --semantic を打ち直す。Skill は `skills/<name>`、Agent は `agents/<name>`。

### 書く手順を受け取る

最初の返事を作る前に、次を打つ。

- Skill：`agents-skills-creator skills create skills/<name> --semantic`
- Agent：`agents-skills-creator agents create agents/<name> --semantic`

最初の返事の構造の像は、--semantic の flow の形に従って AI が組む。validate の承認の材料の structure_image が使えるのは書かせたあと（2往復目から）。

返る出力のうち、最初の返事のために読むのは `flow`（最初の返事で見せる4つ・構造の像・聞く所の見分け方・問いの形）、書くために読むのは semantic-agent-schema の各 step の current ・ rules と command（値を書かせる creator のコマンド）。

### 最初の返事で見せる4つ

| 見せるもの | 作り方 |
|---|---|
| 目的の仮説 | 作る人が言ったことから、誰が・どんな場面で・何を得るかを1文にする。【推測】を付け、作る人が進めてよいと答えて決まる |
| 完成の像 | 使い手がこう頼むと、こう返る、を具体の文で見せる |
| 構造の像 | 受け持つ範囲・資料と出力の関係（Agent なら呼ぶ Skill と受け渡し）・手順と分かれ道・共通の所と変わる所。箱の名前は、使い手がすることの動詞で書く。フォルダの名前もここに【推測】で出す |
| 推奨付きの問い 0〜2個 | 下の「聞く所の見分け方」を両方満たす所だけ |

### 聞く所の見分け方

次の両方を満たす所だけを聞く。満たさない所は AI が決め、【推測】を付けて像の中に出す。

- 値を変えると、完成の像が目に見えて変わる
- 会話・リポジトリ・既にある Skill のどれからも決まらない

問い1つは、決めること・推奨・ほかの選択肢・完成の像で変わる所の4つで出す。2往復目からは、変わった所だけを「前 → 後」で出し、残りの問いの数を付ける。

### Agent のとき

- 配下の Skill は、`agents-skills-creator skills validate skills/<Skill の名前>` の結果のうち、`.skills[0].approval.name`・`.skills[0].approval.description`・`.skills[0].approval.structure.inputs`・`.skills[0].approval.structure.outputs` だけを見て選ぶ（skill.json を直接読まない）。
- `agents/<name>/skills/` に、配下の Skill のフォルダへのシンボリックリンクを、Skill と同じ名前で、相対の経路で張る（例：`agents/<name>/skills/<Skill の名前>` → `../../../skills/<Skill の名前>`）。リンクは、--value で書かせたあと、write-agent-md の前に張る。

### 道具と環境の書き分け

PATH に置くコマンドは uses（外の道具。used_by で使うステップか分かれ道を指す）、AI ツールの機能や環境（ウェブの取得など）は compatibility に書く。

## §3 書かせて確かめる（手順3〜7。承認の前）

- 手順3：作る人が目的の仮説で進めてよいと答え、問いに答えたら、`--semantic` で受け取った手順に従って値を組み、`semantic-agent-schema.command` のとおり `agents-skills-creator skills create <フォルダ> --value <JSON か @<ファイル>>`（Agent なら `agents-skills-creator agents create`）で Semantic Agent Schema に書かせる。
- 手順4：`agents-skills-creator skills write-skill-md <フォルダ>`（Agent なら `agents-skills-creator agents write-agent-md`）を打つ。
- 手順5：`agents-skills-creator skills validate <フォルダ>`（Agent なら `agents-skills-creator agents validate`）を打ち、検出があれば直す。
- 手順6：意味の rule の判定。semantic-agent-schema validate <ファイル> --semantic で判定の手順を受け取り、別の会話で判定する（承認の材料の delegate の a）。examples の rule example-runs-as-expected だけは、手順7で例を動かした会話が判定する。
- 手順7：例を動かす。承認の材料の delegate の b のとおり、別の会話に SKILL.md（Agent なら AGENT.md）だけを読ませて例を動かし、その会話が example-runs-as-expected を判定する（reason に例ごとの実際の答えを書く）。実際の答えは完成の像と並べて見せる。
- 手順6と手順7の判定は、1つの答えのファイルにまとめて渡す（手順の answer.schema に従う JSON。rule が1つでも欠けた答えは Semantic Agent Schema が断るので、a と b で分けて渡せない）：`semantic-agent-schema validate <ファイル> --answer <答えのファイル>`。そのあと validate を打ち直すと、渡した判定が `approval.judgments` に、どの操作の判定かと一緒に出る。判定がそろえば delegate は出ない。violated があれば `approvable` は false のままなので、§6 の層の選び方で直す。`approvable` が true になるまで承認を求めない。

### 承認の前の直し

承認の前に自分で気づいた直しも、`agents-skills-creator skills update <フォルダ> --patch <JSON Patch か @<ファイル>> --if-match <ハッシュ> --reason 層：<層>`（Agent なら `agents-skills-creator agents update`）で書かせる。ファイルを直接書かない。

### 値をファイルで渡す

`--value` と `--patch` には、`@<ファイル>` でファイルの中身を渡せる（例：`--value @work/value.json`）。

## §4 承認と配布（手順8・9）

- 手順8：承認は人が書く。validate の承認の材料に `approvable` が true で出る `approve.command`（`agents-skills-creator approve <フォルダ>`）を作る人に示し、人が自分の端末で打つ。approve は、判定まで済んだ確かめの操作 ID とハッシュを自分で引いて /approved を書く。AI のセッションから打つと Semantic Agent Schema が断り、人が打つコマンドが返る。承認したあとの validate は、承認した確かめの操作 ID を `approval.approved_by` に出す。
- 手順9：配る。`agents-skills-creator skills distribute <フォルダ> --tool <claude-code|codex> --scope <repository|user>`（Agent なら `agents-skills-creator agents distribute`）。

## §5 取り下げ（分かれ道 1b）

`agents-skills-creator skills withdraw <フォルダ> --reason <理由>`（Agent なら `agents-skills-creator agents withdraw`）を打ち、返る影響（配布先の置き場所）と、人が打つコマンドを作る人に示す。外すのは、影響を見た人が自分の端末で打ったときだけ。

## §6 直す（手順2〜9）

使われて外れた具体を受け取ったら、最初の返事の前に `agents-skills-creator skills update <フォルダ> --semantic --task <外れた具体>`（Agent なら `agents-skills-creator agents update`）を打ち、返る `flow` に従う。

### 直すときの最初の返事（手順2）

| 見せるもの | 作り方 |
|---|---|
| 外れた具体 | 頼んだこと・出たもの・期待したもの |
| 足す例 | 外れた具体から作る examples の1件（from は「使われて外れた具体」）。返事で見せるだけで、まだ書かない |
| 層ごとの前と後 | examples → edge_cases → rules → role → purpose の層ごとに、直す前と後の像 |
| 推奨 | 外れ方から、どの層で直すかを推奨付きで1つ |

### --reason と covers

- `--reason` は「層：<層>」で始め、続けて外れた具体を1文で書いてよい（例：「層：examples。全角の空白を含む項目名に「無い」と返した」）。
- 足す例の covers は、その例が確かめる完了条件か分かれ道の id。外れた振る舞いが完了条件を破っているならその完了条件を入れる。
- 足す例の given ・ input が既にある例と同じなら、足さずにその例の expected ・ covers を直し、結び先を変える（同じ入力の例が2つあると、validate の examples-distinct が violated になる）。

### 層の選び方

| 外れ方 | 直す層 |
|---|---|
| 既にある決まりが外れた振る舞いをすでに言っている | examples（足す例の id を、その決まり（rules）か分かれ道（edge_cases）の examples に足して結ぶ。中身の文は変えない） |
| 外れた振る舞いを言っている決まりが無い | rules |
| 外れた場面の分かれ道が無い | edge_cases |
| 受け持つ範囲の外 | role か purpose |

手順7で例を動かしてまだ外れるなら、一つ広い層で直す（分かれ道 7a）。外れた例が一度も直されていない（作るときに書いたまま）なら、起点は examples とみなし、次は edge_cases から選ぶ。ただし層の選び方の基準で決まる層がそれより広ければそちら。

作る人が層を選んだら（手順2の完了）、足す例と選んだ層の直しを1つの JSON Patch にし、書くのは作る人が層を選んで進めてよいと答えたあと1回：`semantic-agent-schema.command` のとおり `agents-skills-creator skills update <フォルダ> --patch <JSON Patch か @<ファイル>> --if-match <ハッシュ> --reason 層：<層>`（手順3）。そのあとは手順4〜9 の順に進める。
