# 実行の結果で比べる

契約は言語に依存しない。ここでは、名前の解決が必要な言語から2つを例に置く。
どちらも、層の並びを逆にして、置き場所の名前だけで渡した場合である。
正しく検査できていれば、違反が出る。

| 例 | 読み込みの名前 | 名前を層へ対応づける情報（言語の表の契約の欄） |
|---|---|---|
| Rust（このリポジトリの acdr） | `use acd_parts::…` | Cargo.toml の crate 名 |
| TypeScript（試験データ） | `import … from "@app/core/name"` | tsconfig.json の paths |

## 例1：Rust

### 変更前（いまの Skill）

**逆の並びなのに合格する。** 読み込みの名前（crate 名）が、どの層にも一致しないためである。

```
$ no-more-spaghetti inward rust .claude/skills/acdr/rs mcp=mcp cli=cli declare=declare parts=parts
向きは内向きである ── 辺 72 件を見た
  （測り方）字面で測っている ── 経路の別名と探索路の指定は解決していない
```

### 変更後（試作）

**違反を、ファイルと行を添えて出す。**

```
食い違い　3 件 ／ 確認した依存 72 件
  ・cli/src/main.rs:11 ── cli → declare（内側が外側を参照している）
  ・declare/src/lib.rs:11 ── declare → parts（内側が外側を参照している）
  ・mcp/src/main.rs:20 ── mcp → declare（内側が外側を参照している）
  （測り方）字面で測っている ── 別名と検索パスは、次の設定から解決した：Cargo.toml 4件
```

## 例2：TypeScript

adapter が `@app/core/name` を読み込み、別のファイルが層を宣言しない `@app/stray/x` を読み込む。

### 変更前（いまの Skill）

**逆の並びなのに合格し、層を宣言しない参照も報告しない。**

```
$ no-more-spaghetti inward typescript fx/ts adapter=src/adapter core=src/core
向きは内向きである ── 辺 2 件を見た
  （測り方）字面で測っている ── 経路の別名と探索路の指定は解決していない
```

### 変更後（試作）

**違反と、層を宣言しない参照の両方を出す。**

```
食い違い　2 件 ／ 確認した依存 2 件
  ・src/adapter/label.ts:1 ── adapter → core（内側が外側を参照している）
  ・判定できていない ── src/adapter/extra.ts:1 ── src/stray/x はどの層にも属さない
  （測り方）字面で測っている ── 別名と検索パスは、次の設定から解決した：tsconfig.json 1件
```

## 10言語での結果

試作の契約の試験は、10言語すべてで7つの義務を満たした。
名前の解決が必要な言語と、ソースが自分の名前を宣言する言語の両方を含む。

| 言語 | 名前を層へ対応づける情報 |
|---|---|
| Rust ・ Go ・ TypeScript | 別名：Cargo.toml ・ go.mod ・ package.json と tsconfig.json |
| Python ・ Ruby ・ C++ | 検索パス：pyproject.toml ・ gemspec ・ CMakeLists.txt |
| Java ・ Kotlin ・ C# ・ PHP | 不要（ソースが package ・ namespace を宣言する） |
