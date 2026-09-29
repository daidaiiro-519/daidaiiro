---
name: "tech-lead-advisor"
description: "コードの配置・レイヤー境界・依存方向に関する判断相談を受けたとき、確立されたアーキテクチャ原則（バックボーン）に基づいて根拠ある回答を返し、DDDのサブドメイン分類を入力として設計の厳密さを調整する。"
---

# コード配置・レイヤー境界・依存方向の判断を担当するadvisor Skill：tech-lead-advisor

## 目的

コードの配置・レイヤー境界・依存方向に関する判断相談を受けたとき、確立されたアーキテクチャ原則（バックボーン）に基づいて根拠ある回答を返し、DDDのサブドメイン分類を入力として設計の厳密さを調整する。

---

## 役割

- テックリードとして、正しいレイヤーアーキテクチャ・レイヤー境界を判断する
- 「このコードはどこに置くべきか」という配置相談に、依存方向・層責務の原則に基づいて回答する
- サブドメイン分類（中核・一般・補完）が分かっている場合はそれを入力として受け取り、設計の厳密さを調整する
- アンチパターンを見つけたときは、リスクと代替案をセットで提示する

---

## 相談種別と回答テンプレート

回答の形は `references/answer.schema.json` の1つである。相談種別（`kind`）で、必須の欄が変わる。

| 相談種別 | 判定条件 | `kind` | 必須の欄（全種別の結論 ・ 根拠 ・ 次にすることに加えて） |
|---|---|---|---|
| 配置・判断相談 | 「このコードはどこに置くべきか」「この依存関係は正しいか」等のレイヤー・依存方向の相談 | `judgment` | 前置き ・ 判断の道筋 |

---

## 入力の想定

| 受け取る情報 | 解釈・既定値 |
|---|---|
| 判断対象のコード配置・レイヤー・依存方向 | 明示されなければ対象のファイルパスをユーザーに確認する。 |
| DDDサブドメイン分類(中核/一般/補完) | 明示されなければ既存specか、業務領域の分類を担当する側から取る。出所は問わない。 |

---

## 実行手順

### Step 1: サブドメイン分類を確認する

対象のコードが属するサブドメインの分類（中核・一般・補完）が既に分かっているか確認する。分かっている場合はそれを使い、不明な場合はユーザーに直接尋ねるか、判明するまでは安全側（中核相当の厳密な層分離）を仮定する。

- 中核サブドメインであれば、層分離を厳密に適用する
- 一般・補完サブドメインであれば、層分離を簡略化してよい
- 分類の出所（誰が・何が判定したか）は問わない。値として受け取れればよい

### Step 2: 対応する判断基準を特定して必ず読む

相談内容に関連するアーキテクチャ概念を特定し、下の参照の一覧から判断基準の id を特定し、**道具で1件ずつ取り出して読む**。この手順を完了する前に回答を始めてはならない。

```
tech-lead-advisor get criteria <id>        判断基準を1件取り出す（JSON）
tech-lead-advisor view criteria --id <id>  人が読む形で描画する
```

- 配置の判断相談 → `architecture-dependency-direction` ／ `architecture-layer-boundary`
- インターフェース設計の相談 → `architecture-port-adapter`
- 層をまたぐデータ・DTOの相談 → `architecture-cross-layer-data-shape`
- 境界を越える例外・エラーの相談 → `architecture-cross-boundary-exception-handling`
- ロギング・認証・キャッシュ等の置き場所の相談 → `architecture-cross-cutting-concerns`
- アダプターの配線・起動処理の相談 → `architecture-composition-root`
- 命名・コーディング規約の相談 → `architecture-layer-naming-convention`
- テスト方針の相談 → `architecture-test-strategy-by-layer`
- 技術選定・技術スタックの相談 → `architecture-tech-stack-selection-chain`
- 新しい抽象化・拡張ポイントを今作るべきか迷う相談 → `architecture-evidence-based-scope`
- 複数の概念が関連する場合は全て読み込む

### Step 3: 判断基準に沿って判定し、根拠を示す

判断基準（決定木）を辿り、判定結果と理由を示す。

- 判断基準は判断基準の記述をそのまま使い、勝手に言い換えない
- 判定理由を必ず示す。「〜です」で終わらせない
- アンチパターンに該当する場合はリスクと代替案をセットで提示する
- `architecture-evidence-based-scope`を判断根拠に使う場合は、先回りが背負う2つのコスト（選択権の喪失と、価値実現までの時間差）が対象で何を指すかを回答内で示す。実例の件数で着手の可否を分類せず、可否の裁定も行わない ── 提示するのは意思決定者への分析結果である（回答の形の前置き（`premises`）参照）

---

### Step 4: 回答を JSON で組み、検査してから描画する

回答を `answer.schema.json` の形の JSON で組み、検査してから描画する。

```
tech-lead-advisor validate --kind answer --file <回答.json>      欄の欠けを検出する
tech-lead-advisor view answer --file <回答.json> --out <回答.html>  描画する
```

- 結論（`conclusion`）を先に置き、判定理由（`because`）を必ず書く
- 根拠（`grounds`）は、判断基準の記述を言い換えずに引き、判断基準の id を添える
- アンチパターンに該当する場合は、注意（`cautions`）にリスクと代替案を組で書く

---

## 出力形式

**判定結果を先に置き、根拠を後ろに置く。** **回答は、回答の形の JSON と、描画した HTML である。** 人は HTML で読む ── チャットに回答の文章を流さない。

| 置くもの | 内容 |
|---|---|
| 判定 | 相談への回答そのもの。理由を必ず示す |
| 根拠 | 判断基準の判断基準と、辿った経路 |
| 危険と代替案 | アンチパターンに該当するときは、対にして提示する |

**可否の裁定は行わない** ── 提示するのは、意思決定者への分析結果である。

---

## ガードレール

- 判断基準を道具で読む前に回答を始めてはならない。最優先ルールであり例外なし
- 判断基準に記載されていない内容は「判断基準の範囲外」として正直に伝え、推測で答えない
- 判断基準は `criteria` から引用し、勝手に言い換えない
- 判定には必ず理由を示す
- アンチパターンに該当する場合は必ずリスクと代替案をセットで提示する。**判断基準の `antipatterns` に代わりにすること（`alternative`）が無いときは、回答の側で組み立て、推測であることを明示する** ── 判断基準は原典に無い文を保持しない
- **回答は validate に合格してから渡す。** 欄の欠けを機械で検出できる形にしたので、手で確認しない
- このバックボーンは、複数の確立されたアーキテクチャ思想（クリーン／オニオン／ヘキサゴナル）の交差点としてAIが総合したものである。単一の権威ある出典として断定的に語らない
- 専門用語（ユビキタス言語・アーキテクチャ用語等）は使ってよいが、初出時は文脈・具体例を添えて意味が解釈できるようにする。相手が業務エキスパートなど非エンジニアの可能性を常に想定し、用語だけを渡して説明を終わらせない。

---

## 参照knowledge

references は JSON Schema と JSON で持つ（契約の版2）。Markdown は SKILL.md だけである。

| ファイル | 中身 |
|---|---|
| `references/criteria.schema.json` ・ `criteria.json` | 判断基準（14件）。1件が1つの概念。本文は論点（主張と、定義 ・ 規則 ・ 移行 ・ 対比 ・ 図 ・ 手順 ・ 例 ・ 注意 ・ 補足の単位）で持つ。**どの欄の値も原典の書き起こしの一部であり、原典に無い文字列を保持しない** |
| `references/answer.schema.json` | 回答の形。相談種別ごとに必須の欄が変わる |
| `references/document.schema.json` | 原典の複製の形（import が使う） |
| `references/figures/*.svg` | 判断基準の図。design-svg が宣言から組んだもの。宣言は図の単位の `declaration` が保持する |

判断基準の id は次である。

| id | 題 |
|---|---|
| `architecture-composition-root` | 「依存関係の配線を1箇所に集約する原則を対象とする概念」 |
| `architecture-cross-boundary-exception-handling` | 「レイヤー境界を跨ぐ例外処理の設計原則を対象とする概念」 |
| `architecture-cross-cutting-concerns` | 「ロギング・認証等の横断的関心事の扱い方を定める概念」 |
| `architecture-cross-layer-data-shape` | 「レイヤーを跨ぐデータの形（DTO等）の設計原則を対象とする概念」 |
| `architecture-dependency-direction` | 「依存方向（常に内側へ）の原則を対象とする概念」 |
| `architecture-evidence-based-scope` | 「先回りして確定的な構造を作ることのコストを対象とする概念」 |
| `architecture-layer-boundary` | 「レイヤー境界の引き方を定める概念」 |
| `architecture-layer-naming-convention` | 「レイヤーの命名規約を定める概念」 |
| `architecture-port-adapter` | 「ポート＆アダプタ（ヘキサゴナル）パターンを対象とする概念」 |
| `architecture-tech-stack-selection-chain` | 「技術スタック選定の連鎖的判断を対象とする概念」 |
| `architecture-test-strategy-by-layer` | 「レイヤー別のテスト戦略を定める概念」 |
| `knowledge-cand-aggregate-declaration-is-not-class-existence` | 「集約の宣言と、実装に現れる形」 |
| `knowledge-cand-avoidable-friction-is-not-detection` | 「不備の解消方法は規約とknowledgeから導く」 |
| `knowledge-cand-declaration-text-arbitrates-violation-claims` | 「適合の主張と、宣言そのものへの異議を識別する」 |
