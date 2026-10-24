# 依存の規則を、既存ツールはどう書いているか

## この文書の範囲

アーキテクチャと言語に寄らない「依存の向きの検査ツール」を内作する設計の材料である。
7つの対象について、規則の書き方を原文から集め、引用は全部 `fact-check verify --as quote` で照合した。

- 表の各欄は、`Q番号`（末尾の「引用の一覧」にある原文の引用）か、「原文に無い」か、「推測」のどれかを持つ
- 「原文に無い」は、**下の「取得した原文」の範囲を読んで無かった**という意味である。同じツールの別のページには在りうる
- 識別子は原文の綴りのまま書く

---

## 取得した原文

取得日はすべて 2026-10-24。置き場所は `.brainstorming/dependency-rules/sources/`。各ファイルの隣に `<名前>.meta.json`（URL ・ 取得日時 ・ sha256 ・ バイト数 ・ 行数）がある。
GitHub の raw は commit を固定して取得した（dependency-cruiser と go-arch-lint の README だけは枝名 `main` ・ `master` の URL で取得。取得時の先頭 commit は dependency-cruiser `60f8a7ef72cbd69a01f62c61c8efcedd8a67d83c`、go-arch-lint `bf473afb033c7c3eb7aba68dc399cf919287131e`）。

| 略号 | 対象 | URL | 行数 | sha256 |
|---|---|---|---|---|
| il-index | 1 import-linter | https://raw.githubusercontent.com/seddonym/import-linter/857ae28bc9cb383ce9c831e7e8345eb9b40667ae/docs/contract_types/index.md | 116 | `8b9d317539d4a35db03d54d37e1868b34632b7b337ceda60e42f8e89d5572291` |
| il-layers | 1 import-linter | https://raw.githubusercontent.com/seddonym/import-linter/857ae28bc9cb383ce9c831e7e8345eb9b40667ae/docs/contract_types/layers.md | 332 | `10cc9a7b574f3d9c04fe128f59b92ac9956ce7ca617d71dd5b7b137a5bdd2ace` |
| il-forbidden | 1 import-linter | https://raw.githubusercontent.com/seddonym/import-linter/857ae28bc9cb383ce9c831e7e8345eb9b40667ae/docs/contract_types/forbidden.md | 179 | `c6721c3f099dd6ab3be9b9cfc4328bd10471011ae014fad6852dd426e4b0fbbc` |
| il-independence | 1 import-linter | https://raw.githubusercontent.com/seddonym/import-linter/857ae28bc9cb383ce9c831e7e8345eb9b40667ae/docs/contract_types/independence.md | 47 | `f938a5e298e8ebcad6e84d6a71b6afd0adff3b9f556e1b80da367088aec58c21` |
| il-protected | 1 import-linter | https://raw.githubusercontent.com/seddonym/import-linter/857ae28bc9cb383ce9c831e7e8345eb9b40667ae/docs/contract_types/protected.md | 125 | `352f5279f1454b73172975bcd86ab777d8808adc6f81f0978fa1a868961e715c` |
| il-acyclic | 1 import-linter | https://raw.githubusercontent.com/seddonym/import-linter/857ae28bc9cb383ce9c831e7e8345eb9b40667ae/docs/contract_types/acyclic_siblings.md | 119 | `775fdd2365c0ab4046b13714cb7322bf4cbd84f3871ed33a03664e07a16f9142` |
| il-configure | 1 import-linter | https://raw.githubusercontent.com/seddonym/import-linter/857ae28bc9cb383ce9c831e7e8345eb9b40667ae/docs/get_started/configure.md | 124 | `ba712c5c733436818ac4656ae650acb0495975ec888c5bdb6a71eff6e594793d` |
| il-run | 1 import-linter | https://raw.githubusercontent.com/seddonym/import-linter/857ae28bc9cb383ce9c831e7e8345eb9b40667ae/docs/get_started/run.md | 116 | `b4174a6d55490b26b8a8630d1b18552f95e74dedf96107da88a4c8767db7056c` |
| dc-rules | 2 dependency-cruiser | https://raw.githubusercontent.com/sverweij/dependency-cruiser/main/doc/rules-reference.md | 1311 | `c89726c4e300bcd45894e7433d6360ed29327ccde74ff357a59ab9bbf876a334` |
| dc-cli | 2 dependency-cruiser | https://raw.githubusercontent.com/sverweij/dependency-cruiser/main/doc/cli.md | 1563 | `8371319f3cfb000c4bd805b3a71794e0f711977b9c540468b241e714c42e20d8` |
| au-003 | 3 ArchUnit | https://raw.githubusercontent.com/TNG/ArchUnit/5f0884eb24991b46c20dff1d8876c5d3792fcfe2/docs/userguide/003_Getting_Started.adoc | 125 | `f6162fe6f40f750413c5aca954fdbcd59487e3a03224daac365832809869f4e6` |
| au-004 | 3 ArchUnit | https://raw.githubusercontent.com/TNG/ArchUnit/5f0884eb24991b46c20dff1d8876c5d3792fcfe2/docs/userguide/004_What_to_Check.adoc | 335 | `1ef27629953955cde9ebda1026e311c85e053f5e381e5f3c2a474cf7f3f37cc4` |
| au-008 | 3 ArchUnit | https://raw.githubusercontent.com/TNG/ArchUnit/5f0884eb24991b46c20dff1d8876c5d3792fcfe2/docs/userguide/008_The_Library_API.adoc | 816 | `65a77c7488aee53dda2061033eb0049fa26708ec819e3ec2db454f86b2474f63` |
| au-009 | 3 ArchUnit | https://raw.githubusercontent.com/TNG/ArchUnit/5f0884eb24991b46c20dff1d8876c5d3792fcfe2/docs/userguide/009_JUnit_Support.adoc | 287 | `b421db5b457914f53da1f3ef429d1e0b79f457b94bd74ac0adc46d7e0df55369` |
| au-010 | 3 ArchUnit | https://raw.githubusercontent.com/TNG/ArchUnit/5f0884eb24991b46c20dff1d8876c5d3792fcfe2/docs/userguide/010_Advanced_Configuration.adoc | 228 | `c3e07a0d193285932780a95c9e451471c51d8e0ce11f3926bf475f1379dbb690` |
| au-src | 3 ArchUnit（実装） | https://raw.githubusercontent.com/TNG/ArchUnit/5f0884eb24991b46c20dff1d8876c5d3792fcfe2/archunit/src/main/java/com/tngtech/archunit/library/Architectures.java | 1137 | `8254d756a6a00bc25670cd3e8e2319878805afea9293488924a64816278a34e5` |
| dt-concepts | 4 deptrac | https://raw.githubusercontent.com/deptrac/deptrac/66595d1ca307afff492d99501ffaeba147e848b0/docs/concepts.md | 327 | `4e5de4eb3f6e70d9a03e6faa3d982d3b6e93a3418c05da7fdf25fb7ee6e0a877` |
| dt-config | 4 deptrac | https://raw.githubusercontent.com/deptrac/deptrac/66595d1ca307afff492d99501ffaeba147e848b0/docs/configuration.md | 347 | `adb5f2c91b3807b577811ecab070aa79c46bb3e17ba963e04279d53561c826d5` |
| dt-collectors | 4 deptrac | https://raw.githubusercontent.com/deptrac/deptrac/66595d1ca307afff492d99501ffaeba147e848b0/docs/collectors.md | 387 | `155455441314ee470a6e6b7c3aea497e758a14fca533b1db8e2344ffb3ea3caa` |
| dt-formatters | 4 deptrac | https://raw.githubusercontent.com/deptrac/deptrac/66595d1ca307afff492d99501ffaeba147e848b0/docs/formatters.md | 469 | `7e0db3cb7525b639ad659e23664b0b29c6060e09f2c4f4eac71e1ca4de32ee56` |
| nx-features | 5 Nx | https://raw.githubusercontent.com/nrwl/nx/7c8e7070e404fba91954a51e932cea3e86970b69/astro-docs/src/content/docs/features/enforce-module-boundaries.mdoc | 351 | `9a8c52c1f8fad738853dff5511885fc731ae0f8a74e05f646d667eadc8afc688` |
| nx-kb | 5 Nx | https://raw.githubusercontent.com/nrwl/nx/7c8e7070e404fba91954a51e932cea3e86970b69/astro-docs/src/content/docs/kb/enforce-module-boundaries.mdoc | 147 | `c1f4464622ab8e125fb60314ad0d57acba13c6c141e2319d21b355e8b03caafa` |
| nx-ban | 5 Nx | https://raw.githubusercontent.com/nrwl/nx/7c8e7070e404fba91954a51e932cea3e86970b69/astro-docs/src/content/docs/guides/Enforce%20Module%20Boundaries/ban-dependencies-with-tags.mdoc | 47 | `b68d791290473a0b06e33664244c8951f3310183155b49118ea10c30f76ce00d` |
| nx-allow | 5 Nx | https://raw.githubusercontent.com/nrwl/nx/7c8e7070e404fba91954a51e932cea3e86970b69/astro-docs/src/content/docs/guides/Enforce%20Module%20Boundaries/tags-allow-list.mdoc | 15 | `8a57001ec6a21e6d971825c6a0c36574ae5f5036723118065910ca05ee122730` |
| nx-ts | 5 Nx（実装） | https://raw.githubusercontent.com/nrwl/nx/7c8e7070e404fba91954a51e932cea3e86970b69/packages/eslint-plugin/src/rules/enforce-module-boundaries.ts | 906 | `b1c6dc778776d7fd617857d6811a4d78d7803105c98a70152c9796831f90a318` |
| gal-syntax | 6 go-arch-lint | https://raw.githubusercontent.com/fe3dback/go-arch-lint/bf473afb033c7c3eb7aba68dc399cf919287131e/docs/syntax/README.md | 31 | `9eaee7a485bf7ad68736f5b3dafa8457d497ce6d6d307cad0887f3e52b4ea91e` |
| gal-readme | 6 go-arch-lint | https://raw.githubusercontent.com/fe3dback/go-arch-lint/master/README.md | 157 | `c6822c2ce35df030aa39eb303b34459c8d3d484d4c7588a013b6e59d2d08c61c` |
| cm | 7 Cargo | https://doc.rust-lang.org/cargo/commands/cargo-metadata.html | 822 | `09bf9297094c6af1ad72cfb679419b02631a49f8c23db0860afe26a67bac4cfa` |
| cd-bans | 7 cargo-deny | https://embarkstudios.github.io/cargo-deny/checks/bans/cfg.html | 569 | `07dfbdf7124f64518e548943a059ce78487e8d3b84fecb3f6bdf28783c41cb4e` |
| cd-cfg | 7 cargo-deny | https://embarkstudios.github.io/cargo-deny/checks/cfg.html | 381 | `f8a0e3ca41c89cf6c0272d3b1a7168cd294402dde01f022c2454995b4d999543` |
| cd-check | 7 cargo-deny | https://embarkstudios.github.io/cargo-deny/cli/check.html | 362 | `70d584ca3ea140c3a06bc75828d93ac75455083818538d38ef5070f4c52cae59` |
| cd-common | 7 cargo-deny | https://embarkstudios.github.io/cargo-deny/cli/common.html | 290 | `000fef1c508366cbf0b3085d3cc833b79f03854502454298c87e871fdd5e0358` |

`cm` と `cd-bans` は `.brainstorming/semantic-contract-core/sources/rust/` から meta.json ごと複製した。

---

## ツールごとの表

| ツール | 規則の単位 | 許す/禁じるの書き方 | 辺の種類の区別 | 循環の検査 | 例外（既知の違反を一時的に許す） | 出力の形 |
|---|---|---|---|---|---|---|
| 1 import-linter | 契約（contract）。`type` で種類を選ぶ（Q23）。種類は `forbidden` ・ `protected` ・ `layers` ・ `independence` ・ `acyclic_siblings`（Q1〜Q5）。中身はモジュール名と `*` ・ `**`（Q9） | 両方。禁止リスト型 `forbidden`（Q15）、許可リスト型 `protected` の `allowed_importers`（Q19）、順序で書く `layers`（Q10, Q11）、相互に禁止する `independence`（Q17） | `exclude_type_checking_imports` で `if TYPE_CHECKING:` の下の import を外せる（Q22）。間接の import も検査する（Q12, Q18）。間接を許す `allow_indirect_imports`（Q16）。実行時 ・ 開発時の区別は原文に無い | 持つ。`acyclic_siblings`（Q5, Q20） | `ignore_imports` に辺を1本ずつ書く（Q6, Q7）。一致しない式は既定で `error`（Q8）。違反をまとめて記録する仕組み（基準線）は原文に無い | テキストの例のみ（Q26, Q21）。終了コードと JSON は原文に無い |
| 2 dependency-cruiser | パスの正規表現 `path` ・ `pathNot`（Q34, Q35）を、辺の両端 `from` ・ `to`（Q33）に書く。`scope` で `module` か `folder`（Q44） | 3種。`forbidden`（Q30）・ `allowed`（Q31）・ `required`（Q32） | 持つ。`dependencyTypes` ・ `dependencyTypesNot`（Q45）。値の例 `npm-dev`（Q41）・ `type-only`（Q42）・ `pre-compilation-only`（Q43） | 持つ。`circular`（Q39）。通る経路の条件 `via` ・ `viaOnly`（Q40） | `--baseline` で `.dependency-cruiser-known-violations.json` を作る（Q48）。`--ignore-known` はそこにある違反を `ignore` に下げる（Q49）。`--baseline-mode` の `shrink-only`（Q50）。規則ごとの `severity` に `ignore`（Q38） | `err` は `error` の違反の数を終了コードにする（Q46）。`json`（Q47）。`severity` は `error` ・ `warn` ・ `info` ・ `ignore`（Q36, Q37） |
| 3 ArchUnit | 層＝パッケージ識別子の集合（`.layer("Service").definedBy("..service..")` Q61）。ほかにスライス（Q63, Q69）と onion の `domainModels` ・ `adapter` 等（Q64, Q65） | 両方。許可リスト型 `mayOnlyBeAccessedByLayers`（Q62, Q71）・ `mayOnlyAccessLayers`（Q73）。禁止型 `mayNotBeAccessedByAnyLayer`（Q60, Q70）・ `mayNotAccessAnyLayer`（Q72） | importer は member types ・ accesses ・ supertypes 等を区別する（Q78, Q79）が、文書が書くのは解決の反復回数の設定である（Q80）。規則の側で辺の種類を選ぶ書き方は原文に無い。どの辺を数えるかは `consideringOnlyDependenciesInLayers` 等で選ぶ（Q75） | 持つ。`slices()...beFreeOfCycles()`（Q63） | `ignoreDependency`（Q74）。`FreezingArchRule` が既存の違反を `ViolationStore` に記録し、新しい違反だけを出す（Q66, Q67）。直った違反は記録から自動で減る（Q68） | `ArchRule` を `check` で評価する（Q82）。失敗の表示形式は差し替えられる（Q81）。JSON と終了コードは原文に無い。テストの失敗として出ることは推測 |
| 4 deptrac | 層（layer）＝名前と collector の一覧（Q90）。collector がノードを層に振り分ける（Q101） | 許可リスト型。既定で層の間の依存はすべて禁止（Q91）、`ruleset` で許す（Q92）。`+` を付けると相手が許す層も許す（Q97）。層の外から参照させない `@deptrac-internal`（Q94）と collector の `private`（Q102） | `analyser.types` で解析する構文を選ぶ（Q99）。実行時 ・ 開発時 ・ 型だけの区別は原文に無い | 原文に無い | `skip_violations`（Q93）。これは失敗の終了コードにしない（Q98）。一致しない項目は ERROR と表示（Q96）。baseline formatter が `skip_violations` を生成する（Q103） | `--formatter=json`（Q104）。`--fail-on-uncovered`（Q95）。終了コードの値は原文に無い |
| 5 Nx | プロジェクトに付けた `tags`（Q110）。規則は `depConstraints` の `sourceTag` ・ `allSourceTags`。タグは完全一致 ・ 正規表現 ・ glob（Q112） | 両方。`onlyDependOnLibsWithTags`（Q114）・ `notDependOnLibsWithTags`（Q115, Q124）。合う制約はすべて AND（Q116）。タグの無いプロジェクトは依存できない（Q111） | 静的 import と遅延読み込みを区別する（Q122, Q120）。`package.json` に無い推移的な依存を禁じる `banTransitiveDependencies`（Q119）。開発時 ・ 型だけの区別は原文に無い | 持つ。`noCircularDependencies`（Q121）。除外は `ignoredCircularDependencies`（Q118） | `allow`（Q117, Q123）。違反をまとめて記録する仕組みは原文に無い | ESLint のルールとして動く（Q125）。終了コードと JSON は Nx の原文に無い（ESLint 側が持つことは推測）。言語に寄らない版は `nx conformance:check`（Q113） |
| 6 go-arch-lint | component＝Go パッケージの集まり（Q130）。`in` に glob（Q134）。外部は `vendors` | 許可リスト型。`mayDependOn`（Q131）・ `canUse`（Q132）。どこからでも使える `commonComponents`（Q135）、何にでも依存できる `anyProjectDeps`（Q133） | プロジェクト内の component と vendor を分ける（Q131, Q132）。実行時 ・ 開発時 ・ 型だけの区別は原文に無い | 原文に無い | 違反を設定に足して `todo` のラベルを付ける手順（Q139）。専用のキーは原文に無い | 終了コード 0（Q136）・ 1（Q137）。`--json`（Q138）、`--output-type` は `ascii` ・ `json`（Q141） |
| 7 Cargo（`cargo metadata`） | パッケージ。`resolve.nodes`（Q153）、`workspace_members`（Q155） | 原文に無い（規則を持たず、グラフを出す） | 持つ。`kind` は `"dev"` ・ `"build"` ・ null（Q150）、`dep_kinds`（Q151）、プラットフォームの `target`（Q152）。target の種類 `test` 等（Q156） | 原文に無い | 原文に無い | JSON（Q158）。`--format-version` を付けるよう勧める（Q157）。`--no-deps` で `resolve` が null（Q154） |
| 7 cargo-deny（`[bans]`） | crate（PackageSpec）（Q160） | 両方。`deny`（Q160）、`allow` は1件でも書くと許可リスト型になる（Q162）。`wrappers` は直接の依存だけ許し推移的な依存を禁じる（Q161） | `exclude-dev` で dev-dependencies を外す（Q166） | 原文に無い | `skip` は重複検出から一時的に外す（Q163）。一致しなくなった `skip` は警告（Q169）。`reason` を診断に出す（Q165） | 終了コードは検査ごとのビット集合（Q167）。`--format` に json（Q168） |

---

## ツールをまたいで共通する概念

| 概念 | 持つツール（根拠） |
|---|---|
| 辺の両端を「元」と「先」に分けて規則を書く | import-linter `source_modules` ・ `forbidden_modules`（Q15）／ dependency-cruiser `from` ・ `to`（Q33）／ ArchUnit `mayOnlyBeAccessedByLayers` と `mayOnlyAccessLayers`（Q71, Q73）／ deptrac `ruleset`（Q92）／ Nx `sourceTag` と `onlyDependOnLibsWithTags`（Q114）／ go-arch-lint `deps` の `mayDependOn`（Q131） |
| 要素を名前の付いた集まりにまとめ、規則はその集まりの間に書く | 層（import-linter Q11 ・ ArchUnit Q61 ・ deptrac Q90）、タグ（Nx Q110）、component（go-arch-lint Q130）。dependency-cruiser は集まりを持たず、正規表現を規則に直接書く（Q34） |
| 集まりの中身をパターンで決める | ワイルドカード（Q9）、正規表現（Q34, Q100, Q112）、glob（Q112, Q134）、パッケージ識別子（Q61） |
| 許可リスト型 | import-linter `protected`（Q19）／ dependency-cruiser `allowed`（Q31）／ ArchUnit `mayOnly...`（Q71, Q73）／ deptrac 既定禁止＋`ruleset`（Q91, Q92）／ Nx `onlyDependOnLibsWithTags`（Q114）／ go-arch-lint `mayDependOn`（Q131）／ cargo-deny `allow`（Q162） |
| 禁止リスト型 | import-linter `forbidden`（Q15）／ dependency-cruiser `forbidden`（Q30）／ ArchUnit `mayNot...`（Q70, Q72）／ Nx `notDependOnLibsWithTags`（Q115）／ cargo-deny `deny`（Q160） |
| 辺を1本ずつ名指しで除外する | import-linter `ignore_imports`（Q6）／ ArchUnit `ignoreDependency`（Q74）／ deptrac `skip_violations`（Q93）／ Nx `allow`（Q117） |
| 既存の違反を記録して、新しい違反だけを失敗にする | dependency-cruiser `--baseline` ・ `--ignore-known`（Q48, Q49）／ ArchUnit `FreezingArchRule`（Q66, Q67）／ deptrac baseline formatter（Q103） |
| 一致しなくなった例外を知らせる | import-linter `unmatched_ignore_imports_alerting`（Q8）／ deptrac（Q96）／ cargo-deny `skip`（Q169）／ ArchUnit は自動で減らす（Q68） |
| 循環の検査 | import-linter（Q20）／ dependency-cruiser（Q39）／ ArchUnit（Q63）／ Nx（Q121）。deptrac ・ go-arch-lint ・ Cargo ・ cargo-deny は原文に無い |
| 辺の種類で規則を絞る | dependency-cruiser `dependencyTypes`（Q45）／ import-linter `exclude_type_checking_imports`（Q22）／ cargo-deny `exclude-dev`（Q166）／ Nx 静的 import と遅延読み込み（Q122）。Cargo は種類を出力に持つ（Q150） |
| どの集まりにも属さない要素を検出する | import-linter `exhaustive`（Q13）／ ArchUnit `ensureAllClassesAreContainedInArchitecture`（Q77）／ deptrac `--fail-on-uncovered`（Q95）／ Nx タグの無いプロジェクト（Q111） |
| 違反の有無を終了コードで返す | dependency-cruiser（Q46）／ deptrac（Q98）／ go-arch-lint（Q136, Q137）／ cargo-deny（Q167） |
| 機械が読む出力（JSON） | dependency-cruiser（Q47）／ deptrac（Q104）／ go-arch-lint（Q138）／ Cargo（Q158）／ cargo-deny（Q168） |

## ツールごとにしか無い概念

| ツール | 概念（根拠） |
|---|---|
| import-linter | 同じ層の中の兄弟を `\|` で互いに独立、`:` で依存可にする（Q14）／ 兄弟の間の循環を世代ごとに掘る `acyclic_siblings` の `depth`（Q20） |
| dependency-cruiser | 依存を「持たねばならない」と書く `required`（Q32）／ 規則を `folder` に適用する `scope`（Q44）／ 循環の経路の条件 `viaOnly`（Q40）／ `orphans` ・ `reachable` ・ `moreUnstable` の条件（識別子を照合済み） |
| ArchUnit | onion の規則を API で持つ（Q64, Q65）／ 層を空にしてはならない既定と `optionalLayer` ・ `withOptionalLayers`（Q76、識別子を照合済み）／ 違反の記録 `ViolationStore` が直った分を自動で減らす（Q68）／ `adhereToPlantUmlDiagram`（識別子を照合済み） |
| deptrac | 1つの要素を複数の層に入れられる collector（Q90, Q101）／ `+` による推移的な許可（Q97）／ 層の内部にだけ見せる `@deptrac-internal` と collector の `private`（Q94, Q102） |
| Nx | 複数タグの AND を `allSourceTags` で書く（Q116）／ 遅延読み込みのライブラリの静的 import を禁じる（Q122）／ 外部パッケージの許可 ・ 禁止 `allowedExternalImports` ・ `bannedExternalImports`（識別子を照合済み） |
| go-arch-lint | 自分のコードの component と外部の vendor で許可のキーを分ける（`mayDependOn` ・ `canUse`、Q131, Q132） |
| Cargo ・ cargo-deny | 辺がプラットフォームの条件 `target` を持つ（Q152）／ 直接の依存だけを許す `wrappers`（Q161）／ 終了コードが検査ごとのビット集合（Q167） |

---

## 照合の結果

| 種類 | 照合した数 | 原文に在った | 読めた範囲に無い | 読めなかった範囲 |
|---|---|---|---|---|
| quote（引用） | 132 | 132 | 0 | 無し（全部読めた） |
| identifier（識別子） | 166 | 166 | 0 | 無し（全部読めた） |

- 引用は1件ずつ `fact-check verify <原文> --as quote --from <1行のファイル>` で照合した。行番号はその出力の位置である
- 識別子は原文ごとに `--as identifier --from` で照合した（例：`mayOnlyAccessLayers` ・ `dep_kinds` ・ `notDependOnLibsWithTags` ・ `skip_violations` ・ `allow_indirect_imports`）
- 途中で1件（`--json                   (alias for --output-type=json)` を空白1つにして渡した引用）が一致しなかった。原文の空白を詰めずに `(alias for --output-type=json)` へ直して照合し直した
- 「原文に無い」は、次の検索が0件だったことに基づく ── import-linter の終了コード ・ JSON（il-* と docs の api.md ・ ui.md ・ index.md を `exit|json|return code` で検索）／ deptrac ・ go-arch-lint ・ Cargo ・ cargo-deny の循環（`cycl` で検索）／ ArchUnit の JSON ・ 終了コード／ Nx の終了コード
- 「推測」は2か所 ── ArchUnit の違反がテストの失敗として出ること／ Nx の終了コードと JSON を ESLint 側が持つこと

---

## 引用の一覧

行番号は、上の「取得した原文」の略号のファイルの行である。

| 番号 | 出どころ | 原文 | 照合 |
|---|---|---|---|
| Q1 | il-index:10 | Prevent one set of modules being imported by another. | ok |
| Q2 | il-index:20 | Prevent modules from being directly imported, except by modules in an allow-list. | ok |
| Q3 | il-index:30 | Enforce a 'layered architecture'. | ok |
| Q4 | il-index:40 | Prevent a set of modules depending on each other. | ok |
| Q5 | il-index:50 | Forbid dependency cycles between siblings. | ok |
| Q6 | il-index:62 | `ignore_imports`: Optional list of imports, each in the form `mypackage.foo.importer -> mypackage.bar.imported`. | ok |
| Q7 | il-index:63 | These imports will be ignored: if the import would cause a contract to be broken, adding it to the list will cause the contract be kept instead. | ok |
| Q8 | il-index:69 | `error`: Error if there are any unmatched expressions (default). | ok |
| Q9 | il-index:106 | `*` stands in for a module name, without including subpackages. `**` includes subpackages too. | ok |
| Q10 | il-layers:5 | Layers contracts enforce a 'layered architecture', where higher layers may depend on lower layers, but not the other way around. | ok |
| Q11 | il-layers:11 | An ordered list with the name of each layer module. | ok |
| Q12 | il-layers:35 | This includes indirect imports (i.e. chains of imports via other modules) | ok |
| Q13 | il-layers:21 | If true, check that the contract declares every possible layer in its list of layers to check. | ok |
| Q14 | il-layers:282 | To allow siblings to depend on each other, use colons instead of pipes to separate them: | ok |
| Q15 | il-forbidden:5 | Forbidden contracts check that one set of modules is not imported by another set of modules. | ok |
| Q16 | il-forbidden:173 | `allow_indirect_imports`: If `True`, allow indirect imports to forbidden modules without interpreting them as a reason to mark the contract broken. | ok |
| Q17 | il-independence:5 | Independence contracts check that a set of modules do not depend on each other. | ok |
| Q18 | il-independence:7 | They do this by checking that there are no imports in any direction between the modules, even indirectly. | ok |
| Q19 | il-protected:5 | Protected contracts prevent modules from being directly imported, except by modules in an allow-list. | ok |
| Q20 | il-acyclic:5 | An *acyclic siblings* contract forbids dependency cycles between siblings. | ok |
| Q21 | il-acyclic:104 | It could be made acyclic by removing 1 dependency: | ok |
| Q22 | il-configure:75 | Whether to exclude imports made in type checking guards. If this is `True`, any import made under an `if TYPE_CHECKING:` statement will not be added to the graph. | ok |
| Q23 | il-configure:113 | `type`: The type of contract to use | ok |
| Q24 | il-configure:70 | Whether to include external packages when building the import graph. | ok |
| Q25 | il-run:3 | To check that your project adheres to the contracts you've defined, run: | ok |
| Q26 | il-acyclic:100 | Broken acyclic_siblings contract | ok |
| Q30 | dc-rules:70 | A list of rules that describe dependencies that are not allowed. | ok |
| Q31 | dc-rules:76 | A list of rules that describe dependencies that are _allowed_. dependency-cruiser will emit a 'not-in-allowed' message for each dependency that does not satisfy at least one of them. | ok |
| Q32 | dc-rules:89 | A list of rules that describe what dependencies modules _must_ have, like 'every controller _must depend on_ the base controller'. | ok |
| Q33 | dc-rules:229 | A rule consists at least of a `from` and a `to` attribute | ok |
| Q34 | dc-rules:326 | A regular expression an end of a dependency should match to be caught by this rule. | ok |
| Q35 | dc-rules:339 | A regular expression an end of a dependency should NOT match to be caught by this rule. | ok |
| Q36 | dc-rules:283 | The 'error' severity will make some reporters (at least the `err` one) return a non-zero exit code | ok |
| Q37 | dc-rules:288 | The other values you can use are `info`, `warn` and `ignore`. | ok |
| Q38 | dc-rules:291 | With the severity set to `ignore` dependency-cruiser will not check the rule at all. | ok |
| Q39 | dc-rules:725 | A Boolean indicating whether or not to match module dependencies that end up where you started (a.k.a. circular dependencies). | ok |
| Q40 | dc-rules:748 | `viaOnly`: matches against _all_ of the modules in the cycle | ok |
| Q41 | dc-rules:1026 | it's a module in package.json's `devDependencies` | ok |
| Q42 | dc-rules:1070 | as 'type only' - only available for TypeScript sources, only for tsPreCompilationDeps !== false. | ok |
| Q43 | dc-rules:1062 | but the dependency will disappear at runtime. | ok |
| Q44 | dc-rules:301 | What to apply the rule to - either `module` (the default) or `folder`. | ok |
| Q45 | dc-rules:1014 | If you don't specify dependencyTypes (or dependencyTypesNot) in a rule, dependency-cruiser will ignore them in the evaluation of that rule. | ok |
| Q46 | dc-cli:117 | will exit with exit code _number of violations with severity `error` found_ | ok |
| Q47 | dc-cli:499 | This emits the internal representation of a cruise as json. | ok |
| Q48 | dc-cli:840 | By default known violations go into `.dependency-cruiser-known-violations.json` | ok |
| Q49 | dc-cli:890 | For all violations dependency-cruiser finds in the known violations file it will lower the severity to `ignore`. | ok |
| Q50 | dc-cli:870 | "shrink-only": makes --baseline only _remove_ existing violations, and not add any new ones. | ok |
| Q60 | au-004:287 | .whereLayer("Controller").mayNotBeAccessedByAnyLayer() | ok |
| Q61 | au-004:284 | .layer("Service").definedBy("..service..") | ok |
| Q62 | au-004:288 | .whereLayer("Service").mayOnlyBeAccessedByLayers("Controller") | ok |
| Q63 | au-004:333 | slices().matching("com.myapp.(*)..").should().beFreeOfCycles() | ok |
| Q64 | au-008:17 | At the moment this only provides a convenient check for a layered architecture and onion architecture. | ok |
| Q65 | au-008:53 | No adapter may depend on another adapter. | ok |
| Q66 | au-008:442 | `FreezingArchRule` can help in these scenarios by recording all existing violations to a `ViolationStore`. | ok |
| Q67 | au-008:443 | Consecutive runs will then only report new violations and ignore known violations. | ok |
| Q68 | au-008:444 | If violations are fixed, `FreezingArchRule` will automatically reduce the known stored violations to prevent any regression. | ok |
| Q69 | au-008:149 | SlicesRuleDefinition.slices().matching("..myapp.(**).service..").should().notDependOnEachOther() | ok |
| Q70 | au-src:598 | Forbids any {@link Dependency dependency} from another layer to this layer. | ok |
| Q71 | au-src:607 | Restricts this layer to only allow the specified layers to have {@link Dependency dependencies} to this layer. | ok |
| Q72 | au-src:617 | Forbids any {@link Dependency dependency} from this layer to any other layer. | ok |
| Q73 | au-src:626 | Restricts this layer to only allow {@link Dependency dependencies} to the specified layers. | ok |
| Q74 | au-src:380 | Configures the rule to ignore any violation from a specific {@code origin} class to a specific {@code target} class. | ok |
| Q75 | au-src:713 | All dependencies that either have an origin or a target outside any defined layer will be ignored. | ok |
| Q76 | au-src:155 | By default, layers defined with {@link #layer(String)} must not be empty, i.e. contain at least one class. | ok |
| Q77 | au-src:254 | Ensure that all classes under test are contained within a defined layer of the architecture. | ok |
| Q78 | au-010:86 | In particular, the ArchUnit importer distinguishes the following types of import dependencies: | ok |
| Q79 | au-010:90 | supertypes (i.e. superclasses that are extended and interfaces that are implemented) | ok |
| Q80 | au-010:96 | For each of these dependency types it is possible to configure the maximum number of iterations to traverse the dependencies. | ok |
| Q81 | au-010:189 | You can configure a custom format to display the failures of a rule. | ok |
| Q82 | au-003:54 | The returned object of type `ArchRule` can now be evaluated against a set of imported classes: | ok |
| Q90 | dt-concepts:17 | Each layer has a unique name and a list of one or more collectors | ok |
| Q91 | dt-concepts:39 | **By default, all dependencies between layers are forbidden!** | ok |
| Q92 | dt-concepts:119 | Allowed dependencies between layers are configured in a *ruleset*. | ok |
| Q93 | dt-concepts:296 | Violations can be skipped by provided list of dependencies in *skip_violations* | ok |
| Q94 | dt-concepts:237 | the marked class cannot be referenced by any class outside its layer. | ok |
| Q95 | dt-concepts:229 | You can use the `--fail-on-uncovered` option to let Deptrac fail when any uncovered dependencies are encountered. | ok |
| Q96 | dt-concepts:319 | Skipped violation "Core\Unmatched" for "Library\LibClass" was not matched. | ok |
| Q97 | dt-config:253 | If you prepend a layer with <code>+</code> then not only this layer is allowed, but also all layers it allows. | ok |
| Q98 | dt-config:276 | This violations will be ignored in your pipeline and not trigger a failing return code. | ok |
| Q99 | dt-config:49 | analyses file definitions for use statements. | ok |
| Q100 | dt-config:122 | A list of regular expression-patterns to determine which files or directories to exclude, | ok |
| Q101 | dt-collectors:3 | Collectors decide if a node (typically a class) is part of a layer. | ok |
| Q102 | dt-collectors:382 | This means that tokens collected by this specific collector can be referenced only by other tokens in the same layer. | ok |
| Q103 | dt-formatters:13 | The Baseline formatter is a console formatter, which generates the `skip_violations` section to the given File. | ok |
| Q104 | dt-formatters:296 | By default, Json formatter dumps information to *STDOUT*. | ok |
| Q110 | nx-features:36 | use your project configuration (in `project.json` or `package.json`) to annotate your projects with `tags`. | ok |
| Q111 | nx-features:302 | Projects without any tags cannot depend on any other projects. | ok |
| Q112 | nx-features:328 | `regex`: allow tags matching the regular expression | ok |
| Q113 | nx-features:24 | This rule checks dependencies in the Nx graph during `nx conformance:check`. | ok |
| Q114 | nx-kb:115 | The source **can depend only** on projects that contain at least one of these tags | ok |
| Q115 | nx-kb:116 | The source **can not depend** on projects that contain at least one of these tags | ok |
| Q116 | nx-kb:108 | The constraints are applied with **AND** logical operation | ok |
| Q117 | nx-kb:96 | List of imports that should be allowed without any checks | ok |
| Q118 | nx-kb:99 | List of project pairs that should be skipped from `Circular dependencies` checks | ok |
| Q119 | nx-kb:98 | Ban import of dependencies that were not specified in the root or project's `package.json` | ok |
| Q120 | nx-kb:100 | List of imports that should be skipped for `Imports of lazy-loaded libraries forbidden` checks. | ok |
| Q121 | nx-ts:187 | noCircularDependencies: `Circular dependency between | ok |
| Q122 | nx-ts:193 | Static imports of lazy-loaded libraries are forbidden. | ok |
| Q123 | nx-allow:9 | Each project can set an `allow` property in the project configuration to override the tagging rules that have been set up. | ok |
| Q124 | nx-ban:28 | "notDependOnLibsWithTags": ["scope:admin"], | ok |
| Q125 | nx-kb:11 | The `@nx/enforce-module-boundaries` lint rule enables you to define strict rules for accessing resources between | ok |
| Q130 | gal-syntax:13 | component is abstraction on go packages. One component = one or more go packages | ok |
| Q131 | gal-syntax:25 | list of components that can by imported in %name% | ok |
| Q132 | gal-syntax:26 | list of vendors that can by imported in %name% | ok |
| Q133 | gal-syntax:24 | all component code can import any other project code, useful for DI/main component | ok |
| Q134 | gal-syntax:15 | one or more relative directory name, support glob masking | ok |
| Q135 | gal-syntax:19 | list of components, allow import them into any code | ok |
| Q136 | gal-readme:129 | Project has correct architecture | ok |
| Q137 | gal-readme:130 | Found warnings | ok |
| Q138 | gal-readme:119 | (alias for --output-type=json) | ok |
| Q139 | gal-readme:102 | Linter find some issues in the project. Don’t fix them right now, but “legalize” them by adding them to the config and marking `todo` with the label | ok |
| Q140 | gal-readme:141 | compares the actual (code) and desired (config) dependency graph | ok |
| Q150 | cm:274,471 | "dev", "build", or null for a normal dependency. | ok |
| Q151 | cm:467 | Array of dependency kinds. Added in Cargo 1.40. | ok |
| Q152 | cm:287,474 | The target platform for the dependency. | ok |
| Q153 | cm:443 | Array of nodes within the dependency graph. | ok |
| Q154 | cm:437 | This is null if --no-deps is specified. | ok |
| Q155 | cm:421 | Array of members of the workspace. | ok |
| Q156 | cm:316 | integration test is ["test"] | ok |
| Q157 | cm:196 | It is recommended to include the <code>--format-version</code> flag to future-proof your code | ok |
| Q160 | cd-bans:292 | Determines specific crates that are denied. | ok |
| Q161 | cd-bans:296 | This field allows specific crates to have a direct dependency on the banned crate but denies all transitive dependencies on it. | ok |
| Q162 | cd-bans:313 | If the <code>allow</code> list has one or more entries, then any crate not in that list will be denied, so use with care. | ok |
| Q163 | cd-bans:369 | allows you to temporarily ignore a crate during duplicate detection so that no errors are emitted, until it is no longer need. | ok |
| Q164 | cd-bans:235 | Determines what happens when multiple versions of the same crate are encountered. | ok |
| Q165 | cd-bans:305,329 | that is surfaced in diagnostic output | ok |
| Q166 | cd-cfg:276 | even one for workspace crates, are not included in the crate graph used for any of the checks. | ok |
| Q167 | cd-check:302 | the exit code for the check command is a bitset of the checks that were executed that had 1 or more errors. | ok |
| Q168 | cd-common:210 | Each log message/diagnostic is outputted as a single line JSON object | ok |
| Q141 | gal-readme:122 | type of command output, variants: [ascii, json] | ok |
| Q158 | cm:218 | The JSON output has the following format: | ok |
| Q169 | cd-bans:370 | cargo-deny will emit warnings when any entry in the <code>skip</code> list no longer matches a crate in your graph | ok |
