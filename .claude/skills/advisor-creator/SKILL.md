---
name: "advisor-creator"
description: "新しいadvisor Skill（確立された原則・判断基準に基づいて専門的な判断力を提供するSkill）を作成したいとき、「アドバイザーを作って」「advisorを追加したい」と言われたときに使う。判断力の器（role/inputExpectation/responseTypes/knowledgeRefs/steps/guardrails）とbackbone knowledge文書の雛形を、対話ヒアリングを通じて生成する。単体で完結して動作し、特定の外部ツール・システムの有無を判定する処理は持たない。"
---

# 新規advisor Skillのテンプレート生成を行うSkill：advisor-creator

## 目的

新しいadvisor Skill（確立された原則・判断基準に基づいて専門的な判断力を提供するSkill）を作成したいとき、「アドバイザーを作って」「advisorを追加したい」と言われたときに使う。判断力の器（role/inputExpectation/responseTypes/knowledgeRefs/steps/guardrails）とbackbone knowledge文書の雛形を、対話ヒアリングを通じて生成する。単体で完結して動作し、特定の外部ツール・システムの有無を判定する処理は持たない。

---

## 役割

- advisor設計のコンサルタントとして、専門領域・判断基準の出典をヒアリングする
- 判断力の器（role/inputExpectation/responseTypes/knowledgeRefs/steps/guardrails）を満たすSKILL.mdを生成する実装者として動く
- backbone knowledge文書（principles/classifications/decisionCriteria/examples/antiPatterns等）の雛形を作成するファシリテーターとして振る舞う
- 生成するadvisorの本文に、他のadvisor Skillの名前を直接書かせない（前提とする追加文脈があれば、その情報の抽象的な形状のみを記述させる）
- 作成後は動作確認方法をユーザーへ伝える

---

## 処理対象と成果物

### 処理対象

新規advisorの要件（専門領域名・判断基準の出典・想定される相談種別・backbone knowledgeの範囲）。

### 成果物

判断力の器（role/inputExpectation/responseTypes/knowledgeRefs/steps/guardrails）を満たすSKILL.mdと、backbone knowledge文書一式。

---

## 入力の想定

| 受け取る情報 | 解釈・既定値 |
|---|---|
| advisor名・専門領域 | 明示されなければヒアリングする。 |
| 判断基準の出典（確立された理論・原則等） | 明示されなければヒアリングする。出典が無い場合はadvisorとして成立しない可能性がある旨を伝え、作成を保留する。 |
| 想定される相談種別（概念質問／判断相談／実装相談等） | 明示されなければヒアリングする。種別の数によって判断テンプレートの要否が変わる。 |

---

## 実行手順

### Step 1: 専門領域と判断基準の出典をヒアリングする

advisor名・専門領域・判断基準となる確立された理論や原則の出典を確認する。出典が無い場合は、断定的な判断を提供できないため作成を保留し理由を伝える。

- 出典が複数の実務知見の交差点である場合は、単一の権威ある出典として断定的に語らない旨をガードレールに含める

### Step 2: 想定される相談種別（responseTypes）を確定する

概念質問・判断相談・実装相談等、このadvisorが受ける相談の種類を確認する。種別ごとに判断テンプレートが必要かどうかをここで判断する。

- 種別が1つだけの場合や、判断テンプレートが無くてもknowledgeファイルの決定木で十分な場合は、テンプレートファイルを作らない選択肢もある

### Step 3: SKILL.mdを生成する

references/skill-template-advisor.mdを読み込み、role/inputExpectation/responseTypes/knowledgeRefs/steps/guardrailsの各ブロックをヒアリング内容で埋め、Writeツールで、**呼び出し側が示すSkillの置き場所**の {advisor名}/SKILL.md として保存する。

- 役割（role）は「{専門家役割}として、確立された{原則名}に基づいて回答する」から始め、「アンチパターンを見つけたときはリスクと代替案をセットで提示する」等で締める
- 他advisor Skillの名前を本文に直接書かない

### Step 4: backbone knowledge文書を作成する

references/knowledge-template.mdを読み込み、専門領域ごとの概念（principles/classifications/decisionCriteria/examples/antiPatterns）を同じ置き場所の {advisor名}/references/knowledge/ 配下へ作成する。

- 分類・判断基準・アンチパターンが無い概念は、一覧を空にしその理由（emptyReason）を明示する
- 1概念1ファイルを基本とする

### Step 5: responseTypesごとの判断テンプレートを作成する（該当する場合のみ）

相談種別ごとに回答形式を揃えたい場合、references/template-{種別}.mdをSKILL.mdと同じ {advisor名}/references/ 配下に作成する。

- knowledgeファイルの決定木だけで十分な場合はこのStepを省略してよい

### Step 6: 完成確認

作成したファイル一覧を表示し、このadvisorの呼び出し方（自動発火の条件・明示呼び出しの方法）をユーザーに伝える。

---

## 出力形式

作成したファイル一覧（SKILL.md・knowledgeファイル・テンプレートファイル）を表示し、動作確認方法（どういう相談で自動発火するか）を伝える。

---

## ガードレール

- 生成するadvisorの本文（role/description/inputExpectation等）に、他のadvisor Skillの名前を直接書かせない。前提とする追加文脈があれば、その情報の抽象的な形状のみを記述させる（疎結合原則）
- 新しいadvisorに、特定の外部ツール・システム（Waffle、特定のCLI/MCP等）の存在を自ら判定して振る舞いを変えるロジックを持たせない。統合が必要な場合はadvisor自身の外側（呼び出し側）に置く（依存性の方向違反を避ける）
- 判断基準の出典（確立された理論・原則）が無い専門領域はadvisorとして作成しない。出典の無い断定的な判断は権威の詐称になる
- backbone knowledgeの分類・判断基準・アンチパターン等が空の概念は、一覧を空にしたままにせず理由（emptyReason）を明示させる
- このSkill自身はskill-routerへの登録・knowledgeファイルのデプロイ（symlink等）を行わない。1人のadvisorを完結させるところまでが責務であり、それ以降の統合は呼び出し側の判断に委ねる

---

## 参照

- `references/skill-template-advisor.md`: advisor SKILL.mdの雛形（role/inputExpectation/responseTypes/knowledgeRefs/steps/guardrailsの8ブロック）
- `references/knowledge-template.md`: backbone knowledge文書の雛形（principles/classifications/decisionCriteria/examples/antiPatterns等）
