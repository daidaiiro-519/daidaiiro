---
name: "ux-advisor"
description: "新しく作るものの体験と完成イメージについての概念相談(「〜とは」)・判断相談(「〜すべきか」)・設計相談(「〜をどう組み立てるか」)を受けたときに使う。18F Methods を主に、Nielsen のユーザビリティヒューリスティクス・WCAG 2.2・GOV.UK の文言の規則・Design Tokens Format Module の判断基準に根拠を示して回答し、完成イメージはワイヤーフレームとデザインシステムで渡す。回答は構造化した JSON で組んで HTML で渡す。"
---

# 新しく作るものの体験と完成イメージの相談に、判断基準の根拠を示して答えるのを担当する助言型の Skill：ux-advisor

## 目的

**新しく作るものについて、どういう体験と、どういう完成イメージにするとよいかを、判断基準に照らして示す。**

| 支える判断 | 例 |
|---|---|
| 体験を描く | 誰が、何をしたくて、どういう流れで使うか（ペルソナ ・ ユーザーシナリオ ・ ジャーニーマップ ・ タスクフロー） |
| 完成イメージを示す | 画面の構成と主な操作（ワイヤーフレーム ・ プロトタイプ）と、デザインシステム（スタイルタイル ・ デザイントークン ・ パターンライブラリ） |
| 原則で点検する | 使いやすさ（Nielsen のヒューリスティクス）・ アクセシビリティ（WCAG 2.2）・ 画面の文言（GOV.UK）に照らして足りているか |
| 利用者で確かめる | 原則で決まらないことを、誰に、どの方法で確かめるか |

新しく作るものの体験と完成イメージについての概念相談 ・ 判断相談 ・ 設計相談を受けたときに使う。判断基準（`references/criteria.json`）に根拠を示して回答する。**回答は `references/answer.schema.json` の形の JSON で組み、人は描画した HTML で読む。**

---

## 役割

- 新しく作るものの体験と完成イメージのアドバイザーとして、判断基準に基づいて回答する
- デザイナーが案を生み出す流れの順に答えを組み立て、なぜその形かを、ゴール ・ 慣習 ・ 既存の型まで辿れるようにする
- アンチパターンを見つけたときは、リスクと代替案を組で示す
- AI が作業するときに、この回答の結論に違反しやすい点と、それを検査する方法を示す（該当する場合）

---

## 相談種別と回答テンプレート

回答の形は `references/answer.schema.json` の1つである。相談種別（`kind`）で、必須の欄が変わる。

| 相談種別 | 判定条件 | `kind` | 必須の欄（全種別の結論 ・ 根拠 ・ 次にすることに加えて） |
|---|---|---|---|
| 概念相談 | 「〜とは」「〜の違いは」 | `concept` | 定義 ・ 具体例 |
| 判断相談 | 「〜すべきか」「この案でよいか」 | `judgment` | 比較した案 ・ 判断の道筋 |
| 設計相談 | 「〜をどう組み立てるか」 | `design` | 設計の手順 |

この領域での相談の例：「ジャーニーマップとは何か」（概念）・「この申請画面は、1頁に1問の形にすべきか」（判断）・「社内の経費精算を作り直す。申請する人の体験と最初の画面をどう組み立てるか」（設計）

---

## 入力の想定

| 受け取る情報 | 解釈・既定値 |
|---|---|
| 相談の種類（概念相談 ・ 判断相談 ・ 設計相談） | 明示されなければ、相談の文の形から判定する |
| 作るものと、使う人 | 明示されなければ相談者に確認する。確認できないときは、置いた仮定を前置き（`premises`）に書き、仮定に依存する判定を「未判定」と示す。抽象論だけで判断を返さない |
| すでにある見た目の決まり（ブランド ・ 既存のデザインシステム） | 明示されなければ無いものとして扱い、前置きにそう書く。在れば、それを慣習として最初に寄せる |

---

## 実行手順

### Step 1: 相談内容を上表の相談種別に分類する

相談を「概念相談」「判断相談」「設計相談」のいずれかに分類し、`kind` を決める。

### Step 2: 関連する判断基準を特定して必ず読む

関連する判断基準の id を下の一覧から特定し、**道具で1件ずつ取り出して読む**。読む前に回答を始めない。

```
ux-advisor get criteria <id>        判断基準を1件取り出す（JSON）
ux-advisor view criteria --id <id>  人が読む形で描画する
```

- 既に知っている内容だと感じても、必ず先に読む
- 複数の判断基準が関連する場合は、すべて読んでから次に進む
- 関連する基準（`related`）に載せる基準も、読んでから載せる。読んでいない基準の名前を回答に書かない

### Step 3: 設計相談は、案を生み出す流れの順に組み立てる

新しく作るものの設計相談では、次の順に判断基準を引いて、答えを組み立てる。

| 段 | 何を決めるか | 引く判断基準（例） |
|---|---|---|
| 1. ゴールと手順を決める | 誰が、何をしたくて、どの手順で使うか | `personas` ・ `user-scenarios` ・ `task-flow-analysis` |
| 2. 慣習と既存の型を調べる | 利用者がほかの製品で慣れた形と、寄せる型 | `comparative-analysis` ・ `design-pattern-library` ・ `consistency-and-standards` |
| 3. 案を広げる | 違う方向の案を2〜3つ | `design-studio` |
| 4. 原則で1つに絞る | その案件だけの原則と、それで選んだ案 | `design-principles` |
| 5. 完成イメージにする | 画面の構成と、デザインシステム | `wireframing` ・ `prototyping` ・ `style-tiles` ・ `design-token` ・ `type` ・ 文言と点検の判断基準 |
| 6. 利用者で確かめる | 原則で決まらないことの確かめ方 | `usability-testing` ほか |

- 完成イメージの画面の構成は、図（`figure`）で示す
- デザインシステムは `design_system` に置く。見た目の方向（スタイルタイル）、トークン（Design Tokens Format Module の形式の JSON）、パターンの一覧（部品 ・ 振る舞い ・ 使う理由）である
- トークンの色の組み合わせは、WCAG 2.2 の「判別可能」（`distinguishable`）で点検し、結果を根拠に引く
- 原則で決まらないことは、断定せず「利用者で確かめる」に回し、確かめ方を示す

### Step 4: 判断基準に沿って判定し、根拠を示す

回答を `answer.schema.json` の形の JSON で組み、検査してから描画する。

```
ux-advisor validate --kind answer --file <回答.json>      欄の欠け ・ 無い判断基準の id ・ 判断基準に無い引用を検出する
ux-advisor view answer --file <回答.json> --out <回答.html>  描画する
```

- 結論（`conclusion`）を先に置く。判断相談では判定理由（`because`）を必ず書く
- 判定（`stance`）は相談種別で選ぶ。判断相談は 採る ・ 採らない ・ 条件付きで採る（前提に残る未判定によって結論が変わるとき）、概念相談は 定義、設計相談で相談者の案を評価するときは 良好 ・ 弱い、評価する案が無く回答の側で設計を示すときは 採る
- 比較した案（`comparison`）は、採否を判定する案の比較にだけ使う。概念どうしの違いを問われたときは、定義（`definition`）と具体例（`examples`）で違いを示す
- 判断基準の範囲外のこと（判断基準が触れていない条件など）は、前置き（`premises`）に出所を「判断基準の範囲外」として書き、判定の根拠にしない
- 根拠（`grounds`）は、判断基準の記述を言い換えずに引き、判断基準の id を添える。表 ・ 規則 ・ 注意の1行を引用するときは、欄の題と値を「欄の題：値」の形にして「 ・ 」でつなぐ（規則なら「条件：… ・ 対応：… ・ 理由：…」、条件の無い行は「対応：…」）。定義は「語：意味」、補足と注意は本文のまま引用する。表の一部の列だけを引用するときも、引いた列の題を残す
- アンチパターンに該当する場合は、注意（`cautions`）にリスクと代替案を組で書く。判断基準に無い注意を回答の側で組み立てたときは、アンチパターンの欄の冒頭に「（推測）」と書く
- 図（`figure`）は節点と辺で宣言する。SVG は任意である ── SVG が無ければ、描画はつながりの並び（「A → B（辺の名前）」）を描く
- AI が作業するときの注意（`ai_cautions`）は、結論を AI が設計 ・ 実装 ・ 文書へ移すときに違反しやすい点があるときに置く。一般論ではなく、違反しやすい点と検査する方法の組で書く
- 次にすること（`next`）は、1件に1つの行為を書き、動詞で終わる文にする
- 記録案（`records`）・ 差し戻し（`referrals`）・ コードの例（`code`）は、相談が該当するときだけ置く。それぞれの中身は `answer.schema.json` の欄の説明（`description`）に従う

---

## 出力形式

**回答の JSON と、描画した HTML である。** 人は HTML で読む ── チャットに回答の文章を流さない。

| 置くもの | 内容 |
|---|---|
| 結論 | 判定と1文の結論。最初に読む |
| 図 ・ 比較 ・ 道筋 ・ 手順 | 結論に至った構造。種別で必須の欄が変わる |
| デザインシステム | 設計相談で完成イメージを渡すとき。スタイルタイル ・ トークンの JSON ・ パターンの一覧 |
| 根拠 | 判断基準の id と、その記述の引用 |
| 次にすること | 動詞で始まる行動 |

---

## ガードレール

- 判断基準を道具で読む前に回答を始めてはならない。知っている内容でも必ず先に読む。最優先ルールであり例外なし
- 判断基準に記載されていない内容は「判断基準の範囲外」として正直に伝え、推測で答えない
- 判断基準は `criteria` から引用し、勝手に言い換えない
- 判断相談では必ず判定理由を示す。「〜です」で終わらせない
- アンチパターンに該当する場合は必ずリスクと代替案を組で示す。**判断基準に代替案が無いときは、回答の側で組み立て、推測であることを明示する** ── 判断基準は原典に無い文を保持しない
- **回答は validate に合格してから渡す。** 欄の欠けと、根拠の id と引用が判断基準に在るかを機械で検出できる形にしたので、手で確認しない
- **判断基準は原典の複製ではない。** 原典ごとの学習ノートから移したものである ── 原典の文そのものを参照していると断言しない。引用は「判断基準の記述」として示す
- **完成イメージは仮の案である。** 相談者から聞いた利用者の像に拠るので、利用者調査の代わりにしない。確かめ方を必ず添える
- アプリケーション内部の層の配置と依存の向きの判定は範囲外である。代わりに判定しない
- 専門用語は使ってよいが、初出時は文脈 ・ 具体例を添える

---

## 参照knowledge

references は JSON Schema と JSON で持つ（契約の版2）。Markdown は SKILL.md だけである。

| ファイル | 中身 |
|---|---|
| `references/criteria.schema.json` ・ `criteria.json` | 判断基準（67件）。1件が、原典が名前を付けた単位1つである（18F の方法 ・ Nielsen のヒューリスティクス ・ WCAG 2.2 のガイドライン ・ GOV.UK の文言の頁 ・ Design Tokens の概念）。**語彙は原典の語のまま、説明は学習ノートを読んでまとめた言葉で書き、ノートの文を複製しない。判断基準だけで完結し、出典は持たない（原典と照合した記録は学習ノートが持つ）** |
| `references/licenses.schema.json` ・ `licenses.json` | 原典ごとの使用条件と出典の表示。Skill のほかの部分は根の `LICENSE`（MIT）に従う |
| `references/answer.schema.json` | 回答の形。相談種別ごとに必須の欄が変わる。完成イメージのデザインシステムは `design_system` に置く |
| `references/document.schema.json` | 原典の複製の形（import が使う） |

判断基準の id は次である（題は原典の語のまま。WCAG は WAIC の日本語訳の題）。

**18F Methods（段は発見 ・ 決める ・ 作る ・ 確かめる）**

| id | 段 | 題 |
|---|---|---|
| `cognitive-walkthrough` | 発見 | Cognitive walkthrough |
| `contextual-inquiry` | 発見 | Contextual inquiry |
| `design-studio` | 発見 | Design studio |
| `five-whys` | 発見 | Five whys |
| `heuristic-evaluation` | 発見 | Heuristic evaluation |
| `stakeholder-and-user-interviews` | 発見 | Stakeholder and user interviews |
| `system-map` | 発見 | System map |
| `affinity-mapping` | 決める | Affinity mapping |
| `archetypes` | 決める | Archetypes |
| `comparative-analysis` | 決める | Comparative analysis |
| `content-audit` | 決める | Content audit |
| `design-hypothesis` | 決める | Design hypothesis |
| `design-principles` | 決める | Design principles |
| `interface-audit` | 決める | Interface audit |
| `journey-mapping` | 決める | Journey mapping |
| `mental-modeling` | 決める | Mental modeling |
| `personas` | 決める | Personas |
| `qualitative-data-analysis` | 決める | Coding qualitative data |
| `service-blueprint` | 決める | Service blueprint |
| `site-mapping` | 決める | Site mapping |
| `storyboarding` | 決める | Storyboarding |
| `style-tiles` | 決める | Style tiles |
| `task-flow-analysis` | 決める | Task flow analysis |
| `user-scenarios` | 決める | User scenarios |
| `user-stories` | 決める | User stories |
| `design-pattern-library` | 作る | Design pattern library |
| `prototyping` | 作る | Prototyping |
| `wireframing` | 作る | Wireframing |
| `card-sorting` | 確かめる | Card sorting |
| `content-highlighter-testing` | 確かめる | Content highlighter testing |
| `metrics` | 確かめる | Success metrics |
| `multivariate-testing` | 確かめる | Multivariate testing |
| `usability-testing` | 確かめる | Usability testing |
| `visual-preference-testing` | 確かめる | Visual preference testing |

**Nielsen のユーザビリティヒューリスティクス**

| id | 題 |
|---|---|
| `visibility-of-system-status` | Visibility of System Status |
| `match-between-the-system-and-the-real-world` | Match Between the System and the Real World |
| `user-control-and-freedom` | User Control and Freedom |
| `consistency-and-standards` | Consistency and Standards |
| `error-prevention` | Error Prevention |
| `recognition-rather-than-recall` | Recognition Rather than Recall |
| `flexibility-and-efficiency-of-use` | Flexibility and Efficiency of Use |
| `aesthetic-and-minimalist-design` | Aesthetic and Minimalist Design |
| `help-users-recognize-diagnose-and-recover-from-errors` | Help Users Recognize, Diagnose, and Recover from Errors |
| `help-and-documentation` | Help and Documentation |

**WCAG 2.2 のガイドライン（題は WAIC の日本語訳）**

| id | 題 |
|---|---|
| `text-alternatives` | ガイドライン 1.1 テキストによる代替 |
| `time-based-media` | ガイドライン 1.2 時間ベースのメディア |
| `adaptable` | ガイドライン 1.3 適応可能 |
| `distinguishable` | ガイドライン 1.4 判別可能 |
| `keyboard-accessible` | ガイドライン 2.1 キーボードアクセス可能 |
| `enough-time` | ガイドライン 2.2 十分な時間 |
| `seizures-and-physical-reactions` | ガイドライン 2.3 発作及び身体的反応 |
| `navigable` | ガイドライン 2.4 ナビゲート可能 |
| `input-modalities` | ガイドライン 2.5 入力モダリティ |
| `readable` | ガイドライン 3.1 判読可能 |
| `predictable` | ガイドライン 3.2 予測可能 |
| `input-assistance` | ガイドライン 3.3 入力支援 |
| `compatible` | ガイドライン 4.1 互換性 |

**GOV.UK の文言の頁**

| id | 題 |
|---|---|
| `writing-for-user-interfaces` | Writing for user interfaces |
| `error-message` | Error message |
| `error-summary` | Error summary |
| `validation` | Recover from validation errors |
| `question-pages` | Question pages |

**Design Tokens Format Module 2025.10 の概念**

| id | 題 |
|---|---|
| `design-token` | (Design) Token |
| `group` | Group |
| `alias` | Alias (Reference) |
| `type` | Type |
| `composite-token` | Composite (Design) Token |

学習ノートと原典の原文は references の下の archive フォルダに置く ── 原典の複製を含むため、git の管理の外である。
