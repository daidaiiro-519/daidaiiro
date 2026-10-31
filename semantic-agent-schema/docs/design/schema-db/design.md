# SAS の JSON Schema の設計と DB の設計

本体は design.json である。この文書は、その要点を人が読む順に並べる。

## 結論

schemas/v1 は14のファイルにする。1つのファイルは1つの境界（誰が書き、誰が読むか）だけを持つ。DB は SQLite の5つの表で、DDL は公開しない。DB のどの欄も CLI の出力のどこかに出る。

## 判断基準

| 番号 | JSON Schema にするとき |
|---|---|
| 1 | 入力と出力を JSON で表したい |
| 2 | 決めた型で制御したい |
| 3 | 入力や出力に意味を持たせたい |
| 4 | 出力を決まった JSON の型で返したい |

## スキーマの一覧（schemas/v1）

| ファイル | 中身 | 書く側 → 読む側 | 判断基準 |
|---|---|---|---|
| semantic-abstract-schema.json | Abstract の本体。x- の10の値の形と、欄 ・ rule ・ 読み方の形 | SAS → Concrete を書く人と AI | 2 ・ 3 |
| vocabulary.schema.json | 入れ子に掛かる x- の形（vocab-meta から改名） | SAS → 検査器 | 2 |
| dialect.schema.json | 2020-12 に SAS の語彙を足した方言 | SAS → 検査器 | 2 |
| glossary.schema.json | 用語集の型（中身は使う側） | SAS → Concrete の $ref | 2 ・ 3 |
| operation-request.schema.json | 操作の依頼。コマンドごとの入力 | 呼び出し元 → CLI | 1 ・ 2 |
| operation-result.schema.json | 操作の結果。CLI の出力 | CLI → 呼び出し元 | 1 ・ 4 |
| procedure-prompt.schema.json | 手順のプロンプトの構造化された部分 | SAS → エージェントと呼び出し元 | 2 ・ 3 |
| session-request.schema.json | セッションの依頼と、アダプターの返り | SAS ⇄ アダプター | 1 ・ 2 |
| agent-answer.schema.json | エージェントの答え | エージェント → SAS | 2 ・ 4 |
| session.schema.json | セッションの姿（4つの型 ・ 回 ・ 終わり方） | SAS → 呼び出し元 | 4 |
| diff.schema.json | 差分（欄ごとの前後 ・ 指紋 ・ 版の参照） | SAS → 呼び出し元と手順のプロンプト | 4 |
| agent-connector.schema.json | エージェントの接続の定義と能力の宣言 | 使う側 → SAS | 1 ・ 2 |
| design-system.schema.json | デザインシステムの受け取りの型 | 使う側 → SAS | 1 ・ 2 |
| settings.schema.json | 設定（アダプター ・ 上限 ・ Concrete の置き場所） | 使う側 → SAS | 1 ・ 2 |

### Concrete を書くための x- 項目

| x- | 置く所 | 中身 |
|---|---|---|
| x-instance-path ・ x-instance-label | 根 | インスタンスの置き場所と、指されたときの名前 |
| x-delegates | 根 | rule を任せる先の立場と種類（judging ・ working）。ツールの名前は書かない |
| x-extensions | 根 | Concrete が足す x- の定義（schema ・ title ・ description ・ prompt ・ published） |
| x-prompt | 根と欄 | 読み方（shown ・ interpret ・ when）と書き方の rule |
| x-relationship ・ x-derived-from ・ x-decided-by ・ x-published | 欄 | 参照先 ・ 導く決まり ・ 人が決める欄 ・ 公開するか |
| x-view（足す） | 根と欄 | 描画の種類の名前。デザインシステムのテンプレートの種類と合わせる |

x-prompt はいまの vocab-meta に在る。足すのは x-view だけで、デザインシステムの契約が前提にしているのに、どのスキーマにも無かった。方言は `$dynamicAnchor: meta` を持ち、2020-12 のメタスキーマの `$dynamicRef: "#meta"` がここへ置き換わることで、語彙の形が入れ子のサブスキーマに掛かる（Core 8.2.3.2 ・ Appendix D.2 で照合済み）。

## スキーマにしない境界

| 境界 | 理由 |
|---|---|
| DB の表 | 公開しない決定。外部キー ・ 一意 ・ 追記だけは DDL と trigger で守る |
| 版の中身を読むポート ・ 検査器 ・ テンプレートの処理系 | プロセスの外へ JSON が出ない。検査器の出力は SAS の中で rule の結果へ直す |
| 手順のプロンプトの文 | 文は型で縛れない。縛る部分は procedure-prompt に出した |

## DB の表

| 表 | 持つもの | 主キー ・ 一意 |
|---|---|---|
| sessions | 呼び出し元のセッション（文脈「スキーマとインスタンス」） | id。開いているものはアダプター × tool_ref で1つ、人は1つ |
| launched_sessions | SAS が立てたセッション（文脈「立てたセッション」）。呼び出し元のセッションを ID で指す | id |
| session_turns | 手順のプロンプトを渡して答えを受け取った1回（開始 ・ 再開 ・ 形の直し） | (session_id, seq)。索引 (request_id, attempt) |
| operations | 操作。追記だけ。呼び出し元のセッションを session_id で、立てたセッションで行ったときはその回も指す | id。(session_id, seq)。request_id は1回（削除に伴う書き換えを除く） |
| field_changes | 変わった欄の前後 | (operation_id, pointer) |
| rule_results | rule ごとの結果（validate だけ） | (operation_id, rule_schema, rule_id, pointer) |

session_turns を足した理由は、継続のセッションが手順のプロンプトを何度も受け、形の直しも1回ずつになるためである。launched_sessions の1行には1つしか持てない。逆に、state ・ 再開できる期限 ・ 文脈の上限は回と能力の写しから導けるので、欄に持たない。

DDL は design.json の 4_db.ddl にあり、SQLite 3.46 で表を作れることと、主な CHECK ・ 一意の索引が違反を断ることを確かめた。CHECK は式が NULL のとき違反にしないので、NULL になりうる欄に = ・ IN を使う2つの CHECK を IS ・ IS NOT NULL で閉じた（照合と実測で確かめた）。必要な最低の版は、部分索引 3.8.0、JSON の関数が既定で組み込み 3.38.0。trigger は `SELECT RAISE(ABORT, '<文字列のリテラル>')` で書く（文言に式を使えるのは 3.47.0 から）。

## スキーマと DB の対応の要点

| 向き | 要点 |
|---|---|
| 書く | 操作の依頼の intent ・ brief ・ セッションの型 → operations と sessions。アダプターの返り → session_turns。エージェントの答え → field_changes（差分を求めて）と rule_results |
| 読む | sessions と session_turns → session.schema.json。operations と field_changes → diff.schema.json。rule_results → rule-result |
| DB にだけある欄 | 無い |
| スキーマにだけある欄 | 記録しないと決めたものだけ（描画 ・ 検索の入力、当てる前の patch、決めきれず返した点、検索の答え、ファイルが正本の設定） |
| 名前 | DB の名前は出力に出さない（parent_id → parent、prompt_hash → prompt.fingerprint など） |

## 意味の操作の流れ

| ステップ | 検証 | 更新 |
|---|---|---|
| CLI の入力 | operation-request の validate（型の指定は受けない） | operation-request の update（semantic ・ intent ・ brief ・ 型の指定） |
| 手順のプロンプト | procedure-prompt（judge のステップ） | procedure-prompt（write のステップ。再開で対象が変わったら差分を先頭へ） |
| CLI の1行目 | operation-result の procedure | operation-result の procedure |
| セッションの依頼 | session-request の start | session-request の start ・ continue ・ fork |
| エージェントの答え | agent-answer の judgments | agent-answer の field-values |
| 記録 | operations ＋ rule_results ＋ sessions を閉じる | operations ＋ field_changes（単発なら sessions を閉じる） |
| CLI の最後の行 | operation-result の validation | operation-result の change |

答えの各項目はステップの番号と JSON ポインタを持ち、SAS はステップにある欄だけへ操作する。形が誤れば同じセッションで2回まで直させる。時間切れは新しいセッションで2回までやり直す。

## 外したもの

| 外したもの | 引き継ぐ先 |
|---|---|
| judgment | rule_results と rule-result |
| change | operations ・ field_changes と diff |
| change-summary | 使う側（SAS は記録を読む形で材料を渡す） |
| call | 役目は残す。rules ・ inputs ・ task は procedure-prompt へ、delegate ・ handoff は session-request へ |
| Abstract の output-judging ・ output-working | agent-answer |
| delegate.run.session | 操作の依頼の型の指定と、検証を固定する決まり |

## 原典と照合した結果（2026-10-31）

8項目すべてを原文で照合した。原文は sources/ に、取得日と sha256 と一緒に残してある。

| 項目 | 原典（版） | 結果と、設計で直した所 |
|---|---|---|
| JSON Schema の出力の形 | json-schema.org Core 2020-12 の12章 | 欄名はすべて在った。一覧に absoluteKeywordLocation と error（単数）を足した。出力の形は実装に SHOULD |
| $vocabulary ・ $dynamicAnchor ・ $dynamicRef | Core 2020-12 とメタスキーマ | 方言に `$dynamicAnchor: meta` を置く。true の語彙を知らない検査器は方言を断る（MUST） |
| JSON Patch | RFC 6902 | op ・ path ・ value ・ from と op の6つが在った。「確かめる」を外した |
| Design Tokens | Design Tokens Format Module 2025.10 | 版を固定した。「群」を原典の group に直した |
| MCP の inputSchema | MCP 2026-07-28 | 根は type: "object"、外の $ref は解かれない。道具ごとに1つのスキーマへ束ねて渡す |
| コーディングエージェント | Claude Code ・ Codex CLI ・ Gemini CLI ・ Copilot CLI ・ Kiro CLI の公式文書 | 環境変数でセッション ID を取れるのは Claude Code だけ。caller.env を任意にし、無いときは --caller。tool_ref.from に assigned を足した。commands は start ・ continue を必須にし、close ・ status を省略できる形にした。fork は「非対話で fork できる」の意味 |
| SQLite | sqlite.org | CHECK の NULL のすり抜け2つを直した。最低の版と RAISE の書き方を書いた |
| Git の掃除 | git-scm.com（2.52.0 で最後に更新された頁） | 指されないオブジェクトは2週間で消える。SAS は版を残さない（Git は読むだけ）ので、この条件は設計に関わらない |

読めなかった範囲：Claude Code の sessions.md、Gemini の automation のチュートリアル、Codex の app-server、Copilot の SDK は取得していない。ここに環境変数や欄が在る可能性は残る。


## 10-31 の利用者の判断で直したこと

| こと | 直した形 |
|---|---|
| DB の役目 | 動いているセッションを回す（再開・回・形の直し・やり直し）のが主。終わったあとの記録は分析に使える程度。DB の中身はコミットしない |
| 版を残す仕組み | 持たない。refs/sas/versions/ で版を指すこと、版を残す操作、復元の話を外した |
| 差分の取り方 | SAS が書いた操作は書く前後の中身から求めて field_changes に持つ。SAS を通らない変更と、継続で対象が変わったときは、いまのファイルと Git のコミット済みの版（たとえば HEAD）を読んで比べる。前の版が無ければ before_unavailable |
| Git | 読むだけ。ポートは「版の中身を読む」（version_reader）だけ |
