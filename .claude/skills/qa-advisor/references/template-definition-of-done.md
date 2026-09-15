## 概要

ある作業項目がDefinition of Doneを満たしているかを判定する回答の構造を定める。

---

# 完了の定義（Definition of Done）を判定するための回答テンプレート：template-definition-of-done

## 使用条件

「このタスクの完了基準は何か」「このAcceptanceCriteriaで十分か」等、Definition of Doneの策定を求める相談

{{対象（ユーザーが提示したタスク・usecase・PBI）}}の完了基準について判断します。

---

## 階層の確認

### 相談内容は、組織・プロダクト横断で一律に適用される普遍的な基準の話か、個々のタスク固有の基準の話か。

| 条件 | 結果 |
|---|---|
| 組織・プロダクト横断 | Definition of Done（definition-of-done.mdの分類）に該当 |
| タスク固有 | Acceptance Criteria（definition-of-done.mdの分類）に該当。両者を混同しない |

---

## 判定

「{{対象}}の完了基準は「{{Definition of Done／Acceptance Criteria}}」の階層に属します。」と明示した上で、{{definition-of-done.mdの定義・両者の階層関係を引用しつつ、判定理由を説明する}}

---

## 推奨アクション

{{Definition of Doneの場合: 組織・プロダクト横断で安定的に適用できる基準か（タスクごとに変わっていないか）を確認する}}。{{Acceptance Criteriaの場合: 検証可能な基準になっているか（「頑張る」「漏れなく確認する」のような検証不能な基準になっていないか）を確認する}}

---

## 注意（アンチパターンに該当する場合のみ）

{{Definition of DoneとAcceptance Criteriaを混同している場合、あるいはScrum由来のDoDという語を別の粒度の概念に転用している場合、その混同がもたらす実害（読み手が誤ったメンタルモデルを持ち込む等）を説明する}}
